//! `/v1/hello`, `/v1/ready`, strings and values that arrive at run time.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::client::DeviceClass;
use crate::version::VersionRange;

const fn yes() -> bool {
    true
}

const fn ltr() -> TextDirection {
    TextDirection::Ltr
}

words! {
    /// State of a part of the service.
    pub enum ServiceState {
        Ready,
        Degraded,
        Down,
    }
}

words! {
    /// Which way a language's lines run. The apps lay out, mirror and move focus by it; the player's
    /// timeline and the brand's art never turn.
    pub enum TextDirection {
        Ltr,
        Rtl,
    }
}

wire! {
    /// Feature values. With no answer from the server, accounts are required.
    pub struct Features {
        #[serde(default = "yes")]
        pub accounts_required: bool,
        #[serde(default)]
        pub games: bool,
        #[serde(default)]
        pub clips: bool,
        #[serde(default)]
        pub original_audio: bool,
        /// Tier for visitors. Present only when accounts are not required.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub visitor_tier: Option<String>,
    }

    /// Limits served from one place, so no client holds a copy.
    pub struct Limits {
        pub profiles_max: u32,
        pub profile_name_max: u32,
        pub pin_digits: u32,
        pub password_min: u32,
        pub step_up_s: u32,
        pub phone_digits_min: u32,
        pub phone_digits_max: u32,
        pub blocked_title_max: u32,
        pub blocked_titles_max: u32,
        pub name_max: u32,
        pub handle_min: u32,
        pub handle_max: u32,
        pub code_digits: u32,
        pub code_valid_s: u32,
        pub command_batch_max: u32,
    }

    /// Lowest app version a class may run.
    pub struct ClientFloor {
        pub class: DeviceClass,
        pub version: String,
    }

    /// One value per client class.
    pub struct ClassValue {
        pub class: DeviceClass,
        pub value: u32,
    }

    /// A staged rollout. The bucket comes from the household, so a home moves together.
    pub struct Rollout {
        pub name: String,
        pub percent: u32,
    }

    /// One row of the moved address table.
    pub struct MovedAddress {
        pub from: String,
        pub to: String,
    }

    /// Name and contact, filled into text at render time.
    pub struct BrandValues {
        pub name: String,
        pub support_email: String,
    }

    /// First answer a client reads.
    pub struct Hello {
        /// Wire dates this server answers.
        pub contract: VersionRange,
        pub client_min: Vec<ClientFloor>,
        pub features: Features,
        pub limits: Limits,
        /// False stops playback everywhere with a notice.
        pub playback: bool,
        /// Builds refused by name.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub killed_builds: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub rollouts: Vec<Rollout>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub moved: Vec<MovedAddress>,
        pub start_deadline_ms: Vec<ClassValue>,
        pub brand: BrandValues,
        /// Seconds this answer stays good.
        pub ttl_s: u32,
    }

    /// `/v1/ready`, for outside checks.
    pub struct Ready {
        pub build: String,
        pub data_hash: String,
        pub data_age_s: u64,
        pub index: ServiceState,
        pub finder: ServiceState,
    }

    /// `/v1/strings/{lang}`. Text holds the placeholders below.
    pub struct Strings {
        pub lang: String,
        /// Which way this language runs. Absent from older servers, which only spoke left to right.
        #[serde(default = "ltr")]
        pub dir: TextDirection,
        pub texts: BTreeMap<String, String>,
    }

    /// A language this server has text for, named in itself.
    pub struct Language {
        pub code: String,
        pub name: String,
        #[serde(default = "ltr")]
        pub dir: TextDirection,
    }

    pub struct LanguageList {
        pub languages: Vec<Language>,
    }
}

/// Placeholder for the product name in strings.
pub const PLACEHOLDER_BRAND: &str = "{brand}";

/// Placeholder for the support address in strings.
pub const PLACEHOLDER_SUPPORT_EMAIL: &str = "{support_email}";

impl BrandValues {
    /// Fill the placeholders of one text.
    #[must_use]
    pub fn fill(&self, text: &str) -> String {
        text.replace(PLACEHOLDER_BRAND, &self.name)
            .replace(PLACEHOLDER_SUPPORT_EMAIL, &self.support_email)
    }
}

impl Default for Features {
    fn default() -> Self {
        Self {
            accounts_required: true,
            games: false,
            clips: false,
            original_audio: false,
            visitor_tier: None,
        }
    }
}
