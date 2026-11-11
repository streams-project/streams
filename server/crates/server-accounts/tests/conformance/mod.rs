//! Rules both stores must answer alike. Each body is written once here and run twice: against the
//! memory store in `stores.rs`, and against a real Postgres in `postgres.rs` where one is set. The
//! defects these cover were all cases of the two stores differing, so a rule proved on one store
//! only was not proved at all.

use std::sync::Arc;

use server_accounts::pin::{PinCheck, PinOutcome};
use server_accounts::profile::{AVATAR_MAX_CHARS, starting_profiles};
use server_accounts::{
    AccountId, AccountStore, Consent, DeviceCard, DeviceCodeRow, DeviceSession, NewAccount,
    Profile, ProfileId, ProfileKind, ProfilePatch, SessionId, StoreError, hash_token,
};

/// The time these bodies run at. Any fixed time will do: nothing here reads a real clock.
pub const NOW_MS: u64 = 1_700_000_000_000;

/// One store with two accounts whose rows the caller has already made.
pub struct Both<'a> {
    pub store: &'a Arc<dyn AccountStore>,
    pub one: AccountId,
    pub two: AccountId,
}

/// The rows of one account, as the door makes them.
///
/// # Panics
/// When the store refuses, which is a failure of the test.
pub async fn make_account(store: &Arc<dyn AccountStore>, email: &str) -> AccountId {
    let account = NewAccount {
        id: AccountId::new(),
        email: email.to_owned(),
        first_name: String::from("Ada"),
        last_name: String::from("Lace"),
    };
    let profiles = starting_profiles(&account, &server_accounts::limits::DEFAULT_LIMITS);
    store
        .create_account(&account, &profiles)
        .await
        .expect("the rows of a new account");
    account.id
}

/// A session row with hashes of its own. Postgres holds each hash once, so every session here is
/// built from a name no other session uses.
fn session(account: AccountId, name: &str) -> DeviceSession {
    DeviceSession {
        id: SessionId::new(),
        account,
        device_name: String::from("test device"),
        access_hash: hash_token(&format!("{name}-access")),
        access_expires_ms: NOW_MS + 900_000,
        refresh_current: hash_token(&format!("{name}-current")),
        refresh_next: hash_token(&format!("{name}-next")),
        refresh_salt: String::from("salt"),
        refresh_first_used_ms: None,
        active_profile: None,
        entered_at_ms: None,
        step_up_at_ms: None,
        step_up_method: None,
        consent: Consent::default(),
        last_seen_ms: NOW_MS,
        created_ms: NOW_MS,
    }
}

fn only_name(name: &str) -> ProfilePatch {
    ProfilePatch {
        name: Some(name.to_owned()),
        ..ProfilePatch::default()
    }
}

/// A refresh that read the row before another one finished writes nothing, and the rotation that
/// finished still stands. Without the condition on the hashes that were read, the stale snapshot
/// goes back and the device is left holding a token the row does not know.
///
/// # Panics
/// When the store refuses, or a stale write is taken.
pub async fn tmt1_a_token_write_that_lost_the_race_changes_nothing(both: &Both<'_>) {
    let read = session(both.one, &format!("race-{}", SessionId::new()));
    both.store
        .insert_session(&read)
        .await
        .expect("a session row");

    let mut promoted = read.clone();
    promoted.refresh_current = hash_token("after-the-rotation");
    promoted.refresh_next = hash_token("after-the-rotation-next");
    assert!(
        both.store
            .update_tokens(&promoted, &read.refresh_current, &read.refresh_next)
            .await
            .expect("the first write"),
        "the refresh that read the row first writes"
    );

    let mut stale = read.clone();
    stale.refresh_first_used_ms = Some(NOW_MS + 1);
    assert!(
        !both
            .store
            .update_tokens(&stale, &read.refresh_current, &read.refresh_next)
            .await
            .expect("the second write"),
        "a write under hashes the row no longer holds is refused"
    );
    let after = both
        .store
        .session(read.id)
        .await
        .expect("a read")
        .expect("the row");
    assert_eq!(
        (after.refresh_current, after.refresh_next),
        (promoted.refresh_current, promoted.refresh_next),
        "the rotation that finished is still the one the row holds"
    );
}

