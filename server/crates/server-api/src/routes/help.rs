//! Help. Open routes, so a person who cannot sign in can still read how to.

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;
use server_help::{Help, Named};

use crate::answer::{Keep, send};
use crate::error::{ApiError, ApiResult};
use crate::state::{AppState, Shared};

/// Seconds a client may keep a help page.
const HELP_TTL_S: u32 = 300;

/// The query of help routes.
#[derive(Debug, Deserialize)]
pub struct HelpQuery {
    lang: Option<String>,
    #[serde(default)]
    q: String,
}

fn library(state: &AppState) -> ApiResult<&Help> {
    state.help.as_ref().ok_or_else(|| ApiError::off("help"))
}

fn named(state: &AppState) -> Named<'_> {
    Named {
        brand: &state.config.brand_name,
        support_email: &state.config.support_email,
    }
}

/// `GET /v1/help`.
pub async fn home(
    State(state): State<Shared>,
    Query(query): Query<HelpQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let page = library(&state)?
        .home(query.lang.as_deref(), named(&state))
        .ok_or_else(ApiError::not_found)?;
    send(&headers, &page, Keep::Public(HELP_TTL_S))
}

/// `GET /v1/help/articles/{slug}`.
pub async fn article(
    State(state): State<Shared>,
    Path(slug): Path<String>,
    Query(query): Query<HelpQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let page = library(&state)?
        .article(&slug, query.lang.as_deref(), named(&state))
        .ok_or_else(ApiError::not_found)?;
    send(&headers, &page, Keep::Public(HELP_TTL_S))
}

/// `GET /v1/help/policies/{id}`.
pub async fn policy(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Query(query): Query<HelpQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let page = library(&state)?
        .policy(&id, query.lang.as_deref(), named(&state))
        .ok_or_else(ApiError::not_found)?;
    send(&headers, &page, Keep::Public(HELP_TTL_S))
}

/// `GET /v1/help/search`.
pub async fn search(
    State(state): State<Shared>,
    Query(query): Query<HelpQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let page = library(&state)?
        .search(&query.q, query.lang.as_deref(), named(&state))
        .ok_or_else(ApiError::not_found)?;
    send(&headers, &page, Keep::Never)
}
