//! Every JSON session runs with the offline layer on and off, through the JSON boundary.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use kit_core::host::{Common, Mode, Report, Script, run};

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("sessions")
}

fn common() -> Common {
    let text = fs::read_to_string(folder().join("common.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn scripts() -> Vec<Script> {
    let mut paths: Vec<PathBuf> = fs::read_dir(folder())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter(|path| path.file_name().is_some_and(|name| name != "common.json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).unwrap();
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path:?}: {error}"))
        })
        .collect()
}

#[test]
fn every_session_runs_with_the_layer_on_and_off() {
    let common = common();
    let scripts = scripts();
    assert!(scripts.len() >= 40, "only {} sessions", scripts.len());
    let mut failures = Vec::new();
    for script in &scripts {
        for mode in [Mode::LayerOff, Mode::LayerOn] {
            if let Err(error) = run(script, &common, mode) {
                failures.push(format!("[{mode:?}] {error}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} runs failed:\n{}",
        failures.len(),
        scripts.len() * 2,
        failures.join("\n\n")
    );
}

#[test]
fn a_session_gives_the_same_bytes_twice() {
    let common = common();
    for script in scripts() {
        for mode in [Mode::LayerOff, Mode::LayerOn] {
            let first = run(&script, &common, mode).unwrap();
            let second = run(&script, &common, mode).unwrap();
            assert_eq!(first.transcript, second.transcript, "{}", script.name);
        }
    }
}

fn ledger() -> BTreeMap<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contract/ledger.csv");
    let text = fs::read_to_string(path).unwrap();
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split(',');
            let id = parts.next()?.to_string();
            let _kind = parts.next()?;
            let _status = parts.next()?;
            Some((id, parts.next()?.to_string()))
        })
        .collect()
}

#[test]
fn every_session_names_ids_the_ledger_gives_to_the_core_or_the_layer() {
    let ledger = ledger();
    for script in scripts() {
        assert!(!script.covers.is_empty(), "{}", script.name);
        assert!(!script.about.is_empty(), "{}", script.name);
        for id in &script.covers {
            let home = ledger
                .get(id)
                .unwrap_or_else(|| panic!("{} names {id}, not in the ledger", script.name));
            assert!(
                home == "core" || home == "offline",
                "{} names {id}, which lives in {home}",
                script.name
            );
        }
    }
}

fn saw(report: &Report, line: &str) -> usize {
    report.sent.iter().filter(|sent| sent.line == line).count()
}

#[test]
fn a_device_with_no_storage_completes_sign_in_home_and_play() {
    let common = common();
    let scripts = scripts();
    let script = scripts
        .iter()
        .find(|script| script.name == "no_storage_signs_in_opens_home_and_plays")
        .expect("the session exists");
    assert!(!script.device.secret && !script.device.store);
    assert_eq!(script.device.storage_bytes, 0);
    for mode in [Mode::LayerOff, Mode::LayerOn] {
        let report = run(script, &common, mode).unwrap();
        assert_eq!(saw(&report, "POST /v1/auth/password"), 1);
        assert_eq!(saw(&report, "GET /v1/screens/home"), 1);
        assert_eq!(saw(&report, "POST /v1/play"), 1);
        let positions = report
            .ops
            .iter()
            .filter(|op| op["command"] == "position")
            .count();
        assert_eq!(positions, 2, "{mode:?}");
    }
}
