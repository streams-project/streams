//! The play gate through the real router, with a scripted finder.
#![allow(clippy::disallowed_types)] // a test reads answers as a client does

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::http::{Method, StatusCode};
use common::{Harness, call, get, harness_play};
use contract::finder::{FindAnswer, FindOutcome, FoundSource};
use contract::play::{AttemptKind, PlayAnswer};
use serde_json::json;
use server_catalog::sample::special;
use server_play::{FoundFile, MemoryFinder, MemorySubtitles, PlayLog, Subtitles};

fn source(url: &str) -> FoundSource {
    FoundSource {
        url: url.to_owned(),
        kind: AttemptKind::Hls,
        group: String::from("a-provider"),
        headers: BTreeMap::new(),
        verified: true,
        container: None,
        height: None,
        audio: Vec::new(),
    }
}

fn found(urls: &[&str]) -> FindAnswer {
    FindAnswer {
        outcome: FindOutcome::Found,
        sources: urls.iter().map(|url| source(url)).collect(),
        job: None,
    }
}

async fn enter(server: &Harness, kids: bool) -> String {
    let (_, token) = common::member(&server.door).await;
    let list = get("/v1/profiles").token(&token).on(&server.state).await;
    let profiles = list.body["profiles"].as_array().unwrap();
    let id = profiles.iter().find(|p| p["kids"] == kids).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    call(Method::POST, &format!("/v1/profiles/{id}/enter"))
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    token
}

fn play(title: &str) -> common::Call {
    call(Method::POST, "/v1/play").json(json!({ "title": title }))
}

#[tokio::test]
async fn s112_a_play_answers_ranked_attempts_with_where_to_start() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/1.m3u8",
    ])));
    let server = harness_play(finder.clone());
    let token = enter(&server, false).await;
    let batch = json!({ "ops": [{ "op_id": "a", "command": "position", "title": "movie:100", "position_s": 120, "duration_s": 6000 }] });
    call(Method::POST, "/v1/commands")
        .token(&token)
        .json(batch)
        .on(&server.state)
        .await;
    let answer = play("movie:100").token(&token).on(&server.state).await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let answer: PlayAnswer = serde_json::from_value(answer.body).expect("PlayAnswer");
    assert_eq!(answer.attempts.len(), 1);
    assert_eq!(answer.start_at_ms, 120_000, "C31");
    assert!(answer.autoplay_next, "S46");
    assert_eq!(finder.seen.lock().unwrap()[0].budget_ms, 8000);
    assert!(answer.age_card.is_some());
}

#[tokio::test]
async fn tmk2_h35_h46_h79_h90_a_refused_title_reaches_no_finder() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/1.m3u8",
    ])));
    let server = harness_play(finder.clone());
    let token = enter(&server, true).await;
    let adult = format!("movie:{}", special::ADULT_FLAG);
    let answer = play(&adult).token(&token).on(&server.state).await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN);
    assert_eq!(answer.code(), "refused");
    assert!(
        finder.seen.lock().unwrap().is_empty(),
        "no provider work for a refused title"
    );
}

#[tokio::test]
async fn h78_title_facts_come_from_the_catalogue_and_the_client_cannot_steer_a_search() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/1.m3u8",
    ])));
    let server = harness_play(finder.clone());
    let token = enter(&server, false).await;
    let body = json!({ "title": "movie:9002", "name": "Something Else", "year": 1900 });
    let answer = call(Method::POST, "/v1/play")
        .token(&token)
        .json(body)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let seen = finder.seen.lock().unwrap();
    assert_eq!(seen[0].name, "Twin Film");
    assert_eq!(seen[0].year, Some(2020));
}

#[tokio::test]
async fn h57_a_bare_id_does_not_parse_and_an_unknown_title_is_gone() {
    let finder = Arc::new(MemoryFinder::default());
    let server = harness_play(finder);
    let token = enter(&server, false).await;
    let bare = call(Method::POST, "/v1/play")
        .token(&token)
        .json(json!({ "title": 100 }))
        .on(&server.state)
        .await;
    assert_eq!(bare.status, StatusCode::BAD_REQUEST);
    let gone = play("movie:999999").token(&token).on(&server.state).await;
    assert_eq!(gone.code(), "no_longer_available");
}

