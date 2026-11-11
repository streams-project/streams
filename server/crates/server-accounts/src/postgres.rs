//! The account store on Postgres. Queries use the run time API: compile time checked queries need
//! a live database to prepare, and none is available where this was written. The tests in
//! `tests/postgres.rs` run these queries and need a real database (`TEST_DATABASE_URL`).

use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Transaction};
use time::OffsetDateTime;

use crate::pin::{self, PinCheck, PinOutcome, PinRow};
use crate::profile::{
    AccountId, NewAccount, Profile, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch,
    SessionId,
};
use crate::session::{Consent, DeviceSession};
use crate::store::{
    AccountNames, AccountStore, DeviceCard, DeviceCodeRow, StoreError, StoreResult,
};
use crate::token::TokenHash;

/// Accounts on one pool.
#[derive(Clone, Debug)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    /// A store on an open pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn fail(error: sqlx::Error) -> StoreError {
    match &error {
        sqlx::Error::RowNotFound => StoreError::NotFound,
        sqlx::Error::Database(db) => match db.constraint() {
            Some("profiles_user_name_key") => StoreError::NameTaken,
            Some("profile_settings_game_handle_key") => StoreError::HandleTaken,
            Some("profiles_main_not_kids") => StoreError::Rule("main_profile_kids"),
            _ => StoreError::Unavailable,
        },
        _ => StoreError::Unavailable,
    }
}

