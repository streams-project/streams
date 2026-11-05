//! The generated files. Covers U21, H119, H129, C16, C112, S36, S175, U8.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use contract::api::ROUTES;
use contract_gen::schemas::MAX_SAFE_INTEGER;
use serde_json::Value;

fn contract_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn fixtures() -> Vec<String> {
    contract_gen::fixture_names(&contract_dir()).unwrap()
}

fn files() -> BTreeMap<String, String> {
    contract_gen::generate(&fixtures())
}

fn openapi() -> Value {
    serde_json::from_str(&files()["openapi.json"]).unwrap()
}

fn schemas() -> Value {
    openapi()["components"]["schemas"].clone()
}

fn walk<'a>(value: &'a Value, visit: &mut dyn FnMut(&'a serde_json::Map<String, Value>)) {
    match value {
        Value::Object(map) => {
            visit(map);
            map.values().for_each(|inner| walk(inner, visit));
        }
        Value::Array(items) => items.iter().for_each(|inner| walk(inner, visit)),
        _ => {}
    }
}

fn read_tree(folder: &Path, base: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            read_tree(&path, base, into);
        } else {
            let name = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            into.insert(name, fs::read(&path).unwrap());
        }
    }
}

fn run_generator(folder: &Path) -> BTreeMap<String, Vec<u8>> {
    let _ = fs::remove_dir_all(folder);
    let status = Command::new(env!("CARGO_BIN_EXE_contract-gen"))
        .arg("write")
        .arg(folder)
        .status()
        .unwrap();
    assert!(status.success());
    let mut tree = BTreeMap::new();
    read_tree(folder, folder, &mut tree);
    tree
}

#[test]
fn generating_twice_in_two_processes_gives_no_diff() {
    let base = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let first = run_generator(&base.join("generate-one"));
    let second = run_generator(&base.join("generate-two"));
    assert!(first.len() > 80, "{}", first.len());
    assert_eq!(first, second);
}

#[test]
fn committed_files_match_what_the_contract_generates() {
    let problems = contract_gen::differences(&files(), &contract_dir().join("generated")).unwrap();
    assert!(problems.is_empty(), "run contract-gen write: {problems:?}");
}

#[test]
fn u21_openapi_lists_every_route_and_no_other() {
    let document = openapi();
    let mut listed = BTreeSet::new();
    for (path, item) in document["paths"].as_object().unwrap() {
        for method in item.as_object().unwrap().keys() {
            listed.insert((method.to_uppercase(), path.clone()));
        }
    }
    let table: BTreeSet<(String, String)> = ROUTES
        .iter()
        .map(|route| (route.method.to_string(), route.path.to_string()))
        .collect();
    assert_eq!(table.len(), ROUTES.len(), "a route is listed twice");
    assert_eq!(listed, table);
}

#[test]
fn every_operation_name_is_unique_and_every_operation_can_fail_with_a_code() {
    let mut names = BTreeSet::new();
    for route in ROUTES {
        assert!(names.insert(route.name), "{}", route.name);
    }
    for item in openapi()["paths"].as_object().unwrap().values() {
        for operation in item.as_object().unwrap().values() {
            let schema =
                &operation["responses"]["default"]["content"]["application/json"]["schema"];
            assert_eq!(schema["$ref"], "#/components/schemas/ErrorBody");
        }
    }
}

#[test]
fn openapi_is_version_3_1_and_names_no_server_address() {
    let document = openapi();
    assert_eq!(document["openapi"], "3.1.0");
    assert!(document.get("servers").is_none());
}

#[test]
fn every_reference_resolves() {
    let document = openapi();
    let mut targets = Vec::new();
    walk(&document, &mut |map| {
        if let Some(Value::String(target)) = map.get("$ref") {
            targets.push(target.clone());
        }
    });
    assert!(targets.len() > 100);
    for target in targets {
        let pointer = target.trim_start_matches('#');
        assert!(document.pointer(pointer).is_some(), "{target}");
    }
}