#[tokio::test]
async fn c34_a_series_needs_an_episode_and_a_film_takes_none() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/1.m3u8",
    ])));
    let server = harness_play(finder);
    let token = enter(&server, false).await;
    let series = format!("tv:{}", special::SHARED_ID);
    assert_eq!(
        play(&series).token(&token).on(&server.state).await.status,
        StatusCode::BAD_REQUEST
    );
    let film_episode = call(Method::POST, "/v1/play")
        .token(&token)
        .json(json!({ "title": "movie:100", "episode": { "season": 1, "episode": 1 } }))
        .on(&server.state)
        .await;
    assert_eq!(film_episode.status, StatusCode::BAD_REQUEST);
    let good = call(Method::POST, "/v1/play")
        .token(&token)
        .json(json!({ "title": series, "episode": { "season": 1, "episode": 2 } }))
        .on(&server.state)
        .await;
    assert_eq!(good.status, StatusCode::OK, "{}", good.body);
}

#[tokio::test]
async fn the_answer_names_the_next_episode_and_none_after_the_last() {
    for (episode, next) in [(1, Some(json!({ "season": 1, "episode": 2 }))), (2, None)] {
        // The test finder answers one request.
        let finder = Arc::new(MemoryFinder::answering(found(&[
            "https://host.example/1.m3u8",
        ])));
        let server = harness_play(finder);
        let token = enter(&server, false).await;
        let series = format!("tv:{}", special::SHARED_ID);
        let answer = call(Method::POST, "/v1/play")
            .token(&token)
            .json(json!({ "title": series, "episode": { "season": 1, "episode": episode } }))
            .on(&server.state)
            .await;
        assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
        assert_eq!(answer.body.get("next_episode").cloned(), next);
    }
}

#[tokio::test]
async fn h17_a_miss_is_a_real_answer_and_a_dead_finder_is_not_a_miss() {
    let finder = Arc::new(MemoryFinder::default());
    let server = harness_play(finder.clone());
    let token = enter(&server, false).await;
    let miss = play("movie:100").token(&token).on(&server.state).await;
    assert_eq!(miss.status, StatusCode::NOT_FOUND);
    assert_eq!(miss.code(), "no_source");
    finder.down.store(true, std::sync::atomic::Ordering::SeqCst);
    let dead = play("movie:100").token(&token).on(&server.state).await;
    assert_eq!(dead.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(dead.headers.contains_key("retry-after"));
}

#[tokio::test]
async fn tmk15_a_play_id_stops_working_after_a_profile_switch_and_for_anyone_else() {
    let finder = Arc::new(MemoryFinder::answering(FindAnswer {
        outcome: FindOutcome::Partial,
        sources: vec![source("https://host.example/1.m3u8")],
        job: Some(String::from("job-1")),
    }));
    finder.push_rest(found(&["https://host.example/late.m3u8"]));
    let server = harness_play(finder);
    let token = enter(&server, false).await;
    let other = enter(&server, false).await;
    let answer = play("movie:100").token(&token).on(&server.state).await;
    assert_eq!(answer.body["more"], true, "{}", answer.body);
    let id = answer.body["play_id"].as_str().unwrap().to_owned();
    let path = format!("/v1/play/{id}/more");
    let stolen = call(Method::POST, &path)
        .token(&other)
        .on(&server.state)
        .await;
    assert_eq!(stolen.status, StatusCode::NOT_FOUND);
    let good = call(Method::POST, &path)
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(
        good.body["attempts"][0]["url"], "https://host.example/late.m3u8",
        "{}",
        good.body
    );
    assert_eq!(good.body["more"], false);

    let list = get("/v1/profiles").token(&token).on(&server.state).await;
    let kids = list.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kids"] == true)
        .unwrap();
    let enter_kids = format!("/v1/profiles/{}/enter", kids["id"].as_str().unwrap());
    call(Method::POST, &enter_kids)
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    let events = format!("/v1/play/{id}/events");
    let late = call(Method::POST, &events)
        .token(&token)
        .json(json!({ "events": [] }))
        .on(&server.state)
        .await;
    assert_eq!(late.status, StatusCode::NOT_FOUND, "K15");
}

#[tokio::test]
async fn s213_with_no_finder_playback_says_it_is_off() {
    let server = common::harness(&[]);
    let token = enter(&server, false).await;
    let answer = play("movie:100").token(&token).on(&server.state).await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer.code(), "unsupported");
}

#[tokio::test]
async fn h5_the_saved_card_filters_attempts_and_says_how_many_went() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/1.m3u8",
        "https://host.example/2.mkv",
    ])));
    let server = harness_play(finder);
    let token = enter(&server, false).await;
    let mut card: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../platform/contract/fixtures/2026-11-05/valid/CapabilityCard.web.json"
    ))
    .unwrap();
    card["containers"] = json!(["ts", "mp4"]);
    let saved = call(Method::POST, "/v1/devices/me")
        .token(&token)
        .json(json!({ "device_key": "0123456789abcdef", "card_hash": "h1", "card": card }))
        .on(&server.state)
        .await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    let answer = play("movie:100").token(&token).on(&server.state).await;
    assert_eq!(
        answer.body["attempts"].as_array().unwrap().len(),
        1,
        "{}",
        answer.body
    );
    assert_eq!(answer.body["dropped"], 1);
}

