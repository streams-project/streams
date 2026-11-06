//! Who is calling: device class, form and the capability card.

use alloc::string::String;
use alloc::vec::Vec;

use crate::play::AttemptKind;
use crate::screen::SectionKind;

words! {
    /// Client class. Minimum versions, start deadlines and reachability are kept per class.
    pub enum DeviceClass {
        Web,
        Phone,
        Tablet,
        Desktop,
        Tv,
        Box,
        Car,
    }
}

words! {
    /// The form a client draws in. A browser on a phone is class web, form phone.
    pub enum Form {
        Phone,
        Tablet,
        Desktop,
        Tv,
    }
}

words! {
    /// Where the core runs for this device.
    pub enum CoreMode {
        Local,
        Hosted,
    }
}

words! {
    /// Input a device has.
    pub enum InputKind {
        Touch,
        Pointer,
        Keyboard,
        Remote,
        Gamepad,
        Rotary,
    }
}

wire! {
    /// What a device can do. The server filters attempts and sections by this card.
    pub struct CapabilityCard {
        pub app: String,
        pub version: String,
        pub class: DeviceClass,
        pub form: Form,
        pub os: String,
        pub os_version: String,
        pub screen_width: u32,
        pub screen_height: u32,
        /// Section kinds this build draws. Others fall back to a plain row.
        pub section_kinds: Vec<SectionKind>,
        /// Attempt kinds this player opens. An embed appears only when listed here.
        pub attempt_kinds: Vec<AttemptKind>,
        /// Containers this player opens, such as ts, fmp4, mp4.
        pub containers: Vec<String>,
        /// Codecs this player opens, such as avc1, hvc1, mp4a.
        pub codecs: Vec<String>,
        pub max_height: u32,
        /// True when the player switches variants during play.
        pub adaptive: bool,
        /// Request headers the player can set on media, such as Referer.
        pub media_headers: Vec<String>,
        /// True when the driver reports video width or has_picture.
        pub reports_picture: bool,
        /// True when the player can load a side subtitle file.
        pub side_subtitles: bool,
        /// True when the device has a durable secret store.
        pub secret_store: bool,
        /// Bytes this device gives to kept commands and screens. 0 means none.
        pub storage_bytes: u64,
        pub input: Vec<InputKind>,
        pub languages: Vec<String>,
    }
}