/// Of two polls on one approved device code, only one is told it took the row, so only one goes on
/// to open a session.
///
/// # Panics
/// When the store refuses, or both polls win.
pub async fn n11_an_approved_device_code_is_taken_once(both: &Both<'_>) {
    let name = format!("device-{}", SessionId::new());
    let hash = hash_token(&name);
    let row = DeviceCodeRow {
        device_hash: hash,
        user_code: name.chars().take(8).collect(),
        device_name: String::from("tv"),
        account: None,
        denied: false,
        last_poll_ms: None,
        expires_ms: NOW_MS + 900_000,
    };
    both.store
        .insert_device_code(&row)
        .await
        .expect("a waiting sign in");
    assert!(
        both.store
            .approve_device_code(&hash, both.one)
            .await
            .expect("the approval"),
        "the code was waiting"
    );
    let (first, second) = (Arc::clone(both.store), Arc::clone(both.store));
    let polls = tokio::join!(
        tokio::spawn(async move { first.delete_device_code(&hash).await }),
        tokio::spawn(async move { second.delete_device_code(&hash).await }),
    );
    let won = [polls.0, polls.1]
        .into_iter()
        .filter(|poll| matches!(poll, Ok(Ok(true))))
        .count();
    assert_eq!(won, 1, "exactly one poll takes an approved code");
}

/// A profile that moves to another account leaves its PIN and the Kids flag that is one per
/// account behind, both inside the move itself.
///
/// # Panics
/// When the store refuses, or either travels.
pub async fn tmp8_a_moved_profile_leaves_its_pin_and_kids_flag_behind(both: &Both<'_>) {
    let kids = both
        .store
        .profiles_of(both.one)
        .await
        .expect("the profiles")
        .into_iter()
        .find(|row| row.auto_kids)
        .expect("the Kids profile of a new account");
    // Both accounts came with a profile of this name, and a name is held once per account.
    both.store
        .update_profile(both.one, kids.id, &only_name("Little"))
        .await
        .expect("a rename");
    both.store
        .set_pin(kids.id, Some(String::from("a-pin-hash")))
        .await
        .expect("a PIN");

    both.store
        .move_profile(both.one, kids.id, both.two, 5)
        .await
        .expect("the move, which the Kids flag of the other account must not stop");

    let moved = both
        .store
        .profile(both.two, kids.id)
        .await
        .expect("a read")
        .expect("the moved profile");
    assert!(!moved.auto_kids, "the one per account flag does not travel");
    assert!(!moved.has_pin, "the PIN does not travel");
    let check = PinCheck {
        pin: None,
        now_ms: NOW_MS,
    };
    assert_eq!(
        both.store
            .pin_attempt(kids.id, check)
            .await
            .expect("a PIN check"),
        PinOutcome::Pass,
        "no PIN row is left behind for the new account to meet"
    );
}

/// A move the destination has no room for changes nothing at all: the profile stays where it was,
/// with its PIN.
///
/// # Panics
/// When the store refuses for another reason, or anything changed.
pub async fn tmp8_a_refused_move_changes_nothing(both: &Both<'_>) {
    let held = both
        .store
        .profiles_of(both.two)
        .await
        .expect("the profiles")
        .len();
    for n in held..5 {
        let filler = Profile {
            id: ProfileId::new(),
            account: both.two,
            name: format!("Full {n}"),
            avatar: String::from("default"),
            kind: ProfileKind::Adult,
            is_main: false,
            auto_kids: false,
            has_pin: false,
            sort_order: i16::try_from(n).unwrap_or(0),
            age_limit: String::from("18"),
            maturity: String::from("adult"),
            blocked_names: Vec::new(),
        };
        both.store
            .insert_profile(&filler, 5)
            .await
            .expect("a filler profile");
    }

    let mine = both
        .store
        .profiles_of(both.one)
        .await
        .expect("the profiles")
        .into_iter()
        .find(|row| !row.is_main)
        .expect("a profile that may move");
    both.store
        .set_pin(mine.id, Some(String::from("a-pin-hash")))
        .await
        .expect("a PIN");

    assert_eq!(
        both.store
            .move_profile(both.one, mine.id, both.two, 5)
            .await,
        Err(StoreError::LimitReached),
        "the destination is full"
    );
    let after = both
        .store
        .profile(both.one, mine.id)
        .await
        .expect("a read")
        .expect("the profile stayed");
    assert!(after.has_pin, "a refused move leaves the PIN alone");
    assert_eq!(after.auto_kids, mine.auto_kids);
}