#[tokio::test]
async fn c53_s108_marks_answer_for_a_play_and_a_report_needs_a_member() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/1.m3u8",
    ])));
    let server = harness_play(finder);
    let token = enter(&server, false).await;
    let answer = play("movie:100").token(&token).on(&server.state).await;
    let id = answer.body["play_id"].as_str().unwrap().to_owned();
    let marks = call(Method::POST, &format!("/v1/play/{id}/marks"))
        .token(&token)
        .json(json!({ "attempt": "a0", "duration_ms": 1_000_000 }))
        .on(&server.state)
        .await;
    assert_eq!(marks.body["windows"], json!([]), "{}", marks.body);
    let report = json!({ "code": "player", "build": "b1" });
    let none = call(Method::POST, "/v1/reports")
        .json(report.clone())
        .on(&server.state)
        .await;
    assert_eq!(none.status, StatusCode::UNAUTHORIZED);
    let good = call(Method::POST, "/v1/reports")
        .token(&token)
        .json(report)
        .on(&server.state)
        .await;
    assert_eq!(good.status, StatusCode::OK);
}

fn srt(lines: usize) -> Vec<u8> {
    let mut out = String::new();
    for n in 0..lines {
        let minute = n / 60;
        let second = n % 60;
        out.push_str(&format!(
            "{}\n00:{minute:02}:{second:02},000 --> 00:{minute:02}:{second:02},900\nStay with me now, please\n\n",
            n + 1
        ));
    }
    out.into_bytes()
}

fn with_subtitles(server: &mut Harness, files: Vec<(FoundFile, Vec<u8>)>) {
    let source = MemorySubtitles { files };
    let state = Arc::get_mut(&mut server.state).expect("one owner");
    state.subtitles = Some(Subtitles::new(Arc::new(source)));
}

fn file(locator: &str, language: &str, hearing_impaired: bool) -> FoundFile {
    FoundFile {
        locator: locator.to_owned(),
        language: language.to_owned(),
        hearing_impaired,
    }
}

#[tokio::test]
async fn c39_s178_s176_a_list_groups_by_language_puts_hearing_impaired_last_and_hides_the_source() {
    let mut server = common::harness(&[]);
    with_subtitles(
        &mut server,
        vec![
            (
                file("https://files.example/a?key=hidden", "EN", true),
                srt(20),
            ),
            (file("https://files.example/b", "English", false), srt(20)),
            (file("https://files.example/c", "BR_PT", false), srt(20)),
            (
                file("https://files.example/d", "Spanish - Forced", false),
                srt(20),
            ),
        ],
    );
    let token = enter(&server, false).await;
    let answer = get("/v1/subtitles?title=movie:100")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let text = answer.body.to_string();
    assert!(
        !text.contains("hidden") && !text.contains("files.example"),
        "S176: {text}"
    );
    let languages = answer.body["languages"].as_array().unwrap();
    assert_eq!(languages.len(), 2, "forced files leave the list");
    assert_eq!(languages[0]["lang"], "en");
    assert_eq!(languages[0]["files"][0]["hearing_impaired"], false);
    assert_eq!(languages[0]["files"][1]["hearing_impaired"], true);
    assert_eq!(languages[1]["lang"], "pt-BR");
}

