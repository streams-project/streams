//! How a good answer leaves: cache headers per route kind, and an ETag where a client may hold it.

use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use server_rules::answer::Cleared;
use sha2::{Digest, Sha256};

use crate::error::{ApiError, ApiResult};

/// Who may keep an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    /// Anyone, for this many seconds.
    Public(u32),
    /// The client only, and it asks again with its ETag.
    Private,
    /// Nobody: tokens and account answers.
    Never,
}

fn write<T: Serialize>(request: &HeaderMap, value: &T, keep: Keep) -> ApiResult<Response> {
    let body = serde_json::to_vec(value).map_err(|_| ApiError::bad("unwritable"))?;
    let policy = match keep {
        Keep::Public(seconds) => format!("public, max-age={seconds}"),
        Keep::Private => String::from("private, no-cache"),
        Keep::Never => String::from("no-store"),
    };
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Ok(value) = HeaderValue::from_str(&policy) {
        headers.insert(CACHE_CONTROL, value);
    }
    if keep == Keep::Never {
        return Ok((StatusCode::OK, headers, body).into_response());
    }
    let tag: String = Sha256::digest(&body)
        .iter()
        .take(12)
        .map(|b| format!("{b:02x}"))
        .collect();
    let tag = format!("\"{tag}\"");
    if let Ok(value) = HeaderValue::from_str(&tag) {
        headers.insert(ETAG, value);
    }
    let held = request
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|one| one.trim() == tag));
    if held {
        return Ok((StatusCode::NOT_MODIFIED, headers).into_response());
    }
    Ok((StatusCode::OK, headers, body).into_response())
}

/// Send a value that holds no title.
///
/// # Errors
/// The value could not be written.
pub fn send<T: Serialize>(request: &HeaderMap, value: &T, keep: Keep) -> ApiResult<Response> {
    write(request, value, keep)
}

/// Send a value that passed the rules. Every answer holding a title or an image comes through
/// here, so a route cannot hand out a value the rules did not build.
///
/// # Errors
/// The value could not be written.
pub fn send_cleared<T: Cleared>(request: &HeaderMap, value: &T, keep: Keep) -> ApiResult<Response> {
    write(request, value, keep)
}

/// A JSON request body. A body that does not read is a bad request in the one error shape.
#[derive(Clone, Copy, Debug)]
pub struct Body<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for Body<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(request: axum::extract::Request, state: &S) -> ApiResult<Self> {
        let axum::Json(value) = axum::Json::<T>::from_request(request, state)
            .await
            .map_err(|_| ApiError::bad("body"))?;
        Ok(Self(value))
    }
}
