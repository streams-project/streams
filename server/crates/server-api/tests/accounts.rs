//! Sign in, step up, device sign in and account routes, with an auth service in memory.

mod common;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use common::{Answer, call, get, harness_with_auth};
use contract::account::{DeviceCode, DeviceTokenAnswer, SignedIn};
use serde_json::json;
use server_accounts::auth::MEMORY_CODE;
use server_accounts::{AuthService, MemoryAuth};

fn signed_in(answer: Answer) -> SignedIn {
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    serde_json::from_value(answer.body).expect("SignedIn")
}

fn sign_up_body(email: &str) -> serde_json::Value {
    json!({ "email": email, "password": "long-enough", "first_name": "Ada", "last_name": "Lace" })
}

async fn new_account(server: &common::Harness, email: &str) -> SignedIn {
    let answer = call(Method::POST, "/v1/auth/signup")
        .json(sign_up_body(email))
        .on(&server.state)
        .await;
    signed_in(answer)
}

fn post(path: &str) -> common::Call {
    call(Method::POST, path)
}

#[tokio::test]
async fn s37_sign_up_makes_a_main_and_a_kids_profile() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let list = get("/v1/profiles")
        .token(&signed.access_token)
        .on(&server.state)
        .await;
    let names: Vec<&str> = list.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|profile| profile["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Ada", "Kids"]);
}

#[tokio::test]
async fn s58_c70_a_short_password_or_bad_address_stops_sign_up() {
    let server = harness_with_auth(&[]);
    let mut body = sign_up_body("a@b.test");
    body["password"] = json!("short");
    let answer = post("/v1/auth/signup").json(body).on(&server.state).await;
    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    let answer = post("/v1/auth/signup")
        .json(sign_up_body("no-at-sign"))
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn t13_an_unknown_address_and_a_wrong_password_answer_alike() {
    let server = harness_with_auth(&[]);
    new_account(&server, "ada@example.test").await;
    let wrong = post("/v1/auth/password")
        .json(json!({ "email": "ada@example.test", "password": "wrong-password" }))
        .on(&server.state)
        .await;
    let unknown = post("/v1/auth/password")
        .json(json!({ "email": "nobody@example.test", "password": "wrong-password" }))
        .on(&server.state)
        .await;
    assert_eq!((wrong.status, wrong.body), (unknown.status, unknown.body));
}

#[tokio::test]
async fn t13_a_code_and_a_recovery_answer_the_same_for_any_address() {
    let server = harness_with_auth(&[]);
    new_account(&server, "ada@example.test").await;
    for path in ["/v1/auth/code", "/v1/auth/recovery/start"] {
        let known = post(path)
            .json(json!({ "email": "ada@example.test" }))
            .on(&server.state)
            .await;
        let unknown = post(path)
            .json(json!({ "email": "nobody@example.test" }))
            .on(&server.state)
            .await;
        assert_eq!(
            (known.status, known.body),
            (unknown.status, unknown.body),
            "{path}"
        );
    }
}

#[tokio::test]
async fn s58_a_code_signs_in() {
    let server = harness_with_auth(&[]);
    new_account(&server, "ada@example.test").await;
    let answer = post("/v1/auth/code/verify")
        .json(json!({ "email": "ada@example.test", "code": MEMORY_CODE }))
        .on(&server.state)
        .await;
    assert!(!signed_in(answer).access_token.is_empty());
}

#[tokio::test]
async fn t7_password_tries_hit_429_on_the_account_and_the_address() {
    let server = harness_with_auth(&[]);
    let body = json!({ "email": "ada@example.test", "password": "wrong-password" });
    for _ in 0..10 {
        let answer = post("/v1/auth/password")
            .json(body.clone())
            .on(&server.state)
            .await;
        assert_eq!(answer.status, StatusCode::FORBIDDEN);
    }
    let over = post("/v1/auth/password").json(body).on(&server.state).await;
    assert_eq!(over.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(over.headers.contains_key("retry-after"));
}

#[tokio::test]
async fn s42_a_refresh_gives_a_working_token_and_never_renews_a_step_up() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    server.clock.advance_ms(600_000);
    let fresh = post("/v1/auth/refresh")
        .json(json!({ "refresh_token": signed.refresh_token }))
        .on(&server.state)
        .await;
    let fresh = signed_in(fresh);
    assert_eq!(fresh.user_id, signed.user_id);
    let export = get("/v1/account/export")
        .token(&fresh.access_token)
        .on(&server.state)
        .await;
    assert_eq!(export.status, StatusCode::FORBIDDEN);
    assert_eq!(export.code(), "step_up_required");
}

#[tokio::test]
async fn t10_a_refused_refresh_ends_the_session_by_name() {
    let server = harness_with_auth(&[]);
    let answer = post("/v1/auth/refresh")
        .json(json!({ "refresh_token": "made-up.token" }))
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED);
    assert_eq!(answer.code(), "session_ended");
}

