//! The words a page carries, in the profile's language. They live in `strings/{lang}.json` with
//! every other word; row names made from the data are translated whole in `strings/rows/{lang}.json`
//! (English name to name), so a name reads as the language writes it, not as English pieces glued
//! together. A word a language lacks comes back in English. The name comes from the brand, so
//! neither the name nor any word is typed into the code.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use contract::hello::PLACEHOLDER_BRAND;
use serde_json::Value;

/// The language every other one falls back to.
pub const ENGLISH_CODE: &str = "en";

static ENGLISH: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../strings/en.json")).unwrap_or_default()
});

/// Opens every line of a right to left language, so a line that starts with a Latin name (the
/// brand, a title) still runs right to left in any client.
const RIGHT_TO_LEFT_MARK: char = '\u{200F}';

/// Start and end of a value put into a sentence: Unicode isolates, so a number, a title or an
/// address keeps its own order inside a line of either direction.
const ISOLATE: (char, char) = ('\u{2068}', '\u{2069}');

/// Every language's words, read once from the strings folder.
#[derive(Debug, Default)]
pub struct Books {
    texts: BTreeMap<String, BTreeMap<String, String>>,
    rows: BTreeMap<String, BTreeMap<String, String>>,
}

fn read_map(path: &Path) -> Option<BTreeMap<String, String>> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

fn plain_code(code: &str) -> bool {
    (2..=12).contains(&code.len()) && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn maps_in(dir: &Path) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let Some(code) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !plain_code(code) || path.extension().is_none_or(|end| end != "json") {
            continue;
        }
        if let Some(map) = read_map(&path) {
            out.insert(code.to_owned(), map);
        }
    }
    out
}

impl Books {
    /// The strings folder: `{lang}.json` for words, `rows/{lang}.json` for row names. A folder
    /// that cannot be read gives no books, and every word is English.
    #[must_use]
    pub fn read(dir: &Path) -> Self {
        Self {
            texts: maps_in(dir),
            rows: maps_in(&dir.join("rows")),
        }
    }

    /// Books from maps, for tests.
    #[must_use]
    pub fn from_maps(
        texts: BTreeMap<String, BTreeMap<String, String>>,
        rows: BTreeMap<String, BTreeMap<String, String>>,
    ) -> Self {
        Self { texts, rows }
    }
}

/// The server's own words, with this brand's name in them, in one language.
#[derive(Clone, Debug)]
pub struct Words {
    brand: String,
    lang: String,
    rtl: bool,
    books: Arc<Books>,
}

impl Default for Words {
    fn default() -> Self {
        Self::new("")
    }
}

/// A language code reduced to what the strings folder is named by: `ar-SA` reads `ar`.
fn base_code(lang: &str) -> &str {
    lang.split(['-', '_']).next().unwrap_or(lang)
}

impl Words {
    #[must_use]
    pub fn new(brand: &str) -> Self {
        Self {
            brand: brand.to_owned(),
            lang: String::from(ENGLISH_CODE),
            rtl: false,
            books: Arc::new(Books::default()),
        }
    }

    /// These words, reading every language from `books`.
    #[must_use]
    pub fn with_books(mut self, books: Arc<Books>) -> Self {
        self.books = books;
        self
    }

    /// The same words in another language. An unknown language speaks English.
    #[must_use]
    pub fn speaking(&self, lang: &str) -> Self {
        let code = base_code(lang).to_lowercase();
        let lang = if code == ENGLISH_CODE || self.books.texts.contains_key(&code) {
            code
        } else {
            String::from(ENGLISH_CODE)
        };
        // The language's own file says which way it runs (`language.dir`).
        let rtl = self
            .books
            .texts
            .get(&lang)
            .and_then(|book| book.get("language.dir"))
            .is_some_and(|dir| dir == "rtl");
        Self {
            brand: self.brand.clone(),
            lang,
            rtl,
            books: self.books.clone(),
        }
    }

