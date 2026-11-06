//! Device sign in: the core asks for a code, polls and hands the tokens to the normal sign in.
//! The host only draws `view.device_sign_in`. Time is scripted, so every wait is read exactly.

use contract::version::WIRE_VERSION;
use platform_core::{Core, step_json};
use serde_json::{Value, json};

const CONFIG: &str = r#"{"app":"a","class":"tv","contract":"2026-11-05","form":"tv",
  "mode":"local","os":"o","seed":"00112233445566778899aabbccddeeff","type":"core_config",
  "version":"1.0.0"}"#;

const CODE: &str = "/v1/auth/device/code";
const TOKEN: &str = "/v1/auth/device/token";
const SECRET_CODE: &str = "device-code-that-stays-inside";

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

fn effects<'a>(out: &'a Value, kind: &str) -> Vec<&'a Value> {
    out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == kind)
        .collect()
}

fn paths(out: &Value) -> Vec<String> {
    effects(out, "http")
        .iter()
        .map(|effect| effect["path"].as_str().unwrap().to_string())
        .collect()
}

fn id_of(out: &Value, path: &str) -> String {
    effects(out, "http")
        .iter()
        .find(|effect| effect["path"] == path)
        .map_or_else(
            || panic!("no request for {path} in {out}"),
            |effect| effect["id"].as_str().unwrap().to_string(),
        )
}

fn sent_body(out: &Value, path: &str) -> Value {
    let effect = effects(out, "http")
        .into_iter()
        .find(|effect| effect["path"] == path)
        .unwrap_or_else(|| panic!("no request for {path} in {out}"));
    serde_json::from_str(effect["body"].as_str().unwrap()).unwrap()
}

/// A core that knows no session was kept. Hello stays unanswered, so no wake here comes from it.
fn visitor() -> Core {
    let mut core = Core::open_json(CONFIG).unwrap();
    let first = step(&mut core, 0, &json!([]));
    let kept = first["effects"][1]["id"].clone();
    step(&mut core, 1, &json!([{"id": kept, "type": "secret_done"}]));
    core
}

fn action(core: &mut Core, mono_ms: u64, action: &Value) -> Value {
    step(
        core,
        mono_ms,
        &json!([{"id": "a1", "type": "action", "action": action}]),
    )
}

fn press(core: &mut Core, mono_ms: u64) -> Value {
    action(
        core,
        mono_ms,
        &json!({"action": "device_sign_in", "device_name": "Living room TV"}),
    )
}

fn stop(core: &mut Core, mono_ms: u64) -> Value {
    action(core, mono_ms, &json!({"action": "device_sign_in_stop"}))
}

fn code_answer(device_code: &str, user_code: &str, expires_in: u32, interval: u32) -> String {
    json!({
        "device_code": device_code,
        "user_code": user_code,
        "verification_uri": "https://sign.example/tv",
        "verification_uri_complete": format!("https://sign.example/tv?code={user_code}"),
        "expires_in": expires_in,
        "interval": interval,
    })
    .to_string()
}

fn done(core: &mut Core, mono_ms: u64, id: &str, status: u32, body: &str) -> Value {
    step(
        core,
        mono_ms,
        &json!([{"id": id, "type": "http_done", "status": status, "body": body}]),
    )
}

fn failed(core: &mut Core, mono_ms: u64, id: &str, reason: &str) -> Value {
    step(
        core,
        mono_ms,
        &json!([{"id": id, "type": "http_failed", "reason": reason}]),
    )
}

fn word(core: &mut Core, mono_ms: u64, id: &str, word: &str) -> Value {
    done(core, mono_ms, id, 200, &json!({"poll": word}).to_string())
}

/// A visitor who pressed at 10 and read a code at 20, so the first poll is due at 5020.
fn showing() -> Core {
    let mut core = visitor();
    let out = press(&mut core, 10);
    let id = id_of(&out, CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 900, 5);
    done(&mut core, 20, &id, 200, &body);
    core
}