#[tokio::test]
async fn s42_t12_export_and_delete_need_a_fresh_step_up_and_confirm_gives_it() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    server.clock.advance_ms(301_000);
    assert_eq!(
        get("/v1/account/export")
            .token(token)
            .on(&server.state)
            .await
            .code(),
        "step_up_required"
    );
    let refused = call(Method::DELETE, "/v1/account")
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(refused.code(), "step_up_required");
    let bad = post("/v1/auth/confirm")
        .token(token)
        .json(json!({ "password": "wrong" }))
        .on(&server.state)
        .await;
    assert_eq!(bad.status, StatusCode::FORBIDDEN);
    let good = post("/v1/auth/confirm")
        .token(token)
        .json(json!({ "password": "long-enough" }))
        .on(&server.state)
        .await;
    assert_eq!(good.body["valid_s"], 300, "{}", good.body);
    let export = get("/v1/account/export")
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(export.status, StatusCode::OK, "{}", export.body);
    assert_eq!(export.body["profiles"].as_array().unwrap().len(), 2);
    assert!(!export.body.to_string().contains("pin_hash"), "S130");
}

#[tokio::test]
async fn s55_deleting_an_account_ends_it_at_once() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    let done = call(Method::DELETE, "/v1/account")
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    assert_eq!(
        get("/v1/profiles")
            .token(token)
            .on(&server.state)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let again = post("/v1/auth/password")
        .json(json!({ "email": "ada@example.test", "password": "long-enough" }))
        .on(&server.state)
        .await;
    assert_eq!(again.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn s54_a_new_first_name_moves_the_main_profile_that_carried_the_old_one() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    let answer = call(Method::PATCH, "/v1/account")
        .token(token)
        .json(json!({ "first_name": "Grace" }))
        .on(&server.state)
        .await;
    assert_eq!(answer.body["first_name"], "Grace", "{}", answer.body);
    assert_eq!(answer.body["last_name"], "Lace");
    let list = get("/v1/profiles").token(token).on(&server.state).await;
    assert_eq!(list.body["profiles"][0]["name"], "Grace");
}

#[tokio::test]
async fn h58_recovery_sets_a_password_with_no_old_one_and_ends_every_session() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let answer = post("/v1/auth/recovery/finish")
        .json(json!({ "email": "ada@example.test", "code": MEMORY_CODE, "new_password": "new-long-password" }))
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let old = get("/v1/profiles")
        .token(&signed.access_token)
        .on(&server.state)
        .await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    let new = post("/v1/auth/password")
        .json(json!({ "email": "ada@example.test", "password": "new-long-password" }))
        .on(&server.state)
        .await;
    signed_in(new);
}

#[tokio::test]
async fn s59_changes_of_password_email_and_phone_need_a_step_up_and_check_shape() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    let phone = post("/v1/account/phone")
        .token(token)
        .json(json!({ "phone": "123" }))
        .on(&server.state)
        .await;
    assert_eq!(phone.status, StatusCode::BAD_REQUEST, "S192");
    let phone = post("/v1/account/phone")
        .token(token)
        .json(json!({ "phone": "+44 7700 900000" }))
        .on(&server.state)
        .await;
    assert_eq!(phone.status, StatusCode::OK, "{}", phone.body);
    server.clock.advance_ms(301_000);
    let email = post("/v1/account/email")
        .token(token)
        .json(json!({ "email": "new@example.test" }))
        .on(&server.state)
        .await;
    assert_eq!(email.code(), "step_up_required");
}

