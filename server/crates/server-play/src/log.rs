//! What devices report: play events and error reports, kept 30 days.

use std::sync::Mutex;

use async_trait::async_trait;
use contract::play::{PlayEvent, Report};
use server_accounts::ProfileId;
use sqlx::PgPool;
use time::OffsetDateTime;

/// Where reports go. A failed write loses a report and never a viewer's request.
#[async_trait]
pub trait PlayLog: Send + Sync {
    /// Keep what happened during play.
    async fn events(&self, profile: ProfileId, events: &[PlayEvent]);
    /// Keep a client error report.
    async fn report(&self, report: &Report);
    /// Delete reports older than this.
    async fn prune(&self, before_ms: u64) -> u64;
}

/// Reports in memory, for tests.
#[derive(Debug, Default)]
pub struct MemoryPlayLog {
    pub events: Mutex<Vec<PlayEvent>>,
    pub reports: Mutex<Vec<Report>>,
}

#[async_trait]
impl PlayLog for MemoryPlayLog {
    async fn events(&self, _: ProfileId, events: &[PlayEvent]) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(events);
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
    async fn events(&self, profile: ProfileId, events: &[PlayEvent]) {
        for event in events {
            let body = serde_json::to_value(event).unwrap_or_default();
            let _ = sqlx::query("insert into play_events (profile_id, body) values ($1, $2)")
                .bind(profile.0)
                .bind(body)
                .execute(&self.pool)
                .await;
        }
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
