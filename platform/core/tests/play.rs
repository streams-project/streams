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

/// A core that signed in and entered a profile, so a saved position can leave the core.
fn watching() -> Core {
    let mut core = Core::open_json(CONFIG).unwrap();
    let first = step(&mut core, 0, &json!([]));
    let kept = first["effects"][1]["id"].clone();
    step(&mut core, 1, &json!([{"id": kept, "type": "secret_done"}]));
    let out = step(
        &mut core,
        2,
        &action(
            "s1",
            json!({"action": "sign_in", "email": "a@b.example", "password": "x"}),
        ),
    );
    let sign_in = effect_id(&out, "/v1/auth/password");
    let tokens = json!({
        "user_id": "u1",
        "email": "a@b.example",
        "access_token": "at",
        "refresh_token": "rt",
        "expires_in": 3_600,
    })
    .to_string();
    step(
        &mut core,
        3,
        &json!([{"id": sign_in, "type": "http_done", "status": 200, "body": tokens}]),
    );
    let out = step(
        &mut core,
        4,
        &action("p1", json!({"action": "enter_profile", "profile": "p1"})),
    );
    let enter = effect_id(&out, "/v1/profiles/p1/enter");
    let profile = json!({"id": "p1", "name": "A", "kids": false, "main": true, "has_pin": false});
    let entered = json!({"profile": profile, "home": "home"}).to_string();
    step(
        &mut core,
        5,
        &json!([{"id": enter, "type": "http_done", "status": 200, "body": entered}]),
    );
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
    assert_eq!(
        http_paths(&moved),
        ["/v1/play/p1/events", "/v1/profiles/p1/enter"],
        "the play that was cut off still says what it saw at its host"
    );
}

#[test]
fn c32_c68_h42_nothing_is_saved_before_the_first_frame() {
    let mut core = watching();
    let out = step(
        &mut core,
        6,
        &action("a1", json!({"action": "play", "title": "movie:7"})),
    );
    let play = effect_id(&out, "/v1/play");
    // the answer carries a position another device left
    let answer = json!({
        "play_id": "p1",
        "attempts": [attempt("one", FIRST, "g1")],
        "start_at_ms": 30_000,
        "dropped": 0,
        "more": false,
    });
    let out = step(
        &mut core,
        7,
        &json!([{"id": play, "type": "http_done", "status": 200, "body": answer.to_string()}]),
    );
    assert_eq!(player_ops(&out), [open(1, 30_000, FIRST)]);
    // the viewer leaves before any frame, so this device saw nothing to save
    let out = step(&mut core, 8, &action("a2", json!({"action": "stop"})));
    assert_eq!(player_ops(&out), [json!({"op": "close"})]);
    assert_eq!(
        http_paths(&out),
        ["/v1/play/p1/events"],
        "no position may go out, or the server's own value would be written back: {out}"
    );
    // The one thing that does go out is the play report, and this is the shape that says the
    // viewer left before the picture ever came: an end, and no first frame beside it.
    let events = reported(&out, "p1");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["name"], "ended");
    assert_eq!(events[0]["host"], "media1.example");
}