#[tokio::test]
async fn t11_device_sign_in_runs_through_approval_and_carries_no_step_up() {
    let server = harness_with_auth(&[("SITE_ADDRESS", "https://site.host.example")]);
    let signed = new_account(&server, "ada@example.test").await;
    let started = post("/v1/auth/device/code")
        .json(json!({ "device_name": "Living room" }))
        .on(&server.state)
        .await;
    assert_eq!(started.status, StatusCode::OK, "{}", started.body);
    let code: DeviceCode = serde_json::from_value(started.body).unwrap();
    assert!(code.verification_uri_complete.contains(&code.user_code));
    assert_eq!((code.expires_in, code.interval), (900, 5));
    let poll = || post("/v1/auth/device/token").json(json!({ "device_code": code.device_code }));
    let first: DeviceTokenAnswer =
        serde_json::from_value(poll().on(&server.state).await.body).unwrap();
    assert_eq!(format!("{:?}", first.poll), "Some(AuthorizationPending)");
    let second: DeviceTokenAnswer =
        serde_json::from_value(poll().on(&server.state).await.body).unwrap();
    assert_eq!(format!("{:?}", second.poll), "Some(SlowDown)");
    let approve = post("/v1/auth/device/approve")
        .token(&signed.access_token)
        .json(json!({ "user_code": code.user_code.to_lowercase() }))
        .on(&server.state)
        .await;
    assert_eq!(approve.status, StatusCode::OK, "{}", approve.body);
    server.clock.advance_ms(5000);
    let third: DeviceTokenAnswer =
        serde_json::from_value(poll().on(&server.state).await.body).unwrap();
    let tv = third.signed_in.expect("tokens");
    let export = get("/v1/account/export")
        .token(&tv.access_token)
        .on(&server.state)
        .await;
    assert_eq!(
        export.code(),
        "step_up_required",
        "a device session has no step up power"
    );
    let used: DeviceTokenAnswer =
        serde_json::from_value(poll().on(&server.state).await.body).unwrap();
    assert_eq!(
        format!("{:?}", used.poll),
        "Some(ExpiredToken)",
        "a device code works once"
    );
}

#[tokio::test]
async fn t11_a_denied_or_lapsed_device_code_ends() {
    let server = harness_with_auth(&[("SITE_ADDRESS", "https://site.host.example")]);
    let signed = new_account(&server, "ada@example.test").await;
    let start = || post("/v1/auth/device/code").json(json!({ "device_name": "tv" }));
    let denied: DeviceCode = serde_json::from_value(start().on(&server.state).await.body).unwrap();
    post("/v1/auth/device/deny")
        .token(&signed.access_token)
        .json(json!({ "user_code": denied.user_code }))
        .on(&server.state)
        .await;
    let answer = post("/v1/auth/device/token")
        .json(json!({ "device_code": denied.device_code }))
        .on(&server.state)
        .await;
    assert_eq!(answer.body["poll"], "access_denied");
    let lapsed: DeviceCode = serde_json::from_value(start().on(&server.state).await.body).unwrap();
    server.clock.advance_ms(901_000);
    let answer = post("/v1/auth/device/token")
        .json(json!({ "device_code": lapsed.device_code }))
        .on(&server.state)
        .await;
    assert_eq!(answer.body["poll"], "expired_token");
}

#[tokio::test]
async fn s213_device_sign_in_is_off_with_no_site_address_and_says_so() {
    let server = harness_with_auth(&[]);
    let answer = post("/v1/auth/device/code")
        .json(json!({ "device_name": "tv" }))
        .on(&server.state)
        .await;
    assert_eq!(answer.code(), "unsupported");
}