/// Signing out the other devices reaches every one of them, past the end of any device list.
///
/// # Panics
/// When the store refuses, or a session outlives the call.
pub async fn tmt2_only_the_session_that_is_kept_survives(both: &Both<'_>) {
    let run = SessionId::new();
    let mut ids = Vec::new();
    for n in 0..60 {
        let row = session(both.one, &format!("many-{run}-{n}"));
        both.store
            .insert_session(&row)
            .await
            .expect("a session row");
        ids.push(row.id);
    }
    let keep = ids[0];
    both.store
        .delete_other_sessions(both.one, keep)
        .await
        .expect("the sign out");
    for id in ids {
        let alive = both
            .store
            .session(id)
            .await
            .expect("a read")
            .is_some_and(|row| row.account == both.one);
        assert_eq!(alive, id == keep, "session {id}");
    }
}

/// A session belongs to its account: another account's call removes nothing, and only the call
/// that removed a row is told so. Ledger S57 is the row this proves — deny by default, where a
/// query takes a scope the door minted and nothing wider. The name said H31 before, which is the
/// row about the active profile on the session and the step up, and is not this.
///
/// # Panics
/// When the store refuses, or an answer is wrong.
pub async fn s57_a_session_of_another_account_is_not_theirs_to_remove(both: &Both<'_>) {
    let row = session(both.one, &format!("mine-{}", SessionId::new()));
    both.store
        .insert_session(&row)
        .await
        .expect("a session row");
    assert!(
        !both
            .store
            .delete_session(both.two, row.id)
            .await
            .expect("a delete"),
        "another account's session is not theirs to remove"
    );
    assert!(
        both.store
            .delete_session(both.one, row.id)
            .await
            .expect("a delete"),
        "the account that holds it removes it"
    );
    assert!(
        !both
            .store
            .delete_session(both.one, row.id)
            .await
            .expect("a delete"),
        "and the second call is told there was nothing left"
    );
}

/// Deleting an account leaves no row of it on either store: no profile, no session, no device
/// card, no PIN, no audit row, no waiting device code. Postgres loses them by cascade; the memory
/// store kept the PIN hashes and the device cards, because nothing in it holds the account on
/// those rows, and no body covered account deletion at all — so the one promise a person cares
/// most about was proved on neither store.
///
/// # Panics
/// When the store refuses, or anything of the account is still there.
pub async fn s55_deleting_an_account_leaves_nothing_of_it(both: &Both<'_>) {
    let profiles = both
        .store
        .profiles_of(both.one)
        .await
        .expect("the profiles");
    assert!(!profiles.is_empty(), "a new account has profiles");
    for row in &profiles {
        both.store
            .set_pin(row.id, Some(String::from("a-pin-hash")))
            .await
            .expect("a PIN");
    }
    let device = session(both.one, &format!("gone-{}", SessionId::new()));
    both.store
        .insert_session(&device)
        .await
        .expect("a session row");
    let key = format!("device-key-{}", SessionId::new());
    both.store
        .save_card(&DeviceCard {
            session: device.id,
            account: both.one,
            device_key: &key,
            card_hash: "hash-of-the-card",
            card: Some(&serde_json::json!({ "class": "tv" })),
        })
        .await
        .expect("a card");
    both.store
        .log_event(both.one, "signed_in")
        .await
        .expect("an audit row");
    let waiting = DeviceCodeRow {
        device_hash: hash_token(&format!("waiting-{}", SessionId::new())),
        user_code: SessionId::new().to_string().chars().take(8).collect(),
        device_name: String::from("tv"),
        account: Some(both.one),
        denied: false,
        last_poll_ms: None,
        expires_ms: NOW_MS + 900_000,
    };
    both.store
        .insert_device_code(&waiting)
        .await
        .expect("a waiting sign in");

    both.store
        .delete_account(both.one)
        .await
        .expect("the deletion");

    assert!(
        both.store
            .profiles_of(both.one)
            .await
            .expect("a read")
            .is_empty(),
        "no profile of the account is left"
    );
    assert!(
        both.store
            .sessions_of(both.one)
            .await
            .expect("a read")
            .is_empty(),
        "no session of the account is left"
    );
    assert_eq!(
        both.store.card(device.id).await.expect("a read"),
        None,
        "no device card of the account is left"
    );
    assert!(
        both.store
            .events_of(both.one, 100)
            .await
            .expect("a read")
            .is_empty(),
        "no audit row of the account is left"
    );
    assert_eq!(
        both.store
            .device_code(&waiting.device_hash)
            .await
            .expect("a read"),
        None,
        "no waiting sign in of the account is left"
    );
    for row in &profiles {
        let check = PinCheck {
            pin: Some("0000"),
            now_ms: NOW_MS,
        };
        assert_eq!(
            both.store
                .pin_attempt(row.id, check)
                .await
                .expect("a PIN check"),
            PinOutcome::Pass,
            "no PIN hash of a deleted profile is left to be met"
        );
    }
    // The other account is untouched, so this is deletion and not a truncation.
    assert!(
        !both
            .store
            .profiles_of(both.two)
            .await
            .expect("a read")
            .is_empty(),
        "the other account keeps its profiles"
    );
}

