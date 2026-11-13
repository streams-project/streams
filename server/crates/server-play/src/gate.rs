//! `POST /v1/play`: the door knows device and profile, eligibility runs before any finder call, and
//! a play id stays bound to the session and profile that made it.

use std::sync::Arc;
use std::time::Duration;

use contract::client::CapabilityCard;
use contract::finder::{FindOutcome, FindRequest};
use contract::play::{AgeCard, Attempt, MoreAttempts, PlayRequest};
use contract::{EpisodeRef, TitleKey, TitleKind};
use moka::sync::Cache;
use server_accounts::ProfileScope;
use server_catalog::catalog::cert_label;
use server_catalog::manifest::role;
use server_catalog::seasons::next_episode;
use server_catalog::{DataSet, item_key};
use server_library::Library;
use server_rules::Viewer;
use server_rules::answer::{PlayExtras, SafePlayAnswer};
use thiserror::Error;
use uuid::Uuid;

use crate::finder::Finder;
use crate::rank::{Viewing, rank};
use crate::streams::{Claim, STREAMS_MAX_DEFAULT, StreamKey, StreamStore};

/// A play session lives this long.
const PLAY_TTL_S: u64 = 6 * 3600;
const PLAY_SESSIONS_MAX: u64 = 100_000;
/// The budget of a find when the operator set none for this class.
const DEFAULT_BUDGET_MS: u32 = 8000;
/// Most attempts one answer holds.
const ATTEMPTS_MAX: usize = 24;

/// Why a play has no answer.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayError {
    /// The title is outside the catalogue.
    #[error("no longer available")]
    Gone,
    /// The rules refused this viewer.
    #[error("refused")]
    Refused,
    /// The request does not fit the title.
    #[error("bad request")]
    Bad,
    /// No finder answered. The rest of the service works.
    #[error("no finder answered")]
    NoFinder,
    /// Finders answered and found nothing.
    #[error("no source")]
    NoSource,
    /// Not a play of this session and profile.
    #[error("no such play")]
    Unknown,
    /// The account is already playing as many streams at once as it may.
    #[error("too many streams")]
    TooMany,
}

/// One play in progress.
#[derive(Clone, Debug)]
struct PlaySession {
    owner: String,
    title: TitleKey,
    episode: Option<EpisodeRef>,
    job: Option<String>,
    viewing: Viewing,
    card: Option<CapabilityCard>,
}

/// What a play request brings besides its title.
#[derive(Clone, Debug)]
pub struct PlayAsk {
    pub request: PlayRequest,
    pub card: Option<CapabilityCard>,
    pub budget_ms: Option<u32>,
    pub embeds: bool,
}

/// The play gate.
pub struct Play {
    finder: Arc<dyn Finder>,
    /// Plays in progress, for the id checks. Per process, and never the stream count: see
    /// [`crate::streams`].
    sessions: Cache<String, PlaySession>,
    /// Where simultaneous streams are counted. None leaves the limit off.
    streams: Option<Arc<dyn StreamStore>>,
    streams_max: u32,
}

impl std::fmt::Debug for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Play")
    }
}

fn owner_of(scope: &ProfileScope) -> String {
    format!("{}:{}", scope.session(), scope.profile())
}

fn packed(key: TitleKey) -> u32 {
    let word = if key.kind() == TitleKind::Tv {
        "tv"
    } else {
        "movie"
    };
    item_key(word, key.id())
}

fn episode_fits(title: TitleKey, episode: Option<&EpisodeRef>) -> bool {
    match episode {
        Some(found) => {
            title.kind() == TitleKind::Tv && found.season <= 9_999 && found.episode <= 99_999
        }
        None => title.kind() != TitleKind::Tv,
    }
}

