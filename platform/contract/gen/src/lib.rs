//! Writes the files generated from the contract. Same input, same bytes.

pub mod kotlin;
pub mod openapi;
pub mod schemas;
pub mod shape;
pub mod swift;
pub mod typescript;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use contract::version::WIRE_VERSION;
use serde_json::{Value, json};

use crate::schemas::DEFS_PATH;

/// Pretty JSON with a final newline. Object keys are sorted by the map type.
fn pretty(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).expect("a Value always prints");
    text.push('\n');
    text
}

fn reachable(start: &Value, defs: &BTreeMap<String, Value>, seen: &mut BTreeSet<String>) {
    match start {
        Value::Object(map) => {
            if let Some(Value::String(target)) = map.get("$ref") {
                let name = target.rsplit('/').next().unwrap_or_default().to_string();
                if seen.insert(name.clone()) {
                    reachable(&defs[&name], defs, seen);
                }
            }
            map.values().for_each(|inner| reachable(inner, defs, seen));
        }
        Value::Array(items) => items.iter().for_each(|inner| reachable(inner, defs, seen)),
        _ => {}
    }
}

fn point_at_local_defs(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(target)) = map.get_mut("$ref") {
                *target = target.replace(&format!("#{DEFS_PATH}/"), "#/$defs/");
            }
            map.values_mut().for_each(point_at_local_defs);
        }
        Value::Array(items) => items.iter_mut().for_each(point_at_local_defs),
        _ => {}
    }
}

/// One JSON Schema file that stands alone.
fn standalone(name: &str, defs: &BTreeMap<String, Value>) -> Value {
    let mut seen = BTreeSet::new();
    reachable(&defs[name], defs, &mut seen);
    let mut parts = serde_json::Map::new();
    parts.insert(
        "$schema".into(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    parts.insert("title".into(), json!(name));
    if let Value::Object(map) = &defs[name] {
        parts.extend(map.clone());
    }
    let local: serde_json::Map<String, Value> = seen
        .iter()
        .map(|inner| (inner.clone(), defs[inner].clone()))
        .collect();
    if !local.is_empty() {
        parts.insert("$defs".into(), Value::Object(local));
    }
    let mut schema = Value::Object(parts);
    point_at_local_defs(&mut schema);
    schema
}

/// Names of the valid fixtures of the current wire date, sorted.
///
/// # Errors
/// The fixture folder cannot be read.
pub fn fixture_names(contract_dir: &Path) -> io::Result<Vec<String>> {
    let folder = contract_dir
        .join("fixtures")
        .join(WIRE_VERSION)
        .join("valid");
    let mut names = Vec::new();
    for entry in fs::read_dir(folder)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// Every generated file, by path under `generated/`.
#[must_use]
pub fn generate(fixtures: &[String]) -> BTreeMap<String, String> {
    let (defs, roots) = schemas::definitions();
    let items = shape::items(&defs);
    let mut files = BTreeMap::new();
    files.insert(
        "openapi.json".to_string(),
        pretty(&openapi::document(&defs, fixtures)),
    );
    for root in &roots {
        files.insert(
            format!("schemas/{root}.json"),
            pretty(&standalone(root, &defs)),
        );
    }
    files.insert("types/contract.ts".into(), typescript::write(&items));
    files.insert("types/Contract.kt".into(), kotlin::write(&items));
    files.insert("types/Contract.swift".into(), swift::write(&items));
    files
}

/// Write every file under `out`.
///
/// # Errors
/// A file cannot be written.
pub fn write_all(files: &BTreeMap<String, String>, out: &Path) -> io::Result<()> {
    for (path, text) in files {
        let target = out.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, text)?;
    }
    Ok(())
}

/// Paths under `out` that differ from `files`, or are extra, or are missing.
///
/// # Errors
/// A folder cannot be read.
pub fn differences(files: &BTreeMap<String, String>, out: &Path) -> io::Result<Vec<String>> {
    fn walk(folder: &Path, base: &Path, found: &mut Vec<String>) -> io::Result<()> {
        for entry in fs::read_dir(folder)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(&path, base, found)?;
            } else if let Ok(relative) = path.strip_prefix(base) {
                found.push(relative.to_string_lossy().into_owned());
            }
        }
        Ok(())
    }
    let mut found = Vec::new();
    if out.is_dir() {
        walk(out, out, &mut found)?;
    }
    let mut problems: Vec<String> = found
        .iter()
        .filter(|path| !files.contains_key(*path))
        .map(|path| format!("extra {path}"))
        .collect();
    for (path, text) in files {
        match fs::read_to_string(out.join(path)) {
            Ok(old) if old == *text => {}
            Ok(_) => problems.push(format!("changed {path}")),
            Err(_) => problems.push(format!("missing {path}")),
        }
    }
    problems.sort();
    Ok(problems)
}