#[test]
fn c58_a_skip_window_with_nothing_left_to_show_is_dropped() {
    let mut core = Core::open_json(CONFIG).unwrap();
    step(&mut core, 0, &json!([]));
    let out = step(
        &mut core,
        1,
        &action(
            "a1",
            json!({"action": "play", "title": "tv:9",
                   "episode": {"season": 1, "episode": 2}}),
        ),
    );
    let play = effect_id(&out, "/v1/play");
    let answer = json!({
        "play_id": "p1",
        "attempts": [attempt("one", FIRST, "g1")],
        "start_at_ms": 0,
        "next_episode": {"season": 1, "episode": 3},
        "dropped": 0,
        "more": false,
    });
    step(
        &mut core,
        2,
        &json!([{"id": play, "type": "http_done", "status": 200, "body": answer.to_string()}]),
    );
    step(&mut core, 3, &driver(1, &json!({"kind": "first_frame"})));
    // a known length makes the core ask for the marks, series only
    let out = step(&mut core, 4, &driver(1, &position(10_000, 1_920)));
    let marks = effect_id(&out, "/v1/play/p1/marks");
    // the recap runs 500 ms, so hiding its button 1 s early leaves nothing to show
    let windows = json!({"windows": [
        {"kind": "recap", "start_ms": 1_000, "end_ms": 1_500},
        {"kind": "intro", "start_ms": 5_000, "end_ms": 20_000},
    ]})
    .to_string();
    let out = step(
        &mut core,
        5,
        &json!([{"id": marks, "type": "http_done", "status": 200, "body": windows}]),
    );
    let shown = out["view"]["play"]["windows"].as_array().unwrap().clone();
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert_eq!(shown[0]["kind"], "intro");
    assert_eq!(shown[0]["end_ms"], 19_000);
    assert_eq!(shown[0]["seek_to_ms"], 20_000);
}

#[test]
fn a_viewer_who_keeps_seeking_is_not_a_dead_source_but_a_stall_after_it_still_runs_out() {
    let mut core = playing();
    // the first seek stalls at 5 s, and with no more seeks the wait would end at 20 s
    step(&mut core, 5_000, &driver(1, &json!({"kind": "stalled"})));
    step(
        &mut core,
        10_000,
        &action("a", json!({"action": "seek", "to_ms": 100_000})),
    );
    step(
        &mut core,
        18_000,
        &action("b", json!({"action": "seek", "to_ms": 200_000})),
    );
    let waiting = step(&mut core, 25_000, &json!([]));
    assert!(
        player_ops(&waiting).is_empty(),
        "the last seek was only 7 s ago"
    );
    // 15 s after the last seek the source really is gone
    let moved = step(&mut core, 33_001, &json!([]));
    assert_eq!(
        player_ops(&moved),
        [json!({"op": "close"}), open(2, 200_000, SECOND)]
    );
}

/// Starts this episode, as the address of an episode that follows would, up to its first frame.
fn episode_plays(core: &mut Core, mono_ms: u64, out: &Value, generation: u32, next: u32) -> Value {
    let play = effect_id(out, "/v1/play");
    let answer = json!({
        "play_id": format!("p{generation}"),
        "attempts": [attempt("one", FIRST, "g1")],
        "start_at_ms": 0,
        "next_episode": {"season": 1, "episode": next},
        "autoplay_next": true,
        "dropped": 0,
        "more": false,
    });
    step(
        core,
        mono_ms,
        &json!([{"id": play, "type": "http_done", "status": 200, "body": answer.to_string()}]),
    );
    step(
        core,
        mono_ms + 1,
        &driver(generation, &json!({"kind": "first_frame"})),
    )
}

#[test]
fn the_third_episode_in_a_row_with_nobody_there_waits_to_be_asked() {
    let mut core = watching();
    let mut out = step(
        &mut core,
        6,
        &action(
            "a1",
            json!({"action": "play", "title": "tv:7", "episode": {"season": 1, "episode": 1}}),
        ),
    );
    let mut clock = 10;
    for episode in 1..=2 {
        episode_plays(&mut core, clock, &out, episode, episode + 1);
        out = step(
            &mut core,
            clock + 2,
            &driver(episode, &json!({"kind": "ended"})),
        );
        assert_ne!(
            out["view"]["play"]["still_asking"], true,
            "episode {episode}"
        );
        assert!(
            http_paths(&out).iter().any(|path| path == "/v1/play"),
            "episode {episode} moved on by itself: {out}"
        );
        clock += 10;
    }
    episode_plays(&mut core, clock, &out, 3, 4);
    let out = step(&mut core, clock + 2, &driver(3, &json!({"kind": "ended"})));
    assert_eq!(out["view"]["play"]["still_asking"], true, "{out}");
    assert!(http_paths(&out).is_empty(), "nothing starts: {out}");
}

