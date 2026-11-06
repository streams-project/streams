//! Rules of the wire shapes. Each test names the behaviour or defect id it covers.

use contract::command::{Command, CommandBatch, Outcome, SettingKey};
use contract::core_msg::{CoreIn, CoreOut, Effect, EffectBody, Event, EventBody};
use contract::driver::DriverEvent;
use contract::error::{ErrorBody, ErrorCode};
use contract::hello::{BrandValues, Features};
use contract::play::{Attempt, AttemptKind};
use contract::screen::{Section, SectionKind};
use contract::version::{VersionFit, VersionRange, WireDate};
use contract::{EpisodeRef, TitleKey};
use serde_json::json;

fn date(text: &str) -> WireDate {
    WireDate::parse(text).unwrap()
}

fn attempt() -> serde_json::Value {
    json!({
        "id": "a1",
        "url": "https://media.example/a.m3u8",
        "kind": "hls",
        "group": "g1",
        "headers": {},
        "verified": true
    })
}

#[test]
fn h4_h119_s2_range_never_means_exact_equality() {
    let range = VersionRange {
        min: date("2026-01-01"),
        max: date("2026-11-05"),
    };
    assert_eq!(range.fit(&date("2026-06-01")), VersionFit::Supported);
    assert_eq!(range.fit(&date("2026-01-01")), VersionFit::Supported);
    assert_eq!(range.fit(&date("2026-11-05")), VersionFit::Supported);
    assert_eq!(range.fit(&date("2025-12-31")), VersionFit::TooOld);
    assert_eq!(range.fit(&date("2027-01-01")), VersionFit::Newer);
}

#[test]
fn h119_wire_date_refuses_other_shapes() {
    for text in [
        "",
        "2026",
        "2026-11",
        "2026-1-5",
        "20261105",
        "2026/11/05",
        "2026-11-0x",
    ] {
        assert!(WireDate::parse(text).is_err(), "{text:?} parsed");
    }
}

#[test]
fn c108_with_no_answer_accounts_are_required_and_games_are_off() {
    let features = Features::default();
    assert!(features.accounts_required);
    assert!(!features.games);
    let read: Features = serde_json::from_value(json!({})).unwrap();
    assert_eq!(read, features);
}

#[test]
fn c137_brand_is_a_value_filled_at_render_time() {
    let brand = BrandValues {
        name: "Sample".into(),
        support_email: "help@host.example".into(),
    };
    assert_eq!(
        brand.fill("{brand} answers at {support_email}. {brand}."),
        "Sample answers at help@host.example. Sample."
    );
}

#[test]
fn unknown_section_kind_reads_and_draws_as_a_plain_row() {
    let section: Section = serde_json::from_value(json!({
        "id": "s1",
        "kind": "kind_from_the_future",
        "title": "Row",
        "tiles": [{"title": "movie:1", "name": "One"}]
    }))
    .unwrap();
    assert_eq!(section.kind, SectionKind::Unknown);
    assert_eq!(section.kind.drawn_as(), SectionKind::Row);
    assert_eq!(section.tiles.len(), 1);
}

#[test]
fn unknown_attempt_kind_does_not_break_the_answer() {
    let mut value = attempt();
    value["kind"] = json!("kind_from_the_future");
    let read: Attempt = serde_json::from_value(value).unwrap();
    assert_eq!(read.kind, AttemptKind::Unknown);
}

#[test]
fn unknown_error_code_reads_as_unknown() {
    let body: ErrorBody = serde_json::from_value(json!({
        "code": "code_from_the_future",
        "label": "error.future",
        "build": "b1"
    }))
    .unwrap();
    assert_eq!(body.code, ErrorCode::Unknown);
}

#[test]
fn unknown_event_keeps_its_id_so_it_can_be_answered_unsupported() {
    let read: Event = serde_json::from_value(json!({
        "id": "e7",
        "type": "event_from_the_future",
        "payload": {"a": 1}
    }))
    .unwrap();
    assert_eq!(read.id.as_deref(), Some("e7"));
    assert_eq!(read.body, EventBody::Unknown);
}

