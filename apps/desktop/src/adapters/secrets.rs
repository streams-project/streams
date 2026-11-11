//! The secrets port, on one file per secret under the host's own folder.
//!
//! This is not an OS keychain. `DECISIONS.md` records why one is not here yet and what it would
//! take, so nobody reads this as more than it is: a file the owner alone can read, which is
//! durable, which is what the core asks of this port.

use std::path::{Path, PathBuf};

use crate::adapters::{file_name, write_whole};
use crate::ports::Secrets;

pub struct FileSecrets {
    folder: PathBuf,
}

impl FileSecrets {
    /// Secrets under this folder. The folder is made on the first write, so a machine that refuses
    /// to make it answers refused rather than failing at the start.
    #[must_use]
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into(),
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.folder.join(file_name(name))
    }
}

impl Secrets for FileSecrets {
    fn get(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.path(name)).ok()
    }

    fn put(&self, name: &str, value: &str) -> bool {
        write_whole(&self.path(name), value)
    }

    fn delete(&self, name: &str) {
        let path: &Path = &self.path(name);
        let _ = std::fs::remove_file(path);
    }
}
