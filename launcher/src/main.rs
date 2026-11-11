//! The one command. Its name comes from `brand.json`, so this file never states it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use launcher::address::{self, Site};
use launcher::brand::Brand;
use launcher::data::{self, Data, Found};
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
/// A finder is a small program with no database, so it answers long before the server does.
const FINDER_READY_WITHIN: Duration = Duration::from_secs(20);

fn hex_secret() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Ports for the children, every one of them chosen before any child starts.
///
/// Each listener is held open until `release`, so two children are never handed the same number:
/// asking the system for a free port and letting go of it each time could answer with the same one
/// twice. The window between letting go and a child binding is still there, which is why a child
/// that cannot bind has to be noticed rather than left restarting into a log file.
///
/// Every number handed out is remembered whether or not its listener is still held, so a port let
/// go of for a child that starts at once cannot be handed out a second time by the system.
#[derive(Debug, Default)]
struct Ports {
    held: Vec<std::net::TcpListener>,
    given: Vec<u16>,
}

impl Ports {
    /// Tries at finding a number this run has not already spoken for.
    const TRIES: u8 = 64;

    /// A port for a child started after `release`, held open until then.
    fn take(&mut self) -> Result<u16> {
        let (listener, port) = self.pick()?;
        self.held.push(listener);
        Ok(port)
    }

    /// A port for a child asked to bind it right now, let go of before this returns.
    ///
    /// The database starts in the middle of the run, so holding its port until `release` told it to
    /// bind a port this program still had open. Only this one number is let go of early, and it
    /// stays spoken for, so the rest of the handover is unchanged.
    fn take_and_free(&mut self) -> Result<u16> {
        let (listener, port) = self.pick()?;
        drop(listener);
        Ok(port)
    }

    /// A fresh listener on a number this run has not handed out before.
    fn pick(&mut self) -> Result<(std::net::TcpListener, u16)> {
        for _ in 0..Self::TRIES {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            let port = listener.local_addr()?.port();
            if !self.given.contains(&port) {
                self.given.push(port);
                return Ok((listener, port));
            }
        }
        bail!("no free port was found for a child to listen on")
    }

    /// Let the children have them.
    fn release(self) {}
}

/// One finder's name and the address it listens on.
type Listening = (String, String);

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

/// The showreel and the peek hold title art, so they are kept out of the repository with the
/// opening's two sounds. A brand puts its own in `media/`, and the apps run without them. The
/// opening itself is drawn by each app.
fn copy_tv_media(assets: &Path, tv: &Path) -> Result<()> {
    for name in [
        "showreel.mp4",
        "peek.jpg",
        "opening-chime.m4a",
        "opening-swell.m4a",
    ] {
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
///
/// The sample is written into a folder of its own with its key under a name of its own, so the
/// operator's `data/` stays theirs and a real set can never be checked against the sample's
/// published key. The sample's film list goes with it, so a real catalogue drops that list
/// instead of inheriting one test film for every id.
fn data_set(stack: &Stack) -> Result<Data> {
    let sample_key = server_catalog::sample::sample_key_pem();
    match data::read(&stack.home, &sample_key)? {
        Found::Operator(own) => {
            // An install that once ran without a catalogue kept the seeded film list, and the
            // finder that reads it answers before the catalogue, so a test film stood in front of
            // every real title with the same id. It is decided here on every start, so an install
            // that already has the file is reached and not only a fresh one.
            if let Some(aside) = films::retire_seed(&own.films)? {
                println!(
                    "The sample film list was moved to {}, so this catalogue answers for itself.",
                    aside.display()
                );
            }
            return Ok(own);
        }
        Found::Nothing => {
            println!(
                "No catalogue yet, so the site shows a generated sample with invented titles."
            );
        }
        // An older version of this program wrote the sample into the operator's own folder, which
        // left the published sample key standing as this install's trust root. That is refused
        // now, and until this branch existed the refusal asked a person to delete two files by
        // hand to undo something this program had done to them.
        Found::Reseeded { manifest, key } => println!(
            "This install was holding the generated sample under the names a real catalogue is \
             read from, so the sample's key would have accepted a set signed by anyone. The \
             manifest was moved to {} and its key to {}, and the sample is written fresh in a \
             folder of its own.",
            manifest.display(),
            key.display()
        ),
    }
    let (folder, key) = data::sample_at(&stack.home);
    let manifest = server_catalog::sample::write_sample(&folder);
    std::fs::write(&key, &sample_key)?;
    let films = folder.join("films.json");
    let titles: Vec<films::Title> = server_catalog::sample::sample_titles()
        .into_iter()
        .map(|title| (title.kind.to_owned(), title.id))
        .collect();
    films::seed(&films, &titles)?;
    Ok(Data {
        manifest,
        key,
        films,
        sample: true,
    })
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
/// The pack that answers for subtitles. Every other pack finds films.
const SUBTITLE_PACK: &str = "subtitles";

/// The subtitle pack if this copy has one, and where it listens.
fn subtitle_pack(
    stack: &Stack,
    token: &str,
    ports: &mut Ports,
) -> Result<Option<(Program, String)>> {
    let path = stack
        .root
        .join("packs")
        .join(SUBTITLE_PACK)
        .join("target/release")
        .join(SUBTITLE_PACK);
    if !path.exists() {
        return Ok(None);
    }
    let port = ports.take()?;
    let program = Program {
        log: log_path(&stack.home, SUBTITLE_PACK),
        name: SUBTITLE_PACK.to_owned(),
        path,
        env: base_env(&[
            ("FINDER_BIND", format!("127.0.0.1:{port}")),
            ("FINDER_SERVICE_TOKEN", token.to_owned()),
            // Programs start with a clean environment. This one keeps its key in the home folder.
            ("HOME", std::env::var("HOME").unwrap_or_default()),
        ]),
    };
    Ok(Some((program, format!("http://127.0.0.1:{port}"))))
}

fn packs(stack: &Stack) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(stack.root.join("packs")) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let program = entry.path().join("target/release").join(&name);
            (program.exists() && name != SUBTITLE_PACK).then_some((name, program))
        })
        .collect()
}

