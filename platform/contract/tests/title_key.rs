//! Title identity. Covers H57, H91, C25, C78, C22, C18, C63, C133.

use contract::{MAX_KEYS_PER_LIST, Rating, TitleKey, TitleKind};
use contract::{TitleKeyError, is_completed, watch_percent};

#[test]
fn h57_h91_bare_id_does_not_parse_from_text() {
    assert_eq!(TitleKey::parse("550"), Err(TitleKeyError::NoKind));
}

#[test]
fn h57_h91_bare_id_does_not_parse_from_json_string() {
    assert!(serde_json::from_str::<TitleKey>("\"550\"").is_err());
}

#[test]
fn h57_h91_bare_id_does_not_parse_from_json_number() {
    assert!(serde_json::from_str::<TitleKey>("550").is_err());
}

#[test]
fn h57_h91_same_id_of_each_kind_stays_two_titles() {
    let film = TitleKey::parse("movie:1001").unwrap();
    let series = TitleKey::parse("tv:1001").unwrap();
    assert_ne!(film, series);
    assert_eq!(film.id(), series.id());
    assert_eq!(film.kind(), TitleKind::Movie);
    assert_eq!(series.kind(), TitleKind::Tv);
}

#[test]
fn c25_key_text_round_trips() {
    for text in ["movie:1", "tv:1399", "movie:2147483647"] {
        let key = TitleKey::parse(text).unwrap();
        assert_eq!(key.to_string(), text);
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, format!("\"{text}\""));
        assert_eq!(serde_json::from_str::<TitleKey>(&json).unwrap(), key);
    }
}

#[test]
fn c25_c78_every_broken_key_is_refused() {
    let broken = [
        "",
        ":",
        ":550",
        "movie",
        "movie:",
        "tv:",
        "show:1",
        "Movie:1",
        "TV:1",
        "movie:0",
        "tv:0",
        "movie:-1",
        "movie:+1",
        "movie: 1",
        " movie:1",
        "movie:1 ",
        "movie:01",
        "movie:1:2",
        "movie:1.5",
        "movie:abc",
        "movie:2147483648",
        "movie:99999999999999999999",
    ];
    for text in broken {
        assert!(TitleKey::parse(text).is_err(), "{text:?} parsed");
        let json = serde_json::to_string(text).unwrap();
        assert!(
            serde_json::from_str::<TitleKey>(&json).is_err(),
            "{text:?} parsed from json"
        );
    }
}

#[test]
fn c78_a_key_is_only_built_with_a_kind_and_a_real_id() {
    assert!(TitleKey::new(TitleKind::Movie, 0).is_err());
    assert!(TitleKey::new(TitleKind::Tv, 1 << 31).is_err());
    assert_eq!(TitleKey::new(TitleKind::Tv, 7).unwrap().to_string(), "tv:7");
}

#[test]
fn c78_packed_key_keeps_the_kind_and_comes_back() {
    let film = TitleKey::parse("movie:1001").unwrap();
    let series = TitleKey::parse("tv:1001").unwrap();
    assert_eq!(film.packed(), 1001);
    assert_eq!(series.packed(), 1001 | (1 << 31));
    assert_eq!(TitleKey::from_packed(film.packed()), Ok(film));
    assert_eq!(TitleKey::from_packed(series.packed()), Ok(series));
    assert!(TitleKey::from_packed(0).is_err());
    assert!(TitleKey::from_packed(1 << 31).is_err());
}

#[test]
fn c25_list_skips_junk_removes_repeats_and_folds_case() {
    let keys = TitleKey::parse_list("movie:1, TV:2,junk,550,movie:1,tv:,show:3, tv:3 ");
    let text: Vec<String> = keys.iter().map(ToString::to_string).collect();
    assert_eq!(text, ["movie:1", "tv:2", "tv:3"]);
}

#[test]
fn c25_list_stops_at_forty_keys() {
    assert_eq!(MAX_KEYS_PER_LIST, 40);
    let long: Vec<String> = (1..=60).map(|n| format!("movie:{n}")).collect();
    let keys = TitleKey::parse_list(&long.join(","));
    assert_eq!(keys.len(), 40);
    assert_eq!(keys[39].id(), 40);
}

#[test]
fn c25_empty_list_is_empty() {
    assert!(TitleKey::parse_list("").is_empty());
}

#[test]
fn c133_record_and_episode_keys() {
    let film = TitleKey::parse("movie:550").unwrap();
    let series = TitleKey::parse("tv:1399").unwrap();
    assert_eq!(film.record_key(), "movie_550");
    assert_eq!(series.record_key(), "tv_1399");
    assert_eq!(series.episode_key(1, 3), "tv_1399_1_3");
}

#[test]
fn c133_percent_is_clamped_and_needs_a_duration() {
    assert_eq!(watch_percent(50_000, 100_000), Some(50));
    assert_eq!(watch_percent(150_000, 100_000), Some(100));
    assert_eq!(watch_percent(1, 0), None);
    assert_eq!(watch_percent(0, 100_000), Some(0));
}

#[test]
fn c133_completed_at_ninety_five_percent() {
    assert!(is_completed(95_000, 100_000));
    assert!(!is_completed(94_999, 100_000));
    assert!(!is_completed(10, 0));
}

#[test]
fn c22_rating_words_map_both_ways() {
    let pairs = [
        ("like", Rating::Up),
        ("love", Rating::Love),
        ("dislike", Rating::Down),
    ];
    for (word, rating) in pairs {
        assert_eq!(Rating::from_client_word(word), Some(rating));
        assert_eq!(rating.client_word(), Some(word));
    }
    assert_eq!(Rating::from_client_word("meh"), None);
    assert_eq!(Rating::Unknown.client_word(), None);
}

#[test]
fn c22_c63_rating_values_on_the_wire() {
    assert_eq!(serde_json::to_string(&Rating::Up).unwrap(), "\"up\"");
    assert_eq!(serde_json::to_string(&Rating::Down).unwrap(), "\"down\"");
    assert_eq!(
        serde_json::from_str::<Rating>("\"love\"").unwrap(),
        Rating::Love
    );
}
