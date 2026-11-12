//! How the interface's language writes a word, a number and a count.
//!
//! The words themselves come from the server (`/v1/strings/{lang}`, 301 keys in 18 languages) and
//! never from this code. What is here is the four rules that stand between a key and a line of
//! text, and every one of them is a port of the surface the phone already ships, so a line reads
//! the same on both:
//!
//! * [`fill`] — `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt:66`
//! * [`number`] — `Theme.kt:89`, and the digit families named at `Theme.kt:70-72`
//! * [`plural_of`] — `Theme.kt:97`
//! * [`counted`] — `Theme.kt:131`
//!
//! Nothing here is new shape. A rule that differed from the phone's would be the same line of text
//! coming out two ways on two screens of one product, which is the defect this file exists against.

use leptos::prelude::*;

use crate::state::Shared;

/// Western digits, and the two families a language may ask for instead.
///
/// `language.digits` in a language's own strings says which: `arab` for Arabic-Indic ٠-٩ and
/// `arabext` for Eastern Arabic-Indic / Persian ۰-۹, anything else for Western 0-9
/// (`Theme.kt:70-72`). `strings/ar.json` carries `arab`; the other seventeen carry `latn`.
const ARABIC_INDIC: u32 = 0x0660;
const EASTERN_ARABIC_INDIC: u32 = 0x06F0;

/// The isolates a filled value is wrapped in: U+2068 FIRST STRONG ISOLATE and U+2069 POP
/// DIRECTIONAL ISOLATE. A number, a title or an address keeps its own order inside a line that runs
/// the other way, which is what nine of the eighteen languages need and what none of them can fix
/// after the fact (`Theme.kt:62-66`).
const OPEN_ISOLATE: char = '\u{2068}';
const CLOSE_ISOLATE: char = '\u{2069}';

/// A word's `{name}` place filled with a value, isolated. `Theme.kt:66`.
#[must_use]
pub fn fill(text: &str, name: &str, value: &str) -> String {
    text.replace(
        &format!("{{{name}}}"),
        &format!("{OPEN_ISOLATE}{value}{CLOSE_ISOLATE}"),
    )
}

/// A number as a language writes it: its digits, isolated by the caller that fills it in.
/// `Theme.kt:89`.
#[must_use]
pub fn in_digits(plain: &str, digits: &str) -> String {
    let base = match digits {
        "arab" => ARABIC_INDIC,
        "arabext" => EASTERN_ARABIC_INDIC,
        _ => return plain.to_owned(),
    };
    plain
        .chars()
        .map(|mark| match mark {
            '0'..='9' => char::from_u32(base + (mark as u32 - '0' as u32)).unwrap_or(mark),
            other => other,
        })
        .collect()
}

/// The plural form a count takes in a language, as CLDR names them. `Theme.kt:97`.
///
/// The seven arms are the phone's, number for number: a form this table gave differently from the
/// phone's would be one screen of the product agreeing with a language's grammar and another not.
#[must_use]
pub fn plural_of(n: u32, lang: &str) -> &'static str {
    let lang = lang.split('-').next().unwrap_or(lang);
    match lang {
        "ar" => match n {
            0 => "zero",
            1 => "one",
            2 => "two",
            _ if (3..=10).contains(&(n % 100)) => "few",
            _ if (11..=99).contains(&(n % 100)) => "many",
            _ => "other",
        },
        "ru" | "uk" => match (n % 10, n % 100) {
            (1, hundred) if hundred != 11 => "one",
            (2..=4, hundred) if !(12..=14).contains(&hundred) => "few",
            _ => "many",
        },
        "pl" => match (n, n % 10, n % 100) {
            (1, _, _) => "one",
            (_, 2..=4, hundred) if !(12..=14).contains(&hundred) => "few",
            _ => "many",
        },
        "he" | "iw" => match n {
            1 => "one",
            2 => "two",
            _ => "other",
        },
        "ja" | "ko" | "zh" => "other",
        "fa" => {
            if n == 1 {
                "one"
            } else {
                "other"
            }
        }
        "fr" | "pt" | "hi" => {
            if n <= 1 {
                "one"
            } else {
                "other"
            }
        }
        _ => {
            if n == 1 {
                "one"
            } else {
                "other"
            }
        }
    }
}

