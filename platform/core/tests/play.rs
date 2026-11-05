//! Whole play sessions through the message boundary. The pure parts are in player.rs; the rules
//! here need a session that is open and playing, so each test drives the core itself.

use contract::version::WIRE_VERSION;
use platform_core::{Core, step_json};
use serde_json::{Value, json};

const CONFIG: &str = r#"{"app":"a","class":"web","contract":"2026-11-05","form":"desktop",
  "mode":"local","os":"o","seed":"00112233445566778899aabbccddeeff","type":"core_config",
  "version":"1.0.0"}"#;

const FIRST: &str = "https://media1.example/a.mp4";
const SECOND: &str = "https://media2.example/b.mp4";

fn step(core: &mut Core, mono_ms: u64, events: &Value) -> Value {
    let input = json!({
        "contract": WIRE_VERSION,
        "type": "core_in",
        "now_ms": 1_793_836_800_000_u64 + mono_ms,
        "mono_ms": mono_ms,
        "events": events,
    });
    let text = step_json(core, &input.to_string()).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Every command the core sent to the driver in this step, in order.
fn player_ops(out: &Value) -> Vec<Value> {
    out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "player")
        .map(|effect| effect["command"].clone())
        .collect()
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

/// Each attempt comes from another group, so the group skip rule leaves the second one standing.
fn attempt(id: &str, url: &str, group: &str) -> Value {
    json!({"id": id, "url": url, "kind": "hls", "group": group, "headers": {}, "verified": true})
}

fn open(generation: u32, start_at_ms: u64, url: &str) -> Value {
    json!({
        "op": "open",
        "url": url,
        "headers": {},
        "start_at_ms": start_at_ms,
        "generation": generation,
    })
}

fn driver(generation: u32, event: &Value) -> Value {
    json!([{"type": "driver", "generation": generation, "event": event}])
}

/// A position report. `video_width` of 0 is the driver saying the picture is gone.
fn position(time_ms: u64, video_width: u32) -> Value {
    json!({
        "kind": "position",
        "time_ms": time_ms,
        "duration_ms": 6_000_000,
        "video_width": video_width,
    })
}

fn action(id: &str, action: Value) -> Value {
    json!([{"id": id, "type": "action", "action": action}])
}

/// A play that is open on the first host, has shown a frame and sits at 10 s, with a second host
/// to move to.
fn playing() -> Core {
    let mut core = Core::open_json(CONFIG).unwrap();
    step(&mut core, 0, &json!([]));
    let out = step(
        &mut core,
        1,
        &action("a1", json!({"action": "play", "title": "movie:7"})),
    );
    let play = effect_id(&out, "/v1/play");
    let answer = json!({
        "play_id": "p1",
        "attempts": [attempt("one", FIRST, "g1"), attempt("two", SECOND, "g2")],
        "start_at_ms": 0,
        "dropped": 0,
        "more": false,
    });
    let out = step(
        &mut core,
        2,
        &json!([{"id": play, "type": "http_done", "status": 200, "body": answer.to_string()}]),
    );
    assert_eq!(player_ops(&out), [open(1, 0, FIRST)]);
    let out = step(&mut core, 3, &driver(1, &json!({"kind": "first_frame"})));
    assert_eq!(out["view"]["play"]["state"], "playing");
    // the viewer reaches 10 s with a picture
    step(&mut core, 4, &driver(1, &position(10_000, 1_920)));
    core
}

#[test]
fn c30_p39_no_picture_moves_on_only_past_3_s_by_the_clock_and_2_5_s_of_video() {
    let mut core = playing();
    // the picture goes at 5 s by the clock, with the video at 10 s
    let first = step(&mut core, 5_000, &driver(1, &position(10_000, 0)));
    assert!(player_ops(&first).is_empty(), "the rule has just started");
    // 5 s by the clock, but the video moved 2 s: sound with no picture is still playing
    let waiting = step(&mut core, 10_000, &driver(1, &position(12_000, 0)));
    assert!(
        player_ops(&waiting).is_empty(),
        "2 s of video is not 2.5 s, so nothing moves yet"
    );
    // past both: the next host opens at the second the viewer was on
    let moved = step(&mut core, 10_001, &driver(1, &position(12_600, 0)));
    assert_eq!(
        player_ops(&moved),
        [json!({"op": "close"}), open(2, 12_600, SECOND)]
    );
}

#[test]
fn c103_p35_a_failover_while_paused_opens_the_next_host_paused() {
    let mut core = playing();
    let held = step(&mut core, 5_000, &action("a2", json!({"action": "pause"})));
    assert_eq!(player_ops(&held), [json!({"op": "pause"})]);
    assert_eq!(held["view"]["play"]["state"], "paused");
    let moved = step(
        &mut core,
        6_000,
        &driver(1, &json!({"kind": "error", "class": "network"})),
    );
    assert_eq!(
        player_ops(&moved),
        [
            json!({"op": "close"}),
            open(2, 10_000, SECOND),
            json!({"op": "pause"}),
        ],
        "a paused viewer, or one behind the picker, must not be made to watch again"
    );
}

#[test]
fn c103_p35_entering_a_profile_stops_what_plays() {
    let mut core = playing();
    let moved = step(
        &mut core,
        5_000,
        &action("a3", json!({"action": "enter_profile", "profile": "p1"})),
    );
    assert_eq!(
        player_ops(&moved),
        [json!({"op": "close"})],
        "nothing plays on into another profile"
    );
    assert_eq!(moved["view"]["play"]["state"], "ended");
    assert_eq!(http_paths(&moved), ["/v1/profiles/p1/enter"]);
}
