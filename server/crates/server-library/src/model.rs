//! The rows of a profile's library and the limits on them.

use contract::command::FeedbackEvent;
use contract::{EpisodeRef, Rating, TitleKey};

/// Bounds carried over from the old sync rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Most ops in one batch. A larger batch is refused whole.
    pub batch_max: usize,
    /// Most rows in one page of changes.
    pub changes_page: usize,
    /// Continue Watching shows positions above this many seconds.
    pub continue_min_s: u32,
    /// Longest op id kept for idempotency.
    pub op_id_max: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            batch_max: 500,
            changes_page: 2000,
            continue_min_s: 15,
            op_id_max: 64,
        }
    }
}

/// List and rating of one title, each with its own version and change time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleState {
    pub title: TitleKey,
    pub in_list: bool,
    pub list_version: u64,
    pub list_changed_ms: u64,
    pub added_ms: Option<u64>,
    pub rating: Option<Rating>,
    pub rating_version: u64,
    pub rating_changed_ms: u64,
    pub seq: u64,
}

impl TitleState {
    /// A row with nothing set.
    #[must_use]
    pub const fn empty(title: TitleKey) -> Self {
        Self {
            title,
            in_list: false,
            list_version: 0,
            list_changed_ms: 0,
            added_ms: None,
            rating: None,
            rating_version: 0,
            rating_changed_ms: 0,
            seq: 0,
        }
    }
}

/// One saved position. A film has season and episode 0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpisodeProgress {
    pub title: TitleKey,
    pub episode: Option<EpisodeRef>,
    pub position_s: u32,
    pub duration_s: u32,
    pub completed: bool,
    pub version: u64,
    pub changed_ms: u64,
    pub seq: u64,
}

/// A title hidden from history. For a series this covers every episode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HiddenTitle {
    pub title: TitleKey,
    pub hidden_ms: u64,
    pub seq: u64,
}

/// One entry of Continue Watching: the newest record of a title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Continue {
    pub title: TitleKey,
    pub episode: Option<EpisodeRef>,
    pub percent: u32,
    pub changed_ms: u64,
}

/// The newest place a profile reached in a title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reached {
    pub episode: Option<EpisodeRef>,
    pub position_s: u32,
    /// Marked complete, or within the last moments of the video.
    pub finished: bool,
}

/// One viewing signal kept for 180 days.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeedRow {
    pub title: TitleKey,
    pub event: FeedbackEvent,
    pub at_ms: u64,
}

/// When an op happened by the server's clock: age replaces any device clock, and an op of unknown
/// age counts as arriving now.
#[must_use]
pub fn age_to_time(now_ms: u64, age_ms: Option<u64>) -> u64 {
    age_ms.map_or(now_ms, |age| now_ms.saturating_sub(age))
}
