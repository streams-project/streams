//! The words a page carries. They live in `strings/en.json` with every other word, and the name
//! comes from the brand, so neither is typed into the code. A family that shows another language
//! reads the same keys from `/v1/strings/{lang}`.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use contract::hello::PLACEHOLDER_BRAND;
use serde_json::Value;

static ENGLISH: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../strings/en.json")).unwrap_or_default()
});

/// The server's own words, with this brand's name in them.
#[derive(Clone, Debug, Default)]
pub struct Words {
    brand: String,
}

impl Words {
    #[must_use]
    pub fn new(brand: &str) -> Self {
        Self {
            brand: brand.to_owned(),
        }
    }

    /// The text of a key, if the strings file has it.
    #[must_use]
    pub fn known(&self, key: &str) -> Option<String> {
        ENGLISH
            .get(key)
            .map(|text| text.replace(PLACEHOLDER_BRAND, &self.brand).trim().to_owned())
    }

    /// The text of a key, or the key itself, which a test then shows as a missing word.
    #[must_use]
    pub fn say(&self, key: &str) -> String {
        self.known(key).unwrap_or_else(|| key.to_owned())
    }

    /// A row's title: its key's text when the strings file has one.
    #[must_use]
    pub fn row_title(&self, meta: Option<&Value>, title: &str) -> String {
        meta.and_then(|meta| meta.get("title_key"))
            .and_then(Value::as_str)
            .and_then(|key| self.known(key))
            .unwrap_or_else(|| title.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_comes_from_the_brand() {
        assert_eq!(Words::new("Sample").say("nav.mine"), "My Sample");
        assert_eq!(Words::new("Other").say("nav.mine"), "My Other");
    }

    #[test]
    fn a_row_without_a_known_key_keeps_its_title() {
        let words = Words::new("Sample");
        let meta = serde_json::json!({ "title_key": "row.my_list" });
        assert_eq!(words.row_title(Some(&meta), "ignored"), "My List");
        let meta = serde_json::json!({ "title_key": "row.not_in_strings" });
        assert_eq!(words.row_title(Some(&meta), "Kept"), "Kept");
    }
}
