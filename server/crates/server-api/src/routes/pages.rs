//! Pages: the menu, screens, more of a screen, and search. Page building is CPU work, so it runs
//! on blocking threads behind one semaphore, and a request past its deadline is shed by the guard.

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::screen::ScreenName;
use serde::Deserialize;
use server_pages::{Personal, ScreenAsk, personal_of};

use crate::answer::{Keep, send, send_cleared};
use crate::caller::{Reader, client_class};
use crate::error::{ApiError, ApiResult};
use crate::routes::titles::data;
use crate::state::Shared;

/// The query of `/v1/screens/{name}`.
#[derive(Debug, Deserialize)]
pub struct ScreenQuery {
    genre: Option<String>,
}

/// The query of search routes.
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    q: String,
}

fn name_of(word: &str) -> ApiResult<ScreenName> {
    let name: ScreenName =
        serde_json::from_value(serde_json::Value::from(word)).map_err(|_| ApiError::not_found())?;
    if name == ScreenName::Unknown {
        return Err(ApiError::not_found());
    }
    Ok(name)
}

/// Run page work off the async threads, a bounded number at a time.
async fn work<T: Send + 'static>(
    state: &Shared,
    job: impl FnOnce() -> T + Send + 'static,
) -> ApiResult<T> {
    let closed = || ApiError::busy(contract::error::ErrorCode::Overloaded, "overloaded", 2);
    let permit = state
        .page_work
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| closed())?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        job()
    })
    .await
    .map_err(|_| closed())
}

/// What this caller's request asks of the page builder.
async fn ask_of(
    state: &Shared,
    reader: &Reader,
    screen: ScreenName,
    genre: Option<String>,
    headers: &HeaderMap,
) -> ApiResult<ScreenAsk> {
    let now_ms = state.clock.now_ms();
    let class = client_class(headers);
    let Some(watching) = &reader.watching else {
        return Ok(ScreenAsk {
            screen,
            genre,
            class,
            now_ms,
            owner: String::from("visitor"),
            seed: String::new(),
            personal: Personal::default(),
        });
    };
    let scope = &watching.scope;
    let personal = match &state.library {
        Some(library) => personal_of(library, scope).await?,
        None => Personal::default(),
    };
    // with consent off no session seed exists, so a viewer gets the plain order
    let seed = if scope.consent().personal {
        scope.profile().to_string()
    } else {
        String::new()
    };
    Ok(ScreenAsk {
        screen,
        genre,
        class,
        now_ms,
        owner: format!("{}:{}", scope.session(), scope.profile()),
        seed,
        personal,
    })
}

/// `GET /v1/nav`.
pub async fn nav(
    State(state): State<Shared>,
    reader: Reader,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let set = data(&state)?;
    let tier = reader.viewer.tier();
    let job = state.clone();
    let menu = work(&state, move || job.pages.nav(&set, tier)).await?;
    send(&headers, &menu, Keep::Private)
}

/// `GET /v1/screens/{name}`.
pub async fn screen(
    State(state): State<Shared>,
    reader: Reader,
    Path(name): Path<String>,
    Query(query): Query<ScreenQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let name = name_of(&name)?;
    let set = data(&state)?;
    let ask = ask_of(&state, &reader, name, query.genre, &headers).await?;
    let job = state.clone();
    let viewer = reader.viewer;
    let page = work(&state, move || job.pages.screen(&set, &viewer, &ask)).await??;
    send_cleared(&headers, &page, Keep::Private)
}

/// `GET /v1/more/{cursor}`: the screen comes from the cursor, and the rows are filtered again.
pub async fn more(
    State(state): State<Shared>,
    reader: Reader,
    Path(cursor): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let set = data(&state)?;
    let ask = ask_of(&state, &reader, ScreenName::Home, None, &headers).await?;
    let job = state.clone();
    let viewer = reader.viewer;
    let part = work(&state, move || job.pages.more(&set, &viewer, &cursor, &ask)).await??;
    send_cleared(&headers, &part, Keep::Never)
}

/// `GET /v1/search`.
pub async fn search(
    State(state): State<Shared>,
    reader: Reader,
    Query(query): Query<SearchQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    if query.q.trim().is_empty() {
        return Err(ApiError::bad("q"));
    }
    let set = data(&state)?;
    let ask = ask_of(&state, &reader, ScreenName::Search, None, &headers).await?;
    let job = state.clone();
    let viewer = reader.viewer;
    let page = work(&state, move || {
        job.pages.search(&set, &viewer, &query.q, &ask)
    })
    .await?;
    send_cleared(&headers, &page, Keep::Never)
}

/// `GET /v1/search/suggest`.
pub async fn suggest(
    State(state): State<Shared>,
    reader: Reader,
    Query(query): Query<SearchQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let set = data(&state)?;
    let ask = ask_of(&state, &reader, ScreenName::Search, None, &headers).await?;
    let job = state.clone();
    let viewer = reader.viewer;
    let names = work(&state, move || {
        job.pages.suggest(&set, &viewer, &query.q, &ask)
    })
    .await?;
    send_cleared(&headers, &names, Keep::Never)
}
