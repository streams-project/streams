//! The PIN rule, one home. The store holds the row lock; the rule decides here.

use crate::limits::AccountLimits;

/// Wrong tries before a lock.
pub const PIN_TRIES: u16 = 5;
/// First lock, in seconds.
pub const PIN_LOCK_S: u64 = 900;
/// Longest lock. Each lock after the first doubles up to here (P3, target value).
pub const PIN_LOCK_MAX_S: u64 = 86_400;

/// The stored row of a profile with a PIN.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinRow {
    pub hash: String,
    pub failed: u16,
    pub locked_until_ms: Option<u64>,
    /// Locks since the last right PIN.
    pub locks: u16,
}

/// A PIN check: what was typed and the time.
#[derive(Clone, Copy, Debug)]
pub struct PinCheck<'a> {
    pub pin: Option<&'a str>,
    pub now_ms: u64,
}

/// What a check found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinOutcome {
    /// Right PIN, or the profile has none.
    Pass,
    /// Wrong. The new counter is written and survives (never an error that rolls back).
    Wrong,
    /// Locked. A locked answer differs from a wrong one.
    Locked { retry_after_s: u32 },
    /// The profile has a PIN and none was given.
    Missing,
}

/// The cost for new hashes.
pub const PIN_COST: u32 = 10;

/// Four digits, nothing else.
#[must_use]
pub fn pin_shape_ok(pin: &str, limits: &AccountLimits) -> bool {
    pin.len() == limits.pin_digits as usize && pin.bytes().all(|b| b.is_ascii_digit())
}

/// A hash for a new PIN.
///
/// # Panics
/// Never for a PIN of four digits and a fixed cost.
#[must_use]
pub fn hash_pin(pin: &str) -> String {
    bcrypt::hash(pin, PIN_COST).expect("bcrypt hashes short text at a valid cost")
}

/// Decide a check and return the row to write, if it changed.
#[must_use]
pub fn decide(row: &PinRow, check: PinCheck<'_>) -> (PinOutcome, Option<PinRow>) {
    if let Some(until) = row.locked_until_ms.filter(|until| *until > check.now_ms) {
        let left = (until - check.now_ms).div_ceil(1000);
        let retry_after_s = u32::try_from(left).unwrap_or(u32::MAX);
        return (PinOutcome::Locked { retry_after_s }, None);
    }
    let Some(pin) = check.pin else {
        return (PinOutcome::Missing, None);
    };
    if bcrypt::verify(pin, &row.hash).unwrap_or(false) {
        let clean = PinRow {
            hash: row.hash.clone(),
            failed: 0,
            locked_until_ms: None,
            locks: 0,
        };
        let changed = clean != *row;
        return (PinOutcome::Pass, changed.then_some(clean));
    }
    let mut next = row.clone();
    next.failed += 1;
    if next.failed >= PIN_TRIES {
        let shift = u32::from(next.locks).min(8);
        let lock_s = (PIN_LOCK_S << shift).min(PIN_LOCK_MAX_S);
        next.failed = 0;
        next.locks = next.locks.saturating_add(1);
        next.locked_until_ms = Some(check.now_ms + lock_s * 1000);
    }
    (PinOutcome::Wrong, Some(next))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::DEFAULT_LIMITS;

    fn row() -> PinRow {
        // Cost 4 keeps the test quick. The cost is read from the hash.
        PinRow {
            hash: bcrypt::hash("1234", 4).unwrap(),
            failed: 0,
            locked_until_ms: None,
            locks: 0,
        }
    }

    fn check(pin: &str, now_ms: u64) -> PinCheck<'_> {
        PinCheck {
            pin: Some(pin),
            now_ms,
        }
    }

    #[test]
    fn s43_shape_is_exactly_four_digits() {
        assert!(pin_shape_ok("0042", &DEFAULT_LIMITS));
        for bad in ["123", "12345", "12a4", ""] {
            assert!(!pin_shape_ok(bad, &DEFAULT_LIMITS), "{bad}");
        }
    }

    #[test]
    fn s43_five_wrong_tries_lock_for_fifteen_minutes_and_reset_the_counter() {
        let mut current = row();
        for n in 1..=4 {
            let (outcome, next) = decide(&current, check("0000", 0));
            assert_eq!(outcome, PinOutcome::Wrong);
            current = next.unwrap();
            assert_eq!(current.failed, n);
        }
        let (outcome, next) = decide(&current, check("0000", 0));
        assert_eq!(outcome, PinOutcome::Wrong);
        let locked = next.unwrap();
        assert_eq!((locked.failed, locked.locked_until_ms), (0, Some(900_000)));
        // A locked answer differs from a wrong one, and the right PIN does not open it early.
        assert_eq!(
            decide(&locked, check("1234", 1_000)).0,
            PinOutcome::Locked { retry_after_s: 899 }
        );
        assert_eq!(decide(&locked, check("1234", 900_000)).0, PinOutcome::Pass);
    }

    #[test]
    fn p3_each_lock_after_the_first_lasts_longer() {
        let mut current = row();
        current.locks = 2;
        current.failed = PIN_TRIES - 1;
        let (_, next) = decide(&current, check("0000", 0));
        assert_eq!(next.unwrap().locked_until_ms, Some(3_600_000));
        current.locks = 8;
        let (_, next) = decide(&current, check("0000", 0));
        assert_eq!(next.unwrap().locked_until_ms, Some(PIN_LOCK_MAX_S * 1000));
    }

    #[test]
    fn s125_a_right_pin_clears_the_counters_and_no_pin_asks_for_one() {
        let mut current = row();
        current.failed = 3;
        let (outcome, next) = decide(&current, check("1234", 0));
        assert_eq!(outcome, PinOutcome::Pass);
        assert_eq!(next.unwrap().failed, 0);
        let none = PinCheck {
            pin: None,
            now_ms: 0,
        };
        assert_eq!(decide(&row(), none).0, PinOutcome::Missing);
    }
}
