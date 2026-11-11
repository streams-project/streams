//! The store rules, run against the memory store. The same bodies run against a real Postgres in
//! `postgres.rs`, so a rule can never hold on one store and not on the other.

mod conformance;

use std::sync::Arc;

use conformance::{Both, make_account};
use server_accounts::{AccountId, AccountStore, MemoryStore};

/// A memory store with two accounts, as every body here needs.
async fn both() -> (Arc<dyn AccountStore>, AccountId, AccountId) {
    let store: Arc<dyn AccountStore> = Arc::new(MemoryStore::new());
    let one = make_account(&store, "one@example.test").await;
    let two = make_account(&store, "two@example.test").await;
    (store, one, two)
}

#[tokio::test]
async fn tmt1_a_token_write_that_lost_the_race_changes_nothing() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::tmt1_a_token_write_that_lost_the_race_changes_nothing(&both).await;
}

#[tokio::test]
async fn n11_an_approved_device_code_is_taken_once() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::n11_an_approved_device_code_is_taken_once(&both).await;
}

#[tokio::test]
async fn tmp8_a_moved_profile_leaves_its_pin_and_kids_flag_behind() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::tmp8_a_moved_profile_leaves_its_pin_and_kids_flag_behind(&both).await;
}

#[tokio::test]
async fn tmp8_a_refused_move_changes_nothing() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::tmp8_a_refused_move_changes_nothing(&both).await;
}

#[tokio::test]
async fn tmt2_only_the_session_that_is_kept_survives() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::tmt2_only_the_session_that_is_kept_survives(&both).await;
}

#[tokio::test]
async fn h31_a_session_of_another_account_is_not_theirs_to_remove() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::h31_a_session_of_another_account_is_not_theirs_to_remove(&both).await;
}

#[tokio::test]
async fn s39_a_picture_name_at_the_limit_is_kept() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::s39_a_picture_name_at_the_limit_is_kept(&both).await;
}
