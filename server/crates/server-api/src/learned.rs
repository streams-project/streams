//! Writes that exist to learn from a person.
//!
//! Consent lives on the device session row and reaches a handler on the extractor
//! ([`server_accounts::ProfileScope::consent`]). The rule this module carries is that a write of
//! that kind is reached only through something that already took the person's answer: the log
//! itself is private to [`crate::state::AppState`], and [`Learned`] is the only way to it, so a
//! handler that wants to record playback cannot skip the test. Before this, one handler checked
//! and its neighbour did not.

use std::sync::Arc;

use contract::play::PlayEvent;
use server_accounts::{ProfileId, ProfileScope};
use server_play::PlayLog;

/// A writer of telemetry about one person, which cannot be built without their answer about
/// personal rows. `None` from [`Learned::of`] means nothing is written, and the call still
/// answers normally.
pub struct Learned<'a> {
    log: &'a Arc<dyn PlayLog>,
    profile: ProfileId,
}

impl<'a> Learned<'a> {
    /// The writer for this scope, or `None` when the person said no to personal rows, or this
    /// deployment keeps no log.
    pub(crate) fn of(log: Option<&'a Arc<dyn PlayLog>>, scope: &ProfileScope) -> Option<Self> {
        let log = log?;
        scope.consent().personal.then(|| Self {
            log,
            profile: scope.profile(),
        })
    }

    /// Keep what happened during one play.
    pub async fn play_events(&self, events: &[PlayEvent]) {
        self.log.events(self.profile, events).await;
    }
}
