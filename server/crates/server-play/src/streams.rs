//! How many streams one account may play at once.
//!
//! The count is not held in this process. [`Play`](crate::Play) already has a `moka` cache of
//! plays in progress (`gate.rs`), and that cache is the wrong place to count from: it is per
//! process, so the moment a second server answers `/v1/play` each one would count only its own
//! half and an account would get its allowance once per server. Every number this module reads is
//! read back out of a store that both servers share, and the Postgres store reads the time from
//! the database as well, so two servers whose clocks disagree still agree on what is live.
//!
//! One row per (session, profile), not per play: a device plays one thing at a time, so a viewer
//! who changes title takes their own row again and never a second slot. Nothing here caps how
//! many devices an account may register; it caps only what is playing at this moment.
//!
//! A row that stops being heard from stops counting after [`STREAM_STALE_S`]. That is the half of
//! this that matters: a counter that only ever goes up locks a viewer out of their own account
//! the first time a device loses power mid-film.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use server_accounts::{AccountId, ProfileId, SessionId};
use sqlx::PgPool;
use uuid::Uuid;

/// How many streams one account may play at once when the operator sets no other number.
pub const STREAMS_MAX_DEFAULT: u32 = 2;

/// A stream nothing has been heard from for this long stops counting.
///
/// The core sends a position while a play runs every `HEARTBEAT_MS`, which is 30 s
/// (`platform/core/src/player.rs:17`), so this is five missed beats.
pub const STREAM_STALE_S: u32 = 150;

/// Which stream. The account is what the limit is about; the session and profile are which one of
/// that account's streams this is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamKey {
    pub account: AccountId,
    pub session: SessionId,
    pub profile: ProfileId,
}

/// What asking for a slot came to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Claim {
    /// The slot is this stream's.
    Taken,
    /// The account is already playing this many, which is all it may.
    Full { live: u32 },
}

/// Where the live streams of an account are counted. A store every server reaches, never a field
/// of one server.
#[async_trait]
pub trait StreamStore: Send + Sync {
    /// Take a slot for this stream, or refuse. Taking again for a key that already holds one
    /// replaces it, so a second play on one device is not a second stream.
    async fn claim(&self, key: StreamKey, play_id: &str, max: u32) -> Claim;
    /// This stream is still playing. A beat for a play id the row no longer holds does nothing.
    async fn beat(&self, key: StreamKey, play_id: &str);
    /// This stream is over, so its slot goes now rather than when it goes stale.
    async fn release(&self, key: StreamKey, play_id: &str);
    /// The streams of this account heard from inside the window.
    async fn live(&self, account: AccountId) -> u32;
}

/// One held slot.
#[derive(Clone, Debug)]
struct Row {
    account: Uuid,
    play_id: String,
    fresh_ms: u64,
}

/// Slots in memory, for tests and for a single server with no database.
///
/// Share one of these between two [`Play`](crate::Play) gates and they count as one account, the
/// same way two servers on one database do. Its clock is set by hand, so a test that proves a
/// stream goes stale needs no sleep.
#[derive(Debug)]
pub struct MemoryStreamStore {
    rows: Mutex<HashMap<(Uuid, Uuid), Row>>,
    now_ms: AtomicU64,
    stale_ms: u64,
}

impl Default for MemoryStreamStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStreamStore {
    /// A store with the usual window.
    #[must_use]
    pub fn new() -> Self {
        Self::with_stale_s(STREAM_STALE_S)
    }

    /// A store whose window is this many seconds.
    #[must_use]
    pub fn with_stale_s(stale_s: u32) -> Self {
        Self {
            rows: Mutex::new(HashMap::new()),
            now_ms: AtomicU64::new(0),
            stale_ms: u64::from(stale_s) * 1000,
        }
    }

    /// The time this store believes it is.
    #[must_use]
    pub fn now_ms(&self) -> u64 {
        self.now_ms.load(Ordering::SeqCst)
    }

    /// Move this store's clock on.
    pub fn advance_ms(&self, by_ms: u64) {
        self.now_ms.fetch_add(by_ms, Ordering::SeqCst);
    }

    fn held(&self) -> std::sync::MutexGuard<'_, HashMap<(Uuid, Uuid), Row>> {
        self.rows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Rows of this account that went stale are dropped, and the rest counted.
    fn sweep(
        &self,
        rows: &mut HashMap<(Uuid, Uuid), Row>,
        account: Uuid,
        except: Option<(Uuid, Uuid)>,
    ) -> u32 {
        let now = self.now_ms();
        rows.retain(|_, row| row.account != account || row.fresh_ms + self.stale_ms > now);
        let live = rows
            .iter()
            .filter(|(key, row)| row.account == account && except != Some(**key))
            .count();
        u32::try_from(live).unwrap_or(u32::MAX)
    }
}

