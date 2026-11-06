//! A finder that serves open films only, over the published finder contract.
//!
//! The operator lists each film with its licence in a file. Nothing is fetched from anywhere:
//! a title in the list is found, any other title is a real not found.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use contract::TitleKey;
use contract::finder::{FindAnswer, FindOutcome, FindRequest, FinderHealth, FoundSource};
use contract::hello::ServiceState;
use contract::play::AttemptKind;
use serde::Deserialize;

/// One film in the operator's list. A film with no licence cannot be listed.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenFilm {
    pub title: TitleKey,
    pub url: String,
    pub kind: AttemptKind,
    pub licence: String,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub audio: Vec<String>,
}

/// Why a film list was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum ListError {
    Unreadable(String),
    NoLicence(String),
    BadUrl(String),
    EmbedRefused(String),
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable(why) => write!(f, "the film list cannot be read: {why}"),
            Self::NoLicence(title) => write!(f, "{title} has no licence"),
            Self::BadUrl(title) => write!(f, "{title} needs an http or https address"),
            Self::EmbedRefused(title) => write!(f, "{title} must be a stream or a file"),
        }
    }
}

impl std::error::Error for ListError {}

/// Read and check a film list.
///
/// # Errors
/// Any entry with no licence, an address that is not http or https, or an embed.
pub fn parse_films(text: &str) -> Result<Vec<OpenFilm>, ListError> {
    let films: Vec<OpenFilm> =
        serde_json::from_str(text).map_err(|e| ListError::Unreadable(e.to_string()))?;
    for film in &films {
        let name = film.title.to_string();
        if film.licence.trim().is_empty() {
            return Err(ListError::NoLicence(name));
        }
        if !(film.url.starts_with("https://") || film.url.starts_with("http://")) {
            return Err(ListError::BadUrl(name));
        }
        if !matches!(film.kind, AttemptKind::Hls | AttemptKind::File) {
            return Err(ListError::EmbedRefused(name));
        }
    }
    Ok(films)
}

/// Settings read at start. A missing one turns its feature off and says so.
#[derive(Clone, Debug, Default)]
pub struct Settings {
    pub build: String,
    /// Accepted service tokens. Two during a rotation, none turns finding off.
    pub tokens: Vec<String>,
    pub films: Vec<OpenFilm>,
    /// What is off and why, reported in the log at start.
    pub off: Vec<String>,
}

fn same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |seen, (x, y)| seen | (x ^ y)) == 0
}

fn allowed(tokens: &[String], headers: &HeaderMap) -> bool {
    let Some(given) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    else {
        return false;
    };
    // Every token is compared, so the time taken does not show which one matched.
    tokens
        .iter()
        .fold(false, |hit, t| same(t.as_bytes(), given.as_bytes()) | hit)
}

fn answer_for(films: &[OpenFilm], request: &FindRequest) -> FindAnswer {
    // A film has no episode, so a request for an episode finds nothing.
    let sources: Vec<FoundSource> = if request.episode.is_some() {
        Vec::new()
    } else {
        films
            .iter()
            .filter(|film| film.title == request.title)
            .map(|film| FoundSource {
                url: film.url.clone(),
                kind: film.kind,
                group: "demo".to_owned(),
                headers: BTreeMap::new(),
                verified: true,
                container: None,
                height: film.height,
                audio: film.audio.clone(),
            })
            .collect()
    };
    let outcome = if sources.is_empty() {
        FindOutcome::NotFound
    } else {
        FindOutcome::Found
    };
    FindAnswer {
        outcome,
        sources,
        job: None,
    }
}

async fn find(
    State(settings): State<Arc<Settings>>,
    headers: HeaderMap,
    Json(request): Json<FindRequest>,
) -> Response {
    if settings.tokens.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "finding is off: no service token is set",
        )
            .into_response();
    }
    if !allowed(&settings.tokens, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(answer_for(&settings.films, &request)).into_response()
}

async fn health(State(settings): State<Arc<Settings>>) -> Json<FinderHealth> {
    let state = if settings.tokens.is_empty() || settings.films.is_empty() {
        ServiceState::Degraded
    } else {
        ServiceState::Ready
    };
    Json(FinderHealth {
        build: settings.build.clone(),
        state,
    })
}

/// The demo answers in one step, so it holds no jobs and `/v1/find/{job}` is absent.
pub fn router(settings: Settings) -> Router {
    Router::new()
        .route("/v1/find", post(find))
        .route("/v1/health", get(health))
        .with_state(Arc::new(settings))
}
