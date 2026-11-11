//! Work on a timer, in this process: reload the data set when it changes, and delete what has
//! lapsed. No database cron exists.

use std::sync::Arc;
use std::time::Duration;

use tracing::{info, warn};

use crate::state::Shared;

/// How often the manifest is read again.
const RELOAD_EVERY: Duration = Duration::from_secs(300);
/// How often lapsed rows are deleted.
const PRUNE_EVERY: Duration = Duration::from_secs(3600);
const DAY_MS: u64 = 86_400_000;
/// Viewing signals are kept this long.
const FEED_DAYS: u64 = 180;
/// Device reports are kept this long.
const REPORT_DAYS: u64 = 30;

/// Read the manifest again. A refused set leaves the last good one in place, and the index of a
/// new set opens or builds in the background.
pub async fn reload_data(state: &Shared) {
    let Some(slot) = state.data.clone() else {
        return;
    };
    let index = state.config.index_path.clone();
    let done = tokio::task::spawn_blocking(move || {
        let changed = slot.reload();
        if matches!(changed, Ok(true))
            && let Some(path) = index
        {
            let set = slot.load();
            if !set.open_index(&path)
                && let Err(error) = set.build_index(&path)
            {
                warn!("Index not built: {error}");
            }
        }
        changed
    })
    .await;
    match done {
        Ok(Ok(true)) => info!("A new data set is in use"),
        Ok(Ok(false)) => {}
        Ok(Err(error)) => warn!("New data set refused, the last good one stays: {error}"),
        Err(error) => warn!("Reload stopped: {error}"),
    }
}

/// Delete lapsed sessions, audit rows, device codes, viewing signals and reports.
pub async fn prune(state: &Shared) -> u64 {
    let now = state.clock.now_ms();
    let mut gone = 0;
    if let Some(door) = &state.door {
        gone += door.prune().await.unwrap_or(0);
    }
    if let Some(library) = &state.library {
        let before = now.saturating_sub(FEED_DAYS * DAY_MS);
        gone += library.store().prune_feed(before).await.unwrap_or(0);
    }
    if let Some(log) = state.log_of_no_one() {
        gone += log.prune(now.saturating_sub(REPORT_DAYS * DAY_MS)).await;
    }
    gone
}

/// Start the timers.
pub fn start(state: &Shared) {
    let reloading = Arc::clone(state);
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(RELOAD_EVERY);
        timer.tick().await;
        loop {
            timer.tick().await;
            reload_data(&reloading).await;
        }
    });
    let pruning = Arc::clone(state);
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(PRUNE_EVERY);
        loop {
            timer.tick().await;
            let gone = prune(&pruning).await;
            if gone > 0 {
                info!("Deleted {gone} lapsed rows");
            }
        }
    });
}
