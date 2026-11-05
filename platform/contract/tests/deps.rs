//! H129: the contract's normal build carries serde and its own parts only, and stays no_std.

use std::path::Path;
use std::process::Command;

const ALLOWED: &[&str] = &[
    "contract",
    "serde",
    "serde_core",
    "serde_derive",
    "proc-macro2",
    "quote",
    "syn",
    "unicode-ident",
];

fn tree(extra: &[&str]) -> Vec<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new(cargo)
        .args([
            "tree",
            "--locked",
            "--offline",
            "-p",
            "contract",
            "--edges",
            "normal",
        ])
        .args(["--prefix", "none", "--format", "{p}", "--manifest-path"])
        .arg(manifest)
        .args(extra)
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
fn h129_normal_build_is_serde_and_its_own_parts() {
    let names = tree(&[]);
    assert!(names.iter().any(|name| name == "serde"), "{names:?}");
    let extra: Vec<&String> = names
        .iter()
        .filter(|name| !ALLOWED.contains(&name.as_str()))
        .collect();
    assert!(extra.is_empty(), "not allowed: {extra:?}");
}

#[test]
fn h129_the_browser_target_tree_is_the_same() {
    let names = tree(&["--target", "wasm32v1-none"]);
    let extra: Vec<&String> = names
        .iter()
        .filter(|name| !ALLOWED.contains(&name.as_str()))
        .collect();
    assert!(extra.is_empty(), "not allowed: {extra:?}");
}

#[test]
fn h129_the_generator_feature_is_the_only_door_to_schemars() {
    let names = tree(&["--features", "schema"]);
    assert!(names.iter().any(|name| name == "schemars"), "{names:?}");
}

#[test]
fn h129_source_stays_no_std() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(src).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        if path.file_name().is_some_and(|name| name == "lib.rs") {
            assert!(text.starts_with("#![no_std]"), "{path:?}");
        }
        for line in text.lines() {
            let code = line.split("//").next().unwrap_or("");
            assert!(!code.contains("std::"), "{path:?}: {line}");
        }
    }
}
