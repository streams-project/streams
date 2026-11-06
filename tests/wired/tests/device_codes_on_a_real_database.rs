//! The device codes on a real Postgres, migrated from nothing. A code gets one answer, and of two
//! answers at once only one counts, which memory cannot show.

use std::path::Path;
use std::sync::Arc;

use launcher::postgres::{Postgres, install};
use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{
    AccountId, AccountStore, BuiltInAuth, DevicePollState, Door, DoorError, MailPost, ManualClock,
    Member, NewAccount, PgStore, StepUpMethod, hash_token,
};

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
    listener.local_addr().expect("address").port()
}

async fn database() -> (Postgres, sqlx::PgPool) {
    let dir = std::env::temp_dir().join(format!("wired_device_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let cache = Path::new(env!("CARGO_TARGET_TMPDIR")).join("database");
    let bin = install(&cache).await.expect("the database installs");
    let running = Postgres::start(&bin, &dir.join("data"), free_port()).expect("it starts");
    let pool = sqlx::PgPool::connect(&running.url)
        .await
        .expect("it answers");
    sqlx::migrate!("../../server/migrations")
        .run(&pool)
        .await
        .expect("the migrations apply from nothing");
    (running, pool)
}

struct Person {
    account: AccountId,
    member: Member,
}

async fn person(door: &Door, pool: &sqlx::PgPool, email: &str) -> Person {
    let account = NewAccount {
        id: AccountId::new(),
        email: email.into(),
        first_name: "Pg".into(),
        last_name: "Test".into(),
    };
    sqlx::query("insert into auth.users (id, email) values ($1, $2)")
        .bind(account.id.0)
        .bind(email)
        .execute(pool)
        .await
        .expect("an auth row");
    door.create_account(&account).await.expect("an account");
    let (_, tokens) = door
        .open_session(account.id, "phone", StepUpMethod::Password)
        .await
        .expect("a session");
    Person {
        account: account.id,
        member: door.member(&tokens.access).await.expect("a member"),
    }
}

const DECIDED: DoorError = DoorError::Refused("device_decided");
const ADDRESS: &str = "192.0.2.1";

#[tokio::test]
async fn n11_a_device_code_takes_one_answer_on_a_real_database() {
    let (running, pool) = database().await;
    let clock = Arc::new(ManualClock::new(1_000_000));
    let store: Arc<dyn AccountStore> = Arc::new(PgStore::new(pool.clone()));
    let post = MailPost {
        mail: None,
        name: String::from("Wired"),
        help_email: String::new(),
        site_url: String::new(),
        help_url: String::new(),
    };
    let door = Door::new(store.clone(), clock.clone(), DEFAULT_LIMITS)
        .with_auth(Arc::new(BuiltInAuth::new(pool.clone(), Arc::new(post))));
    let ada = person(&door, &pool, "ada@example.test").await;
    let eve = person(&door, &pool, "eve@example.test").await;
    let approve = |who: &Person, code: &str| {
        let (door, member, code) = (door.clone(), who.member.clone(), code.to_owned());
        async move { door.device_approve(&member, &code, ADDRESS).await }
    };
    let deny = |who: &Person, code: &str| {
        let (door, member, code) = (door.clone(), who.member.clone(), code.to_owned());
        async move { door.device_deny(&member, &code, ADDRESS).await }
    };

    // Nobody has answered, so a poll waits, and an unknown or a lapsed code is not found.
    let tv = door.device_start("Living room TV").await.expect("a code");
    assert_eq!(
        door.device_poll(&tv.device_code).await,
        Ok(DevicePollState::Pending)
    );
    let hash = hash_token(&tv.device_code);
    let row = store.device_code(&hash).await.expect("read").expect("row");
    assert_eq!(row.device_name, "Living room TV");
    assert_eq!(row.last_poll_ms, Some(1_000_000));
    assert_eq!(
        approve(&ada, "NOTACODE").await,
        Err(DoorError::NotFound),
        "an unknown code"
    );

    // A poll that read the row before the answer only writes its own time. The answer stays.
    approve(&ada, &tv.user_code).await.expect("approved");
    store
        .note_device_poll(&hash, 1_000_500)
        .await
        .expect("a late poll");
    let row = store.device_code(&hash).await.expect("read").expect("row");
    assert_eq!(row.account, Some(ada.account));
    assert_eq!(row.last_poll_ms, Some(1_000_500));

    // The first answer stands.
    assert_eq!(approve(&eve, &tv.user_code).await, Err(DECIDED));
    assert_eq!(approve(&ada, &tv.user_code).await, Err(DECIDED));
    assert_eq!(deny(&ada, &tv.user_code).await, Err(DECIDED));
    clock.advance_ms(5000);
    match door.device_poll(&tv.device_code).await {
        Ok(DevicePollState::Approved(found)) => assert_eq!(found.identity.account, ada.account),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        door.device_poll(&tv.device_code).await,
        Ok(DevicePollState::Expired),
        "a device code works once"
    );

    // Refused stays refused.
    clock.advance_ms(60_000);
    let tv = door.device_start("Bedroom TV").await.expect("a code");
    deny(&eve, &tv.user_code).await.expect("refused");
    assert_eq!(approve(&ada, &tv.user_code).await, Err(DECIDED));
    assert_eq!(deny(&ada, &tv.user_code).await, Err(DECIDED));
    assert_eq!(
        door.device_poll(&tv.device_code).await,
        Ok(DevicePollState::Denied)
    );

    // A code past its life takes no answer.
    clock.advance_ms(60_000);
    let tv = door.device_start("Kitchen TV").await.expect("a code");
    clock.advance_ms(901_000);
    assert_eq!(approve(&ada, &tv.user_code).await, Err(DoorError::NotFound));

    // Two people at once, many times over: one answer lands, and the row says whose.
    for round in 0..40 {
        clock.advance_ms(60_000);
        let tv = door.device_start("Hall TV").await.expect("a code");
        let (one, two) = tokio::join!(
            tokio::spawn(approve(&ada, &tv.user_code)),
            tokio::spawn(approve(&eve, &tv.user_code)),
        );
        let (one, two) = (one.expect("a task"), two.expect("a task"));
        let winner = match (&one, &two) {
            (Ok(()), Err(lost)) if *lost == DECIDED => ada.account,
            (Err(lost), Ok(())) if *lost == DECIDED => eve.account,
            other => panic!("round {round}: {other:?}"),
        };
        let row = store
            .device_code(&hash_token(&tv.device_code))
            .await
            .expect("read")
            .expect("row");
        assert_eq!(row.account, Some(winner), "round {round}");
    }
    drop(running);
}