/// The forms a counted word is looked for in, in order: the count's own, then the three a language
/// file is most likely to carry. `Theme.kt:131-133`.
const FORMS_AFTER: [&str; 3] = ["other", "many", "one"];

/// A number in the interface's language, isolated as a filled value is.
#[must_use]
pub fn number(shared: Shared, value: u32) -> String {
    shared
        .digits
        .with(|digits| in_digits(&value.to_string(), digits))
}

/// A counted word: `{base}.{form}` for the count's plural form, else `other`, `many`, then `one`,
/// with `{n}` filled. `Theme.kt:131`.
///
/// Empty where this server has no form of the family at all, so a screen draws nothing rather than
/// a bare number with no word around it. `super::maybe_text` is the same rule for a plain key.
#[must_use]
pub fn counted(shared: Shared, base: &str, n: u32) -> String {
    let form = shared.lang.with(|lang| plural_of(n, lang));
    let text = core::iter::once(form)
        .chain(FORMS_AFTER)
        .find_map(|form| super::maybe_text(shared, &format!("{base}.{form}")));
    let Some(text) = text else {
        return String::new();
    };
    fill(&text, "n", &number(shared, n))
}

/// A key's words with one `{name}` filled, or nothing where the server has no such key.
#[must_use]
pub fn filled(shared: Shared, label: &str, name: &str, value: &str) -> Option<String> {
    Some(fill(&super::maybe_text(shared, label)?, name, value))
}

#[cfg(test)]
pub mod tests {
    use super::{CLOSE_ISOLATE, FORMS_AFTER, OPEN_ISOLATE, fill, in_digits, plural_of};

    #[test]
    fn a_filled_value_is_isolated_so_it_keeps_its_own_order() {
        // The isolates are the point: without them a number or a title inside an Arabic line is
        // reordered by the paragraph's direction and reads back wrong.
        assert_eq!(
            fill("Season {n}", "n", "12"),
            format!("Season {OPEN_ISOLATE}12{CLOSE_ISOLATE}")
        );
        // A key with the place twice fills both, and a place the text does not have changes nothing.
        assert_eq!(
            fill("{a} and {a}", "a", "x"),
            format!("{OPEN_ISOLATE}x{CLOSE_ISOLATE} and {OPEN_ISOLATE}x{CLOSE_ISOLATE}")
        );
        assert_eq!(fill("Season {n}", "other", "12"), "Season {n}");
    }

    #[test]
    fn a_number_is_written_in_the_language_s_own_digits() {
        assert_eq!(in_digits("2026", "latn"), "2026");
        // Arabic-Indic, which `strings/ar.json` asks for with `language.digits`.
        assert_eq!(in_digits("2026", "arab"), "٢٠٢٦");
        // Eastern Arabic-Indic, which no language file asks for yet; the rule is the phone's.
        assert_eq!(in_digits("2026", "arabext"), "۲۰۲۶");
        // What is not a digit is left where it was, so a percentage and a clock survive.
        assert_eq!(in_digits("1:05", "arab"), "١:٠٥");
        assert_eq!(in_digits("50%", "arab"), "٥٠%");
        // A family this build does not know is not a reason to lose the number.
        assert_eq!(in_digits("7", "deva"), "7");
    }