/// The poll that goes out at this moment.
fn poll(core: &mut Core, mono_ms: u64) -> String {
    let out = step(core, mono_ms, &json!([]));
    id_of(&out, TOKEN)
}

fn tokens() -> String {
    json!({"signed_in": {
        "user_id": "u1",
        "email": "a@b.example",
        "access_token": "access-1",
        "refresh_token": "refresh-1",
        "expires_in": 900,
    }})
    .to_string()
}

fn error(code: &str, retry_after_s: Option<u32>) -> String {
    let mut body = json!({"code": code, "label": "a-label", "build": "b"});
    if let Some(seconds) = retry_after_s {
        body["retry_after_s"] = json!(seconds);
    }
    body.to_string()
}

#[test]
fn n9_the_core_asks_for_a_code_and_the_view_shows_what_a_viewer_reads() {
    let mut core = visitor();
    let out = press(&mut core, 10);
    assert_eq!(paths(&out), [CODE]);
    assert_eq!(
        sent_body(&out, CODE),
        json!({"device_name": "Living room TV"})
    );
    let effect = effects(&out, "http")[0];
    assert_eq!(effect["class"], "plain");
    assert_eq!(effect["headers"].get("Authorization"), None);
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "waiting"}));
    assert_eq!(out.get("next_wake_ms"), None);

    let id = id_of(&out, CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 900, 5);
    let out = done(&mut core, 20, &id, 200, &body);
    assert_eq!(
        out["view"]["device_sign_in"],
        json!({
            "state": "waiting",
            "user_code": "BCDFGHJK",
            "verification_uri": "https://sign.example/tv",
            "verification_uri_complete": "https://sign.example/tv?code=BCDFGHJK",
            "expires_in": 900,
        })
    );
    assert_eq!(out["next_wake_ms"], 5_020);
    assert_eq!(out["effects"], json!([]));
}

#[test]
fn n9_authorization_pending_polls_again_one_interval_after_the_answer_arrived() {
    let mut core = showing();
    assert_eq!(step(&mut core, 5_019, &json!([]))["effects"], json!([]));
    let out = step(&mut core, 5_020, &json!([]));
    assert_eq!(paths(&out), [TOKEN]);
    assert_eq!(sent_body(&out, TOKEN), json!({"device_code": SECRET_CODE}));
    assert_eq!(
        effects(&out, "http")[0]["headers"].get("Authorization"),
        None
    );
    assert_eq!(out.get("next_wake_ms"), None, "one poll at a time");

    // a slow answer moves the next poll, so it is never inside the interval
    let id = id_of(&out, TOKEN);
    let out = word(&mut core, 5_700, &id, "authorization_pending");
    assert_eq!(out["next_wake_ms"], 10_700);
    assert_eq!(out.get("view"), None, "nothing changed for the screen");
    assert_eq!(out["effects"], json!([]));
    assert_eq!(
        paths(&step(&mut core, 10_699, &json!([]))),
        Vec::<String>::new()
    );
    assert_eq!(paths(&step(&mut core, 10_700, &json!([]))), [TOKEN]);
}

#[test]
fn n9_slow_down_adds_5_s_for_this_and_every_later_poll() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = word(&mut core, 5_030, &id, "slow_down");
    assert_eq!(out["next_wake_ms"], 15_030);
    assert_eq!(out.get("view"), None);

    let id = poll(&mut core, 15_030);
    let out = word(&mut core, 15_040, &id, "authorization_pending");
    assert_eq!(out["next_wake_ms"], 25_040, "the slower interval stays");

    let id = poll(&mut core, 25_040);
    let out = word(&mut core, 25_050, &id, "slow_down");
    assert_eq!(out["next_wake_ms"], 40_050, "and each slow_down adds again");
}

