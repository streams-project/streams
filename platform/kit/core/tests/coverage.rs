//! Every ledger row homed in the core or the layer is either proved by a test or a session that
//! names it, or written down in `open.csv` with what is missing and what unblocks it. An id in a
//! test name proves the test exists. Whether it holds is what the test itself checks.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Ids of the rows whose home is the core or the layer.
fn rows() -> BTreeSet<String> {
    let text = fs::read_to_string(root().join("contract/ledger.csv")).unwrap();
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split(',');
            let id = parts.next()?;
            let home = parts.nth(2)?;
            (home == "core" || home == "offline").then(|| id.to_string())
        })
        .collect()
}

/// An id in a test name is a letter and digits between underscores, such as `c17` in
/// `c17_c28_a_failed_host`.
fn ids_in(name: &str) -> impl Iterator<Item = String> + '_ {
    name.split('_').filter_map(|part| {
        let (first, rest) = part.split_at_checked(1)?;
        let known = matches!(first, "c" | "h" | "p" | "n" | "s" | "u");
        (known && !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| part.to_ascii_uppercase())
    })
}

fn test_names(folder: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let text = fs::read_to_string(path).unwrap();
        let mut lines = text.lines();
        while let Some(line) = lines.next() {
            if line.trim() == "#[test]"
                && let Some(next) = lines.next()
                && let Some(rest) = next.trim().strip_prefix("fn ")
            {
                names.push(rest.split('(').next().unwrap().to_string());
            }
        }
    }
    assert!(!names.is_empty(), "{folder:?} holds no test");
    names
}

/// Every id named by a test or a session.
fn proved() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for folder in ["core/tests", "offline/tests", "kit/core/tests"] {
        for name in test_names(&root().join(folder)) {
            found.extend(ids_in(&name));
        }
    }
    for entry in fs::read_dir(root().join("kit/core/sessions")).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().is_some_and(|name| name == "common.json") {
            continue;
        }
        let script: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        for id in script["covers"].as_array().unwrap() {
            found.insert(id.as_str().unwrap().to_string());
        }
    }
    found
}

/// `open.csv`: id, built (no or part), what is missing, what unblocks it.
fn open_rows() -> BTreeMap<String, String> {
    let text = fs::read_to_string(root().join("kit/core/open.csv")).unwrap();
    let mut rows = BTreeMap::new();
    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        assert_eq!(parts.len(), 4, "{line}");
        assert!(matches!(parts[1], "no" | "part"), "{line}");
        assert!(!parts[2].is_empty() && !parts[3].is_empty(), "{line}");
        rows.insert(parts[0].to_string(), parts[1].to_string());
    }
    rows
}

#[test]
fn every_core_and_layer_row_has_a_test_or_a_written_reason() {
    let proved = proved();
    let open = open_rows();
    let rows = rows();
    let missing: Vec<&String> = rows
        .iter()
        .filter(|id| !proved.contains(*id) && !open.contains_key(*id))
        .collect();
    assert!(missing.is_empty(), "no test and no reason: {missing:?}");
}

#[test]
fn a_row_written_down_as_not_built_has_no_test_claiming_it() {
    let proved = proved();
    let rows = rows();
    for (id, built) in open_rows() {
        assert!(rows.contains(&id), "{id} is not a core or layer row");
        match built.as_str() {
            "no" => assert!(!proved.contains(&id), "{id} is open and also claimed"),
            _ => assert!(proved.contains(&id), "{id} is part built with no test"),
        }
    }
}

#[test]
fn an_id_in_a_test_name_is_read_whole() {
    let name = "c17_c28_h9_a_failed_host_and_its_group";
    let found: Vec<String> = ids_in(name).collect();
    assert_eq!(found, ["C17", "C28", "H9"]);
    assert_eq!(ids_in("the_core_marks_each_request").count(), 0);
}
