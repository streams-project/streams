//! What the door can answer. 401, 429 and 503 stay apart.

use thiserror::Error;

use crate::store::StoreError;

/// Why a call was stopped.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum DoorError {
    /// No token, or one this server does not know.
    #[error("not signed in")]
    Unauthorized,
    /// The refresh token was refused for good. This is the one end of a session a client obeys.
    #[error("session ended")]
    SessionEnded,
    /// A content route with no active profile on the session.
    #[error("choose a profile")]
    ProfileRequired,
    /// The profile has a PIN and none was sent.
    #[error("PIN required")]
    PinRequired,
    /// The PIN was wrong. The counter moved.
    #[error("wrong PIN")]
    PinWrong,
    /// Too many wrong PINs.
    #[error("PIN locked")]
    PinLocked { retry_after_s: u32 },
    /// The action needs a recent password or code.
    #[error("step up required")]
    StepUpRequired,
    /// A rule refused the change.
    #[error("refused: {0}")]
    Refused(&'static str),
    /// The value is not acceptable.
    #[error("bad value: {0}")]
    BadValue(&'static str),
    /// No such row for this account.
    #[error("not found")]
    NotFound,
    /// Too many calls from this caller.
    #[error("rate limited")]
    RateLimited { retry_after_s: u32 },
    /// The store could not answer. Never read as a sign out.
    #[error("try again")]
    Unavailable { retry_after_s: u32 },
}

impl From<StoreError> for DoorError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Unavailable => Self::Unavailable { retry_after_s: 5 },
            StoreError::NotFound => Self::NotFound,
            StoreError::LimitReached => Self::Refused("profile_limit"),
            StoreError::NameTaken => Self::Refused("profile_name_taken"),
            StoreError::HandleTaken => Self::Refused("handle_taken"),
            StoreError::Rule(rule) => Self::Refused(rule),
        }
    }
}
