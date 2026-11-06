//! The gate of the pure core. These fail when the core or the offline layer gains std, async, a
//! clock or IO, a foreign call, a global, a dependency, or a Wasm feature past mutable globals.
//! Covers H1 H2 H3 H9 H106 H107 H108 H109 H113 H114 H118 H130 and C109.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use kit_core::paths::{cargo, root, target_folder, workspace};
use kit_core::scan::{addresses, code_only, scan};
use kit_core::wasm::scan_file;

const CRATES: &[&str] = &["core", "offline"];

/// What a normal build may hold: the crates themselves, serde and serde_json with their parts.
const ALLOWED: &[&str] = &[
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

fn sources(krate: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut folders = vec![root().join(krate).join("src")];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    assert!(!found.is_empty(), "{krate} has no source");
    found
}

fn tree(krate: &str, extra: &[&str]) -> Vec<String> {
    let output = Command::new(cargo())
        .args(["tree", "--locked", "--offline", "--edges", "normal"])
        .args(["--prefix", "none", "--format", "{p}", "--manifest-path"])
        .arg(workspace().join("Cargo.toml"))
        .args(["-p", krate])
        .args(extra)
        .output()
        .expect("cargo tree runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("cargo tree prints text");
    let mut names: Vec<String> = text
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect();
    names.sort();
    names.dedup();
    names
}

#[test]
fn h1_h2_h3_h9_h106_h107_h108_h109_h113_h114_h118_h130_sources_hold_no_std_async_thread_clock_io_or_foreign_code()
 {
    let mut hits = Vec::new();
    for krate in CRATES {
        for file in sources(krate) {
            let text = fs::read_to_string(&file).unwrap();
            for hit in scan(&text) {
                hits.push(format!("{}:{} {hit:?}", file.display(), hit.line));
            }
        }
    }
    assert!(hits.is_empty(), "{hits:#?}");
}

#[test]
fn h1_h106_each_crate_root_is_no_std_and_forbids_unsafe_code() {
    for krate in CRATES {
        let text = fs::read_to_string(root().join(krate).join("src/lib.rs")).unwrap();
        let code = code_only(&text);
        assert!(code.contains("#![no_std]"), "{krate} is not no_std");
        assert!(
            code.contains("#![forbid(unsafe_code)]"),
            "{krate} does not forbid unsafe code"
        );
    }
}

#[test]
fn h106_h109_a_normal_build_holds_serde_serde_json_and_nothing_else() {
    for krate in CRATES {
        for extra in [&[][..], &["--target", "wasm32v1-none"][..]] {
            let names = tree(krate, extra);
            let bad: Vec<&String> = names
                .iter()
                .filter(|name| !ALLOWED.contains(&name.as_str()))
                .collect();
            assert!(bad.is_empty(), "{krate} {extra:?} holds {bad:?}");
        }
    }
}

#[test]
fn h106_the_core_never_knows_the_offline_layer_exists() {
    let names = tree("core", &[]);
    assert!(!names.iter().any(|name| name == "offline"), "{names:?}");
    let manifest = fs::read_to_string(root().join("core/Cargo.toml")).unwrap();
    assert!(!manifest.contains("offline"), "core names the layer");
    assert!(
        !manifest.contains("[features]"),
        "a feature could bring std back"
    );
    let manifest = fs::read_to_string(root().join("offline/Cargo.toml")).unwrap();
    assert!(!manifest.contains("[features]"));
}

#[test]
fn c109_no_address_in_core_or_layer_source() {
    for krate in CRATES {
        for file in sources(krate) {
            let text = fs::read_to_string(&file).unwrap();
            assert_eq!(addresses(&text), Vec::<usize>::new(), "{}", file.display());
        }
    }
}

fn wasm_build(target_dir: &Path, extra: &[&str]) -> std::process::Output {
    Command::new(cargo())
        .args([
            "build",
            "--locked",
            "--offline",
            "--target",
            "wasm32v1-none",
        ])
        .args(["-p", "core", "-p", "offline", "--manifest-path"])
        .arg(workspace().join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target_dir)
        .args(extra)
        .output()
        .expect("cargo build runs")
}

#[test]
fn the_target_itself_offers_mutable_globals_only() {
    let output = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .args(["--print", "cfg", "--target", "wasm32v1-none"])
        .output()
        .expect("rustc runs");
    let text = String::from_utf8(output.stdout).unwrap();
    let features: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("target_feature="))
        .collect();
    assert_eq!(features, ["target_feature=\"mutable-globals\""]);
}

#[test]
fn compiled_core_and_layer_use_mutable_globals_only() {
    let target_dir = target_folder("kit-scan");
    let built = wasm_build(&target_dir, &[]);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let deps = target_dir.join("wasm32v1-none/debug/deps");
    let mut features = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut modules = 0;
    for entry in fs::read_dir(deps).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rlib") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            seen.insert(name.split('-').next().unwrap().to_string());
            modules += scan_file(&fs::read(&path).unwrap(), &mut features).unwrap();
        }
    }
    for wanted in ["libplatform_core", "liboffline", "libserde_json"] {
        assert!(seen.contains(wanted), "{wanted} not built: {seen:?}");
    }
    assert!(modules > 10, "only {modules} objects read");
    let want: BTreeSet<String> = ["mutable-globals".to_string()].into();
    assert_eq!(features, want);
}

