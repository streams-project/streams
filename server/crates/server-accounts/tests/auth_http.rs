//! The HTTP client against a local stand in on 127.0.0.1. No outside address is called.

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use server_accounts::{AccountId, AuthError, AuthService, HttpAuth};

const USER: &str = "7b1f6f5e-3c55-4c5e-9a52-5a0f8e3f1a11";

fn user() -> Value {
    json!({ "id": USER, "email": "ada@example.test", "phone": "" })
}

async fn token(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    if body["password"] == "right-password" {
        (
            StatusCode::OK,
            Json(json!({ "access_token": "x", "user": user() })),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error_code": "invalid_credentials" })),
        )
    }
}

async fn sign_up(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    match body["email"].as_str() {
        Some("taken@example.test") => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error_code": "email_exists" })),
        ),
        _ if body["password"] == "weak" => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error_code": "weak_password" })),
        ),
        _ => (StatusCode::OK, Json(user())),
    }
}

async fn broken() -> (StatusCode, &'static str) {
    (StatusCode::BAD_GATEWAY, "down")
}

async fn admin_user(headers: axum::http::HeaderMap) -> (StatusCode, Json<Value>) {
    let held = headers.get("authorization").and_then(|v| v.to_str().ok());
    if held == Some("Bearer admin-token") {
        (StatusCode::OK, Json(user()))
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({})))
    }
}

async fn stand_in() -> String {
    let app = Router::new()
        .route("/token", post(token))
        .route("/signup", post(sign_up))
        .route("/otp", post(broken))
        .route("/admin/users/{id}", get(admin_user));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await });
    format!("http://{address}/")
}

#[tokio::test]
async fn c69_a_good_password_reads_a_person_and_a_bad_one_is_refused() {
    let auth = HttpAuth::new(&stand_in().await, "admin-token").unwrap();
    let person = auth
        .check_password("ada@example.test", "right-password")
        .await
        .unwrap();
    assert_eq!(person.account.to_string(), USER);
    assert_eq!(person.phone, None);
    let bad = auth.check_password("ada@example.test", "wrong").await;
    assert_eq!(bad, Err(AuthError::Refused));
}

#[tokio::test]
async fn c69_sign_up_names_a_taken_address_and_a_weak_password_apart() {
    let auth = HttpAuth::new(&stand_in().await, "admin-token").unwrap();
    assert!(
        auth.sign_up("new@example.test", "long-enough")
            .await
            .is_ok()
    );
    assert_eq!(
        auth.sign_up("taken@example.test", "long-enough").await,
        Err(AuthError::Taken)
    );
    assert_eq!(
        auth.sign_up("new@example.test", "weak").await,
        Err(AuthError::WeakPassword)
    );
}

#[tokio::test]
async fn t10_a_failing_auth_service_is_an_outage_and_never_a_refusal() {
    let auth = HttpAuth::new(&stand_in().await, "admin-token").unwrap();
    assert_eq!(
        auth.send_code("ada@example.test").await,
        Err(AuthError::Unavailable)
    );
}

#[tokio::test]
async fn s176_the_admin_route_carries_the_token_and_a_read_finds_the_person() {
    let address = stand_in().await;
    let id = AccountId::parse(USER).unwrap();
    let auth = HttpAuth::new(&address, "admin-token").unwrap();
    assert_eq!(auth.identity(id).await.unwrap().email, "ada@example.test");
    let wrong = HttpAuth::new(&address, "other").unwrap();
    assert_eq!(wrong.identity(id).await, Err(AuthError::Refused));
}

#[tokio::test]
async fn t10_no_answer_at_all_is_an_outage() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let auth = HttpAuth::new(&format!("http://{address}"), "admin-token").unwrap();
    assert_eq!(
        auth.check_password("a@b.test", "x").await,
        Err(AuthError::Unavailable)
    );
}
