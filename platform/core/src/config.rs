//! What the core takes at start and what it holds from `/v1/hello`. The host holds the base
//! address, so no address enters here. Limits and feature values come from the server and the
//! core keeps no copy of its own.

use alloc::string::String;
use contract::client::{CapabilityCard, DeviceClass, Form};
use contract::core_msg::CoreConfig;
use contract::hello::{Hello, Limits, MovedAddress};
use contract::version::{VersionFit, WireDate};

/// Start deadline used only until hello names one for this class. Without a value here a dead
/// source would hang, which is H39. The figure is assumed: no server has measured one yet.
pub const DEFAULT_START_DEADLINE_MS: u32 = 15_000;

/// Whether this build may go on, from what hello said.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Standing {
    Fine,
    Upgrade,
}

/// The start message and the last hello.
pub struct Config {
    start: CoreConfig,
    hello: Option<Hello>,
    unreachable: bool,
}

impl Config {
    #[must_use]
    pub const fn new(start: CoreConfig) -> Self {
        Self {
            start,
            hello: None,
            unreachable: false,
        }
    }

    #[must_use]
    pub const fn class(&self) -> DeviceClass {
        self.start.class
    }

    #[must_use]
    pub const fn form(&self) -> Form {
        self.start.form
    }

    #[must_use]
    pub fn app(&self) -> &str {
        &self.start.app
    }

    #[must_use]
    pub fn version(&self) -> &str {
        &self.start.version
    }

    #[must_use]
    pub fn os(&self) -> &str {
        &self.start.os
    }

    #[must_use]
    pub fn seed(&self) -> &str {
        &self.start.seed
    }

    #[must_use]
    pub const fn card(&self) -> Option<&CapabilityCard> {
        self.start.card.as_ref()
    }

    /// Hold a hello answer and say whether this build may go on.
    pub fn take_hello(&mut self, hello: Hello) -> Standing {
        let too_old = matches!(hello.contract.fit(&WireDate::current()), VersionFit::TooOld);
        let below = hello
            .client_min
            .iter()
            .filter(|floor| floor.class == self.start.class)
            .any(|floor| version_below(&self.start.version, &floor.version));
        let killed = hello.killed_builds.contains(&self.start.version);
        self.hello = Some(hello);
        self.unreachable = false;
        if too_old || below || killed {
            Standing::Upgrade
        } else {
            Standing::Fine
        }
    }

    /// The server gave no answer. A known answer stays.
    pub const fn unreachable(&mut self) {
        self.unreachable = true;
    }

    /// `None` while the answer is unknown. An unreachable server counts as required.
    #[must_use]
    pub fn accounts_required(&self) -> Option<bool> {
        match &self.hello {
            Some(hello) => Some(hello.features.accounts_required),
            None if self.unreachable => Some(true),
            None => None,
        }
    }

    /// The last hello read.
    #[must_use]
    pub const fn hello(&self) -> Option<&Hello> {
        self.hello.as_ref()
    }

    #[must_use]
    pub fn limits(&self) -> Option<&Limits> {
        self.hello.as_ref().map(|hello| &hello.limits)
    }

    #[must_use]
    pub fn moved(&self) -> &[MovedAddress] {
        self.hello.as_ref().map_or(&[], |hello| &hello.moved)
    }

    /// False stops play. Unknown counts as allowed: the server still answers each play request.
    #[must_use]
    pub fn playback(&self) -> bool {
        self.hello.as_ref().is_none_or(|hello| hello.playback)
    }

    #[must_use]
    pub fn start_deadline_ms(&self) -> u32 {
        self.hello
            .as_ref()
            .and_then(|hello| {
                hello
                    .start_deadline_ms
                    .iter()
                    .find(|value| value.class == self.start.class)
            })
            .map_or(DEFAULT_START_DEADLINE_MS, |value| value.value)
    }
}

/// True when `version` is below `floor`. Parts compare as numbers; a missing part reads as 0.
#[must_use]
pub fn version_below(version: &str, floor: &str) -> bool {
    let mut left = version.split('.');
    let mut right = floor.split('.');
    loop {
        let (a, b) = (left.next(), right.next());
        if a.is_none() && b.is_none() {
            return false;
        }
        let (a, b) = (number(a), number(b));
        if a != b {
            return a < b;
        }
    }
}

fn number(part: Option<&str>) -> u64 {
    let part = part.unwrap_or_default();
    let digits = part
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(part.len());
    part[..digits].parse().unwrap_or(0)
}

/// The app, version, class and OS headers every request carries.
pub(crate) fn client_headers(config: &Config) -> [(&'static str, String); 4] {
    use alloc::string::ToString;
    let class = serde_json::to_value(config.class())
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_default();
    [
        ("Client-App", config.app().to_string()),
        ("Client-Version", config.version().to_string()),
        ("Client-Class", class),
        ("Client-Os", config.os().to_string()),
    ]
}
