//! The same session gives the same bytes on every side of the boundary: native, through the C host
//! and in a browser from a local file. Native means the kit's own host calling the six functions.
//! Covers H110 H111 H112 H115 H116 H117.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use kit_core::host::{Mode, run};
use kit_core::paths::{cargo, root, target_folder, target_root, workspace};
use kit_core::sessions::{common, scripts};
use kit_core::wasm::{I32, Signature, scan_file, shape};
use serde_json::{Value, json};

/// The target for the linked module is 400 KB, which it misses. This ceiling stops it growing
/// unseen until the size work lands.
const SIZE_CEILING: u64 = 700_000;

/// One run of one session: its name, its tape and the bytes the native run wrote.
struct Run {
    name: String,
    tape: String,
    transcript: String,
}

fn runs() -> Vec<Run> {
    let common = common();
    let mut found = Vec::new();
    for script in scripts() {
        for (mode, word) in [(Mode::LayerOff, "off"), (Mode::LayerOn, "on")] {
            let report = run(&script, &common, mode).unwrap();
            found.push(Run {
                name: format!("{}.{word}", script.name),
                tape: report.tape,
                transcript: report.transcript,
            });
        }
    }
    found
}

fn folder(part: &str) -> PathBuf {
    target_folder(&format!("kit-cross/{part}"))
}

fn differ(name: &str, wanted: &str, have: &str) -> Option<String> {
    if wanted == have {
        return None;
    }
    let line = wanted
        .lines()
        .zip(have.lines())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| wanted.lines().count().min(have.lines().count()));
    Some(format!(
        "{name}: differs at line {line}\n want {}\n have {}",
        wanted.lines().nth(line).unwrap_or("(end)"),
        have.lines().nth(line).unwrap_or("(end)")
    ))
}

/// Build the terminal host into the folder this build writes into, and give back its path. The
/// template takes that folder from `TARGET_DIR`, so the library it links is the one this build
/// just wrote and no fixed `target` name is assumed.
fn terminal() -> PathBuf {
    let target = target_root();
    let built = Command::new("make")
        .arg("-C")
        .arg(root().join("templates/terminal"))
        .arg(format!("TARGET_DIR={}", target.display()))
        .env("CARGO", cargo())
        .output()
        .expect("make runs");
    assert!(
        built.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&built.stdout),
        String::from_utf8_lossy(&built.stderr)
    );
    target.join("terminal/terminal")
}

