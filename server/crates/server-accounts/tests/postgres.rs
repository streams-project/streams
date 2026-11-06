//! Needs a real database: set `TEST_DATABASE_URL` and run with `--ignored`. The database needs an
//! `auth.users` table with an `id uuid` column and the chain in `migrations/` applied.

use std::sync::Arc;

use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{
    AccountId, AccountStore, Door, DoorError, ManualClock, NewAccount, PgStore, ProfileKind,
    StepUpMethod,
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
