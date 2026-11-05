//! Blocked names: a normalised match on case, accents and punctuation.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Lower case, no accents, letters and digits only, single spaces.
#[must_use]
pub fn normalise_name(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = true;
    for c in text.nfkd().filter(|c| !is_combining_mark(*c)) {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            space = false;
        } else if !space {
            out.push(' ');
            space = true;
        }
    }
    out.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmk13_case_accents_and_punctuation_do_not_matter() {
        assert_eq!(normalise_name("Café  Münster!"), "cafe munster");
        assert_eq!(normalise_name("CAFE-MUNSTER"), "cafe munster");
        assert_eq!(normalise_name(" Ａｍｅｌｉｅ "), "amelie");
    }
}
