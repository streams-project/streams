//! Needs a real database: set `TEST_DATABASE_URL` and run with `--ignored`. The database needs an
//! `auth.users` table with an `id uuid` column and the chain in `migrations/` applied.
//!
//! The bodies in `conformance` are the ones that must answer the same here as on the memory store,
//! and `stores.rs` runs the very same bodies there. Everything in this file waits for a database,
//! so a rule proved only here is not proved on any machine without one: `tests/wired` is where
//! these belong, and the handoff for that move names them.

mod conformance;

use std::sync::Arc;

use async_trait::async_trait;
use conformance::{Both, make_account};
use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{
    AccountId, AccountStore, AuthError, AuthService, BuiltInAuth, CodePost, Door, DoorError,
    ManualClock, NewAccount, PgStore, ProfileKind, StepUpMethod,
};

async fn door() -> (Door, AccountId) {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    let account = NewAccount {
        id: AccountId::new(),
        email: "pg@example.test".into(),
        first_name: "Pg".into(),
        last_name: "Test".into(),
    };
    sqlx::query("insert into auth.users (id) values ($1)")
        .bind(account.id.0)
        .execute(&pool)
        .await
        .expect("auth row");
    let store: Arc<dyn AccountStore> = Arc::new(PgStore::new(pool));
    let door = Door::new(store, Arc::new(ManualClock::new(1_000_000)), DEFAULT_LIMITS);
    door.create_account(&account).await.expect("account");
    (door, account.id)
}

async fn pool() -> sqlx::PgPool {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    sqlx::PgPool::connect(&url).await.expect("connect")
}

/// A store on a real database, with two accounts whose `auth.users` rows are there.
async fn both() -> (Arc<dyn AccountStore>, AccountId, AccountId) {
    let pool = pool().await;
    let store: Arc<dyn AccountStore> = Arc::new(PgStore::new(pool.clone()));
    let mut made = Vec::new();
    for _ in 0..2 {
        let id = AccountId::new();
        sqlx::query("insert into auth.users (id) values ($1)")
            .bind(id.0)
            .execute(&pool)
            .await
            .expect("auth row");
        made.push(make_account(&store, &format!("{id}@example.test")).await);
    }
    (store, made[0], made[1])
}

/// A post that keeps the codes it was given, so a test can read one.
#[derive(Default)]
struct Letterbox(std::sync::Mutex<Vec<String>>);

#[async_trait]
impl CodePost for Letterbox {
    async fn deliver(&self, _email: &str, code: &str, _recovery: bool) -> Result<(), AuthError> {
        self.0.lock().expect("the box").push(code.to_owned());
        Ok(())
    }
}

#[tokio::test]
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
async fn tmt13_one_right_code_signs_in_once_however_many_ask_at_once() {
    let pool = pool().await;
    let post = Arc::new(Letterbox::default());
    let auth = BuiltInAuth::new(pool, post.clone());
    let email = format!("{}@example.test", AccountId::new());
    auth.sign_up(&email, "a-long-enough-password")
        .await
        .expect("an account");
    auth.send_code(&email).await.expect("a code");
    let code = post
        .0
        .lock()
        .expect("the box")
        .last()
        .cloned()
        .expect("a code");
    let auth = Arc::new(auth);
    let (first, second) = (Arc::clone(&auth), Arc::clone(&auth));
    let (one, two) = (email.clone(), email.clone());
    let (right, code_again) = (code.clone(), code.clone());
    let tries = tokio::join!(
        tokio::spawn(async move { first.check_code(&one, &right).await }),
        tokio::spawn(async move { second.check_code(&two, &code_again).await }),
    );
    let signed = [tries.0, tries.1]
        .into_iter()
        .filter(|try_| matches!(try_, Ok(Ok(_))))
        .count();
    assert_eq!(signed, 1, "a code is taken by one submission only");
}

#[tokio::test]
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
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
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
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
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
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
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
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
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
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
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
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
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
async fn s39_a_picture_name_at_the_limit_is_kept() {
    let (store, one, two) = both().await;
    let both = Both {
        store: &store,
        one,
        two,
    };
    conformance::s39_a_picture_name_at_the_limit_is_kept(&both).await;
}

#[tokio::test]
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
async fn s38_parallel_inserts_cannot_pass_the_limit() {
    let (door, account) = door().await;
    let (_, tokens) = door
        .open_session(account, "pg", StepUpMethod::Password)
        .await
        .expect("session");
    let member = door.member(&tokens.access).await.expect("member");
    let tries = (0..8).map(|n| {
        let (door, member) = (door.clone(), member.clone());
        tokio::spawn(async move { door.create_profile(&member, &format!("P{n}"), false).await })
    });
    let mut made = 0;
    for task in tries {
        if task.await.expect("task").is_ok() {
            made += 1;
        }
    }
    assert_eq!(made, 3, "two rows exist, five is the limit");
}

#[tokio::test]
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
async fn h60_tmp2_deleting_the_active_profile_clears_the_session_row() {
    let (door, account) = door().await;
    let (_, tokens) = door
        .open_session(account, "pg", StepUpMethod::Password)
        .await
        .expect("session");
    let member = door.member(&tokens.access).await.expect("member");
    let kids = door.profiles(&member).await.expect("profiles");
    let kids = kids
        .iter()
        .find(|row| row.kind == ProfileKind::Kids)
        .expect("kids")
        .id;
    door.enter_profile(&member, kids, None)
        .await
        .expect("enter");
    let member = door.member(&tokens.access).await.expect("member");
    door.delete_profile(&member, kids).await.expect("delete");
    let member = door.member(&tokens.access).await.expect("member");
    assert_eq!(door.scope(&member).await, Err(DoorError::ProfileRequired));
}
