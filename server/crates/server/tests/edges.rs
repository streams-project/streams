//! The allowed crate edges, read from the manifests. A Rust module cannot stop a cycle or a wrong
//! dependency; this does (note the table it enforces in each crate's own docs).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Each library crate and the local crates it may take.
const ALLOWED: &[(&str, &[&str])] = &[
    ("server-accounts", &[]),
    ("server-catalog", &[]),
    (
        "server-rules",
        &["server-accounts", "server-catalog", "contract"],
    ),
    ("server-library", &["server-accounts", "contract"]),
    ("server-help", &["contract"]),
    (
        "server-pages",
        &[
            "server-rules",
            "server-catalog",
            "server-library",
            "server-accounts",
            "contract",
        ],
    ),
    (
        "server-play",
        &[
            "server-rules",
            "server-catalog",
            "server-library",
            "server-accounts",
            "contract",
        ],
    ),
    (
        "server-api",
        &[
            "server-accounts",
            "server-catalog",
            "server-rules",
            "server-help",
            "server-library",
            "server-pages",
            "server-play",
            "contract",
        ],
    ),
];

fn local_dependencies(name: &str) -> BTreeSet<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(name)
        .join("Cargo.toml");
    let text = std::fs::read_to_string(path).expect("manifest");
    let mut section = String::new();
    let mut found = BTreeSet::new();
    for line in text.lines() {
        if line.starts_with('[') {
            section = line.to_owned();
            continue;
        }
        if section == "[dependencies]"
            && let Some((dependency, _)) = line.split_once(['.', ' '])
            && (dependency.starts_with("server") || dependency == "contract")
        {
            found.insert(dependency.to_owned());
        }
    }
    found
}

#[test]
fn only_the_edges_of_the_table_exist() {
    let allowed: BTreeMap<&str, &[&str]> = ALLOWED.iter().copied().collect();
    for (name, may) in &allowed {
        for dependency in local_dependencies(name) {
            assert!(
                may.contains(&dependency.as_str()),
                "{name} may not take {dependency}"
            );
        }
    }
}

#[test]
fn tmk14_the_rules_read_no_setting() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../server-rules/src");
    for entry in std::fs::read_dir(folder).expect("folder") {
        let text = std::fs::read_to_string(entry.expect("entry").path()).expect("source");
        assert!(
            !text.contains("env::var"),
            "no environment value reaches the rules"
        );
    }
}

#[test]
fn h26_only_the_binary_reads_the_process_environment() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in [
        "server-accounts",
        "server-catalog",
        "server-library",
        "server-pages",
        "server-play",
        "server-rules",
    ] {
        let mut stack = vec![crates.join(name).join("src")];
        while let Some(folder) = stack.pop() {
            for entry in std::fs::read_dir(folder).expect("folder") {
                let path = entry.expect("entry").path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let text = std::fs::read_to_string(&path).expect("source");
                    assert!(!text.contains("env::var"), "{}", path.display());
                }
            }
        }
    }
}

#[test]
fn a_project_that_takes_only_accounts_gets_no_video_word() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../server-accounts/src");
    let words = ["movie", "episode", "season", "trailer", "catalog"];
    let mut stack = vec![folder];
    while let Some(folder) = stack.pop() {
        for entry in std::fs::read_dir(folder).expect("folder") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = std::fs::read_to_string(&path)
                    .expect("source")
                    .to_lowercase();
                for word in words {
                    assert!(!text.contains(word), "{} holds {word}", path.display());
                }
            }
        }
    }
}

#[test]
fn h77_handlers_live_in_a_library_so_tests_reach_them() {
    let binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let text = std::fs::read_to_string(binary).expect("source");
    assert!(!text.contains(".route("), "routes belong in server-api");
}
