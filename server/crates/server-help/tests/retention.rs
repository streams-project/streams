//! The retention list of the privacy policy, against the one list of what an account owns.
//!
//! The policy's "how long we keep it" section is not written from memory: every bullet names the
//! tables it accounts for, and this check reads `server-accounts/src/holdings.rs` to prove the two
//! agree. A table added to the schema reaches that list (its own test sees to that) and then
//! fails here until the policy says how long it lives. The file is read as text, the way
//! `holdings.rs` itself reads the migrations, so no crate edge is added for a check.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use server_help::{Help, Named};

const NAMED: Named<'static> = Named {
    brand: "{brand}",
    support_email: "{support_email}",
};

fn held() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../server-accounts/src/holdings.rs")
}

/// Every table of the inventory with the life it is declared to have.
fn inventory() -> BTreeMap<String, String> {
    let text = std::fs::read_to_string(held()).expect("the inventory of what an account owns");
    let mut found = BTreeMap::new();
    let mut table: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("table: \"") {
            table = rest.split('"').next().map(str::to_owned);
        }
        if let Some(rest) = line.strip_prefix("lives: Lives::")
            && let Some(name) = table.take()
        {
            let life = rest.trim_end_matches(',').to_owned();
            found.insert(name, life);
        }
    }
    assert!(
        found.len() > 20,
        "the inventory did not read: {} tables",
        found.len()
    );
    found
}

fn retention() -> (Help, Vec<String>, Vec<Vec<String>>) {
    let help = Help::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../help"))
        .expect("the real help folder");
    let policy = help
        .policy("privacy", None, NAMED)
        .expect("the privacy policy");
    let section = policy
        .sections
        .iter()
        .find(|section| section.title.as_deref() == Some("How long we keep it"))
        .expect("the retention section");
    let lines = section
        .blocks
        .iter()
        .flat_map(|block| &block.lines)
        .map(|line| {
            line.runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>()
        })
        .collect::<Vec<String>>();
    let tables = help.tables("privacy", "keep");
    (help, lines, tables)
}

#[test]
fn n2_every_table_the_server_holds_is_named_by_exactly_one_retention_bullet() {
    let inventory = inventory();
    let (_, _, tables) = retention();
    let mut named: Vec<String> = tables.iter().flatten().cloned().collect();
    let unique: BTreeSet<String> = named.iter().cloned().collect();
    named.sort();
    assert_eq!(
        named.len(),
        unique.len(),
        "a table is named twice: {named:?}"
    );
    let holds: BTreeSet<String> = inventory.keys().cloned().collect();
    let missing: Vec<&String> = holds.difference(&unique).collect();
    assert!(
        missing.is_empty(),
        "the policy does not say how long these live: {missing:?}"
    );
    let unknown: Vec<&String> = unique.difference(&holds).collect();
    assert!(
        unknown.is_empty(),
        "the policy names tables the server does not hold: {unknown:?}"
    );
}

#[test]
fn n2_a_retention_bullet_states_the_life_of_the_tables_it_names() {
    let inventory = inventory();
    let (_, lines, tables) = retention();
    // The bullets are the lines of the list block, which follow the paragraphs of the body.
    let bullets = &lines[lines.len() - tables.len()..];
    for (bullet, group) in bullets.iter().zip(&tables) {
        for table in group {
            let life = &inventory[table];
            let words = match life.as_str() {
                "WhileHeld" => vec!["while the account exists", "while the profile exists"],
                "UntilSpent" => vec!["until it is used"],
                "WithTheBuild" => vec!["No person is named"],
                other => {
                    let number = other
                        .trim_start_matches("Days(")
                        .trim_start_matches("Newest(")
                        .trim_end_matches(')');
                    assert!(
                        bullet.contains(number),
                        "{table} lives {other} and its bullet does not say {number}: {bullet}"
                    );
                    continue;
                }
            };
            assert!(
                words.iter().any(|phrase| bullet.contains(phrase)),
                "{table} lives {life} and its bullet does not say so: {bullet}"
            );
        }
    }
}
