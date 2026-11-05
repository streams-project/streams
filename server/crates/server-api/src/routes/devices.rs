//! Devices and consent. The card is opaque to the accounts crate and read by play.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::account::{
    Consent as ConsentWire, DeviceCardAnswer, DeviceCardRequest, DeviceList, DeviceSession, Done,
    SignOut,
};
use contract::client::DeviceClass;
use serde::Deserialize;
use server_accounts::{Consent, Door, SessionId};

use crate::answer::{Body, Keep, send};
use crate::caller::Caller;
use crate::error::{ApiError, ApiResult};
use crate::state::Shared;

/// Most bytes of a device key, as the old row allowed.
const DEVICE_KEY_MIN: usize = 16;
const DEVICE_KEY_MAX: usize = 64;

fn door(state: &Shared) -> ApiResult<&Door> {
    state.door.as_ref().ok_or_else(|| ApiError::off("accounts"))
}

/// `POST /v1/devices/me`: a hash at each start, and the whole card when asked for.
pub async fn register(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<DeviceCardRequest>,
) -> ApiResult<Response> {
    if !(DEVICE_KEY_MIN..=DEVICE_KEY_MAX).contains(&asked.device_key.len()) {
        return Err(ApiError::bad("device_key"));
    }
    let card = asked
        .card
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|_| ApiError::bad("card"))?;
    let send_card = door(&state)?
        .save_card(&member, &asked.card_hash, card.as_ref())
        .await?;
    send(&headers, &DeviceCardAnswer { send_card }, Keep::Never)
}

/// `GET /v1/devices`.
pub async fn list(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let sessions = door(&state)?.devices(&member).await?;
    let devices = sessions
        .iter()
        .map(|session| DeviceSession {
            id: session.id.to_string(),
            name: session.device_name.clone(),
            class: DeviceClass::Unknown,
            last_seen_ms: session.last_seen_ms,
            current: session.id == member.session_id(),
        })
        .collect();
    send(&headers, &DeviceList { devices }, Keep::Never)
}

/// The query of `DELETE /v1/devices`.
#[derive(Debug, Deserialize)]
pub struct WhichDevice {
    id: Option<String>,
}

/// `DELETE /v1/devices`: one device, or every other device with no id.
pub async fn remove(
    State(state): State<Shared>,
    Caller(member): Caller,
    Query(which): Query<WhichDevice>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let door = door(&state)?;
    match which.id {
        Some(id) => door.remove_device(&member, SessionId::parse(&id)?).await?,
        None => door.remove_other_devices(&member).await?,
    }
    send(&headers, &Done { done: true }, Keep::Never)
}

/// `PUT /v1/consent`: on this session, default off.
pub async fn set_consent(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<ConsentWire>,
) -> ApiResult<Response> {
    let consent = Consent {
        personal: asked.personal,
    };
    door(&state)?.set_consent(&member, consent).await?;
    // withdrawal deletes what was learned for the profile on this session
    if let (false, Some(library), Some(profile)) =
        (asked.personal, &state.library, member.active_profile())
    {
        library
            .store()
            .forget_taste(profile)
            .await
            .map_err(ApiError::from)?;
    }
    send(&headers, &asked, Keep::Never)
}

/// `POST /v1/auth/signout`.
pub async fn sign_out(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<SignOut>,
) -> ApiResult<Response> {
    door(&state)?.sign_out(&member, asked.everywhere).await?;
    send(&headers, &Done { done: true }, Keep::Never)
}
