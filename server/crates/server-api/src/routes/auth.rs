//! Sign in. Every route answers the same shape, tokens leave once, and no answer says whether an
//! address has an account.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use contract::account::{
    CodeSend, CodeSent, CodeSignIn, Confirm, Confirmed, DeviceCode, DeviceCodeRequest,
    DeviceDecision, DevicePoll, DeviceTokenAnswer, DeviceTokenRequest, Done, PasswordSignIn,
    RecoveryFinish, RecoveryStart, RefreshRequest, SignUp, SignedIn as SignedInWire,
};
use server_accounts::flows::{DEVICE_CODE_S, DEVICE_POLL_S};
use server_accounts::{DevicePollState, Door, IssuedTokens, SignUpAsk, SignedIn};

use crate::answer::{Body, Keep, send};
use crate::caller::{Caller, ClientOrigin};
use crate::error::{ApiError, ApiResult};
use crate::state::Shared;

fn door(state: &Shared) -> ApiResult<&Door> {
    state.door.as_ref().ok_or_else(|| ApiError::off("accounts"))
}

fn wire(found: SignedIn) -> SignedInWire {
    let SignedIn {
        identity, tokens, ..
    } = found;
    let IssuedTokens {
        access,
        refresh,
        expires_in,
    } = tokens;
    SignedInWire {
        user_id: identity.account.to_string(),
        email: identity.email,
        access_token: access,
        refresh_token: refresh,
        expires_in,
    }
}

/// `POST /v1/auth/password`.
pub async fn password(
    State(state): State<Shared>,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<PasswordSignIn>,
) -> ApiResult<Response> {
    let found = door(&state)?
        .password_sign_in(&asked.email, &asked.password, from.as_origin())
        .await?;
    send(&headers, &wire(found), Keep::Never)
}

/// `POST /v1/auth/code`.
pub async fn send_code(
    State(state): State<Shared>,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<CodeSend>,
) -> ApiResult<Response> {
    let expires_in = door(&state)?.send_code(&asked.email, &from.address).await?;
    send(&headers, &CodeSent { expires_in }, Keep::Never)
}

/// `POST /v1/auth/code/verify`.
pub async fn verify_code(
    State(state): State<Shared>,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<CodeSignIn>,
) -> ApiResult<Response> {
    let found = door(&state)?
        .code_sign_in(&asked.email, &asked.code, from.as_origin())
        .await?;
    send(&headers, &wire(found), Keep::Never)
}

/// `POST /v1/auth/signup`.
pub async fn sign_up(
    State(state): State<Shared>,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<SignUp>,
) -> ApiResult<Response> {
    let new = SignUpAsk {
        email: &asked.email,
        password: &asked.password,
        first_name: &asked.first_name,
        last_name: &asked.last_name,
    };
    let found = door(&state)?.sign_up(&new, from.as_origin()).await?;
    send(&headers, &wire(found), Keep::Never)
}

/// `POST /v1/auth/refresh`. A failed store answers 503 and the device keeps its tokens.
pub async fn refresh(
    State(state): State<Shared>,
    headers: HeaderMap,
    Body(asked): Body<RefreshRequest>,
) -> ApiResult<Response> {
    let (account, tokens) = door(&state)?.refresh_for(&asked.refresh_token).await?;
    let answer = SignedInWire {
        user_id: account.to_string(),
        email: String::new(),
        access_token: tokens.access,
        refresh_token: tokens.refresh,
        expires_in: tokens.expires_in,
    };
    send(&headers, &answer, Keep::Never)
}

/// `POST /v1/auth/confirm`: a step up holds for its window, and a refresh never renews it.
pub async fn confirm(
    State(state): State<Shared>,
    Caller(member): Caller,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<Confirm>,
) -> ApiResult<Response> {
    let valid_s = door(&state)?
        .confirm(
            &member,
            asked.password.as_deref(),
            asked.code.as_deref(),
            &from.address,
        )
        .await?;
    send(&headers, &Confirmed { valid_s }, Keep::Never)
}

/// `POST /v1/auth/recovery/start`.
pub async fn recovery_start(
    State(state): State<Shared>,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<RecoveryStart>,
) -> ApiResult<Response> {
    door(&state)?
        .recovery_start(&asked.email, &from.address)
        .await?;
    send(&headers, &Done { done: true }, Keep::Never)
}

/// `POST /v1/auth/recovery/finish`.
pub async fn recovery_finish(
    State(state): State<Shared>,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<RecoveryFinish>,
) -> ApiResult<Response> {
    door(&state)?
        .recovery_finish(
            &asked.email,
            &asked.code,
            &asked.new_password,
            &from.address,
        )
        .await?;
    send(&headers, &Done { done: true }, Keep::Never)
}

/// `POST /v1/auth/device/code`. The screen shows the code even when it shows a QR code.
pub async fn device_code(
    State(state): State<Shared>,
    headers: HeaderMap,
    Body(asked): Body<DeviceCodeRequest>,
) -> ApiResult<Response> {
    let site = state
        .config
        .site_address
        .as_deref()
        .ok_or_else(|| ApiError::off("device_sign_in"))?;
    let started = door(&state)?.device_start(&asked.device_name).await?;
    let verification_uri = format!("{}/activate", site.trim_end_matches('/'));
    let answer = DeviceCode {
        verification_uri_complete: format!("{verification_uri}?code={}", started.user_code),
        verification_uri,
        device_code: started.device_code,
        user_code: started.user_code,
        expires_in: DEVICE_CODE_S,
        interval: DEVICE_POLL_S,
    };
    send(&headers, &answer, Keep::Never)
}

/// `POST /v1/auth/device/token`.
pub async fn device_token(
    State(state): State<Shared>,
    headers: HeaderMap,
    Body(asked): Body<DeviceTokenRequest>,
) -> ApiResult<Response> {
    let poll = |word| DeviceTokenAnswer {
        signed_in: None,
        poll: Some(word),
    };
    let answer = match door(&state)?.device_poll(&asked.device_code).await? {
        DevicePollState::Pending => poll(DevicePoll::AuthorizationPending),
        DevicePollState::SlowDown => poll(DevicePoll::SlowDown),
        DevicePollState::Denied => poll(DevicePoll::AccessDenied),
        DevicePollState::Expired => poll(DevicePoll::ExpiredToken),
        DevicePollState::Approved(found) => DeviceTokenAnswer {
            signed_in: Some(wire(*found)),
            poll: None,
        },
    };
    send(&headers, &answer, Keep::Never)
}

/// `POST /v1/auth/device/approve`.
pub async fn device_approve(
    State(state): State<Shared>,
    Caller(member): Caller,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<DeviceDecision>,
) -> ApiResult<Response> {
    door(&state)?
        .device_approve(&member, &asked.user_code, &from.address)
        .await?;
    send(&headers, &Done { done: true }, Keep::Never)
}

/// `POST /v1/auth/device/deny`.
pub async fn device_deny(
    State(state): State<Shared>,
    Caller(member): Caller,
    from: ClientOrigin,
    headers: HeaderMap,
    Body(asked): Body<DeviceDecision>,
) -> ApiResult<Response> {
    door(&state)?
        .device_deny(&member, &asked.user_code, &from.address)
        .await?;
    send(&headers, &Done { done: true }, Keep::Never)
}
