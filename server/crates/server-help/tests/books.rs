//! A help folder read the way the server reads it.

use std::path::Path;

use contract::help::HelpBlockKind;
use server_help::{Help, Named};

const NAMED: Named<'static> = Named {
    brand: "Sample",
    support_email: "help@host.example",
};

fn help() -> Help {
    Help::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/books"))
        .expect("the folder reads")
}

#[test]
fn english_holds_the_tree_and_the_names_are_filled_in() {
    let home = help().home(None, NAMED).unwrap();
    assert_eq!(home.topics[0].title, "Start here");
    assert_eq!(
        home.topics[0].groups[0].articles[0].summary.as_deref(),
        Some("How Sample starts")
    );
    assert_eq!(
        home.popular.len(),
        1,
        "a popular entry with no article is left out"
    );
    assert_eq!(home.policies[0].title, "Terms");
}

#[test]
fn a_translation_gives_what_it_has_and_english_fills_the_rest() {
    let help = help();
    let home = help.home(Some("de"), NAMED).unwrap();
    assert_eq!(home.topics[0].title, "Start");
    assert_eq!(home.topics[0].groups[0].title, "Basics");
    assert_eq!(home.topics[0].groups[0].articles[0].title, "Erste Schritte");
    assert_eq!(home.topics[0].groups[0].articles[1].title, "Subtitles");
    let article = help.article("first", Some("de"), NAMED).unwrap();
    assert_eq!(article.title, "Erste Schritte");
    assert_eq!(
        article.sections[0].blocks[0].lines[0].runs[0].text,
        "Welcome to Sample."
    );
    assert_eq!(article.related[0].slug, "second");
}

#[test]
fn an_unknown_language_reads_as_english_and_an_unknown_page_as_nothing() {
    let help = help();
    assert_eq!(
        help.article("first", Some("xx"), NAMED).unwrap().title,
        "Getting started"
    );
    assert!(help.article("nothing", None, NAMED).is_none());
    assert!(help.policy("nothing", None, NAMED).is_none());
}

#[test]
fn a_folded_section_holds_its_own_text() {
    let article = help().article("first", None, NAMED).unwrap();
    assert_eq!(article.sections.len(), 2);
    assert!(article.sections[1].folded);
    assert_eq!(
        article.sections[1].blocks[0].lines[0].runs[0].text,
        "Ask help@host.example"
    );
}

#[test]
fn a_policy_keeps_its_text_and_bullets() {
    let policy = help().policy("terms", None, NAMED).unwrap();
    assert_eq!(policy.updated.as_deref(), Some("2026-11-07"));
    let kinds: Vec<_> = policy.sections[0]
        .blocks
        .iter()
        .map(|block| block.kind)
        .collect();
    assert_eq!(kinds, [HelpBlockKind::Paragraph, HelpBlockKind::List]);
}

#[test]
fn search_wants_every_word_and_puts_title_matches_first() {
    let help = help();
    let first = |query| help.search(query, None, NAMED).unwrap().articles;
    assert_eq!(first("subtitles")[0].slug, "second");
    assert_eq!(first("audio")[0].slug, "second");
    assert_eq!(first("started begin")[0].slug, "first");
    assert!(first("started audio").is_empty());
    assert!(first("   ").is_empty());
}

#[test]
fn a_folder_with_no_english_is_refused() {
    let dir = std::env::temp_dir().join(format!("server_help_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("de.json"), "{}").unwrap();
    assert!(Help::open(&dir).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_policy_section_is_found_by_id_and_a_half_written_one_stays_english() {
    let policy = help().policy("terms", Some("de"), NAMED).unwrap();
    let titles: Vec<_> = policy
        .sections
        .iter()
        .map(|section| section.title.as_deref().unwrap_or_default())
        .collect();
    assert_eq!(
        titles,
        ["Use", "Kontakt"],
        "English holds the order; the German section is found by its id, not its position. \
         `use` is half written in German (no bullets where English has two), so it is read whole \
         in English rather than as a German heading over English bullets."
    );
    let words = |at: usize| -> Vec<String> {
        policy.sections[at]
            .blocks
            .iter()
            .flat_map(|block| &block.lines)
            .map(|line| line.runs.iter().map(|run| run.text.as_str()).collect())
            .collect()
    };
    assert_eq!(words(0), ["Plain terms.", "One", "Two"]);
    assert_eq!(words(1), ["Schreiben an help@host.example."]);
}

#[test]
fn a_bullet_is_marked_up_like_any_other_line() {
    let dir = std::env::temp_dir().join(format!("server_help_bullet_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("en.json"),
        r#"{"policies":{"p":{"sections":[{"id":"one","title":"T","body":"B",
           "bullets":["<b>Lead:</b> and the rest"]}]}}}"#,
    )
    .unwrap();
    let help = Help::open(&dir).unwrap();
    std::fs::remove_dir_all(&dir).ok();
    let policy = help.policy("p", None, NAMED).unwrap();
    let list = policy.sections[0].blocks.last().unwrap();
    let runs = &list.lines[0].runs;
    assert_eq!(runs.len(), 2, "the tag opens a bold run, it is not shown");
    assert_eq!(runs[0].text, "Lead:");
    assert!(runs[0].bold);
    assert_eq!(runs[1].text, " and the rest");
}

#[test]
fn a_book_whose_bullets_and_tables_do_not_pair_is_refused() {
    let dir = std::env::temp_dir().join(format!("server_help_tables_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("en.json"),
        r#"{"policies":{"p":{"sections":[{"id":"one","title":"T","body":"B",
           "bullets":["a","b"],"tables":[["t1"]]}]}}}"#,
    )
    .unwrap();
    let refused = Help::open(&dir);
    std::fs::remove_dir_all(&dir).ok();
    assert!(
        refused.is_err(),
        "two bullets and one table list would tie a sentence to the wrong rows"
    );
}