/// A play that has shown a frame, from an answer with these languages.
fn playing_with(core: &mut Core, audio: Option<&str>, subtitles: Option<&str>) {
    let out = step(
        core,
        6,
        &action("a1", json!({"action": "play", "title": "movie:7"})),
    );
    let play = effect_id(&out, "/v1/play");
    let answer = json!({
        "play_id": "p1",
        "attempts": [attempt("one", FIRST, "g1")],
        "start_at_ms": 0,
        "audio_default": audio,
        "subtitle_default": subtitles,
        "dropped": 0,
        "more": false,
    });
    step(
        core,
        7,
        &json!([{"id": play, "type": "http_done", "status": 200, "body": answer.to_string()}]),
    );
    step(core, 8, &driver(1, &json!({"kind": "first_frame"})));
}

fn tracks() -> Value {
    json!({
        "kind": "tracks",
        "audio": [{"id": "a1", "lang": "en", "selected": true}, {"id": "a2", "lang": "de"}],
        "text": [{"id": "t1", "lang": "en", "selected": true}, {"id": "t2", "lang": "fr-CA"}],
    })
}

#[test]
fn the_languages_a_profile_keeps_are_set_once_the_tracks_are_known() {
    let mut core = watching();
    playing_with(&mut core, Some("de"), Some("fr"));
    let out = step(&mut core, 9, &driver(1, &tracks()));
    assert_eq!(
        player_ops(&out),
        [
            json!({"op": "select_audio", "track": "a2"}),
            json!({"op": "select_text", "track": "t2"}),
        ]
    );
    // the same report after the change is not a reason to set them again
    let again = step(&mut core, 10, &driver(1, &tracks()));
    assert!(player_ops(&again).is_empty());
}

#[test]
fn with_no_language_kept_subtitles_stay_off_whatever_the_source_marks() {
    let mut core = watching();
    playing_with(&mut core, None, None);
    let out = step(&mut core, 9, &driver(1, &tracks()));
    assert_eq!(player_ops(&out), [json!({"op": "select_text"})]);
}

#[test]
fn a_choice_in_the_player_is_kept_for_the_profile() {
    let mut core = watching();
    playing_with(&mut core, None, None);
    step(&mut core, 9, &driver(1, &tracks()));
    let out = step(
        &mut core,
        10,
        &action("a2", json!({"action": "select_audio", "track": "a2"})),
    );
    assert_eq!(
        player_ops(&out),
        [json!({"op": "select_audio", "track": "a2"})]
    );
    let saved = out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["path"] == "/v1/commands")
        .unwrap_or_else(|| panic!("nothing saved: {out}"));
    let body = saved["body"].as_str().unwrap();
    assert!(
        body.contains("audio_language") && body.contains("\"de\""),
        "{body}"
    );
}

