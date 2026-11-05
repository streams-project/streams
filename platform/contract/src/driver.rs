//! The player driver contract, about ten commands and six events.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

words! {
    /// Failure classes a driver reports. The core picks the next attempt from these.
    pub enum ErrorClass {
        Network,
        Manifest,
        Segment,
        Decode,
        Key,
        Format,
        Refused,
    }
}

wire! {
    /// A command to the player. Attempts carry their headers here.
    #[serde(tag = "op", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum DriverCommand {
        Open {
            url: String,
            headers: BTreeMap<String, String>,
            start_at_ms: u64,
            generation: u32,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            max_height: Option<u32>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            max_bitrate: Option<u32>,
        },
        Play,
        Pause,
        Seek { to_ms: u64 },
        Rate { percent: u32 },
        Volume { percent: u32 },
        SelectAudio { track: String },
        SelectText {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            track: Option<String>,
        },
        Close,
        #[serde(other)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        Unknown,
    }

    /// One audio or text track, with the tag the manifest gave it.
    pub struct Track {
        pub id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub lang: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub label: Option<String>,
    }

    /// What the driver reports. No driver decides failover or no picture by itself.
    #[serde(tag = "kind", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum DriverEvent {
        FirstFrame,
        Position {
            time_ms: u64,
            /// 0 or absent when not known.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            duration_ms: Option<u64>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            video_width: Option<u32>,
            /// Absent means unknown, and the no picture rule stays off.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            has_picture: Option<bool>,
        },
        Stalled,
        Ended,
        Tracks {
            audio: Vec<Track>,
            text: Vec<Track>,
        },
        Error { class: ErrorClass },
        #[serde(other)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        Unknown,
    }
}
