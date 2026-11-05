//! Reads the JSON sessions of this kit.

use std::fs;
use std::path::{Path, PathBuf};

use crate::host::{Common, Script};

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("sessions")
}

/// The shared parts of the sessions.
///
/// # Panics
/// The file is missing or does not read.
#[must_use]
pub fn common() -> Common {
    let text = fs::read_to_string(folder().join("common.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Every session, sorted by file name.
///
/// # Panics
/// A file is missing or does not read.
#[must_use]
pub fn scripts() -> Vec<Script> {
    let mut paths: Vec<PathBuf> = fs::read_dir(folder())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter(|path| path.file_name().is_some_and(|name| name != "common.json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).unwrap();
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path:?}: {error}"))
        })
        .collect()
}
