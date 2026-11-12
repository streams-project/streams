//! The account. Export and delete need a fresh step up.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use contract::account::{
    Account, AccountExport, AccountUpdate, Done, EmailChange, PasswordChange, PhoneChange,
};
use server_accounts::{AccountView, Door};

use crate::answer::{Body, Keep, send};
use crate::caller::Caller;
use crate::deadline;
use crate::error::{ApiError, ApiResult};
use crate::routes::devices::session_view_with_card;
use crate::routes::profiles::shown;
use crate::state::Shared;

fn door(state: &Shared) -> ApiResult<&Door> {
    state.door.as_ref().ok_or_else(|| ApiError::off("accounts"))
}

fn wire(found: AccountView) -> Account {
    Account {
        id: found.identity.account.to_string(),
        email: found.identity.email,
        first_name: found.names.first_name,
        last_name: found.names.last_name,
        phone: found.identity.phone,
    }
}

fn done() -> Done {
    Done { done: true }
}

/// `GET /v1/account`.
pub async fn account(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let found = deadline::store(door(&state)?.account(&member)).await?;
    send(&headers, &wire(found), Keep::Never)
}

/// `PATCH /v1/account`.
pub async fn update(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<AccountUpdate>,
) -> ApiResult<Response> {
    let found = deadline::store(door(&state)?.update_names(
        &member,
        asked.first_name.as_deref(),
        asked.last_name.as_deref(),
    ))
    .await?;
    send(&headers, &wire(found), Keep::Never)
}

/// `DELETE /v1/account`: the account and everything it owns, at once, as the policy says. One
/// function does it for both routes that delete, and it walks `holdings::HELD`; see
/// [`crate::erase`].
pub async fn delete(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    crate::erase::account(&state, &member).await?;
    send(&headers, &done(), Keep::Never)
}

/// Rows of one table one profile's section carries. The tables it bounds are the two with no
/// retention rule of their own; everything else is already capped by the schema.
const PER_PROFILE_MAX: usize = 10_000;

/// `GET /v1/account/export`: everything held about the account, with no PIN hash.
///
/// The sections are not a list kept here. They are the entries of
/// `server_accounts::holdings::HELD` marked [`server_accounts::Export::Under`], which is the same
/// list the deletion path and the schema check read, so a table that is held and not exported is
/// a failing test and not a silent omission. `exported_sections` is checked against that list by
/// `tests/accounts.rs`.
///
/// Three things are deliberately left out and each says so in its entry: the hashes of the
/// password, the PIN and a one-time code. Five more are held and not carried yet, each with its
/// reason in the same place.
pub async fn export(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let door = door(&state)?;
    door.require_step_up(&member)?;
    let profiles = deadline::store(door.profiles(&member)).await?;
    let sessions = deadline::store(door.devices(&member)).await?;
    let mut shape = serde_json::Map::new();
    // The three the contract already declares, so an old client reads this body as it did before.
    let known = AccountExport {
        account: wire(deadline::store(door.account(&member)).await?),
        profiles: {
            let mut rows = Vec::with_capacity(profiles.len());
            for row in &profiles {
                rows.push(shown(&state, row).await?);
            }
            rows
        },
        devices: {
            let mut rows = Vec::with_capacity(sessions.len());
            for row in sessions {
                let card = deadline::store(door.card(row.id)).await?;
                rows.push(session_view_with_card(
                    &row,
                    member.session_id(),
                    card.as_ref(),
                ));
            }
            rows
        },
    };
    let serde_json::Value::Object(known) =
        serde_json::to_value(&known).map_err(|_| body_failed())?
    else {
        return Err(body_failed());
    };
    shape.extend(known);
    shape.insert(
        String::from("profile_limits"),
        serde_json::Value::Array(
            profiles
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "profile": row.id.to_string(),
                        "age_rating": row.age_limit,
                        "blocked_titles": row.blocked_names,
                    })
                })
                .collect(),
        ),
    );
    shape.insert(
        String::from("account_events"),
        serde_json::json!(deadline::store(door.whole_audit_log(&member)).await?),
    );
    for (name, value) in library_sections(&state, &profiles).await? {
        shape.insert(name, value);
    }
    let missing: Vec<&str> = server_accounts::holdings::export_names()
        .into_iter()
        .filter(|name| !shape.contains_key(*name))
        .collect();
    debug_assert!(
        missing.is_empty(),
        "the export names {missing:?} in holdings::HELD and carries no section for them"
    );
    send(&headers, &serde_json::Value::Object(shape), Keep::Never)
}

