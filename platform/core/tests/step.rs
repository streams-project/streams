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

fn http_paths(out: &Value) -> Vec<String> {
    out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "http")
        .map(|effect| effect["path"].as_str().unwrap().to_string())
        .collect()
}

fn secret_names(out: &Value) -> Vec<String> {
    out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "secret")
        .map(|effect| effect["name"].as_str().unwrap().to_string())
        .collect()
}

fn effect_id(out: &Value, path: &str) -> String {
    out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == "http" && effect["path"] == path)
        .map_or_else(
            || panic!("no request for {path} in {out}"),
            |effect| effect["id"].as_str().unwrap().to_string(),
        )
}

fn tokens(expires_in: u32) -> String {
    json!({
        "user_id": "u1",
        "email": "a@b.example",
        "access_token": "at",
        "refresh_token": "rt",
        "expires_in": expires_in,
    })
    .to_string()
}

fn profile(id: &str) -> Value {
    json!({"id": id, "name": "A", "kids": false, "main": true, "has_pin": false})
}

/// A core that answered no kept session and then signed in, with a token of this life in
/// seconds. The last step's output comes back with it.
fn member(expires_in: u32) -> (Core, Value) {
    let mut core = core();
    let first = step(&mut core, 0, &json!([]));
    let kept = first["effects"][1]["id"].clone();
    step(&mut core, 1, &json!([{"id": kept, "type": "secret_done"}]));
    let out = step(
        &mut core,
        2,
        &json!([{"id": "s1", "type": "action",
                 "action": {"action": "sign_in", "email": "a@b.example", "password": "x"}}]),
    );
    let sign_in = effect_id(&out, "/v1/auth/password");
    let out = step(
        &mut core,
        3,
        &json!([{"id": sign_in, "type": "http_done", "status": 200, "body": tokens(expires_in)}]),
    );
    (core, out)
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
fn c111_a_request_tells_the_host_a_60_s_deadline() {
    let out = step(&mut core(), 0, &json!([]));
    assert_eq!(out["effects"][2]["path"], "/v1/hello");
    assert_eq!(out["effects"][2]["deadline_ms"], 60_000);
}

#[test]
fn c42_a_prepared_request_may_name_only_the_servers_own_routes() {
    let mut core = core();
    step(&mut core, 0, &json!([]));
    let out = step(
        &mut core,
        1,
        &json!([
            {"id": "r1", "type": "action",
             "action": {"action": "request", "method": "GET", "path": "/v2/changes"}},
            {"id": "r2", "type": "action",
             "action": {"action": "request", "method": "GET",
                        "path": "http://host.example/v1/changes"}},
        ]),
    );
    assert!(
        http_paths(&out).is_empty(),
        "a path outside /v1/ never goes out: {out}"
    );
    let out = step(
        &mut core,
        2,
        &json!([{"id": "r3", "type": "action",
                 "action": {"action": "request", "method": "GET", "path": "/v1/changes"}}]),
    );
    assert_eq!(http_paths(&out), ["/v1/changes"]);
    assert_eq!(out["effects"][0]["id"], "r3");
}

/// A screen waits on the id it sent, so a 401 must not change it.
#[test]
fn c7_a_prepared_request_refused_with_401_goes_out_again_under_its_own_id() {
    let (mut core, _) = member(3_600);
    let approve = json!([{"id": "ask-1", "type": "action", "action": {
        "action": "request", "method": "POST", "path": "/v1/auth/device/approve",
        "body": r#"{"user_code":"BCDFGHJK"}"#,
    }}]);
    let out = step(&mut core, 4, &approve);
    assert_eq!(effect_id(&out, "/v1/auth/device/approve"), "ask-1");
    let out = step(
        &mut core,
        5,
        &json!([{"id": "ask-1", "type": "http_done", "status": 401}]),
    );
    assert_eq!(http_paths(&out), ["/v1/auth/refresh"]);
    let refresh = effect_id(&out, "/v1/auth/refresh");
    let out = step(
        &mut core,
        6,
        &json!([{"id": refresh, "type": "http_done", "status": 200, "body": tokens(3_600)}]),
    );
    assert_eq!(effect_id(&out, "/v1/auth/device/approve"), "ask-1");
}

#[test]
fn c5_c71_a_cancelled_refresh_is_answered_so_the_request_waiting_on_it_goes_out() {
    // 60 s of life is inside the 120 s renewal margin, so the first request waits for a refresh
    let (mut core, out) = member(60);
    assert_eq!(http_paths(&out), ["/v1/auth/refresh"]);
    let refresh = effect_id(&out, "/v1/auth/refresh");
    let out = step(&mut core, 4, &json!([{"id": refresh, "type": "cancelled"}]));
    assert_eq!(
        http_paths(&out),
        ["/v1/profiles"],
        "a dropped refresh counts as no answer, so nothing waits for ever: {out}"
    );
}

#[test]
fn a_refresh_with_a_lost_answer_is_tried_again_inside_the_servers_reuse_window() {
    // the first try may have reached the server, which then keeps the token good for only 300 s
    let (mut core, out) = member(60);
    let mut refresh = effect_id(&out, "/v1/auth/refresh");
    let mut clock = 4;
    for wait in [5_000, 20_000, 60_000, 120_000] {
        let out = step(
            &mut core,
            clock,
            &json!([{"id": refresh, "type": "cancelled"}]),
        );
        assert_eq!(out["next_wake_ms"], clock + wait, "{out}");
        clock += wait;
        let out = step(&mut core, clock, &json!([]));
        refresh = effect_id(&out, "/v1/auth/refresh");
    }
    assert!(
        clock < 300_000,
        "every try fell inside the window, at {clock}"
    );
}

#[test]
fn c9_c103_after_sign_out_no_one_is_asked_who_is_watching() {
    let (mut core, out) = member(3_600);
    let list = effect_id(&out, "/v1/profiles");
    let profiles = json!({"profiles": [profile("p1"), profile("p2")]}).to_string();
    step(
        &mut core,
        4,
        &json!([{"id": list, "type": "http_done", "status": 200, "body": profiles}]),
    );
    let out = step(
        &mut core,
        5,
        &json!([{"id": "a1", "type": "action",
                 "action": {"action": "enter_profile", "profile": "p1"}}]),
    );
    let enter = effect_id(&out, "/v1/profiles/p1/enter");
    let entered = json!({"profile": profile("p1"), "home": "home"}).to_string();
    step(
        &mut core,
        6,
        &json!([{"id": enter, "type": "http_done", "status": 200, "body": entered}]),
    );
    step(
        &mut core,
        7,
        &json!([{"id": "a2", "type": "action",
                 "action": {"action": "sign_out", "everywhere": false}}]),
    );
    // 41 minutes away is past a desktop's limit, but the account went with the session
    let back = step(&mut core, 7 + 41 * 60_000, &json!([{"type": "activity"}]));
    assert_eq!(
        back["view"]["gate"],
        Value::Null,
        "signed out, there is no profile to ask about: {back}"
    );
}

#[test]
fn the_sign_in_address_holds_when_hello_arrives_after_it() {
    let mut core = core();
    let first = step(&mut core, 0, &json!([]));
    let hello = effect_id(&first, "/v1/hello");
    let kept = first["effects"][1]["id"].clone();
    step(&mut core, 1, &json!([{"id": kept, "type": "secret_done"}]));
    let out = step(
        &mut core,
        2,
        &json!([{"type": "address", "address": "/login"}]),
    );
    assert_eq!(out["view"]["route"]["target"], "sign_in", "{out}");
    let out = step(
        &mut core,
        3,
        &json!([{"id": hello, "type": "http_failed", "reason": "offline"}]),
    );
    assert_ne!(out["view"]["route"]["target"], "start", "{out}");
}

#[test]
fn search_opens_with_no_page_to_read_so_it_never_lands_on_not_found() {
    let (mut core, _) = member(3_600);
    let out = step(
        &mut core,
        4,
        &json!([{"id": "a1", "type": "action",
                 "action": {"action": "enter_profile", "profile": "p1"}}]),
    );
    let enter = effect_id(&out, "/v1/profiles/p1/enter");
    let entered = json!({"profile": profile("p1"), "home": "home"}).to_string();
    step(
        &mut core,
        5,
        &json!([{"id": enter, "type": "http_done", "status": 200, "body": entered}]),
    );
    let out = step(
        &mut core,
        6,
        &json!([{"type": "address", "address": "/search"}]),
    );
    assert_eq!(out["view"]["route"]["screen"], "search", "{out}");
    assert!(
        !http_paths(&out)
            .iter()
            .any(|path| path.starts_with("/v1/screens/")),
        "{out}"
    );
}

#[test]
fn p35_a_title_waiting_in_the_address_survives_the_profile_pick() {
    let (mut core, out) = member(3_600);
    assert_eq!(out["view"]["route"]["target"], "profiles");
    step(
        &mut core,
        4,
        &json!([{"type": "address", "address": "/browse?title=tv:9"}]),
    );
    let out = step(
        &mut core,
        5,
        &json!([{"id": "a1", "type": "action",
                 "action": {"action": "enter_profile", "profile": "p1"}}]),
    );
    let enter = effect_id(&out, "/v1/profiles/p1/enter");
    let entered = json!({"profile": profile("p1"), "home": "home"}).to_string();
    let out = step(
        &mut core,
        6,
        &json!([{"id": enter, "type": "http_done", "status": 200, "body": entered}]),
    );
    assert_eq!(out["view"]["route"]["target"], "home");
    assert_eq!(out["view"]["route"]["title"], "tv:9", "{out}");
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
            {"type": "address", "address": "/browse/my-list"},
        ]),
    );
    assert_eq!(out["view"]["route"], Value::Null, "{out}");
    let out = step(
        &mut core,
        2,
        &json!([{"id": session_id, "type": "secret_done"}]),
    );
    assert_eq!(out["view"]["route"]["target"], "sign_in");
    assert_eq!(out["view"]["route"]["next"], "/browse/my-list");
}

