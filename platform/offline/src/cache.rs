//! Kept answers by server ttl, scoped by account and profile, with one shared request per key.
//! Bodies live in the store; only this index stays in memory. Failures are forgotten. An entry
//! past its ttl stays for when the network is gone, so the last good hello is always there.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use contract::core_msg::BodyTarget;
use serde::{Deserialize, Serialize};

/// Key of the index in the store.
pub const INDEX_KEY: &str = "index";

/// Prefix of kept bodies in the store.
pub const BODY_PREFIX: &str = "c/";

/// One kept answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub store_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub ttl_ms: u64,
    pub stored_mono: u64,
    pub boot: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    pub bytes: u64,
    pub seq: u64,
    /// True after the server said this changed.
    #[serde(default)]
    pub stale: bool,
}

impl Entry {
    /// A new entry. Its key, scope and order are set when it is stored.
    #[must_use]
    pub fn new(
        ttl_ms: u64,
        stored_mono: u64,
        boot: &str,
        etag: Option<String>,
        bytes: u64,
    ) -> Self {
        Self {
            store_key: String::new(),
            scope: None,
            ttl_ms,
            stored_mono,
            boot: boot.into(),
            etag,
            bytes,
            seq: 0,
            stale: false,
        }
    }

    /// Freshness needs the same boot, because monotonic time restarts with it.
    #[must_use]
    pub fn fresh(&self, boot: &str, mono_ms: u64) -> bool {
        !self.stale
            && self.boot == boot
            && mono_ms >= self.stored_mono
            && mono_ms - self.stored_mono < self.ttl_ms
    }
}

/// The part that survives a restart.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    #[serde(default)]
    pub entries: BTreeMap<String, Entry>,
    #[serde(default)]
    pub cursors: BTreeMap<String, String>,
    #[serde(default)]
    pub next: u64,
}

/// One effect waiting for a shared answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Waiter {
    pub id: String,
    pub to: BodyTarget,
}

/// The cache.
#[derive(Default)]
pub struct Cache {
    pub index: Index,
    /// Requests on their way, by key.
    pub waiting: BTreeMap<String, Vec<Waiter>>,
}

/// The key of a request: scope and path.
#[must_use]
pub fn key_of(scope: Option<&str>, path: &str) -> String {
    format!("{}|{path}", scope.unwrap_or_default())
}

impl Cache {
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.index.entries.values().map(|entry| entry.bytes).sum()
    }

    /// Note a stored body and return its store key.
    pub fn stored(&mut self, key: String, scope: Option<String>, mut entry: Entry) -> String {
        entry.seq = self.index.next;
        entry.scope = scope;
        self.index.next += 1;
        entry.store_key = self.index.entries.get(&key).map_or_else(
            || format!("{BODY_PREFIX}{:016x}", entry.seq),
            |old| old.store_key.clone(),
        );
        let store_key = entry.store_key.clone();
        self.index.entries.insert(key, entry);
        store_key
    }

    /// The server says the kept copy is still good.
    pub fn renewed(&mut self, key: &str, stored_mono: u64, boot: &str) {
        if let Some(entry) = self.index.entries.get_mut(key) {
            entry.stored_mono = stored_mono;
            entry.boot = boot.into();
            entry.stale = false;
        }
    }

    /// Drop the oldest entry, scoped ones first, so the last good hello goes last.
    pub fn evict_one(&mut self) -> Option<String> {
        let key = self
            .index
            .entries
            .iter()
            .min_by_key(|(_, entry)| (entry.scope.is_none(), entry.seq))
            .map(|(key, _)| key.clone())?;
        self.index.entries.remove(&key).map(|entry| entry.store_key)
    }

    /// Forget one key, as a missing body shows it is gone.
    pub fn forget(&mut self, key: &str) -> Option<String> {
        self.index.entries.remove(key).map(|entry| entry.store_key)
    }

    /// The server changed something for this scope, so its kept pages are read again.
    pub fn mark_stale(&mut self, scope: &str) {
        for entry in self.index.entries.values_mut() {
            if entry.scope.as_deref() == Some(scope) {
                entry.stale = true;
            }
        }
    }

    /// Forget requests of a session that ended. Keys with no scope start with the separator.
    pub fn forget_waiting(&mut self) {
        self.waiting.retain(|key, _| key.starts_with('|'));
    }

    /// Forget account data. Answers with no scope, such as hello, stay. Returns keys to delete.
    pub fn clear_scoped(&mut self) -> Vec<String> {
        let keys = self
            .index
            .entries
            .values()
            .filter(|entry| entry.scope.is_some())
            .map(|entry| entry.store_key.clone())
            .collect();
        self.index.entries.retain(|_, entry| entry.scope.is_none());
        self.index.cursors.clear();
        keys
    }
}