fn pair(key: StreamKey) -> (Uuid, Uuid) {
    (key.session.0, key.profile.0)
}

#[async_trait]
impl StreamStore for MemoryStreamStore {
    async fn claim(&self, key: StreamKey, play_id: &str, max: u32) -> Claim {
        let mut rows = self.held();
        let mine = pair(key);
        let live = self.sweep(&mut rows, key.account.0, Some(mine));
        if live >= max {
            return Claim::Full { live };
        }
        rows.insert(
            mine,
            Row {
                account: key.account.0,
                play_id: play_id.to_owned(),
                fresh_ms: self.now_ms(),
            },
        );
        Claim::Taken
    }

    async fn beat(&self, key: StreamKey, play_id: &str) {
        let now = self.now_ms();
        let mut rows = self.held();
        if let Some(row) = rows.get_mut(&pair(key))
            && row.play_id == play_id
        {
            row.fresh_ms = now;
        }
    }

    async fn release(&self, key: StreamKey, play_id: &str) {
        let mut rows = self.held();
        let mine = pair(key);
        if rows.get(&mine).is_some_and(|row| row.play_id == play_id) {
            rows.remove(&mine);
        }
    }

    async fn live(&self, account: AccountId) -> u32 {
        let mut rows = self.held();
        self.sweep(&mut rows, account.0, None)
    }
}

/// Slots on Postgres: the store every server of a deployment shares.
///
/// Three things make the count right under more than one server. The rows are in the database, so
/// no server holds a count of its own. `now()` is the database's clock, so a server whose clock
/// is wrong cannot make a stream look fresh or stale. And a claim takes an advisory lock on the
/// account for its transaction, so two servers claiming for one account at the same moment are
/// serialised and the second sees the first's row; without it, under the default isolation, both
/// would count the other's uncommitted row as absent and both would be let in.
#[derive(Clone, Debug)]
pub struct PgStreamStore {
    pool: PgPool,
    stale_s: f64,
}

impl PgStreamStore {
    /// A store on an open pool, with the usual window.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self::with_stale_s(pool, STREAM_STALE_S)
    }

    /// A store whose window is this many seconds.
    #[must_use]
    pub fn with_stale_s(pool: PgPool, stale_s: u32) -> Self {
        Self {
            pool,
            stale_s: f64::from(stale_s),
        }
    }
}

