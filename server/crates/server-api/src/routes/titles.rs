//! Title routes. `/v1/titles` answers for any row of the catalogue, and every answer goes through
//! the rules, so a refused title and a title the catalogue lacks are told apart by code alone.

use axum::extract::{Path, State};
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use contract::error::ErrorCode;
use contract::screen::{Episode, SeasonSummary};
use contract::{MAX_KEYS_PER_LIST, TitleKey, TitleKind};
use server_catalog::manifest::role;
use server_catalog::seasons::{episodes_of, seasons_of};
use server_catalog::{DataSet, item_key};
use server_rules::Viewer;
use server_rules::age::{RatingSystem, system_of_country};
use server_rules::answer::{SafeSeason, SafeTitleDetail};

use crate::answer::{Keep, send_cleared};
use crate::caller::Reader;
use crate::error::{ApiError, ApiResult};
use crate::state::{AppState, Shared};

/// Seconds a visitor's title page may be kept by anyone.
const PUBLIC_TTL_S: u32 = 3600;

/// The key of a route's kind and id. A bare id has no route, and bad words are not found.
fn key_of(kind: &str, id: &str) -> ApiResult<TitleKey> {
    TitleKey::parse(&format!("{kind}:{id}")).map_err(|_| ApiError::not_found())
}

/// The data set in use.
///
/// # Errors
/// The feature is off with no data set.
pub fn data(state: &AppState) -> ApiResult<std::sync::Arc<DataSet>> {
    let slot = state.data.as_ref().ok_or_else(|| ApiError::off("titles"))?;
    Ok(slot.load())
}

fn no_longer_available() -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        ErrorCode::NoLongerAvailable,
        "no_longer_available",
    )
}

fn packed(key: TitleKey) -> u32 {
    let word = match key.kind() {
        TitleKind::Tv => "tv",
        _ => "movie",
    };
    item_key(word, key.id())
}

/// The rating system of a request, from the region in its language. A profile's setting will sit
/// in front of this.
fn rating_system(headers: &HeaderMap) -> RatingSystem {
    let region = headers
        .get(ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.split([',', ';']).next())
        .and_then(|tag| tag.split('-').nth(1));
    system_of_country(region)
}

/// The title page of a key for a viewer. A title outside the catalogue answers no longer
/// available, and a title the rules refuse answers refused.
fn detail(
    state: &AppState,
    set: &DataSet,
    viewer: &Viewer,
    key: TitleKey,
    system: RatingSystem,
) -> ApiResult<SafeTitleDetail> {
    let packed = packed(key);
    if !set.catalog.contains_key(&packed) {
        return Err(no_longer_available());
    }
    let visible = viewer
        .see(set, packed)
        .ok_or_else(|| ApiError::refused("title_refused"))?;
    state
        .pages
        .title_page(set, viewer, visible, system)
        .ok_or_else(no_longer_available)
}

/// The catalogue file of a data set.
fn catalogue(set: &DataSet) -> Option<&std::path::Path> {
    set.files.get(role::CATALOG)
}

/// The seasons a series has. A film has none.
fn season_list(set: &DataSet, key: TitleKey) -> Vec<SeasonSummary> {
    if key.kind() != TitleKind::Tv {
        return Vec::new();
    }
    catalogue(set)
        .map(|db| seasons_of(db, key.id()))
        .unwrap_or_default()
        .into_iter()
        .map(|season| SeasonSummary {
            number: season.number,
            episode_count: season.episodes,
        })
        .collect()
}

/// `GET /v1/titles/{kind}/{id}/seasons/{n}`.
pub async fn season(
    State(state): State<Shared>,
    reader: Reader,
    Path((kind, id, number)): Path<(String, String, u32)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let key = key_of(&kind, &id)?;
    let set = data(&state)?;
    let visible = reader
        .viewer
        .see(&set, packed(key))
        .ok_or_else(|| ApiError::refused("title_refused"))?;
    let (name, facts) = catalogue(&set)
        .filter(|_| key.kind() == TitleKind::Tv)
        .and_then(|db| episodes_of(db, key.id(), number))
        .ok_or_else(ApiError::not_found)?;
    let episodes = facts
        .into_iter()
        .map(|fact| Episode {
            number: fact.number,
            name: fact.name,
            overview: fact.overview,
            runtime_min: fact.runtime,
            images: state.pages.still(fact.still.as_deref()),
            percent: None,
        })
        .collect();
    let page = SafeSeason::new(visible, number, name, episodes);
    send_cleared(&headers, &page, Keep::Private)
}

/// `GET /v1/titles/{kind}/{id}`.
pub async fn title(
    State(state): State<Shared>,
    reader: Reader,
    Path((kind, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let key = key_of(&kind, &id)?;
    let set = data(&state)?;
    let mut page = detail(&state, &set, &reader.viewer, key, rating_system(&headers))?
        .with_seasons(season_list(&set, key));
    if let (Some(watching), Some(library)) = (&reader.watching, &state.library) {
        let saved = library
            .store()
            .list(watching.scope.profile(), MAX_KEYS_PER_LIST)
            .await?;
        let rated = library
            .store()
            .rated(watching.scope.profile(), MAX_KEYS_PER_LIST)
            .await?;
        let said = rated.iter().find(|(title, _)| *title == key);
        page = page
            .saved(saved.contains(&key))
            .rated(said.map(|(_, rating)| *rating));
    }
    send_cleared(&headers, &page, Keep::Private)
}

/// `GET /v1/public/titles/{kind}/{id}`: under the visitor tier, with no profile. Off when the
/// deployer stated no tier, so a missing setting never means no limit.
pub async fn public_title(
    State(state): State<Shared>,
    Path((kind, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let tier = state
        .config
        .visitor_tier
        .ok_or_else(|| ApiError::off("public_titles"))?;
    let key = key_of(&kind, &id)?;
    let set = data(&state)?;
    let page = detail(
        &state,
        &set,
        &Viewer::visitor(tier),
        key,
        rating_system(&headers),
    )?;
    send_cleared(&headers, &page, Keep::Public(PUBLIC_TTL_S))
}