#[test]
fn n9_expired_token_fetches_a_new_code_with_no_press() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = word(&mut core, 5_030, &id, "slow_down");
    assert_eq!(out["next_wake_ms"], 15_030);

    let id = poll(&mut core, 15_030);
    let out = word(&mut core, 15_040, &id, "expired_token");
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "expired"}));
    assert_eq!(paths(&out), [CODE]);
    assert_eq!(
        sent_body(&out, CODE),
        json!({"device_name": "Living room TV"})
    );

    let id = id_of(&out, CODE);
    let body = code_answer("the-next-device-code", "LMNPQRST", 900, 5);
    let out = done(&mut core, 15_050, &id, 200, &body);
    assert_eq!(out["view"]["device_sign_in"]["state"], "waiting");
    assert_eq!(out["view"]["device_sign_in"]["user_code"], "LMNPQRST");
    assert_eq!(
        out["next_wake_ms"], 20_050,
        "a new code starts on the server's interval"
    );
    let out = step(&mut core, 20_050, &json!([]));
    assert_eq!(
        sent_body(&out, TOKEN),
        json!({"device_code": "the-next-device-code"})
    );
}

#[test]
fn n9_access_denied_shows_denied_and_a_press_asks_for_a_new_code() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = word(&mut core, 5_030, &id, "access_denied");
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "denied"}));
    assert_eq!(out["effects"], json!([]));
    assert_eq!(out.get("next_wake_ms"), None);
    assert_eq!(step(&mut core, 60_000, &json!([]))["effects"], json!([]));

    let out = press(&mut core, 70_000);
    assert_eq!(paths(&out), [CODE]);
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "waiting"}));
}

#[test]
fn n9_tokens_feed_the_normal_sign_in_and_the_address_moves_to_the_picker() {
    let mut core = visitor();
    let out = step(
        &mut core,
        5,
        &json!([{"type": "address", "address": "/sign-in"}]),
    );
    assert_eq!(out["view"]["route"]["target"], "sign_in");
    let id = id_of(&press(&mut core, 10), CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 900, 5);
    done(&mut core, 20, &id, 200, &body);
    let id = poll(&mut core, 5_020);

    let out = done(&mut core, 5_030, &id, 200, &tokens());
    assert_eq!(out["view"]["session"]["state"], "signed_in");
    assert_eq!(out["view"]["session"]["user_id"], "u1");
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "idle"}));
    assert_eq!(out["view"]["route"]["target"], "profiles");
    assert_eq!(paths(&out), ["/v1/profiles"]);
    assert_eq!(effects(&out, "secret")[0]["name"], "session");

    let text = out["view"].to_string();
    assert!(!text.contains("access-1") && !text.contains("refresh-1"));

    // the code is spent, so nothing polls it again
    let later = step(&mut core, 30_000, &json!([]));
    assert_eq!(later["effects"], json!([]));
}

#[test]
fn n9_the_device_code_reaches_no_view_and_no_effect_but_its_poll() {
    let mut core = visitor();
    let mut outs = vec![press(&mut core, 10)];
    let id = id_of(&outs[0], CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 900, 5);
    outs.push(done(&mut core, 20, &id, 200, &body));
    let out = step(&mut core, 5_020, &json!([]));
    let id = id_of(&out, TOKEN);
    outs.push(word(&mut core, 5_030, &id, "authorization_pending"));
    outs.push(failed(&mut core, 5_040, "unknown-id", "offline"));
    outs.push(stop(&mut core, 5_050));
    for out in &outs {
        assert!(!out.to_string().contains(SECRET_CODE), "{out}");
    }
    assert!(out.to_string().contains(SECRET_CODE), "the poll carries it");
}

#[test]
fn n9_a_network_failure_keeps_polling_with_no_notice_and_backs_off() {
    let mut core = showing();
    let mut at = 5_020;
    // 2, 4, 8 and 16 s of back off, never under the 5 s interval
    for wait in [5_000, 5_000, 8_000, 16_000] {
        let id = poll(&mut core, at);
        let out = failed(&mut core, at + 10, &id, "offline");
        assert_eq!(out["effects"], json!([]), "no notice");
        assert_eq!(out.get("view"), None, "the code stays on the screen");
        at = at + 10 + wait;
        assert_eq!(out["next_wake_ms"], at);
    }
    let id = poll(&mut core, at);
    let out = failed(&mut core, at + 10, &id, "timeout");
    assert_eq!(out["next_wake_ms"], at + 10 + 32_000);
    at = at + 10 + 32_000;

    // one answer ends the back off
    let id = poll(&mut core, at);
    let out = word(&mut core, at + 10, &id, "authorization_pending");
    assert_eq!(out["next_wake_ms"], at + 10 + 5_000);
}