    /// The books these words read from, to keep them when the brand changes.
    #[must_use]
    pub fn books(&self) -> Arc<Books> {
        self.books.clone()
    }

    /// The language these words are in.
    #[must_use]
    pub fn lang(&self) -> &str {
        &self.lang
    }

    fn branded(&self, text: &str) -> String {
        self.marked(text.replace(PLACEHOLDER_BRAND, &self.brand).trim())
    }

    /// A line of a right to left language opened with the right to left mark; any other unchanged.
    fn marked(&self, text: &str) -> String {
        if self.rtl && !text.starts_with(RIGHT_TO_LEFT_MARK) {
            format!("{RIGHT_TO_LEFT_MARK}{text}")
        } else {
            text.to_owned()
        }
    }

    /// True when this language runs right to left.
    #[must_use]
    pub fn rtl(&self) -> bool {
        self.rtl
    }

    /// The text of a key in this language, else in English, if any strings file has it.
    #[must_use]
    pub fn known(&self, key: &str) -> Option<String> {
        let own = self
            .books
            .texts
            .get(&self.lang)
            .and_then(|book| book.get(key));
        let english = || {
            self.books
                .texts
                .get(ENGLISH_CODE)
                .and_then(|book| book.get(key))
                .or_else(|| ENGLISH.get(key))
        };
        own.or_else(english).map(|text| self.branded(text))
    }

    /// The text of a key, or the key itself, which a test then shows as a missing word.
    #[must_use]
    pub fn say(&self, key: &str) -> String {
        self.known(key).unwrap_or_else(|| key.to_owned())
    }

    /// A key's text with its `{name}` places filled, each value isolated.
    #[must_use]
    pub fn fill(&self, key: &str, args: &serde_json::Map<String, Value>) -> Option<String> {
        let mut text = self.known(key)?;
        for (name, value) in args {
            let value = match value {
                Value::String(text) => text.clone(),
                Value::Null => continue,
                other => other.to_string(),
            };
            text = text.replace(&format!("{{{name}}}"), &isolate(&value));
        }
        Some(text)
    }

    /// A name made from the data, in this language when the row names table has it. Names that
    /// carry a title or a genre (`Like {title}`, `More {genre}`) are written from their templates.
    #[must_use]
    pub fn name(&self, english: &str) -> String {
        if self.lang == ENGLISH_CODE {
            return english.to_owned();
        }
        let whole = self
            .books
            .rows
            .get(&self.lang)
            .and_then(|book| book.get(english))
            .filter(|text| !text.trim().is_empty())
            .cloned();
        let shaped = || {
            let one = |key: &str, name: &str, value: String| {
                let mut args = serde_json::Map::new();
                args.insert(name.to_owned(), Value::String(value));
                self.fill(key, &args)
            };
            if let Some(title) = english.strip_prefix("Like ") {
                one("row.like", "title", title.to_owned())
            } else if let Some(genre) = english.strip_prefix("More ") {
                self.known(&genre_key(genre))
                    .and_then(|name| one("row.more_genre", "genre", name))
            } else {
                None
            }
        };
        self.marked(&whole.or_else(shaped).unwrap_or_else(|| english.to_owned()))
    }

    /// A genre's name in this language: the `genre.<name>` key, else the name as the data has it.
    #[must_use]
    pub fn genre(&self, english: &str) -> String {
        self.known(&genre_key(english))
            .unwrap_or_else(|| english.to_owned())
    }

    /// A row's title: its key's text with the arguments given (`args` overrides the row's own),
    /// else the row's name translated whole, else the name as made.
    #[must_use]
    pub fn row_title(
        &self,
        meta: Option<&Value>,
        title: &str,
        args: Option<&serde_json::Map<String, Value>>,
    ) -> String {
        let key = meta
            .and_then(|meta| meta.get("title_key"))
            .and_then(Value::as_str);
        let own = meta
            .and_then(|meta| meta.get("title_args"))
            .and_then(Value::as_object);
        let empty = serde_json::Map::new();
        key.and_then(|key| self.fill(key, args.or(own).unwrap_or(&empty)))
            .unwrap_or_else(|| self.name(title))
    }
}

