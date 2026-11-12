//! The schema crate's normal build carries prost and its own parts only.
//!
//! The twin of `platform/contract/tests/deps.rs`, which holds the same line for the
//! JSON contract under H129, and it is here for the same reason at a different
//! scale. The whole case for a binary wire is a size case: a module that round trips
//! sixteen types costs 127,244 bytes against serde_json's 228,543, and every byte of
//! that rests on prost being `no_std` and on nothing else getting in. A std
//! dependency arriving in this crate - a logger, an error library with a `Display`
//! that formats, anything that wants `std::io` - would not fail any other check in
//! this repository. `./check` builds this crate for `wasm32v1-none`, which catches
//! the ones that cannot compile at all; this test catches the ones that can.
//!
//! What is deliberately NOT in the list: `prost-reflect` (a dev dependency, for the
//! text format corpus and for reading the options back out of the descriptor),
//! `prost-build` and `protox` (build dependencies, which compile the schema with no
//! `protoc` installed). `--edges normal` sees none of the three, and that is the
//! point of their being where they are.

use std::path::Path;
use std::process::Command;

/// prost and what prost brings. `anyhow`, `either` and `itertools` arrive through
/// `prost-derive`, which is a proc macro: it runs in the compiler and links nothing
/// into the module.
const ALLOWED: &[&str] = &[
    "contract-proto",
    "prost",
    "prost-derive",
    "bytes",
    "anyhow",
    "either",
    "itertools",
    "proc-macro2",
    "quote",
    "syn",
    "unicode-ident",
];

fn tree() -> Vec<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new(cargo)
        .args([
            "tree",
            "--locked",
            "--offline",
            "-p",
            "contract-proto",
            "--edges",
            "normal",
        ])
        .args(["--prefix", "none", "--format", "{p}", "--manifest-path"])
        .arg(manifest)
        .output()
        .expect("cargo tree runs");
    let text = String::from_utf8(output.stdout).expect("cargo tree prints text");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
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
fn the_normal_build_is_prost_and_its_own_parts() {
    let names = tree();
    assert!(names.iter().any(|name| name == "prost"), "{names:?}");
    let extra: Vec<&String> = names
        .iter()
        .filter(|name| !ALLOWED.contains(&name.as_str()))
        .collect();
    assert!(extra.is_empty(), "not allowed: {extra:?}");
}

#[test]
fn prost_is_not_dragging_std_in_through_its_default_features() {
    // `default-features = false` in the root manifest is the line that keeps prost
    // `no_std`. Its `std` feature turns on `bytes/std` and `prost-derive`'s host
    // side, and the symptom would be a crate that still compiles for the host and
    // no longer compiles for `wasm32v1-none` - which `./check` catches, but only
    // once somebody runs the wasm line. This names the cause instead.
    let names = tree();
    for forbidden in ["std", "once_cell", "log", "thiserror"] {
        assert!(
            !names.iter().any(|name| name == forbidden),
            "{forbidden} is in the normal build: {names:?}"
        );
    }
}
