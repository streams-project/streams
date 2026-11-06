//! Sign in state. Tokens stay in this module and never reach a view. Expiry comes from
//! `expires_in` and monotonic time, never from the device clock read against a date.

use alloc::format;
use alloc::string::{String, ToString};
use contract::account::SignedIn;
use contract::core_msg::{EndReason, SessionState, SessionView};
use serde::{Deserialize, Serialize};

use crate::Now;

/// Renew when less than this is left. 120 s holds over the older 60 s.
const RENEW_MARGIN_MS: u64 = 120_000;

/// Wait after a refresh that got no usable answer.
const CHECK_EVERY_MS: u64 = 300_000;

/// Secret name of the session.
pub(crate) const SESSION_SECRET: &str = "session";

/// Secret name of the device key, which outlives a session.
pub(crate) const DEVICE_SECRET: &str = "device";

/// What survives a restart. Only the refresh token: the access token is short lived.
#[derive(Serialize, Deserialize)]
pub(crate) struct Kept {
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// True when the kept profile's home is the Kids home.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub kids: bool,
    /// Wall time of the last activity, so time away reaches across a restart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seen_ms: Option<u64>,
}

/// One owner of tokens, so one refresher per device.
pub(crate) struct Session {
    state: SessionState,
    user_id: Option<String>,
    email: Option<String>,
    profile: Option<String>,
    kids: bool,
    ended: Option<EndReason>,
    access: String,
    refresh: String,
    /// Counts tokens, so a request can tell it carried an old one.
    serial: u32,
    expires_mono: Option<u64>,
    expires_wall: Option<u64>,
    refreshing: bool,
    retry_at_mono: u64,
    device_key: String,
}

impl Session {
    pub(crate) fn new(seed: &str) -> Self {
        Self {
            state: SessionState::Starting,
            user_id: None,
            email: None,
            profile: None,
            kids: false,
            ended: None,
            access: String::new(),
            refresh: String::new(),
            serial: 0,
            expires_mono: None,
            expires_wall: None,
            refreshing: false,
            retry_at_mono: 0,
            device_key: seed.chars().take(32).collect(),
        }
    }

    pub(crate) const fn state(&self) -> SessionState {
        self.state
    }

    pub(crate) fn signed_in(&self) -> bool {
        self.state == SessionState::SignedIn
    }

    pub(crate) fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    pub(crate) fn device_key(&self) -> &str {
        &self.device_key
    }

    pub(crate) fn set_device_key(&mut self, key: String) {
        self.device_key = key;
    }

    /// No session was kept.
    pub(crate) const fn none_kept(&mut self) {
        self.state = SessionState::SignedOut;
    }

    /// A kept session is back. Its access token is unknown, so renewal is due at once.
    pub(crate) fn restore(&mut self, kept: Kept) {
        self.state = SessionState::SignedIn;
        self.user_id = Some(kept.user_id);
        self.email = Some(kept.email);
        self.profile = kept.profile;
        self.kids = kept.kids;
        self.refresh = kept.refresh_token;
        self.expires_mono = None;
    }

    /// Hold tokens from a sign in or a refresh.
    pub(crate) fn accept(&mut self, signed_in: SignedIn, now: Now) {
        let same_user = self.user_id.as_deref() == Some(signed_in.user_id.as_str());
        if !same_user {
            self.profile = None;
            self.kids = false;
        }
        let life_ms = u64::from(signed_in.expires_in) * 1_000;
        self.state = SessionState::SignedIn;
        self.ended = None;
        self.user_id = Some(signed_in.user_id);
        if !signed_in.email.is_empty() {
            self.email = Some(signed_in.email);
        }
        self.access = signed_in.access_token;
        self.refresh = signed_in.refresh_token;
        self.serial += 1;
        self.expires_mono = Some(now.mono_ms + life_ms);
        self.expires_wall = Some(now.wall_ms + life_ms);
        self.refreshing = false;
        self.retry_at_mono = 0;
    }

