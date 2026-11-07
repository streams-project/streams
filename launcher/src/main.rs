//! The one command. Its name comes from `brand.json`, so this file never states it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use launcher::address::{self, Site};
use launcher::brand::Brand;
use launcher::films;
use launcher::postgres::{self, Postgres};
use launcher::supervise::{Program, answers, keep, log_path};
use rand::Rng;
use serde_json::{Value, json};

/// Everything found once at the start.
struct Stack {
    root: PathBuf,
    brand: Brand,
    home: PathBuf,
    programs: PathBuf,
}

const DEFAULT_PORT: u16 = 8080;
const READY_WITHIN: Duration = Duration::from_secs(60);

fn hex_secret() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

/// This repository, from where the program sits: `bin/` or `target/<profile>/`.
fn find_root() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let mut folder = exe.parent().map(Path::to_path_buf);
    while let Some(here) = folder {
        if here.join("brand.json").exists() {
            return Ok(here);
        }
        folder = here.parent().map(Path::to_path_buf);
    }
    let here = std::env::current_dir()?;
    if here.join("brand.json").exists() {
        return Ok(here);
    }
    bail!("run this from a copy of the repository: brand.json was not found")
}

fn open_stack() -> Result<Stack> {
    let root = find_root()?;
    let brand = Brand::read(&root.join("brand.json"))?;
    let home = match std::env::var_os("STACK_HOME") {
        Some(home) => PathBuf::from(home),
        None => {
            let user = std::env::var_os("HOME").context("HOME is not set")?;
            Path::new(&user).join(format!(".{}", brand.command))
        }
    };
    let exe = std::env::current_exe()?;
    let programs = exe
        .parent()
        .context("the program has no folder")?
        .to_path_buf();
    Ok(Stack {
        root,
        brand,
        home,
        programs,
    })
}

/// The private values of this install, made once.
fn secrets(stack: &Stack) -> Result<Value> {
    let path = stack.home.join("secrets.json");
    if let Ok(text) = std::fs::read_to_string(&path) {
        return Ok(serde_json::from_str(&text)?);
    }
    let made = json!({"finder_token": hex_secret()});
    postgres::write_private(&path, &made.to_string())?;
    Ok(made)
}

