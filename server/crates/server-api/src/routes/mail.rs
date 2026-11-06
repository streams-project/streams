//! The auth service's send email hook, on the address only it reaches. Mail in sixteen languages
//! stays in this server. The refusal shape is the one the hook reads.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use server_accounts::mail::hook::{Hook, requests};
use server_accounts::mail::render::{Brand, render};
use server_accounts::mail::webhook::verify;
use server_accounts::mail::{Letter, MailError};
use tracing::warn;

use crate::state::Shared;

/// Minutes a code stays good, as the auth service is set.
const CODE_MINUTES: u32 = 10;

fn refuse(status: StatusCode, message: &str) -> Response {
    let body = json!({ "error": { "http_code": status.as_u16(), "message": message } });
    (status, Json(body)).into_response()
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
}

/// `POST /internal/mail`.
pub async fn hook(State(state): State<Shared>, headers: HeaderMap, body: String) -> Response {
    let (Some(secret), Some(sender), Some(site)) = (
        state.config.mail_hook_secret.as_deref(),
        state.mail.as_ref(),
        state.config.site_address.as_deref(),
    ) else {
        return refuse(StatusCode::SERVICE_UNAVAILABLE, "mail is off");
    };
    let now_s = i64::try_from(state.clock.now_ms() / 1000).unwrap_or(0);
    let signed = verify(
        secret,
        header(&headers, "webhook-id"),
        header(&headers, "webhook-timestamp"),
        header(&headers, "webhook-signature"),
        &body,
        now_s,
    );
    if signed.is_err() {
        return refuse(StatusCode::UNAUTHORIZED, "bad signature");
    }
    let Ok(parsed) = serde_json::from_str::<Hook>(&body) else {
        return refuse(StatusCode::BAD_REQUEST, "bad body");
    };
    let Ok(messages) = requests(&parsed, site, CODE_MINUTES) else {
        return refuse(StatusCode::BAD_REQUEST, "unknown action");
    };
    let brand = Brand {
        name: &state.config.brand_name,
        help_email: &state.config.support_email,
        site_url: site,
        help_url: state.config.help_address.as_deref().unwrap_or(site),
    };
    for message in &messages {
        let page = render(&brand, message);
        let letter = Letter {
            to: message.to.clone(),
            subject: page.subject,
            html: page.html,
            text: page.text,
        };
        if let Err(MailError(detail)) = sender.send(&letter).await {
            warn!("Mail not sent: {detail}");
            return refuse(StatusCode::INTERNAL_SERVER_ERROR, "send failed");
        }
    }
    (StatusCode::OK, Json(json!({}))).into_response()
}
