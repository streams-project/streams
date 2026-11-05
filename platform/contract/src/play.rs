//! Play: attempts, subtitles, skip marks and play reports.
//!
//! The server answers addresses only. Video goes straight from the host to the viewer.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::client::CoreMode;
use crate::driver::ErrorClass;
use crate::title::{EpisodeRef, TitleKey};

words! {
    /// How an attempt is played.
    pub enum AttemptKind {
        Hls,
        File,
        Embed,
    }
}

words! {
    /// Windows of time inside a video.
    pub enum WindowKind {
        Recap,
        Intro,
        Credits,
        AgeCard,
        Cue,
    }
}

words! {
    /// What a play report says happened.
    pub enum PlayEventName {
        FirstFrame,
        Rebuffer,
        Failure,
        Ended,
    }
}

wire! {
    /// Ask to play a title.
    pub struct PlayRequest {
        pub title: TitleKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub episode: Option<EpisodeRef>,
    }

    /// One way to play. Attempts arrive in the order to try, ranked once on the server.
    /// `headers` is required, even when empty, and no provider name is carried.
    pub struct Attempt {
        pub id: String,
        pub url: String,
        pub kind: AttemptKind,
        /// Opaque group. After a failure the core skips other attempts of this group.
        pub group: String,
        /// Headers the player must send, such as Referer, Origin, User-Agent.
        pub headers: BTreeMap<String, String>,
        /// True when the finder fetched and read this address.
        pub verified: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub container: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub codecs: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub height: Option<u32>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub audio: Vec<String>,
        /// Wall time, ms since 1970, after which the address stops working.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub expires_at_ms: Option<u64>,
    }

    /// Age card shown once at the start of play.
    pub struct AgeCard {
        pub label: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub reasons: Vec<String>,
    }

    /// Answer to a play request.
    pub struct PlayAnswer {
        pub play_id: String,
        pub attempts: Vec<Attempt>,
        /// Where to start.
        pub start_at_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub age_card: Option<AgeCard>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub audio_default: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub subtitle_default: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub next_episode: Option<EpisodeRef>,
        /// Attempts removed by the capability card.
        pub dropped: u32,
        /// True when more attempts may arrive through `/v1/play/{id}/more`.
        pub more: bool,
    }

    /// Late attempts.
    pub struct MoreAttempts {
        pub attempts: Vec<Attempt>,
        pub more: bool,
    }

    /// One subtitle file of a language, in the order to try.
    pub struct SubtitleFile {
        pub id: String,
        #[serde(default)]
        pub hearing_impaired: bool,
    }

    /// One language.
    pub struct SubtitleLanguage {
        pub lang: String,
        pub label: String,
        pub files: Vec<SubtitleFile>,
    }

    /// `/v1/subtitles`.
    pub struct SubtitleList {
        pub languages: Vec<SubtitleLanguage>,
    }

    /// One cue.
    pub struct Cue {
        pub start_ms: u64,
        pub end_ms: u64,
        pub text: String,
    }

    /// `/v1/subtitles/{file}`. The core checks `last_end_ms` against the driver's duration.
    pub struct SubtitleBody {
        pub cues: Vec<Cue>,
        pub last_end_ms: u64,
    }

    /// A span of a video the host compares to player time. Nothing ticks across the boundary.
    pub struct TimeWindow {
        pub kind: WindowKind,
        pub start_ms: u64,
        pub end_ms: u64,
        /// Where a skip button moves to.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub seek_to_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
    }

    /// Ask for skip marks, after first frame, series only.
    pub struct MarksRequest {
        pub attempt: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub file: Option<String>,
        pub duration_ms: u64,
        #[serde(default)]
        pub delay_ms: i32,
    }

    /// Skip marks.
    pub struct Marks {
        pub windows: Vec<TimeWindow>,
    }

    /// One thing that happened during play.
    pub struct PlayEvent {
        pub name: PlayEventName,
        pub at_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub attempt: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub first_frame_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub failure: Option<ErrorClass>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub host: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rung: Option<CoreMode>,
    }

    /// `/v1/play/{id}/events`.
    pub struct PlayEvents {
        pub events: Vec<PlayEvent>,
    }

    /// `/v1/reports`.
    pub struct Report {
        pub code: String,
        pub build: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub detail: Option<String>,
    }
}
