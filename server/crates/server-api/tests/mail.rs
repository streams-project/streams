//! The mail hook through the real router, with a provider in memory.

mod common;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use common::{Harness, call, harness};
use serde_json::json;
use server_accounts::mail::MemoryMail;
use server_accounts::mail::webhook::signature_header;

const SECRET: &str = "v1,whsec_c2FtcGxlLXNlY3JldA==";

fn server(mail: Arc<MemoryMail>) -> Harness {
    let mut server = harness(&[
        ("MAIL_HOOK_SECRET", SECRET),
        ("SITE_ADDRESS", "https://site.host.example"),
        ("BRAND_NAME", "Sample"),
        ("SUPPORT_EMAIL", "help@host.example"),
    ]);
    Arc::get_mut(&mut server.state).expect("one owner").mail = Some(mail);
    server
}

fn body(action: &str) -> String {
    json!({
        "user": { "email": "ada@example.test", "user_metadata": { "lang": "de", "first_name": "Ada" } },
        "email_data": { "token": "12345678", "token_hash": "abc", "email_action_type": action }
    })
    .to_string()
}

async fn post(server: &Harness, text: &str, age_s: i64) -> common::Answer {
    let now = i64::try_from(common::START_MS / 1000).unwrap() - age_s;
    let stamp = now.to_string();
    let signature = signature_header(SECRET, "m1", &stamp, text).unwrap();
    call(Method::POST, "/internal/mail")
        .no_version()
        .header("webhook-id", "m1")
        .header("webhook-timestamp", &stamp)
        .header("webhook-signature", &signature)
        .raw(text)
        .on(&server.state)
        .await
}

#[tokio::test]
async fn c23_s59_a_signed_hook_writes_the_viewers_language_and_sends_it() {
    let mail = Arc::new(MemoryMail::default());
    let server = server(mail.clone());
    let answer = post(&server, &body("recovery"), 0).await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let sent = mail.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "ada@example.test");
    assert!(
        sent[0]
            .html
            .contains("https://site.host.example/auth/confirm?token_hash=abc")
    );
    assert!(
        !sent[0].subject.contains("reset"),
        "German, not English: {}",
        sent[0].subject
    );
}

#[tokio::test]
async fn s59_a_bad_signature_an_old_time_and_an_unknown_action_are_refused_in_the_hook_shape() {
    let mail = Arc::new(MemoryMail::default());
    let server = server(mail.clone());
    let forged = call(Method::POST, "/internal/mail")
        .no_version()
        .header("webhook-id", "m1")
        .header("webhook-timestamp", "1")
        .header("webhook-signature", "v1,AAAA")
        .raw(&body("recovery"))
        .on(&server.state)
        .await;
    assert_eq!(forged.status, StatusCode::UNAUTHORIZED);
    assert_eq!(forged.body["error"]["http_code"], 401);
    let old = post(&server, &body("recovery"), 301).await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED, "300 s tolerance");
    let unknown = post(&server, &body("something_new"), 0).await;
    assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
    assert!(mail.sent.lock().unwrap().is_empty());
}

#[tokio::test]
async fn s213_with_no_provider_mail_is_off() {
    let server = harness(&[]);
    let answer = call(Method::POST, "/internal/mail")
        .no_version()
        .raw("{}")
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
}
