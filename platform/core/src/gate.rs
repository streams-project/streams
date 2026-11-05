//! Who is watching, asked again after time away. The core only decides when to ask; the server
//! decides the outcome and no PIN is held here.

use contract::client::Form;

const MINUTE_MS: u64 = 60_000;

/// Activity counts at most this often.
const WRITE_EVERY_MS: u64 = 30_000;

/// Time away before asking. The TV value waits for a reference capture, so a TV is not asked yet.
#[must_use]
pub const fn idle_limit_ms(form: Form) -> Option<u64> {
    match form {
        Form::Phone => Some(120 * MINUTE_MS),
        Form::Tablet | Form::Desktop => Some(40 * MINUTE_MS),
        _ => None,
    }
}

/// Wall time is used here: a sleeping device stops monotonic time, and sleep is time away.
pub struct Gate {
    limit_ms: Option<u64>,
    last_seen: Option<u64>,
    profiles: usize,
    active: bool,
    ask: bool,
}

impl Gate {
    #[must_use]
    pub const fn new(form: Form) -> Self {
        Self {
            limit_ms: idle_limit_ms(form),
            last_seen: None,
            profiles: 0,
            active: false,
            ask: false,
        }
    }

    pub const fn set_profiles(&mut self, count: usize) {
        self.profiles = count;
    }

    pub const fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Note the time with no check.
    pub const fn seen(&mut self, wall_ms: u64) {
        self.last_seen = Some(wall_ms);
    }

    #[must_use]
    pub const fn last_seen(&self) -> Option<u64> {
        self.last_seen
    }

    #[must_use]
    pub const fn asking(&self) -> bool {
        self.ask
    }

    fn away(&self, wall_ms: u64) -> bool {
        let long = match (self.limit_ms, self.last_seen) {
            (Some(limit), Some(seen)) => wall_ms.saturating_sub(seen) > limit,
            _ => false,
        };
        long && self.profiles >= 2 && self.active
    }

    /// Something happened. True when this is the moment to ask.
    pub fn activity(&mut self, wall_ms: u64) -> bool {
        let ask_now = !self.ask && self.away(wall_ms);
        self.ask = self.ask || ask_now;
        let due = self
            .last_seen
            .is_none_or(|seen| wall_ms.saturating_sub(seen) > WRITE_EVERY_MS);
        if due {
            self.last_seen = Some(wall_ms);
        }
        ask_now
    }

    /// The viewer left. Time away counts from now.
    pub const fn left(&mut self, wall_ms: u64) {
        self.last_seen = Some(wall_ms);
    }

    /// A profile was entered.
    pub const fn answered(&mut self, wall_ms: u64) {
        self.ask = false;
        self.active = true;
        self.last_seen = Some(wall_ms);
    }

    /// The session ended.
    pub const fn clear(&mut self) {
        self.ask = false;
        self.active = false;
        self.profiles = 0;
    }
}
