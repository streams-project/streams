//! Three contracts, versioned apart: the wire by date, the C boundary by number, crates by semver.

use alloc::string::String;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize};

/// Wire date this build writes. Sent in the `Api-Version` header and in every core message.
pub const WIRE_VERSION: &str = "2026-11-05";

/// Oldest wire date this build still reads.
pub const WIRE_MIN: &str = "2026-11-05";

/// Number of the six function C boundary. It stays frozen; changes live inside the messages.
pub const ABI_VERSION: u32 = 1;

/// Version of this crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Header that carries the wire date on every API call.
pub const API_VERSION_HEADER: &str = "Api-Version";

/// Every wire date with what changed. Entries are never removed.
pub const VERSION_LOG: &[(&str, &str)] = &[("2026-11-05", "First shape. Not released.")];

/// Results of `core_step`. Everything richer travels inside the messages.
pub mod step_status {
    /// The step ran. Read the out message.
    pub const OK: i32 = 0;
    /// The in message could not be read. Core state is unchanged.
    pub const BAD_INPUT: i32 = 1;
    /// The handle is not open.
    pub const BAD_HANDLE: i32 = 2;
    /// The core stopped. Open a new one from what the host stored, or cold.
    pub const FAILED: i32 = 3;
}

/// A wire date, `YYYY-MM-DD`. Text order is date order.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct WireDate(String);

impl WireDate {
    /// Read a date.
    ///
    /// # Errors
    /// Text that is not `YYYY-MM-DD`.
    pub fn parse(text: &str) -> Result<Self, BadWireDate> {
        let digit = |b: &u8| b.is_ascii_digit();
        let shaped = matches!(
            text.as_bytes(),
            [a, b, c, d, b'-', e, f, b'-', g, h]
                if [a, b, c, d, e, f, g, h].iter().all(|x| digit(x))
        );
        if shaped {
            Ok(Self(String::from(text)))
        } else {
            Err(BadWireDate)
        }
    }

    /// The date this build writes.
    #[must_use]
    pub fn current() -> Self {
        Self(String::from(WIRE_VERSION))
    }

    /// The date as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for WireDate {
    fn default() -> Self {
        Self::current()
    }
}

/// A text that is not a wire date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadWireDate;

impl fmt::Display for BadWireDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a wire date is YYYY-MM-DD")
    }
}

impl core::error::Error for BadWireDate {}

impl<'de> Deserialize<'de> for WireDate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for WireDate {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "WireDate".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}$",
            "description": "A wire date, YYYY-MM-DD."
        })
    }
}

wire! {
    /// The wire dates a server answers. Never compared for equality.
    pub struct VersionRange {
        pub min: WireDate,
        pub max: WireDate,
    }
}

/// How a client's date sits against a server's range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum VersionFit {
    /// Inside the range.
    Supported,
    /// Older than the range: answered `upgrade_required`.
    TooOld,
    /// Newer than the range: served, because additive changes need no bump.
    Newer,
}

impl VersionRange {
    /// Where `date` sits.
    #[must_use]
    pub fn fit(&self, date: &WireDate) -> VersionFit {
        if *date < self.min {
            VersionFit::TooOld
        } else if *date > self.max {
            VersionFit::Newer
        } else {
            VersionFit::Supported
        }
    }
}
