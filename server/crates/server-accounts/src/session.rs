//! The device session row: token hashes, active profile, step up and consent.

use crate::profile::{AccountId, ProfileId, SessionId};
use crate::token::{TokenHash, derive_next};

/// Consent for personal rows lives on the session, default off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Consent {
    pub personal: bool,
}

/// One signed in device. Stored hashed, so a read of the table signs nobody in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceSession {
    pub id: SessionId,
    pub account: AccountId,
    pub device_name: String,
    pub access_hash: TokenHash,
    pub access_expires_ms: u64,
    /// Hash of the refresh token the device holds.
    pub refresh_current: TokenHash,
    /// Hash of the refresh token the server has offered.
    pub refresh_next: TokenHash,
    /// Salt of the next token, which follows from the current one.
    pub refresh_salt: String,
    /// When current was first presented. It stops being reusable after the reuse window.
    pub refresh_first_used_ms: Option<u64>,
    /// Written only by entering a profile.
    pub active_profile: Option<ProfileId>,
    pub entered_at_ms: Option<u64>,
    pub step_up_at_ms: Option<u64>,
    pub step_up_method: Option<String>,
    pub consent: Consent,
    pub last_seen_ms: u64,
    pub created_ms: u64,
}

/// Tokens handed to a device once.
#[derive(Clone, PartialEq, Eq)]
pub struct IssuedTokens {
    pub access: String,
    pub refresh: String,
    pub expires_in: u32,
}

impl std::fmt::Debug for IssuedTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IssuedTokens(hidden)")
    }
}

/// What a presented refresh token means for its session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshOutcome {
    /// The device holds current: offer next, the same one each time.
    OfferNext,
    /// The device used next: it becomes current, and the old current dies.
    Promote,
    /// Dead, or out of the reuse window: the session ends.
    End,
}

impl DeviceSession {
    /// How a presented token relates to this row. Anything that is neither current
    /// nor next is an older token of this session, which ends it.
    #[must_use]
    pub fn classify_refresh(
        &self,
        token: &TokenHash,
        now_ms: u64,
        reuse_ms: u64,
    ) -> RefreshOutcome {
        if *token == self.refresh_next {
            return RefreshOutcome::Promote;
        }
        if *token == self.refresh_current {
            let spent = self
                .refresh_first_used_ms
                .is_some_and(|first| now_ms.saturating_sub(first) > reuse_ms);
            return if spent {
                RefreshOutcome::End
            } else {
                RefreshOutcome::OfferNext
            };
        }
        RefreshOutcome::End
    }

    /// The next token for a presented current token. It carries the session id, so any older
    /// token of this session still names it and ends it when shown.
    #[must_use]
    pub(crate) fn next_for(&self, current: &str) -> String {
        format!("{}.{}", self.id, derive_next(current, &self.refresh_salt))
    }
}