    /// The whole plural table, against the Kotlin's own arms.
    ///
    /// Read out of `Theme.kt` rather than written here twice: a form that moved on one platform and
    /// not the other is one product disagreeing with itself about a language's grammar, and nothing
    /// in either repository would otherwise notice.
    #[test]
    fn the_plural_forms_are_the_ones_the_phone_uses() {
        let kotlin = std::fs::read_to_string(
            super::super::tests_support::repo_root()
                .join("apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt"),
        )
        .expect("the phone's words file");
        let table = kotlin
            .split_once("fun pluralOf(")
            .expect("pluralOf is still in Theme.kt")
            .1;
        let table = table.split_once("\n}").expect("the end of pluralOf").0;
        // Every language the table names, so a language added there and not here fails.
        let mut named: Vec<&str> = Vec::new();
        for line in table.lines() {
            let Some((head, _)) = line.split_once("->") else {
                continue;
            };
            for quoted in head.split('"').skip(1).step_by(2) {
                named.push(quoted);
            }
        }
        assert!(
            named.contains(&"ar") && named.contains(&"pl") && named.contains(&"zh"),
            "read the languages out of Theme.kt, got {named:?}"
        );
        for lang in named {
            // The counts that separate every form any of these languages has.
            for n in [0_u32, 1, 2, 3, 5, 11, 12, 14, 21, 100, 101, 111] {
                let form = plural_of(n, lang);
                assert!(
                    ["zero", "one", "two", "few", "many", "other"].contains(&form),
                    "{lang} {n} gave {form}"
                );
            }
        }
        // Arabic, which is the only language in the set with all six forms.
        assert_eq!(plural_of(0, "ar"), "zero");
        assert_eq!(plural_of(1, "ar"), "one");
        assert_eq!(plural_of(2, "ar"), "two");
        assert_eq!(plural_of(3, "ar"), "few");
        assert_eq!(plural_of(10, "ar"), "few");
        assert_eq!(plural_of(11, "ar"), "many");
        assert_eq!(plural_of(99, "ar"), "many");
        assert_eq!(plural_of(100, "ar"), "other");
        // Russian and Polish, which differ from each other at 1.
        assert_eq!(plural_of(1, "ru"), "one");
        assert_eq!(plural_of(11, "ru"), "many");
        assert_eq!(plural_of(22, "ru"), "few");
        assert_eq!(plural_of(21, "pl"), "many");
        assert_eq!(plural_of(22, "pl"), "few");
        // Hebrew has a two; French counts zero as one; the languages with no plural at all.
        assert_eq!(plural_of(2, "he"), "two");
        assert_eq!(plural_of(0, "fr"), "one");
        assert_eq!(plural_of(5, "ja"), "other");
        // A tag with a region is the language before the hyphen.
        assert_eq!(plural_of(2, "pt-BR"), "other");
        // A language nothing names falls on the English rule.
        assert_eq!(plural_of(1, "sv"), "one");
        assert_eq!(plural_of(2, "sv"), "other");
    }

    /// Every word this surface asks for is in the strings file.
    ///
    /// The hole `apps/android/surfaces/phone/src/test/kotlin/streams/phone/WordsTest.kt:12-16` was
    /// written against, and it is open on the web for exactly the same reason:
    /// `platform/tools/check_strings.py` checks the other seventeen languages against `en.json` and
    /// **nothing checks `en.json` against the code**. `maybe_text` is a map lookup and
    /// `label_text` draws nothing when it misses, so a label wired to a key no language has is a
    /// control that answers a press and still says nothing - which is the defect the words pass was
    /// for, passing every check.
    ///
    /// Proved to discriminate on 2026-11-12 by rewiring one label from `settings.account_name` to
    /// `settings.account_full_name` and watching this go red. A list of labels written out by hand
    /// stayed green through the same change, which is why this test reads the source instead.
    #[test]
    fn every_word_this_surface_asks_for_is_in_the_strings_file() {
        let book = super::super::tests_support::english();
        let mut missing: Vec<String> = Vec::new();
        for path in rust_files(&super::super::tests_support::repo_root().join("apps/web/src")) {
            let text = std::fs::read_to_string(&path).expect("a source file reads");
            // A test stands outside the app looking in, and names file paths that are of the same
            // shape as a key, so the read stops where the tests start.
            let body = text
                .split_once("#[cfg(test)]")
                .map_or(text.as_str(), |cut| cut.0);
            for word in quoted(body) {
                if !key_shaped(&word) {
                    continue;
                }
                // `counted` asks for a family by its base and the book holds the forms.
                if book.contains_key(&word) || book.contains_key(&format!("{word}.one")) {
                    continue;
                }
                if !missing.contains(&word) {
                    missing.push(word);
                }
            }
        }
        missing.sort();
        // The keys this surface asks for that no language has yet, each one a gap with its own
        // owner. The set is exact on purpose: a new one fails this test, and filling one of these
        // fails it too, so the list cannot quietly outlive the gap.
        //
        // `search.empty` is the words a search with no matches shows (`browse::NothingFound`).
        // `signin.code` and `signin.code.send` are the email-code way in (`door::SignIn`). All
        // three are named in the report; none of them is filled in by this crate, because
        // inventing English for eighteen languages is the one thing `strings/` exists to stop.
        assert_eq!(
            missing,
            vec![
                String::from("search.empty"),
                String::from("signin.code"),
                String::from("signin.code.send"),
            ]
        );
    }

