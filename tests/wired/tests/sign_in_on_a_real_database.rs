//! Built in sign in against a real Postgres, migrated from nothing.

use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use launcher::postgres::{Postgres, install};
use server_accounts::{AuthError, AuthService, BuiltInAuth, CodePost};

#[derive(Default)]
struct Letterbox(Mutex<Vec<(String, bool)>>);

#[async_trait]
impl CodePost for Letterbox {
    async fn deliver(&self, _email: &str, code: &str, recovery: bool) -> Result<(), AuthError> {
        self.0
            .lock()
            .expect("the box")
            .push((code.to_owned(), recovery));
        Ok(())
    }
}

impl Letterbox {
    fn last(&self) -> String {
        let letters = self.0.lock().expect("the box");
        letters.last().expect("a letter").0.clone()
    }
    fn count(&self) -> usize {
        self.0.lock().expect("the box").len()
    }
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
    listener.local_addr().expect("address").port()
}

async fn database(name: &str) -> (Postgres, sqlx::PgPool) {
    let dir = std::env::temp_dir().join(format!("wired_auth_{name}_{}", std::process::id()));
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

const EMAIL: &str = "ada@example.test";
const PASSWORD: &str = "correct-horse-battery";

#[tokio::test]
async fn sign_in_holds_on_a_real_database() {
    let (running, pool) = database("all").await;
    let post = Arc::new(Letterbox::default());
    let auth = BuiltInAuth::new(pool.clone(), post.clone());

    // Joining, and the address is one person whatever its case.
    let ada = auth.sign_up(EMAIL, PASSWORD).await.expect("joins");
    assert_eq!(
        auth.sign_up("ADA@example.test", PASSWORD).await.err(),
        Some(AuthError::Taken)
    );
    assert_eq!(
        auth.check_password(" Ada@Example.test ", PASSWORD)
            .await
            .expect("signs in")
            .account,
        ada.account
    );
    assert_eq!(
        auth.check_password(EMAIL, "wrong-password").await.err(),
        Some(AuthError::Refused)
    );
    assert_eq!(
        auth.check_password("nobody@example.test", PASSWORD)
            .await
            .err(),
        Some(AuthError::Refused)
    );

    // A password carried over from the old service signs in unchanged.
    let carried = bcrypt::hash("an-old-password", 10).expect("a hash");
    sqlx::query("insert into auth.users (email, encrypted_password) values ($1, $2)")
        .bind("old@example.test")
        .bind(carried)
        .execute(&pool)
        .await
        .expect("an old account");
    assert!(
        auth.check_password("old@example.test", "an-old-password")
            .await
            .is_ok()
    );

    // A code works once, and no stored form of it can be read back.
    auth.send_code(EMAIL).await.expect("a code");
    let code = post.last();
    let stored: String = sqlx::query_scalar("select code_hash from auth.login_codes")
        .fetch_one(&pool)
        .await
        .expect("a row");
    assert!(!stored.contains(&code));
    assert_eq!(
        auth.check_code(EMAIL, &code).await.expect("right").account,
        ada.account
    );
    assert_eq!(
        auth.check_code(EMAIL, &code).await.err(),
        Some(AuthError::Refused)
    );

    // Five wrong tries end a code, even for the person holding the right one.
    auth.send_code(EMAIL).await.expect("a code");
    let code = post.last();
    let wrong = if code == "00000000" {
        "00000001"
    } else {
        "00000000"
    };
    for _ in 0..5 {
        assert_eq!(
            auth.check_code(EMAIL, wrong).await.err(),
            Some(AuthError::Refused)
        );
    }
    assert_eq!(
        auth.check_code(EMAIL, &code).await.err(),
        Some(AuthError::Refused)
    );

    // An expired code is refused.
    auth.send_code(EMAIL).await.expect("a code");
    let code = post.last();
    sqlx::query("update auth.login_codes set expires_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .expect("expired");
    assert_eq!(
        auth.check_code(EMAIL, &code).await.err(),
        Some(AuthError::Refused)
    );

    // A sign in code cannot finish a recovery.
    auth.send_code(EMAIL).await.expect("a code");
    let sign_in_code = post.last();
    assert_eq!(
        auth.finish_recovery(EMAIL, &sign_in_code, "a-new-password")
            .await
            .err(),
        Some(AuthError::Refused)
    );

    // Recovery changes the password.
    auth.send_recovery(EMAIL).await.expect("a code");
    let code = post.last();
    auth.finish_recovery(EMAIL, &code, "a-new-password")
        .await
        .expect("recovers");
    assert!(auth.check_password(EMAIL, "a-new-password").await.is_ok());
    assert_eq!(
        auth.check_password(EMAIL, PASSWORD).await.err(),
        Some(AuthError::Refused)
    );

    // Asking for an unknown address looks the same and sends nothing.
    let sent = post.count();
    assert!(auth.send_code("nobody@example.test").await.is_ok());
    assert_eq!(post.count(), sent);

    // Changes stay on the one person, and a taken address is refused.
    auth.set_phone(ada.account, "+15550100")
        .await
        .expect("phone");
    assert_eq!(
        auth.identity(ada.account)
            .await
            .expect("found")
            .phone
            .as_deref(),
        Some("+15550100")
    );
    assert_eq!(
        auth.set_email(ada.account, "old@example.test").await.err(),
        Some(AuthError::Taken)
    );
    auth.set_email(ada.account, "new@example.test")
        .await
        .expect("moves");
    assert_eq!(
        auth.find_email("NEW@example.test").await.expect("asked"),
        Some(ada.account)
    );
    auth.delete(ada.account).await.expect("gone");
    assert_eq!(
        auth.find_email("new@example.test").await.expect("asked"),
        None
    );

    drop(running);
}
