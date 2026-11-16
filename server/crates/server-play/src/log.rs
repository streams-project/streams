//! What devices report: playback health and error reports.
//!
//! Neither is about a person. [`PlayLog::health`] takes a client class, a day and a set of counts
//! keyed by host; [`PlayLog::report`] takes a build and a code. No method here has a profile, an
//! account or a session parameter, so there is no call site at which one could be added by
//! accident, and no caller has to remember to ask whether a person agreed to be recorded before
//! writing a count about a streaming host.
//!
//! Before this, playback events went to `play_events` keyed to `profile_id` and gated on the
//! personal-rows consent answer. Nothing ever wrote a row there: no client has ever called
//! `POST /v1/play/{id}/events`. The table and its thirty day prune stay, because dropping it
//! would change published policy text, and that is the owner's call; nothing writes to it.

use std::sync::Mutex;

use async_trait::async_trait;
use contract::play::Report;
use sqlx::PgPool;
use time::{Date, OffsetDateTime};

use crate::health::{Health, HostSummary, Tally, group, summarize};

/// Where reports go. A failed write loses a report and never a viewer's request.
#[async_trait]
pub trait PlayLog: Send + Sync {
    /// Add these counts to the day they name. Each is one (class, host, day, fault) key.
    async fn health(&self, rows: &[Tally]);
    /// Playback health from this day forward, newest day first, grouped by (class, host, day).
    async fn health_since(&self, day: Date) -> Vec<Health>;
    /// Playback health from this day forward filtered by optional client class and/or host.
    async fn health_for(&self, day: Date, class: Option<&str>, host: Option<&str>) -> Vec<Health> {
        self.health_since(day)
            .await
            .into_iter()
            .filter(|row| {
                class.is_none_or(|wanted| row.class == wanted)
                    && host.is_none_or(|wanted| row.host == wanted)
            })
            .collect()
    }
    /// Playback health rolled up by `(class, host)` from this day forward.
    async fn health_summary(&self, day: Date) -> Vec<HostSummary> {
        summarize(&self.health_since(day).await)
    }
    /// Keep a client error report.
    async fn report(&self, report: &Report);
    /// Delete reports older than this.
    async fn prune(&self, before_ms: u64) -> u64;
}

/// Reports in memory, for tests. The health store merges on the same key the table does, so it is
/// a model of the Postgres upsert and not a different thing that happens to pass.
#[derive(Debug, Default)]
pub struct MemoryPlayLog {
    pub health: Mutex<Vec<Tally>>,
    pub reports: Mutex<Vec<Report>>,
}

