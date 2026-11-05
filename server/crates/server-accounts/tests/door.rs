//! Door, session and profile rules on the memory store.

use std::sync::Arc;

use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{
    AccountId, Consent, Door, DoorError, IssuedTokens, ManualClock, Member, MemoryStore,
    NewAccount, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch, StepUpMethod,
};

struct Bench {
    store: Arc<MemoryStore>,
    clock: Arc<ManualClock>,
    door: Door,
    account: AccountId,
}

async fn bench() -> Bench {
    let store = Arc::new(MemoryStore::new());
    let clock = Arc::new(ManualClock::new(1_000_000));
    let door = Door::new(store.clone(), clock.clone(), DEFAULT_LIMITS);
    let account = NewAccount {
        id: AccountId::new(),
        email: "ada@example.test".into(),
        first_name: "Ada".into(),
        last_name: "L".into(),
    };
    door.create_account(&account).await.unwrap();
    Bench {
        store,
        clock,
        door,
        account: account.id,
    }
}

impl Bench {
    async fn sign_in(&self) -> (Member, IssuedTokens) {
        let (_, tokens) = self
            .door
            .open_session(self.account, "test device", StepUpMethod::Password)
            .await
            .unwrap();
        let member = self.door.member(&tokens.access).await.unwrap();
        (member, tokens)
    }

    async fn profile(&self, member: &Member, kind: ProfileKind) -> ProfileId {
        let all = self.door.profiles(member).await.unwrap();
        all.iter().find(|row| row.kind == kind).unwrap().id
    }
}

#[tokio::test]
async fn h30_tmk1_a_session_with_no_profile_gets_no_scope() {
    let bench = bench().await;
    let (member, _) = bench.sign_in().await;
    assert_eq!(
        bench.door.scope(&member).await,
        Err(DoorError::ProfileRequired)
    );
}

#[tokio::test]
async fn h31_tmp1_the_scope_comes_from_the_session_row() {
    let bench = bench().await;
    let (member, tokens) = bench.sign_in().await;
    let kids = bench.profile(&member, ProfileKind::Kids).await;
    bench.door.enter_profile(&member, kids, None).await.unwrap();
    let member = bench.door.member(&tokens.access).await.unwrap();
    let scope = bench.door.scope(&member).await.unwrap();
    assert_eq!(scope.profile(), kids);
    assert_eq!(scope.age_limit(), "PG");
    assert_eq!(scope.kind(), ProfileKind::Kids);
}

#[tokio::test]
async fn h31_a_profile_of_another_account_is_not_found() {
    let first = bench().await;
    let (member, _) = first.sign_in().await;
    let other = NewAccount {
        id: AccountId::new(),
        email: "b@example.test".into(),
        first_name: "B".into(),
        last_name: "C".into(),
    };
    first.door.create_account(&other).await.unwrap();
    let (_, tokens) = first
        .door
        .open_session(other.id, "other", StepUpMethod::Password)
        .await
        .unwrap();
    let stranger = first.door.member(&tokens.access).await.unwrap();
    let theirs = first.door.profiles(&stranger).await.unwrap()[0].id;
    assert_eq!(
        first.door.enter_profile(&member, theirs, None).await,
        Err(DoorError::NotFound)
    );
}

#[tokio::test]
async fn h60_tmp2_deleting_the_active_profile_brings_the_picker_back() {
    let bench = bench().await;
    let (member, tokens) = bench.sign_in().await;
    let extra = bench
        .door
        .create_profile(&member, "Tom", false)
        .await
        .unwrap();
    bench
        .door
        .enter_profile(&member, extra.id, None)
        .await
        .unwrap();
    let member = bench.door.member(&tokens.access).await.unwrap();
    bench.door.delete_profile(&member, extra.id).await.unwrap();
    let member = bench.door.member(&tokens.access).await.unwrap();
    assert_eq!(
        bench.door.scope(&member).await,
        Err(DoorError::ProfileRequired)
    );
}