#[test]
fn h110_h112_every_session_gives_the_same_bytes_through_the_c_host() {
    let host = terminal();
    let folder = folder("c");
    let runs = runs();
    assert!(runs.len() >= 168, "only {} runs", runs.len());
    let mut failures = Vec::new();
    for run in &runs {
        let path = folder.join(format!("{}.tape", run.name));
        fs::write(&path, &run.tape).unwrap();
        fs::write(
            folder.join(format!("{}.native.txt", run.name)),
            &run.transcript,
        )
        .unwrap();
        let output = Command::new(&host)
            .arg(&path)
            .output()
            .expect("the host runs");
        if !output.status.success() {
            failures.push(format!(
                "{}: {}",
                run.name,
                String::from_utf8_lossy(&output.stderr)
            ));
            continue;
        }
        let text = String::from_utf8(output.stdout).unwrap();
        fs::write(folder.join(format!("{}.c.txt", run.name)), &text).unwrap();
        failures.extend(differ(&run.name, &run.transcript, &text));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

fn first_tape(part: &str, change: impl Fn(&str) -> String) -> (std::process::Output, Run) {
    let host = terminal();
    let run = runs().into_iter().next().unwrap();
    let path = folder(part).join("changed.tape");
    fs::write(&path, change(&run.tape)).unwrap();
    (Command::new(&host).arg(&path).output().unwrap(), run)
}

#[test]
fn a_tape_that_does_not_fit_what_the_core_asked_stops_the_c_host() {
    let (output, _) = first_tape("no-fit", |tape| {
        tape.replace("\"/v1/hello\"", "\"/v1/other\"")
    });
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(text.contains("does not fit"), "{text}");
}

#[test]
fn a_changed_clock_changes_the_bytes_so_the_compare_can_fail() {
    let (output, run) = first_tape("clock", |tape| tape.replace("wall 1793", "wall 1794"));
    assert!(output.status.success());
    assert_ne!(String::from_utf8(output.stdout).unwrap(), run.transcript);
}

fn linked_module(extra: &[&str], target_dir: &Path) -> PathBuf {
    let built = Command::new(cargo())
        .args(extra)
        .args(["build", "--locked", "--offline", "--profile", "wasm"])
        .args(["-p", "bridge-wasm", "--target", "wasm32v1-none"])
        .arg("--manifest-path")
        .arg(workspace().join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target_dir)
        .output()
        .expect("cargo build runs");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    target_dir.join("wasm32v1-none/wasm/bridge_wasm.wasm")
}

#[test]
fn the_kit_finds_the_folder_this_build_writes_into() {
    let target = target_root();
    let exe = std::env::current_exe().unwrap();
    assert!(target.is_dir(), "{} is no folder", target.display());
    assert!(
        exe.starts_with(&target),
        "{} is not inside {}",
        exe.display(),
        target.display()
    );
    assert!(folder("c").starts_with(&target));
}

#[test]
fn h110_h115_h117_the_linked_module_shows_six_functions_of_i32_and_imports_nothing() {
    let bytes = fs::read(linked_module(&[], &target_root())).unwrap();
    let shape = shape(&bytes).unwrap();
    assert_eq!(shape.imports, 0);
    assert_eq!(shape.others, ["memory"]);
    let i32s = |count: usize| vec![I32; count];
    let want: BTreeMap<String, Signature> = [
        ("core_abi", 0, 1),
        ("core_alloc", 1, 1),
        ("core_close", 1, 0),
        ("core_free", 2, 0),
        ("core_open", 2, 1),
        ("core_step", 5, 1),
    ]
    .into_iter()
    .map(|(name, params, results)| (name.to_string(), (i32s(params), i32s(results))))
    .collect();
    assert_eq!(shape.functions, want);
}

#[test]
fn the_shape_reader_sees_an_import_and_an_i64() {
    let module = |sections: &[&[u8]]| {
        let mut bytes = b"\0asm\x01\0\0\0".to_vec();
        for section in sections {
            bytes.extend(*section);
        }
        bytes
    };
    let kinds: &[u8] = &[1, 5, 1, 0x60, 1, 0x7e, 0];
    let functions: &[u8] = &[3, 2, 1, 0];
    let export: &[u8] = &[7, 5, 1, 1, b'f', 0, 0];
    let body: &[u8] = &[10, 4, 1, 2, 0, 0x0b];
    let found = shape(&module(&[kinds, functions, export, body])).unwrap();
    assert_eq!(found.functions["f"], (vec![0x7e], Vec::new()));
    let import: &[u8] = &[2, 7, 1, 1, b'a', 1, b'b', 0, 0];
    assert_eq!(shape(&module(&[kinds, import])).unwrap().imports, 1);
    assert!(
        shape(b"\0asm\x0d\0\x01\0").is_err(),
        "a component is no module"
    );
}

#[test]
fn h115_the_linked_module_runs_on_the_first_version_of_the_format() {
    let bytes = fs::read(linked_module(&[], &target_root())).unwrap();
    let mut found = BTreeSet::new();
    scan_file(&bytes, &mut found).unwrap();
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn h115_the_feature_list_of_the_linked_module_shows_mutable_globals_only() {
    let extra = ["--config", "profile.wasm.strip=false"];
    let bytes = fs::read(linked_module(&extra, &target_folder("kit-unstripped"))).unwrap();
    let mut found = BTreeSet::new();
    scan_file(&bytes, &mut found).unwrap();
    let want: BTreeSet<String> = ["mutable-globals".to_string()].into();
    assert_eq!(found, want);
}

#[test]
fn the_linked_module_stays_under_its_ceiling() {
    let size = fs::metadata(linked_module(&[], &target_root()))
        .unwrap()
        .len();
    assert!(size <= SIZE_CEILING, "core.wasm is {size} bytes");
}

#[test]
fn h111_no_binding_generator_reaches_a_bridge() {
    let output = Command::new(cargo())
        .args(["tree", "--locked", "--offline", "--edges", "normal"])
        .args(["--prefix", "none", "--format", "{p}", "--manifest-path"])
        .arg(workspace().join("Cargo.toml"))
        .args(["-p", "bridge-c", "-p", "bridge-wasm"])
        .output()
        .expect("cargo tree runs");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let names: BTreeSet<&str> = text
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let allowed = [
        "bridge-c",
        "bridge-wasm",
        "contract",
        "core",
        "itoa",
        "memchr",
        "offline",
        "proc-macro2",
        "quote",
        "serde",
        "serde_core",
        "serde_derive",
        "serde_json",
        "syn",
        "unicode-ident",
        "zmij",
    ];
    let extra: Vec<&&str> = names
        .iter()
        .filter(|name| !allowed.contains(*name))
        .collect();
    assert!(extra.is_empty(), "{extra:?}");
    assert!(names.contains("bridge-wasm"));
    let loader = fs::read_to_string(root().join("bridges/ts/core.ts")).unwrap();
    assert!(!loader.lines().any(|line| line.starts_with("import")));
}

#[test]
fn h116_the_toolchain_is_pinned_to_one_exact_version() {
    let text = fs::read_to_string(workspace().join("rust-toolchain.toml")).unwrap();
    let line = text
        .lines()
        .find(|line| line.starts_with("channel"))
        .unwrap();
    let version = line.split('"').nth(1).unwrap();
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "{version}");
    assert!(
        parts.iter().all(|part| part.parse::<u32>().is_ok()),
        "{version}"
    );
}

fn base64(bytes: &[u8]) -> String {
    const WORDS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len() / 3 * 4 + 4);
    for part in bytes.chunks(3) {
        let group = u32::from(part[0]) << 16
            | u32::from(*part.get(1).unwrap_or(&0)) << 8
            | u32::from(*part.get(2).unwrap_or(&0));
        for at in 0..4 {
            if at <= part.len() {
                text.push(char::from(WORDS[(group >> (18 - at * 6)) as usize & 63]));
            } else {
                text.push('=');
            }
        }
    }
    text
}

fn from_hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}