fn to_time(ms: u64) -> OffsetDateTime {
    let nanos = i128::from(ms) * 1_000_000;
    OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

fn to_ms(at: OffsetDateTime) -> u64 {
    u64::try_from(at.unix_timestamp_nanos() / 1_000_000).unwrap_or(0)
}

fn hash_of(bytes: Vec<u8>) -> StoreResult<TokenHash> {
    <[u8; 32]>::try_from(bytes)
        .map(TokenHash)
        .map_err(|_| StoreError::Unavailable)
}

fn session_row(row: &PgRow) -> StoreResult<DeviceSession> {
    let get_time = |name: &str| -> StoreResult<u64> {
        row.try_get::<OffsetDateTime, _>(name)
            .map(to_ms)
            .map_err(fail)
    };
    let opt_time = |name: &str| -> StoreResult<Option<u64>> {
        row.try_get::<Option<OffsetDateTime>, _>(name)
            .map(|at| at.map(to_ms))
            .map_err(fail)
    };
    Ok(DeviceSession {
        id: SessionId(row.try_get("id").map_err(fail)?),
        account: AccountId(row.try_get("user_id").map_err(fail)?),
        device_name: row.try_get("device_name").map_err(fail)?,
        access_hash: hash_of(row.try_get("access_hash").map_err(fail)?)?,
        access_expires_ms: get_time("access_expires_at")?,
        refresh_current: hash_of(row.try_get("refresh_current").map_err(fail)?)?,
        refresh_next: hash_of(row.try_get("refresh_next").map_err(fail)?)?,
        refresh_salt: row.try_get("refresh_salt").map_err(fail)?,
        refresh_first_used_ms: opt_time("refresh_first_used_at")?,
        active_profile: row
            .try_get::<Option<uuid::Uuid>, _>("active_profile")
            .map_err(fail)?
            .map(ProfileId),
        entered_at_ms: opt_time("entered_at")?,
        step_up_at_ms: opt_time("step_up_at")?,
        step_up_method: row.try_get("step_up_method").map_err(fail)?,
        consent: Consent {
            personal: row.try_get("consent_personal").map_err(fail)?,
        },
        last_seen_ms: get_time("last_seen_at")?,
        created_ms: get_time("created_at")?,
    })
}

const SESSION_COLUMNS: &str = "id, user_id, device_name, access_hash, access_expires_at, \
    refresh_current, refresh_next, refresh_salt, refresh_first_used_at, \
    active_profile, entered_at, step_up_at, step_up_method, consent_personal, last_seen_at, \
    created_at";

const PROFILE_SELECT: &str = "select p.id, p.user_id, p.name, p.avatar, p.kind, p.is_main, \
    p.auto_kids, p.has_pin, p.sort_order, p.maturity, coalesce(l.age_limit, '') as age_limit, \
    coalesce(l.blocked_names, '[]'::jsonb) as blocked_names \
    from profiles p left join profile_limits l on l.profile_id = p.id";

fn profile_row(row: &PgRow) -> StoreResult<Profile> {
    let kind: String = row.try_get("kind").map_err(fail)?;
    let blocked: serde_json::Value = row.try_get("blocked_names").map_err(fail)?;
    Ok(Profile {
        id: ProfileId(row.try_get("id").map_err(fail)?),
        account: AccountId(row.try_get("user_id").map_err(fail)?),
        name: row.try_get("name").map_err(fail)?,
        avatar: row.try_get("avatar").map_err(fail)?,
        kind: ProfileKind::from_word(&kind),
        is_main: row.try_get("is_main").map_err(fail)?,
        auto_kids: row.try_get("auto_kids").map_err(fail)?,
        has_pin: row.try_get("has_pin").map_err(fail)?,
        sort_order: row.try_get("sort_order").map_err(fail)?,
        age_limit: row.try_get("age_limit").map_err(fail)?,
        maturity: row.try_get("maturity").map_err(fail)?,
        blocked_names: serde_json::from_value(blocked).unwrap_or_default(),
    })
}

async fn insert_profile_row(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    profile: &Profile,
) -> StoreResult<()> {
    sqlx::query(
        "insert into profiles (id, user_id, name, avatar, kind, maturity, has_pin, is_main, \
         auto_kids, sort_order) values ($1, $2, $3, $4, $5, $6, false, $7, $8, $9)",
    )
    .bind(profile.id.0)
    .bind(profile.account.0)
    .bind(&profile.name)
    .bind(&profile.avatar)
    .bind(profile.kind.word())
    .bind(&profile.maturity)
    .bind(profile.is_main)
    .bind(profile.auto_kids)
    .bind(profile.sort_order)
    .execute(&mut **tx)
    .await
    .map_err(fail)?;
    sqlx::query("insert into profile_limits (profile_id, age_limit) values ($1, $2)")
        .bind(profile.id.0)
        .bind(&profile.age_limit)
        .execute(&mut **tx)
        .await
        .map_err(fail)?;
    Ok(())
}

fn device_row(row: &PgRow) -> Result<DeviceCodeRow, sqlx::Error> {
    let hash: Vec<u8> = row.try_get("device_hash")?;
    let bytes: [u8; 32] = hash.try_into().map_err(|_| sqlx::Error::RowNotFound)?;
    Ok(DeviceCodeRow {
        device_hash: TokenHash(bytes),
        user_code: row.try_get("user_code")?,
        device_name: row.try_get("device_name")?,
        account: row
            .try_get::<Option<uuid::Uuid>, _>("user_id")?
            .map(AccountId),
        denied: row.try_get("denied")?,
        last_poll_ms: row
            .try_get::<Option<OffsetDateTime>, _>("last_poll")?
            .map(to_ms),
        expires_ms: to_ms(row.try_get("expires_at")?),
    })
}

#[async_trait]
impl AccountStore for PgStore {
    async fn create_account(&self, account: &NewAccount, profiles: &[Profile]) -> StoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        sqlx::query(
            "insert into account_details (user_id, first_name, last_name) values ($1, $2, $3) \
             on conflict (user_id) do nothing",
        )
        .bind(account.id.0)
        .bind(&account.first_name)
        .bind(&account.last_name)
        .execute(&mut *tx)
        .await
        .map_err(fail)?;
        for profile in profiles {
            insert_profile_row(&mut tx, profile).await?;
        }
        tx.commit().await.map_err(fail)
    }

    async fn insert_session(&self, s: &DeviceSession) -> StoreResult<()> {
        sqlx::query(
            "insert into device_sessions (id, user_id, device_name, access_hash, \
             access_expires_at, refresh_current, refresh_next, refresh_salt, step_up_at, \
             step_up_method, last_seen_at, created_at) \
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(s.id.0)
        .bind(s.account.0)
        .bind(&s.device_name)
        .bind(s.access_hash.0.as_slice())
        .bind(to_time(s.access_expires_ms))
        .bind(s.refresh_current.0.as_slice())
        .bind(s.refresh_next.0.as_slice())
        .bind(&s.refresh_salt)
        .bind(s.step_up_at_ms.map(to_time))
        .bind(s.step_up_method.as_deref())
        .bind(to_time(s.last_seen_ms))
        .bind(to_time(s.created_ms))
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn session_by_access(&self, hash: &TokenHash) -> StoreResult<Option<DeviceSession>> {
        let sql = format!("select {SESSION_COLUMNS} from device_sessions where access_hash = $1");
        let row = sqlx::query(&sql)
            .bind(hash.0.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(fail)?;
        row.as_ref().map(session_row).transpose()
    }

    async fn session(&self, id: SessionId) -> StoreResult<Option<DeviceSession>> {
        let sql = format!("select {SESSION_COLUMNS} from device_sessions where id = $1");
        let row = sqlx::query(&sql)
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(fail)?;
        row.as_ref().map(session_row).transpose()
    }

    async fn update_tokens(
        &self,
        s: &DeviceSession,
        from_current: &TokenHash,
        from_next: &TokenHash,
    ) -> StoreResult<bool> {
        // The two hashes that were read are part of the where, so a refresh that read the row
        // before another one finished writes nothing instead of putting the old pair back.
        let done = sqlx::query(
            "update device_sessions set access_hash = $2, access_expires_at = $3, \
             refresh_current = $4, refresh_next = $5, refresh_first_used_at = $6, \
             last_seen_at = $7 where id = $1 \
             and refresh_current = $8 and refresh_next = $9",
        )
        .bind(s.id.0)
        .bind(s.access_hash.0.as_slice())
        .bind(to_time(s.access_expires_ms))
        .bind(s.refresh_current.0.as_slice())
        .bind(s.refresh_next.0.as_slice())
        .bind(s.refresh_first_used_ms.map(to_time))
        .bind(to_time(s.last_seen_ms))
        .bind(from_current.0.as_slice())
        .bind(from_next.0.as_slice())
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        Ok(done.rows_affected() == 1)
    }

    async fn set_active_profile(
        &self,
        session: SessionId,
        profile: ProfileId,
        at_ms: u64,
    ) -> StoreResult<()> {
        sqlx::query(
            "update device_sessions set active_profile = $2, entered_at = $3 where id = $1",
        )
        .bind(session.0)
        .bind(profile.0)
        .bind(to_time(at_ms))
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn set_step_up(&self, session: SessionId, at_ms: u64, method: &str) -> StoreResult<()> {
        sqlx::query(
            "update device_sessions set step_up_at = $2, step_up_method = $3 where id = $1",
        )
        .bind(session.0)
        .bind(to_time(at_ms))
        .bind(method)
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn delete_session(&self, account: AccountId, id: SessionId) -> StoreResult<bool> {
        let done = sqlx::query("delete from device_sessions where id = $1 and user_id = $2")
            .bind(id.0)
            .bind(account.0)
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(done.rows_affected() == 1)
    }

    async fn delete_sessions(&self, account: AccountId) -> StoreResult<()> {
        sqlx::query("delete from device_sessions where user_id = $1")
            .bind(account.0)
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn delete_other_sessions(&self, account: AccountId, keep: SessionId) -> StoreResult<()> {
        sqlx::query("delete from device_sessions where user_id = $1 and id <> $2")
            .bind(account.0)
            .bind(keep.0)
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn sessions_of(&self, account: AccountId) -> StoreResult<Vec<DeviceSession>> {
        let sql = format!(
            "select {SESSION_COLUMNS} from device_sessions where user_id = $1 \
             order by last_seen_at desc limit 50"
        );
        let rows = sqlx::query(&sql)
            .bind(account.0)
            .fetch_all(&self.pool)
            .await
            .map_err(fail)?;
        rows.iter().map(session_row).collect()
    }

    async fn profile(&self, account: AccountId, id: ProfileId) -> StoreResult<Option<Profile>> {
        let sql = format!("{PROFILE_SELECT} where p.id = $1 and p.user_id = $2");
        let row = sqlx::query(&sql)
            .bind(id.0)
            .bind(account.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(fail)?;
        row.as_ref().map(profile_row).transpose()
    }

    async fn profiles_of(&self, account: AccountId) -> StoreResult<Vec<Profile>> {
        let sql =
            format!("{PROFILE_SELECT} where p.user_id = $1 order by p.sort_order, p.created_at");
        let rows = sqlx::query(&sql)
            .bind(account.0)
            .fetch_all(&self.pool)
            .await
            .map_err(fail)?;
        rows.iter().map(profile_row).collect()
    }

    async fn insert_profile(&self, profile: &Profile, max: u32) -> StoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        // Two parallel inserts for one account must not both pass the count.
        sqlx::query("select pg_advisory_xact_lock(hashtextextended($1::text, 0))")
            .bind(profile.account.0)
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        let count: i64 = sqlx::query_scalar("select count(*) from profiles where user_id = $1")
            .bind(profile.account.0)
            .fetch_one(&mut *tx)
            .await
            .map_err(fail)?;
        if count >= i64::from(max) {
            return Err(StoreError::LimitReached);
        }
        insert_profile_row(&mut tx, profile).await?;
        tx.commit().await.map_err(fail)
    }

    async fn update_profile(
        &self,
        account: AccountId,
        id: ProfileId,
        patch: &ProfilePatch,
    ) -> StoreResult<Profile> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        let maturity = patch.kind.map(|kind| match kind {
            ProfileKind::Kids => "older_kids",
            ProfileKind::Adult => "adult",
        });
        let done = sqlx::query(
            "update profiles set name = coalesce($3, name), avatar = coalesce($4, avatar), \
             kind = coalesce($5, kind), maturity = coalesce($6, maturity), updated_at = now() \
             where id = $1 and user_id = $2",
        )
        .bind(id.0)
        .bind(account.0)
        .bind(patch.name.as_deref())
        .bind(patch.avatar.as_deref())
        .bind(patch.kind.map(ProfileKind::word))
        .bind(maturity)
        .execute(&mut *tx)
        .await
        .map_err(fail)?;
        if done.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        if patch.kind == Some(ProfileKind::Kids) {
            sqlx::query("update profile_limits set age_limit = 'PG' where profile_id = $1")
                .bind(id.0)
                .execute(&mut *tx)
                .await
                .map_err(fail)?;
        }
        tx.commit().await.map_err(fail)?;
        self.profile(account, id).await?.ok_or(StoreError::NotFound)
    }

    async fn delete_profile(&self, account: AccountId, id: ProfileId) -> StoreResult<()> {
        // `active_profile` is `on delete set null`, so the picker returns on every session.
        let done = sqlx::query("delete from profiles where id = $1 and user_id = $2")
            .bind(id.0)
            .bind(account.0)
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        if done.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn set_main_profile(&self, account: AccountId, id: ProfileId) -> StoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        sqlx::query(
            "update profiles set is_main = false where user_id = $1 and is_main and id <> $2",
        )
        .bind(account.0)
        .bind(id.0)
        .execute(&mut *tx)
        .await
        .map_err(fail)?;
        sqlx::query("update profiles set is_main = true where id = $1 and user_id = $2")
            .bind(id.0)
            .bind(account.0)
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        tx.commit().await.map_err(fail)
    }

    async fn pin_attempt(
        &self,
        profile: ProfileId,
        check: PinCheck<'_>,
    ) -> StoreResult<PinOutcome> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        let row = sqlx::query(
            "select pin_hash, failed_attempts, lock_count, locked_until from profile_pins \
             where profile_id = $1 for update",
        )
        .bind(profile.0)
        .fetch_optional(&mut *tx)
        .await
        .map_err(fail)?;
        let Some(row) = row else {
            return Ok(PinOutcome::Pass);
        };
        let current = PinRow {
            hash: row.try_get("pin_hash").map_err(fail)?,
            failed: row
                .try_get::<i16, _>("failed_attempts")
                .map_err(fail)?
                .unsigned_abs(),
            locks: row
                .try_get::<i16, _>("lock_count")
                .map_err(fail)?
                .unsigned_abs(),
            locked_until_ms: row
                .try_get::<Option<OffsetDateTime>, _>("locked_until")
                .map_err(fail)?
                .map(to_ms),
        };
        let (outcome, next) = pin::decide(&current, check);
        if let Some(next) = next {
            sqlx::query(
                "update profile_pins set failed_attempts = $2, lock_count = $3, locked_until = $4 \
                 where profile_id = $1",
            )
            .bind(profile.0)
            .bind(i16::try_from(next.failed).unwrap_or(i16::MAX))
            .bind(i16::try_from(next.locks).unwrap_or(i16::MAX))
            .bind(next.locked_until_ms.map(to_time))
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
            if next.locked_until_ms.is_some() && next.locked_until_ms != current.locked_until_ms {
                sqlx::query(
                    "insert into account_events (user_id, event) \
                     select user_id, 'pin_locked' from profiles where id = $1",
                )
                .bind(profile.0)
                .execute(&mut *tx)
                .await
                .map_err(fail)?;
            }
        }
        tx.commit().await.map_err(fail)?;
        Ok(outcome)
    }

    async fn set_pin(&self, profile: ProfileId, hash: Option<String>) -> StoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        match &hash {
            Some(hash) => {
                sqlx::query(
                    "insert into profile_pins (profile_id, pin_hash) values ($1, $2) \
                     on conflict (profile_id) do update set pin_hash = excluded.pin_hash, \
                     failed_attempts = 0, lock_count = 0, locked_until = null, updated_at = now()",
                )
                .bind(profile.0)
                .bind(hash)
                .execute(&mut *tx)
                .await
                .map_err(fail)?;
            }
            None => {
                sqlx::query("delete from profile_pins where profile_id = $1")
                    .bind(profile.0)
                    .execute(&mut *tx)
                    .await
                    .map_err(fail)?;
            }
        }
        sqlx::query("update profiles set has_pin = $2 where id = $1")
            .bind(profile.0)
            .bind(hash.is_some())
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        tx.commit().await.map_err(fail)
    }

    async fn set_limits(
        &self,
        account: AccountId,
        profile: ProfileId,
        limits: &ProfileLimitsRow,
    ) -> StoreResult<()> {
        let blocked =
            serde_json::to_value(&limits.blocked_names).map_err(|_| StoreError::Unavailable)?;
        let done = sqlx::query(
            "update profile_limits l set age_limit = $3, blocked_names = $4, updated_at = now() \
             from profiles p where l.profile_id = p.id and p.id = $1 and p.user_id = $2",
        )
        .bind(profile.0)
        .bind(account.0)
        .bind(&limits.age_limit)
        .bind(blocked)
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        if done.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn set_consent(&self, session: SessionId, consent: Consent) -> StoreResult<()> {
        sqlx::query("update device_sessions set consent_personal = $2 where id = $1")
            .bind(session.0)
            .bind(consent.personal)
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn log_event(&self, account: AccountId, event: &str) -> StoreResult<()> {
        sqlx::query("insert into account_events (user_id, event) values ($1, $2)")
            .bind(account.0)
            .bind(event)
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn events_of(&self, account: AccountId, limit: usize) -> StoreResult<Vec<String>> {
        let rows = sqlx::query(
            "select event from account_events where user_id = $1 order by ts desc limit $2",
        )
        .bind(account.0)
        .bind(i64::try_from(limit).unwrap_or(i64::MAX))
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        rows.iter()
            .map(|row| row.try_get("event").map_err(fail))
            .collect()
    }

    async fn names(&self, account: AccountId) -> StoreResult<AccountNames> {
        let row =
            sqlx::query("select first_name, last_name from account_details where user_id = $1")
                .bind(account.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(fail)?;
        let Some(row) = row else {
            return Ok(AccountNames::default());
        };
        Ok(AccountNames {
            first_name: row.try_get("first_name").map_err(fail)?,
            last_name: row.try_get("last_name").map_err(fail)?,
        })
    }

    async fn profile_transfer_allowed(&self, account: AccountId) -> StoreResult<bool> {
        let row = sqlx::query("select profile_transfer from account_details where user_id = $1")
            .bind(account.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(fail)?;
        row.map_or(Ok(true), |row| {
            row.try_get("profile_transfer").map_err(fail)
        })
    }

    async fn set_names(
        &self,
        account: AccountId,
        names: &AccountNames,
        main: Option<(ProfileId, String)>,
    ) -> StoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        sqlx::query(
            "insert into account_details (user_id, first_name, last_name) values ($1, $2, $3) \
             on conflict (user_id) do update set first_name = excluded.first_name, \
             last_name = excluded.last_name, updated_at = now()",
        )
        .bind(account.0)
        .bind(&names.first_name)
        .bind(&names.last_name)
        .execute(&mut *tx)
        .await
        .map_err(fail)?;
        if let Some((id, name)) = main {
            sqlx::query(
                "update profiles set name = $3, updated_at = now() where id = $2 and user_id = $1",
            )
            .bind(account.0)
            .bind(id.0)
            .bind(name)
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        }
        tx.commit().await.map_err(fail)
    }

    async fn delete_account(&self, account: AccountId) -> StoreResult<()> {
        sqlx::query("delete from account_details where user_id = $1")
            .bind(account.0)
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn move_profile(
        &self,
        from: AccountId,
        profile: ProfileId,
        to: AccountId,
        max: u32,
    ) -> StoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        sqlx::query("select pg_advisory_xact_lock(hashtextextended($1::text, 0))")
            .bind(to.0.to_string())
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        let held: i64 = sqlx::query("select count(*) as held from profiles where user_id = $1")
            .bind(to.0)
            .fetch_one(&mut *tx)
            .await
            .map_err(fail)?
            .try_get("held")
            .map_err(fail)?;
        if held >= i64::from(max) {
            return Err(StoreError::LimitReached);
        }
        // The PIN belongs to the account that set it, and only one profile an account may hold is
        // the automatic Kids one, so both go with the same statement that moves the row. A refusal
        // past here rolls the whole transaction back and the profile keeps its PIN.
        let moved = sqlx::query(
            "update profiles set user_id = $3, is_main = false, auto_kids = false, \
             has_pin = false where id = $2 and user_id = $1",
        )
        .bind(from.0)
        .bind(profile.0)
        .bind(to.0)
        .execute(&mut *tx)
        .await
        .map_err(fail)?;
        if moved.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        sqlx::query("delete from profile_pins where profile_id = $1")
            .bind(profile.0)
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        sqlx::query("update device_sessions set active_profile = null where active_profile = $1")
            .bind(profile.0)
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        tx.commit().await.map_err(fail)
    }

    async fn insert_device_code(&self, row: &DeviceCodeRow) -> StoreResult<()> {
        sqlx::query(
            "insert into device_codes (device_hash, user_code, device_name, expires_at) \
             values ($1, $2, $3, $4)",
        )
        .bind(row.device_hash.0.as_slice())
        .bind(&row.user_code)
        .bind(&row.device_name)
        .bind(to_time(row.expires_ms))
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn device_code(&self, hash: &TokenHash) -> StoreResult<Option<DeviceCodeRow>> {
        let row = sqlx::query("select * from device_codes where device_hash = $1")
            .bind(hash.0.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(fail)?;
        row.as_ref().map(device_row).transpose().map_err(fail)
    }

    async fn device_code_by_user(&self, user_code: &str) -> StoreResult<Option<DeviceCodeRow>> {
        let row = sqlx::query("select * from device_codes where user_code = $1")
            .bind(user_code)
            .fetch_optional(&self.pool)
            .await
            .map_err(fail)?;
        row.as_ref().map(device_row).transpose().map_err(fail)
    }

    async fn note_device_poll(&self, hash: &TokenHash, at_ms: u64) -> StoreResult<()> {
        sqlx::query("update device_codes set last_poll = $2 where device_hash = $1")
            .bind(hash.0.as_slice())
            .bind(to_time(at_ms))
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn approve_device_code(&self, hash: &TokenHash, account: AccountId) -> StoreResult<bool> {
        let done = sqlx::query(
            "update device_codes set user_id = $2 \
             where device_hash = $1 and user_id is null and not denied",
        )
        .bind(hash.0.as_slice())
        .bind(account.0)
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        Ok(done.rows_affected() == 1)
    }

    async fn deny_device_code(&self, hash: &TokenHash) -> StoreResult<bool> {
        let done = sqlx::query(
            "update device_codes set denied = true \
             where device_hash = $1 and user_id is null and not denied",
        )
        .bind(hash.0.as_slice())
        .execute(&self.pool)
        .await
        .map_err(fail)?;
        Ok(done.rows_affected() == 1)
    }

    async fn delete_device_code(&self, hash: &TokenHash) -> StoreResult<bool> {
        // One statement decides who took the row, so two polls on one approved code cannot both
        // go on to open a session.
        let done = sqlx::query("delete from device_codes where device_hash = $1")
            .bind(hash.0.as_slice())
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(done.rows_affected() == 1)
    }

    async fn prune(
        &self,
        idle_before_ms: u64,
        events_before_ms: u64,
        now_ms: u64,
    ) -> StoreResult<u64> {
        let mut gone = 0;
        for (statement, time) in [
            (
                "delete from device_sessions where last_seen_at < $1",
                idle_before_ms,
            ),
            ("delete from account_events where ts < $1", events_before_ms),
            ("delete from device_codes where expires_at <= $1", now_ms),
        ] {
            let done = sqlx::query(statement)
                .bind(to_time(time))
                .execute(&self.pool)
                .await
                .map_err(fail)?;
            gone += done.rows_affected();
        }
        Ok(gone)
    }

    async fn card(&self, session: SessionId) -> StoreResult<Option<serde_json::Value>> {
        let found: Option<Option<serde_json::Value>> =
            sqlx::query_scalar("select card from device_sessions where id = $1")
                .bind(session.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(fail)?;
        Ok(found.flatten())
    }

    async fn save_card(&self, seen: &DeviceCard<'_>) -> StoreResult<bool> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        let known: Option<bool> = sqlx::query_scalar(
            "select card is not null from device_sessions where id = $1 and card_hash = $2",
        )
        .bind(seen.session.0)
        .bind(seen.card_hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(fail)?;
        let known = known.unwrap_or(false);
        if !known {
            sqlx::query("update device_sessions set card_hash = $2, card = $3 where id = $1")
                .bind(seen.session.0)
                .bind(seen.card_hash)
                .bind(seen.card)
                .execute(&mut *tx)
                .await
                .map_err(fail)?;
        }
        // The key the device keeps for itself names it across sessions, so each start writes the
        // device row as well as the session's card.
        sqlx::query(
            "insert into account_devices (user_id, device_key, session_id) values ($1, $2, $3) \
             on conflict (user_id, device_key) do update \
             set session_id = excluded.session_id, last_seen = now()",
        )
        .bind(seen.account.0)
        .bind(seen.device_key)
        .bind(seen.session.0)
        .execute(&mut *tx)
        .await
        .map_err(fail)?;
        tx.commit().await.map_err(fail)?;
        Ok(!known && seen.card.is_none())
    }
}