#[test]
fn every_schema_file_stands_alone() {
    let all = files();
    let list: Vec<&String> = all
        .keys()
        .filter(|path| path.starts_with("schemas/"))
        .collect();
    assert!(list.len() > 80);
    for path in list {
        let schema: Value = serde_json::from_str(&all[path]).unwrap();
        assert!(schema["$schema"].is_string(), "{path}");
        let mut targets = Vec::new();
        walk(&schema, &mut |map| {
            if let Some(Value::String(target)) = map.get("$ref") {
                targets.push(target.clone());
            }
        });
        for target in targets {
            assert!(target.starts_with("#/$defs/"), "{path}: {target}");
            let found = schema.pointer(target.trim_start_matches('#'));
            assert!(found.is_some(), "{path}: {target}");
        }
    }
}

#[test]
fn every_example_points_at_a_fixture_that_exists() {
    let generated = contract_dir().join("generated");
    let mut count = 0;
    walk(&openapi(), &mut |map| {
        if let Some(Value::String(target)) = map.get("externalValue") {
            assert!(generated.join(target).is_file(), "{target}");
            count += 1;
        }
    });
    assert!(count > 150, "{count}");
}

#[test]
fn s36_s175_u8_no_route_answers_video_or_a_stream() {
    for route in ROUTES {
        for word in ["media", "proxy", "stream", "segment", "manifest"] {
            assert!(!route.path.contains(word), "{}", route.path);
        }
        if let Some(form) = route.text_form {
            assert_eq!(form, "text/vtt", "{}", route.path);
        }
    }
    for item in openapi()["paths"].as_object().unwrap().values() {
        for operation in item.as_object().unwrap().values() {
            let answers = operation["responses"].as_object().unwrap();
            for answer in answers.values() {
                let Some(content) = answer.get("content").and_then(Value::as_object) else {
                    continue;
                };
                for media_type in content.keys() {
                    assert!(
                        media_type == "application/json" || media_type == "text/vtt",
                        "{media_type}"
                    );
                }
            }
        }
    }
}