#[test]
fn n9_c5_a_network_return_cuts_a_retry_and_never_the_interval() {
    let mut core = showing();
    let online = json!([{"type": "network", "online": true}]);
    assert_eq!(step(&mut core, 1_000, &online)["effects"], json!([]));

    let id = poll(&mut core, 5_020);
    failed(&mut core, 5_030, &id, "offline");
    let out = step(&mut core, 6_000, &online);
    assert_eq!(paths(&out), [TOKEN]);
    let id = id_of(&out, TOKEN);
    word(&mut core, 6_010, &id, "authorization_pending");
    assert_eq!(step(&mut core, 7_000, &online)["effects"], json!([]));
}

#[test]
fn n9_a_failed_code_request_keeps_asking_with_no_notice() {
    let mut core = visitor();
    let out = press(&mut core, 10);
    let id = id_of(&out, CODE);
    let out = failed(&mut core, 20, &id, "offline");
    assert_eq!(out["next_wake_ms"], 2_020);
    assert_eq!(out["effects"], json!([]));
    assert_eq!(out.get("view"), None);
    let out = step(&mut core, 2_020, &json!([]));
    let id = id_of(&out, CODE);
    let out = failed(&mut core, 2_030, &id, "timeout");
    assert_eq!(out["next_wake_ms"], 6_030);
    let out = step(&mut core, 6_030, &json!([]));
    let id = id_of(&out, CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 900, 5);
    let out = done(&mut core, 6_040, &id, 200, &body);
    assert_eq!(out["view"]["device_sign_in"]["user_code"], "BCDFGHJK");
    assert_eq!(out["next_wake_ms"], 11_040);
}

#[test]
fn n9_a_503_unsupported_sets_off_and_a_press_asks_again() {
    let mut core = visitor();
    let id = id_of(&press(&mut core, 10), CODE);
    let out = done(&mut core, 20, &id, 503, &error("unsupported", None));
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "off"}));
    assert_eq!(out["effects"], json!([]));
    assert_eq!(out.get("next_wake_ms"), None);
    assert_eq!(step(&mut core, 60_000, &json!([]))["effects"], json!([]));
    assert_eq!(paths(&press(&mut core, 70_000)), [CODE]);
}

#[test]
fn n9_a_503_from_an_outage_keeps_polling_and_keeps_the_code() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = done(&mut core, 5_030, &id, 503, &error("overloaded", Some(10)));
    assert_eq!(out.get("view"), None);
    assert_eq!(out["effects"], json!([]));
    assert_eq!(out["next_wake_ms"], 10_030);
    let id = poll(&mut core, 10_030);
    let out = done(&mut core, 10_040, &id, 500, "not json");
    assert_eq!(out["next_wake_ms"], 15_040);
}

#[test]
fn n9_a_busy_answer_waits_as_long_as_the_server_says() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = done(&mut core, 5_030, &id, 429, &error("rate_limited", Some(45)));
    assert_eq!(out["next_wake_ms"], 50_030);
    assert_eq!(out["effects"], json!([]));
}

#[test]
fn n9_a_refusal_that_would_repeat_ends_it_with_the_servers_word() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = done(&mut core, 5_030, &id, 400, &error("bad_request", None));
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "off"}));
    assert_eq!(effects(&out, "view")[0]["notice"]["label"], "a-label");
    assert_eq!(out.get("next_wake_ms"), None);

    let mut core = visitor();
    let id = id_of(&press(&mut core, 10), CODE);
    let out = done(&mut core, 20, &id, 404, "");
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "off"}));
    assert_eq!(
        effects(&out, "view")[0]["notice"]["label"],
        "problem-failed"
    );
}

