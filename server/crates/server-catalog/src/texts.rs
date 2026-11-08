//! A title's name, story and line in another language, from the catalog's `translations` column.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use moka::sync::Cache;
use rusqlite::{Connection, OpenFlags, OptionalExtension};

/// How many title texts stay in memory, across languages.
const KEPT: u64 = 50_000;
/// How long a read text stays.
const KEPT_S: u64 = 6 * 60 * 60;

/// The region read first for a language when TMDB has several: the one most viewers of it read.
const PREFERRED: &[&str] = &[
    "ar-SA", "ar-AE", "de-DE", "es-ES", "es-MX", "fr-FR", "fr-CA", "hi-IN", "it-IT", "ja-JP",
    "ko-KR", "nl-NL", "pl-PL", "pt-BR", "pt-PT", "ru-RU", "sv-SE", "tr-TR", "zh-CN", "zh-TW",
];

/// A title's words in one language. An empty field has no translation and shows English.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TitleText {
    pub title: String,
    pub overview: String,
    pub tagline: String,
}

#[derive(serde::Deserialize, Default)]
struct Held {
    #[serde(default)]
    title: String,
    #[serde(default)]
    overview: String,
    #[serde(default)]
    tagline: String,
}

/// Reads translations from the catalog file.
pub struct TitleTexts {
    path: PathBuf,
    conn: Mutex<Option<Connection>>,
    kept: Cache<(u32, String), Option<Arc<TitleText>>>,
}

impl std::fmt::Debug for TitleTexts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TitleTexts").field("path", &self.path).finish_non_exhaustive()
    }
}

/// The best text for a language among TMDB's regions: the preferred region first, then any region,
/// field by field, so a missing story in one region is taken from another.
#[must_use]
pub fn pick(held: &BTreeMap<String, TitleText>, lang: &str) -> Option<TitleText> {
    let base = |tag: &str| tag.split(['-', '_']).next().unwrap_or(tag).to_lowercase();
    let mut tags: Vec<&String> = held.keys().filter(|tag| base(tag) == lang).collect();
    let rank = |tag: &str| PREFERRED.iter().position(|p| *p == tag).unwrap_or(PREFERRED.len());
    tags.sort_by_key(|tag| (rank(tag), (*tag).clone()));
    let first = |field: fn(&TitleText) -> &String| {
        tags.iter()
            .map(|tag| field(&held[*tag]).trim())
            .find(|text| !text.is_empty())
            .map(str::to_owned)
            .unwrap_or_default()
    };
    let text = TitleText {
        title: first(|t| &t.title),
        overview: first(|t| &t.overview),
        tagline: first(|t| &t.tagline),
    };
    (text != TitleText::default()).then_some(text)
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
        }
    }

    fn read(&self, media_type: &str, title_id: u32) -> Option<BTreeMap<String, TitleText>> {
        let mut conn = self.conn.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if conn.is_none() {
            *conn = Connection::open_with_flags(
                &self.path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .ok();
        }
        let raw: Option<String> = conn
            .as_ref()?
            .query_row(
                "SELECT translations FROM titles WHERE tmdb_id = ?1 AND media_type = ?2",
                rusqlite::params![title_id, media_type],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten();
        let held: BTreeMap<String, Held> = serde_json::from_str(raw.as_deref()?).ok()?;
        Some(
            held.into_iter()
                .map(|(tag, h)| {
                    let text = TitleText {
                        title: h.title,
                        overview: h.overview,
                        tagline: h.tagline,
                    };
                    (tag, text)
                })
                .collect(),
        )
    }

    /// A title's text in `lang` (a code such as `ar`), or None for English or no translation.
    #[must_use]
    pub fn get(&self, key: u32, media_type: &str, title_id: u32, lang: &str) -> Option<Arc<TitleText>> {
        if lang.is_empty() || lang == "en" {
            return None;
        }
        self.kept
            .get_with((key, lang.to_owned()), || {
                self.read(media_type, title_id)
                    .and_then(|held| pick(&held, lang))
                    .map(Arc::new)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(title: &str, overview: &str) -> TitleText {
        TitleText {
            title: title.to_owned(),
            overview: overview.to_owned(),
            tagline: String::new(),
        }
    }

    #[test]
    fn the_preferred_region_wins_and_others_fill_its_gaps() {
        let held = BTreeMap::from([
            (String::from("ar-AE"), text("الأوديسة", "قصة من الإمارات")),
            (String::from("ar-SA"), text("الأوديسة", "")),
            (String::from("de-DE"), text("Die Odyssee", "Geschichte")),
        ]);
        let ar = pick(&held, "ar").expect("arabic");
        assert_eq!(ar.title, "الأوديسة");
        assert_eq!(ar.overview, "قصة من الإمارات");
        assert!(pick(&held, "ja").is_none());
    }
}
