//! Time as an input, so every rule with a window can be tested with scripted time.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Reads the time in whole milliseconds since 1970.
pub trait Clock: Send + Sync {
    /// Milliseconds since 1970.
    fn now_ms(&self) -> u64;
}

/// The machine clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
    }
}

/// A clock a test moves by hand.
#[derive(Debug, Default)]
pub struct ManualClock(AtomicU64);

impl ManualClock {
    /// A clock that starts at `ms`.
    #[must_use]
    pub const fn new(ms: u64) -> Self {
        Self(AtomicU64::new(ms))
    }

    /// Move forward.
    pub fn advance_ms(&self, ms: u64) {
        self.0.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}
