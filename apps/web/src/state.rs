//! What the screens read. The host writes these, and no screen keeps its own copy.

use std::collections::BTreeMap;

use contract::account::Profile;
use contract::core_msg::{PlayView, RouteTarget, SessionView};
use contract::driver::Track;
use contract::error::Notice;
use contract::hello::Hello;
use contract::screen::{NavItem, Screen, TitleDetail};
use leptos::prelude::*;

/// Position of the playing title.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Clock {
    pub time_ms: u64,
    pub duration_ms: u64,
}

/// Every lasting thing the core or the server told this page.
#[derive(Clone, Copy, Debug)]
pub struct Shared {
    pub session: RwSignal<Option<SessionView>>,
    pub route: RwSignal<Option<RouteTarget>>,
    pub gate: RwSignal<bool>,
    pub play: RwSignal<Option<PlayView>>,
    pub profiles: RwSignal<Vec<Profile>>,
    pub hello: RwSignal<Option<Hello>>,
    pub nav: RwSignal<Vec<NavItem>>,
    pub screen: RwSignal<Option<Screen>>,
    pub detail: RwSignal<Option<TitleDetail>>,
    pub notice: RwSignal<Option<Notice>>,
    pub clock: RwSignal<Clock>,
    pub audio: RwSignal<Vec<Track>>,
    pub text: RwSignal<Vec<Track>>,
    /// Interface text by label, from the server.
    pub strings: RwSignal<BTreeMap<String, String>>,
    /// The language the words above are in, as the server named it. It decides plural forms, so a
    /// screen reads it rather than guessing from the browser: the two differ as soon as a viewer
    /// asks for a language this server does not have and the words fall back to English.
    pub lang: RwSignal<String>,
    /// Which digits this language writes, from its own `language.digits`.
    pub digits: RwSignal<String>,
    /// How many notices are waiting, for the bell. The phone's `seen.unreadNotifications`
    /// (`apps/android/surfaces/phone/.../browse/Home.kt:231`).
    pub unread: RwSignal<u32>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            session: RwSignal::new(None),
            route: RwSignal::new(None),
            gate: RwSignal::new(false),
            play: RwSignal::new(None),
            profiles: RwSignal::new(Vec::new()),
            hello: RwSignal::new(None),
            nav: RwSignal::new(Vec::new()),
            screen: RwSignal::new(None),
            detail: RwSignal::new(None),
            notice: RwSignal::new(None),
            clock: RwSignal::new(Clock::default()),
            audio: RwSignal::new(Vec::new()),
            text: RwSignal::new(Vec::new()),
            strings: RwSignal::new(BTreeMap::new()),
            lang: RwSignal::new(String::new()),
            digits: RwSignal::new(String::new()),
            unread: RwSignal::new(0),
        }
    }
}
