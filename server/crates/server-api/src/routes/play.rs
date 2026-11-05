//! Play routes. A title refused by the rules reaches no finder, and a play id works only for the
//! session and profile that made it.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::account::Done;
use contract::client::CapabilityCard;
use contract::play::{Marks, MarksRequest, PlayEvents, PlayRequest, Report};
use server_play::{Play, PlayAsk, pick_marks};
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

/// Windows from the marks the data set holds. With a cue detector this falls back here.
fn windows(entry: &serde_json::Value, duration_ms: u64) -> Vec<contract::play::TimeWindow> {
    use contract::play::{TimeWindow, WindowKind};
    let at = |ratio: f64| {
        let share = ratio.clamp(0.0, 1.0);
        let duration = duration_ms as f64;
        (share * duration) as u64
    };
    let span = |value: &serde_json::Value, kind: WindowKind| -> Option<TimeWindow> {
        let pair = value.as_array()?;
        let (start, end) = (at(pair.first()?.as_f64()?), at(pair.get(1)?.as_f64()?));
        Some(TimeWindow {
            kind,
            start_ms: start,
            end_ms: end,
            seek_to_ms: Some(end),
            text: None,
        })
    };
    let mut out: Vec<TimeWindow> = Vec::new();
    out.extend(span(&entry["recap"], WindowKind::Recap));
    out.extend(span(&entry["intro"], WindowKind::Intro));
    if let Some(start) = entry["credits"].as_f64().map(at) {
        out.push(TimeWindow {
            kind: WindowKind::Credits,
            start_ms: start,
            end_ms: duration_ms,
            seek_to_ms: None,
            text: None,
        });
    }
    out
}

/// `POST /v1/play/{id}/marks`: after first frame, series only. With no cue reader yet the answer
/// is the engine marks, as a title with no cues gets.
pub async fn marks(
    State(state): State<Shared>,
    watching: Watching,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<MarksRequest>,
) -> ApiResult<Response> {
    let title = gate(&state)?.check(&id, &watching.scope)?;
    let set = data(&state)?;
    let found = set.play_facts.marks.get(&title.to_string());
    let picked = pick_marks(found, None, false);
    let answer = Marks {
        windows: windows(&picked, asked.duration_ms),
    };
    send(&headers, &answer, Keep::Never)
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
    if let Some(log) = &state.play_log {
        log.events(watching.scope.profile(), &asked.events).await;
    }
    send(&headers, &Done { done: true }, Keep::Never)
}

/// `POST /v1/reports`.
pub async fn report(
    State(state): State<Shared>,
    Caller(_): Caller,
    headers: HeaderMap,
    Body(asked): Body<Report>,
) -> ApiResult<Response> {
    if let Some(log) = &state.play_log {
        log.report(&asked).await;
    }
    send(&headers, &Done { done: true }, Keep::Never)
}
