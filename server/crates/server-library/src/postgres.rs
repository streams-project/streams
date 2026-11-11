//! The library on Postgres. Queries use the run time API, as `server-accounts` does: compile time
//! checks need a live database to prepare, and none is available where this was written. The
//! tests in `tests/postgres.rs` run these queries and need a real database.

use async_trait::async_trait;
use contract::command::{Change, FeedbackEvent, Outcome, SettingKey};
use contract::{EpisodeRef, Rating, TitleKey, TitleKind};
use server_accounts::ProfileId;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row as _, Transaction};
use time::OffsetDateTime;

use crate::apply::{progress_change, title_change};
use crate::model::{
    EpisodeProgress, FeedRow, HiddenTitle, NotificationReadState, TitleReminder, TitleState,
};
use crate::reads::SettingsView;
use crate::store::{LibraryError, LibraryRead, LibraryStore, LibraryTx, Row};
use crate::taste::Taste;

/// Rows of refused trailers kept per profile.
const REFUSED_KEPT: i64 = 300;

/// The library on one pool.
#[derive(Clone, Debug)]
pub struct PgLibrary {
    pool: PgPool,
}

impl PgLibrary {
    /// A store on an open pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn fail(_: sqlx::Error) -> LibraryError {
    LibraryError::Unavailable
}

fn to_time(ms: u64) -> OffsetDateTime {
    let nanos = i128::from(ms) * 1_000_000;
    OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

fn to_ms(time: OffsetDateTime) -> u64 {
    u64::try_from(time.unix_timestamp_nanos() / 1_000_000).unwrap_or(0)
}

fn opt_ms(time: Option<OffsetDateTime>) -> u64 {
    time.map_or(0, to_ms)
}

fn big(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn wide(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

fn small(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn narrow(value: i32) -> u32 {
    u32::try_from(value).unwrap_or(0)
}

fn kind_word(title: TitleKey) -> &'static str {
    title.kind().word()
}

fn id_of(title: TitleKey) -> i32 {
    small(title.id())
}

fn key_of(kind: &str, id: i32) -> Result<TitleKey, LibraryError> {
    let kind = if kind == "tv" {
        TitleKind::Tv
    } else {
        TitleKind::Movie
    };
    TitleKey::new(kind, narrow(id)).map_err(|_| LibraryError::Unavailable)
}

fn rating_word(rating: Option<Rating>) -> Option<&'static str> {
    match rating {
        Some(Rating::Down) => Some("down"),
        Some(Rating::Up) => Some("up"),
        Some(Rating::Love) => Some("love"),
        _ => None,
    }
}

fn rating_of(word: Option<&str>) -> Option<Rating> {
    match word {
        Some("down") => Some(Rating::Down),
        Some("up") => Some(Rating::Up),
        Some("love") => Some(Rating::Love),
        _ => None,
    }
}

const fn outcome_word(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Applied => "applied",
        Outcome::Superseded => "superseded",
        _ => "rejected",
    }
}

fn outcome_of(word: &str) -> Outcome {
    match word {
        "applied" => Outcome::Applied,
        "superseded" => Outcome::Superseded,
        _ => Outcome::Rejected,
    }
}

const fn event_word(event: FeedbackEvent) -> Option<&'static str> {
    match event {
        FeedbackEvent::Click => Some("click"),
        FeedbackEvent::Start => Some("start"),
        FeedbackEvent::Complete => Some("complete"),
        FeedbackEvent::Like => Some("like"),
        FeedbackEvent::Skip => Some("skip"),
        _ => None,
    }
}

fn parts(episode: Option<EpisodeRef>) -> (i32, i32) {
    episode.map_or((0, 0), |e| (small(e.season), small(e.episode)))
}

fn episode_of(season: i32, episode: i32) -> Option<EpisodeRef> {
    (season > 0 || episode > 0).then(|| EpisodeRef {
        season: narrow(season),
        episode: narrow(episode),
    })
}

