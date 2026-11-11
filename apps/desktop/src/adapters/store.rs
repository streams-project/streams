//! The store port, on one file per key under the host's own folder. Only the offline layer uses
//! it: waiting commands and kept pages.

use std::path::PathBuf;

use crate::adapters::{file_name, key_of, write_whole};
use crate::ports::Store;

pub struct FileStore {
    folder: PathBuf,
}

impl FileStore {
    #[must_use]
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into(),
        }
    }

    fn path(&self, key: &str) -> PathBuf {
        self.folder.join(file_name(key))
    }
}

impl Store for FileStore {
    fn get(&self, key: &str) -> Option<String> {
        std::fs::read_to_string(self.path(key)).ok()
    }

    fn put(&self, key: &str, value: &str) -> bool {
        write_whole(&self.path(key), value)
    }

    fn delete(&self, key: &str) {
        let _ = std::fs::remove_file(self.path(key));
    }

    /// Every key under this prefix. A file this host did not write is passed over, so a stray file
    /// in the folder cannot come back as a key the layer then tries to read.
    fn keys(&self, prefix: &str) -> Vec<String> {
        let Ok(reading) = std::fs::read_dir(&self.folder) else {
            return Vec::new();
        };
        let mut found: Vec<String> = reading
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter_map(|name| key_of(&name))
            .filter(|key| key.starts_with(prefix))
            .collect();
        // The layer reads the queue in order, and a folder hands its files back in whatever order
        // it keeps them, so the order is made here instead of being hoped for.
        found.sort();
        found
    }
}
