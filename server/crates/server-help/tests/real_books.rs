//! The real `help/` folder, read the way the server reads it.
//!
//! Nothing here uses a fixture. Every assertion is about the 17 books the product ships, because
//! the defect this file exists for could not be seen in a two-article fixture: the English privacy
//! policy had 14 sections and every translation had 8, and the sections were paired by position,
//! so a reader of any language but English was served one book's heading over another book's
//! words. Ledger N2: help and legal text comes from the server.

use std::path::{Path, PathBuf};

use server_help::{Gap, Help, Named};

/// With the placeholders as the names, filling is the identity, so a served line can be looked
/// for in the book file exactly as it is written there.
const NAMED: Named<'static> = Named {
    brand: "{brand}",
    support_email: "{support_email}",
};

const POLICIES: &[&str] = &[
    "cookie-policy",
    "credits",
    "disclaimer",
    "dmca",
    "privacy",
    "terms",
];

/// Every policy a person is read in their own language, whole or not at all.
const WHOLE: &[&str] = &["privacy", "cookie-policy"];

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../help")
}

fn help() -> Help {
    Help::open(&folder()).expect("the real help folder reads")
}

fn languages() -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(folder())
        .expect("the help folder")
        .filter_map(|entry| {
            let path = entry.expect("an entry").path();
            (path.extension()? == "json").then(|| path.file_stem()?.to_str().map(str::to_owned))?
        })
        .collect();
    found.sort();
    found
}

/// One book on its own, read as if it were the English one. What it serves from there is what it
/// holds and nothing else, so it is the only honest expectation for a section served in that
/// language.
fn alone(code: &str) -> Help {
    let dir = std::env::temp_dir().join(format!("help_alone_{}_{code}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a folder");
    std::fs::copy(folder().join(format!("{code}.json")), dir.join("en.json")).expect("the book");
    let one = Help::open(&dir).expect("one book reads");
    std::fs::remove_dir_all(&dir).ok();
    one
}

#[test]
fn n2_the_folder_holds_every_shipped_language() {
    let codes = languages();
    assert!(codes.contains(&"en".to_owned()), "English is the tree");
    assert_eq!(codes.len(), 17, "17 books ship: {codes:?}");
    let help = help();
    for code in &codes {
        for policy in POLICIES {
            assert!(
                help.policy(policy, Some(code), NAMED).is_some(),
                "{code} serves {policy}"
            );
        }
    }
}

#[test]
fn n2_a_served_policy_section_comes_whole_from_one_book() {
    let help = help();
    let english = alone("en");
    for code in languages() {
        let own = alone(&code);
        for policy in POLICIES {
            let served = help.policy(policy, Some(&code), NAMED).expect("a policy");
            let from_english = english.policy(policy, None, NAMED).expect("the English");
            let from_own = own.policy(policy, None, NAMED);
            assert_eq!(
                served.sections.len(),
                from_english.sections.len(),
                "{code}/{policy}: English holds the order and the count"
            );
            for (at, section) in served.sections.iter().enumerate() {
                let english_section = &from_english.sections[at];
                // The section this book would serve on its own, found by its heading, because a
                // short book has its sections at other positions.
                let own_section = from_own.as_ref().and_then(|book| {
                    book.sections
                        .iter()
                        .find(|one| one.title == section.title && !one.blocks.is_empty())
                });
                let one_book = section == english_section || Some(section) == own_section;
                assert!(
                    one_book,
                    "{code}/{policy} section {at} ({:?}) is not whole from one book",
                    section.title
                );
            }
        }
    }
}

#[test]
fn n2_the_privacy_and_cookie_policies_are_read_in_every_language() {
    let gaps: Vec<Gap> = help()
        .gaps()
        .into_iter()
        .filter(|gap| WHOLE.contains(&gap.policy.as_str()))
        .collect();
    assert!(
        gaps.is_empty(),
        "every reader gets these two in their own words: {gaps:?}"
    );
}

#[test]
fn n2_no_book_is_half_written_and_none_holds_a_section_english_lacks() {
    let bad: Vec<Gap> = help()
        .gaps()
        .into_iter()
        .filter(|gap| gap.reason != "missing")
        .collect();
    assert!(
        bad.is_empty(),
        "a half written section is what the by-position pairing turned into a scramble: {bad:?}"
    );
}

#[test]
fn n2_what_is_still_english_is_only_ever_a_whole_section() {
    // The books that are short of a section are read in English for that section. This is the
    // list, and it is allowed to shrink, not to grow unnoticed.
    let gaps = help().gaps();
    for gap in &gaps {
        assert_eq!(gap.reason, "missing", "{gap:?}");
        assert!(
            !WHOLE.contains(&gap.policy.as_str()),
            "{gap:?} is one of the two every reader must have"
        );
    }
    assert!(
        gaps.len() <= 140,
        "{} sections are still English; the number is falling, not rising",
        gaps.len()
    );
}

/// The English articles corrected in this wave. Their translations still describe facilities the
/// product does not have, so those bodies are served in English until they are written again.
const CORRECTED: &[&str] = &[
    "change-email-password",
    "clear-data",
    "cookies-explained",
    "delete-account",
    "devices-sign-out",
    "download-data",
    "kids-profile",
    "need-account",
    "privacy-data",
    "recommendations",
    "saved-data",
    "sign-in",
    "transfer-profile",
    "viewing-history",
];

#[test]
fn n2_a_translation_older_than_the_english_it_names_is_not_served() {
    let help = help();
    let stale = help.stale();
    assert!(
        !stale.is_empty(),
        "the books carry fingerprints, so the corrected articles must show here"
    );
    for (language, slug) in &stale {
        assert!(
            CORRECTED.contains(&slug.as_str()),
            "{language}/{slug}: the English body changed and this translation was not written \
             again. Translate it, or add the slug to CORRECTED with a line in the report."
        );
    }
    // And what is stale is read in English, not in the old words.
    let english = help
        .article("download-data", None, NAMED)
        .expect("the English article");
    for (language, slug) in stale {
        if slug != "download-data" {
            continue;
        }
        let served = help
            .article(&slug, Some(&language), NAMED)
            .expect("the article");
        assert_eq!(
            served.sections, english.sections,
            "{language}/{slug} is served in English while its translation is older"
        );
    }
}