fn title_row(row: &PgRow) -> Result<TitleState, sqlx::Error> {
    let kind: String = row.try_get("kind")?;
    let id: i32 = row.try_get("title_id")?;
    let rating: Option<String> = row.try_get("rating")?;
    Ok(TitleState {
        title: key_of(&kind, id).map_err(|_| sqlx::Error::RowNotFound)?,
        in_list: row.try_get("in_list")?,
        list_version: wide(row.try_get("list_version")?),
        list_changed_ms: opt_ms(row.try_get("list_changed_at")?),
        added_ms: row
            .try_get::<Option<OffsetDateTime>, _>("added_at")?
            .map(to_ms),
        rating: rating_of(rating.as_deref()),
        rating_version: wide(row.try_get("rating_version")?),
        rating_changed_ms: opt_ms(row.try_get("rating_changed_at")?),
        seq: wide(row.try_get("seq")?),
    })
}

fn progress_row(row: &PgRow) -> Result<EpisodeProgress, sqlx::Error> {
    let kind: String = row.try_get("kind")?;
    let id: i32 = row.try_get("title_id")?;
    Ok(EpisodeProgress {
        title: key_of(&kind, id).map_err(|_| sqlx::Error::RowNotFound)?,
        episode: episode_of(row.try_get("season")?, row.try_get("episode")?),
        position_s: narrow(row.try_get("position_s")?),
        duration_s: narrow(row.try_get("duration_s")?),
        completed: row.try_get("completed")?,
        version: wide(row.try_get("version")?),
        changed_ms: to_ms(row.try_get("changed_at")?),
        seq: wide(row.try_get("seq")?),
    })
}

fn settings_row(row: &PgRow) -> Result<SettingsView, sqlx::Error> {
    let style: serde_json::Value = row.try_get("subtitle_style")?;
    Ok(SettingsView {
        autoplay_next: row.try_get("autoplay_next")?,
        autoplay_previews: row.try_get("autoplay_previews")?,
        audio_language: row.try_get("audio_language")?,
        subtitle_language: row.try_get("subtitle_language")?,
        subtitle_style: serde_json::from_value(style).unwrap_or_default(),
        ui_language: row.try_get("ui_language")?,
        handle: row.try_get("game_handle")?,
        gender: row.try_get("gender")?,
    })
}

const fn setting_column(key: SettingKey) -> Option<&'static str> {
    match key {
        SettingKey::AutoplayNext => Some("autoplay_next"),
        SettingKey::AutoplayPreviews => Some("autoplay_previews"),
        SettingKey::AudioLanguage => Some("audio_language"),
        SettingKey::SubtitleLanguage => Some("subtitle_language"),
        SettingKey::SubtitleStyle => Some("subtitle_style"),
        SettingKey::UiLanguage => Some("ui_language"),
        SettingKey::Handle => Some("game_handle"),
        SettingKey::Gender => Some("gender"),
        _ => None,
    }
}

struct PgTx {
    tx: Transaction<'static, Postgres>,
    profile: ProfileId,
}

