//! Device sign in (RFC 8628). The core asks for the code and polls, so a host only draws the view.
//! The device code stays here and leaves only in the body of a poll.

use alloc::string::String;
use contract::account::{DeviceCode, DevicePoll};
use contract::core_msg::{DeviceSignInState, DeviceSignInView};

use crate::net::Backoff;

/// RFC 8628 3.5: the interval with no usable value, and what `slow_down` adds to it.
const POLL_STEP_MS: u64 = 5_000;

/// What to send now.
pub(crate) enum Due {
    /// Ask for a code for the device of this name.
    Code(String),
    /// Ask for tokens with this device code, which is one poll.
    Token(String),
}

struct Held {
    device_code: String,
    interval_ms: u64,
    expires_mono: u64,
}

const fn bare(state: DeviceSignInState) -> DeviceSignInView {
    DeviceSignInView {
        state,
        user_code: None,
        verification_uri: None,
        verification_uri_complete: None,
        expires_in: None,
    }
}

/// One device sign in, with one owner.
pub(crate) struct Pairing {
    device_name: String,
    held: Option<Held>,
    /// Goes up whenever a sign in starts or ends, so an older answer is dropped.
    serial: u32,
    /// Monotonic time of the next request. Empty while one is on its way or nothing is running.
    send_at_mono: Option<u64>,
    on_the_way: bool,
    /// True while the wait is a retry, which a network return cuts short.
    failing: bool,
    backoff: Backoff,
    view: DeviceSignInView,
    /// What the screen was last told.
    told: DeviceSignInView,
}

impl Pairing {
    pub(crate) const fn new() -> Self {
        Self {
            device_name: String::new(),
            held: None,
            serial: 0,
            send_at_mono: None,
            on_the_way: false,
            failing: false,
            backoff: Backoff::new(),
            view: bare(DeviceSignInState::Idle),
            told: bare(DeviceSignInState::Idle),
        }
    }

    pub(crate) const fn serial(&self) -> u32 {
        self.serial
    }

    pub(crate) const fn is_current(&self, serial: u32) -> bool {
        serial == self.serial
    }

    fn clear(&mut self, state: DeviceSignInState) {
        self.serial += 1;
        self.held = None;
        self.send_at_mono = None;
        self.on_the_way = false;
        self.failing = false;
        self.backoff.reset();
        self.view = bare(state);
    }

    /// Start, or start again after a denied code or a server that was off. A live code stays.
    pub(crate) fn start(&mut self, device_name: String, mono_ms: u64) {
        if matches!(
            self.view.state,
            DeviceSignInState::Waiting | DeviceSignInState::Expired
        ) {
            return;
        }
        self.clear(DeviceSignInState::Waiting);
        self.device_name = device_name;
        self.send_at_mono = Some(mono_ms);
    }

    pub(crate) fn stop(&mut self) {
        self.clear(DeviceSignInState::Idle);
    }

    /// True when this answer belongs to the running sign in. The next request may then go.
    const fn landed(&mut self, serial: u32) -> bool {
        let current = serial == self.serial && self.on_the_way;
        if current {
            self.on_the_way = false;
        }
        current
    }

    const fn answered(&mut self) {
        self.failing = false;
        self.backoff.reset();
    }

    /// The code in hand is dead, so a new one is asked for with no press.
    fn lapse(&mut self, mono_ms: u64) {
        self.held = None;
        self.view = bare(DeviceSignInState::Expired);
        self.send_at_mono = Some(mono_ms);
    }

    /// The request due now, if any. A code past its life is renewed here, so a poll that cannot
    /// reach the server never leaves a dead code on the screen.
    pub(crate) fn due(&mut self, mono_ms: u64) -> Option<Due> {
        if self.on_the_way {
            return None;
        }
        if self
            .held
            .as_ref()
            .is_some_and(|held| mono_ms >= held.expires_mono)
        {
            self.lapse(mono_ms);
        }
        if self.send_at_mono.is_none_or(|at| mono_ms < at) {
            return None;
        }
        self.send_at_mono = None;
        self.on_the_way = true;
        Some(self.held.as_ref().map_or_else(
            || Due::Code(self.device_name.clone()),
            |held| Due::Token(held.device_code.clone()),
        ))
    }

    pub(crate) fn code_read(&mut self, serial: u32, code: DeviceCode, mono_ms: u64) {
        if !self.landed(serial) {
            return;
        }
        self.answered();
        let interval_ms = match u64::from(code.interval) * 1_000 {
            0 => POLL_STEP_MS,
            interval_ms => interval_ms,
        };
        // Every code gets one poll, so renewing can never outrun the interval.
        let life_ms = (u64::from(code.expires_in) * 1_000).max(interval_ms);
        self.held = Some(Held {
            device_code: code.device_code,
            interval_ms,
            expires_mono: mono_ms + life_ms,
        });
        self.view = DeviceSignInView {
            state: DeviceSignInState::Waiting,
            user_code: Some(code.user_code),
            verification_uri: Some(code.verification_uri),
            verification_uri_complete: Some(code.verification_uri_complete),
            expires_in: Some(code.expires_in),
        };
        self.send_at_mono = Some(mono_ms + interval_ms);
    }

    pub(crate) fn polled(&mut self, serial: u32, word: DevicePoll, mono_ms: u64) {
        if !self.landed(serial) {
            return;
        }
        self.answered();
        match word {
            DevicePoll::ExpiredToken => self.lapse(mono_ms),
            DevicePoll::AccessDenied => self.clear(DeviceSignInState::Denied),
            word => {
                if let Some(held) = &mut self.held {
                    if word == DevicePoll::SlowDown {
                        held.interval_ms += POLL_STEP_MS;
                    }
                    self.send_at_mono = Some(mono_ms + held.interval_ms);
                }
            }
        }
    }

    /// No usable answer came, so the same request goes again, never inside the interval. With
    /// no wait from the server it backs off, as RFC 8628 3.5 asks after a timeout.
    pub(crate) fn no_answer(&mut self, serial: u32, wait_ms: Option<u64>, mono_ms: u64) {
        if !self.landed(serial) {
            return;
        }
        let wait_ms = wait_ms.unwrap_or_else(|| self.backoff.next_wait());
        let interval_ms = self.held.as_ref().map_or(0, |held| held.interval_ms);
        self.failing = true;
        self.send_at_mono = Some(mono_ms + wait_ms.max(interval_ms));
    }

    /// The server cannot do this. True when the answer belonged to the running sign in.
    pub(crate) fn off(&mut self, serial: u32) -> bool {
        let current = self.landed(serial);
        if current {
            self.clear(DeviceSignInState::Off);
        }
        current
    }

    /// A network return cuts a retry's wait. A wait that is the interval stays, because a poll
    /// inside it earns `slow_down`.
    pub(crate) const fn cut_wait(&mut self, mono_ms: u64) {
        if self.failing && self.send_at_mono.is_some() {
            self.send_at_mono = Some(mono_ms);
        }
    }

    pub(crate) fn next_wake(&self) -> Option<u64> {
        if self.on_the_way {
            return None;
        }
        let expires = self.held.as_ref().map(|held| held.expires_mono);
        [self.send_at_mono, expires].into_iter().flatten().min()
    }

    /// The view when it changed since the screen was last told.
    pub(crate) fn take_view(&mut self) -> Option<DeviceSignInView> {
        (self.view != self.told).then(|| {
            self.told = self.view.clone();
            self.view.clone()
        })
    }
}
