//! One search across every provider, and the wait for it.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use contract::finder::{FindAnswer, FindOutcome, FindRequest, FoundSource};
use rand::Rng;
use tokio::sync::watch;

use crate::slot::Slot;

/// What makes two requests the same search.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RunKey(String);

impl RunKey {
    pub(crate) fn of(request: &FindRequest) -> Self {
        let episode = serde_json::to_string(&request.episode).unwrap_or_default();
        Self(format!("{}|{episode}", request.title))
    }
}

pub(crate) struct Run {
    pub(crate) job: String,
    found: Mutex<Found>,
    left: AtomicUsize,
    changed: watch::Sender<()>,
}

#[derive(Default)]
struct Found {
    sources: Vec<FoundSource>,
    first_at: Option<Instant>,
    finished_at: Option<Instant>,
}

fn new_job() -> String {
    let bytes: [u8; 16] = rand::rng().random();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl Run {
    pub(crate) fn start(
        request: &FindRequest,
        slots: &[Arc<Slot>],
        deadline: Duration,
    ) -> Arc<Self> {
        let working: Vec<Arc<Slot>> = slots
            .iter()
            .filter(|slot| slot.working())
            .cloned()
            .collect();
        let run = Arc::new(Self {
            job: new_job(),
            found: Mutex::new(Found::default()),
            left: AtomicUsize::new(working.len()),
            changed: watch::channel(()).0,
        });
        if working.is_empty() {
            run.lock().finished_at = Some(Instant::now());
        }
        for slot in working {
            let run = run.clone();
            let request = request.clone();
            tokio::spawn(async move {
                // A second task, so a provider that panics on odd input is one failure.
                let provider = slot.provider.clone();
                let searching = tokio::spawn(async move {
                    tokio::time::timeout(deadline, provider.find(&request)).await
                });
                let sources = match searching.await {
                    Ok(Ok(Ok(sources))) => {
                        slot.succeeded();
                        sources
                    }
                    Ok(Ok(Err(failed))) => {
                        slot.failed(&failed.0);
                        Vec::new()
                    }
                    Ok(Err(_)) => {
                        slot.failed("no answer by the deadline");
                        Vec::new()
                    }
                    Err(_) => {
                        slot.failed("it panicked");
                        Vec::new()
                    }
                };
                run.add(sources);
            });
        }
        run
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Found> {
        self.found.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn add(&self, sources: Vec<FoundSource>) {
        {
            let mut found = self.lock();
            if !sources.is_empty() && found.first_at.is_none() {
                found.first_at = Some(Instant::now());
            }
            for source in sources {
                if !found.sources.iter().any(|seen| seen.url == source.url) {
                    found.sources.push(source);
                }
            }
            if self.left.fetch_sub(1, Ordering::SeqCst) == 1 {
                found.finished_at = Some(Instant::now());
            }
        }
        self.changed.send_replace(());
    }

    pub(crate) fn finished(&self) -> bool {
        self.lock().finished_at.is_some()
    }

    pub(crate) fn expired(&self, keep: Duration) -> bool {
        self.lock()
            .finished_at
            .is_some_and(|at| at.elapsed() >= keep)
    }

    fn first_at(&self) -> Option<Instant> {
        self.lock().first_at
    }

    /// Everything known so far.
    pub(crate) fn answer(&self) -> FindAnswer {
        let found = self.lock();
        let finished = found.finished_at.is_some();
        let outcome = match (finished, found.sources.is_empty()) {
            (true, true) => FindOutcome::NotFound,
            (true, false) => FindOutcome::Found,
            (false, true) => FindOutcome::Pending,
            (false, false) => FindOutcome::Partial,
        };
        FindAnswer {
            outcome,
            sources: found.sources.clone(),
            job: (!finished).then(|| self.job.clone()),
        }
    }
}

/// Wait until the search is done, the budget ends, or the grace after the first source ends.
pub(crate) async fn wait(run: &Run, budget: Duration, grace: Duration) {
    let began = Instant::now();
    let mut changes = run.changed.subscribe();
    loop {
        if run.finished() {
            return;
        }
        let mut until = began + budget;
        if let Some(first) = run.first_at() {
            until = until.min(first + grace);
        }
        let now = Instant::now();
        if now >= until {
            return;
        }
        // A closed channel cannot happen while `run` is held. Waking early only loops.
        let _ = tokio::time::timeout(until - now, changes.changed()).await;
    }
}