/// The body of the one request the core sent to `path`, as JSON.
fn request_body(out: &Value, path: &str) -> Value {
    let effect = out["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == "http" && effect["path"] == path)
        .unwrap_or_else(|| panic!("no request for {path} in {out}"));
    serde_json::from_str(effect["body"].as_str().expect("a body")).expect("a body of JSON")
}

/// Every event of a play report, as (name, host) with the numbers that matter beside them.
fn reported(out: &Value, play_id: &str) -> Vec<Value> {
    request_body(out, &format!("/v1/play/{play_id}/events"))["events"]
        .as_array()
        .expect("events")
        .clone()
}

/// A finished play reports what it saw at its host and nothing about who watched it.
///
/// `POST /v1/play/{id}/events` has been in the contract since `0110_play.sql` and no client ever
/// called it, so until now every number on this page was a number nobody had. The report goes
/// once, as the session closes.
#[test]
fn a_finished_play_reports_its_start_delay_its_rebuffers_and_its_host() {
    let mut core = playing();
    // the picture stops twice and comes back, which is what a rebuffer is
    step(&mut core, 20_000, &driver(1, &json!({"kind": "stalled"})));
    step(&mut core, 22_000, &driver(1, &position(11_000, 1_920)));
    step(&mut core, 40_000, &driver(1, &json!({"kind": "stalled"})));
    step(&mut core, 42_000, &driver(1, &position(12_000, 1_920)));
    let out = step(&mut core, 50_000, &action("a9", json!({"action": "stop"})));
    let events = reported(&out, "p1");
    let names: Vec<&str> = events
        .iter()
        .map(|event| event["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["first_frame", "rebuffer", "rebuffer", "ended"]);
    assert!(
        events
            .iter()
            .all(|event| event["host"] == "media1.example" && event["attempt"] == "one"),
        "every event names the host it is about: {events:?}"
    );
    // the frame came one step after the open, and the open is what the delay is measured from
    assert_eq!(events[0]["first_frame_ms"], 1);
    assert!(
        events.iter().all(|event| event.get("title").is_none()
            && event.get("episode").is_none()
            && event.get("profile").is_none()),
        "a report about a host is not a record of what anyone watched: {events:?}"
    );
    assert!(
        events
            .iter()
            .all(|event| event.get("rebuffer_ms").is_none() && event.get("duration_ms").is_none()),
        "a rebuffer is counted and never timed: the Android driver ticks position at 1 Hz and \
         only while playing, so a measured length is quantised and biased upward"
    );
}

/// Leaving twice reports once.
#[test]
fn a_play_reports_once_however_many_times_it_is_left() {
    let mut core = playing();
    let first = step(&mut core, 50_000, &action("a9", json!({"action": "stop"})));
    assert!(
        http_paths(&first).contains(&"/v1/play/p1/events".to_owned()),
        "{first}"
    );
    let again = step(&mut core, 51_000, &action("a10", json!({"action": "stop"})));
    assert!(
        !http_paths(&again).contains(&"/v1/play/p1/events".to_owned()),
        "a second stop must not report the same play twice: {again}"
    );
}

/// A failover keeps the error class of the host that broke, and an exit before the picture ever
/// came needs no word of its own: the report carries an end and no first frame.
#[test]
fn a_report_keeps_the_error_class_of_the_host_that_broke() {
    let mut core = Core::open_json(CONFIG).unwrap();
    step(&mut core, 0, &json!([]));
    let out = step(
        &mut core,
        1,
        &action("a1", json!({"action": "play", "title": "movie:7"})),
    );
    let play = effect_id(&out, "/v1/play");
    let answer = json!({
        "play_id": "p9",
        "attempts": [attempt("one", FIRST, "g1"), attempt("two", SECOND, "g2")],
        "start_at_ms": 0,
        "dropped": 0,
        "more": false,
    });
    step(
        &mut core,
        2,
        &json!([{"id": play, "type": "http_done", "status": 200, "body": answer.to_string()}]),
    );
    // the first host breaks, the second is opened, and the viewer leaves before any frame
    step(
        &mut core,
        3,
        &driver(1, &json!({"kind": "error", "class": "segment"})),
    );
    let out = step(&mut core, 9_000, &action("a2", json!({"action": "stop"})));
    let events = reported(&out, "p9");
    let failure = events
        .iter()
        .find(|event| event["name"] == "failure")
        .expect("the host that broke");
    assert_eq!(failure["failure"], "segment");
    assert_eq!(failure["host"], "media1.example");
    assert!(
        !events.iter().any(|event| event["name"] == "first_frame"),
        "no frame ever came: {events:?}"
    );
    let ended = events
        .iter()
        .find(|event| event["name"] == "ended")
        .expect("an end");
    assert_eq!(
        ended["host"], "media2.example",
        "it ended at the host it had moved to, which is where it gave up"
    );
}