#[async_trait]
impl LibraryTx for PgTx {
    async fn next_seq(&mut self) -> Row<u64> {
        let row = sqlx::query(
            "update profile_library set seq = seq + 1 where profile_id = $1 returning seq",
        )
        .bind(self.profile.0)
        .fetch_one(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(wide(row.try_get("seq").map_err(fail)?))
    }

    async fn seen_op(&mut self, op_id: &str) -> Row<Option<Outcome>> {
        let row =
            sqlx::query("select outcome from applied_ops where profile_id = $1 and op_id = $2")
                .bind(self.profile.0)
                .bind(op_id)
                .fetch_optional(&mut *self.tx)
                .await
                .map_err(fail)?;
        row.map(|row| row.try_get::<String, _>("outcome").map(|w| outcome_of(&w)))
            .transpose()
            .map_err(fail)
    }

    async fn record_op(&mut self, op_id: &str, outcome: Outcome, now_ms: u64) -> Row<()> {
        sqlx::query("insert into applied_ops (profile_id, op_id, outcome, at) values ($1, $2, $3, $4) on conflict do nothing")
            .bind(self.profile.0)
            .bind(op_id)
            .bind(outcome_word(outcome))
            .bind(to_time(now_ms))
            .execute(&mut *self.tx)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn title(&mut self, title: TitleKey) -> Row<Option<TitleState>> {
        let row = sqlx::query(
            "select * from title_state where profile_id = $1 and kind = $2 and title_id = $3",
        )
        .bind(self.profile.0)
        .bind(kind_word(title))
        .bind(id_of(title))
        .fetch_optional(&mut *self.tx)
        .await
        .map_err(fail)?;
        row.as_ref().map(title_row).transpose().map_err(fail)
    }

    async fn put_title(&mut self, row: &TitleState) -> Row<()> {
        sqlx::query(
            "insert into title_state (profile_id, kind, title_id, in_list, list_version, list_changed_at,
               added_at, rating, rating_version, rating_changed_at, seq)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             on conflict (profile_id, kind, title_id) do update set
               in_list = excluded.in_list, list_version = excluded.list_version,
               list_changed_at = excluded.list_changed_at, added_at = excluded.added_at,
               rating = excluded.rating, rating_version = excluded.rating_version,
               rating_changed_at = excluded.rating_changed_at, seq = excluded.seq",
        )
        .bind(self.profile.0)
        .bind(kind_word(row.title))
        .bind(id_of(row.title))
        .bind(row.in_list)
        .bind(big(row.list_version))
        .bind((row.list_changed_ms > 0).then(|| to_time(row.list_changed_ms)))
        .bind(row.added_ms.map(to_time))
        .bind(rating_word(row.rating))
        .bind(big(row.rating_version))
        .bind((row.rating_changed_ms > 0).then(|| to_time(row.rating_changed_ms)))
        .bind(big(row.seq))
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn progress(
        &mut self,
        title: TitleKey,
        episode: Option<EpisodeRef>,
    ) -> Row<Option<EpisodeProgress>> {
        let (season, number) = parts(episode);
        let row = sqlx::query(
            "select * from episode_progress
             where profile_id = $1 and kind = $2 and title_id = $3 and season = $4 and episode = $5",
        )
        .bind(self.profile.0)
        .bind(kind_word(title))
        .bind(id_of(title))
        .bind(season)
        .bind(number)
        .fetch_optional(&mut *self.tx)
        .await
        .map_err(fail)?;
        row.as_ref().map(progress_row).transpose().map_err(fail)
    }

    async fn put_progress(&mut self, row: &EpisodeProgress) -> Row<()> {
        let (season, number) = parts(row.episode.clone());
        sqlx::query(
            "insert into episode_progress (profile_id, kind, title_id, season, episode, position_s,
               duration_s, completed, version, changed_at, seq)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             on conflict (profile_id, kind, title_id, season, episode) do update set
               position_s = excluded.position_s, duration_s = excluded.duration_s,
               completed = excluded.completed, version = excluded.version,
               changed_at = excluded.changed_at, seq = excluded.seq",
        )
        .bind(self.profile.0)
        .bind(kind_word(row.title))
        .bind(id_of(row.title))
        .bind(season)
        .bind(number)
        .bind(small(row.position_s))
        .bind(small(row.duration_s))
        .bind(row.completed)
        .bind(big(row.version))
        .bind(to_time(row.changed_ms))
        .bind(big(row.seq))
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn hidden(&mut self, title: TitleKey) -> Row<Option<HiddenTitle>> {
        let row = sqlx::query("select hidden_at, seq from hidden_titles where profile_id = $1 and kind = $2 and title_id = $3")
            .bind(self.profile.0)
            .bind(kind_word(title))
            .bind(id_of(title))
            .fetch_optional(&mut *self.tx)
            .await
            .map_err(fail)?;
        row.map(|row| {
            Ok(HiddenTitle {
                title,
                hidden_ms: to_ms(row.try_get("hidden_at").map_err(fail)?),
                seq: wide(row.try_get("seq").map_err(fail)?),
            })
        })
        .transpose()
    }

    async fn put_hidden(&mut self, row: &HiddenTitle) -> Row<()> {
        sqlx::query(
            "insert into hidden_titles (profile_id, kind, title_id, hidden_at, seq) values ($1, $2, $3, $4, $5)
             on conflict (profile_id, kind, title_id) do update set hidden_at = excluded.hidden_at, seq = excluded.seq",
        )
        .bind(self.profile.0)
        .bind(kind_word(row.title))
        .bind(id_of(row.title))
        .bind(to_time(row.hidden_ms))
        .bind(big(row.seq))
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn drop_hidden(&mut self, title: TitleKey) -> Row<()> {
        sqlx::query(
            "delete from hidden_titles where profile_id = $1 and kind = $2 and title_id = $3",
        )
        .bind(self.profile.0)
        .bind(kind_word(title))
        .bind(id_of(title))
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn clear_history(&mut self, at_ms: u64, seq: u64) -> Row<()> {
        let at = to_time(at_ms);
        sqlx::query("delete from episode_progress where profile_id = $1 and changed_at <= $2")
            .bind(self.profile.0)
            .bind(at)
            .execute(&mut *self.tx)
            .await
            .map_err(fail)?;
        sqlx::query("delete from hidden_titles where profile_id = $1 and hidden_at <= $2")
            .bind(self.profile.0)
            .bind(at)
            .execute(&mut *self.tx)
            .await
            .map_err(fail)?;
        sqlx::query("update profile_library set history_cleared_at = $2, history_cleared_seq = $3 where profile_id = $1")
            .bind(self.profile.0)
            .bind(at)
            .bind(big(seq))
            .execute(&mut *self.tx)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn cleared_at(&mut self) -> Row<Option<u64>> {
        let row =
            sqlx::query("select history_cleared_at from profile_library where profile_id = $1")
                .bind(self.profile.0)
                .fetch_one(&mut *self.tx)
                .await
                .map_err(fail)?;
        let at: Option<OffsetDateTime> = row.try_get("history_cleared_at").map_err(fail)?;
        Ok(at.map(to_ms))
    }

    async fn put_setting(
        &mut self,
        key: SettingKey,
        value: &str,
        at_ms: u64,
        seq: u64,
    ) -> Row<bool> {
        let Some(column) = setting_column(key) else {
            return Ok(false);
        };
        sqlx::query("insert into profile_settings (profile_id) values ($1) on conflict do nothing")
            .bind(self.profile.0)
            .execute(&mut *self.tx)
            .await
            .map_err(fail)?;
        let row = sqlx::query(
            "select field_times from profile_settings where profile_id = $1 for update",
        )
        .bind(self.profile.0)
        .fetch_one(&mut *self.tx)
        .await
        .map_err(fail)?;
        let times: serde_json::Value = row.try_get("field_times").map_err(fail)?;
        if times
            .get(column)
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|old| at_ms < old)
        {
            return Ok(false);
        }
        // The column name comes from the closed match above, never from a client.
        let statement = match key {
            SettingKey::AutoplayNext | SettingKey::AutoplayPreviews => {
                format!("update profile_settings set {column} = ($2 = 'true')")
            }
            SettingKey::SubtitleStyle => {
                format!("update profile_settings set {column} = $2::jsonb")
            }
            _ => format!("update profile_settings set {column} = nullif($2, '')"),
        };
        let statement = format!(
            "{statement}, field_times = jsonb_set(field_times, $3, to_jsonb($4::bigint)), version = version + 1, seq = $5, updated_at = now() where profile_id = $1"
        );
        sqlx::query(&statement)
            .bind(self.profile.0)
            .bind(value)
            .bind(vec![column])
            .bind(big(at_ms))
            .bind(big(seq))
            .execute(&mut *self.tx)
            .await
            .map_err(fail)?;
        Ok(true)
    }

    async fn taste(&mut self) -> Row<Taste> {
        let row = sqlx::query(
            "select genre_weights, taste_version from profile_taste where profile_id = $1",
        )
        .bind(self.profile.0)
        .fetch_optional(&mut *self.tx)
        .await
        .map_err(fail)?;
        taste_of(row)
    }

    async fn put_taste(&mut self, taste: &Taste) -> Row<()> {
        sqlx::query(
            "insert into profile_taste (profile_id, genre_weights, taste_version) values ($1, $2, $3)
             on conflict (profile_id) do update set genre_weights = excluded.genre_weights,
               taste_version = excluded.taste_version, updated_at = now()",
        )
        .bind(self.profile.0)
        .bind(serde_json::to_value(&taste.weights).unwrap_or_default())
        .bind(i32::try_from(taste.version).unwrap_or(i32::MAX))
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn add_feed(&mut self, row: FeedRow) -> Row<()> {
        let Some(event) = event_word(row.event) else {
            return Ok(());
        };
        sqlx::query("insert into feed_events (profile_id, ts, event, kind, title_id) values ($1, $2, $3, $4, $5)")
            .bind(self.profile.0)
            .bind(to_time(row.at_ms))
            .bind(event)
            .bind(kind_word(row.title))
            .bind(id_of(row.title))
            .execute(&mut *self.tx)
            .await
            .map_err(fail)?;
        Ok(())
    }

    async fn add_refused_trailer(&mut self, title: TitleKey, at_ms: u64) -> Row<()> {
        sqlx::query(
            "insert into refused_trailers (profile_id, kind, title_id, at) values ($1, $2, $3, $4)
             on conflict (profile_id, kind, title_id) do update set at = excluded.at",
        )
        .bind(self.profile.0)
        .bind(kind_word(title))
        .bind(id_of(title))
        .bind(to_time(at_ms))
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        sqlx::query(
            "delete from refused_trailers where profile_id = $1 and (kind, title_id) not in
             (select kind, title_id from refused_trailers where profile_id = $1 order by at desc limit $2)",
        )
        .bind(self.profile.0)
        .bind(REFUSED_KEPT)
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn reminder(&mut self, title: TitleKey) -> Row<Option<TitleReminder>> {
        let row = sqlx::query(
            "select remind, remind_version, remind_changed_at, seq from profile_reminders where profile_id = $1 and kind = $2 and title_id = $3",
        )
        .bind(self.profile.0)
        .bind(kind_word(title))
        .bind(id_of(title))
        .fetch_optional(&mut *self.tx)
        .await
        .map_err(fail)?;
        row.map(|r| {
            let remind: bool = r.try_get("remind").map_err(fail)?;
            let version: i64 = r.try_get("remind_version").map_err(fail)?;
            let changed_at: Option<OffsetDateTime> =
                r.try_get("remind_changed_at").map_err(fail)?;
            let seq: i64 = r.try_get("seq").map_err(fail)?;
            Ok(TitleReminder {
                title,
                remind,
                version: wide(version),
                changed_ms: opt_ms(changed_at),
                seq: wide(seq),
            })
        })
        .transpose()
    }

    async fn put_reminder(&mut self, row: &TitleReminder) -> Row<()> {
        sqlx::query(
            "insert into profile_reminders (profile_id, kind, title_id, remind, remind_version, remind_changed_at, seq)
             values ($1, $2, $3, $4, $5, $6, $7)
             on conflict (profile_id, kind, title_id) do update set
               remind = excluded.remind, remind_version = excluded.remind_version,
               remind_changed_at = excluded.remind_changed_at, seq = excluded.seq",
        )
        .bind(self.profile.0)
        .bind(kind_word(row.title))
        .bind(id_of(row.title))
        .bind(row.remind)
        .bind(big(row.version))
        .bind((row.changed_ms > 0).then(|| to_time(row.changed_ms)))
        .bind(big(row.seq))
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn mark_notifications_read(&mut self, up_to: &str, now_ms: u64) -> Row<()> {
        sqlx::query(
            "insert into notification_reads (profile_id, read_up_to, dismissed)
             values ($1, $2, jsonb_build_array($3::text))
             on conflict (profile_id) do update set
               read_up_to = excluded.read_up_to,
               dismissed = case when notification_reads.dismissed @> jsonb_build_array($3::text)
                 then notification_reads.dismissed
                 else notification_reads.dismissed || jsonb_build_array($3::text) end",
        )
        .bind(self.profile.0)
        .bind(to_time(now_ms))
        .bind(up_to)
        .execute(&mut *self.tx)
        .await
        .map_err(fail)?;
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Row<()> {
        self.tx.commit().await.map_err(fail)
    }
}

fn taste_of(row: Option<PgRow>) -> Row<Taste> {
    let Some(row) = row else {
        return Ok(Taste::default());
    };
    let weights: serde_json::Value = row.try_get("genre_weights").map_err(fail)?;
    let version: i32 = row.try_get("taste_version").map_err(fail)?;
    Ok(Taste {
        weights: serde_json::from_value(weights).unwrap_or_default(),
        version: u32::try_from(version).unwrap_or(0),
    })
}

#[async_trait]
impl LibraryRead for PgLibrary {
    async fn list(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>> {
        let rows = sqlx::query(
            "select kind, title_id from title_state where profile_id = $1 and in_list
             order by added_at desc, seq desc limit $2",
        )
        .bind(profile.0)
        .bind(big(limit as u64))
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        rows.iter()
            .map(|row| {
                let kind: String = row.try_get("kind").map_err(fail)?;
                key_of(&kind, row.try_get("title_id").map_err(fail)?)
            })
            .collect()
    }

    async fn rated(&self, profile: ProfileId, limit: usize) -> Row<Vec<(TitleKey, Rating)>> {
        let rows = sqlx::query(
            "select kind, title_id, rating from title_state where profile_id = $1 and rating is not null
             order by rating_changed_at desc, seq desc limit $2",
        )
        .bind(profile.0)
        .bind(big(limit as u64))
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let kind: String = row.try_get("kind").map_err(fail)?;
            let key = key_of(&kind, row.try_get("title_id").map_err(fail)?)?;
            let word: Option<String> = row.try_get("rating").map_err(fail)?;
            if let Some(rating) = rating_of(word.as_deref()) {
                out.push((key, rating));
            }
        }
        Ok(out)
    }

    async fn recent(&self, profile: ProfileId, limit: usize) -> Row<Vec<EpisodeProgress>> {
        let rows = sqlx::query(
            "select p.* from episode_progress p
             where p.profile_id = $1 and not exists (
               select 1 from hidden_titles h
               where h.profile_id = p.profile_id and h.kind = p.kind and h.title_id = p.title_id)
             order by p.changed_at desc, p.seq desc limit $2",
        )
        .bind(profile.0)
        .bind(big(limit as u64))
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        rows.iter()
            .map(progress_row)
            .collect::<Result<_, _>>()
            .map_err(fail)
    }

    async fn positions(&self, profile: ProfileId, limit: usize) -> Row<Vec<EpisodeProgress>> {
        let rows = sqlx::query(
            "select * from episode_progress where profile_id = $1
             order by changed_at desc, seq desc limit $2",
        )
        .bind(profile.0)
        .bind(big(limit as u64))
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        rows.iter()
            .map(progress_row)
            .collect::<Result<_, _>>()
            .map_err(fail)
    }

    async fn hidden_titles(&self, profile: ProfileId, limit: usize) -> Row<Vec<HiddenTitle>> {
        let rows = sqlx::query(
            "select kind, title_id, hidden_at, seq from hidden_titles where profile_id = $1
             order by hidden_at desc, seq desc limit $2",
        )
        .bind(profile.0)
        .bind(big(limit as u64))
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let kind: String = row.try_get("kind").map_err(fail)?;
            let at: OffsetDateTime = row.try_get("hidden_at").map_err(fail)?;
            out.push(HiddenTitle {
                title: key_of(&kind, row.try_get("title_id").map_err(fail)?)?,
                hidden_ms: to_ms(at),
                seq: wide(row.try_get("seq").map_err(fail)?),
            });
        }
        Ok(out)
    }

    async fn position(
        &self,
        profile: ProfileId,
        title: TitleKey,
        episode: Option<EpisodeRef>,
    ) -> Row<Option<EpisodeProgress>> {
        let (season, number) = parts(episode);
        let row = sqlx::query(
            "select * from episode_progress
             where profile_id = $1 and kind = $2 and title_id = $3 and season = $4 and episode = $5",
        )
        .bind(profile.0)
        .bind(kind_word(title))
        .bind(id_of(title))
        .bind(season)
        .bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(fail)?;
        row.as_ref().map(progress_row).transpose().map_err(fail)
    }

    async fn taste_of(&self, profile: ProfileId) -> Row<Taste> {
        let row = sqlx::query(
            "select genre_weights, taste_version from profile_taste where profile_id = $1",
        )
        .bind(profile.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(fail)?;
        taste_of(row)
    }

    async fn seen(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>> {
        let rows = self.recent(profile, limit).await?;
        Ok(rows.into_iter().map(|row| row.title).collect())
    }

    async fn settings(&self, profile: ProfileId) -> Row<SettingsView> {
        let row = sqlx::query("select * from profile_settings where profile_id = $1")
            .bind(profile.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(fail)?;
        row.as_ref()
            .map(settings_row)
            .transpose()
            .map_err(fail)
            .map(Option::unwrap_or_default)
    }

    async fn changes_after(
        &self,
        profile: ProfileId,
        after: u64,
        limit: usize,
    ) -> Row<(Vec<Change>, u64)> {
        let (id, after_big, cap) = (profile.0, big(after), big(limit as u64));
        let titles = sqlx::query(
            "select * from title_state where profile_id = $1 and seq > $2 order by seq limit $3",
        )
        .bind(id)
        .bind(after_big)
        .bind(cap)
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        let progress = sqlx::query(
            "select p.*, exists (select 1 from hidden_titles h where h.profile_id = p.profile_id
               and h.kind = p.kind and h.title_id = p.title_id) as is_hidden
             from episode_progress p where p.profile_id = $1 and p.seq > $2 order by p.seq limit $3",
        )
        .bind(id)
        .bind(after_big)
        .bind(cap)
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        let hidden = sqlx::query("select kind, title_id, seq from hidden_titles where profile_id = $1 and seq > $2 order by seq limit $3")
            .bind(id)
            .bind(after_big)
            .bind(cap)
            .fetch_all(&self.pool)
            .await
            .map_err(fail)?;
        let reminders = sqlx::query(
            "select kind, title_id, remind, seq from profile_reminders where profile_id = $1 and seq > $2 order by seq limit $3",
        )
        .bind(id)
        .bind(after_big)
        .bind(cap)
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        let cleared = sqlx::query(
            "select history_cleared_at, history_cleared_seq from profile_library
             where profile_id = $1 and history_cleared_seq > $2",
        )
        .bind(id)
        .bind(after_big)
        .fetch_optional(&self.pool)
        .await
        .map_err(fail)?;
        let mut rows: Vec<(u64, Change)> = Vec::new();
        for row in &titles {
            let state = title_row(row).map_err(fail)?;
            rows.push((state.seq, title_change(&state)));
        }
        for row in &progress {
            let state = progress_row(row).map_err(fail)?;
            let is_hidden: bool = row.try_get("is_hidden").map_err(fail)?;
            rows.push((state.seq, progress_change(&state, is_hidden)));
        }
        for row in &hidden {
            let kind: String = row.try_get("kind").map_err(fail)?;
            let title = key_of(&kind, row.try_get("title_id").map_err(fail)?)?;
            rows.push((
                wide(row.try_get("seq").map_err(fail)?),
                Change::Hidden { title },
            ));
        }
        for row in &reminders {
            let kind: String = row.try_get("kind").map_err(fail)?;
            let title = key_of(&kind, row.try_get("title_id").map_err(fail)?)?;
            let on: bool = row.try_get("remind").map_err(fail)?;
            rows.push((
                wide(row.try_get("seq").map_err(fail)?),
                Change::Remind { title, on },
            ));
        }
        if let Some(row) = cleared {
            let at: OffsetDateTime = row.try_get("history_cleared_at").map_err(fail)?;
            let seq: i64 = row.try_get("history_cleared_seq").map_err(fail)?;
            rows.push((
                wide(seq),
                Change::HistoryCleared {
                    changed_at_ms: to_ms(at),
                },
            ));
        }
        rows.sort_by_key(|(seq, _)| *seq);
        rows.truncate(limit);
        let last = rows.last().map_or(after, |(seq, _)| *seq);
        Ok((rows.into_iter().map(|(_, change)| change).collect(), last))
    }

    async fn forget_taste(&self, profile: ProfileId) -> Row<()> {
        for statement in [
            "delete from profile_taste where profile_id = $1",
            "delete from feed_events where profile_id = $1",
        ] {
            sqlx::query(statement)
                .bind(profile.0)
                .execute(&self.pool)
                .await
                .map_err(fail)?;
        }
        Ok(())
    }

    async fn prune_feed(&self, before_ms: u64) -> Row<u64> {
        let done = sqlx::query("delete from feed_events where ts < $1")
            .bind(to_time(before_ms))
            .execute(&self.pool)
            .await
            .map_err(fail)?;
        Ok(done.rows_affected())
    }

    async fn reminders(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>> {
        let rows = sqlx::query(
            "select kind, title_id from profile_reminders where profile_id = $1 and remind order by remind_changed_at desc limit $2",
        )
        .bind(profile.0)
        .bind(big(u64::try_from(limit).unwrap_or(u64::MAX)))
        .fetch_all(&self.pool)
        .await
        .map_err(fail)?;
        let mut titles = Vec::with_capacity(rows.len());
        for row in rows {
            let kind: String = row.try_get("kind").map_err(fail)?;
            let id: i32 = row.try_get("title_id").map_err(fail)?;
            titles.push(key_of(&kind, id)?);
        }
        Ok(titles)
    }

    async fn is_reminded(&self, profile: ProfileId, title: TitleKey) -> Row<bool> {
        let row = sqlx::query(
            "select remind from profile_reminders where profile_id = $1 and kind = $2 and title_id = $3",
        )
        .bind(profile.0)
        .bind(kind_word(title))
        .bind(id_of(title))
        .fetch_optional(&self.pool)
        .await
        .map_err(fail)?;
        row.map_or(Ok(false), |r| r.try_get("remind").map_err(fail))
    }

    async fn notification_reads(&self, profile: ProfileId) -> Row<NotificationReadState> {
        let row = sqlx::query(
            "select read_up_to, dismissed from notification_reads where profile_id = $1",
        )
        .bind(profile.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(fail)?;
        let Some(row) = row else {
            return Ok(NotificationReadState::default());
        };
        let at: OffsetDateTime = row.try_get("read_up_to").map_err(fail)?;
        let dismissed_val: serde_json::Value = row.try_get("dismissed").map_err(fail)?;
        let dismissed: Vec<String> = serde_json::from_value(dismissed_val).unwrap_or_default();
        Ok(NotificationReadState {
            read_up_to_ms: to_ms(at),
            dismissed,
        })
    }
}

#[async_trait]
impl LibraryStore for PgLibrary {
    /// Takes the counter row lock, so a second batch for this profile waits.
    async fn begin(&self, profile: ProfileId) -> Row<Box<dyn LibraryTx>> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        sqlx::query("insert into profile_library (profile_id) values ($1) on conflict do nothing")
            .bind(profile.0)
            .execute(&mut *tx)
            .await
            .map_err(fail)?;
        sqlx::query("select seq from profile_library where profile_id = $1 for update")
            .bind(profile.0)
            .fetch_one(&mut *tx)
            .await
            .map_err(fail)?;
        Ok(Box::new(PgTx { tx, profile }))
    }

    /// One transaction over every table of this store keyed to the profile. The names come from
    /// `server_accounts::holdings::HELD`, where each of them is an entry, and the counter row goes
    /// last so a batch that is waiting on it is not left holding a lock on a row that is gone.
    async fn delete_profile(&self, profile: ProfileId) -> Row<()> {
        let mut tx = self.pool.begin().await.map_err(fail)?;
        for table in [
            "title_state",
            "episode_progress",
            "hidden_titles",
            "applied_ops",
            "profile_settings",
            "refused_trailers",
            "profile_taste",
            "feed_events",
            "profile_reminders",
            "notification_reads",
            "profile_library",
        ] {
            sqlx::query(&format!("delete from {table} where profile_id = $1"))
                .bind(profile.0)
                .execute(&mut *tx)
                .await
                .map_err(fail)?;
        }
        tx.commit().await.map_err(fail)?;
        Ok(())
    }
}
