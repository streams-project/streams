//! A finder that serves open films only, over the published finder contract.
//!
//! The operator lists each film with its licence in a file. Nothing is fetched from anywhere:
//! a title in the list is found, any other title is a real not found.

use std::collections::BTreeMap;

use async_trait::async_trait;
use contract::TitleKey;
use contract::finder::{FindRequest, FoundSource};
use contract::play::AttemptKind;
use finder_service::{Failed, Provider};
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

/// The operator's film list as a provider.
#[derive(Clone, Debug)]
pub struct OpenFilms(pub Vec<OpenFilm>);

#[async_trait]
impl Provider for OpenFilms {
    fn name(&self) -> &str {
        "open-films"
    }

    async fn find(&self, request: &FindRequest) -> Result<Vec<FoundSource>, Failed> {
        // A film has no episode, so a request for an episode finds nothing.
        if request.episode.is_some() {
            return Ok(Vec::new());
        }
        let listed = self.0.iter().filter(|film| film.title == request.title);
        Ok(listed
            .map(|film| FoundSource {
                url: film.url.clone(),
                kind: film.kind,
                group: "open-films".to_owned(),
                headers: BTreeMap::new(),
                verified: true,
                container: None,
                height: film.height,
                audio: film.audio.clone(),
            })
            .collect())
    }
}