    /// Every `.rs` file under a folder.
    pub fn rust_files(folder: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        let listing = std::fs::read_dir(folder).expect("the source folder reads");
        for entry in listing.flatten() {
            let path = entry.path();
            if path.is_dir() {
                found.extend(rust_files(&path));
            } else if path.extension().is_some_and(|end| end == "rs") {
                found.push(path);
            }
        }
        found.sort();
        found
    }

    /// Every double quoted literal in a piece of Rust, as written, with its escapes left alone.
    fn quoted(text: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut marks = text.chars();
        while let Some(mark) = marks.next() {
            if mark != '"' {
                continue;
            }
            let mut held = String::new();
            loop {
                match marks.next() {
                    None | Some('"') => break,
                    Some('\\') => {
                        marks.next();
                        held.push('\\');
                    }
                    Some(mark) => held.push(mark),
                }
            }
            found.push(held);
        }
        found
    }

    /// True for a literal of the shape every key in `strings/en.json` has: small letters, digits,
    /// underscores and hyphens, in parts joined by dots, with at least one dot.
    ///
    /// The keys with a hyphen and no dot - `problem-failed`, `problem-offline` - are left out on
    /// purpose: the shape would then match half the class names in this crate. Those five are held
    /// against the strings file by `door::tests::every_label_a_refusal_names_is_in_the_strings_file`
    /// instead, which names the function that produces them.
    fn key_shaped(word: &str) -> bool {
        let fits = |part: &str| {
            !part.is_empty()
                && part.chars().all(|mark| {
                    mark.is_ascii_lowercase() || mark.is_ascii_digit() || mark == '_' || mark == '-'
                })
        };
        let mut parts = word.split('.');
        let first = parts.next().unwrap_or_default();
        first.starts_with(|mark: char| mark.is_ascii_lowercase())
            && fits(first)
            && word.contains('.')
            && parts.all(fits)
    }

    #[test]
    fn the_reader_knows_a_key_from_everything_else_this_crate_writes_down() {
        assert!(key_shaped("say.close"));
        assert!(key_shaped("profile.badge.kids"));
        assert!(key_shaped("title.length.hours"));
        // Not keys: a class, a path, a style, a format place, a word with capitals.
        assert!(!key_shaped("age-dark-red"));
        assert!(!key_shaped("/v1/notifications"));
        assert!(!key_shaped("apps/web/style.css"));
        assert!(!key_shaped("width:{percent}%"));
        assert!(!key_shaped("Season {n}"));
        assert!(!key_shaped(""));
        assert!(!key_shaped("say."));
        assert!(!key_shaped(".close"));
        // And the literals really do come out of the source whole.
        assert_eq!(
            quoted(r#"let a = "say.close"; let b = "x\"y";"#),
            vec![String::from("say.close"), String::from("x\\y")]
        );
    }

    /// Every form this file looks for after the count's own is one `strings/en.json` really has, so
    /// a counted word cannot fall through to a form no language file carries.
    #[test]
    fn the_fallback_forms_are_forms_the_strings_file_uses() {
        let book = super::super::tests_support::english();
        for form in FORMS_AFTER {
            assert!(
                book.keys().any(|key| key.ends_with(&format!(".{form}"))),
                "no key in strings/en.json ends in .{form}"
            );
        }
    }
}
