//! Profiles. The active profile lives on the session row and nowhere else.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::account::{
    Done, PinSet, Profile as ProfileView, ProfileCreate, ProfileEnter, ProfileEntered,
    ProfileLimits, ProfileList, ProfileUpdate,
};
use contract::screen::ScreenName;
use server_accounts::{Door, Profile, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch};

use crate::answer::{Body, Keep, send};
use crate::caller::Caller;
use crate::error::{ApiError, ApiResult};
use crate::state::Shared;

fn door(state: &Shared) -> ApiResult<&Door> {
    state.door.as_ref().ok_or_else(|| ApiError::off("accounts"))
}

fn view(row: &Profile) -> ProfileView {
    ProfileView {
        id: row.id.to_string(),
        name: row.name.clone(),
        kids: row.kind == ProfileKind::Kids,
        main: row.is_main,
        has_pin: row.has_pin,
        avatar: Some(row.avatar.clone()),
    }
}

fn done() -> Done {
    Done { done: true }
}

/// `GET /v1/profiles`.
pub async fn list(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let rows = door(&state)?.profiles(&member).await?;
    let profiles = rows.iter().map(view).collect();
    send(&headers, &ProfileList { profiles }, Keep::Never)
}

/// `POST /v1/profiles`.
pub async fn create(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<ProfileCreate>,
) -> ApiResult<Response> {
    let row = door(&state)?
        .create_profile(&member, &asked.name, asked.kids)
        .await?;
    send(&headers, &view(&row), Keep::Never)
}

/// `PATCH /v1/profiles/{id}`.
pub async fn update(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<ProfileUpdate>,
) -> ApiResult<Response> {
    let patch = ProfilePatch {
        name: asked.name,
        avatar: asked.avatar,
        kind: asked.kids.map(|kids| {
            if kids {
                ProfileKind::Kids
            } else {
                ProfileKind::Adult
            }
        }),
    };
    let row = door(&state)?
        .update_profile(&member, ProfileId::parse(&id)?, patch)
        .await?;
    send(&headers, &view(&row), Keep::Never)
}

/// `DELETE /v1/profiles/{id}`.
pub async fn delete(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    door(&state)?
        .delete_profile(&member, ProfileId::parse(&id)?)
        .await?;
    send(&headers, &done(), Keep::Never)
}

/// `POST /v1/profiles/{id}/enter`: the PIN is checked here, and the answer names the home.
pub async fn enter(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<ProfileEnter>,
) -> ApiResult<Response> {
    let row = door(&state)?
        .enter_profile(&member, ProfileId::parse(&id)?, asked.pin.as_deref())
        .await?;
    let home = if row.kind == ProfileKind::Kids {
        ScreenName::Kids
    } else {
        ScreenName::Home
    };
    let answer = ProfileEntered {
        profile: view(&row),
        home,
    };
    send(&headers, &answer, Keep::Never)
}

/// `PUT /v1/profiles/{id}/pin`.
pub async fn set_pin(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<PinSet>,
) -> ApiResult<Response> {
    door(&state)?
        .set_pin(&member, ProfileId::parse(&id)?, asked.pin.as_deref())
        .await?;
    send(&headers, &done(), Keep::Never)
}

/// `POST /v1/profiles/{id}/main`.
pub async fn set_main(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    door(&state)?
        .set_main_profile(&member, ProfileId::parse(&id)?)
        .await?;
    send(&headers, &done(), Keep::Never)
}

/// `GET /v1/profiles/{id}/limits`.
pub async fn limits(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let row = door(&state)?
        .profile(&member, ProfileId::parse(&id)?)
        .await?;
    let answer = ProfileLimits {
        age_rating: row.age_limit,
        blocked_titles: row.blocked_names,
    };
    send(&headers, &answer, Keep::Never)
}

/// `PUT /v1/profiles/{id}/limits`: the one way to change these, and it needs a step up.
pub async fn set_limits(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<ProfileLimits>,
) -> ApiResult<Response> {
    let row = ProfileLimitsRow {
        age_limit: asked.age_rating.clone(),
        blocked_names: asked.blocked_titles.clone(),
    };
    door(&state)?
        .set_limits(&member, ProfileId::parse(&id)?, row)
        .await?;
    send(&headers, &asked, Keep::Never)
}