    pub(crate) fn set_profile(&mut self, profile: String, kids: bool) {
        self.profile = Some(profile);
        self.kids = kids;
    }

    pub(crate) const fn kids(&self) -> bool {
        self.kids
    }

    pub(crate) fn access_token(&self) -> &str {
        &self.access
    }

    pub(crate) fn refresh_token(&self) -> &str {
        &self.refresh
    }

    pub(crate) const fn serial(&self) -> u32 {
        self.serial
    }

    pub(crate) fn has_refresh(&self) -> bool {
        !self.refresh.is_empty()
    }

    pub(crate) fn kept_text(&self, seen_ms: Option<u64>) -> Option<String> {
        let kept = Kept {
            refresh_token: self.refresh.clone(),
            user_id: self.user_id.clone()?,
            email: self.email.clone().unwrap_or_default(),
            profile: self.profile.clone(),
            kids: self.kids,
            seen_ms,
        };
        serde_json::to_string(&kept).ok()
    }

    /// Opaque scope of this account and profile, once a profile is in use.
    pub(crate) fn scope(&self) -> Option<String> {
        Some(format!(
            "{}/{}",
            self.user_id.as_ref()?,
            self.profile.as_ref()?
        ))
    }

    /// True when a renewal should start now.
    pub(crate) fn renewal_due(&self, now: Now) -> bool {
        self.signed_in()
            && self.has_refresh()
            && !self.refreshing
            && now.mono_ms >= self.retry_at_mono
            && self
                .expires_mono
                .is_none_or(|expires| now.mono_ms + RENEW_MARGIN_MS >= expires)
    }

    /// True when a request that takes a token should wait for the renewal.
    pub(crate) fn must_wait(&self, now: Now) -> bool {
        self.refreshing || self.renewal_due(now)
    }

    pub(crate) const fn refreshing(&self) -> bool {
        self.refreshing
    }

    pub(crate) const fn start_refresh(&mut self) {
        self.refreshing = true;
    }

    /// The server refused this token, so it counts as run out whatever its life said.
    pub(crate) const fn token_refused(&mut self, now: Now) {
        self.expires_mono = Some(now.mono_ms);
    }

    /// The refresh got no usable answer. What exists stays, and it is tried later.
    pub(crate) const fn refresh_failed(&mut self, now: Now) {
        self.refreshing = false;
        self.retry_at_mono = now.mono_ms + CHECK_EVERY_MS;
    }

    /// A network return cuts the wait.
    pub(crate) const fn cut_wait(&mut self) {
        self.retry_at_mono = 0;
    }

    /// Monotonic time of the next renewal.
    pub(crate) fn next_wake(&self) -> Option<u64> {
        if !self.signed_in() || !self.has_refresh() || self.refreshing {
            return None;
        }
        let renew_at = self
            .expires_mono
            .map_or(0, |expires| expires.saturating_sub(RENEW_MARGIN_MS));
        Some(renew_at.max(self.retry_at_mono))
    }

    /// Forget everything but the device key.
    pub(crate) fn end(&mut self, reason: EndReason) {
        self.state = SessionState::SignedOut;
        self.user_id = None;
        self.email = None;
        self.profile = None;
        self.kids = false;
        self.ended = Some(reason);
        self.access.clear();
        self.refresh.clear();
        self.serial += 1;
        self.expires_mono = None;
        self.expires_wall = None;
        self.refreshing = false;
        self.retry_at_mono = 0;
    }

    /// The view. It has no field that could hold a token.
    pub(crate) fn view(&self) -> SessionView {
        SessionView {
            state: self.state,
            user_id: self.user_id.clone(),
            email: self.email.clone(),
            expires_at_ms: self.expires_wall,
            profile: self.profile.clone(),
            scope: self.scope(),
            ended: self.ended,
        }
    }
}

/// The start of a seed is the device key, and its end names this run in op ids.
pub(crate) fn run_name(seed: &str) -> String {
    let from = seed.len().saturating_sub(16);
    seed[from..].to_string()
}