/// The retention the prune is asked for is the retention it keeps. The memory store took
/// `events_before_ms` and ignored it, so the 365 day rule on the audit log held on no store that
/// CI runs.
///
/// # Panics
/// When the store refuses, or a row outlives or outlasts its argument.
pub async fn s51_the_prune_keeps_the_retention_it_is_given(both: &Both<'_>) {
    both.store
        .log_event(both.one, "signed_in")
        .await
        .expect("an audit row");
    let now = real_now_ms();
    both.store
        .prune(0, now.saturating_sub(DAY_MS), now)
        .await
        .expect("a prune that reaches nothing");
    assert!(
        !both
            .store
            .events_of(both.one, 100)
            .await
            .expect("a read")
            .is_empty(),
        "an audit row younger than the argument stays"
    );
    both.store
        .prune(0, now + DAY_MS, now)
        .await
        .expect("a prune that reaches it");
    assert!(
        both.store
            .events_of(both.one, 100)
            .await
            .expect("a read")
            .is_empty(),
        "an audit row older than the argument goes"
    );
}

/// A day, for the prune bodies.
const DAY_MS: u64 = 86_400_000;

/// Now by the machine clock. Both stores stamp an audit row with a clock of their own — Postgres
/// with `now()`, the memory store with the clock it was built with, the machine clock by default
/// — so the bodies above compare against the same wall clock the row was written by.
fn real_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(NOW_MS, |since| {
            u64::try_from(since.as_millis()).unwrap_or(NOW_MS)
        })
}

/// A picture name as long as the column takes is kept by both stores, counted in the unit the
/// column counts: 64 characters, which is more than 64 bytes once one of them is not ASCII.
/// Longer than that never reaches a store: the door refuses it, which `tests/door.rs` proves.
///
/// # Panics
/// When the store refuses, or the name comes back changed.
pub async fn s39_a_picture_name_at_the_limit_is_kept(both: &Both<'_>) {
    let main = both
        .store
        .profiles_of(both.one)
        .await
        .expect("the profiles")
        .into_iter()
        .find(|row| row.is_main)
        .expect("the main profile");
    let head = "/avatars/";
    let name = format!("{head}{}", "é".repeat(AVATAR_MAX_CHARS - head.len()));
    assert_eq!(name.chars().count(), AVATAR_MAX_CHARS);
    assert!(name.len() > AVATAR_MAX_CHARS, "the name is not plain ASCII");
    let patch = ProfilePatch {
        avatar: Some(name.clone()),
        ..ProfilePatch::default()
    };
    let after = both
        .store
        .update_profile(both.one, main.id, &patch)
        .await
        .expect("the longest picture name the column takes");
    assert_eq!(after.avatar, name);
}
