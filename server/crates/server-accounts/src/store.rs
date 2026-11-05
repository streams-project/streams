//! Everything the accounts need from a database. Postgres in production, memory in tests.

use async_trait::async_trait;
use thiserror::Error;

use crate::pin::{PinCheck, PinOutcome};
use crate::profile::{
    AccountId, NewAccount, Profile, ProfileId, ProfileLimitsRow, ProfilePatch, SessionId,
};
use crate::session::{Consent, DeviceSession};
use crate::token::TokenHash;

/// Why a store call failed.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreError {
    #[error("the store did not answer")]
    Unavailable,
    #[error("no such row")]
    NotFound,
    #[error("the profile limit is reached")]
    LimitReached,
    #[error("a profile of this account has that name")]
    NameTaken,
    #[error("that handle is taken")]
    HandleTaken,
    #[error("a rule refused the change: {0}")]
    Rule(&'static str),
}

/// One device sign in waiting for approval. Only the hash of the device code is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceCodeRow {
    pub device_hash: TokenHash,
    pub user_code: String,
    pub device_name: String,
    /// Set when a signed in person approved it.
    pub account: Option<AccountId>,
    pub denied: bool,
    pub last_poll_ms: Option<u64>,
    pub expires_ms: u64,
}

/// Names of a person.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountNames {
    pub first_name: String,
    pub last_name: String,
}

/// A store result.
pub type StoreResult<T> = Result<T, StoreError>;

/// The rows behind accounts. Each method is one transaction, so rules that need a lock (profile
/// limit, PIN counter) stay safe under parallel calls.
#[async_trait]
pub trait AccountStore: Send + Sync {
    /// Details, main profile and Kids profile of a new account, in one write.
    async fn create_account(&self, account: &NewAccount, profiles: &[Profile]) -> StoreResult<()>;
    /// Write a new session.
    async fn insert_session(&self, session: &DeviceSession) -> StoreResult<()>;
    /// The session holding this access token.
    async fn session_by_access(&self, hash: &TokenHash) -> StoreResult<Option<DeviceSession>>;
    /// One session by id.
    async fn session(&self, id: SessionId) -> StoreResult<Option<DeviceSession>>;
    /// Write only the token fields and last seen of a session. Each field group has its own write,
    /// so a refresh racing a profile change or a consent change loses neither.
    async fn update_tokens(&self, session: &DeviceSession) -> StoreResult<()>;
    /// Write the active profile of a session. Nothing else writes it.
    async fn set_active_profile(
        &self,
        session: SessionId,
        profile: ProfileId,
        at_ms: u64,
    ) -> StoreResult<()>;
    /// Record a step up on a session.
    async fn set_step_up(&self, session: SessionId, at_ms: u64, method: &str) -> StoreResult<()>;
    /// Remove a session of this account.
    async fn delete_session(&self, account: AccountId, id: SessionId) -> StoreResult<()>;
    /// Remove every session of this account.
    async fn delete_sessions(&self, account: AccountId) -> StoreResult<()>;
    /// Sessions of an account, newest use first.
    async fn sessions_of(&self, account: AccountId) -> StoreResult<Vec<DeviceSession>>;
    /// One profile of an account.
    async fn profile(&self, account: AccountId, id: ProfileId) -> StoreResult<Option<Profile>>;
    /// Profiles of an account in sort order.
    async fn profiles_of(&self, account: AccountId) -> StoreResult<Vec<Profile>>;
    /// Add a profile under the account lock, refusing past `max`.
    async fn insert_profile(&self, profile: &Profile, max: u32) -> StoreResult<()>;
    /// Change names, picture or kind.
    async fn update_profile(
        &self,
        account: AccountId,
        id: ProfileId,
        patch: &ProfilePatch,
    ) -> StoreResult<Profile>;
    /// Delete a profile. Clears the active profile of every session that held it.
    async fn delete_profile(&self, account: AccountId, id: ProfileId) -> StoreResult<()>;
    /// Move the main flag in one transaction.
    async fn set_main_profile(&self, account: AccountId, id: ProfileId) -> StoreResult<()>;
    /// Run a PIN check under the row lock. `decide` is `pin::decide`; the store writes its answer.
    async fn pin_attempt(&self, profile: ProfileId, check: PinCheck<'_>)
    -> StoreResult<PinOutcome>;
    /// Store a PIN hash, or clear it with `None`. Keeps `has_pin` in step.
    async fn set_pin(&self, profile: ProfileId, hash: Option<String>) -> StoreResult<()>;
    /// Write age limit and blocked names. Only the step up command reaches this.
    async fn set_limits(
        &self,
        account: AccountId,
        profile: ProfileId,
        limits: &ProfileLimitsRow,
    ) -> StoreResult<()>;
    /// Set consent on one session.
    async fn set_consent(&self, session: SessionId, consent: Consent) -> StoreResult<()>;
    /// Add a row to the audit log.
    async fn log_event(&self, account: AccountId, event: &str) -> StoreResult<()>;
    /// The newest audit rows of an account.
    async fn events_of(&self, account: AccountId, limit: usize) -> StoreResult<Vec<String>>;
    /// The names of an account.
    async fn names(&self, account: AccountId) -> StoreResult<AccountNames>;
    /// Change the names of an account, and the main profile's name when `main` says so.
    async fn set_names(
        &self,
        account: AccountId,
        names: &AccountNames,
        main: Option<(ProfileId, String)>,
    ) -> StoreResult<()>;
    /// Remove the account's rows. Deleting the auth row cascades in Postgres.
    async fn delete_account(&self, account: AccountId) -> StoreResult<()>;
    /// Move a profile to another account, clearing it from sessions that held it.
    async fn move_profile(
        &self,
        from: AccountId,
        profile: ProfileId,
        to: AccountId,
        max: u32,
    ) -> StoreResult<()>;
    /// Keep a waiting device sign in.
    async fn insert_device_code(&self, row: &DeviceCodeRow) -> StoreResult<()>;
    /// A waiting sign in by the hash of its device code.
    async fn device_code(&self, hash: &TokenHash) -> StoreResult<Option<DeviceCodeRow>>;
    /// A waiting sign in by the code a person types.
    async fn device_code_by_user(&self, user_code: &str) -> StoreResult<Option<DeviceCodeRow>>;
    /// Write the answer and last poll of a waiting sign in.
    async fn update_device_code(&self, row: &DeviceCodeRow) -> StoreResult<()>;
    /// Remove a waiting sign in.
    async fn delete_device_code(&self, hash: &TokenHash) -> StoreResult<()>;
    /// The device card of a session, as sent.
    async fn card(&self, session: SessionId) -> StoreResult<Option<serde_json::Value>>;
    /// Save a device card as opaque JSON, returning true when the hash is new.
    async fn save_card(
        &self,
        session: SessionId,
        card_hash: &str,
        card: Option<&serde_json::Value>,
    ) -> StoreResult<bool>;
}