#[test]
fn h1_h106_c132_a_host_that_cannot_keep_a_secret_is_not_asked_again() {
    let mut core = core();
    let first = step(&mut core, 0, &json!([]));
    assert_eq!(first["effects"][0]["name"], "device");
    let device = first["effects"][0]["id"].clone();
    let out = step(
        &mut core,
        1,
        &json!([{"id": device, "type": "unsupported"}]),
    );
    assert!(
        secret_names(&out).is_empty(),
        "a host with no secret store is asked once, not for ever: {out}"
    );
    // the kept session answer cannot be written back either, and nothing waits on it
    let out = step(
        &mut core,
        2,
        &json!([{"id": "s9", "type": "action",
                 "action": {"action": "sign_in", "email": "a@b.example", "password": "x"}}]),
    );
    let sign_in = effect_id(&out, "/v1/auth/password");
    let out = step(
        &mut core,
        3,
        &json!([{"id": sign_in, "type": "http_done", "status": 200, "body": tokens(3_600)}]),
    );
    assert!(secret_names(&out).is_empty(), "{out}");
    assert_eq!(out["view"]["session"]["state"], "signed_in");
}

#[test]
fn c117_h53_a_command_needs_a_profile_in_use() {
    let press = json!({"id": "c1", "type": "action",
                       "action": {"action": "command",
                                  "command": {"command": "list", "title": "movie:7",
                                              "in_list": true}}});
    let (mut core, _) = member(3_600);
    let out = step(&mut core, 4, &json!([press]));
    assert!(
        http_paths(&out).is_empty(),
        "an op belongs to a profile, and none is in use: {out}"
    );
    let out = step(
        &mut core,
        5,
        &json!([{"id": "a1", "type": "action",
                 "action": {"action": "enter_profile", "profile": "p1"}}]),
    );
    let enter = effect_id(&out, "/v1/profiles/p1/enter");
    let entered = json!({"profile": profile("p1"), "home": "home"}).to_string();
    step(
        &mut core,
        6,
        &json!([{"id": enter, "type": "http_done", "status": 200, "body": entered}]),
    );
    // with a profile in use the same press goes out
    let out = step(&mut core, 7, &json!([press]));
    assert_eq!(http_paths(&out), ["/v1/commands"]);
}

