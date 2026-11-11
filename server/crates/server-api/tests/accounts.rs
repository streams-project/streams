//! Sign in, step up, device sign in and account routes, with an auth service in memory.

mod common;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use common::{Answer, call, get, harness_with_auth};
use contract::account::{DeviceCode, DeviceTokenAnswer, Done, SignedIn};
use contract::api::ROUTES;
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

const PASSWORD: &str = "long-enough";

async fn join(server: &common::Harness, email: &str) -> Answer {
    call(Method::POST, "/v1/auth/signup")
        .json(sign_up_body(email))
        .on(&server.state)
        .await
}

/// A new account and the session of its first sign in. Signing up answers the same for an address
/// that has an account and one that has none, so it hands out no tokens and the device signs in
/// with the password it just chose.
async fn new_account(server: &common::Harness, email: &str) -> SignedIn {
    let joined = join(server, email).await;
    assert_eq!(joined.status, StatusCode::OK, "{}", joined.body);
    let answer = post("/v1/auth/password")
        .json(json!({ "email": email, "password": PASSWORD }))
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
async fn tmt13_an_unknown_address_and_a_wrong_password_answer_alike() {
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
async fn tmt13_a_code_and_a_recovery_answer_the_same_for_any_address() {
    // The auth service in memory now answers in the shape the built in one does: it looks the
    // address up, writes only to one that has an account, and never lets the provider's answer
    // reach the caller. With a provider that refuses, and with no mail at all, the two addresses
    // must still be told the same thing; before this the first case answered 503 for an address
    // that had an account and 200 for one that had none.
    let auth = Arc::new(MemoryAuth::default());
    let server = common::harness_on(&[], auth.clone());
    new_account(&server, "ada@example.test").await;
    let order = std::sync::atomic::Ordering::SeqCst;
    for (provider, want) in [
        ("a provider that works", StatusCode::OK),
        ("a provider that refuses", StatusCode::OK),
        ("no mail at all", StatusCode::SERVICE_UNAVAILABLE),
    ] {
        auth.mail_down
            .store(provider == "a provider that refuses", order);
        auth.mail_off.store(provider == "no mail at all", order);
        // Each round asks four times, and sign in tries are counted by the minute.
        server.clock.advance_ms(60_000);
        for path in ["/v1/auth/code", "/v1/auth/recovery/start"] {
            let known = post(path)
                .json(json!({ "email": "ada@example.test" }))
                .on(&server.state)
                .await;
            let unknown = post(path)
                .json(json!({ "email": "nobody@example.test" }))
                .on(&server.state)
                .await;
            assert_eq!(known.status, want, "{provider} {path}: {}", known.body);
            assert_eq!(
                (known.status, known.body),
                (unknown.status, unknown.body),
                "{provider} {path}"
            );
        }
    }
}

#[tokio::test]
async fn tmt13_a_sign_up_answers_the_same_for_a_taken_and_a_free_address() {
    // Before this a taken address answered 403 error.sign_up and a free one 200 with tokens, which
    // told a stranger who has an account here. Now both are told the same, the person who holds
    // the taken address hears that someone tried, and no answer carries tokens.
    let auth = Arc::new(MemoryAuth::default());
    let server = common::harness_on(&[], auth.clone());
    new_account(&server, "ada@example.test").await;
    // The sign up that made the account already wrote to this address inside this window.
    server.clock.advance_ms(60_000);
    let taken = join(&server, "ada@example.test").await;
    let free = join(&server, "grace@example.test").await;
    assert_eq!(taken.status, StatusCode::OK, "{}", taken.body);
    assert_eq!((taken.status, &taken.body), (free.status, &free.body));
    assert!(
        !taken.body.to_string().contains("token"),
        "no tokens leave a sign up: {}",
        taken.body
    );
    let warned = auth.warned.lock().unwrap().clone();
    assert_eq!(
        warned,
        ["ada@example.test"],
        "the person who holds the address hears about the attempt, and nobody else"
    );
    let sent = auth.sent.lock().unwrap().clone();
    assert!(
        sent.contains(&String::from("grace@example.test")),
        "the new account gets its code, so both branches cost one letter: {sent:?}"
    );

    // A second ask in the same window is answered the same and sends nothing, so this is no way
    // to post letters at an address.
    let again = join(&server, "ada@example.test").await;
    assert_eq!((again.status, again.body), (taken.status, taken.body));
    assert_eq!(
        auth.warned.lock().unwrap().len(),
        1,
        "one letter per address per window"
    );
}

#[tokio::test]
async fn s59_an_address_change_does_not_say_who_holds_an_address() {
    let auth = Arc::new(MemoryAuth::default());
    let server = common::harness_on(&[], auth.clone());
    let ada = new_account(&server, "ada@example.test").await;
    new_account(&server, "grace@example.test").await;
    server.clock.advance_ms(60_000);
    let taken = post("/v1/account/email")
        .token(&ada.access_token)
        .json(json!({ "email": "grace@example.test" }))
        .on(&server.state)
        .await;
    let free = post("/v1/account/email")
        .token(&ada.access_token)
        .json(json!({ "email": "ada2@example.test" }))
        .on(&server.state)
        .await;
    assert_eq!(taken.status, StatusCode::OK, "{}", taken.body);
    assert_eq!((taken.status, taken.body), (free.status, free.body));
    assert_eq!(
        auth.warned.lock().unwrap().clone(),
        ["grace@example.test"],
        "the holder of the address hears that someone tried to move an account onto it"
    );
}

#[tokio::test]
async fn tmt2_a_password_change_signs_the_other_devices_out() {
    // Before this a password change left every other session alive, so the one thing a person does
    // after a token is stolen did not reach the thief, whose refresh chain carried on.
    let server = harness_with_auth(&[]);
    let one = new_account(&server, "ada@example.test").await;
    let two = signed_in(
        post("/v1/auth/password")
            .json(json!({ "email": "ada@example.test", "password": PASSWORD }))
            .on(&server.state)
            .await,
    );
    let changed = post("/v1/account/password")
        .token(&one.access_token)
        .json(json!({ "new_password": "a-new-long-password" }))
        .on(&server.state)
        .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.body);
    let theirs = get("/v1/profiles")
        .token(&two.access_token)
        .on(&server.state)
        .await;
    assert_eq!(
        theirs.status,
        StatusCode::UNAUTHORIZED,
        "the other device is signed out"
    );
    let refreshed = post("/v1/auth/refresh")
        .json(json!({ "refresh_token": two.refresh_token }))
        .on(&server.state)
        .await;
    assert_eq!(
        refreshed.status,
        StatusCode::UNAUTHORIZED,
        "and its refresh chain is gone with it"
    );
    let mine = get("/v1/profiles")
        .token(&one.access_token)
        .on(&server.state)
        .await;
    assert_eq!(
        mine.status,
        StatusCode::OK,
        "the device the person is standing at stays: {}",
        mine.body
    );
}

#[tokio::test]
async fn n16_a_sign_up_answers_the_shape_the_contract_names() {
    // The route stopped handing tokens out, and the contract went on declaring SignedIn for its
    // 200, so the gate that regenerates the contract would have failed and every generated client
    // still believed a shape the server no longer sends.
    let route = ROUTES
        .iter()
        .find(|route| route.method == "POST" && route.path == "/v1/auth/signup")
        .expect("the sign up route");
    assert_eq!(route.response, "Done", "the contract names the answer");
    let server = harness_with_auth(&[]);
    let joined = join(&server, "ada@example.test").await;
    assert_eq!(joined.status, StatusCode::OK, "{}", joined.body);
    let done: Done = serde_json::from_value(joined.body.clone()).expect("a Done");
    assert!(done.done);
    assert!(
        serde_json::from_value::<SignedIn>(joined.body).is_err(),
        "and the shape it used to answer in is gone"
    );
}

#[tokio::test]
async fn tmt2_a_refused_password_change_leaves_every_session_alive() {
    // The sessions went first and the password after, so a password the auth service refuses -
    // one past the 72 bytes bcrypt reads, which the door's minimum length check lets through -
    // signed every other device out for a change that never happened. Before: 400 bad_request and
    // the other device signed out. After: 400 bad_request and the other device still signed in,
    // with the old password still working.
    let server = harness_with_auth(&[]);
    let one = new_account(&server, "ada@example.test").await;
    let two = signed_in(
        post("/v1/auth/password")
            .json(json!({ "email": "ada@example.test", "password": PASSWORD }))
            .on(&server.state)
            .await,
    );
    let refused = post("/v1/account/password")
        .token(&one.access_token)
        .json(json!({ "new_password": "x".repeat(73) }))
        .on(&server.state)
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{}", refused.body);
    assert_eq!(refused.code(), "bad_request", "{}", refused.body);
    for (whose, token) in [("the other device", &two), ("this one", &one)] {
        let theirs = get("/v1/profiles")
            .token(&token.access_token)
            .on(&server.state)
            .await;
        assert_eq!(
            theirs.status,
            StatusCode::OK,
            "{whose} is still signed in: {}",
            theirs.body
        );
    }
    let again = post("/v1/auth/password")
        .json(json!({ "email": "ada@example.test", "password": PASSWORD }))
        .on(&server.state)
        .await;
    assert_eq!(
        again.status,
        StatusCode::OK,
        "the password that was refused did not land: {}",
        again.body
    );
}

#[tokio::test]
async fn s42_a_television_signs_itself_out_and_no_other_device() {
    // A step up on every device removal left a television unable to sign itself out: a session
    // opened from a device code has no step up power until it proves a password or code on that
    // device, and the shipped TV settings screens call DELETE /v1/devices. Signing this device
    // out needs no proof, because POST /v1/auth/signout already ends this very session without
    // one; another device still needs it.
    let server = harness_with_auth(&[("SITE_ADDRESS", "https://site.host.example")]);
    let phone = new_account(&server, "ada@example.test").await;
    let started = post("/v1/auth/device/code")
        .json(json!({ "device_name": "Living room" }))
        .on(&server.state)
        .await;
    let code: DeviceCode = serde_json::from_value(started.body).unwrap();
    post("/v1/auth/device/approve")
        .token(&phone.access_token)
        .json(json!({ "user_code": code.user_code }))
        .on(&server.state)
        .await;
    server.clock.advance_ms(5000);
    let answer: DeviceTokenAnswer = serde_json::from_value(
        post("/v1/auth/device/token")
            .json(json!({ "device_code": code.device_code }))
            .on(&server.state)
            .await
            .body,
    )
    .unwrap();
    let tv = answer.signed_in.expect("tokens");
    let devices = get("/v1/devices")
        .token(&tv.access_token)
        .on(&server.state)
        .await;
    let listed = devices.body["devices"].as_array().unwrap().clone();
    let id = |current: bool| {
        listed
            .iter()
            .find(|device| device["current"] == current)
            .expect("a device")["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let others = call(Method::DELETE, "/v1/devices")
        .token(&tv.access_token)
        .on(&server.state)
        .await;
    assert_eq!(
        others.code(),
        "step_up_required",
        "a television may not throw the other devices off on its own: {}",
        others.body
    );
    let theirs = call(Method::DELETE, &format!("/v1/devices?id={}", id(false)))
        .token(&tv.access_token)
        .on(&server.state)
        .await;
    assert_eq!(
        theirs.code(),
        "step_up_required",
        "nor another named device: {}",
        theirs.body
    );
    let itself = call(Method::DELETE, &format!("/v1/devices?id={}", id(true)))
        .token(&tv.access_token)
        .on(&server.state)
        .await;
    assert_eq!(
        itself.status,
        StatusCode::OK,
        "but it signs itself out: {}",
        itself.body
    );
    let after = get("/v1/devices")
        .token(&tv.access_token)
        .on(&server.state)
        .await;
    assert_eq!(
        after.status,
        StatusCode::UNAUTHORIZED,
        "and its own token is gone with the session"
    );
    let mine = get("/v1/profiles")
        .token(&phone.access_token)
        .on(&server.state)
        .await;
    assert_eq!(mine.status, StatusCode::OK, "the phone is untouched");
}

#[tokio::test]
async fn tmt12_signing_a_device_out_needs_a_step_up() {
    // The account flows say a device removal needs a recent password or code; the code did not ask
    // for one, so a stolen access token could throw every other device off.
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    signed_in(
        post("/v1/auth/password")
            .json(json!({ "email": "ada@example.test", "password": PASSWORD }))
            .on(&server.state)
            .await,
    );
    let devices = get("/v1/devices").token(token).on(&server.state).await;
    let other = devices.body["devices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|device| device["current"] == false)
        .expect("the other device")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let which = format!("/v1/devices?id={other}");
    server.clock.advance_ms(301_000);
    let refused = call(Method::DELETE, &which)
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(refused.code(), "step_up_required", "{}", refused.body);
    let refused = call(Method::DELETE, "/v1/devices")
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(refused.code(), "step_up_required", "{}", refused.body);
    let proved = post("/v1/auth/confirm")
        .token(token)
        .json(json!({ "password": PASSWORD }))
        .on(&server.state)
        .await;
    assert_eq!(proved.status, StatusCode::OK, "{}", proved.body);
    let done = call(Method::DELETE, "/v1/devices")
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    let left = get("/v1/devices").token(token).on(&server.state).await;
    assert_eq!(
        left.body["devices"].as_array().unwrap().len(),
        1,
        "only this device is left: {}",
        left.body
    );
}

#[tokio::test]
async fn s203_the_device_key_a_device_sends_is_kept() {
    // The route checked the length of the key and then dropped it, so nothing named the device
    // across its sessions.
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "ada@example.test").await;
    let key = "0123456789abcdef0123";
    let saved = post("/v1/devices/me")
        .token(&signed.access_token)
        .json(json!({ "device_key": key, "card_hash": "h1" }))
        .on(&server.state)
        .await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    let member = server
        .door
        .member(&signed.access_token)
        .await
        .expect("the caller");
    assert_eq!(
        server.store.device_session(member.account(), key),
        Some(member.session_id()),
        "the key is kept against the account and the session that sent it"
    );
    let short = post("/v1/devices/me")
        .token(&signed.access_token)
        .json(json!({ "device_key": "short", "card_hash": "h1" }))
        .on(&server.state)
        .await;
    assert_eq!(short.status, StatusCode::BAD_REQUEST, "{}", short.body);
}

#[tokio::test]
async fn s39_a_picture_the_brand_did_not_publish_is_refused() {
    let site = common::folder("avatar-rules");
    std::fs::create_dir_all(site.join("avatars")).unwrap();
    let sets = json!({ "sets": [
        { "name": "Soft", "kids": true, "pictures": [{ "name": "One", "path": "/avatars/one.webp" }] },
        { "name": "Dark", "kids": false, "pictures": [{ "name": "Two", "path": "/avatars/two.webp" }] },
    ]});
    std::fs::write(site.join("avatars/sets.json"), sets.to_string()).unwrap();
    let path = site.to_string_lossy().into_owned();
    let server = harness_with_auth(&[("WEB_ROOT", path.as_str())]);
    let signed = new_account(&server, "ada@example.test").await;
    let token = signed.access_token.as_str();
    let rows = get("/v1/profiles").token(token).on(&server.state).await;
    let main = rows.body["profiles"][0]["id"].as_str().unwrap().to_owned();
    let kids = rows.body["profiles"][1]["id"].as_str().unwrap().to_owned();
    let set = |id: &str, avatar: &str| {
        call(Method::PATCH, &format!("/v1/profiles/{id}"))
            .token(token)
            .json(json!({ "avatar": avatar }))
    };
    // A name the column cannot hold used to reach the store, which answered "busy, try again" for
    // ever; a name no set names reached it too.
    let long = set(&main, &"a".repeat(65)).on(&server.state).await;
    assert_eq!(long.status, StatusCode::BAD_REQUEST, "{}", long.body);
    let stranger = set(&main, "/avatars/three.webp").on(&server.state).await;
    assert_eq!(
        stranger.status,
        StatusCode::BAD_REQUEST,
        "{}",
        stranger.body
    );
    let grown_up = set(&kids, "/avatars/two.webp").on(&server.state).await;
    assert_eq!(
        grown_up.status,
        StatusCode::BAD_REQUEST,
        "a Kids profile is offered only the Kids sets: {}",
        grown_up.body
    );
    let kept = set(&kids, "/avatars/one.webp").on(&server.state).await;
    assert_eq!(kept.status, StatusCode::OK, "{}", kept.body);
    assert_eq!(kept.body["avatar"], "/avatars/one.webp");
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
async fn tmt7_password_tries_hit_429_on_the_account_and_the_address() {
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
async fn tmt10_a_refused_refresh_ends_the_session_by_name() {
    let server = harness_with_auth(&[]);
    let answer = post("/v1/auth/refresh")
        .json(json!({ "refresh_token": "made-up.token" }))
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED);
    assert_eq!(answer.code(), "session_ended");
}

#[tokio::test]
async fn s42_tmt12_export_and_delete_need_a_fresh_step_up_and_confirm_gives_it() {
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
async fn tmt11_device_sign_in_runs_through_approval_and_carries_no_step_up() {
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
async fn tmt11_a_denied_or_lapsed_device_code_ends() {
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
async fn n11_the_first_answer_to_a_device_code_stands() {
    let server = harness_with_auth(&[("SITE_ADDRESS", "https://site.host.example")]);
    let ada = new_account(&server, "ada@example.test").await;
    let eve = new_account(&server, "eve@example.test").await;
    let start = || post("/v1/auth/device/code").json(json!({ "device_name": "tv" }));
    let answer = |word: &'static str, token: &str, code: &str| {
        post(&format!("/v1/auth/device/{word}"))
            .token(token)
            .json(json!({ "user_code": code }))
    };
    let tv: DeviceCode = serde_json::from_value(start().on(&server.state).await.body).unwrap();

    // Approved: a second person takes nothing, and a change of mind counts for nothing.
    let done = answer("approve", &ada.access_token, &tv.user_code)
        .on(&server.state)
        .await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    for (word, token) in [
        ("approve", &eve.access_token),
        ("approve", &ada.access_token),
        ("deny", &ada.access_token),
    ] {
        let again = answer(word, token, &tv.user_code).on(&server.state).await;
        assert_eq!(again.status, StatusCode::FORBIDDEN, "{}", again.body);
        assert_eq!(again.body["label"], "error.device_decided");
    }
    let tokens: DeviceTokenAnswer = serde_json::from_value(
        post("/v1/auth/device/token")
            .json(json!({ "device_code": tv.device_code }))
            .on(&server.state)
            .await
            .body,
    )
    .unwrap();
    assert_eq!(tokens.signed_in.expect("tokens").user_id, ada.user_id);

    // Refused: it stays refused, and the device hears so.
    server.clock.advance_ms(60_000);
    let tv: DeviceCode = serde_json::from_value(start().on(&server.state).await.body).unwrap();
    let done = answer("deny", &ada.access_token, &tv.user_code)
        .on(&server.state)
        .await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    for word in ["approve", "deny"] {
        let again = answer(word, &ada.access_token, &tv.user_code)
            .on(&server.state)
            .await;
        assert_eq!(again.body["label"], "error.device_decided");
    }
    let poll = post("/v1/auth/device/token")
        .json(json!({ "device_code": tv.device_code }))
        .on(&server.state)
        .await;
    assert_eq!(poll.body["poll"], "access_denied");
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
async fn tmt10_an_auth_outage_answers_503_and_keeps_tokens() {
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
    // Two devices on one account: the device that signed up, and a second that signs in after it.
    let one = new_account(&server, "ada@example.test").await.access_token;
    let two = signed_in(
        post("/v1/auth/password")
            .header("client-app", "phone")
            .json(json!({ "email": "ada@example.test", "password": "long-enough" }))
            .on(&server.state)
            .await,
    )
    .access_token;
    let devices = get("/v1/devices").token(&two).on(&server.state).await;
    assert_eq!(
        devices.body["devices"].as_array().unwrap().len(),
        2,
        "this test needs two sessions to have two devices"
    );

    // The second device reads the list and keeps this copy. Every change after it goes unseen.
    let held = get("/v1/profiles").token(&two).on(&server.state).await;
    let stale = held.body["profiles"].as_array().unwrap();
    assert_eq!(stale.len(), 2, "sign up makes a main and a Kids profile");
    let id_of = |rows: &[serde_json::Value], flag: &str| -> String {
        rows.iter().find(|row| row[flag] == true).expect("profile")["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let main = id_of(stale, "main");
    let kids = id_of(stale, "kids");

    // The first device renames the main profile and adds one the second device never sees.
    let renamed = call(Method::PATCH, &format!("/v1/profiles/{main}"))
        .token(&one)
        .json(json!({ "name": "Ada Two" }))
        .on(&server.state)
        .await;
    assert_eq!(renamed.status, StatusCode::OK, "{}", renamed.body);
    let added = post("/v1/profiles")
        .token(&one)
        .json(json!({ "name": "Grace" }))
        .on(&server.state)
        .await;
    assert_eq!(added.status, StatusCode::OK, "{}", added.body);
    let grace = added.body["id"].as_str().unwrap().to_owned();

    // The stale device sends its edit: one id, one field, and nothing of what it still holds.
    let edit = call(Method::PATCH, &format!("/v1/profiles/{kids}"))
        .token(&two)
        .json(json!({ "avatar": "bear" }))
        .on(&server.state)
        .await;
    assert_eq!(edit.status, StatusCode::OK, "{}", edit.body);

    // One profile changed. The stale copy neither undid the rename nor dropped what it never saw.
    let after = get("/v1/profiles").token(&one).on(&server.state).await;
    let rows = after.body["profiles"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        3,
        "the profile the stale device never saw is still here"
    );
    let row = |id: &str| {
        rows.iter()
            .find(|row| row["id"].as_str() == Some(id))
            .expect("profile")
    };
    assert_eq!(row(&kids)["avatar"], "bear", "the field it sent, applied");
    assert_eq!(row(&kids)["name"], "Kids", "no other field of it moved");
    assert_eq!(
        row(&main)["name"],
        "Ada Two",
        "the stale device did not undo the rename"
    );
    assert_eq!(row(&grace)["name"], "Grace");

    // No route takes a list, so a stale copy can never be sent as the whole truth.
    let snapshot = call(Method::PUT, "/v1/profiles")
        .token(&two)
        .json(json!({ "profiles": stale }))
        .on(&server.state)
        .await;
    assert_eq!(
        snapshot.status,
        StatusCode::METHOD_NOT_ALLOWED,
        "a list write would let a stale device delete by leaving a profile out"
    );

    // A delete names one id. The stale device's delete of a gone profile removes nothing else.
    let gone = call(Method::DELETE, &format!("/v1/profiles/{grace}"))
        .token(&one)
        .on(&server.state)
        .await;
    assert_eq!(gone.status, StatusCode::OK, "{}", gone.body);
    let again = call(Method::DELETE, &format!("/v1/profiles/{grace}"))
        .token(&two)
        .on(&server.state)
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND, "{}", again.body);
    let last = get("/v1/profiles").token(&one).on(&server.state).await;
    let names: Vec<&str> = last.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Ada Two", "Kids"]);
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
async fn tmt12_tmp8_a_transfer_needs_a_step_up_and_leaves_the_pin_behind() {
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

#[tokio::test]
async fn tmp8_a_refused_transfer_leaves_the_pin_where_it_was() {
    // The PIN was cleared in its own committed write before the move was tried, so a transfer the
    // other account had no room for answered "refused" and took the PIN with it anyway.
    let server = harness_with_auth(&[]);
    let first = new_account(&server, "ada@example.test").await;
    let second = new_account(&server, "grace@example.test").await;
    let token = first.access_token.as_str();
    for n in 0..3 {
        let made = post("/v1/profiles")
            .token(&second.access_token)
            .json(json!({ "name": format!("Extra {n}") }))
            .on(&server.state)
            .await;
        assert_eq!(made.status, StatusCode::OK, "{}", made.body);
    }
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
    let refused = post(&format!("/v1/profiles/{id}/transfer"))
        .token(token)
        .json(json!({ "to_account_email": "grace@example.test" }))
        .on(&server.state)
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
    assert_eq!(refused.body["label"], "error.profile_limit");
    let mine = get("/v1/profiles").token(token).on(&server.state).await;
    let row = mine.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|profile| profile["id"] == id.as_str())
        .expect("the profile stayed where it was");
    assert_eq!(
        row["has_pin"], true,
        "a transfer that was refused leaves the PIN alone"
    );
}

#[tokio::test]
async fn a_profile_keeps_a_language_the_server_has_and_refuses_one_it_does_not() {
    let strings = common::folder("languages");
    std::fs::write(strings.join("en.json"), r#"{"language.name":"English"}"#).unwrap();
    std::fs::write(strings.join("de.json"), r#"{"language.name":"Deutsch"}"#).unwrap();
    std::fs::write(strings.join("xx.json"), r#"{"no.name":"none"}"#).unwrap();
    let path = strings.to_string_lossy().into_owned();
    let server = harness_with_auth(&[("STRINGS_DIR", path.as_str())]);
    let signed = new_account(&server, "lang@example.test").await;

    let list = get("/v1/languages").on(&server.state).await;
    assert_eq!(list.status, StatusCode::OK, "{}", list.body);
    let names: Vec<&str> = list.body["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|language| language["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["Deutsch", "English"],
        "a file with no name is no language"
    );

    let rows = get("/v1/profiles")
        .token(&signed.access_token)
        .on(&server.state)
        .await;
    let id = rows.body["profiles"][0]["id"].as_str().unwrap().to_owned();
    let change = |code: &str| {
        call(Method::PATCH, &format!("/v1/profiles/{id}"))
            .token(&signed.access_token)
            .json(json!({ "language": code }))
    };
    let refused = change("xx").on(&server.state).await;
    assert_ne!(refused.status, StatusCode::OK, "{}", refused.body);

    let kept = change("de").on(&server.state).await;
    assert_eq!(kept.status, StatusCode::OK, "{}", kept.body);
    assert_eq!(kept.body["language"], "de");
    let rows = get("/v1/profiles")
        .token(&signed.access_token)
        .on(&server.state)
        .await;
    assert_eq!(rows.body["profiles"][0]["language"], "de");
    assert!(
        rows.body["profiles"][1].get("language").is_none(),
        "the other profile did not change"
    );
}

#[tokio::test]
async fn a_kids_profile_is_offered_only_the_kids_pictures() {
    let site = common::folder("pictures");
    std::fs::create_dir_all(site.join("avatars")).unwrap();
    let sets = json!({ "sets": [
        { "name": "Soft", "kids": true, "pictures": [{ "name": "One", "path": "/avatars/one.webp" }] },
        { "name": "Dark", "kids": false, "pictures": [{ "name": "Two", "path": "/avatars/two.webp" }] },
    ]});
    std::fs::write(site.join("avatars/sets.json"), sets.to_string()).unwrap();
    let path = site.to_string_lossy().into_owned();
    let server = harness_with_auth(&[("WEB_ROOT", path.as_str())]);
    let signed = new_account(&server, "pictures@example.test").await;
    let rows = get("/v1/profiles")
        .token(&signed.access_token)
        .on(&server.state)
        .await;
    let pictures = |row: usize| {
        let id = rows.body["profiles"][row]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        get(&format!("/v1/profiles/{id}/avatars")).token(&signed.access_token)
    };
    let main = pictures(0).on(&server.state).await;
    assert_eq!(main.status, StatusCode::OK, "{}", main.body);
    assert_eq!(main.body["sets"].as_array().unwrap().len(), 2);
    let kids = pictures(1).on(&server.state).await;
    assert_eq!(kids.body["sets"].as_array().unwrap().len(), 1);
    assert_eq!(kids.body["sets"][0]["name"], "Soft");
}

#[tokio::test]
async fn a_brand_with_no_pictures_answers_an_empty_list() {
    let server = harness_with_auth(&[]);
    let signed = new_account(&server, "plain@example.test").await;
    let rows = get("/v1/profiles")
        .token(&signed.access_token)
        .on(&server.state)
        .await;
    let id = rows.body["profiles"][0]["id"].as_str().unwrap().to_owned();
    let answer = get(&format!("/v1/profiles/{id}/avatars"))
        .token(&signed.access_token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert!(answer.body["sets"].as_array().unwrap().is_empty());
}
