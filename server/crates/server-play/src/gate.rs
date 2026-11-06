//! `POST /v1/play`: the door knows device and profile, eligibility runs before any finder call, and
//! a play id stays bound to the session and profile that made it.

use std::sync::Arc;
use std::time::Duration;

use contract::client::CapabilityCard;
use contract::finder::{FindOutcome, FindRequest};
use contract::play::{AgeCard, Attempt, MoreAttempts, PlayRequest};
use contract::{EpisodeRef, TitleKey, TitleKind};
use moka::sync::Cache;
use server_accounts::ProfileScope;
use server_catalog::catalog::cert_label;
use server_catalog::{DataSet, item_key};
use server_library::Library;
use server_rules::Viewer;
use server_rules::answer::{PlayExtras, SafePlayAnswer};
use thiserror::Error;
use uuid::Uuid;

use crate::finder::Finder;
use crate::rank::{Viewing, rank};

/// A play session lives this long.
const PLAY_TTL_S: u64 = 6 * 3600;
const PLAY_SESSIONS_MAX: u64 = 100_000;
/// The budget of a find when the operator set none for this class.
const DEFAULT_BUDGET_MS: u32 = 8000;
/// Most attempts one answer holds.
const ATTEMPTS_MAX: usize = 24;

/// Why a play has no answer.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayError {
    /// The title is outside the catalogue.
    #[error("no longer available")]
    Gone,
    /// The rules refused this viewer.
    #[error("refused")]
    Refused,
    /// The request does not fit the title.
    #[error("bad request")]
    Bad,
    /// No finder answered. The rest of the service works.
    #[error("no finder answered")]
    NoFinder,
    /// Finders answered and found nothing.
    #[error("no source")]
    NoSource,
    /// Not a play of this session and profile.
    #[error("no such play")]
    Unknown,
}

/// One play in progress.
#[derive(Clone, Debug)]
struct PlaySession {
    owner: String,
    title: TitleKey,
    episode: Option<EpisodeRef>,
    job: Option<String>,
    viewing: Viewing,
    card: Option<CapabilityCard>,
}

/// What a play request brings besides its title.
#[derive(Clone, Debug)]
pub struct PlayAsk {
    pub request: PlayRequest,
    pub card: Option<CapabilityCard>,
    pub budget_ms: Option<u32>,
    pub embeds: bool,
}

/// The play gate.
pub struct Play {
    finder: Arc<dyn Finder>,
    sessions: Cache<String, PlaySession>,
}

impl std::fmt::Debug for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Play")
    }
}

fn owner_of(scope: &ProfileScope) -> String {
    format!("{}:{}", scope.session(), scope.profile())
}

fn packed(key: TitleKey) -> u32 {
    let word = if key.kind() == TitleKind::Tv {
        "tv"
    } else {
        "movie"
    };
    item_key(word, key.id())
}

fn episode_fits(title: TitleKey, episode: Option<&EpisodeRef>) -> bool {
    match episode {
        Some(found) => {
            title.kind() == TitleKind::Tv && found.season <= 9_999 && found.episode <= 99_999
        }
        None => title.kind() != TitleKind::Tv,
    }
}

