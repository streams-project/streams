//! Commands a device sends and changes it reads back.

use alloc::string::String;
use alloc::vec::Vec;

use crate::screen::Cursor;
use crate::title::{EpisodeRef, Rating, TitleKey};

words! {
    /// Answer per op.
    pub enum Outcome {
        Applied,
        Superseded,
        Rejected,
    }
}

words! {
    /// Signals a viewer gives.
    pub enum FeedbackEvent {
        Click,
        Start,
        Complete,
        Like,
        Skip,
    }
}

words! {
    /// Settings a queued command may change. Age rating and blocked titles are absent on
    /// purpose: they need a step up, so a queued write can never touch them.
    pub enum SettingKey {
        AutoplayNext,
        AutoplayPreviews,
        AudioLanguage,
        SubtitleLanguage,
        SubtitleStyle,
        UiLanguage,
        Handle,
        Gender,
    }
}

wire! {
    /// One change to a profile's facts.
    #[serde(tag = "command", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum Command {
        /// Add to or remove from My List. Leaves the rating alone.
        List { title: TitleKey, in_list: bool },
        /// Set or clear a rating. Leaves the list entry alone.
        Rate {
            title: TitleKey,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            rating: Option<Rating>,
        },
        /// Hide a title from history. For a series this covers every episode.
        Hide { title: TitleKey },
        /// Clear all history of this profile, as one operation.
        ClearHistory,
        /// Save a position, in whole seconds.
        Position {
            title: TitleKey,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            episode: Option<EpisodeRef>,
            position_s: u32,
            duration_s: u32,
        },
        /// Change one profile setting. Booleans as true or false, style as JSON text.
        Setting { key: SettingKey, value: String },
        /// A viewer signal. Discarded by the server when consent is off.
        Feedback { title: TitleKey, event: FeedbackEvent },
        /// A trailer failed to play for this title.
        TrailerRefused { title: TitleKey },
        #[serde(other)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        Unknown,
    }

    /// One queued op. A missing `age_ms` means age unknown, and the server orders by arrival.
    pub struct CommandOp {
        pub op_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub age_ms: Option<u64>,
        #[serde(flatten)]
        pub command: Command,
    }

    /// `POST /v1/commands`. Idempotent by op id.
    pub struct CommandBatch {
        pub ops: Vec<CommandOp>,
    }

    /// Answer for one op.
    pub struct OpAnswer {
        pub op_id: String,
        pub outcome: Outcome,
    }

    /// Answer for a batch, one entry per op.
    pub struct CommandAnswer {
        pub answers: Vec<OpAnswer>,
    }

    /// List state of a title, versioned apart from its rating.
    pub struct ListState {
        pub in_list: bool,
        pub version: u64,
        pub changed_at_ms: u64,
    }

    /// Rating state of a title, versioned apart from its list state.
    pub struct RatingState {
        /// Absent when no rating is set.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rating: Option<Rating>,
        pub version: u64,
        pub changed_at_ms: u64,
    }

    /// A position, key by title and episode.
    pub struct ProgressState {
        pub title: TitleKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub episode: Option<EpisodeRef>,
        pub position_s: u32,
        pub duration_s: u32,
        pub completed: bool,
        pub hidden: bool,
        pub version: u64,
        pub changed_at_ms: u64,
    }

    /// One change a device reads.
    #[serde(tag = "change", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum Change {
        Title {
            title: TitleKey,
            list: ListState,
            rating: RatingState,
        },
        Progress { state: ProgressState },
        /// Hidden as a whole, every episode of a series included.
        Hidden { title: TitleKey },
        HistoryCleared { changed_at_ms: u64 },
        Setting { key: SettingKey, value: String },
        #[serde(other)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        Unknown,
    }

    /// `GET /v1/changes`.
    pub struct Changes {
        pub changes: Vec<Change>,
        pub cursor: Cursor,
        pub has_more: bool,
    }
}