/// The content reasons of a video, words from a fixed list.
fn reasons_of(data: &DataSet, title: TitleKey, episode: Option<&EpisodeRef>) -> Vec<String> {
    let entry = data.play_facts.reasons.get(&title.to_string());
    let (season, number) = (episode.map(|e| e.season), episode.map(|e| e.episode));
    crate::marks::pick_reasons(entry, season, number)
        .as_array()
        .map(|words| {
            words
                .iter()
                .filter_map(|word| word.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

impl Play {
    /// A gate over a finder, with no limit on simultaneous streams.
    #[must_use]
    pub fn new(finder: Arc<dyn Finder>) -> Self {
        Self {
            finder,
            sessions: Cache::builder()
                .max_capacity(PLAY_SESSIONS_MAX)
                .time_to_live(Duration::from_secs(PLAY_TTL_S))
                .build(),
            streams: None,
            streams_max: STREAMS_MAX_DEFAULT,
        }
    }

    /// This gate, counting simultaneous streams in this store and allowing this many at once.
    ///
    /// The store is shared, not a field of one process, which is what makes the count right when
    /// a deployment answers from more than one server.
    #[must_use]
    pub fn limiting(mut self, streams: Arc<dyn StreamStore>, max: u32) -> Self {
        self.streams = Some(streams);
        self.streams_max = max.max(1);
        self
    }

    fn stream_key(scope: &ProfileScope) -> StreamKey {
        StreamKey {
            account: scope.account(),
            session: scope.session(),
            profile: scope.profile(),
        }
    }

    /// Give back the slot of a play that never started. A device that was already playing
    /// something else gives back the slot that play was using too, because the claim replaced it;
    /// that errs towards letting a viewer play, which is the direction a count should err in.
    async fn release(&self, scope: &ProfileScope, play_id: &str) {
        if let Some(streams) = &self.streams {
            streams.release(Self::stream_key(scope), play_id).await;
        }
    }

    /// Ask to play. Refused titles reach no finder.
    ///
    /// # Errors
    /// See [`PlayError`].
    pub async fn play(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        scope: &ProfileScope,
        library: &Library,
        ask: PlayAsk,
    ) -> Result<SafePlayAnswer, PlayError> {
        let PlayRequest { title, episode } = ask.request;
        if !episode_fits(title, episode.as_ref()) {
            return Err(PlayError::Bad);
        }
        let key = packed(title);
        if !data.catalog.contains_key(&key) {
            return Err(PlayError::Gone);
        }
        let visible = viewer.see(data, key).ok_or(PlayError::Refused)?;
        let item = visible.item();
        let settings = library
            .store()
            .settings(scope.profile())
            .await
            .unwrap_or_default();
        // Eligibility is settled and no finder has been asked anything yet, which is where the
        // stream limit belongs: a play this account may not have reaches no finder, and a play
        // that fails to find a source gives its slot straight back below.
        let play_id = Uuid::new_v4().to_string();
        if let Some(streams) = &self.streams {
            let claim = streams
                .claim(Self::stream_key(scope), &play_id, self.streams_max)
                .await;
            if matches!(claim, Claim::Full { .. }) {
                return Err(PlayError::TooMany);
            }
        }
        let request = FindRequest {
            title,
            episode: episode.clone(),
            name: item.title.clone(),
            year: item.release_year.and_then(|year| u32::try_from(year).ok()),
            budget_ms: ask.budget_ms.unwrap_or(DEFAULT_BUDGET_MS),
        };
        let found = match self.finder.find(&request).await {
            Ok(found) => found,
            Err(_) => {
                self.release(scope, &play_id).await;
                return Err(PlayError::NoFinder);
            }
        };
        if found.sources.is_empty() && found.outcome != FindOutcome::Pending && found.job.is_none()
        {
            self.release(scope, &play_id).await;
            return Err(PlayError::NoSource);
        }
        let viewing = Viewing {
            language: settings
                .audio_language
                .clone()
                .unwrap_or_else(|| String::from("en")),
            original_language: item.original_language.clone(),
            kids: viewer.tier().is_kids(),
            embeds: ask.embeds,
        };
        let (mut attempts, dropped) = rank(found.sources, ask.card.as_ref(), &viewing, &play_id);
        attempts.truncate(ATTEMPTS_MAX);
        let start_at_ms = library
            .start_at_ms(scope.profile(), title, episode.clone())
            .await
            .unwrap_or(0);
        let more = found.job.is_some();
        let reasons = reasons_of(data, title, episode.as_ref());
        let next_episode = episode.as_ref().and_then(|now| {
            let catalog = data.files.get(role::CATALOG)?;
            let (season, episode) = next_episode(catalog, title.id(), now.season, now.episode)?;
            Some(EpisodeRef { season, episode })
        });
        self.sessions.insert(
            play_id.clone(),
            PlaySession {
                owner: owner_of(scope),
                title,
                episode: episode.clone(),
                job: found.job,
                viewing,
                card: ask.card,
            },
        );
        let extras = PlayExtras {
            play_id,
            start_at_ms,
            age_card: Some(AgeCard {
                label: cert_label(item.cert).to_owned(),
                reasons,
            }),
            audio_default: settings.audio_language,
            subtitle_default: settings.subtitle_language,
            next_episode,
            autoplay_next: settings.autoplay_next,
            dropped,
            more,
        };
        Ok(SafePlayAnswer::new(visible, attempts, extras))
    }

    fn session(&self, id: &str, scope: &ProfileScope) -> Result<PlaySession, PlayError> {
        self.sessions
            .get(id)
            .filter(|held| held.owner == owner_of(scope))
            .ok_or(PlayError::Unknown)
    }

    /// Attempts that arrived late. The title is checked again, so a play outlives no change.
    ///
    /// # Errors
    /// `Unknown` for a play of another session or profile.
    pub async fn more(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        scope: &ProfileScope,
        id: &str,
    ) -> Result<MoreAttempts, PlayError> {
        let mut held = self.session(id, scope)?;
        viewer
            .see(data, packed(held.title))
            .ok_or(PlayError::Refused)?;
        let Some(job) = held.job.clone() else {
            return Ok(MoreAttempts {
                attempts: Vec::new(),
                more: false,
            });
        };
        let found = self
            .finder
            .rest(&job)
            .await
            .map_err(|_| PlayError::NoFinder)?;
        let attempts: Vec<Attempt> = rank(found.sources, held.card.as_ref(), &held.viewing, id).0;
        held.job = found.job;
        let more = held.job.is_some();
        self.sessions.insert(id.to_owned(), held);
        Ok(MoreAttempts { attempts, more })
    }

    /// Check that a play belongs to this session and profile.
    ///
    /// Every caller of this is a device saying something about a play it is in the middle of, so
    /// it also says the stream is alive. The beat is sent off to run on its own: the slot it
    /// keeps fresh is worth nothing if a lost write can cost a viewer their playback, and
    /// [`Play::beat`] is the same work for a caller that can wait for it.
    ///
    /// # Errors
    /// `Unknown` otherwise.
    pub fn check(
        &self,
        id: &str,
        scope: &ProfileScope,
    ) -> Result<(TitleKey, Option<EpisodeRef>), PlayError> {
        let held = self.session(id, scope)?;
        self.beat_soon(id, scope);
        Ok((held.title, held.episode))
    }

    /// This play is still running, so its stream keeps its slot.
    ///
    /// # Errors
    /// `Unknown` for a play of another session or profile.
    pub async fn beat(&self, id: &str, scope: &ProfileScope) -> Result<(), PlayError> {
        self.session(id, scope)?;
        if let Some(streams) = &self.streams {
            streams.beat(Self::stream_key(scope), id).await;
        }
        Ok(())
    }

    /// The beat of [`Play::check`], on whatever runtime the caller is on. Nothing without one,
    /// and nothing with the limit off.
    fn beat_soon(&self, id: &str, scope: &ProfileScope) {
        let Some(streams) = self.streams.clone() else {
            return;
        };
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let (key, id) = (Self::stream_key(scope), id.to_owned());
        handle.spawn(async move { streams.beat(key, &id).await });
    }

    /// How many streams this account has running, for a test or an operator's count.
    pub async fn live_streams(&self, scope: &ProfileScope) -> u32 {
        match &self.streams {
            Some(streams) => streams.live(scope.account()).await,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering};

    use contract::finder::{FindAnswer, FindOutcome, FoundSource};
    use contract::play::{AttemptKind, PlayRequest};
    use contract::{TitleKey, TitleKind};
    use server_accounts::limits::DEFAULT_LIMITS;
    use server_accounts::{
        AccountId, AccountStore, Door, ManualClock, MemoryStore, NewAccount, ProfileScope,
        StepUpMethod,
    };
    use server_catalog::sample::{sample_trust, write_sample};
    use server_catalog::{DataSet, DataSlot};
    use server_library::{Library, LibraryStore, MemoryLibrary};
    use server_rules::Viewer;

    use super::{Play, PlayAsk, PlayError};
    use crate::finder::MemoryFinder;
    use crate::streams::{MemoryStreamStore, STREAM_STALE_S};

    /// The sample set, built once for every test in this file.
    fn data() -> Arc<DataSet> {
        static SET: OnceLock<Arc<DataSet>> = OnceLock::new();
        SET.get_or_init(|| {
            let dir = std::env::temp_dir().join(format!("server_play_gate_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("sample folder");
            let manifest = write_sample(&dir);
            DataSlot::open(&manifest, sample_trust())
                .expect("sample set")
                .load()
        })
        .clone()
    }

    /// A finder that answers every call with one source.
    fn finder() -> Arc<MemoryFinder> {
        Arc::new(MemoryFinder::default())
    }

    fn answer() -> FindAnswer {
        FindAnswer {
            outcome: FindOutcome::Found,
            sources: vec![FoundSource {
                url: String::from("https://host.example/1.m3u8"),
                kind: AttemptKind::Hls,
                group: String::from("a-provider"),
                headers: std::collections::BTreeMap::new(),
                verified: true,
                container: None,
                height: None,
                audio: Vec::new(),
            }],
            job: None,
        }
    }

    /// A door on an empty store, and one account on it.
    async fn account() -> (Door, AccountId) {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let clock = Arc::new(ManualClock::new(1_700_000_000_000));
        let store: Arc<dyn AccountStore> = Arc::new(MemoryStore::default());
        let door = Door::new(store, clock, DEFAULT_LIMITS);
        let id = NEXT.fetch_add(1, Ordering::SeqCst);
        let account = NewAccount {
            id: AccountId::new(),
            email: format!("a{id}@b.test"),
            first_name: String::from("Ada"),
            last_name: String::from("L"),
        };
        door.create_account(&account).await.expect("an account");
        (door, account.id)
    }

    /// One more device of this account, signed in and on its main profile.
    async fn device(door: &Door, account: AccountId) -> ProfileScope {
        let (_, tokens) = door
            .open_session(account, "a-device", StepUpMethod::Password)
            .await
            .expect("a session");
        let member = door.member(&tokens.access).await.expect("a member");
        let main = door.profiles(&member).await.expect("profiles").remove(0);
        door.enter_profile(&member, main.id, None)
            .await
            .expect("the profile");
        let member = door.member(&tokens.access).await.expect("a member");
        door.scope(&member).await.expect("a scope")
    }

    fn library() -> Library {
        let books: Arc<dyn LibraryStore> = Arc::new(Arc::new(MemoryLibrary::default()));
        Library::new(books, Arc::new(ManualClock::new(1_700_000_000_000)))
    }

    fn ask() -> PlayAsk {
        PlayAsk {
            request: PlayRequest {
                title: TitleKey::new(TitleKind::Movie, 100).expect("a title"),
                episode: None,
            },
            card: None,
            budget_ms: None,
            embeds: false,
        }
    }

    /// Start a play on this gate for this device, and give back its play id.
    async fn start(gate: &Play, scope: &ProfileScope) -> Result<String, PlayError> {
        let set = data();
        let viewer = Viewer::of(scope).expect("a viewer");
        gate.play(&set, &viewer, scope, &library(), ask())
            .await
            .map(|answer| answer.wire().play_id.clone())
    }

    fn gate_with(store: Arc<MemoryStreamStore>, max: u32) -> Play {
        let found = finder();
        for _ in 0..8 {
            found.push(answer());
        }
        Play::new(found).limiting(store, max)
    }

    #[tokio::test]
    async fn a_third_stream_is_refused_where_two_are_allowed() {
        let store = Arc::new(MemoryStreamStore::new());
        let gate = gate_with(store, 2);
        let (door, account) = account().await;
        let one = device(&door, account).await;
        let two = device(&door, account).await;
        let three = device(&door, account).await;
        assert!(start(&gate, &one).await.is_ok());
        assert!(start(&gate, &two).await.is_ok());
        assert_eq!(start(&gate, &three).await, Err(PlayError::TooMany));
        assert_eq!(gate.live_streams(&one).await, 2);
    }

    #[tokio::test]
    async fn a_stream_that_stopped_beating_stops_counting() {
        let store = Arc::new(MemoryStreamStore::new());
        let gate = gate_with(store.clone(), 2);
        let (door, account) = account().await;
        let one = device(&door, account).await;
        let two = device(&door, account).await;
        let three = device(&door, account).await;
        let playing = start(&gate, &one).await.expect("a play");
        assert!(start(&gate, &two).await.is_ok());
        assert_eq!(start(&gate, &three).await, Err(PlayError::TooMany));
        // One keeps playing and says so; two is a device that lost power.
        let stale_ms = u64::from(STREAM_STALE_S) * 1000;
        for _ in 0..4 {
            store.advance_ms(stale_ms / 2);
            gate.beat(&playing, &one).await.expect("a beat");
        }
        assert_eq!(gate.live_streams(&one).await, 1, "only the one beating");
        assert!(
            start(&gate, &three).await.is_ok(),
            "the dead stream must not hold its slot for ever"
        );
    }

    #[tokio::test]
    async fn the_count_is_shared_between_two_servers() {
        let store = Arc::new(MemoryStreamStore::new());
        // Two gates, each with its own `moka` cache of plays, over one store.
        let first = gate_with(store.clone(), 2);
        let second = gate_with(store, 2);
        let (door, account) = account().await;
        let one = device(&door, account).await;
        let two = device(&door, account).await;
        let three = device(&door, account).await;
        assert!(start(&first, &one).await.is_ok());
        assert!(start(&second, &two).await.is_ok());
        assert_eq!(
            start(&first, &three).await,
            Err(PlayError::TooMany),
            "the second server's play must count against the first"
        );
    }

    /// The same two servers with a store each, which is what a count held in this process would
    /// be. The third play is let through, so the refusal above came from the shared store and
    /// not from anything else in the test.
    #[tokio::test]
    async fn a_count_per_server_would_let_the_third_stream_through() {
        let first = gate_with(Arc::new(MemoryStreamStore::new()), 2);
        let second = gate_with(Arc::new(MemoryStreamStore::new()), 2);
        let (door, account) = account().await;
        let one = device(&door, account).await;
        let two = device(&door, account).await;
        let three = device(&door, account).await;
        assert!(start(&first, &one).await.is_ok());
        assert!(start(&second, &two).await.is_ok());
        assert!(start(&first, &three).await.is_ok());
    }

    #[tokio::test]
    async fn a_play_that_found_no_source_gives_its_slot_back() {
        let store = Arc::new(MemoryStreamStore::new());
        let gate = Play::new(finder()).limiting(store, 1);
        let (door, account) = account().await;
        let one = device(&door, account).await;
        let two = device(&door, account).await;
        assert_eq!(start(&gate, &one).await, Err(PlayError::NoSource));
        assert_eq!(gate.live_streams(&one).await, 0);
        assert_eq!(start(&gate, &two).await, Err(PlayError::NoSource));
    }

    #[tokio::test]
    async fn with_no_store_nothing_is_counted() {
        let gate = {
            let found = finder();
            for _ in 0..4 {
                found.push(answer());
            }
            Play::new(found)
        };
        let (door, account) = account().await;
        let one = device(&door, account).await;
        let two = device(&door, account).await;
        let three = device(&door, account).await;
        assert!(start(&gate, &one).await.is_ok());
        assert!(start(&gate, &two).await.is_ok());
        assert!(start(&gate, &three).await.is_ok());
        assert_eq!(gate.live_streams(&one).await, 0);
    }
}
