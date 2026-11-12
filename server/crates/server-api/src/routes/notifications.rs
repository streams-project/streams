//! The notification feed, unread count and read marks for a profile.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::account::Done;
use contract::notification::NotificationRead;
use serde::Deserialize;
use server_catalog::DataSet;
use server_library::Library;
use server_rules::Viewer;

use crate::answer::{Body, Keep, send};
use crate::caller::Watching;
use crate::deadline;
use crate::error::{ApiError, ApiResult};
use crate::state::Shared;

fn library(state: &Shared) -> ApiResult<&Library> {
    state
        .library
        .as_ref()
        .ok_or_else(|| ApiError::off("accounts"))
}

fn data(state: &Shared) -> ApiResult<Arc<DataSet>> {
    state
        .data
        .as_ref()
        .map(|slot| slot.load())
        .ok_or_else(|| ApiError::off("catalog"))
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct NotificationQuery {
    pub cursor: Option<String>,
}

/// `GET /v1/notifications`.
pub async fn list(
    State(state): State<Shared>,
    watching: Watching,
    Query(query): Query<NotificationQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let set = data(&state)?;
    let lib = library(&state)?;
    let store = lib.store();
    let profile = watching.scope.profile();

    let viewer = Viewer::of(&watching.scope)?;
    let now_ms = state.clock.now_ms();

    let reminded = deadline::store(store.reminders(profile, 100)).await?;
    let in_list = deadline::store(store.list(profile, 100)).await?;
    let in_progress = deadline::store(store.seen(profile, 100)).await?;
    let read_state = deadline::store(store.notification_reads(profile)).await?;

    let answer = state.pages.notifications(
        &set,
        &viewer,
        now_ms,
        &reminded,
        &in_list,
        &in_progress,
        &read_state,
        query.cursor.as_deref(),
    );
    send(&headers, &answer, Keep::Never)
}

/// `GET /v1/notifications/count`.
pub async fn count(
    State(state): State<Shared>,
    watching: Watching,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let set = data(&state)?;
    let lib = library(&state)?;
    let store = lib.store();
    let profile = watching.scope.profile();

    let viewer = Viewer::of(&watching.scope)?;
    let now_ms = state.clock.now_ms();

    let reminded = deadline::store(store.reminders(profile, 100)).await?;
    let in_list = deadline::store(store.list(profile, 100)).await?;
    let in_progress = deadline::store(store.seen(profile, 100)).await?;
    let read_state = deadline::store(store.notification_reads(profile)).await?;

    let answer = state.pages.notification_count(
        &set,
        &viewer,
        now_ms,
        &reminded,
        &in_list,
        &in_progress,
        &read_state,
    );
    send(&headers, &answer, Keep::Never)
}

/// `POST /v1/notifications/read`.
pub async fn mark_read(
    State(state): State<Shared>,
    watching: Watching,
    headers: HeaderMap,
    Body(body): Body<NotificationRead>,
) -> ApiResult<Response> {
    let lib = library(&state)?;
    let mut tx = deadline::store(lib.store().begin(watching.scope.profile())).await?;
    let now_ms = state.clock.now_ms();
    deadline::store(tx.mark_notifications_read(&body.up_to, now_ms)).await?;
    deadline::store(tx.commit()).await?;
    send(&headers, &Done { done: true }, Keep::Never)
}