#[test]
fn c74_c130_the_last_activity_is_written_at_most_every_30_s() {
    let (mut core, _) = member(3_600);
    // the first activity past 30 s writes the session, so time away survives a restart
    let out = step(&mut core, 40_000, &json!([{"type": "activity"}]));
    assert_eq!(secret_names(&out), ["session"], "{out}");
    // 10 s later nothing is written
    let out = step(&mut core, 50_000, &json!([{"type": "activity"}]));
    assert!(
        secret_names(&out).is_empty(),
        "a write for every event would wear out the store: {out}"
    );
}

#[test]
fn c108_c131_a_hello_with_no_ttl_is_not_read_again_at_once() {
    let text = include_str!("../../contract/fixtures/2026-11-05/valid/Hello.basic.json");
    let mut hello: Value = serde_json::from_str(text).unwrap();
    hello["ttl_s"] = json!(0);
    let mut core = core();
    let first = step(&mut core, 0, &json!([]));
    let id = effect_id(&first, "/v1/hello");
    let out = step(
        &mut core,
        1,
        &json!([{"id": id, "type": "http_done", "status": 200, "body": hello.to_string()}]),
    );
    assert!(http_paths(&out).is_empty(), "{out}");
    assert_eq!(out.get("next_wake_ms"), None, "nothing is armed: {out}");
    let out = step(&mut core, 2, &json!([]));
    assert!(
        http_paths(&out).is_empty(),
        "a ttl of 0 is no reason to ask again at once: {out}"
    );
}
