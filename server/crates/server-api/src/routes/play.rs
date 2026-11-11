//! Play routes. A title refused by the rules reaches no finder, and a play id works only for the
//! session and profile that made it.

use axum::extract::{Path, Query, State};
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use contract::account::Done;
use contract::client::CapabilityCard;
use contract::error::ErrorCode;
use contract::play::{Marks, MarksRequest, PlayEvents, PlayRequest, Report};
use contract::{EpisodeRef, TitleKey, TitleKind};
use serde::Deserialize;
use server_play::subtitles::detect::detect_marks_from_cues;
use server_play::{Play, PlayAsk, SubtitleError, Subtitles, marks_of, pick_marks, windows_of};
use server_rules::Viewer;

use crate::answer::{Body, Keep, send, send_cleared};
use crate::caller::{Caller, Watching, client_class};
use crate::error::{ApiError, ApiResult};
use crate::routes::titles::data;
use crate::state::Shared;

fn gate(state: &Shared) -> ApiResult<&Play> {
    state.play.as_ref().ok_or_else(|| ApiError::off("playback"))
}

/// `POST /v1/play`.
pub async fn play(
    State(state): State<Shared>,
    watching: Watching,
    headers: HeaderMap,
    Body(request): Body<PlayRequest>,
) -> ApiResult<Response> {
    let live = state.live.values();
    if !live.playback {
        return Err(ApiError::off("playback"));
    }
    let set = data(&state)?;
    let viewer = Viewer::of(&watching.scope)?;
    let library = state
        .library
        .as_ref()
        .ok_or_else(|| ApiError::off("accounts"))?;
    let door = state
        .door
        .as_ref()
        .ok_or_else(|| ApiError::off("accounts"))?;
    let card: Option<CapabilityCard> = door
        .card(watching.scope.session())
        .await?
        .and_then(|value| serde_json::from_value(value).ok());
    let class = client_class(&headers);
    let budget_ms = live
        .start_deadline_ms
        .iter()
        .find(|entry| entry.class == class)
        .map(|entry| entry.value);
    let ask = PlayAsk {
        request,
        card,
        budget_ms,
        embeds: live.embeds,
    };
    let answer = gate(&state)?
        .play(&set, &viewer, &watching.scope, library, ask)
        .await?;
    send_cleared(&headers, &answer, Keep::Never)
}

/// `POST /v1/play/{id}/more`.
pub async fn more(
    State(state): State<Shared>,
    watching: Watching,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let set = data(&state)?;
    let viewer = Viewer::of(&watching.scope)?;
    let attempts = gate(&state)?
        .more(&set, &viewer, &watching.scope, &id)
        .await?;
    send(&headers, &attempts, Keep::Never)
}

/// `POST /v1/play/{id}/marks`: after first frame, series only. With a file the detector reads its
/// cues against the engine marks; with none the answer is the engine marks.
pub async fn marks(
    State(state): State<Shared>,
    watching: Watching,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<MarksRequest>,
) -> ApiResult<Response> {
    let (title, episode) = gate(&state)?.check(&id, &watching.scope)?;
    if title.kind() != TitleKind::Tv {
        return send(
            &headers,
            &Marks {
                windows: Vec::new(),
            },
            Keep::Never,
        );
    }
    let set = data(&state)?;
    let number = episode.map(|found| found.episode);
    let entry = set.play_facts.marks.get(&title.to_string());
    let prior = marks_of(&pick_marks(entry, number, false));
    let cues = match (&state.subtitles, &asked.file) {
        (Some(service), Some(file)) => service.detector_cues(file).await.ok(),
        _ => None,
    };
    let duration = asked.duration_ms as f64 / 1000.0;
    let delay = f64::from(asked.delay_ms) / 1000.0;
    let found = cues.map_or_else(
        || prior.clone(),
        |cues| detect_marks_from_cues(&cues, duration, delay, true, Some(&prior)).marks,
    );
    let answer = Marks {
        windows: windows_of(&found, asked.duration_ms),
    };
    send(&headers, &answer, Keep::Never)
}

/// The query of `/v1/subtitles`.
#[derive(Debug, Deserialize)]
pub struct SubtitleQuery {
    title: String,
    episode: Option<String>,
}

