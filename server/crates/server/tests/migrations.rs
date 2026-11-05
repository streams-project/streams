//! The forward chain, checked without a database: it parses, it only goes forward, no rule hides in
//! the database, and every table and column the stores name exists. Applying it to a real Postgres
//! needs `TEST_DATABASE_URL`; see `server-accounts/tests/postgres.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use sqlparser::ast::{CreateTable, Statement};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every migration, in chain order, as name and text.
fn chain() -> Vec<(String, String)> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("migrations"))
        .expect("migrations folder")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(path).expect("migration"))
        })
        .collect()
}

fn statements(sql: &str) -> Vec<Statement> {
    Parser::parse_sql(&PostgreSqlDialect {}, sql).expect("the migration parses")
}

/// Columns by table, from the whole chain.
fn tables() -> BTreeMap<String, BTreeSet<String>> {
    let mut found = BTreeMap::new();
    for (_, sql) in chain() {
        for statement in statements(&sql) {
            if let Statement::CreateTable(CreateTable { name, columns, .. }) = statement {
                let columns = columns.iter().map(|c| c.name.value.clone()).collect();
                found.insert(name.to_string(), columns);
            }
        }
    }
    found
}

#[test]
fn g5_every_file_parses_and_the_numbers_only_rise() {
    let files = chain();
    assert!(files.len() >= 3);
    let mut last = 0u32;
    for (name, sql) in &files {
        let (number, rest) = name.split_once('_').expect("number and name");
        let number: u32 = number.parse().expect("a number first");
        assert!(number > last, "{name} does not follow {last}");
        assert!(rest.ends_with(".sql"));
        last = number;
        assert!(!statements(sql).is_empty(), "{name} is empty");
    }
}

#[test]
fn g5_a_migration_only_adds_so_the_previous_release_keeps_working() {
    let removal =
        Regex::new(r"(?i)\b(drop\s+(table|column|index)|rename\s+(to|column)|truncate)\b").unwrap();
    for (name, sql) in chain() {
        assert!(!removal.is_match(&sql), "{name} removes something");
    }
}

#[test]
fn h26_h31_no_rule_hides_in_the_database() {
    let banned = Regex::new(
        r"(?i)\b(create\s+(or\s+replace\s+)?(function|trigger|policy|procedure)|row\s+level\s+security|cron\.|auth\.uid|auth\.jwt|security\s+definer)\b",
    )
    .unwrap();
    for (name, sql) in chain() {
        assert!(
            !banned.is_match(&sql),
            "{name} holds a database rule, and rules live in Rust"
        );
    }
}

#[test]
fn a_project_that_takes_only_accounts_stops_before_any_video_word() {
    let video =
        Regex::new(r"(?i)\b(title|movie|episode|season|series|film|video|trailer|tmdb)").unwrap();
    for (name, sql) in chain() {
        let number: u32 = name.split('_').next().unwrap().parse().unwrap();
        if number < 100 {
            for line in sql
                .lines()
                .filter(|line| !line.trim_start().starts_with("--"))
            {
                assert!(!video.is_match(line), "{name}: {line}");
            }
        }
    }
}



/// Every store source file.
fn store_sources() -> Vec<String> {
    [
        "crates/server-accounts/src/postgres.rs",
        "crates/server-library/src/postgres.rs",
    ]
    .iter()
    .map(|path| std::fs::read_to_string(root().join(path)).expect("store source"))
    .collect()
}

#[test]
fn every_table_a_store_names_exists() {
    let tables = tables();
    let named = Regex::new(r"(?i)\b(?:from|into|update|join)\s+([a-z_]+)\b").unwrap();
    let queries = Regex::new(r#"(?s)"((?:select|insert|update|delete)[^"]*)""#).unwrap();
    let words = [
        "the", "a", "this", "one", "memory", "rows", "disk", "any", "every", "an", "session",
    ];
    for source in store_sources() {
        for query in queries.captures_iter(&source) {
            for table in named.captures_iter(&query[1]) {
                let table = &table[1];
                if table == "auth" || table == "set" || words.contains(&table) {
                    continue;
                }
                assert!(
                    tables.contains_key(table),
                    "no table {table} in: {}",
                    &query[1]
                );
            }
        }
    }
}

#[test]
fn every_column_a_store_writes_exists() {
    let tables = tables();
    let inserts = Regex::new(r"(?is)insert into ([a-z_]+)\s*\(([^)]*)\)").unwrap();
    let updates = Regex::new(r"(?is)update ([a-z_]+) set (.*?)(?: where |$)").unwrap();
    let assigned = Regex::new(r"(?:^|,)\s*\\?\s*([a-z_]+)\s*=").unwrap();
    for source in store_sources() {
        for found in inserts.captures_iter(&source) {
            let columns = tables
                .get(&found[1])
                .unwrap_or_else(|| panic!("no table {}", &found[1]));
            for column in found[2]
                .split(',')
                .map(|c| c.trim().trim_matches('\\').trim())
            {
                if !column.is_empty() {
                    assert!(columns.contains(column), "{}.{column}", &found[1]);
                }
            }
        }
        for found in updates.captures_iter(&source) {
            let Some(columns) = tables.get(&found[1]) else {
                continue;
            };
            let set = found[2].replace("\\\n", " ");
            for column in assigned.captures_iter(&set) {
                assert!(columns.contains(&column[1]), "{}.{}", &found[1], &column[1]);
            }
        }
    }
}