/// Every finder this copy has, with the address each one listens on.
fn finders(
    stack: &Stack,
    token: &str,
    films: &Path,
    ports: &mut Ports,
) -> Result<(Vec<Program>, Vec<Listening>)> {
    let mut list = vec![(
        String::from("open-films"),
        stack.programs.join("finder-open-films"),
    )];
    list.extend(packs(stack));
    if !films.exists() {
        std::fs::write(films, "[]")?;
    }
    let mut programs = Vec::new();
    let mut listening = Vec::new();
    for (name, path) in list {
        let port = ports.take()?;
        listening.push((name.clone(), format!("http://127.0.0.1:{port}")));
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
    Ok((programs, listening))
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
    let data = data_set(stack)?;
    println!("Preparing the database...");
    let bin = postgres::install(&stack.home.join("postgres")).await?;
    let mut ports = Ports::default();
    let (mut programs, listening) = finders(stack, &token, &data.films, &mut ports)?;
    let subtitle_url = match subtitle_pack(stack, &token, &mut ports)? {
        Some((program, url)) => {
            programs.push(program);
            url
        }
        None => String::new(),
    };
    // The database binds its port as it starts, so this one number is let go of here and not
    // at `release` below, where the children that start after this point take theirs.
    let database = Postgres::start(
        &bin,
        &stack.home.join("database/data"),
        ports.take_and_free()?,
    )?;
    migrate(stack, &database)?;
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
            (
                "DATA_MANIFEST",
                data.manifest.to_string_lossy().into_owned(),
            ),
            ("DATA_PUBLIC_KEYS", data.key.to_string_lossy().into_owned()),
            (
                "FINDER_URLS",
                listening
                    .iter()
                    .map(|(_, url)| url.as_str())
                    .collect::<Vec<&str>>()
                    .join(","),
            ),
            ("FINDER_TOKEN", token.clone()),
            ("SUBTITLE_URL", subtitle_url),
            ("WEB_ROOT", site.to_string_lossy().into_owned()),
            (
                "STRINGS_DIR",
                if stack.root.join("strings").exists() {
                    stack.root.join("strings")
                } else {
                    std::path::PathBuf::from("strings")
                }
                .to_string_lossy()
                .into_owned(),
            ),
            (
                "HELP_DIR",
                if stack.root.join("help").exists() {
                    stack.root.join("help")
                } else {
                    std::path::PathBuf::from("help")
                }
                .to_string_lossy()
                .into_owned(),
            ),
            ("SITE_ADDRESS", site_address.clone()),
            ("BRAND_NAME", stack.brand.name.clone()),
            ("SUPPORT_EMAIL", stack.brand.support_email.clone()),
            // Where a built catalogue's posters live. A sample has none, so it stays empty.
            (
                "IMAGE_BASE",
                std::env::var("IMAGE_BASE").unwrap_or_default(),
            ),
            (
                "IMAGE_BASE_ART",
                std::env::var("IMAGE_BASE_ART").unwrap_or_default(),
            ),
            // What a visitor may look at before signing in: U, PG, 12 or 15. Unset, the landing
            // shows its showreel and no title cards.
            (
                "VISITOR_TIER",
                std::env::var("VISITOR_TIER").unwrap_or_default(),
            ),
        ]),
    };

    // The children take the ports from here on, so the holder lets go of them first.
    ports.release();
    let mut running = tokio::task::JoinSet::new();
    for program in programs {
        running.spawn(keep(program));
    }
    running.spawn(keep(server));
    // A finder that never came up used to show as nothing at all: play simply found no sources.
    //
    // All of them at once, and alongside the server's own wait: asked one after another, every
    // finder that never answers pushed the line below back by its whole wait, so a copy with
    // three dead packs took a minute of waits before it said it was running.
    let mut probing = tokio::task::JoinSet::new();
    for (name, url) in listening {
        probing.spawn(async move {
            (
                name,
                answers(&format!("{url}/v1/health"), FINDER_READY_WITHIN).await,
            )
        });
    }
    let server_ready = answers(&format!("{local}/v1/ready"), READY_WITHIN).await;
    while let Some(probed) = probing.join_next().await {
        match probed {
            Ok((_, true)) => {}
            Ok((name, false)) => eprintln!(
                "PROBLEM the {name} finder is not answering, so its sources are missing. See {}",
                log_path(&stack.home, &name).display()
            ),
            Err(_) => eprintln!("PROBLEM a finder could not be asked whether it came up"),
        }
    }
    if !server_ready {
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
    let mut ports = Ports::default();
    let database = Postgres::start(&bin, &data, ports.take_and_free()?)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole of `data_set` over a home that is in the state an older version of this program
    /// left behind. The type's own rule is tested in `data.rs`; this is the call path, which is
    /// the half that decided whether a person had to delete two files by hand.
    #[test]
    fn an_install_holding_the_sample_at_the_operators_path_starts_on_a_fresh_sample() {
        let root = find_root().expect("the repository");
        let home = std::env::temp_dir().join(format!("data-set-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(home.join("data")).expect("a home");
        let stack = Stack {
            brand: Brand::read(&root.join("brand.json")).expect("the brand"),
            programs: root.join("bin"),
            root,
            home: home.clone(),
        };
        // The sample written where the operator's own set is read from, which is what the older
        // version did and what the owner's install had held since 8 November.
        let built = server_catalog::sample::write_sample(&home.join("data"));
        assert_eq!(built, home.join("data/manifest.json"));
        std::fs::write(
            home.join("data/data_key.pem"),
            server_catalog::sample::sample_key_pem(),
        )
        .expect("a key");

        let data = data_set(&stack).expect("the start refused to carry on");

        // It carries on, on the sample, under the sample's own names.
        assert!(data.sample);
        let (folder, key) = data::sample_at(&home);
        assert_eq!(data.manifest, folder.join("manifest.json"));
        assert_eq!(data.key, key);
        assert!(data.key.exists() && data.manifest.exists());
        // And the published key is no longer the trust root of the operator's path.
        assert!(!home.join("data/data_key.pem").exists());
        assert!(home.join("data/data_key.pem.sample").exists());
        assert!(home.join("data/manifest.json.sample").exists());
        // A second start has nothing left to move and still runs.
        assert!(data_set(&stack).expect("a second start").sample);
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    /// The name of the variable that turns the test below into a port binder.
    const BIND_PORT: &str = "LAUNCHER_TEST_BIND_PORT";

    /// A child process that binds the port it was handed, which is what the database is. With the
    /// variable unset this does nothing, so it costs an ordinary run nothing; the test below runs
    /// this same binary again with it set.
    #[test]
    fn bind_the_port_this_run_was_given() {
        let Ok(asked) = std::env::var(BIND_PORT) else {
            return;
        };
        let port: u16 = asked.parse().expect("a port number");
        let bound = std::net::TcpListener::bind(("127.0.0.1", port));
        assert!(bound.is_ok(), "the child could not bind {port}: {bound:?}");
    }

    /// The same proof as below, against a real other process rather than this one. It is the test
    /// the audit asked for and the one whose absence let the boot breaker through: every proof of
    /// `Ports` was over a type, and what broke was a child being told to bind a number this
    /// program still held.
    #[test]
    fn a_port_a_child_process_is_handed_is_bound_by_it_and_a_held_one_is_not() {
        let mut ports = Ports::default();
        let now = ports.take_and_free().expect("a port to bind now");
        let held = ports.take().expect("a held port");
        let child = |port: u16| {
            Command::new(std::env::current_exe().expect("this test binary"))
                .args(["tests::bind_the_port_this_run_was_given", "--exact"])
                .env(BIND_PORT, port.to_string())
                .output()
                .expect("the child ran")
        };
        assert!(
            child(now).status.success(),
            "a port handed over for a child to bind now was still held by this program"
        );
        assert!(
            !child(held).status.success(),
            "a port this program is holding was free for a child to take, so two children can \
             be handed the same number"
        );
    }

    /// The database is handed a port and binds it at once, so the launcher must not still have it.
    /// Holding it until `release` is what stopped `up` and `backup` starting the database at all.
    #[test]
    fn a_port_for_a_child_that_binds_now_is_let_go_of_and_never_handed_out_twice() {
        let mut ports = Ports::default();
        let held = ports.take().expect("a held port");
        let now = ports.take_and_free().expect("a port to bind now");
        assert_ne!(held, now);
        // What the database does with it, which only works because nothing here holds it.
        let child = std::net::TcpListener::bind(("127.0.0.1", now)).expect("the child binds it");
        drop(child);
        // A port still held here cannot be bound by a child, which is the race this type closes.
        assert!(
            std::net::TcpListener::bind(("127.0.0.1", held)).is_err(),
            "a held port was free for a child to take"
        );
        // And letting one go early does not put its number back in the hat.
        let mut more: Vec<u16> = Vec::new();
        for _ in 0..16 {
            more.push(ports.take().expect("a port"));
        }
        more.push(ports.take_and_free().expect("another port"));
        assert!(!more.contains(&now), "a freed port was handed out twice");
        assert!(!more.contains(&held));
        let mut seen = more.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), more.len(), "two children got the same port");
    }
}