#[tokio::test]
async fn s43_a_pin_is_checked_on_the_server_and_locks() {
    let bench = bench().await;
    let (member, _) = bench.sign_in().await;
    let main = bench.profile(&member, ProfileKind::Adult).await;
    bench
        .door
        .set_pin(&member, main, Some("1234"))
        .await
        .unwrap();
    assert_eq!(
        bench.door.enter_profile(&member, main, None).await,
        Err(DoorError::PinRequired)
    );
    for _ in 0..5 {
        assert_eq!(
            bench.door.enter_profile(&member, main, Some("0000")).await,
            Err(DoorError::PinWrong)
        );
    }
    assert!(matches!(
        bench.door.enter_profile(&member, main, Some("1234")).await,
        Err(DoorError::PinLocked { .. })
    ));
    assert!(
        bench
            .store
            .events(bench.account)
            .contains(&"pin_locked".to_owned())
    );
    bench.clock.advance_ms(900_001);
    assert!(
        bench
            .door
            .enter_profile(&member, main, Some("1234"))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn s42_a_step_up_lasts_300_seconds_and_refresh_never_renews_it() {
    let bench = bench().await;
    let (member, tokens) = bench.sign_in().await;
    let main = bench.profile(&member, ProfileKind::Adult).await;
    assert!(bench.door.has_step_up(&member));
    bench.clock.advance_ms(301_000);
    let renewed = bench.door.refresh(&tokens.refresh).await.unwrap();
    let member = bench.door.member(&renewed.access).await.unwrap();
    assert!(!bench.door.has_step_up(&member));
    assert_eq!(
        bench.door.set_pin(&member, main, Some("1234")).await,
        Err(DoorError::StepUpRequired)
    );
}

#[tokio::test]
async fn h27_tmp6_limits_need_a_step_up() {
    let bench = bench().await;
    let (member, _) = bench.sign_in().await;
    let main = bench.profile(&member, ProfileKind::Adult).await;
    bench.clock.advance_ms(301_000);
    let member = bench
        .door
        .member(&bench.sign_in_no_step_up().await)
        .await
        .unwrap();
    let row = ProfileLimitsRow {
        age_limit: "12".into(),
        blocked_names: vec![],
    };
    assert_eq!(
        bench.door.set_limits(&member, main, row).await,
        Err(DoorError::StepUpRequired)
    );
}

impl Bench {
    async fn sign_in_no_step_up(&self) -> String {
        let (_, tokens) = self
            .door
            .open_device_session(self.account, "device code")
            .await
            .unwrap();
        tokens.access
    }
}

#[tokio::test]
async fn tmt11_a_device_code_session_has_no_step_up_power() {
    let bench = bench().await;
    let access = bench.sign_in_no_step_up().await;
    let member = bench.door.member(&access).await.unwrap();
    assert!(!bench.door.has_step_up(&member));
}

#[tokio::test]
async fn s40_main_and_last_profile_rules() {
    let bench = bench().await;
    let (member, _) = bench.sign_in().await;
    let main = bench.profile(&member, ProfileKind::Adult).await;
    let kids = bench.profile(&member, ProfileKind::Kids).await;
    assert_eq!(
        bench.door.delete_profile(&member, main).await,
        Err(DoorError::Refused("main_profile"))
    );
    assert_eq!(
        bench.door.set_main_profile(&member, kids).await,
        Err(DoorError::Refused("main_profile_kids"))
    );
    let patch = ProfilePatch {
        kind: Some(ProfileKind::Kids),
        ..ProfilePatch::default()
    };
    assert_eq!(
        bench.door.update_profile(&member, main, patch).await,
        Err(DoorError::Refused("main_profile_kids"))
    );
    bench.door.delete_profile(&member, kids).await.unwrap();
    assert_eq!(
        bench.door.delete_profile(&member, main).await,
        Err(DoorError::Refused("last_profile"))
    );
}

#[tokio::test]
async fn s38_five_profiles_kids_included_and_names_are_unique() {
    let bench = bench().await;
    let (member, _) = bench.sign_in().await;
    assert_eq!(
        bench.door.create_profile(&member, " ada ", false).await,
        Err(DoorError::Refused("profile_name_taken"))
    );
    for name in ["One", "Two", "Three"] {
        bench
            .door
            .create_profile(&member, name, false)
            .await
            .unwrap();
    }
    assert_eq!(
        bench.door.create_profile(&member, "Six", false).await,
        Err(DoorError::Refused("profile_limit"))
    );
}

#[tokio::test]
async fn tmt10_an_outage_is_not_a_sign_out() {
    let bench = bench().await;
    let (_, tokens) = bench.sign_in().await;
    bench.store.set_down(true);
    assert!(matches!(
        bench.door.member(&tokens.access).await,
        Err(DoorError::Unavailable { .. })
    ));
    assert!(matches!(
        bench.door.refresh(&tokens.refresh).await,
        Err(DoorError::Unavailable { .. })
    ));
    bench.store.set_down(false);
    assert!(bench.door.member(&tokens.access).await.is_ok());
}

#[tokio::test]
async fn tmt2_a_revoked_session_stops_on_the_next_request() {
    let bench = bench().await;
    let (member, tokens) = bench.sign_in().await;
    bench.door.sign_out(&member, false).await.unwrap();
    assert_eq!(
        bench.door.member(&tokens.access).await,
        Err(DoorError::Unauthorized)
    );
}

#[tokio::test]
async fn s58_the_access_token_lapses_after_fifteen_minutes() {
    let bench = bench().await;
    let (_, tokens) = bench.sign_in().await;
    bench.clock.advance_ms(900_000);
    assert_eq!(
        bench.door.member(&tokens.access).await,
        Err(DoorError::Unauthorized)
    );
    assert!(bench.door.refresh(&tokens.refresh).await.is_ok());
}

#[tokio::test]
async fn tmt1_a_lost_reply_gets_the_same_next_token() {
    let bench = bench().await;
    let (_, tokens) = bench.sign_in().await;
    let first = bench.door.refresh(&tokens.refresh).await.unwrap();
    let again = bench.door.refresh(&tokens.refresh).await.unwrap();
    assert_eq!(first.refresh, again.refresh);
    // Using next kills the old current, and reuse of a dead token ends the session.
    let third = bench.door.refresh(&first.refresh).await.unwrap();
    let fourth = bench.door.refresh(&third.refresh).await.unwrap();
    assert_eq!(
        bench.door.refresh(&tokens.refresh).await,
        Err(DoorError::SessionEnded)
    );
    assert_eq!(
        bench.door.refresh(&fourth.refresh).await,
        Err(DoorError::SessionEnded)
    );
}

#[tokio::test]
async fn tmt1_strict_alternation_ends_past_the_reuse_window() {
    // A thief and the device hold one token and take turns, so no dead token is ever shown.
    let bench = bench().await;
    let (_, tokens) = bench.sign_in().await;
    let thief = bench.door.refresh(&tokens.refresh).await.unwrap();
    let device = bench.door.refresh(&tokens.refresh).await.unwrap();
    assert_eq!(thief.refresh, device.refresh);
    let thief = bench.door.refresh(&thief.refresh).await.unwrap();
    bench
        .clock
        .advance_ms(u64::from(DEFAULT_LIMITS.refresh_reuse_s) * 1000 + 1);
    assert_eq!(
        bench.door.refresh(&device.refresh).await,
        Err(DoorError::SessionEnded)
    );
    assert_eq!(
        bench.door.refresh(&thief.refresh).await,
        Err(DoorError::SessionEnded)
    );
}

#[tokio::test]
async fn s126_a_session_idle_for_thirty_days_ends() {
    let bench = bench().await;
    let (_, tokens) = bench.sign_in().await;
    bench
        .clock
        .advance_ms(u64::from(DEFAULT_LIMITS.idle_s) * 1000 + 1);
    assert_eq!(
        bench.door.refresh(&tokens.refresh).await,
        Err(DoorError::SessionEnded)
    );
}

#[tokio::test]
async fn tmt9_invented_tokens_are_metered_before_a_lookup() {
    let bench = bench().await;
    for _ in 0..600 {
        bench.door.meter_call("203.0.113.9").unwrap();
    }
    assert!(matches!(
        bench.door.meter_call("203.0.113.9"),
        Err(DoorError::RateLimited { .. })
    ));
    assert!(bench.door.meter_call("203.0.113.10").is_ok());
}

#[tokio::test]
async fn tmt7_sign_in_tries_are_metered_by_name_and_address() {
    let bench = bench().await;
    for n in 0..10 {
        bench
            .door
            .meter_sign_in("Ada@example.test", &format!("198.51.100.{n}"))
            .unwrap();
    }
    assert!(matches!(
        bench
            .door
            .meter_sign_in(" ada@example.test ", "198.51.100.99"),
        Err(DoorError::RateLimited { .. })
    ));
}

#[tokio::test]
async fn tmt1_a_device_many_steps_behind_a_thief_ends_the_session() {
    let bench = bench().await;
    let (_, tokens) = bench.sign_in().await;
    let mut thief = bench.door.refresh(&tokens.refresh).await.unwrap();
    for _ in 0..20 {
        thief = bench.door.refresh(&thief.refresh).await.unwrap();
    }
    assert_eq!(
        bench.door.refresh(&tokens.refresh).await,
        Err(DoorError::SessionEnded)
    );
    assert_eq!(
        bench.door.refresh(&thief.refresh).await,
        Err(DoorError::SessionEnded)
    );
}

#[tokio::test]
async fn h31_a_stale_copy_of_a_session_cannot_undo_a_consent_or_a_profile() {
    let bench = bench().await;
    let (held, tokens) = bench.sign_in().await;
    let door = &bench.door;
    door.set_consent(&held, Consent { personal: true })
        .await
        .unwrap();
    let profile = bench.profile(&held, ProfileKind::Adult).await;
    door.enter_profile(&held, profile, None).await.unwrap();
    door.record_step_up(&held, StepUpMethod::Code)
        .await
        .unwrap();
    let scope = door
        .scope(&door.member(&tokens.access).await.unwrap())
        .await
        .unwrap();
    assert!(scope.consent().personal, "entering a profile kept consent");
    door.refresh(&tokens.refresh).await.unwrap();
    let after = door
        .store()
        .session(held.session_id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        after.active_profile,
        Some(profile),
        "a refresh kept the profile"
    );
    assert!(after.consent.personal, "a refresh kept consent");
}
