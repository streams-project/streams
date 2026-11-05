//! Pure rules of the player. Whole sessions run in kit/core with the layer on and off.

use std::collections::BTreeMap;

use contract::play::{Attempt, AttemptKind};
use platform_core::player::{host_of, next_attempt};

fn attempt(url: &str, group: &str, kind: AttemptKind) -> Attempt {
    Attempt {
        id: url.to_string(),
        url: url.to_string(),
        kind,
        group: group.to_string(),
        headers: BTreeMap::new(),
        verified: true,
        container: None,
        codecs: Vec::new(),
        height: None,
        audio: Vec::new(),
        expires_at_ms: None,
    }
}

#[test]
fn c17_c28_c84_c98_a_failed_host_and_its_groups_streams_are_skipped() {
    let list = [
        attempt("https://a.example/1/2160", "g1", AttemptKind::Hls),
        attempt("https://a.example/1/1080", "g1", AttemptKind::Hls),
        attempt("https://b.example/s/1", "g1", AttemptKind::Hls),
        attempt("https://c.example/embed/1", "g2", AttemptKind::Embed),
        attempt("https://d.example/m/1", "g2", AttemptKind::Embed),
    ];
    assert_eq!(next_attempt(&list, 0), Some(3));
    // an embed that fails moves to the next embed of the same group on another host
    assert_eq!(next_attempt(&list, 3), Some(4));
    assert_eq!(next_attempt(&list, 4), None);
}

#[test]
fn c28_an_embed_of_the_failed_group_stays_eligible_on_another_host() {
    let list = [
        attempt("https://a.example/1", "g1", AttemptKind::Hls),
        attempt("https://b.example/1", "g1", AttemptKind::Hls),
        attempt("https://c.example/1", "g1", AttemptKind::Embed),
    ];
    assert_eq!(next_attempt(&list, 0), Some(2));
}

#[test]
fn c28_there_is_no_wrap_around() {
    let list = [
        attempt("https://a.example/1", "g1", AttemptKind::Hls),
        attempt("https://b.example/1", "g2", AttemptKind::Hls),
    ];
    assert_eq!(next_attempt(&list, 1), None);
    assert_eq!(next_attempt(&list, 9), None);
}

#[test]
fn c84_host_compare_is_case_insensitive_and_stops_at_the_first_slash() {
    assert_eq!(host_of("https://A.Example:8443/x/y"), "a.example:8443");
    assert_eq!(host_of("nonsense"), "");
    let list = [
        attempt("https://a.example/1", "g1", AttemptKind::Hls),
        attempt("https://A.EXAMPLE/2", "g2", AttemptKind::Hls),
        attempt("https://b.example/3", "g2", AttemptKind::File),
    ];
    assert_eq!(next_attempt(&list, 0), Some(2));
}
