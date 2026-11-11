//! OpenAPI 3.1 from the route table, the definitions and the fixtures. No server address: the
//! operator holds it.

use std::collections::BTreeMap;

use contract::TITLE_ID_PATTERN;
use contract::api::{Auth, CLIENT_HEADERS, Group, ROUTES, Route};
use contract::version::{API_VERSION_HEADER, WIRE_VERSION};
use serde_json::{Map, Value, json};

use crate::schemas::DEFS_PATH;

const JSON: &str = "application/json";

/// Fixture path as seen from `generated/openapi.json`.
fn fixture_path(name: &str) -> String {
    format!("../fixtures/{WIRE_VERSION}/valid/{name}")
}

fn type_ref(name: &str) -> Value {
    json!({ "$ref": format!("#{DEFS_PATH}/{name}") })
}

fn examples(type_name: &str, fixtures: &[String]) -> Map<String, Value> {
    let prefix = format!("{type_name}.");
    fixtures
        .iter()
        .filter(|name| name.starts_with(&prefix))
        .map(|name| {
            let case = name[prefix.len()..].trim_end_matches(".json").to_string();
            (case, json!({ "externalValue": fixture_path(name) }))
        })
        .collect()
}

fn json_body(type_name: &str, fixtures: &[String]) -> Value {
    json!({
        "schema": type_ref(type_name),
        "examples": examples(type_name, fixtures),
    })
}

const fn tag(group: Group) -> &'static str {
    match group {
        Group::Start => "start",
        Group::Auth => "auth",
        Group::Account => "account",
        Group::Profiles => "profiles",
        Group::Pages => "pages",
        Group::Commands => "commands",
        Group::Play => "play",
        _ => "finder",
    }
}

fn security(auth: Auth) -> Value {
    match auth {
        Auth::Member => json!([{ "member": [] }]),
        Auth::Profile => json!([{ "profile": [] }]),
        Auth::Device => json!([{ "device": [] }]),
        Auth::Service => json!([{ "service": [] }]),
        _ => json!([]),
    }
}

fn path_schema(route: &Route, name: &str) -> Value {
    match name {
        "kind" => type_ref("TitleKind"),
        "name" => type_ref("ScreenName"),
        "n" => json!({ "type": "integer", "minimum": 0 }),
        "id" if route.path.contains("/titles/") => {
            json!({ "type": "string", "pattern": format!("^{TITLE_ID_PATTERN}$") })
        }
        _ => json!({ "type": "string" }),
    }
}

fn path_names(path: &str) -> impl Iterator<Item = &str> {
    path.split('/').filter_map(|part| {
        part.strip_prefix('{')
            .and_then(|part| part.strip_suffix('}'))
    })
}

fn parameters(route: &Route) -> Vec<Value> {
    let mut list: Vec<Value> = Vec::new();
    if route.group != Group::Finder {
        list.push(json!({ "$ref": "#/components/parameters/ApiVersion" }));
        for (index, _) in CLIENT_HEADERS.iter().enumerate() {
            list.push(json!({ "$ref": format!("#/components/parameters/Client{index}") }));
        }
    }
    for name in path_names(route.path) {
        list.push(json!({
            "name": name,
            "in": "path",
            "required": true,
            "schema": path_schema(route, name),
        }));
    }
    for query in route.query {
        list.push(json!({
            "name": query.name,
            "in": "query",
            "required": query.required,
            "description": query.about,
            "schema": { "type": "string" },
        }));
    }
    list
}

fn responses(route: &Route, fixtures: &[String]) -> Map<String, Value> {
    let mut content = Map::new();
    content.insert(JSON.into(), json_body(route.response, fixtures));
    if let Some(text) = route.text_form {
        content.insert(text.into(), json!({ "schema": { "type": "string" } }));
    }
    let mut answers = Map::new();
    answers.insert(
        "200".into(),
        json!({ "description": route.about, "content": content }),
    );
    if route.cacheable {
        answers.insert(
            "304".into(),
            json!({ "description": "Not changed since the ETag in If-None-Match." }),
        );
    }
    answers.insert(
        "default".into(),
        json!({
            "description": "An error with a stable code.",
            "content": { JSON: json_body("ErrorBody", fixtures) },
        }),
    );
    answers
}

fn operation(route: &Route, fixtures: &[String]) -> Value {
    let mut item = Map::new();
    item.insert("operationId".into(), json!(route.name));
    item.insert("summary".into(), json!(route.about));
    item.insert("tags".into(), json!([tag(route.group)]));
    item.insert("security".into(), security(route.auth));
    item.insert("parameters".into(), json!(parameters(route)));
    if let Some(request) = route.request {
        item.insert(
            "requestBody".into(),
            json!({
                "required": true,
                "content": { JSON: json_body(request, fixtures) },
            }),
        );
    }
    item.insert(
        "responses".into(),
        Value::Object(responses(route, fixtures)),
    );
    Value::Object(item)
}

fn paths(fixtures: &[String]) -> Map<String, Value> {
    let mut paths: BTreeMap<&str, Map<String, Value>> = BTreeMap::new();
    for route in ROUTES {
        paths
            .entry(route.path)
            .or_default()
            .insert(route.method.to_lowercase(), operation(route, fixtures));
    }
    paths
        .into_iter()
        .map(|(path, item)| (path.to_string(), Value::Object(item)))
        .collect()
}

fn header_parameters() -> Map<String, Value> {
    let mut list = Map::new();
    list.insert(
        "ApiVersion".into(),
        json!({
            "name": API_VERSION_HEADER,
            "in": "header",
            "required": true,
            "description": "The wire date this client speaks.",
            "schema": { "$ref": format!("#{DEFS_PATH}/WireDate") },
        }),
    );
    for (index, (name, about)) in CLIENT_HEADERS.iter().enumerate() {
        list.insert(
            format!("Client{index}"),
            json!({
                "name": name,
                "in": "header",
                "required": true,
                "description": about,
                "schema": { "type": "string" },
            }),
        );
    }
    list
}

/// The whole document.
#[must_use]
pub fn document(defs: &BTreeMap<String, Value>, fixtures: &[String]) -> Value {
    let bearer = |about: &str| json!({ "type": "http", "scheme": "bearer", "description": about });
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Open video platform",
            "version": WIRE_VERSION,
            "license": { "name": "MIT OR Apache-2.0", "identifier": "MIT OR Apache-2.0" },
        },
        "tags": [
            { "name": "start" }, { "name": "auth" }, { "name": "account" },
            { "name": "profiles" }, { "name": "pages" }, { "name": "commands" },
            { "name": "play" }, { "name": "finder", "description": "Private, service token." },
        ],
        "paths": paths(fixtures),
        "components": {
            "schemas": defs,
            "parameters": header_parameters(),
            "securitySchemes": {
                "member": bearer("An opaque member token."),
                "profile": bearer("An opaque member token on a session with an active profile."),
                "device": bearer("The token of a hosted device."),
                "service": bearer("The service token between server and finder."),
            },
        },
    })
}