#[test]
fn unknown_effect_kind_keeps_its_id() {
    let read: Effect = serde_json::from_value(json!({
        "id": "x1",
        "kind": "kind_from_the_future",
        "payload": 1
    }))
    .unwrap();
    assert_eq!(read.id, "x1");
    assert_eq!(read.body, EffectBody::Unknown);
}

#[test]
fn unknown_driver_event_kind_reads_as_unknown() {
    let read: DriverEvent = serde_json::from_value(json!({"kind": "later", "x": 1})).unwrap();
    assert_eq!(read, DriverEvent::Unknown);
}

#[test]
fn core_in_needs_its_date_its_type_and_both_clocks() {
    let good = json!({
        "contract": "2026-11-05",
        "type": "core_in",
        "now_ms": 1,
        "mono_ms": 2,
        "events": []
    });
    assert!(serde_json::from_value::<CoreIn>(good.clone()).is_ok());
    for field in ["contract", "type", "now_ms", "mono_ms", "events"] {
        let mut value = good.clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<CoreIn>(value).is_err(),
            "no {field}"
        );
    }
    let mut wrong = good;
    wrong["type"] = json!("core_out");
    assert!(serde_json::from_value::<CoreIn>(wrong).is_err());
}

#[test]
fn core_out_absent_view_means_no_change() {
    let out: CoreOut = serde_json::from_value(json!({
        "contract": "2026-11-05",
        "type": "core_out",
        "effects": []
    }))
    .unwrap();
    assert!(out.view.is_none());
    assert!(out.next_wake_ms.is_none());
}

#[test]
fn c16_every_required_attempt_field_is_required() {
    let good = attempt();
    assert!(serde_json::from_value::<Attempt>(good.clone()).is_ok());
    for field in ["id", "url", "kind", "group", "headers", "verified"] {
        let mut value = good.clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<Attempt>(value).is_err(),
            "no {field}"
        );
    }
}

#[test]
fn c16_optional_attempt_fields_are_optional_and_stay_out_when_unset() {
    let read: Attempt = serde_json::from_value(attempt()).unwrap();
    let back = serde_json::to_value(&read).unwrap();
    assert_eq!(back, attempt());
}

#[test]
fn c16_an_attempt_carries_no_provider_name() {
    let mut value = attempt();
    value["provider"] = json!("named");
    let read: Attempt = serde_json::from_value(value).unwrap();
    let back = serde_json::to_value(&read).unwrap();
    assert!(back.get("provider").is_none());
}

#[test]
fn c18_c63_list_and_rating_keep_their_own_version_and_time() {
    use contract::command::Change;
    let read: Change = serde_json::from_value(json!({
        "change": "title",
        "title": "tv:1001",
        "list": {"in_list": false, "version": 4, "changed_at_ms": 200},
        "rating": {"rating": "love", "version": 9, "changed_at_ms": 100}
    }))
    .unwrap();
    let Change::Title { list, rating, .. } = read else {
        panic!("not a title change");
    };
    assert!(!list.in_list);
    assert_eq!(rating.rating, Some(contract::Rating::Love));
    assert_ne!(list.version, rating.version);
    assert_ne!(list.changed_at_ms, rating.changed_at_ms);
}

#[test]
fn c18_c63_list_and_rate_commands_touch_one_field_each() {
    let list = serde_json::to_value(Command::List {
        title: TitleKey::parse("movie:1").unwrap(),
        in_list: false,
    })
    .unwrap();
    assert!(list.get("rating").is_none());
    let rate = serde_json::to_value(Command::Rate {
        title: TitleKey::parse("movie:1").unwrap(),
        rating: None,
    })
    .unwrap();
    assert!(rate.get("in_list").is_none());
}

