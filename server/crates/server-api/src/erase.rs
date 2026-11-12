//! Deletion, in one place, walking the one list of what we hold.
//!
//! `server_accounts::holdings::HELD` says for every table how deletion reaches it. Three answers
//! exist and all three are answered here:
//!
//! * `Removal::Cascade` — the foreign keys carry the row away with `auth.users`. The test beside
//!   the list reads the migration files and fails when a claim of this kind is not true of the
//!   schema, which is how `play_events` was found keeping a nulled row for thirty days after the
//!   account was gone.
//! * `Removal::Erase(by)` — no key reaches it, so the named function removes it. Today that is
//!   `auth.login_codes`, removed inside `BuiltInAuth::delete`.
//! * `Removal::Keeps` — nothing about a person, so deletion leaves it.
//!
//! The library store is called for every profile that goes. On Postgres the keys have already
//! taken those rows and the call changes nothing; it is here because the memory store is the one
//! CI runs and nothing in it modelled the cascade, so "deleting a profile removes its library"
//! was a claim proved against nothing.

use server_accounts::{Member, Profile, ProfileId};

use crate::deadline;
use crate::error::{ApiError, ApiResult};
use crate::state::Shared;

fn door(state: &Shared) -> ApiResult<&server_accounts::Door> {
    state.door.as_ref().ok_or_else(|| ApiError::off("accounts"))
}

/// Remove the account and everything it owns. The profiles are read first, because the rows that
/// name them are gone by the time the account is.
///
/// # Errors
/// The door's refusal, which for this route is a missing step up.
pub async fn account(state: &Shared, member: &Member) -> ApiResult<()> {
    let profiles: Vec<Profile> = deadline::store(door(state)?.profiles(member))
        .await
        .unwrap_or_default();
    deadline::store(door(state)?.delete_account(member)).await?;
    for row in &profiles {
        library_of(state, row.id).await;
    }
    Ok(())
}

/// Remove one profile of the account and everything that profile owns.
///
/// # Errors
/// The door's refusal: another account's profile is not found, and the main or last profile is
/// refused.
pub async fn profile(state: &Shared, member: &Member, id: ProfileId) -> ApiResult<()> {
    deadline::store(door(state)?.delete_profile(member, id)).await?;
    library_of(state, id).await;
    Ok(())
}

/// The library rows of one profile. The account is already gone when this runs, so a store that
/// does not answer leaves rows nothing can reach and nobody can sign in to: that is worth a line
/// in the log and not a failure handed to a person who has just been told their account is gone.
async fn library_of(state: &Shared, profile: ProfileId) {
    if let Some(library) = &state.library
        && let Err(error) = deadline::store(library.store().delete_profile(profile)).await
    {
        tracing::warn!(
            "the library rows of a deleted profile are still there: {}",
            error.label
        );
    }
}