fn compiled_scripts(compiler: &str, out: &Path) -> (String, String) {
    let done = Command::new(compiler)
        .args(["--strict", "--target", "es2017", "--lib", "es2017,dom"])
        .args(["--module", "esnext", "--moduleResolution", "bundler"])
        .args(["--types", "", "--rootDir", "."])
        .arg("--outDir")
        .arg(out)
        .args(["bridges/ts/core.ts", "kit/core/browser/player.ts"])
        .current_dir(root())
        .output()
        .expect("the compiler runs");
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stdout)
    );
    (
        fs::read_to_string(out.join("bridges/ts/core.js")).unwrap(),
        fs::read_to_string(out.join("kit/core/browser/player.js")).unwrap(),
    )
}

/// Run the page in the browser and give back what it wrote.
fn browser_run(browser: &str, page: &Path, work: &Path) -> Value {
    let dump = work.join("dom.html");
    let mut child = Command::new(browser)
        .args(["--headless=new", "--disable-gpu", "--no-first-run"])
        .args(["--no-default-browser-check", "--disable-extensions"])
        .args(["--disable-sync", "--disable-background-networking"])
        .arg("--host-resolver-rules=MAP * ~NOTFOUND")
        .arg(format!(
            "--user-data-dir={}",
            work.join("profile").display()
        ))
        .args(["--virtual-time-budget=600000", "--dump-dom"])
        .arg(format!("file://{}", page.display()))
        .stdout(File::create(&dump).unwrap())
        .stderr(Stdio::null())
        .spawn()
        .expect("the browser starts");
    let limit = Instant::now() + Duration::from_secs(300);
    while child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < limit, "the browser did not finish");
        std::thread::sleep(Duration::from_millis(100));
    }
    let text = fs::read_to_string(&dump).unwrap();
    let start = text
        .find("<pre id=\"result\">")
        .expect("the page has a result")
        + 17;
    let end = start + text[start..].find("</pre>").unwrap();
    assert!(end > start, "the page wrote no result");
    serde_json::from_slice(&from_hex(&text[start..end])).unwrap()
}

#[test]
#[ignore = "needs KIT_BROWSER, a browser binary, and TSC, a TypeScript compiler"]
fn h115_every_session_gives_the_same_bytes_in_a_headless_browser_from_a_local_file() {
    let browser = std::env::var("KIT_BROWSER").expect("KIT_BROWSER names a browser binary");
    let compiler = std::env::var("TSC").expect("TSC names a TypeScript compiler");
    let work = folder("browser");
    let (core, player) = compiled_scripts(&compiler, &work.join("scripts"));
    let wasm = fs::read(linked_module(&[], &target_root())).unwrap();
    fs::write(work.join("core.wasm"), &wasm).unwrap();
    let runs = runs();
    let tapes: BTreeMap<&str, &str> = runs
        .iter()
        .map(|run| (run.name.as_str(), run.tape.as_str()))
        .collect();
    let tapes = serde_json::to_string(&tapes)
        .unwrap()
        .replace('<', "\\u003c");
    let config = common().config.to_string();
    let step = json!({
        "contract": "2026-11-05", "events": [], "mono_ms": 0, "now_ms": 1000, "type": "core_in",
    })
    .to_string();
    let template = fs::read_to_string(root().join("kit/core/browser/page.html")).unwrap();
    let page = template
        .replace("/*TAPES*/", &tapes)
        .replace("/*CORE*/", &core)
        .replace("/*PLAYER*/", &player)
        .replace("/*CONFIG*/", &json!(config).to_string())
        .replace("/*STEP*/", &json!(step).to_string())
        .replace("/*WASM*/", &base64(&wasm));
    let path = work.join("page.html");
    fs::write(&path, page).unwrap();
    let answer = browser_run(&browser, &path.canonicalize().unwrap(), &work);
    assert!(answer["error"].is_null(), "{}", answer["error"]);
    let mut failures = Vec::new();
    for run in &runs {
        let hex = answer["runs"][&run.name].as_str().expect("a run per tape");
        let text = String::from_utf8(from_hex(hex)).unwrap();
        fs::write(work.join(format!("{}.browser.txt", run.name)), &text).unwrap();
        failures.extend(differ(&run.name, &run.transcript, &text));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    let stats = &answer["stats"];
    let mut failed = String::new();
    for (name, held) in stats["checks"].as_object().unwrap() {
        if held != &json!(true) {
            let _ = writeln!(failed, "{name}");
        }
    }
    assert!(failed.is_empty(), "{failed}");
    println!("{stats:#}");
}