#[tokio::test]
async fn c43_a_file_reads_as_clean_cues_or_webvtt_and_a_bad_file_tells_the_core_to_move_on() {
    let mut server = common::harness(&[]);
    with_subtitles(
        &mut server,
        vec![
            (file("good", "en", false), srt(20)),
            (
                file("note", "de", false),
                b"a note pointing at a download site".to_vec(),
            ),
        ],
    );
    let token = enter(&server, false).await;
    let listed = get("/v1/subtitles?title=movie:100")
        .token(&token)
        .on(&server.state)
        .await;
    let languages = listed.body["languages"].as_array().unwrap();
    let id_of = |lang: &str| {
        let found = languages.iter().find(|l| l["lang"] == lang).unwrap();
        found["files"][0]["id"].as_str().unwrap().to_owned()
    };
    let good = get(&format!("/v1/subtitles/{}", id_of("en")))
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(
        good.body["cues"].as_array().unwrap().len(),
        20,
        "{}",
        good.body
    );
    assert_eq!(good.body["last_end_ms"], 19_900);
    let text = get(&format!("/v1/subtitles/{}?delay_ms=1000", id_of("en")))
        .token(&token)
        .header("accept", "text/vtt")
        .on(&server.state)
        .await;
    assert!(
        text.headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/vtt")
    );
    let bad = get(&format!("/v1/subtitles/{}", id_of("de")))
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let unknown = get("/v1/subtitles/nothing")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn tmk3_a_subtitle_list_and_file_stay_refused_for_a_title_the_viewer_may_not_see() {
    let mut server = common::harness(&[]);
    with_subtitles(&mut server, vec![(file("one", "en", false), srt(20))]);
    let adult = enter(&server, false).await;
    let kids = enter(&server, true).await;
    let path = format!("/v1/subtitles?title=movie:{}", special::ADULT_FLAG);
    let listed = get(&path).token(&adult).on(&server.state).await;
    let id = listed.body["languages"][0]["files"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        get(&path).token(&kids).on(&server.state).await.status,
        StatusCode::FORBIDDEN
    );
    let read = get(&format!("/v1/subtitles/{id}"))
        .token(&kids)
        .on(&server.state)
        .await;
    assert_eq!(read.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn s213_with_no_source_subtitles_say_they_are_off() {
    let server = common::harness(&[]);
    let token = enter(&server, false).await;
    let answer = get("/v1/subtitles?title=movie:100")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.code(), "unsupported");
}

#[tokio::test]
async fn c54_a_film_gets_no_marks() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/1.m3u8",
    ])));
    let server = harness_play(finder);
    let token = enter(&server, false).await;
    let answer = play("movie:100").token(&token).on(&server.state).await;
    let id = answer.body["play_id"].as_str().unwrap().to_owned();
    let marks = call(Method::POST, &format!("/v1/play/{id}/marks"))
        .token(&token)
        .json(json!({ "attempt": "a0", "duration_ms": 3_000_000 }))
        .on(&server.state)
        .await;
    assert_eq!(marks.body["windows"], json!([]));
}

/// What a play did at its hosts reaches the table, and the read path gives it back.
///
/// This is the pair that was missing. `POST /v1/play/{id}/events` has been in the contract and
/// mounted since `0110_play.sql`, and no client ever called it; `reachability` has been in the
/// schema just as long with no insert, update or select anywhere in the tree. A write with no
/// reader is how both of those happened, so the write and the read are tested together or not at
/// all.
///
/// The report names a host, an attempt and three numbers. It names no profile, and there is
/// nowhere in `reachability` to put one.
#[tokio::test]
async fn c36_c104_a_play_report_reaches_the_table_and_reads_back_by_class_host_and_day() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/one.m3u8",
    ])));
    let mut server = harness_play(finder);
    let log = Arc::new(server_play::MemoryPlayLog::default());
    Arc::get_mut(&mut server.state)
        .expect("one owner")
        .keep_play_log(log.clone());
    let token = enter(&server, false).await;
    let answer = play("movie:100").token(&token).on(&server.state).await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let id = answer.body["play_id"]
        .as_str()
        .expect("a play id")
        .to_owned();
    let events = format!("/v1/play/{id}/events");
    let body = json!({
        "events": [
            { "name": "failure", "at_ms": 0, "host": "bad.example", "failure": "segment" },
            { "name": "first_frame", "at_ms": 0, "host": "host.example", "first_frame_ms": 1_200 },
            { "name": "rebuffer", "at_ms": 30_000, "host": "host.example" },
            { "name": "rebuffer", "at_ms": 61_000, "host": "host.example" },
            { "name": "ended", "at_ms": 90_000, "host": "host.example" }
        ]
    });
    let sent = call(Method::POST, &events)
        .token(&token)
        .header("client-class", "tv")
        .json(body)
        .on(&server.state)
        .await;
    assert_eq!(sent.status, StatusCode::OK, "{}", sent.body);
    assert_eq!(sent.body["done"], true);

    let rows = log.rows();
    assert!(!rows.is_empty(), "the report reached the table");
    assert!(
        rows.iter().all(|row| row.class == "tv"),
        "the class is the one the server read from the header: {rows:?}"
    );

    let read = log.health_since(server_play::health::today()).await;
    let host = read
        .iter()
        .find(|day| day.host == "host.example")
        .expect("the host that played");
    assert_eq!(host.tries, 1);
    assert_eq!(host.starts, 1);
    assert_eq!(host.mean_start_ms(), Some(1_200));
    assert_eq!(host.rebuffers, 2, "a count of rebuffers, never a length");
    assert_eq!(host.gave_up, 0);
    assert!(host.failures.is_empty());
    let bad = read
        .iter()
        .find(|day| day.host == "bad.example")
        .expect("the host that broke");
    assert_eq!(bad.failures, vec![("segment".to_owned(), 1)]);
    assert_eq!(bad.tries, 1, "it was tried");
    assert_eq!(bad.starts, 0, "and it never showed a frame");
    assert_eq!(
        bad.gave_up, 0,
        "the play did not end here, it moved on, which is a failure and not a walk away"
    );

    // And the shape an operator sees on the hourly timer, which is what stops this being a third
    // table nobody reads.
    let lines = server_play::health::lines(&read);
    let line = lines
        .iter()
        .find(|line| line.contains("host.example"))
        .expect("a line per class, host and day");
    assert!(line.contains("tv"), "{line}");
    assert!(line.contains("first frame 1200ms"), "{line}");
    assert!(line.contains("rebuffers 2"), "{line}");
    let broke = lines
        .iter()
        .find(|line| line.contains("bad.example"))
        .expect("the host that broke");
    assert!(broke.contains("[segment 1]"), "{broke}");
}