#[test]
fn n9_stop_ends_polling_and_clears_the_code_from_the_view() {
    let mut core = showing();
    let out = stop(&mut core, 1_000);
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "idle"}));
    assert_eq!(out.get("next_wake_ms"), None);
    assert_eq!(step(&mut core, 5_020, &json!([]))["effects"], json!([]));
}

#[test]
fn n9_an_answer_after_a_stop_is_dropped_tokens_and_all() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    stop(&mut core, 5_021);
    let out = done(&mut core, 5_030, &id, 200, &tokens());
    assert_eq!(out.get("view"), None);
    assert_eq!(out["effects"], json!([]));

    let mut core = showing();
    let id = poll(&mut core, 5_020);
    stop(&mut core, 5_021);
    assert_eq!(
        word(&mut core, 5_030, &id, "slow_down").get("next_wake_ms"),
        None
    );

    // a code that lands after a stop shows nothing, and a new press is not confused by it
    let mut core = visitor();
    let old = id_of(&press(&mut core, 10), CODE);
    stop(&mut core, 11);
    let new = id_of(&press(&mut core, 12), CODE);
    let body = code_answer("an-old-device-code", "BCDFGHJK", 900, 5);
    let out = done(&mut core, 20, &old, 200, &body);
    assert_eq!(out.get("view"), None);
    assert_eq!(out.get("next_wake_ms"), None);
    let body = code_answer(SECRET_CODE, "LMNPQRST", 900, 5);
    let out = done(&mut core, 30, &new, 200, &body);
    assert_eq!(out["view"]["device_sign_in"]["user_code"], "LMNPQRST");
}

#[test]
fn n9_a_second_press_on_a_live_code_changes_nothing() {
    let mut core = showing();
    let out = press(&mut core, 1_000);
    assert_eq!(out["effects"], json!([]));
    assert_eq!(out.get("view"), None);
    assert_eq!(out["next_wake_ms"], 5_020);

    let mut core = visitor();
    assert_eq!(paths(&press(&mut core, 10)), [CODE]);
    assert_eq!(paths(&press(&mut core, 11)), Vec::<String>::new());
}

#[test]
fn n9_a_code_past_its_life_renews_itself_when_polls_cannot_reach_the_server() {
    let mut core = visitor();
    let id = id_of(&press(&mut core, 10), CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 20, 5);
    done(&mut core, 20, &id, 200, &body);

    // 5 s, 5 s, then a back off of 8 s that would run past the end of the code
    let id = poll(&mut core, 5_020);
    let out = failed(&mut core, 5_030, &id, "offline");
    assert_eq!(out["next_wake_ms"], 10_030);
    let id = poll(&mut core, 10_030);
    let out = failed(&mut core, 10_040, &id, "offline");
    assert_eq!(out["next_wake_ms"], 15_040);
    let id = poll(&mut core, 15_040);
    let out = failed(&mut core, 15_050, &id, "offline");
    assert_eq!(
        out["next_wake_ms"], 20_020,
        "the wake comes at the end of the code"
    );

    let out = step(&mut core, 20_020, &json!([]));
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "expired"}));
    assert_eq!(paths(&out), [CODE]);
}

#[test]
fn n9_a_poll_on_its_way_when_the_code_ends_still_counts() {
    let mut core = visitor();
    let id = id_of(&press(&mut core, 10), CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 6, 5);
    done(&mut core, 20, &id, 200, &body);
    let id = poll(&mut core, 5_020);
    let out = step(&mut core, 7_000, &json!([]));
    assert_eq!(
        out["effects"],
        json!([]),
        "nothing replaces a poll on its way"
    );
    let out = done(&mut core, 7_010, &id, 200, &tokens());
    assert_eq!(out["view"]["session"]["state"], "signed_in");
}

