//! Who is calling: the bearer token, the client headers and the one trusted address.

use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use contract::client::DeviceClass;
use server_accounts::{Door, Member, ProfileScope};
use server_rules::Viewer;

use crate::error::{ApiError, ApiResult};
use crate::state::Shared;

/// Shortest and longest token the door reads. Anything else counts as no token.
const TOKEN_MIN: usize = 20;
const TOKEN_MAX: usize = 4096;

/// The token of a request: `Bearer` or `bearer`, trimmed, 20 to 4,096 bytes, no spaces.
#[must_use]
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?
        .trim();
    let fits = (TOKEN_MIN..=TOKEN_MAX).contains(&token.len());
    (fits && !token.contains(char::is_whitespace)).then_some(token)
}

/// The class a client states. Unknown words read as `Unknown`.
#[must_use]
pub fn client_class(headers: &HeaderMap) -> DeviceClass {
    headers
        .get("client-class")
        .and_then(|value| value.to_str().ok())
        .and_then(|word| {
            serde_json::from_value(serde_json::Value::from(word.to_ascii_lowercase())).ok()
        })
        .unwrap_or(DeviceClass::Unknown)
}

fn door(state: &Shared) -> ApiResult<&Door> {
    state.door.as_ref().ok_or_else(|| ApiError::off("accounts"))
}

/// A signed in caller, with or without a profile.
#[derive(Clone, Debug)]
pub struct Caller(pub Member);

impl FromRequestParts<Shared> for Caller {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &Shared) -> ApiResult<Self> {
        let door = door(state)?;
        let token = bearer(&parts.headers).ok_or(server_accounts::DoorError::Unauthorized)?;
        Ok(Self(door.member(token).await?))
    }
}

/// A caller with an active profile. A session with none is refused here, before any handler.
#[derive(Clone, Debug)]
pub struct Watching {
    pub member: Member,
    pub scope: ProfileScope,
}

impl FromRequestParts<Shared> for Watching {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &Shared) -> ApiResult<Self> {
        let Caller(member) = Caller::from_request_parts(parts, state).await?;
        let scope = door(state)?.scope(&member).await?;
        Ok(Self { member, scope })
    }
}

/// Whoever may read a title: a watching member, or a visitor where the deployer allows visitors.
#[derive(Clone, Debug)]
pub struct Reader {
    pub viewer: Viewer,
    /// None for a visitor.
    pub watching: Option<Watching>,
}

impl FromRequestParts<Shared> for Reader {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &Shared) -> ApiResult<Self> {
        let visitor = (!state.config.accounts_required)
            .then_some(state.config.visitor_tier)
            .flatten();
        if let (Some(tier), None) = (visitor, bearer(&parts.headers)) {
            return Ok(Self {
                viewer: Viewer::visitor(tier),
                watching: None,
            });
        }
        let watching = Watching::from_request_parts(parts, state).await?;
        Ok(Self {
            viewer: Viewer::of(&watching.scope)?,
            watching: Some(watching),
        })
    }
}