/// The content reasons of a video, words from a fixed list.
fn reasons_of(data: &DataSet, title: TitleKey, episode: Option<&EpisodeRef>) -> Vec<String> {
    let entry = data.play_facts.reasons.get(&title.to_string());
    let (season, number) = (episode.map(|e| e.season), episode.map(|e| e.episode));
    crate::marks::pick_reasons(entry, season, number)
        .as_array()
        .map(|words| {
            words
                .iter()
                .filter_map(|word| word.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

impl Play {
    /// A gate over a finder.
    #[must_use]
    pub fn new(finder: Arc<dyn Finder>) -> Self {
        Self {
            finder,
            sessions: Cache::builder()
                .max_capacity(PLAY_SESSIONS_MAX)
                .time_to_live(Duration::from_secs(PLAY_TTL_S))
                .build(),
        }
    }

    /// Ask to play. Refused titles reach no finder.
    ///
    /// # Errors
    /// See [`PlayError`].
    pub async fn play(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        scope: &ProfileScope,
        library: &Library,
        ask: PlayAsk,
    ) -> Result<SafePlayAnswer, PlayError> {
        let PlayRequest { title, episode } = ask.request;
        if !episode_fits(title, episode.as_ref()) {
            return Err(PlayError::Bad);
        }
        let key = packed(title);
        if !data.catalog.contains_key(&key) {
            return Err(PlayError::Gone);
        }
        let visible = viewer.see(data, key).ok_or(PlayError::Refused)?;
        let item = visible.item();
        let settings = library
            .store()
            .settings(scope.profile())
            .await
            .unwrap_or_default();
        let request = FindRequest {
            title,
            episode: episode.clone(),
            name: item.title.clone(),
            year: item.release_year.and_then(|year| u32::try_from(year).ok()),
            budget_ms: ask.budget_ms.unwrap_or(DEFAULT_BUDGET_MS),
        };
        let found = self
            .finder
            .find(&request)
            .await
            .map_err(|_| PlayError::NoFinder)?;
        if found.sources.is_empty() && found.outcome != FindOutcome::Pending && found.job.is_none()
        {
            return Err(PlayError::NoSource);
        }
        let viewing = Viewing {
            language: settings
                .audio_language
                .clone()
                .unwrap_or_else(|| String::from("en")),
            original_language: item.original_language.clone(),
            kids: viewer.tier().is_kids(),
            embeds: ask.embeds,
        };
        let play_id = Uuid::new_v4().to_string();
        let (mut attempts, dropped) = rank(found.sources, ask.card.as_ref(), &viewing, &play_id);
        attempts.truncate(ATTEMPTS_MAX);
        let start_at_ms = library
            .start_at_ms(scope.profile(), title, episode.clone())
            .await
            .unwrap_or(0);
        let more = found.job.is_some();
        let reasons = reasons_of(data, title, episode.as_ref());
        self.sessions.insert(
            play_id.clone(),
            PlaySession {
                owner: owner_of(scope),
                title,
                episode: episode.clone(),
                job: found.job,
                viewing,
                card: ask.card,
            },
        );
        let extras = PlayExtras {
            play_id,
            start_at_ms,
            age_card: Some(AgeCard {
                label: cert_label(item.cert).to_owned(),
                reasons,
            }),
            audio_default: settings.audio_language,
            subtitle_default: settings.subtitle_language,
            next_episode: None,
            autoplay_next: settings.autoplay_next,
            dropped,
            more,
        };
        Ok(SafePlayAnswer::new(visible, attempts, extras))
    }

    fn session(&self, id: &str, scope: &ProfileScope) -> Result<PlaySession, PlayError> {
        self.sessions
            .get(id)
            .filter(|held| held.owner == owner_of(scope))
            .ok_or(PlayError::Unknown)
    }

    /// Attempts that arrived late. The title is checked again, so a play outlives no change.
    ///
    /// # Errors
    /// `Unknown` for a play of another session or profile.
    pub async fn more(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        scope: &ProfileScope,
        id: &str,
    ) -> Result<MoreAttempts, PlayError> {
        let mut held = self.session(id, scope)?;
        viewer
            .see(data, packed(held.title))
            .ok_or(PlayError::Refused)?;
        let Some(job) = held.job.clone() else {
            return Ok(MoreAttempts {
                attempts: Vec::new(),
                more: false,
            });
        };
        let found = self
            .finder
            .rest(&job)
            .await
            .map_err(|_| PlayError::NoFinder)?;
        let attempts: Vec<Attempt> = rank(found.sources, held.card.as_ref(), &held.viewing, id).0;
        held.job = found.job;
        let more = held.job.is_some();
        self.sessions.insert(id.to_owned(), held);
        Ok(MoreAttempts { attempts, more })
    }

    /// Check that a play belongs to this session and profile.
    ///
    /// # Errors
    /// `Unknown` otherwise.
    pub fn check(
        &self,
        id: &str,
        scope: &ProfileScope,
    ) -> Result<(TitleKey, Option<EpisodeRef>), PlayError> {
        let held = self.session(id, scope)?;
        Ok((held.title, held.episode))
    }
}
