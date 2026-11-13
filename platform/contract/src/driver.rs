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

words! {
    /// A key system a player can open a protected stream with. The word is the contract's own;
    /// each adapter maps it to the name its platform uses.
    pub enum KeySystem {
        Widevine,
        PlayReady,
        FairPlay,
        ClearKey,
    }
}

wire! {
    /// Where a protected stream's keys come from.
    ///
    /// A path, never an address, as an http effect is. The host holds the base address, so the
    /// licence server is the same server the core already talks to, and there is nowhere in this
    /// type to name another one.
    pub struct KeyDelivery {
        pub system: KeySystem,
        /// A path, never an address.
        pub path: String,
        /// Headers the player must send with the licence request.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub headers: BTreeMap<String, String>,
        /// Where the key system's server certificate is read, for the systems that ask for one
        /// before the first request. A path, never an address.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub certificate_path: Option<String>,
    }

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
            /// Absent for a stream that needs no key, which is every stream today.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            key: Option<KeyDelivery>,
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
        /// True for the track playing now.
        #[serde(default, skip_serializing_if = "core::ops::Not::not")]
        pub selected: bool,
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
