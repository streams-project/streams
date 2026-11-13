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
    /// For a series: this one film stands in for each of its episodes.
    #[serde(default)]
    pub every_episode: bool,
}

/// Why a film list was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum ListError {
    Unreadable(String),
    NoLicence(String),
    BadUrl(String),
    EmbedRefused(String),
    TwiceListed(String),
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable(why) => write!(f, "the film list cannot be read: {why}"),
            Self::NoLicence(title) => write!(f, "{title} has no licence"),
            Self::BadUrl(title) => write!(f, "{title} needs an https address"),
            Self::EmbedRefused(title) => write!(f, "{title} must be a stream or a file"),
            Self::TwiceListed(title) => write!(f, "{title} lists the same address twice"),
        }
    }
}

impl std::error::Error for ListError {}

/// Read and check a film list.
///
/// A title may hold as many rows as the operator has addresses for it, and every one of them is
/// answered: that is how a film gets a second place to come from, which is the whole of what a
/// deployer can do about one address going down. The same address listed twice is not a second
/// place, so it is refused rather than passed on as if it were.
///
/// # Errors
/// Any entry with no licence, an address that is not https, an embed, or a title that lists one
/// address twice.
pub fn parse_films(text: &str) -> Result<Vec<OpenFilm>, ListError> {
    let films: Vec<OpenFilm> =
        serde_json::from_str(text).map_err(|e| ListError::Unreadable(e.to_string()))?;
    let mut seen: Vec<(&TitleKey, &str)> = Vec::new();
    for film in &films {
        let name = film.title.to_string();
        if film.licence.trim().is_empty() {
            return Err(ListError::NoLicence(name));
        }
        // The server drops a source that is not https before it reaches a device, so a list that
        // takes http passes its start-up check and then plays nothing: a refusal here is the one
        // place a deployer is told.
        if !film.url.starts_with("https://") {
            return Err(ListError::BadUrl(name));
        }
        if !matches!(film.kind, AttemptKind::Hls | AttemptKind::File) {
            return Err(ListError::EmbedRefused(name));
        }
        if seen.contains(&(&film.title, film.url.as_str())) {
            return Err(ListError::TwiceListed(name));
        }
        seen.push((&film.title, film.url.as_str()));
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
        // A film has no episode, so a request for one finds only a film that stands in for each.
        let asks_episode = request.episode.is_some();
        let listed = self
            .0
            .iter()
            .filter(|film| film.title == request.title && (!asks_episode || film.every_episode));
        Ok(listed
            .map(|film| FoundSource {
                url: film.url.clone(),
                kind: film.kind,
                group: "open-films".to_owned(),
                headers: BTreeMap::new(),
                // The flag means the finder fetched and read this address. This one fetches
                // nothing, so it cannot claim it: the operator vouched for the licence, which is
                // not the same statement. Saying true here made the rule that keeps unchecked
                // sources away from kids tiers do nothing at all.
                verified: false,
                container: None,
                height: film.height,
                audio: film.audio.clone(),
            })
            .collect())
    }
}