impl MemoryPlayLog {
    /// Every stored row, in insertion order.
    #[must_use]
    pub fn rows(&self) -> Vec<Tally> {
        self.health
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl PlayLog for MemoryPlayLog {
    async fn health(&self, rows: &[Tally]) {
        let mut held = self
            .health
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for row in rows {
            let key = |other: &Tally| {
                other.class == row.class
                    && other.host == row.host
                    && other.day == row.day
                    && other.fault == row.fault
            };
            match held.iter_mut().find(|held| key(held)) {
                Some(found) => found.add(row),
                None => held.push(row.clone()),
            }
        }
    }

    async fn health_since(&self, day: Date) -> Vec<Health> {
        self.health_for(day, None, None).await
    }

    async fn health_for(&self, day: Date, class: Option<&str>, host: Option<&str>) -> Vec<Health> {
        let held = self
            .health
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        group(
            held.iter()
                .filter(|row| {
                    row.day >= day
                        && class.is_none_or(|wanted| row.class == wanted)
                        && host.is_none_or(|wanted| row.host == wanted)
                })
                .cloned()
                .collect(),
        )
    }

    async fn report(&self, report: &Report) {
        self.reports
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(report.clone());
    }

    async fn prune(&self, _: u64) -> u64 {
        0
    }
}

/// Reports on Postgres. Needs a real database to run.
#[derive(Clone, Debug)]
pub struct PgPlayLog {
    pool: PgPool,
}

impl PgPlayLog {
    /// A log on an open pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlayLog for PgPlayLog {
    async fn health(&self, rows: &[Tally]) {
        for row in rows {
            let _ = sqlx::query(
                "insert into reachability \
                 (class, host, day, fault, tries, starts, start_ms_total, rebuffers, failures, \
                 gave_up) \
                 values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
                 on conflict (class, host, day, fault) do update set \
                 tries = reachability.tries + excluded.tries, \
                 starts = reachability.starts + excluded.starts, \
                 start_ms_total = reachability.start_ms_total + excluded.start_ms_total, \
                 rebuffers = reachability.rebuffers + excluded.rebuffers, \
                 failures = reachability.failures + excluded.failures, \
                 gave_up = reachability.gave_up + excluded.gave_up, \
                 updated_at = now()",
            )
            .bind(&row.class)
            .bind(&row.host)
            .bind(row.day)
            .bind(&row.fault)
            .bind(i32::try_from(row.tries).unwrap_or(i32::MAX))
            .bind(i32::try_from(row.starts).unwrap_or(i32::MAX))
            .bind(row.start_ms_total)
            .bind(i32::try_from(row.rebuffers).unwrap_or(i32::MAX))
            .bind(i32::try_from(row.failures).unwrap_or(i32::MAX))
            .bind(i32::try_from(row.gave_up).unwrap_or(i32::MAX))
            .execute(&self.pool)
            .await;
        }
    }

    async fn health_since(&self, day: Date) -> Vec<Health> {
        self.health_for(day, None, None).await
    }

    async fn health_for(&self, day: Date, class: Option<&str>, host: Option<&str>) -> Vec<Health> {
        let found = sqlx::query_as::<_, (String, String, Date, String, i32, i32, i64, i32, i32, i32)>(
            "select class, host, day, fault, tries, starts, start_ms_total, rebuffers, failures, \
             gave_up from reachability \
             where day >= $1 \
               and ($2::text is null or class = $2) \
               and ($3::text is null or host = $3) \
             order by day desc, class, host, fault",
        )
        .bind(day)
        .bind(class)
        .bind(host)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        group(
            found
                .into_iter()
                .map(|row| Tally {
                    class: row.0,
                    host: row.1,
                    day: row.2,
                    fault: row.3,
                    tries: i64::from(row.4),
                    starts: i64::from(row.5),
                    start_ms_total: row.6,
                    rebuffers: i64::from(row.7),
                    failures: i64::from(row.8),
                    gave_up: i64::from(row.9),
                })
                .collect(),
        )
    }

    async fn report(&self, report: &Report) {
        let body = serde_json::to_value(report).unwrap_or_default();
        let _ = sqlx::query("insert into client_reports (build, code, body) values ($1, $2, $3)")
            .bind(&report.build)
            .bind(&report.code)
            .bind(body)
            .execute(&self.pool)
            .await;
    }

    async fn prune(&self, before_ms: u64) -> u64 {
        let nanos = i128::from(before_ms) * 1_000_000;
        let before =
            OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap_or(OffsetDateTime::UNIX_EPOCH);
        let mut gone = 0;
        for statement in [
            "delete from play_events where ts < $1",
            "delete from client_reports where ts < $1",
        ] {
            if let Ok(done) = sqlx::query(statement)
                .bind(before)
                .execute(&self.pool)
                .await
            {
                gone += done.rows_affected();
            }
        }
        gone
    }
}

#[cfg(test)]
mod tests {
    use contract::driver::ErrorClass;
    use contract::play::{PlayEvent, PlayEventName};

    use super::*;
    use crate::health::fold;

    fn day(n: u16) -> Date {
        Date::from_ordinal_date(2026, 300 + n).expect("a day")
    }

    fn event(name: PlayEventName, host: &str) -> PlayEvent {
        PlayEvent {
            name,
            at_ms: 0,
            attempt: Some("a1".to_owned()),
            first_frame_ms: None,
            failure: None,
            host: Some(host.to_owned()),
            rung: None,
        }
    }

    #[tokio::test]
    async fn memory_store_merges_matching_keys_and_reads_back_by_day_host_and_class() {
        let store = MemoryPlayLog::default();

        let mut started_one = event(PlayEventName::FirstFrame, "edge-a.example");
        started_one.first_frame_ms = Some(900);
        let mut failed_seg = event(PlayEventName::Failure, "edge-b.example");
        failed_seg.failure = Some(ErrorClass::Segment);

        store
            .health(&fold(
                "tv",
                day(0),
                &[
                    failed_seg.clone(),
                    started_one,
                    event(PlayEventName::Rebuffer, "edge-a.example"),
                    event(PlayEventName::Ended, "edge-a.example"),
                ],
            ))
            .await;

        let mut started_two = event(PlayEventName::FirstFrame, "edge-a.example");
        started_two.first_frame_ms = Some(1_500);
        let mut failed_manifest = event(PlayEventName::Failure, "edge-b.example");
        failed_manifest.failure = Some(ErrorClass::Manifest);

        store
            .health(&fold(
                "tv",
                day(0),
                &[
                    failed_seg,
                    failed_manifest,
                    event(PlayEventName::Ended, "edge-b.example"),
                    started_two,
                ],
            ))
            .await;

        store
            .health(&fold(
                "web",
                day(1),
                &[event(PlayEventName::Ended, "edge-a.example")],
            ))
            .await;

        let all = store.health_since(day(0)).await;
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].day, day(1));
        assert_eq!(all[0].class, "web");
        assert_eq!(all[0].ended_without_picture(), 1);

        let tv_only = store.health_for(day(0), Some("tv"), None).await;
        assert_eq!(tv_only.len(), 2);

        let edge_a = store
            .health_for(day(0), Some("tv"), Some("edge-a.example"))
            .await;
        assert_eq!(edge_a.len(), 1);
        assert_eq!(edge_a[0].tries, 2);
        assert_eq!(edge_a[0].starts, 2);
        assert_eq!(edge_a[0].first_frame_ms_sum(), 2_400);
        assert_eq!(edge_a[0].mean_start_ms(), Some(1_200));
        assert_eq!(edge_a[0].rebuffers, 1);
        assert_eq!(edge_a[0].ended_without_picture(), 0);

        let edge_b = store
            .health_for(day(0), Some("tv"), Some("edge-b.example"))
            .await;
        assert_eq!(edge_b.len(), 1);
        assert_eq!(edge_b[0].tries, 2);
        assert_eq!(edge_b[0].starts, 0);
        assert_eq!(edge_b[0].ended_without_picture(), 1);
        assert_eq!(
            edge_b[0].failures,
            vec![("manifest".to_owned(), 1), ("segment".to_owned(), 2)]
        );
        assert_eq!(edge_b[0].failures_total(), 3);

        let recent = store.health_since(day(1)).await;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].day, day(1));

        let summary = store.health_summary(day(0)).await;
        assert_eq!(summary.len(), 3);
    }
}