#[async_trait]
impl StreamStore for PgStreamStore {
    /// A store that cannot be reached lets the play through. A viewer losing playback because a
    /// database blinked is a worse failure than an account briefly playing one stream too many,
    /// and the stale window closes the second one on its own.
    async fn claim(&self, key: StreamKey, play_id: &str, max: u32) -> Claim {
        let Ok(mut tx) = self.pool.begin().await else {
            return Claim::Taken;
        };
        // Held until this transaction ends, so claims for one account queue and claims for
        // different accounts do not wait for each other.
        if sqlx::query("select pg_advisory_xact_lock(hashtextextended($1::text, 0))")
            .bind(key.account.0)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            return Claim::Taken;
        }
        // The rows this account left behind are swept here, which is the only place that needs
        // them gone, so the limit asks for no prune job of its own.
        let _ = sqlx::query(
            "delete from play_streams \
             where user_id = $1 and fresh_at <= now() - ($2::double precision * interval '1 second')",
        )
        .bind(key.account.0)
        .bind(self.stale_s)
        .execute(&mut *tx)
        .await;
        let counted = sqlx::query_scalar::<_, i64>(
            "select count(*) from play_streams \
             where user_id = $1 and not (session_id = $2 and profile_id = $3)",
        )
        .bind(key.account.0)
        .bind(key.session.0)
        .bind(key.profile.0)
        .fetch_one(&mut *tx)
        .await;
        let Ok(found) = counted else {
            return Claim::Taken;
        };
        let live = u32::try_from(found).unwrap_or(u32::MAX);
        if live >= max {
            // The sweep above is kept even on a refusal, so stale rows go once rather than on
            // every attempt.
            let _ = tx.commit().await;
            return Claim::Full { live };
        }
        let written = sqlx::query(
            "insert into play_streams (user_id, session_id, profile_id, play_id, fresh_at) \
             values ($1, $2, $3, $4, now()) \
             on conflict (session_id, profile_id) do update set \
             play_id = excluded.play_id, fresh_at = now()",
        )
        .bind(key.account.0)
        .bind(key.session.0)
        .bind(key.profile.0)
        .bind(play_id)
        .execute(&mut *tx)
        .await;
        if written.is_err() {
            return Claim::Taken;
        }
        let _ = tx.commit().await;
        Claim::Taken
    }

    async fn beat(&self, key: StreamKey, play_id: &str) {
        let _ = sqlx::query(
            "update play_streams set fresh_at = now() \
             where session_id = $1 and profile_id = $2 and play_id = $3",
        )
        .bind(key.session.0)
        .bind(key.profile.0)
        .bind(play_id)
        .execute(&self.pool)
        .await;
    }

    async fn release(&self, key: StreamKey, play_id: &str) {
        let _ = sqlx::query(
            "delete from play_streams \
             where session_id = $1 and profile_id = $2 and play_id = $3",
        )
        .bind(key.session.0)
        .bind(key.profile.0)
        .bind(play_id)
        .execute(&self.pool)
        .await;
    }

    async fn live(&self, account: AccountId) -> u32 {
        let counted = sqlx::query_scalar::<_, i64>(
            "select count(*) from play_streams \
             where user_id = $1 and fresh_at > now() - ($2::double precision * interval '1 second')",
        )
        .bind(account.0)
        .bind(self.stale_s)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        u32::try_from(counted).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::{Claim, MemoryStreamStore, STREAM_STALE_S, StreamKey, StreamStore};
    use server_accounts::{AccountId, ProfileId, SessionId};

    fn key(account: AccountId, session: SessionId) -> StreamKey {
        StreamKey {
            account,
            session,
            profile: ProfileId::new(),
        }
    }

    #[tokio::test]
    async fn a_third_stream_is_refused_where_two_are_allowed() {
        let store = MemoryStreamStore::new();
        let account = AccountId::new();
        let one = key(account, SessionId::new());
        let two = key(account, SessionId::new());
        let three = key(account, SessionId::new());
        assert_eq!(store.claim(one, "p1", 2).await, Claim::Taken);
        assert_eq!(store.claim(two, "p2", 2).await, Claim::Taken);
        assert_eq!(store.claim(three, "p3", 2).await, Claim::Full { live: 2 });
        assert_eq!(store.live(account).await, 2);
    }

    #[tokio::test]
    async fn another_account_is_counted_apart() {
        let store = MemoryStreamStore::new();
        let mine = AccountId::new();
        let theirs = AccountId::new();
        assert_eq!(
            store.claim(key(mine, SessionId::new()), "p1", 1).await,
            Claim::Taken
        );
        assert_eq!(
            store.claim(key(theirs, SessionId::new()), "p2", 1).await,
            Claim::Taken
        );
        assert_eq!(store.live(mine).await, 1);
        assert_eq!(store.live(theirs).await, 1);
    }

    #[tokio::test]
    async fn one_device_playing_a_second_title_is_still_one_stream() {
        let store = MemoryStreamStore::new();
        let account = AccountId::new();
        let device = key(account, SessionId::new());
        assert_eq!(store.claim(device, "p1", 1).await, Claim::Taken);
        assert_eq!(store.claim(device, "p2", 1).await, Claim::Taken);
        assert_eq!(store.live(account).await, 1);
    }

    #[tokio::test]
    async fn a_stream_that_stops_beating_stops_counting() {
        let store = MemoryStreamStore::new();
        let account = AccountId::new();
        let one = key(account, SessionId::new());
        let two = key(account, SessionId::new());
        assert_eq!(store.claim(one, "p1", 2).await, Claim::Taken);
        assert_eq!(store.claim(two, "p2", 2).await, Claim::Taken);
        // One keeps beating; two is a device that lost power.
        let stale_ms = u64::from(STREAM_STALE_S) * 1000;
        for _ in 0..4 {
            store.advance_ms(stale_ms / 2);
            store.beat(one, "p1").await;
        }
        assert_eq!(store.live(account).await, 1);
        assert_eq!(
            store.claim(key(account, SessionId::new()), "p3", 2).await,
            Claim::Taken
        );
    }

    #[tokio::test]
    async fn a_beat_for_an_old_play_does_not_hold_the_slot() {
        let store = MemoryStreamStore::new();
        let account = AccountId::new();
        let device = key(account, SessionId::new());
        assert_eq!(store.claim(device, "p1", 2).await, Claim::Taken);
        let stale_ms = u64::from(STREAM_STALE_S) * 1000;
        store.advance_ms(stale_ms / 2);
        store.beat(device, "gone").await;
        store.advance_ms(stale_ms / 2 + 1);
        assert_eq!(store.live(account).await, 0);
    }

    #[tokio::test]
    async fn a_released_stream_frees_its_slot_at_once() {
        let store = MemoryStreamStore::new();
        let account = AccountId::new();
        let device = key(account, SessionId::new());
        assert_eq!(store.claim(device, "p1", 1).await, Claim::Taken);
        assert_eq!(
            store.claim(key(account, SessionId::new()), "p2", 1).await,
            Claim::Full { live: 1 }
        );
        store.release(device, "p1").await;
        assert_eq!(store.live(account).await, 0);
        assert_eq!(
            store.claim(key(account, SessionId::new()), "p3", 1).await,
            Claim::Taken
        );
    }
}
