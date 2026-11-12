//! What a route answers when the database has stopped answering.
//!
//! Measured on 2026-11-11 with Postgres down: `POST /v1/auth/password` answered
//! `deadline_exceeded` after 15.003 s and `POST /v1/auth/signup` after 15.017 s, both the
//! server's whole request deadline. The answer this server already had for a part that is not
//! there - `Unavailable`, which the error layer turns into "busy, try again in 5 s" - was never
//! reached, because the call did not fail, it waited. These tests hold the shorter answer in
//! place: a call on the database that does not come back ends at the store deadline.

mod common;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use common::{call, harness_on};
use server_accounts::{AccountId, AuthError, AuthService, Identity};
use server_api::deadline::STORE_DEADLINE_MS;
use server_api::state::REQUEST_DEADLINE_MS;

/// A database that is up but never answers: every call parks for ever, which is what a pool
/// waiting on a host that has gone away does, and is the condition the measurement was taken in.
/// Returning an error instead would prove nothing, because that path already worked.
struct NeverAnswers;

#[async_trait]
impl AuthService for NeverAnswers {
    async fn sign_up(&self, _email: &str, _password: &str) -> Result<Identity, AuthError> {
        std::future::pending().await
    }
    async fn check_password(&self, _email: &str, _password: &str) -> Result<Identity, AuthError> {
        std::future::pending().await
    }
    async fn send_code(&self, _email: &str) -> Result<(), AuthError> {
        std::future::pending().await
    }
    async fn check_code(&self, _email: &str, _code: &str) -> Result<Identity, AuthError> {
        std::future::pending().await
    }
    async fn send_recovery(&self, _email: &str) -> Result<(), AuthError> {
        std::future::pending().await
    }
    async fn send_taken_notice(&self, _email: &str) -> Result<(), AuthError> {
        std::future::pending().await
    }
    async fn take_recovery(&self, _email: &str, _code: &str) -> Result<Identity, AuthError> {
        std::future::pending().await
    }
    async fn identity(&self, _account: AccountId) -> Result<Identity, AuthError> {
        std::future::pending().await
    }
    async fn set_password(&self, _account: AccountId, _password: &str) -> Result<(), AuthError> {
        std::future::pending().await
    }
    async fn set_email(&self, _account: AccountId, _email: &str) -> Result<(), AuthError> {
        std::future::pending().await
    }
    async fn set_phone(&self, _account: AccountId, _phone: &str) -> Result<(), AuthError> {
        std::future::pending().await
    }
    async fn delete(&self, _account: AccountId) -> Result<(), AuthError> {
        std::future::pending().await
    }
    async fn find_email(&self, _email: &str) -> Result<Option<AccountId>, AuthError> {
        std::future::pending().await
    }
}

/// Time runs on its own here, so the waits below cost no real seconds and the numbers asserted
/// are the deadlines themselves rather than a machine's mood.
#[tokio::test(start_paused = true)]
async fn h66_a_database_that_never_answers_ends_the_call_not_the_request() {
    let server = harness_on(&[], Arc::new(NeverAnswers));
    let store_deadline = Duration::from_millis(STORE_DEADLINE_MS);
    for (path, body) in [
        (
            "/v1/auth/password",
            serde_json::json!({ "email": "ada@example.test", "password": "long-enough" }),
        ),
        (
            "/v1/auth/signup",
            serde_json::json!({
                "email": "ada@example.test",
                "password": "long-enough",
                "first_name": "Ada",
                "last_name": "Lace",
            }),
        ),
    ] {
        let began = tokio::time::Instant::now();
        let answer = call(Method::POST, path).json(body).on(&server.state).await;
        let took = began.elapsed();
        assert_eq!(
            answer.status,
            StatusCode::SERVICE_UNAVAILABLE,
            "{path}: {}",
            answer.body
        );
        assert_eq!(
            answer.code(),
            "overloaded",
            "{path} answered {} and not the busy code a store outage already had",
            answer.body
        );
        assert_eq!(
            answer.body["label"], "error.try_again",
            "{path}: {}",
            answer.body
        );
        assert_eq!(
            answer.body["retry_after_s"], 5,
            "{path} must tell the client how long to wait"
        );
        assert_eq!(
            took, store_deadline,
            "{path} answered after {took:?}, not at the store deadline"
        );
        assert!(
            took.as_millis() < u128::from(REQUEST_DEADLINE_MS),
            "{path} spent the whole request deadline waiting, which is the defect itself"
        );
    }
}
