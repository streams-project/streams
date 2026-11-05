//! Calls per minute for a key, checked before any lookup.

use std::collections::HashMap;
use std::sync::Mutex;

/// What a meter counts by. Standard hasher on purpose: the keys come from callers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeterKey {
    /// The one trusted address header, set by the front proxy.
    Address(String),
    /// A sign in attempt on one account name.
    Account(String),
}

/// A fixed one minute window per key.
#[derive(Debug)]
pub struct Meter {
    per_minute: u32,
    windows: Mutex<HashMap<MeterKey, (u64, u32)>>,
}

/// Most keys held before old windows are swept.
const MAX_KEYS: usize = 100_000;

impl Meter {
    /// A meter with a limit per minute.
    #[must_use]
    pub fn new(per_minute: u32) -> Self {
        Self {
            per_minute,
            windows: Mutex::new(HashMap::new()),
        }
    }

    /// Count one call. `Err` holds the seconds to wait.
    ///
    /// # Errors
    /// The seconds left in this window when the limit was passed.
    pub fn check(&self, key: MeterKey, now_ms: u64) -> Result<(), u32> {
        let minute = now_ms / 60_000;
        let mut windows = self
            .windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if windows.len() >= MAX_KEYS {
            windows.retain(|_, (window, _)| *window == minute);
        }
        let entry = windows.entry(key).or_insert((minute, 0));
        if entry.0 != minute {
            *entry = (minute, 0);
        }
        if entry.1 >= self.per_minute {
            let left = 60 - (now_ms / 1000) % 60;
            return Err(u32::try_from(left).unwrap_or(60));
        }
        entry.1 += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t7_the_limit_holds_inside_a_minute_and_clears_after_it() {
        let meter = Meter::new(2);
        let key = || MeterKey::Address("a".into());
        assert!(meter.check(key(), 0).is_ok());
        assert!(meter.check(key(), 1).is_ok());
        assert_eq!(meter.check(key(), 2), Err(60));
        assert!(meter.check(key(), 60_000).is_ok());
    }
}
