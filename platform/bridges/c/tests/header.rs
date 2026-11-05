//! The header is written by hand, because the header tool is not a dependency here. This test
//! reads the header and both bridges and fails when one signature differs. Covers H110 H112.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const NAMES: [&str; 6] = [
    "core_abi",
    "core_alloc",
    "core_free",
    "core_open",
    "core_step",
    "core_close",
];

fn bridges() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// One function: its name, its parameters by name and neutral type, and its return.
#[derive(Debug, PartialEq, Eq)]
struct Signature {
    name: String,
    params: Vec<(String, String)>,
    returns: String,
}

fn neutral_c(text: &str) -> String {
    match text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .as_str()
    {
        "void" => "nothing",
        "uint32_t" => "u32",
        "int32_t" => "i32",
        "size_t" => "size",
        "size_t *" => "size out",
        "uint8_t *" => "bytes",
        "const uint8_t *" => "bytes read",
        "uint8_t **" => "bytes out",
        "CoreHandle *" => "handle",
        other => panic!("no meaning for C type {other}"),
    }
    .to_string()
}

fn neutral_rust(text: &str) -> String {
    match text.trim() {
        "" => "nothing",
        "u32" => "u32",
        "i32" => "i32",
        "usize" => "size",
        "*mut usize" => "size out",
        "*mut u8" => "bytes",
        "*const u8" => "bytes read",
        "*mut *mut u8" => "bytes out",
        "*mut CoreHandle" => "handle",
        other => panic!("no meaning for Rust type {other}"),
    }
    .to_string()
}

fn split_c_param(param: &str) -> (String, String) {
    let param = param.trim();
    let at = param
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map_or(0, |at| at + 1);
    (param[at..].to_string(), neutral_c(&param[..at]))
}

fn header_signatures(text: &str) -> Vec<Signature> {
    let mut found = Vec::new();
    for line in text.lines() {
        let Some(at) = NAMES
            .iter()
            .filter_map(|name| line.find(&format!("{name}(")))
            .min()
        else {
            continue;
        };
        if line.trim_start().starts_with('*') || line.trim_start().starts_with("/*") {
            continue;
        }
        let open = at + line[at..].find('(').unwrap();
        let close = line.rfind(')').unwrap();
        let inner = &line[open + 1..close];
        let params = if inner.trim() == "void" {
            Vec::new()
        } else {
            inner.split(',').map(split_c_param).collect()
        };
        found.push(Signature {
            name: line[at..open].to_string(),
            params,
            returns: neutral_c(&line[..at]),
        });
    }
    found
}

fn bridge_signatures(text: &str) -> Vec<Signature> {
    let mut found = Vec::new();
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut rest = flat.as_str();
    while let Some(at) = rest.find("extern \"C\" fn ") {
        rest = &rest[at + "extern \"C\" fn ".len()..];
        let open = rest.find('(').unwrap();
        let close = rest.find(')').unwrap();
        let params = rest[open + 1..close]
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| {
                let (name, kind) = part.split_once(':').unwrap();
                (name.trim().to_string(), neutral_rust(kind))
            })
            .collect();
        let tail = &rest[close + 1..];
        let body = tail.find('{').unwrap();
        let returns = tail[..body].trim().strip_prefix("->").unwrap_or("");
        found.push(Signature {
            name: rest[..open].to_string(),
            params,
            returns: neutral_rust(returns),
        });
    }
    found
}

fn constants_in_header(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| line.strip_prefix("#define CORE_"))
        .filter_map(|line| line.split_once(' '))
        .map(|(name, value)| (name.to_string(), value.trim().to_string()))
        .collect()
}

fn constants_in_bridge(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub const "))
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            let value = value.split_once('=')?.1.trim().trim_end_matches(';');
            Some((name.trim().to_string(), value.to_string()))
        })
        .collect();
    found.sort();
    found
}

fn header() -> String {
    fs::read_to_string(bridges().join("c/core.h")).unwrap()
}

fn differences(header: &str, bridge: &str) -> Result<(), String> {
    let mut from_header = header_signatures(header);
    let mut from_bridge = bridge_signatures(bridge);
    from_header.sort_by(|a, b| a.name.cmp(&b.name));
    from_bridge.sort_by(|a, b| a.name.cmp(&b.name));
    if from_header != from_bridge {
        return Err(format!("header {from_header:#?}\nbridge {from_bridge:#?}"));
    }
    let mut wanted = constants_in_header(header);
    wanted.sort();
    if wanted != constants_in_bridge(bridge) {
        return Err(format!(
            "header {wanted:?}\nbridge {:?}",
            constants_in_bridge(bridge)
        ));
    }
    Ok(())
}

#[test]
fn h110_the_header_names_exactly_the_six_functions() {
    let found: Vec<String> = header_signatures(&header())
        .into_iter()
        .map(|signature| signature.name)
        .collect();
    assert_eq!(found, NAMES);
}

#[test]
fn h110_h112_the_header_matches_the_c_bridge() {
    let bridge = fs::read_to_string(bridges().join("c/src/lib.rs")).unwrap();
    differences(&header(), &bridge).unwrap();
}

#[test]
fn h110_h112_the_header_matches_the_wasm_bridge() {
    let bridge = fs::read_to_string(bridges().join("wasm/src/lib.rs")).unwrap();
    differences(&header(), &bridge).unwrap();
}

#[test]
fn a_changed_signature_in_either_place_fails_the_compare() {
    let bridge = fs::read_to_string(bridges().join("c/src/lib.rs")).unwrap();
    let header = header();
    for (from, to) in [
        ("size_t cfg_len", "uint32_t cfg_len"),
        ("int32_t core_step", "uint32_t core_step"),
        ("uint8_t **out_ptr_out", "uint8_t *out_ptr_out"),
        ("in_len", "len"),
        ("#define CORE_ABI 1", "#define CORE_ABI 2"),
        ("#define CORE_BAD_CALL 3", "#define CORE_BAD_CALL 4"),
    ] {
        assert!(header.contains(from), "{from}");
        let changed = header.replace(from, to);
        assert!(
            differences(&changed, &bridge).is_err(),
            "{from} went unseen"
        );
    }
    for (from, to) in [
        ("cfg_len: usize", "cfg_len: u32"),
        ("in_ptr: *const u8", "in_ptr: *mut u8"),
    ] {
        assert!(bridge.contains(from), "{from}");
        let changed = bridge.replace(from, to);
        assert!(
            differences(&header, &changed).is_err(),
            "{from} went unseen"
        );
    }
}

#[test]
fn the_header_compiles_as_strict_c_and_as_cplusplus() {
    let path = bridges().join("c/core.h");
    for args in [
        vec![
            "-x",
            "c",
            "-std=c99",
            "-Wall",
            "-Wextra",
            "-Wpedantic",
            "-Werror",
        ],
        vec!["-x", "c++", "-std=c++11", "-Wall", "-Wextra", "-Werror"],
    ] {
        let output = Command::new("cc")
            .args(&args)
            .arg("-fsyntax-only")
            .arg(&path)
            .output()
            .expect("a C compiler runs");
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