fn copy_folder(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_folder(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// The opening and the showreel hold title art, so they are kept out of the repository. A brand
/// puts its own in `media/`, and the page runs without them.
fn copy_tv_media(assets: &Path, tv: &Path) -> Result<()> {
    for name in ["opening.mp4", "showreel.mp4", "peek.jpg"] {
        let from = assets.join("media").join(name);
        if from.exists() {
            std::fs::copy(from, tv.join(name))?;
        }
    }
    Ok(())
}

/// The core for a television: next to this program in `bin/`, or where cargo left it.
fn core_wasm(stack: &Stack) -> Option<PathBuf> {
    [
        stack.programs.join("core.wasm"),
        stack
            .programs
            .join("../wasm32v1-none/wasm/bridge_wasm.wasm"),
    ]
    .into_iter()
    .find(|path| path.exists())
}

/// The built site, with this brand's look and mark laid over it.
fn publish_site(stack: &Stack) -> Result<PathBuf> {
    let built = stack.root.join("apps/web/dist");
    if !built.join("index.html").exists() {
        bail!("the website is not built, run ./up from the repository root");
    }
    let site = stack.home.join("site");
    let _ = std::fs::remove_dir_all(&site);
    copy_folder(&built, &site)?;
    std::fs::write(site.join("brand.css"), stack.brand.css())?;
    copy_folder(&stack.root.join("apps/tv/page"), &site.join("tv"))?;
    // Without it the website still runs, and the TV page says what is missing.
    match core_wasm(stack) {
        // Written fresh, because a copy can keep the build's time. A browser that once got the
        // website's page at this address asks if it changed since that page's date, and a file
        // dated before it gets a 304, which keeps the page in place of the core for good.
        Some(core) => std::fs::write(site.join("tv/core.wasm"), std::fs::read(core)?)?,
        None => println!("The TV core is not built, run ./up from the repository root."),
    }
    let assets = stack.root.join(&stack.brand.assets);
    copy_tv_media(&assets, &site.join("tv"))?;
    // A brand's profile pictures. Optional: with none, profiles show a letter.
    let avatars = assets.join("avatars");
    if avatars.is_dir() {
        copy_folder(&avatars, &site.join("avatars"))?;
    }
    for (from, to) in [
        ("mark.svg", "mark.svg"),
        ("lockup-on-dark.svg", "lockup.svg"),
    ] {
        std::fs::copy(assets.join(from), site.join(to))
            .with_context(|| format!("{from} is missing from {}", assets.display()))?;
    }
    Ok(site)
}

/// The data set in use: this install's own, or a generated sample until one is built.
fn data_set(stack: &Stack) -> Result<(PathBuf, PathBuf)> {
    let data = stack.home.join("data");
    let (manifest, key) = (data.join("manifest.json"), data.join("data_key.pem"));
    if manifest.exists() && key.exists() {
        return Ok((manifest, key));
    }
    println!("No catalogue yet, so the site shows a generated sample with invented titles.");
    films::seed(&stack.home.join("films.json"))?;
    Ok((
        server_catalog::sample::write_sample(&data),
        server_catalog::sample::write_sample_key(&data),
    ))
}

fn base_env(pairs: &[(&str, String)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

fn migrate(stack: &Stack, database: &Postgres) -> Result<()> {
    let output = Command::new(stack.programs.join("server"))
        .arg("migrate")
        .env_clear()
        .env("DATABASE_URL", &database.url)
        .output()
        .context("the server program did not run")?;
    if !output.status.success() {
        bail!(
            "the database update failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Source packs live in `packs/<name>/`, each its own program.
fn packs(stack: &Stack) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(stack.root.join("packs")) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let program = entry.path().join("target/release").join(&name);
            program.exists().then_some((name, program))
        })
        .collect()
}

fn finders(stack: &Stack, token: &str) -> Result<(Vec<Program>, Vec<String>)> {
    let mut list = vec![(
        String::from("open-films"),
        stack.programs.join("finder-open-films"),
    )];
    list.extend(packs(stack));
    let films = stack.home.join("films.json");
    if !films.exists() {
        std::fs::write(&films, "[]")?;
    }
    let mut programs = Vec::new();
    let mut urls = Vec::new();
    for (name, path) in list {
        let port = free_port()?;
        urls.push(format!("http://127.0.0.1:{port}"));
        programs.push(Program {
            log: log_path(&stack.home, &name),
            name,
            path,
            env: base_env(&[
                ("FINDER_BIND", format!("127.0.0.1:{port}")),
                ("FINDER_SERVICE_TOKEN", token.to_owned()),
                ("FINDER_FILMS", films.to_string_lossy().into_owned()),
            ]),
        });
    }
    Ok((programs, urls))
}

/// Ctrl-C, or the stop a service manager sends.
async fn stop_asked() -> Result<()> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut terminate = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
    Ok(())
}

struct UpOptions {
    port: u16,
    public: bool,
    site_address: Option<String>,
}

fn up_options(words: &[String]) -> Result<UpOptions> {
    let mut options = UpOptions {
        port: DEFAULT_PORT,
        public: false,
        site_address: None,
    };
    let mut words = words.iter();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--public" => options.public = true,
            "--port" => {
                let value = words.next().context("--port needs a number")?;
                options.port = value.parse().context("--port needs a number")?;
            }
            "--site-address" => {
                let value = words.next().context("--site-address needs an address")?;
                options.site_address = Some(address::checked(value)?);
            }
            other => bail!("unknown option {other}"),
        }
    }
    Ok(options)
}

async fn up(stack: &Stack, options: &UpOptions) -> Result<()> {
    std::fs::create_dir_all(&stack.home)?;
    let token = secrets(stack)?["finder_token"]
        .as_str()
        .context("the secrets file has no finder token")?
        .to_owned();
    let site = publish_site(stack)?;
    let (manifest, key) = data_set(stack)?;
    println!("Preparing the database...");
    let bin = postgres::install(&stack.home.join("postgres")).await?;
    let database = Postgres::start(&bin, &stack.home.join("database/data"), free_port()?)?;
    migrate(stack, &database)?;

    let (programs, finder_urls) = finders(stack, &token)?;
    let host = if options.public {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };
    let local = format!("http://127.0.0.1:{}", options.port);
    let reach = Site {
        chosen: options.site_address.as_deref(),
        domain: &stack.brand.domain,
        public: options.public,
        port: options.port,
        lan: address::lan_ip(),
    };
    let site_address = reach.address();
    let server = Program {
        name: String::from("server"),
        path: stack.programs.join("server"),
        log: log_path(&stack.home, "server"),
        env: base_env(&[
            ("SERVER_LISTEN", format!("{host}:{}", options.port)),
            ("DATABASE_URL", database.url.clone()),
            ("DATA_MANIFEST", manifest.to_string_lossy().into_owned()),
            ("DATA_PUBLIC_KEYS", key.to_string_lossy().into_owned()),
            ("FINDER_URLS", finder_urls.join(",")),
            ("FINDER_TOKEN", token.clone()),
            ("WEB_ROOT", site.to_string_lossy().into_owned()),
            (
                "STRINGS_DIR",
                stack.root.join("strings").to_string_lossy().into_owned(),
            ),
            (
                "HELP_DIR",
                stack.root.join("help").to_string_lossy().into_owned(),
            ),
            ("SITE_ADDRESS", site_address.clone()),
            ("BRAND_NAME", stack.brand.name.clone()),
            ("SUPPORT_EMAIL", stack.brand.support_email.clone()),
            // Where a built catalogue's posters live. A sample has none, so it stays empty.
            (
                "IMAGE_BASE",
                std::env::var("IMAGE_BASE").unwrap_or_default(),
            ),
        ]),
    };

    let mut running = tokio::task::JoinSet::new();
    for program in programs {
        running.spawn(keep(program));
    }
    running.spawn(keep(server));
    if !answers(&format!("{local}/v1/ready"), READY_WITHIN).await {
        eprintln!(
            "The server is not answering yet, see {}",
            log_path(&stack.home, "server").display()
        );
    }
    println!("{} is running at {local}", stack.brand.name);
    if site_address != local {
        println!("Other devices reach it at {site_address}");
    }
    if reach.stuck_on_this_machine() {
        println!("This machine has no network address, so a phone cannot follow a TV code here.");
        println!("Name one with --site-address.");
    }
    println!("Press Ctrl-C to stop.");
    stop_asked().await?;
    println!("Stopping...");
    running.shutdown().await;
    drop(database);
    Ok(())
}

/// Copy the programs into `bin/`, with this one under the brand's command name.
fn install(stack: &Stack) -> Result<()> {
    let bin = stack.root.join("bin");
    std::fs::create_dir_all(&bin)?;
    let exe = std::env::current_exe()?;
    let target = bin.join(&stack.brand.command);
    if exe != target {
        std::fs::copy(&exe, &target)?;
    }
    for name in ["server", "finder-open-films"] {
        let from = stack.programs.join(name);
        if from != bin.join(name) {
            std::fs::copy(from, bin.join(name))?;
        }
    }
    if let Some(core) = core_wasm(stack).filter(|core| *core != bin.join("core.wasm")) {
        std::fs::copy(core, bin.join("core.wasm"))?;
    }
    println!("{} is ready", target.display());
    Ok(())
}

fn line(ok: bool, text: &str) -> bool {
    println!("{} {text}", if ok { "ok     " } else { "PROBLEM" });
    ok
}

fn is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .is_ok_and(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
}

fn doctor(stack: &Stack) -> bool {
    let mut good = line(true, &format!("brand: {}", stack.brand.name));
    good &= line(
        stack.programs.join("server").exists(),
        "the server program is built",
    );
    good &= line(
        stack.programs.join("finder-open-films").exists(),
        "the finder program is built",
    );
    good &= line(
        stack.root.join("apps/web/dist/index.html").exists(),
        "the website is built",
    );
    good &= line(core_wasm(stack).is_some(), "the TV core is built");
    good &= line(
        !is_root(),
        "not running as root, which the database refuses",
    );
    let data = stack.home.join("data/manifest.json").exists();
    line(
        true,
        if data {
            "catalogue: present"
        } else {
            "catalogue: none yet, a sample is used"
        },
    );
    let packs = packs(stack);
    line(true, &format!("source packs: {}", packs.len()));
    good
}

async fn backup(stack: &Stack, words: &[String]) -> Result<()> {
    let bin = postgres::install(&stack.home.join("postgres")).await?;
    let default = stack.home.join("backups");
    std::fs::create_dir_all(&default)?;
    let file = match words.first() {
        Some(path) => PathBuf::from(path),
        None => default.join(format!("backup-{}.sql", std::process::id())),
    };
    let data = stack.home.join("database/data");
    if !data.join("PG_VERSION").exists() {
        bail!("there is no database yet");
    }
    if data.join("postmaster.pid").exists() {
        bail!("stop the service first, a backup runs while it is stopped");
    }
    let database = Postgres::start(&bin, &data, free_port()?)?;
    let output = Command::new(bin.join("pg_dump"))
        .arg(&database.url)
        .arg("--file")
        .arg(&file)
        .output()?;
    drop(database);
    if !output.status.success() {
        bail!(
            "the backup failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    println!("Saved {}", file.display());
    Ok(())
}

fn build(stack: &Stack, words: &[String]) -> Result<()> {
    match words.first().map(String::as_str) {
        Some("web") => {
            let status = Command::new("trunk")
                .args(["build", "--release"])
                .current_dir(stack.root.join("apps/web"))
                .status()
                .context("trunk did not run. Install it with: cargo install trunk --locked")?;
            if !status.success() {
                bail!("the website did not build");
            }
            publish_site(stack)?;
            Ok(())
        }
        Some(other) => bail!("{other} cannot be built yet. Ready now: web"),
        None => bail!("name what to build. Ready now: web"),
    }
}

fn help(command: &str) {
    println!(
        "{command} up [--port N] [--public] [--site-address URL]   start everything\n\
         {command} doctor                     check this install\n\
         {command} backup [FILE]              save the database, while stopped\n\
         {command} build web                  build the website\n\
         {command} install                    put this program in bin/"
    );
}

async fn run() -> Result<bool> {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let stack = open_stack()?;
    let (word, rest) = words
        .split_first()
        .map_or(("help", &[][..]), |(word, rest)| (word.as_str(), rest));
    match word {
        "up" => up(&stack, &up_options(rest)?).await?,
        "doctor" => return Ok(doctor(&stack)),
        "backup" => backup(&stack, rest).await?,
        "build" => build(&stack, rest)?,
        "install" => install(&stack)?,
        _ => help(&stack.brand.command),
    }
    Ok(true)
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(why) => {
            eprintln!("{why:#}");
            ExitCode::FAILURE
        }
    }
}