#[test]
fn the_wasm_scan_names_a_feature_it_is_shown() {
    // A module with one function holding one simd instruction, and one with sign extension.
    let module = |body: &[u8]| {
        let mut bytes = b"\0asm\x01\0\0\0".to_vec();
        bytes.extend([1, 4, 1, 0x60, 0, 0]);
        bytes.extend([3, 2, 1, 0]);
        let mut code = vec![1, u8::try_from(body.len() + 1).unwrap(), 0];
        code.extend(body);
        bytes.push(10);
        bytes.push(u8::try_from(code.len()).unwrap());
        bytes.extend(code);
        bytes
    };
    let run = |body: &[u8]| {
        let mut found = BTreeSet::new();
        scan_file(&module(body), &mut found).unwrap();
        found
    };
    assert!(run(&[0x0b]).is_empty());
    assert!(run(&[0x41, 0x7f, 0x1a, 0x0b]).is_empty());
    assert!(run(&[0xc0, 0x0b]).contains("sign-ext"));
    assert!(run(&[0xfd, 0x0c]).contains("simd128"));
    assert!(run(&[0xfc, 0x0a, 0, 0, 0x0b]).contains("bulk-memory"));
    assert!(run(&[0xfc, 0x00, 0x0b]).contains("nontrapping-fptoint"));
    assert!(run(&[0x02, 0x00, 0x0b, 0x0b]).contains("multi-value"));
}

#[test]
fn h1_the_scan_catches_planted_std_async_clock_and_io() {
    for (planted, why) in [
        ("use std::collections::HashMap;", "std"),
        ("extern crate std;", "std"),
        ("async fn wait() {}", "async"),
        (
            "fn f(x: impl core::future::Future<Output = ()>) {}",
            "async",
        ),
        (
            "fn now() -> core::time::Duration { Instant::now() }",
            "clock",
        ),
        ("let t = SystemTime::now();", "clock"),
        ("let ready = core::future::ready(());", "async"),
        ("fn f(cx: &mut core::task::Context<'_>) {}", "async"),
        ("let s = TcpStream::connect(a);", "IO"),
        ("let f = File::open(p);", "IO"),
        ("thread_local! { static A: u8 = 0; }", "thread"),
        ("unsafe fn peek() {}", "unsafe"),
    ] {
        let hits = scan(planted);
        assert!(
            hits.iter().any(|hit| hit.why == why),
            "{planted} gave {hits:?}"
        );
    }
    assert_eq!(scan("extern \"C\" { fn time() -> u64; }")[0].word, "extern");
    assert_eq!(scan("static mut N: u8 = 0;")[0].word, "static mut");
    assert_eq!(scan("use offline::Layer;")[0].word, "offline");
}

#[test]
fn the_scan_leaves_docs_strings_and_plain_code_alone() {
    let text = "//! No std, no async.\n/* a File */\nextern crate alloc;\n\
                const WORD: &str = \"std async File\";\nconst C: char = '\"';\n\
                const R: &str = r#\"thread \"Instant\"\"#;\nfn fine() { let standard = 1; }\n";
    assert_eq!(scan(text), Vec::new());
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            fs::copy(&path, &target).unwrap();
        }
    }
}

/// Build a copy of the core with one extra line in its root, against the real wasm target.
fn build_copy(name: &str, plant: &str) -> std::process::Output {
    let base = target_folder("kit-plant");
    let copy = base.join(name);
    if copy.exists() {
        fs::remove_dir_all(&copy).unwrap();
    }
    for krate in ["contract", "core"] {
        fs::create_dir_all(copy.join(krate)).unwrap();
        fs::copy(
            root().join(krate).join("Cargo.toml"),
            copy.join(krate).join("Cargo.toml"),
        )
        .unwrap();
        copy_tree(
            &root().join(krate).join("src"),
            &copy.join(krate).join("src"),
        );
    }
    let manifest = fs::read_to_string(workspace().join("Cargo.toml")).unwrap();
    let start = manifest.find("members = [").unwrap();
    let end = start + manifest[start..].find(']').unwrap() + 1;
    let manifest = format!(
        "{}members = [\"contract\", \"core\"]{}",
        &manifest[..start],
        &manifest[end..]
    );
    fs::write(copy.join("Cargo.toml"), manifest).unwrap();
    fs::copy(workspace().join("Cargo.lock"), copy.join("Cargo.lock")).unwrap();
    fs::copy(
        workspace().join("rust-toolchain.toml"),
        copy.join("rust-toolchain.toml"),
    )
    .unwrap();
    let lib = copy.join("core/src/lib.rs");
    let text = fs::read_to_string(&lib).unwrap();
    fs::write(&lib, format!("{text}\n{plant}\n")).unwrap();
    Command::new(cargo())
        .args([
            "build",
            "--offline",
            "-p",
            "core",
            "--target",
            "wasm32v1-none",
        ])
        .current_dir(&copy)
        .env("CARGO_TARGET_DIR", base.join("target"))
        .output()
        .expect("cargo build runs")
}

#[test]
fn h1_a_planted_std_use_fails_the_wasm_build_and_a_clean_copy_passes() {
    let clean = build_copy("clean", "");
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stderr)
    );
    let planted = build_copy("planted", "extern crate std;");
    assert!(!planted.status.success(), "planted std built");
    let text = String::from_utf8_lossy(&planted.stderr);
    assert!(text.contains("can't find crate for `std`"), "{text}");
}