#[tokio::test]
async fn s162_an_odd_token_counts_as_no_token() {
    let server = harness_with_auth(&[]);
    for header in [
        "Bearer short",
        "Basic abcdefghijklmnopqrstuvwxyz",
        "Bearer two words here and more",
    ] {
        let answer = get("/v1/profiles")
            .header("authorization", header)
            .on(&server.state)
            .await;
        assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{header}");
    }
}

#[tokio::test]
async fn t10_an_auth_outage_answers_503_and_keeps_tokens() {
    let auth = Arc::new(MemoryAuth::default());
    let server = common::harness_on(&[], auth.clone());
    new_account(&server, "ada@example.test").await;
    auth.down.store(true, std::sync::atomic::Ordering::SeqCst);
    let answer = post("/v1/auth/password")
        .json(json!({ "email": "ada@example.test", "password": "long-enough" }))
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(answer.headers.contains_key("retry-after"));
    assert!(auth.find_email("x").await.is_err());
}

#[tokio::test]
async fn h51_h87_h22_an_edit_on_a_stale_device_changes_one_profile_and_deletes_none() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    let made = post("/v1/profiles")
        .token(token)
        .json(json!({ "name": "New One" }))
        .on(&server.state)
        .await;
    assert_eq!(made.status, StatusCode::OK, "{}", made.body);
    let list = get("/v1/profiles").token(token).on(&server.state).await;
    let first = list.body["profiles"][0]["id"].as_str().unwrap();
    let edit = call(Method::PATCH, &format!("/v1/profiles/{first}"))
        .token(token)
        .json(json!({ "name": "Renamed" }))
        .on(&server.state)
        .await;
    assert_eq!(edit.status, StatusCode::OK, "{}", edit.body);
    let after = get("/v1/profiles").token(token).on(&server.state).await;
    assert_eq!(
        after.body["profiles"].as_array().unwrap().len(),
        3,
        "no route deletes by a list"
    );
}

#[tokio::test]
async fn h59_s40_the_main_profile_moves_on_the_account_with_a_step_up() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    let made = post("/v1/profiles")
        .token(token)
        .json(json!({ "name": "Grace" }))
        .on(&server.state)
        .await;
    let id = made.body["id"].as_str().unwrap().to_owned();
    let moved = post(&format!("/v1/profiles/{id}/main"))
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(moved.status, StatusCode::OK, "{}", moved.body);
    let list = get("/v1/profiles").token(token).on(&server.state).await;
    let main: Vec<&serde_json::Value> = list.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["main"] == true)
        .collect();
    assert_eq!(main.len(), 1);
    assert_eq!(main[0]["name"], "Grace");
    server.clock.advance_ms(301_000);
    let kids = list.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kids"] == true)
        .unwrap();
    let late = post(&format!(
        "/v1/profiles/{}/main",
        kids["id"].as_str().unwrap()
    ))
    .token(token)
    .on(&server.state)
    .await;
    assert_eq!(late.code(), "step_up_required");
}

#[tokio::test]
async fn t12_p8_a_transfer_needs_a_step_up_and_leaves_the_pin_behind() {
    let server = harness_with_auth(&[]);
    let first = new_account(&server, "ada@example.test").await;
    let second = new_account(&server, "grace@example.test").await;
    let token = first.access_token.as_str();
    let made = post("/v1/profiles")
        .token(token)
        .json(json!({ "name": "Moving" }))
        .on(&server.state)
        .await;
    let id = made.body["id"].as_str().unwrap().to_owned();
    let pin = call(Method::PUT, &format!("/v1/profiles/{id}/pin"))
        .token(token)
        .json(json!({ "pin": "1234" }))
        .on(&server.state)
        .await;
    assert_eq!(pin.status, StatusCode::OK, "{}", pin.body);
    let moved = post(&format!("/v1/profiles/{id}/transfer"))
        .token(token)
        .json(json!({ "to_account_email": "grace@example.test" }))
        .on(&server.state)
        .await;
    assert_eq!(moved.status, StatusCode::OK, "{}", moved.body);
    let theirs = get("/v1/profiles")
        .token(&second.access_token)
        .on(&server.state)
        .await;
    let found = theirs.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id.as_str())
        .unwrap();
    assert_eq!(found["has_pin"], false, "the PIN never travels");
}
