//! Kept answers. Each test names the behaviour or defect it covers.

use offline::cache::{Cache, Entry, key_of};

const BOOT: &str = "boot-1";

fn store(cache: &mut Cache, key: &str, scope: Option<&str>) -> String {
    cache.stored(
        key.to_string(),
        scope.map(str::to_string),
        Entry::new(60_000, 1_000, BOOT, None, 100),
    )
}

#[test]
fn c117_scope_keeps_one_profile_apart_from_another() {
    assert_ne!(
        key_of(Some("u1/p1"), "/v1/nav"),
        key_of(Some("u1/p2"), "/v1/nav")
    );
    assert_ne!(key_of(None, "/v1/nav"), key_of(Some("u1/p1"), "/v1/nav"));
}

#[test]
fn c100_an_answer_stays_fresh_for_its_ttl_in_one_boot() {
    let mut cache = Cache::default();
    store(&mut cache, "k", None);
    let entry = cache.index.entries["k"].clone();
    assert!(entry.fresh(BOOT, 1_000));
    assert!(entry.fresh(BOOT, 60_999));
    assert!(!entry.fresh(BOOT, 61_000));
}

#[test]
fn h61_monotonic_time_restarts_at_boot_so_another_boot_is_never_fresh() {
    let mut cache = Cache::default();
    store(&mut cache, "k", None);
    let entry = cache.index.entries["k"].clone();
    assert!(!entry.fresh("boot-2", 2_000));
    assert!(!entry.fresh(BOOT, 500), "a clock that ran backwards");
}

#[test]
fn c122_a_new_answer_replaces_the_old_copy_in_place() {
    let mut cache = Cache::default();
    let first = store(&mut cache, "k", None);
    let second = store(&mut cache, "k", None);
    assert_eq!(first, second);
    assert_eq!(cache.bytes(), 100);
}

#[test]
fn c100_the_byte_cap_drops_oldest_scoped_pages_and_the_last_good_hello_goes_last() {
    let mut cache = Cache::default();
    let hello = store(&mut cache, "|/v1/hello", None);
    let old = store(&mut cache, "u1/p1|/v1/nav", Some("u1/p1"));
    let new = store(&mut cache, "u1/p1|/v1/screens/home", Some("u1/p1"));
    assert_eq!(cache.evict_one(), Some(old));
    assert_eq!(cache.evict_one(), Some(new));
    assert_eq!(cache.evict_one(), Some(hello));
    assert_eq!(cache.evict_one(), None);
}

#[test]
fn p13_sign_out_clears_scoped_pages_and_cursors_and_keeps_answers_with_no_scope() {
    let mut cache = Cache::default();
    store(&mut cache, "|/v1/hello", None);
    let page = store(&mut cache, "u1/p1|/v1/nav", Some("u1/p1"));
    cache
        .index
        .cursors
        .insert("u1/p1".to_string(), "c9".to_string());
    assert_eq!(cache.clear_scoped(), vec![page]);
    assert_eq!(cache.index.entries.len(), 1);
    assert!(cache.index.entries.contains_key("|/v1/hello"));
    assert!(cache.index.cursors.is_empty());
}

#[test]
fn c77_a_server_change_marks_only_that_scopes_pages_for_reading_again() {
    let mut cache = Cache::default();
    store(&mut cache, "a", Some("u1/p1"));
    store(&mut cache, "b", Some("u1/p2"));
    cache.mark_stale("u1/p1");
    assert!(!cache.index.entries["a"].fresh(BOOT, 2_000));
    assert!(cache.index.entries["b"].fresh(BOOT, 2_000));
}

#[test]
fn a_304_renews_a_kept_copy() {
    let mut cache = Cache::default();
    store(&mut cache, "k", None);
    cache.mark_stale("none");
    cache.renewed("k", 90_000, BOOT);
    let entry = cache.index.entries["k"].clone();
    assert!(entry.fresh(BOOT, 90_000));
}

#[test]
fn the_index_survives_a_restart_as_json() {
    let mut cache = Cache::default();
    store(&mut cache, "k", Some("u1/p1"));
    let text = serde_json::to_string(&cache.index).unwrap();
    let back = Cache {
        index: serde_json::from_str(&text).unwrap(),
        ..Cache::default()
    };
    assert_eq!(back.index, cache.index);
}
