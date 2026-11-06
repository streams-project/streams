//! A provider and its record. Repeated failures rest it, so a dead source stops costing time.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use crate::Provider;

/// Failures in a row before a provider rests.
const FAILURES_TO_REST: u32 = 5;
/// How long a provider rests before it gets another try.
const REST: Duration = Duration::from_secs(60);

pub(crate) struct Slot {
    pub(crate) provider: Arc<dyn Provider>,
    failures: AtomicU32,
    resting_until: Mutex<Option<Instant>>,
}

impl Slot {
    pub(crate) fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            failures: AtomicU32::new(0),
            resting_until: Mutex::new(None),
        }
    }

    pub(crate) fn working(&self) -> bool {
        let until = self
            .resting_until
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        until.is_none_or(|until| Instant::now() >= until)
    }

    pub(crate) fn succeeded(&self) {
        self.failures.store(0, Ordering::SeqCst);
    }

    pub(crate) fn failed(&self, why: &str) {
        let failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        let name = self.provider.name();
        eprintln!("provider {name} failed: {why}");
        if failures >= FAILURES_TO_REST {
            self.failures.store(0, Ordering::SeqCst);
            let mut until = self
                .resting_until
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            *until = Some(Instant::now() + REST);
            eprintln!("provider {name} rests for {}s", REST.as_secs());
        }
    }
}