#[test]
fn n9_a_cancelled_or_unsupported_request_counts_as_no_answer() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = step(&mut core, 5_030, &json!([{"id": id, "type": "cancelled"}]));
    assert_eq!(out["next_wake_ms"], 10_030);
    let id = poll(&mut core, 10_030);
    let out = step(
        &mut core,
        10_040,
        &json!([{"id": id, "type": "unsupported"}]),
    );
    assert_eq!(out["next_wake_ms"], 15_040);

    let mut core = visitor();
    let id = id_of(&press(&mut core, 10), CODE);
    let out = step(&mut core, 20, &json!([{"id": id, "type": "cancelled"}]));
    assert_eq!(out["next_wake_ms"], 2_020);
}

#[test]
fn n9_an_unknown_word_or_a_body_that_does_not_read_keeps_polling() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = word(&mut core, 5_030, &id, "from_a_later_contract");
    assert_eq!(out["next_wake_ms"], 10_030);
    let id = poll(&mut core, 10_030);
    let out = done(&mut core, 10_040, &id, 200, "not json");
    assert_eq!(out["next_wake_ms"], 15_040);
    let id = poll(&mut core, 15_040);
    let out = done(&mut core, 15_050, &id, 200, "{}");
    assert_eq!(
        out["next_wake_ms"], 20_050,
        "two misses stay under the interval"
    );

    let mut core = visitor();
    let id = id_of(&press(&mut core, 10), CODE);
    let out = done(&mut core, 20, &id, 200, "not json");
    assert_eq!(out["next_wake_ms"], 2_020);
}

#[test]
fn n9_a_zero_interval_or_life_from_the_server_cannot_spin() {
    let mut core = visitor();
    let id = id_of(&press(&mut core, 10), CODE);
    let body = code_answer(SECRET_CODE, "BCDFGHJK", 0, 0);
    let out = done(&mut core, 20, &id, 200, &body);
    assert_eq!(out["next_wake_ms"], 5_020, "the default of RFC 8628 3.5");
    assert_eq!(out["effects"], json!([]));
    let out = step(&mut core, 5_020, &json!([]));
    assert_eq!(paths(&out), [CODE], "a new code, one interval later");
}

#[test]
fn n9_a_session_from_any_other_way_ends_a_running_sign_in() {
    let mut core = Core::open_json(CONFIG).unwrap();
    let first = step(&mut core, 0, &json!([]));
    let kept = first["effects"][1]["id"].clone();
    let out = press(&mut core, 1);
    assert_eq!(paths(&out), [CODE], "the kept session is not known yet");
    let text = json!({"refresh_token": "r", "user_id": "u1", "email": "a@b.example"}).to_string();
    let out = step(
        &mut core,
        2,
        &json!([{"id": kept, "type": "secret_done", "value": text}]),
    );
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "idle"}));
    assert_eq!(out["view"]["session"]["state"], "signed_in");

    let mut core = showing();
    let out = action(
        &mut core,
        1_000,
        &json!({"action": "sign_in", "email": "a@b.example", "password": "x"}),
    );
    let id = id_of(&out, "/v1/auth/password");
    let body = serde_json::from_str::<Value>(&tokens()).unwrap()["signed_in"].to_string();
    let out = done(&mut core, 1_010, &id, 200, &body);
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "idle"}));
    assert!(!paths(&step(&mut core, 5_020, &json!([]))).contains(&TOKEN.to_string()));
}

#[test]
fn n9_a_member_press_does_nothing() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    done(&mut core, 5_030, &id, 200, &tokens());
    let out = press(&mut core, 5_040);
    assert!(!paths(&out).contains(&CODE.to_string()));
    assert_eq!(out.get("view"), None);
}

#[test]
fn n9_an_upgrade_answer_stops_polling() {
    let mut core = showing();
    let id = poll(&mut core, 5_020);
    let out = done(&mut core, 5_030, &id, 426, &error("upgrade_required", None));
    assert_eq!(out["view"]["route"]["target"], "upgrade");
    assert_eq!(out["view"]["device_sign_in"], json!({"state": "idle"}));
    assert_eq!(out.get("next_wake_ms"), None);
    assert_eq!(press(&mut core, 6_000)["effects"], json!([]));
}