/// A body that cannot be built is a failure of this server, not of the request. Nothing in the
/// shape can fail to serialize, so this is unreachable; it is here instead of a panic.
fn body_failed() -> ApiError {
    ApiError::busy(contract::error::ErrorCode::Overloaded, "try_again", 5)
}

/// The sections that come from the library store, one entry per profile in each.
async fn library_sections(
    state: &Shared,
    profiles: &[server_accounts::Profile],
) -> ApiResult<Vec<(String, serde_json::Value)>> {
    let mut library_rows = Vec::new();
    let mut progress_rows = Vec::new();
    let mut hidden_rows = Vec::new();
    let mut settings_rows = Vec::new();
    let mut taste_rows = Vec::new();
    let mut reminder_rows = Vec::new();
    let mut read_rows = Vec::new();
    if let Some(library) = &state.library {
        let store = library.store();
        for row in profiles {
            let id = row.id;
            let at = id.to_string();
            let list = deadline::store(store.list(id, PER_PROFILE_MAX)).await?;
            let rated = deadline::store(store.rated(id, PER_PROFILE_MAX)).await?;
            library_rows.push(serde_json::json!({
                "profile": at,
                "list": list.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "ratings": rated
                    .iter()
                    .map(|(title, rating)| serde_json::json!({
                        "title": title.to_string(),
                        "rating": rating,
                    }))
                    .collect::<Vec<_>>(),
            }));
            progress_rows.push(serde_json::json!({
                "profile": at,
                "positions": store
                    .positions(id, PER_PROFILE_MAX)
                    .await?
                    .iter()
                    .map(|row| serde_json::json!({
                        "title": row.title.to_string(),
                        "season": row.episode.as_ref().map(|at| at.season),
                        "episode": row.episode.as_ref().map(|at| at.episode),
                        "position_s": row.position_s,
                        "duration_s": row.duration_s,
                        "completed": row.completed,
                        "changed_at_ms": row.changed_ms,
                    }))
                    .collect::<Vec<_>>(),
            }));
            hidden_rows.push(serde_json::json!({
                "profile": at,
                "titles": store
                    .hidden_titles(id, PER_PROFILE_MAX)
                    .await?
                    .iter()
                    .map(|row| serde_json::json!({
                        "title": row.title.to_string(),
                        "hidden_at_ms": row.hidden_ms,
                    }))
                    .collect::<Vec<_>>(),
            }));
            let settings = deadline::store(store.settings(id)).await?;
            settings_rows.push(serde_json::json!({
                "profile": at,
                "autoplay_next": settings.autoplay_next,
                "autoplay_previews": settings.autoplay_previews,
                "audio_language": settings.audio_language,
                "subtitle_language": settings.subtitle_language,
                "subtitle_style": settings.subtitle_style,
                "ui_language": settings.ui_language,
                "handle": settings.handle,
                "gender": settings.gender,
            }));
            let taste = deadline::store(store.taste_of(id)).await?;
            taste_rows.push(serde_json::json!({
                "profile": at,
                "genre_weights": taste.weights,
                "version": taste.version,
            }));
            reminder_rows.push(serde_json::json!({
                "profile": at,
                "titles": store
                    .reminders(id, PER_PROFILE_MAX)
                    .await?
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            }));
            let reads = deadline::store(store.notification_reads(id)).await?;
            read_rows.push(serde_json::json!({
                "profile": at,
                "read_up_to_ms": reads.read_up_to_ms,
                "dismissed": reads.dismissed,
            }));
        }
    }
    Ok(vec![
        (String::from("library"), library_rows.into()),
        (String::from("progress"), progress_rows.into()),
        (String::from("hidden"), hidden_rows.into()),
        (String::from("settings"), settings_rows.into()),
        (String::from("taste"), taste_rows.into()),
        (String::from("reminders"), reminder_rows.into()),
        (String::from("notification_reads"), read_rows.into()),
    ])
}

/// `POST /v1/account/password`.
pub async fn password(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<PasswordChange>,
) -> ApiResult<Response> {
    deadline::store(door(&state)?.change_password(&member, &asked.new_password)).await?;
    send(&headers, &done(), Keep::Never)
}

/// `POST /v1/account/email`.
pub async fn email(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<EmailChange>,
) -> ApiResult<Response> {
    deadline::store(door(&state)?.change_email(&member, &asked.email)).await?;
    send(&headers, &done(), Keep::Never)
}

/// `POST /v1/account/phone`.
pub async fn phone(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<PhoneChange>,
) -> ApiResult<Response> {
    deadline::store(door(&state)?.change_phone(&member, &asked.phone)).await?;
    send(&headers, &done(), Keep::Never)
}