#[test]
fn c64_hide_needs_no_episode_and_clear_history_needs_no_title() {
    let hide: Command =
        serde_json::from_value(json!({"command": "hide", "title": "tv:5"})).unwrap();
    assert_eq!(
        hide,
        Command::Hide {
            title: TitleKey::parse("tv:5").unwrap()
        }
    );
    let clear: Command = serde_json::from_value(json!({"command": "clear_history"})).unwrap();
    assert_eq!(clear, Command::ClearHistory);
}

#[test]
fn c64_c133_position_keeps_season_and_episode() {
    let read: Command = serde_json::from_value(json!({
        "command": "position",
        "title": "tv:5",
        "episode": {"season": 1, "episode": 3},
        "position_s": 61,
        "duration_s": 1200
    }))
    .unwrap();
    let Command::Position { episode, .. } = read else {
        panic!("not a position");
    };
    assert_eq!(
        episode,
        Some(EpisodeRef {
            season: 1,
            episode: 3
        })
    );
}

#[test]
fn h57_a_command_with_a_bare_id_does_not_parse() {
    let read = serde_json::from_value::<Command>(json!({
        "command": "list",
        "title": "550",
        "in_list": true
    }));
    assert!(read.is_err());
}

#[test]
fn s44_h61_age_is_optional_and_missing_means_unknown() {
    let batch: CommandBatch = serde_json::from_value(json!({
        "ops": [
            {"op_id": "1", "age_ms": 4000, "command": "clear_history"},
            {"op_id": "2", "command": "clear_history"}
        ]
    }))
    .unwrap();
    assert_eq!(batch.ops[0].age_ms, Some(4000));
    assert_eq!(batch.ops[1].age_ms, None);
    let back = serde_json::to_value(&batch).unwrap();
    assert!(back["ops"][1].get("age_ms").is_none());
}

#[test]
fn s193_s194_a_queued_setting_cannot_name_the_age_rating_or_blocked_titles() {
    for key in ["age_rating", "blocked_titles", "maturity"] {
        let read: Command = serde_json::from_value(json!({
            "command": "setting",
            "key": key,
            "value": "18"
        }))
        .unwrap();
        let Command::Setting { key, .. } = read else {
            panic!("not a setting");
        };
        assert_eq!(key, SettingKey::Unknown);
    }
}

#[test]
fn unknown_command_reads_as_unknown_and_keeps_its_op_id() {
    let batch: CommandBatch = serde_json::from_value(json!({
        "ops": [{"op_id": "9", "command": "command_from_the_future", "x": 1}]
    }))
    .unwrap();
    assert_eq!(batch.ops[0].op_id, "9");
    assert_eq!(batch.ops[0].command, Command::Unknown);
}

#[test]
fn outcome_words_are_the_three_the_note_names() {
    for (word, outcome) in [
        ("applied", Outcome::Applied),
        ("superseded", Outcome::Superseded),
        ("rejected", Outcome::Rejected),
    ] {
        assert_eq!(
            serde_json::from_value::<Outcome>(json!(word)).unwrap(),
            outcome
        );
    }
}

#[test]
fn every_route_is_found_by_its_own_path() {
    use contract::api::{ROUTES, find};
    for route in ROUTES {
        let path = route
            .path
            .replace("{kind}", "movie")
            .replace("{id}", "x1")
            .replace("{name}", "home")
            .replace("{lang}", "en")
            .replace("{n}", "2")
            .replace("{cursor}", "c")
            .replace("{file}", "f1")
            .replace("{job}", "j1");
        let found = find(route.method, &path).expect(route.path);
        assert_eq!(found.method, route.method);
    }
    assert_eq!(
        find("GET", "/v1/profiles/me/settings").map(|route| route.name),
        Some("getSettings"),
    );
    assert!(find("GET", "/v1/nowhere").is_none());
    assert!(find("GET", "/v1/profiles/").is_none());
    assert!(find("DELETE", "/v1/hello").is_none());
}