/// The strings key of a genre: `genre.` and its name in lower case, words joined by `_`.
#[must_use]
pub fn genre_key(english: &str) -> String {
    let slug: String = english
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let slug = slug
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    format!("genre.{slug}")
}

/// A value isolated for a sentence of either direction.
#[must_use]
pub fn isolate(value: &str) -> String {
    format!("{}{value}{}", ISOLATE.0, ISOLATE.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn books() -> Arc<Books> {
        let mut texts = BTreeMap::new();
        texts.insert(
            String::from("en"),
            BTreeMap::from([
                (
                    String::from("row.more_like"),
                    String::from("More Like {title}"),
                ),
                (
                    String::from("genre.science_fiction"),
                    String::from("Science Fiction"),
                ),
            ]),
        );
        texts.insert(
            String::from("ar"),
            BTreeMap::from([
                (
                    String::from("row.more_like"),
                    String::from("المزيد مثل {title}"),
                ),
                (
                    String::from("genre.science_fiction"),
                    String::from("خيال علمي"),
                ),
            ]),
        );
        let mut rows = BTreeMap::new();
        rows.insert(
            String::from("ar"),
            BTreeMap::from([(
                String::from("Classic Comedies"),
                String::from("كوميديا كلاسيكية"),
            )]),
        );
        Arc::new(Books::from_maps(texts, rows))
    }

    #[test]
    fn the_name_comes_from_the_brand() {
        assert_eq!(Words::new("Sample").say("nav.mine"), "My Sample");
        assert_eq!(Words::new("Other").say("nav.mine"), "My Other");
    }

    #[test]
    fn a_row_without_a_known_key_keeps_its_title() {
        let words = Words::new("Sample");
        let meta = serde_json::json!({ "title_key": "row.my_list" });
        assert_eq!(words.row_title(Some(&meta), "ignored", None), "My List");
        let meta = serde_json::json!({ "title_key": "row.not_in_strings" });
        assert_eq!(words.row_title(Some(&meta), "Kept", None), "Kept");
    }

    #[test]
    fn a_language_speaks_its_own_words_and_english_fills_the_gaps() {
        let words = Words::new("Sample").with_books(books()).speaking("ar-SA");
        assert_eq!(words.lang(), "ar");
        assert_eq!(words.genre("Science Fiction"), "خيال علمي");
        assert_eq!(words.say("nav.mine"), "My Sample");
        assert_eq!(words.name("Classic Comedies"), "كوميديا كلاسيكية");
        assert_eq!(words.name("Heist Thrillers"), "Heist Thrillers");
    }

    #[test]
    fn values_in_a_sentence_are_isolated() {
        let words = Words::new("Sample").with_books(books()).speaking("ar");
        let meta =
            serde_json::json!({ "title_key": "row.more_like", "title_args": { "title": "Up" } });
        assert_eq!(
            words.row_title(Some(&meta), "More Like Up", None),
            "المزيد مثل \u{2068}Up\u{2069}"
        );
    }

    #[test]
    fn a_right_to_left_language_opens_its_lines_with_the_mark() {
        let mut texts = BTreeMap::new();
        texts.insert(
            String::from("ar"),
            BTreeMap::from([
                (String::from("language.dir"), String::from("rtl")),
                (String::from("nav.mine"), String::from("{brand} الخاص بي")),
            ]),
        );
        let books = Arc::new(Books::from_maps(texts, BTreeMap::new()));
        let words = Words::new("Sample").with_books(books).speaking("ar");
        assert!(words.rtl());
        assert_eq!(words.say("nav.mine"), "\u{200F}Sample الخاص بي");
        assert_eq!(words.name("Kept"), "\u{200F}Kept");
    }

    #[test]
    fn an_unknown_language_speaks_english() {
        let words = Words::new("Sample").with_books(books()).speaking("xx");
        assert_eq!(words.lang(), "en");
    }
}
