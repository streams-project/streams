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
use crate::error::{ApiError, ApiResult};
use crate::routes::devices::session_view;
use crate::routes::profiles::view;
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
    let found = door(&state)?.account(&member).await?;
    send(&headers, &wire(found), Keep::Never)
}

/// `PATCH /v1/account`.
pub async fn update(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<AccountUpdate>,
) -> ApiResult<Response> {
    let found = door(&state)?
        .update_names(
            &member,
            asked.first_name.as_deref(),
            asked.last_name.as_deref(),
        )
        .await?;
    send(&headers, &wire(found), Keep::Never)
}

/// `DELETE /v1/account`.
pub async fn delete(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    door(&state)?.delete_account(&member).await?;
    send(&headers, &done(), Keep::Never)
}

/// `GET /v1/account/export`: everything held about the account, with no PIN hash.
pub async fn export(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let door = door(&state)?;
    door.require_step_up(&member)?;
    let profiles = door.profiles(&member).await?;
    let sessions = door.devices(&member).await?;
    let answer = AccountExport {
        account: wire(door.account(&member).await?),
        profiles: profiles.iter().map(view).collect(),
        devices: sessions
            .iter()
            .map(|row| session_view(row, member.session_id()))
            .collect(),
    };
    send(&headers, &answer, Keep::Never)
}

/// `POST /v1/account/password`.
pub async fn password(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<PasswordChange>,
) -> ApiResult<Response> {
    door(&state)?
        .change_password(&member, &asked.new_password)
        .await?;
    send(&headers, &done(), Keep::Never)
}

/// `POST /v1/account/email`.
pub async fn email(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<EmailChange>,
) -> ApiResult<Response> {
    door(&state)?.change_email(&member, &asked.email).await?;
    send(&headers, &done(), Keep::Never)
}

/// `POST /v1/account/phone`.
pub async fn phone(
    State(state): State<Shared>,
    Caller(member): Caller,
    headers: HeaderMap,
    Body(asked): Body<PhoneChange>,
) -> ApiResult<Response> {
    door(&state)?.change_phone(&member, &asked.phone).await?;
    send(&headers, &done(), Keep::Never)
}
