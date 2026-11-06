//! What the library needs from a database. Rows are read and written inside one transaction per
//! batch, with the profile's counter row locked, so a profile has one writer.

use async_trait::async_trait;
use contract::command::{Change, Outcome, SettingKey};
use contract::{EpisodeRef, TitleKey};
use server_accounts::ProfileId;
use thiserror::Error;

use crate::model::{EpisodeProgress, FeedRow, HiddenTitle, TitleState};
use crate::reads::SettingsView;
use crate::taste::Taste;

/// Why a library call failed.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum LibraryError {
    /// The store did not answer. The op is not recorded, so a retry applies it.
    #[error("the store did not answer")]
    Unavailable,
    /// The batch is larger than a batch may be.
    #[error("too many ops in one batch")]
    BatchTooLarge,
}

/// Result of a library call.
pub type Row<T> = Result<T, LibraryError>;

/// One open transaction on one profile.
#[async_trait]
pub trait LibraryTx: Send {
    /// The next change number of this profile.
    async fn next_seq(&mut self) -> Row<u64>;
    /// The answer given to an op id already seen.
    async fn seen_op(&mut self, op_id: &str) -> Row<Option<Outcome>>;
    /// Keep the answer of an op.
    async fn record_op(&mut self, op_id: &str, outcome: Outcome, now_ms: u64) -> Row<()>;
    /// The row of a title.
    async fn title(&mut self, title: TitleKey) -> Row<Option<TitleState>>;
    /// Write the row of a title.
    async fn put_title(&mut self, row: &TitleState) -> Row<()>;
    /// The position of a title or episode.
    async fn progress(
        &mut self,
        title: TitleKey,
        episode: Option<EpisodeRef>,
    ) -> Row<Option<EpisodeProgress>>;
    /// Write a position.
    async fn put_progress(&mut self, row: &EpisodeProgress) -> Row<()>;
    /// The hidden fact of a title.
    async fn hidden(&mut self, title: TitleKey) -> Row<Option<HiddenTitle>>;
    /// Hide a title.
    async fn put_hidden(&mut self, row: &HiddenTitle) -> Row<()>;
    /// Remove a hidden fact.
    async fn drop_hidden(&mut self, title: TitleKey) -> Row<()>;
    /// Remove all positions and hidden facts changed at or before this time, in one operation.
    async fn clear_history(&mut self, at_ms: u64, seq: u64) -> Row<()>;
    /// When history was last cleared.
    async fn cleared_at(&mut self) -> Row<Option<u64>>;
    /// Change one setting. Answers false when a newer write already holds the field.
    async fn put_setting(
        &mut self,
        key: SettingKey,
        value: &str,
        at_ms: u64,
        seq: u64,
    ) -> Row<bool>;
    /// The taste row.
    async fn taste(&mut self) -> Row<Taste>;
    /// Write the taste row.
    async fn put_taste(&mut self, taste: &Taste) -> Row<()>;
    /// Keep a viewing signal.
    async fn add_feed(&mut self, row: FeedRow) -> Row<()>;
    /// Keep a refused trailer.
    async fn add_refused_trailer(&mut self, title: TitleKey, at_ms: u64) -> Row<()>;
    /// End the transaction.
    async fn commit(self: Box<Self>) -> Row<()>;
}

/// Reads of a profile's library. These answer keys; titles leave only through the rules.
#[async_trait]
pub trait LibraryRead: Send + Sync {
    /// Titles on the list, newest first.
    async fn list(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>>;
    /// Titles rated up or love, and titles rated down, newest first.
    async fn rated(
        &self,
        profile: ProfileId,
        limit: usize,
    ) -> Row<Vec<(TitleKey, contract::Rating)>>;
    /// The newest positions, hidden titles left out.
    async fn recent(&self, profile: ProfileId, limit: usize) -> Row<Vec<EpisodeProgress>>;
    /// The saved position of a title or episode.
    async fn position(
        &self,
        profile: ProfileId,
        title: TitleKey,
        episode: Option<EpisodeRef>,
    ) -> Row<Option<EpisodeProgress>>;
    /// Taste of a profile.
    async fn taste_of(&self, profile: ProfileId) -> Row<Taste>;
    /// Titles this profile started or finished, for removing from rows.
    async fn seen(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>>;
    /// Settings of a profile.
    async fn settings(&self, profile: ProfileId) -> Row<SettingsView>;
    /// Changes after a change number, oldest first, at most `limit`. The cursor is the last seq.
    async fn changes_after(
        &self,
        profile: ProfileId,
        after: u64,
        limit: usize,
    ) -> Row<(Vec<Change>, u64)>;
    /// Remove every taste row and viewing signal of this profile, on consent withdrawal.
    async fn forget_taste(&self, profile: ProfileId) -> Row<()>;
    /// Delete old viewing signals.
    async fn prune_feed(&self, before_ms: u64) -> Row<u64>;
}

/// A whole library store.
#[async_trait]
pub trait LibraryStore: LibraryRead {
    /// Open the writer of one profile. A second writer waits.
    async fn begin(&self, profile: ProfileId) -> Row<Box<dyn LibraryTx>>;
}
