//! The stream count on a real Postgres, from two pools that do not know about each other.
//!
//! These are the tests the in-memory ones cannot stand in for: that the SQL is valid, that two
//! servers sharing a database share the count, and that two of them claiming at the same moment
//! cannot both be let in. They need a database, so they are `#[ignore]`d, the way `./check`
//! expects of a test that cannot run on a machine with nothing behind it.
//!
//! Run them against a database with the chain of `server/migrations` applied:
//!
//! ```text
//! DATABASE_URL=postgres://user@127.0.0.1:5432/db cargo test -p server-play --test streams_pg \
//!     -- --ignored
//! ```

use std::sync::Arc;

use server_accounts::{AccountId, ProfileId, SessionId};
use server_play::{Claim, PgStreams, StreamKey, StreamStore};
use sqlx::PgPool;
use uuid::Uuid;

/// One account with this many devices signed in, written straight to the tables so the test needs
/// nothing of the door.
struct World {
    account: AccountId,
    profile: ProfileId,
    sessions: Vec<SessionId>,
}

async fn world(pool: &PgPool, devices: usize) -> World {
    let account = AccountId::new();
    let profile = ProfileId::new();
    sqlx::query("insert into auth.users (id, email) values ($1, $2)")
        .bind(account.0)
        .bind(format!("{}@streams.test", account.0))
        .execute(pool)
        .await
        .expect("an account");
    sqlx::query("insert into account_details (user_id) values ($1)")
        .bind(account.0)
        .execute(pool)
        .await
        .expect("account details");
    sqlx::query("insert into profiles (id, user_id, name, is_main) values ($1, $2, 'Main', true)")
        .bind(profile.0)
        .bind(account.0)
        .execute(pool)
        .await
        .expect("a profile");
    let mut sessions = Vec::new();
    for n in 0..devices {
        let id = SessionId::new();
        sqlx::query(
            "insert into device_sessions (id, user_id, device_name, access_hash, \
             access_expires_at, refresh_current, refresh_next, refresh_salt, last_seen_at, \
             created_at) values ($1, $2, $3, $4, now() + interval '1 hour', $5, $6, 'salt', \
             now(), now())",
        )
        .bind(id.0)
        .bind(account.0)
        .bind(format!("device {n}"))
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("a session");
        sessions.push(id);
    }
    World {
        account,
        profile,
        sessions,
    }
}

impl World {
    fn key(&self, device: usize) -> StreamKey {
        StreamKey {
            account: self.account,
            session: self.sessions[device],
            profile: self.profile,
        }
    }
}

/// Two pools on the same database: one per server, as a deployment behind a load balancer has.
async fn two_servers() -> Option<(PgPool, PgPool)> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let first = PgPool::connect(&url)
        .await
        .expect("the first server's pool");
    let second = PgPool::connect(&url)
        .await
        .expect("the second server's pool");
    Some((first, second))
}

#[tokio::test]
#[ignore = "needs a database with server/migrations applied"]
async fn the_count_is_shared_between_two_servers() {
    let Some((first, second)) = two_servers().await else {
        return;
    };
    let world = world(&first, 3).await;
    let one = PgStreams::new(first);
    let two = PgStreams::new(second);
    assert_eq!(one.claim(world.key(0), "p1", 2).await, Claim::Taken);
    assert_eq!(two.claim(world.key(1), "p2", 2).await, Claim::Taken);
    assert_eq!(
        one.claim(world.key(2), "p3", 2).await,
        Claim::Full { live: 2 },
        "the second server's stream must count against the first"
    );
    assert_eq!(one.live(world.account).await, 2);
    assert_eq!(two.live(world.account).await, 2);
}

#[tokio::test]
#[ignore = "needs a database with server/migrations applied"]
async fn a_stream_that_stopped_beating_stops_counting() {
    let Some((first, second)) = two_servers().await else {
        return;
    };
    let world = world(&first, 3).await;
    let one = PgStreams::new(first.clone());
    let two = PgStreams::new(second);
    assert_eq!(one.claim(world.key(0), "p1", 2).await, Claim::Taken);
    assert_eq!(two.claim(world.key(1), "p2", 2).await, Claim::Taken);
    assert_eq!(
        one.claim(world.key(2), "p3", 2).await,
        Claim::Full { live: 2 }
    );
    // The second device loses power: its row stops being written to.
    sqlx::query(
        "update play_streams set fresh_at = now() - interval '10 minutes' where session_id = $1",
    )
    .bind(world.sessions[1].0)
    .execute(&first)
    .await
    .expect("an old row");
    assert_eq!(one.live(world.account).await, 1);
    assert_eq!(
        one.claim(world.key(2), "p3", 2).await,
        Claim::Taken,
        "a dead stream must not hold its slot for ever"
    );
    // And the dead row is gone rather than merely ignored.
    let rows: i64 = sqlx::query_scalar("select count(*) from play_streams where user_id = $1")
        .bind(world.account.0)
        .fetch_one(&first)
        .await
        .expect("a count");
    assert_eq!(rows, 2);
}

#[tokio::test]
#[ignore = "needs a database with server/migrations applied"]
async fn a_beat_keeps_a_slot_and_a_release_frees_it() {
    let Some((first, second)) = two_servers().await else {
        return;
    };
    let world = world(&first, 2).await;
    let one = PgStreams::new(first.clone());
    let two = PgStreams::new(second);
    assert_eq!(one.claim(world.key(0), "p1", 1).await, Claim::Taken);
    sqlx::query(
        "update play_streams set fresh_at = now() - interval '2 minutes' where session_id = $1",
    )
    .bind(world.sessions[0].0)
    .execute(&first)
    .await
    .expect("an older row");
    // A beat from the device that is still playing, seen by the other server.
    two.beat(world.key(0), "p1").await;
    assert_eq!(
        one.claim(world.key(1), "p2", 1).await,
        Claim::Full { live: 1 },
        "a stream that beat is still playing"
    );
    // A beat naming a play that is over does nothing.
    two.beat(world.key(0), "an-old-play").await;
    one.release(world.key(0), "p1").await;
    assert_eq!(one.live(world.account).await, 0);
    assert_eq!(two.claim(world.key(1), "p2", 1).await, Claim::Taken);
}

#[tokio::test]
#[ignore = "needs a database with server/migrations applied"]
async fn eight_claims_at_once_on_two_servers_let_two_through() {
    let Some((first, second)) = two_servers().await else {
        return;
    };
    let world = Arc::new(world(&first, 8).await);
    let one = Arc::new(PgStreams::new(first));
    let two = Arc::new(PgStreams::new(second));
    let mut running = Vec::new();
    for device in 0..8 {
        let store = if device % 2 == 0 {
            Arc::clone(&one)
        } else {
            Arc::clone(&two)
        };
        let world = Arc::clone(&world);
        running.push(tokio::spawn(async move {
            store
                .claim(world.key(device), &format!("p{device}"), 2)
                .await
        }));
    }
    let mut taken = 0;
    for task in running {
        if task.await.expect("a claim") == Claim::Taken {
            taken += 1;
        }
    }
    assert_eq!(
        taken, 2,
        "eight claims racing on two servers must leave exactly the two slots taken"
    );
    assert_eq!(one.live(world.account).await, 2);
}