/// Playback health is not a record of a person, so the personal-rows answer does not govern it.
///
/// It used to. `play_events` keyed every row to `profile_id` and the handler could only write
/// through a writer that had taken the viewer's answer, which made a count about a streaming host
/// into a row about a viewer and let one viewer's answer decide whether anyone could see that a
/// host had stopped working. The rows moved to `reachability`, which has no profile column, and
/// the write that takes them has no parameter a person could be passed through, so this holds
/// whatever the answer on the session is.
#[tokio::test]
async fn c36_c104_playback_health_is_kept_whatever_the_consent_answer() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/one.m3u8",
    ])));
    let mut server = harness_play(finder);
    let log = Arc::new(server_play::MemoryPlayLog::default());
    Arc::get_mut(&mut server.state)
        .expect("one owner")
        .keep_play_log(log.clone());
    let token = enter(&server, false).await;
    let answer = play("movie:100").token(&token).on(&server.state).await;
    let id = answer.body["play_id"]
        .as_str()
        .expect("a play id")
        .to_owned();
    let events = format!("/v1/play/{id}/events");
    let body = json!({
        "events": [
            { "name": "first_frame", "at_ms": 0, "host": "host.example", "first_frame_ms": 900 }
        ]
    });

    // Consent is off until it is given, which is where every session starts.
    let off = call(Method::POST, &events)
        .token(&token)
        .json(body.clone())
        .on(&server.state)
        .await;
    assert_eq!(off.status, StatusCode::OK, "{}", off.body);
    let after_off = log.health_since(server_play::health::today()).await;
    assert_eq!(after_off.len(), 1, "a host said nothing about a person");
    assert_eq!(after_off[0].starts, 1);

    let given = call(Method::PUT, "/v1/consent")
        .token(&token)
        .json(json!({ "personal": true }))
        .on(&server.state)
        .await;
    assert_eq!(given.status, StatusCode::OK, "{}", given.body);
    let on = call(Method::POST, &events)
        .token(&token)
        .json(body)
        .on(&server.state)
        .await;
    assert_eq!(on.status, StatusCode::OK, "{}", on.body);
    let after_on = log.health_since(server_play::health::today()).await;
    assert_eq!(
        after_on[0].starts, 2,
        "the same answer either way: the row has no person in it to consent to"
    );
}

/// A fault report is not a record of a person: the row has no account or profile column, so the
/// personal-rows answer does not govern it and a client with consent off is still able to say
/// that something broke.
#[tokio::test]
async fn c53_s108_a_fault_report_is_kept_whatever_the_consent_answer() {
    let finder = Arc::new(MemoryFinder::answering(found(&[
        "https://host.example/x.m3u8",
    ])));
    let mut server = harness_play(finder);
    let log = Arc::new(server_play::MemoryPlayLog::default());
    Arc::get_mut(&mut server.state)
        .expect("one owner")
        .keep_play_log(log.clone());
    let token = enter(&server, false).await;
    let sent = call(Method::POST, "/v1/reports")
        .token(&token)
        .json(json!({ "code": "player_failed", "build": "test-build" }))
        .on(&server.state)
        .await;
    assert_eq!(sent.status, StatusCode::OK, "{}", sent.body);
    assert_eq!(log.reports.lock().expect("the log").len(), 1);
}
