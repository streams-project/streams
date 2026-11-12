//! Commands and changes. A device says what changed and reads what others changed. No snapshot and
//! no list diff exists on this path.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::TitleKey;
use contract::account::ProfileSettings;
use contract::command::CommandBatch;
use serde::Deserialize;
use server_catalog::{DataSet, item_key};
use server_library::{Library, SettingsView, TitleFacts};
use server_rules::Viewer;
use server_rules::answer::SafeChanges;

use crate::answer::{Body, Keep, send, send_cleared};
use crate::caller::Watching;
use crate::deadline;
use crate::error::{ApiError, ApiResult};
use crate::routes::titles::data;
use crate::state::Shared;

fn library(state: &Shared) -> ApiResult<&Library> {
    state
        .library
        .as_ref()
        .ok_or_else(|| ApiError::off("accounts"))
}

fn packed(key: TitleKey) -> u32 {
    let word = if key.kind() == contract::TitleKind::Tv {
        "tv"
    } else {
        "movie"
    };
    item_key(word, key.id())
}

/// The library asks the catalogue for genres through this, and never reads it.
struct CatalogFacts(Option<std::sync::Arc<DataSet>>);

impl TitleFacts for CatalogFacts {
    fn genres(&self, title: TitleKey) -> Vec<String> {
        let found = self
            .0
            .as_ref()
            .and_then(|set| set.catalog.get(&packed(title)).cloned());
        found.map_or_else(Vec::new, |item| item.genres)
    }
}

/// `POST /v1/commands`.
pub async fn commands(
    State(state): State<Shared>,
    watching: Watching,
    headers: HeaderMap,
    Body(batch): Body<CommandBatch>,
) -> ApiResult<Response> {
    let facts = CatalogFacts(state.data.as_ref().map(|slot| slot.load()));
    let answer = deadline::store(library(&state)?.apply(&watching.scope, &facts, &batch)).await?;
    send(&headers, &answer, Keep::Never)
}

/// The query of `/v1/changes`.
#[derive(Debug, Deserialize)]
pub struct ChangesQuery {
    cursor: Option<String>,
}

/// `GET /v1/changes`: each named title passes the rules first, so a title a profile may no longer
/// see does not come back by another route.
pub async fn changes(
    State(state): State<Shared>,
    watching: Watching,
    Query(query): Query<ChangesQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let scope = &watching.scope;
    let found =
        deadline::store(library(&state)?.changes(scope.profile(), query.cursor.as_deref())).await?;
    let set = data(&state)?;
    let viewer = Viewer::of(scope)?;
    let keep = |title: &TitleKey| viewer.see(&set, packed(*title)).is_some();
    send_cleared(&headers, &SafeChanges::new(found, keep), Keep::Never)
}

fn settings_view(view: SettingsView) -> ProfileSettings {
    ProfileSettings {
        autoplay_next: view.autoplay_next,
        autoplay_previews: view.autoplay_previews,
        audio_language: view.audio_language,
        subtitle_language: view.subtitle_language,
        subtitle_style: view.subtitle_style,
        ui_language: view.ui_language,
        handle: view.handle,
        gender: view.gender,
    }
}

/// `GET /v1/profiles/me/settings`.
pub async fn settings(
    State(state): State<Shared>,
    watching: Watching,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let view = deadline::store(library(&state)?.store().settings(watching.scope.profile())).await?;
    send(&headers, &settings_view(view), Keep::Never)
}
