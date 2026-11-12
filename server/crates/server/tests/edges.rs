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

/// The one file of `server-accounts` that may write a video word, and why.
///
/// `holdings.rs` names every table of `server/migrations` exactly once, by the name the table
/// really has, and two of those tables are called `episode_progress` and `refused_trailers`. A
/// register of what is held about a person cannot name a table by a word the table does not use:
/// the person is entitled to be told we keep how far through an episode they were, and its own
/// test reads the migration files, so a softer word would fail there instead. The exemption is
/// therefore on the register and on nothing else, and `the_register_only_names_tables_and_says
/// _what_they_are_for` below holds it to naming and describing rather than to doing video work.
const REGISTER: &str = "holdings.rs";

/// The words that belong to the catalogue and not to accounts.
const VIDEO_WORDS: [&str; 5] = ["movie", "episode", "season", "trailer", "catalog"];

fn accounts_sources() -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("../server-accounts/src")];
    while let Some(folder) = stack.pop() {
        for entry in std::fs::read_dir(folder).expect("folder") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }
    found
}

#[test]
fn a_project_that_takes_only_accounts_gets_no_video_word() {
    for path in accounts_sources() {
        if path.file_name().is_some_and(|name| name == REGISTER) {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .expect("source")
            .to_lowercase();
        for word in VIDEO_WORDS {
            assert!(!text.contains(word), "{} holds {word}", path.display());
        }
    }
}

/// The register's exemption, bounded: every video word in it names a table or says what the table
/// is for. A `fn`, a `match`, an `if` or a call carrying one of these words would be video work
/// living in the accounts crate, which is what the test above exists to stop, and the exemption
/// must not become the hole that lets it in.
#[test]
fn the_register_only_names_tables_and_says_what_they_are_for() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../server-accounts/src")
        .join(REGISTER);
    let text = std::fs::read_to_string(&path).expect("the register");
    let mut checked = 0usize;
    for (number, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        if !VIDEO_WORDS.iter().any(|word| lower.contains(word)) {
            continue;
        }
        checked += 1;
        let trimmed = lower.trim_start();
        let naming = trimmed.starts_with("table:")
            || trimmed.starts_with("purpose:")
            || trimmed.starts_with("//")
            || trimmed.starts_with("///");
        assert!(
            naming,
            "{}:{}: a video word outside a table name, a purpose or a comment: {}",
            path.display(),
            number + 1,
            line.trim()
        );
    }
    // If the register stops naming those tables at all, this test has stopped reading anything and
    // the exemption above is no longer carrying its weight.
    assert!(
        checked >= 2,
        "the register named a video word on {checked} lines: \
         `episode_progress` and `refused_trailers` are tables of server/migrations and the register \
         names every table once, so if they have gone the exemption should go with them"
    );
}

#[test]
fn h77_handlers_live_in_a_library_so_tests_reach_them() {
    let binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let text = std::fs::read_to_string(binary).expect("source");
    assert!(!text.contains(".route("), "routes belong in server-api");
}
