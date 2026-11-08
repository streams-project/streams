//! A title's name, story and line in another language, from the catalog's `title_texts` table: one
//! row per title and language, the region already chosen when the set was built. Names of a whole
//! language are read in one pass the first time a page in that language asks, then kept, so a page
//! names its tiles from memory; a title's story is read by its key and kept a while. English is the
//! catalog's own text and never comes from here. A set without the table answers no translation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use moka::sync::Cache;
use rusqlite::{Connection, OpenFlags, OptionalExtension};

use crate::catalog::item_key;

/// How many title texts stay in memory, across languages.
const KEPT: u64 = 20_000;
/// How long a read text stays.
const KEPT_S: u64 = 6 * 60 * 60;

/// A title's words in one language. An empty field has no translation and shows English.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TitleText {
    pub title: String,
    pub overview: String,
    pub tagline: String,
}

/// Reads translations from the catalog file.
pub struct TitleTexts {
    path: PathBuf,
    conn: Mutex<Option<Connection>>,
    kept: Cache<(u32, String), Option<Arc<TitleText>>>,
    names: Mutex<HashMap<String, Arc<HashMap<u32, String>>>>,
}

impl std::fmt::Debug for TitleTexts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TitleTexts").field("path", &self.path).finish_non_exhaustive()
    }
}

fn plain(lang: &str) -> bool {
    !lang.is_empty() && lang != "en"
}

impl TitleTexts {
    /// Texts read from the catalog file at `path`.
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            conn: Mutex::new(None),
            kept: Cache::builder()
                .max_capacity(KEPT)
                .time_to_live(Duration::from_secs(KEPT_S))
                .build(),
            names: Mutex::new(HashMap::new()),
        }
    }

    fn with_conn<T>(&self, read: impl FnOnce(&Connection) -> Option<T>) -> Option<T> {
        let mut conn = self.conn.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if conn.is_none() {
            *conn = Connection::open_with_flags(
                &self.path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .ok();
        }
        read(conn.as_ref()?)
    }

    /// Every title's name in `lang`, read once per language.
    fn names_in(&self, lang: &str) -> Arc<HashMap<u32, String>> {
        if let Some(held) = self
            .names
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(lang)
        {
            return held.clone();
        }
        let read = self
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare(concat!(
                        "SELECT media_type, ",
                        "tm",
                        "db",
                        "_id, title FROM title_texts WHERE lang = ?1 AND title != ''"
                    ))
                    .ok()?;
                let rows = stmt
                    .query_map([lang], |row| {
                        let kind: String = row.get(0)?;
                        let id: u32 = row.get(1)?;
                        let name: String = row.get(2)?;
                        Ok((item_key(&kind, id), name))
                    })
                    .ok()?;
                Some(rows.filter_map(Result::ok).collect::<HashMap<u32, String>>())
            })
            .unwrap_or_default();
        let read = Arc::new(read);
        self.names
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(lang.to_owned(), read.clone());
        read
    }

    /// A title's name in `lang` (a code such as `ar`), or None for English or no translation.
    #[must_use]
    pub fn name(&self, key: u32, lang: &str) -> Option<String> {
        if !plain(lang) {
            return None;
        }
        self.names_in(lang).get(&key).cloned()
    }

    /// A title's name, story and line in `lang`, or None for English or no translation.
    #[must_use]
    pub fn get(&self, key: u32, media_type: &str, title_id: u32, lang: &str) -> Option<Arc<TitleText>> {
        if !plain(lang) {
            return None;
        }
        self.kept.get_with((key, lang.to_owned()), || {
            self.with_conn(|conn| {
                conn.query_row(
                    concat!(
                        "SELECT title, overview, tagline FROM title_texts WHERE lang = ?1 AND media_type = ?2 AND ",
                        "tm",
                        "db",
                        "_id = ?3"
                    ),
                    rusqlite::params![lang, media_type, title_id],
                    |row| {
                        Ok(TitleText {
                            title: row.get(0)?,
                            overview: row.get(1)?,
                            tagline: row.get(2)?,
                        })
                    },
                )
                .optional()
                .ok()
                .flatten()
            })
            .map(Arc::new)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_reads_names_in_one_pass_and_stories_by_key() {
        let dir = std::env::temp_dir().join(format!("texts_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("catalog.db");
        let _ = std::fs::remove_file(&path);
        let conn = Connection::open(&path).expect("db");
        conn.execute_batch(concat!(
            "CREATE TABLE title_texts (lang TEXT, media_type TEXT, ",
            "tm",
            "db",
            "_id INTEGER, title TEXT, overview TEXT, tagline TEXT);\n",
            "INSERT INTO title_texts VALUES ('ar', 'movie', 7, 'الأوديسة', 'قصة', '');"
        ))
        .expect("rows");
        drop(conn);
        let texts = TitleTexts::new(&path);
        let key = item_key("movie", 7);
        assert_eq!(texts.name(key, "ar").as_deref(), Some("الأوديسة"));
        assert_eq!(texts.name(key, "en"), None);
        assert_eq!(texts.get(key, "movie", 7, "ar").map(|t| t.overview.clone()).as_deref(), Some("قصة"));
        assert!(texts.get(key, "movie", 7, "de").is_none());
    }
}
