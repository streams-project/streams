//! The message boundary of the core.

use contract::version::WIRE_VERSION;
use platform_core::{BadMessage, Core, Step, step_json};
use serde_json::{Value, json};

const CONFIG: &str = r#"{"app":"a","class":"web","contract":"2026-11-05","form":"desktop",
  "mode":"local","os":"o","seed":"00112233445566778899aabbccddeeff","type":"core_config",
  "version":"1.0.0"}"#;

fn input(mono_ms: u64, events: &Value) -> String {
    json!({
        "contract": WIRE_VERSION,
        "type": "core_in",
        "now_ms": 1_793_836_800_000_u64 + mono_ms,
        "mono_ms": mono_ms,
        "events": events,
    })
    .to_string()
}

fn step(core: &mut Core, mono_ms: u64, events: &Value) -> Value {
    let text = step_json(core, &input(mono_ms, events)).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn core() -> Core {
    Core::open_json(CONFIG).unwrap()
}

#[test]
fn a_seed_must_be_at_least_32_hex_digits() {
    for bad in ["", "00ff", "zz112233445566778899aabbccddeeff"] {
        let text = CONFIG.replace("00112233445566778899aabbccddeeff", bad);
        assert_eq!(Core::open_json(&text).err(), Some(BadMessage), "{bad}");
    }
    assert_eq!(Core::open_json("{}").err(), Some(BadMessage));
}

#[test]
fn the_first_step_asks_for_the_kept_secrets_and_hello_and_shows_a_wait() {
    let out = step(&mut core(), 0, &json!([]));
    let kinds: Vec<&str> = out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|effect| effect["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["secret", "secret", "http", "open"]);
    assert_eq!(out["effects"][0]["name"], "device");
    assert_eq!(out["effects"][1]["name"], "session");
    assert_eq!(out["effects"][2]["path"], "/v1/hello");
    assert_eq!(out["effects"][2]["class"], "cacheable");
    assert_eq!(out["view"]["route"]["target"], "wait");
    assert_eq!(out["view"].get("session"), None);
}

#[test]
fn a_message_that_does_not_read_is_refused_and_changes_nothing() {
    let mut core = core();
    assert_eq!(step_json(&mut core, "not json"), Err(BadMessage));
    assert_eq!(step_json(&mut core, "{}"), Err(BadMessage));
    let out = step(&mut core, 0, &json!([]));
    assert_eq!(out["effects"].as_array().unwrap().len(), 4);
}

#[test]
fn an_unknown_event_is_answered_unsupported_and_an_unknown_action_too() {
    let mut core = core();
    step(&mut core, 0, &json!([]));
    let out = step(
        &mut core,
        1,
        &json!([
            {"id": "x1", "type": "from_a_later_contract", "extra": 1},
            {"id": "x2", "type": "action", "action": {"action": "from_a_later_contract"}},
        ]),
    );
    assert_eq!(out["unsupported"], json!(["x1", "x2"]));
}

#[test]
fn an_action_with_no_rule_yet_says_so_and_is_never_swallowed() {
    let mut core = core();
    step(&mut core, 0, &json!([]));
    let out = step(
        &mut core,
        1,
        &json!([{"id": "x1", "type": "action", "action": {"action": "next_subtitle_file"}}]),
    );
    assert_eq!(out["unsupported"], json!(["x1"]));
}

#[test]
fn c5_a_cancelled_hello_or_secret_never_leaves_the_core_waiting() {
    let mut core = core();
    let first = step(&mut core, 0, &json!([]));
    let session_id = first["effects"][1]["id"].clone();
    let hello_id = first["effects"][2]["id"].clone();
    let out = step(
        &mut core,
        1,
        &json!([
            {"id": session_id, "type": "cancelled"},
            {"id": hello_id, "type": "cancelled"},
        ]),
    );
    assert_eq!(out["view"]["session"]["state"], "signed_out");
    assert_eq!(out["next_wake_ms"], 2_001);
}

#[test]
fn a_timer_fired_is_a_plain_wake_and_the_core_arms_no_timer_of_its_own() {
    let mut core = core();
    step(&mut core, 0, &json!([]));
    let out = step(&mut core, 1, &json!([{"id": "e99", "type": "timer_fired"}]));
    assert_eq!(out.get("unsupported"), None);
    assert_eq!(out["effects"], json!([]));
}

#[test]
fn effect_ids_never_repeat_and_only_the_core_side_prefix_is_used() {
    let mut core = core();
    let mut ids = Vec::new();
    for mono_ms in 0..3 {
        let events = json!([{"type": "address", "address": "/nowhere"}]);
        let out = step(&mut core, mono_ms, &events);
        for effect in out["effects"].as_array().unwrap() {
            ids.push(effect["id"].as_str().unwrap().to_string());
        }
    }
    let count = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), count);
    assert!(ids.iter().all(|id| id.starts_with('e')));
}

#[test]
fn step_takes_typed_messages_for_a_host_written_in_rust() {
    let mut core = core();
    let typed = serde_json::from_str(&input(0, &json!([]))).unwrap();
    let out = core.step(typed);
    assert_eq!(out.effects.len(), 4);
}

#[test]
fn c131_an_address_waits_while_the_kept_session_is_unknown_even_with_hello_in_hand() {
    let hello = include_str!("../../contract/fixtures/2026-11-05/valid/Hello.basic.json");
    let mut core = core();
    let first = step(&mut core, 0, &json!([]));
    let hello_id = first["effects"][2]["id"].clone();
    let session_id = first["effects"][1]["id"].clone();
    // a host answers hello first and the secret later
    let out = step(
        &mut core,
        1,
        &json!([
            {"id": hello_id, "type": "http_done", "status": 200, "body": hello},
            {"type": "address", "address": "/my-list"},
        ]),
    );
    assert_eq!(out["view"]["route"], Value::Null, "{out}");
    let out = step(
        &mut core,
        2,
        &json!([{"id": session_id, "type": "secret_done"}]),
    );
    assert_eq!(out["view"]["route"]["target"], "sign_in");
    assert_eq!(out["view"]["route"]["next"], "/my-list");
}
