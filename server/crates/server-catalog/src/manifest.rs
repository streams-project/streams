//! One manifest names a data set: model, dimensions, row count and a hash per file. A set whose
//! files do not match it is refused, so a wrong file swap cannot start.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// The manifest shape this build reads.
pub const SCHEMA_VERSION: u32 = 1;

/// File roles. A file with no role is never read.
pub mod role {
    pub const CATALOG: &str = "catalog";
    pub const VECTORS: &str = "vectors";
    pub const VECTOR_KEYS: &str = "vector_keys";
    pub const SEEDS: &str = "seeds";
    pub const SEED_NAMES: &str = "seed_names";
    pub const SEED_GROUPS: &str = "seed_groups";
    pub const TAXONOMY_VECTORS: &str = "taxonomy_vectors";
    pub const TAXONOMY_ENTRIES: &str = "taxonomy_entries";
    pub const KIDS_VECTORS: &str = "kids_taxonomy_vectors";
    pub const KIDS_ENTRIES: &str = "kids_taxonomy_entries";
    pub const PROFILES: &str = "title_profiles";
    pub const POPULARITY: &str = "popularity";
    pub const AWARDS: &str = "awards";
    pub const CATEGORIES: &str = "categories";
    pub const TOPICS: &str = "topics";
    pub const TOPICS_META: &str = "topics_meta";
    pub const FRESH: &str = "fresh";
    pub const EVENTS: &str = "events";
    pub const REQUESTS: &str = "row_requests";
    pub const REMOVED_ROWS: &str = "removed_rows";
    pub const ALIASES: &str = "search_aliases";
}

/// One file of a set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Path relative to the manifest.
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    /// A required file that fails stops the set. An optional one is dropped with a warning.
    #[serde(default)]
    pub required: bool,
}

/// One data set, as the data builder writes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetManifest {
    pub schema_version: u32,
    pub dataset_version: String,
    pub build_commit: String,
    pub built_at_ms: u64,
    pub model: String,
    pub dimensions: u32,
    pub quantisation: String,
    pub row_count: u64,
    pub files: BTreeMap<String, FileEntry>,
}

/// Why a set was refused.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestError {
    #[error("the manifest cannot be read: {0}")]
    Unreadable(String),
    #[error("schema {0} is not one this build reads")]
    Schema(u32),
    #[error("quantisation {0} is not one this build reads")]
    Quantisation(String),
    #[error("required file {0} is missing from the manifest")]
    MissingRole(&'static str),
    #[error("file {0} does not exist")]
    MissingFile(String),
    #[error("file {0} has the wrong size")]
    WrongSize(String),
    #[error("file {0} has the wrong hash")]
    WrongHash(String),
    #[error("the vectors do not fit {0} rows of {1} dimensions")]
    WrongShape(u64, u32),
}

/// Paths of the files that passed, by role.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataFiles {
    paths: BTreeMap<String, PathBuf>,
}

impl DataFiles {
    /// A set of files with these paths, for tests and checks.
    #[must_use]
    pub fn from_pairs(pairs: &[(&str, &Path)]) -> Self {
        Self {
            paths: pairs
                .iter()
                .map(|(role, path)| ((*role).to_owned(), path.to_path_buf()))
                .collect(),
        }
    }

    /// The path of a role, when the file passed.
    #[must_use]
    pub fn get(&self, role: &str) -> Option<&Path> {
        self.paths.get(role).map(PathBuf::as_path)
    }
}

/// The lowercase hex hash of a file, read in steps, so a big file costs no memory.
///
/// # Errors
/// The file cannot be read.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    use std::fmt::Write;
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0u8; 1 << 20];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    let mut text = String::with_capacity(64);
    for byte in hash.finalize() {
        let _ = write!(text, "{byte:02x}");
    }
    Ok(text)
}

fn check_file(dir: &Path, entry: &FileEntry) -> Result<PathBuf, ManifestError> {
    let path = dir.join(&entry.path);
    let meta =
        std::fs::metadata(&path).map_err(|_| ManifestError::MissingFile(entry.path.clone()))?;
    if meta.len() != entry.bytes {
        return Err(ManifestError::WrongSize(entry.path.clone()));
    }
    let hash = sha256_file(&path).map_err(|_| ManifestError::MissingFile(entry.path.clone()))?;
    if hash != entry.sha256 {
        return Err(ManifestError::WrongHash(entry.path.clone()));
    }
    Ok(path)
}

impl DatasetManifest {
    /// Read and parse one manifest file, with the hash of its bytes.
    ///
    /// # Errors
    /// The file is missing or is not a manifest.
    pub fn read(path: &Path) -> Result<(Self, String), ManifestError> {
        let bytes = std::fs::read(path).map_err(|e| ManifestError::Unreadable(e.to_string()))?;
        let manifest =
            serde_json::from_slice(&bytes).map_err(|e| ManifestError::Unreadable(e.to_string()))?;
        let hash = sha256_file(path).map_err(|e| ManifestError::Unreadable(e.to_string()))?;
        Ok((manifest, hash))
    }

    /// Check every file against the manifest. Required files must pass; an optional file that
    /// fails is left out and named in the second answer, and the set still loads.
    ///
    /// # Errors
    /// The first required file that fails, or a shape that cannot hold.
    pub fn validate(&self, dir: &Path) -> Result<(DataFiles, Vec<String>), ManifestError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ManifestError::Schema(self.schema_version));
        }
        if self.quantisation != "f16" {
            return Err(ManifestError::Quantisation(self.quantisation.clone()));
        }
        for required in [role::CATALOG, role::VECTORS, role::VECTOR_KEYS] {
            if !self.files.contains_key(required) {
                return Err(ManifestError::MissingRole(required));
            }
        }
        let shape = |role: &str, unit: u64| -> Result<(), ManifestError> {
            let bytes = self.files.get(role).map_or(0, |entry| entry.bytes);
            if bytes == self.row_count * unit {
                Ok(())
            } else {
                Err(ManifestError::WrongShape(self.row_count, self.dimensions))
            }
        };
        shape(role::VECTORS, u64::from(self.dimensions) * 2)?;
        shape(role::VECTOR_KEYS, 4)?;
        let mut files = DataFiles::default();
        let mut dropped = Vec::new();
        for (name, entry) in &self.files {
            let required = entry.required
                || matches!(
                    name.as_str(),
                    role::CATALOG | role::VECTORS | role::VECTOR_KEYS
                );
            match check_file(dir, entry) {
                Ok(path) => {
                    files.paths.insert(name.clone(), path);
                }
                Err(error) if required => return Err(error),
                Err(error) => dropped.push(format!("{name}: {error}")),
            }
        }
        Ok((files, dropped))
    }
}