fn subtitles(state: &Shared) -> ApiResult<&Subtitles> {
    state
        .subtitles
        .as_ref()
        .ok_or_else(|| ApiError::off("subtitles"))
}

fn episode_of(text: Option<&str>) -> ApiResult<Option<EpisodeRef>> {
    let Some(text) = text else {
        return Ok(None);
    };
    let (season, episode) = text
        .split_once(':')
        .ok_or_else(|| ApiError::bad("episode"))?;
    let number = |part: &str| part.parse::<u32>().map_err(|_| ApiError::bad("episode"));
    Ok(Some(EpisodeRef {
        season: number(season)?,
        episode: number(episode)?,
    }))
}

fn packed(key: TitleKey) -> u32 {
    let word = if key.kind() == TitleKind::Tv {
        "tv"
    } else {
        "movie"
    };
    server_catalog::item_key(word, key.id())
}

impl From<SubtitleError> for ApiError {
    fn from(error: SubtitleError) -> Self {
        match error {
            SubtitleError::Unknown => Self::not_found(),
            SubtitleError::Unusable => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::NotFound,
                "subtitle_unusable",
            ),
            _ => Self::busy(ErrorCode::Overloaded, "try_again", 5),
        }
    }
}

/// `GET /v1/subtitles`: files per language, in the order to try. Runs after the stream is found.
pub async fn list_subtitles(
    State(state): State<Shared>,
    watching: Watching,
    Query(query): Query<SubtitleQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let title = TitleKey::parse(&query.title).map_err(|_| ApiError::bad("title"))?;
    let episode = episode_of(query.episode.as_deref())?;
    let set = data(&state)?;
    let viewer = Viewer::of(&watching.scope)?;
    viewer
        .see(&set, packed(title))
        .ok_or_else(|| ApiError::refused("title_refused"))?;
    let list = subtitles(&state)?.list(title, episode.as_ref()).await?;
    send(&headers, &list, Keep::Never)
}

/// The query of `/v1/subtitles/{file}`.
#[derive(Debug, Deserialize)]
pub struct SubtitleFileQuery {
    #[serde(default)]
    delay_ms: i64,
}

/// `GET /v1/subtitles/{file}`: clean JSON cues, or WebVTT shifted by the delay for a player that
/// loads a side file. The rules run again for the file's title.
pub async fn subtitle_file(
    State(state): State<Shared>,
    watching: Watching,
    Path(file): Path<String>,
    Query(query): Query<SubtitleFileQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let service = subtitles(&state)?;
    let title = service.title_of(&file).ok_or_else(ApiError::not_found)?;
    let set = data(&state)?;
    let viewer = Viewer::of(&watching.scope)?;
    viewer
        .see(&set, packed(title))
        .ok_or_else(|| ApiError::refused("title_refused"))?;
    let wants_text = headers
        .get(ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("text/vtt"));
    if wants_text {
        let text = service.vtt(&file, query.delay_ms).await?;
        return Ok(([(CONTENT_TYPE, "text/vtt; charset=utf-8")], text).into_response());
    }
    send(&headers, &service.body(&file).await?, Keep::Never)
}

/// `POST /v1/play/{id}/events`.
pub async fn events(
    State(state): State<Shared>,
    watching: Watching,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<PlayEvents>,
) -> ApiResult<Response> {
    gate(&state)?.check(&id, &watching.scope)?;
    // Playback telemetry is a write that exists to learn from this person, so it is reached only
    // through the writer that took their answer. With consent off nothing is written and the call
    // still answers done, as the feed path does for a discarded signal.
    if let Some(learned) = state.learned(&watching.scope) {
        learned.play_events(&asked.events).await;
    }
    send(&headers, &Done { done: true }, Keep::Never)
}

/// `POST /v1/reports`: a build, a code and the client's own words about a failure. It carries no
/// profile and no account column (`0110_play.sql:19`), so it is not a record of a person's
/// viewing and the personal-rows answer does not govern it; it is kept 30 days like the rest of
/// `client_reports`.
pub async fn report(
    State(state): State<Shared>,
    Caller(_): Caller,
    headers: HeaderMap,
    Body(asked): Body<Report>,
) -> ApiResult<Response> {
    if let Some(log) = state.log_of_no_one() {
        log.report(&asked).await;
    }
    send(&headers, &Done { done: true }, Keep::Never)
}
