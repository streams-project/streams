//! The private route between server and finder. The finder returns facts and no order.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hello::ServiceState;
use crate::play::AttemptKind;
use crate::title::{EpisodeRef, TitleKey};

words! {
    /// How a search for sources ended.
    pub enum FindOutcome {
        Found,
        Partial,
        NotFound,
        Pending,
    }
}

wire! {
    /// Title facts come from the server, so a finder never looks them up.
    pub struct FindRequest {
        pub title: TitleKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub episode: Option<EpisodeRef>,
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub year: Option<u32>,
        /// Total time this search may take.
        pub budget_ms: u32,
    }

    /// One source as facts. The server turns these into ranked attempts.
    pub struct FoundSource {
        pub url: String,
        pub kind: AttemptKind,
        pub group: String,
        pub headers: BTreeMap<String, String>,
        pub verified: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub container: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub height: Option<u32>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub audio: Vec<String>,
    }

    /// Answer, possibly partial when the budget ran out.
    pub struct FindAnswer {
        pub outcome: FindOutcome,
        pub sources: Vec<FoundSource>,
        /// Read the rest through `/v1/find/{job}`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub job: Option<String>,
    }

    /// `/v1/health` of a finder.
    pub struct FinderHealth {
        pub build: String,
        pub state: ServiceState,
    }
}