#[test]
fn c16_no_wire_type_carries_a_provider_name() {
    let all = schemas();
    walk(&all, &mut |map| {
        if let Some(Value::Object(properties)) = map.get("properties") {
            for name in properties.keys() {
                assert!(!name.contains("provider"), "{name}");
            }
        }
    });
    let attempt = &all["Attempt"];
    let required: Vec<&str> = attempt["required"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for field in ["url", "kind", "group", "headers", "verified"] {
        assert!(required.contains(&field), "{field}");
    }
    for field in ["container", "codecs", "height", "audio", "expires_at_ms"] {
        assert!(!required.contains(&field), "{field}");
        assert!(attempt["properties"].get(field).is_some(), "{field}");
    }
}

#[test]
fn c112_no_type_a_screen_reads_can_hold_a_token() {
    let all = schemas();
    let mut seen = BTreeSet::from(["View".to_string()]);
    let mut queue = vec!["View".to_string()];
    while let Some(name) = queue.pop() {
        walk(&all[name.as_str()], &mut |map| {
            if let Some(Value::Object(properties)) = map.get("properties") {
                for (field, shape) in properties {
                    assert!(!field.contains("token"), "{name}.{field}");
                    assert!(!field.contains("secret"), "{name}.{field}");
                    // A minimum length reaches the sign in screen, and a number holds no password.
                    let a_length =
                        name == "Limits" && field == "password_min" && shape["type"] == "integer";
                    assert!(!field.contains("password") || a_length, "{name}.{field}");
                }
            }
            if let Some(Value::String(target)) = map.get("$ref") {
                let next = target.rsplit('/').next().unwrap().to_string();
                if seen.insert(next.clone()) {
                    queue.push(next);
                }
            }
        });
    }
    assert!(seen.len() > 5, "{seen:?}");
}

#[test]
fn effects_are_the_eight_kinds_and_an_http_effect_holds_a_path_only() {
    let all = schemas();
    let options = all["Effect"]["oneOf"].as_array().unwrap();
    let kinds: BTreeSet<&str> = options
        .iter()
        .map(|option| option["properties"]["kind"]["const"].as_str().unwrap())
        .collect();
    let wanted = [
        "http", "log", "open", "player", "secret", "store", "timer", "view",
    ];
    assert_eq!(kinds, BTreeSet::from(wanted));
    let http = options
        .iter()
        .find(|option| option["properties"]["kind"]["const"] == "http")
        .unwrap();
    assert!(http["properties"].get("path").is_some());
    assert!(http["properties"].get("url").is_none());
}

#[test]
fn no_float_appears_and_every_integer_is_bounded() {
    let all = schemas();
    let mut integers = 0;
    walk(&all, &mut |map| {
        let kind = map.get("type").and_then(Value::as_str);
        assert_ne!(kind, Some("number"));
        if kind == Some("integer") {
            integers += 1;
            assert!(map.get("format").is_none());
            let top = map["maximum"].as_u64().unwrap();
            assert!(top <= MAX_SAFE_INTEGER, "{top}");
        }
    });
    assert!(integers > 50, "{integers}");
}

#[test]
fn unknown_fields_stay_allowed_and_null_is_never_written() {
    let all = schemas();
    walk(&all, &mut |map| {
        assert_ne!(map.get("additionalProperties"), Some(&Value::Bool(false)));
        assert_ne!(map.get("type").and_then(Value::as_str), Some("null"));
    });
}

#[test]
fn the_unknown_catch_all_is_not_a_word_a_server_may_send() {
    walk(&schemas(), &mut |map| {
        assert_ne!(map.get("const").and_then(Value::as_str), Some("unknown"));
        if let Some(Value::Array(values)) = map.get("enum") {
            assert!(!values.iter().any(|value| value == "unknown"));
        }
    });
}

#[test]
fn h57_h91_the_schema_of_a_title_key_demands_a_kind() {
    let all = schemas();
    let pattern = all["TitleKey"]["pattern"].as_str().unwrap();
    assert_eq!(pattern, "^(movie|tv):[1-9][0-9]{0,9}$");
    assert_eq!(all["TitleKey"]["type"], "string");
}

#[test]
fn h119_h4_the_version_is_a_date_and_hello_publishes_a_range() {
    let document = openapi();
    assert_eq!(document["info"]["version"], contract::version::WIRE_VERSION);
    let hello = &document["components"]["schemas"]["Hello"];
    assert_eq!(
        hello["properties"]["contract"]["$ref"],
        "#/components/schemas/VersionRange"
    );
    assert!(hello["properties"].get("client_min").is_some());
    let header = &document["components"]["parameters"]["ApiVersion"];
    assert_eq!(header["name"], contract::version::API_VERSION_HEADER);
}

#[test]
fn the_three_type_files_name_every_definition() {
    let all = files();
    for name in schemas().as_object().unwrap().keys() {
        for path in [
            "types/contract.ts",
            "types/Contract.kt",
            "types/Contract.swift",
        ] {
            assert!(all[path].contains(name.as_str()), "{path}: {name}");
        }
    }
}

#[test]
fn h57_h91_a_bare_id_is_refused_in_the_three_client_languages() {
    let all = files();
    assert!(all["types/contract.ts"].contains("`movie:${number}` | `tv:${number}`"));
    assert!(all["types/Contract.kt"].contains("value class TitleKey(val value: String)"));
    assert!(all["types/Contract.kt"].contains("^(movie|tv):[1-9][0-9]{0,9}\\$"));
    assert!(all["types/Contract.swift"].contains("ContractError.refused(\"TitleKey\")"));
    assert!(!all["types/Contract.swift"].contains("typealias TitleKey"));
    assert!(!all["types/Contract.kt"].contains("typealias TitleKey"));
}
