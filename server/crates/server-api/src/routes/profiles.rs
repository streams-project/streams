//! Profiles. The active profile lives on the session row and nowhere else.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::account::{
    AvatarSets, Done, PinSet, Profile as ProfileView, ProfileCreate, ProfileEnter, ProfileEntered,
    ProfileLimits, ProfileList, ProfileTransfer, ProfileUpdate,
};
use contract::command::SettingKey;
use contract::screen::ScreenName;
use server_accounts::profile::STARTING_AVATARS;
use server_accounts::{Door, Profile, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch};
use server_rules::{Tier, Viewer};

use crate::answer::{Body, Keep, send, send_cleared};
use crate::caller::Caller;
use crate::error::{ApiError, ApiResult};
use crate::routes::start::has_language;
use crate::routes::titles::data;
use crate::state::Shared;

fn door(state: &Shared) -> ApiResult<&Door> {
    state.door.as_ref().ok_or_else(|| ApiError::off("accounts"))
}

/// The wire view of a profile: a flag for the PIN, never the PIN.
#[must_use]
pub fn view(row: &Profile) -> ProfileView {
    ProfileView {
        id: row.id.to_string(),
        name: row.name.clone(),
        kids: row.kind == ProfileKind::Kids,
        main: row.is_main,
        has_pin: row.has_pin,
        avatar: Some(row.avatar.clone()),
        language: None,
    }
}

/// The wire view with the profile's language, which the library holds.
pub(crate) async fn shown(state: &Shared, row: &Profile) -> ApiResult<ProfileView> {
    let mut answer = view(row);
    if let Some(library) = &state.library {
        answer.language = library.store().settings(row.id).await?.ui_language;
    }
    Ok(answer)
}

/// The profile being edited need not be the active one, so this writes past the command queue.
async fn keep_language(state: &Shared, profile: ProfileId, code: &str) -> ApiResult<()> {
    let library = state
        .library
        .as_ref()
        .ok_or_else(|| ApiError::off("accounts"))?;
    let mut tx = library.store().begin(profile).await?;
    let seq = tx.next_seq().await?;
    let at = state.clock.now_ms();
    tx.put_setting(SettingKey::UiLanguage, code, at, seq)
        .await?;
    tx.commit().await?;
    Ok(())
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
    let mut profiles = Vec::with_capacity(rows.len());
    for row in &rows {
        profiles.push(shown(&state, row).await?);
    }
    send(&headers, &ProfileList { profiles }, Keep::Never)
}

/// `GET /v1/profiles/{id}/feature`: one title for the picker, chosen for this profile's age.
pub async fn feature(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let rows = door(&state)?.profiles(&member).await?;
    let row = rows
        .iter()
        .find(|row| row.id.to_string() == id)
        .ok_or_else(|| ApiError::refused("profile_refused"))?;
    // One function owns "what tier is this profile": the saved age limit wins over the older
    // level, a Kids profile never rises above PG, and a limit that cannot be read answers 503
    // rather than Adult. Nothing here may compute a tier from the Kids flag alone.
    let tier = Tier::of_profile(row)?;
    let set = data(&state)?;
    let tile = state
        .pages
        .feature(&set, &Viewer::visitor(tier), &id)
        .ok_or_else(|| ApiError::off("feature"))?;
    send_cleared(&headers, &tile, Keep::Never)
}

/// `GET /v1/profiles/gate`: up to three titles over the chooser, each with its line. The chooser
/// shows to the whole account, so the titles keep to the most open tier among its grown up
/// profiles, or among its Kids profiles when it has no other.
pub async fn gate(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let rows = door(&state)?.profiles(&member).await?;
    let tier = Tier::of_account(&rows).ok_or_else(|| ApiError::off("gate"))?;
    let set = data(&state)?;
    let answer = state.pages.gate(&set, tier, state.clock.now_ms());
    send_cleared(&headers, &answer, Keep::Never)
}

/// `GET /v1/profiles/{id}/avatars`: the published sets, and only the kids sets for a Kids profile.
pub async fn avatars(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let row = door(&state)?
        .profile(&member, ProfileId::parse(&id)?)
        .await?;
    let mut sets = published_avatars(&state).sets;
    if row.kind == ProfileKind::Kids {
        sets.retain(|set| set.kids);
    }
    send(&headers, &AvatarSets { sets }, Keep::Never)
}

/// True for a picture this profile may wear: one the brand published, and from a Kids set when the
/// profile is a Kids one, which is exactly what `/avatars` offers it. A brand that publishes no
/// pictures leaves the choice open, and the two names the server writes itself always stand.
fn avatar_published(state: &Shared, kids: bool, avatar: &str) -> bool {
    if STARTING_AVATARS.contains(&avatar) {
        return true;
    }
    let sets = published_avatars(state).sets;
    if sets.is_empty() {
        return true;
    }
    sets.iter()
        .filter(|set| set.kids || !kids)
        .flat_map(|set| set.pictures.iter())
        .any(|picture| picture.path == avatar)
}

/// The site's `avatars/sets.json`. A brand with no pictures has none, and that is a normal answer.
fn published_avatars(state: &Shared) -> AvatarSets {
    state
        .config
        .web_root
        .as_ref()
        .and_then(|root| std::fs::read(std::path::Path::new(root).join("avatars/sets.json")).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(AvatarSets { sets: Vec::new() })
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
    let profile = ProfileId::parse(&id)?;
    if let Some(code) = &asked.language
        && !has_language(&state, code)
    {
        return Err(ApiError::refused("language_refused"));
    }
    if let Some(avatar) = &asked.avatar {
        let row = door(&state)?.profile(&member, profile).await?;
        let kids = asked.kids.unwrap_or(row.kind == ProfileKind::Kids);
        if !avatar_published(&state, kids, avatar) {
            return Err(ApiError::bad("avatar"));
        }
    }
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
        .update_profile(&member, profile, patch)
        .await?;
    if let Some(code) = &asked.language {
        keep_language(&state, profile, code).await?;
    }
    send(&headers, &shown(&state, &row).await?, Keep::Never)
}

/// `DELETE /v1/profiles/{id}`.
pub async fn delete(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    crate::erase::profile(&state, &member, ProfileId::parse(&id)?).await?;
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
        profile: shown(&state, &row).await?,
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

/// `POST /v1/profiles/{id}/transfer`.
pub async fn transfer(
    State(state): State<Shared>,
    Caller(member): Caller,
    Path(id): Path<String>,
    headers: HeaderMap,
    Body(asked): Body<ProfileTransfer>,
) -> ApiResult<Response> {
    door(&state)?
        .transfer_profile(&member, ProfileId::parse(&id)?, &asked.to_account_email)
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
    // The word is checked with the same function that reads it back, so a rating this server
    // cannot turn into a tier never reaches a row. The column takes the same five words, and
    // without this the answer was either "busy, try again" for ever (Postgres refuses the value)
    // or a profile whose every page afterwards says its limit cannot be read.
    if Tier::from_rating(&asked.age_rating).is_none() {
        return Err(ApiError::bad("age_rating"));
    }
    let row = ProfileLimitsRow {
        age_limit: asked.age_rating.clone(),
        blocked_names: asked.blocked_titles.clone(),
    };
    door(&state)?
        .set_limits(&member, ProfileId::parse(&id)?, row)
        .await?;
    send(&headers, &asked, Keep::Never)
}
