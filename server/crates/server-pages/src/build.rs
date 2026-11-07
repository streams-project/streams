//! From a screen name to a safe answer. Rows are built and kept per surface, genre, tier and class;
//! the viewer's blocked names apply last, per request, when each title is turned into a tile.

use std::collections::HashMap as StdMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use contract::EpisodeRef;
use contract::client::DeviceClass;
use contract::screen::{Cursor, Nav, NavItem, RefreshTrigger, ScreenName, SectionKind};
use moka::sync::Cache;
use server_catalog::DataSet;
use server_catalog::catalog::CatalogItem;
use server_catalog::det::HashSet;
use server_catalog::manifest::role;
use server_rules::age::{RatingSystem, age_mark};
use server_rules::answer::{
    DetailExtras, Images, SafeMore, SafeScreen, SafeSection, SafeSuggestions, SafeTile,
    SafeTitleDetail, TileExtras,
};
use server_rules::{Tier, Viewer, Visible};
use thiserror::Error;

use crate::cursor::{CursorError, CursorKey, Position};
use crate::genre_pages::{Surface, page as genre_page};
use crate::genre_spec::{GenreSpec, resolve};
use crate::page::{FIRST_SCREEN, PageAsk, PageContext, Row, RowItem, SLOT_MS};
use crate::plan::{INITIAL_ROWS, INITIAL_ROWS_GENRE, MIN_ROWS_FLOOR, PagePlan, ROWS_CAP, page_cap};
use crate::requests::RequestStore;
use crate::search::{TitleIndex, load_aliases, search_keys};
use crate::signals::Signals;
use crate::tags::{item_badges, item_tags};
use crate::words::Words;

/// Seconds a screen stays good on a client.
const SCREEN_TTL_S: u32 = 30;
/// Tiles sent with a row, and the most one page of a row holds. The rest are read through its
/// cursor, a page at a time.
pub(crate) const ROW_PAGE: usize = 24;
/// A row with fewer titles after sharing out is dropped.
const MIN_ROW_ITEMS: usize = 4;
/// A profile's own list makes a row from its first title.
const MIN_OWN_ROW_ITEMS: usize = 1;
/// Full rows are kept for paging.
const ROWS_KEPT_S: u64 = 600;
/// Bytes the rows of all pages may hold, as an estimate.
const ROWS_KEPT_BYTES: u64 = 256 * 1024 * 1024;
const BYTES_PER_ITEM: usize = 512;
/// Titles a search screen shows.
const SEARCH_TITLES: usize = 40;
/// Most notification tiles.
const NOTIFICATIONS_MAX: usize = 12;

/// Why a screen has no answer.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PageError {
    #[error("no such page")]
    NotFound,
    #[error("this page is not offered now")]
    Refused,
    #[error("this feature is off")]
    Off(&'static str),
    #[error("the cursor was not written for this caller")]
    BadCursor,
}

/// What a page may read about a profile. Consent decides what is filled in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Personal {
    pub signals: Signals,
    /// Genre weights, empty with consent off.
    pub taste: Vec<(String, f32)>,
    /// Titles this profile started, for leaving out of rows.
    pub seen: Vec<u32>,
    /// Percent watched and episode of each Continue Watching title.
    pub progress: StdMap<u32, (u32, Option<EpisodeRef>)>,
    /// Changes when the library changes, so kept rows are not reused.
    pub version: u64,
}

/// One request for a screen.
#[derive(Clone, Debug)]
pub struct ScreenAsk {
    pub screen: ScreenName,
    pub genre: Option<String>,
    pub class: DeviceClass,
    pub now_ms: u64,
    /// Whose answer this is, for cursors: the session and profile, or `visitor`.
    pub owner: String,
    /// Seeds a session's rotation. Empty with consent off.
    pub seed: String,
    pub personal: Personal,
    /// Row titles the operator took off every page, lower case.
    pub blocked_rows: Vec<String>,
}

/// Facts built once per data set.
struct SetFacts {
    hash: String,
    requests: RequestStore,
    titles: TitleIndex,
    aliases: Vec<Vec<String>>,
}

/// Rows kept for one key.
type KeptRows = Arc<Vec<Row>>;

/// Everything that builds pages.
pub struct Pages {
    key: CursorKey,
    images: Images,
    words: Words,
    rows: Cache<String, KeptRows>,
    caps: Cache<String, usize>,
    set: Mutex<Option<Arc<SetFacts>>>,
}

impl std::fmt::Debug for Pages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pages")
    }
}

/// The kept page a request belongs to.
struct Shape {
    surface: Surface,
    spec: Option<GenreSpec>,
    language: Option<String>,
    collection: Option<String>,
    tier: Tier,
}

fn slot_of(now_ms: u64) -> u64 {
    now_ms / SLOT_MS
}

fn screen_word(name: ScreenName) -> String {
    serde_json::to_value(name)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn screen_of_word(word: &str) -> Option<ScreenName> {
    serde_json::from_value(serde_json::Value::from(word)).ok()
}

fn weigh(rows: &KeptRows) -> u32 {
    let items: usize = rows.iter().map(|row| row.items.len()).sum();
    u32::try_from(items.saturating_mul(BYTES_PER_ITEM)).unwrap_or(u32::MAX)
}

impl Pages {
    /// Pages with a random cursor key.
    #[must_use]
    pub fn new(images: Images) -> Self {
        Self::with_key(images, CursorKey::random())
    }

    /// Pages with a cursor key.
    #[must_use]
    pub fn with_key(images: Images, key: CursorKey) -> Self {
        Self {
            key,
            images,
            words: Words::default(),
            rows: Cache::builder()
                .max_capacity(ROWS_KEPT_BYTES)
                .weigher(|_, rows: &KeptRows| weigh(rows))
                .time_to_live(Duration::from_secs(ROWS_KEPT_S))
                .build(),
            caps: Cache::builder()
                .max_capacity(4096)
                .time_to_live(Duration::from_secs(ROWS_KEPT_S))
                .build(),
            set: Mutex::new(None),
        }
    }

    /// The picture of one episode, under this set's image address.
    #[must_use]
    pub fn still(&self, path: Option<&str>) -> Vec<contract::screen::Image> {
        self.images.still(path)
    }

    /// These pages, with this brand's name in their words.
    #[must_use]
    pub fn branded(mut self, name: &str) -> Self {
        self.words = Words::new(name);
        self
    }

    fn facts(&self, data: &DataSet) -> Arc<SetFacts> {
        let mut held = self
            .set
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(facts) = held.as_ref().filter(|facts| facts.hash == data.data_hash) {
            return facts.clone();
        }
        let requests = data
            .files
            .get(role::REQUESTS)
            .map_or_else(RequestStore::default, |path| {
                RequestStore::load(path, &data.catalog, &data.vectors, &data.mean, &data.events)
            });
        let facts = Arc::new(SetFacts {
            hash: data.data_hash.clone(),
            requests,
            titles: TitleIndex::build(data),
            aliases: load_aliases(data),
        });
        *held = Some(facts.clone());
        facts
    }

    fn shape(data: &DataSet, viewer: &Viewer, ask: &ScreenAsk) -> Result<Shape, PageError> {
        let today = server_catalog::taxonomy::month_day(ask.now_ms);
        let mut shape = Shape {
            surface: Surface::Both,
            spec: None,
            language: None,
            collection: None,
            tier: viewer.tier(),
        };
        match ask.screen {
            ScreenName::Home => {}
            ScreenName::Films => shape.surface = Surface::Movie,
            ScreenName::Series => shape.surface = Surface::Tv,
            // an adult who opens Kids sees what a child would, never the other way round
            ScreenName::Kids => shape.tier = viewer.tier().min(Tier::OlderKids),
            ScreenName::Language => {
                let code = ask
                    .genre
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .to_ascii_lowercase();
                let plain =
                    (1..=3).contains(&code.len()) && code.chars().all(|c| c.is_ascii_alphabetic());
                if !plain {
                    return Err(PageError::NotFound);
                }
                shape.language = Some(code);
            }
            ScreenName::Genre => {
                let asked = ask
                    .genre
                    .as_deref()
                    .map(str::trim)
                    .filter(|g| !g.is_empty())
                    .ok_or(PageError::NotFound)?;
                if let Some(id) = asked.strip_prefix("collection.") {
                    let plain = !id.is_empty()
                        && id.len() <= 48
                        && id
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                    if !plain {
                        return Err(PageError::NotFound);
                    }
                    shape.collection = Some(id.to_owned());
                    return Ok(shape);
                }
                let (name, surface) = match (asked.strip_prefix("event."), genre_page(asked)) {
                    (Some(event), _) => (event, Surface::Both),
                    (None, Some(page)) => (page.name, page.surface),
                    (None, None) => return Err(PageError::NotFound),
                };
                let spec = resolve(name, surface.word(), &data.events);
                if !spec.in_season(&data.events, today) {
                    return Err(PageError::Refused);
                }
                if matches!(spec, GenreSpec::Unsupported | GenreSpec::Name(_)) {
                    return Err(PageError::NotFound);
                }
                shape.surface = surface;
                shape.spec = Some(spec);
            }
            _ => return Err(PageError::NotFound),
        }
        Ok(shape)
    }

    fn cap(&self, data: &DataSet, shape: &Shape, composed: bool) -> usize {
        if shape.spec.is_none() && shape.language.is_none() {
            return usize::MAX;
        }
        let key = format!(
            "{}|{}|{:?}|{:?}|{}",
            data.data_hash,
            shape.surface.word(),
            shape.spec,
            shape.language,
            shape.tier.as_str()
        );
        self.caps.get_with(key, || {
            page_cap(
                data,
                shape.surface.word(),
                shape.spec.as_ref(),
                shape.language.as_deref(),
                shape.tier,
                composed,
            )
        })
    }

    /// All rows of a page, built once per key while kept.
    fn rows(
        &self,
        data: &DataSet,
        ask: &ScreenAsk,
        shape: &Shape,
        slot: u64,
        wanted: usize,
    ) -> KeptRows {
        let key = format!(
            "{}|{}|{:?}|{:?}|{:?}|{}|{:?}|{slot}|{}|v{}|{wanted}",
            data.data_hash,
            shape.surface.word(),
            ask.genre,
            shape.language,
            shape.collection,
            shape.tier.as_str(),
            ask.class,
            ask.seed,
            ask.personal.version,
        );
        self.rows.get_with(key, || {
            Arc::new(self.build_rows(data, ask, shape, slot, wanted))
        })
    }

    fn build_rows(
        &self,
        data: &DataSet,
        ask: &ScreenAsk,
        shape: &Shape,
        slot: u64,
        wanted: usize,
    ) -> Vec<Row> {
        let set = self.facts(data);
        let composed = set.requests.version >= 2;
        let plan = PagePlan::new(
            self.cap(data, shape, composed),
            shape.spec.is_some(),
            shape.language.is_some(),
            wanted,
        );
        let taste = if ask.personal.taste.is_empty() {
            vec![0.0; data.vectors.dims]
        } else {
            data.vault.blend_taste(&ask.personal.taste)
        };
        let seed = if ask.seed.is_empty() {
            String::new()
        } else {
            format!("{}|{slot}", ask.seed)
        };
        let page = PageAsk {
            surface: shape.surface.word(),
            genre: ask.genre.as_deref().filter(|_| shape.spec.is_some()),
            genre_spec: shape.spec.as_ref(),
            language: shape.language.as_deref(),
            tier: shape.tier,
            late_night: false,
            taste,
            target_rows: plan.target_rows,
            session: &seed,
            signals: &ask.personal.signals,
            now_ms: ask.now_ms,
        };
        let context = PageContext::new(data, &set.requests, page);
        match shape.collection.as_deref() {
            Some(id) if id.starts_with("kids_") => {
                context.build_kids_collection(id).map(|found| found.0)
            }
            Some(id) => context.build_mood_collection(id).map(|found| found.0),
            None => Some(context.build_page()),
        }
        .unwrap_or_default()
    }

    /// The rows of `new`.
    fn new_rows(data: &DataSet, tier: Tier, now_ms: u64) -> Vec<Row> {
        crate::new_popular::new_popular_rows(data, tier, server_catalog::fresh::days_of_ms(now_ms))
    }

    fn tile(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        row: &Row,
        key: u32,
        rank: Option<u32>,
        ask: &ScreenAsk,
    ) -> Option<SafeTile> {
        let title = viewer.see(data, key)?;
        let item = title.item();
        let release = data
            .fresh
            .get(&item.media_type, item.title_id)
            .map(|fresh| fresh.release_date.as_str());
        let today = server_catalog::fresh::days_of_ms(ask.now_ms);
        let progress = (row.archetype == "continue")
            .then(|| ask.personal.progress.get(&key))
            .flatten();
        let extras = TileExtras {
            badges: item_badges(rank, release, today),
            tags: item_tags(data.profiles.get(item.id), &item.genres, 3),
            episode: progress.and_then(|(_, episode)| episode.clone()),
            percent: progress.map(|(percent, _)| *percent),
        };
        SafeTile::new(title, &self.images, extras)
    }

    /// Rows with the seen titles out, titles shared out, short rows dropped, and each name checked
    /// on what remains. A viewer's own lists keep what it has seen.
    fn share_out(
        data: &DataSet,
        viewer: &Viewer,
        rows: &[Row],
        ask: &ScreenAsk,
        composed: bool,
    ) -> Vec<Shown> {
        let seen = &ask.personal.seen;
        let seen: HashSet<u32> = seen.iter().copied().collect();
        let mut shown_before: HashSet<u32> = HashSet::default();
        let mut out = Vec::new();
        for (position, row) in rows.iter().enumerate() {
            if ask.blocked_rows.contains(&row.title.to_lowercase()) {
                continue;
            }
            let own_list = row.is_own_list();
            let chart = row.archetype == "trending";
            let fresh = |item: &&RowItem| own_list || !seen.contains(&item.id);
            // the blocked names apply here too, so a row is judged on what this viewer will get
            let allowed = |item: &&RowItem| viewer.see(data, item.id).is_some();
            let kept: Vec<u32> = if composed && !chart {
                let (mut head, mut tail, mut moved) = (Vec::new(), Vec::new(), Vec::new());
                for item in row.items.iter().filter(fresh).filter(allowed) {
                    if head.len() >= FIRST_SCREEN {
                        tail.push(item.id);
                    } else if shown_before.contains(&item.id) {
                        moved.push(item.id);
                    } else {
                        head.push(item.id);
                    }
                }
                head.into_iter().chain(tail).chain(moved).collect()
            } else {
                row.items
                    .iter()
                    .filter(fresh)
                    .filter(allowed)
                    .filter(|item| chart || own_list || !shown_before.contains(&item.id))
                    .map(|item| item.id)
                    .collect()
            };
            if kept.len()
                < if own_list {
                    MIN_OWN_ROW_ITEMS
                } else {
                    MIN_ROW_ITEMS
                }
            {
                continue;
            }
            let titles: Vec<&CatalogItem> = kept
                .iter()
                .filter_map(|key| data.catalog.get(key))
                .collect();
            if !crate::row_label::name_fits(&row.title, &titles) {
                continue;
            }
            shown_before.extend(
                kept.iter()
                    .take(if composed { FIRST_SCREEN } else { usize::MAX }),
            );
            out.push(Shown {
                id: format!("{}-{position}", row.archetype),
                position,
                keys: kept,
            });
        }
        out
    }

    fn position(
        &self,
        ask: &ScreenAsk,
        slot: u64,
        section: Option<String>,
        offset: usize,
    ) -> Cursor {
        Cursor(self.key.write(&Position {
            owner: ask.owner.clone(),
            screen: screen_word(ask.screen),
            genre: ask.genre.clone(),
            slot,
            section,
            offset: u32::try_from(offset).unwrap_or(u32::MAX),
        }))
    }

    fn section(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        ask: &ScreenAsk,
        slot: u64,
        rows: &[Row],
        shown: &Shown,
    ) -> SafeSection {
        let row = &rows[shown.position];
        let ranked = row.archetype == "trending";
        let tiles: Vec<SafeTile> = shown
            .keys
            .iter()
            .take(ROW_PAGE)
            .enumerate()
            .filter_map(|(n, key)| {
                let rank = ranked.then(|| u32::try_from(n + 1).unwrap_or(u32::MAX));
                self.tile(data, viewer, row, *key, rank, ask)
            })
            .collect();
        let kind = match row.archetype.as_str() {
            "trending" => SectionKind::Ranked,
            "continue" => SectionKind::Continue,
            _ => SectionKind::Row,
        };
        let cursor = (shown.keys.len() > ROW_PAGE)
            .then(|| self.position(ask, slot, Some(shown.id.clone()), ROW_PAGE));
        SafeSection::new(
            shown.id.clone(),
            kind,
            Some(self.words.row_title(row.meta.as_ref(), &row.title)),
            tiles,
            cursor,
        )
    }

    fn hero(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        ask: &ScreenAsk,
        shape: &Shape,
        slot: u64,
    ) -> Option<SafeSection> {
        let seed = if ask.seed.is_empty() {
            String::new()
        } else {
            format!("{}|{slot}", ask.seed)
        };
        let keys = crate::hero::hero_keys(
            data,
            &crate::hero::HeroAsk {
                tier: shape.tier,
                surface: shape.surface.word(),
                genre: shape.spec.as_ref(),
                session: &seed,
                limit: crate::hero::HERO_DEFAULT,
                seen: &ask.personal.seen,
            },
        );
        let row = Row {
            title: String::new(),
            archetype: String::from("hero"),
            kind: String::new(),
            items: Vec::new(),
            meta: None,
        };
        let tiles: Vec<SafeTile> = keys
            .iter()
            .filter_map(|key| self.tile(data, viewer, &row, *key, None, ask))
            .collect();
        (!tiles.is_empty())
            .then(|| SafeSection::new(String::from("hero"), SectionKind::Hero, None, tiles, None))
    }

    fn wanted(ask: &ScreenAsk, shape: &Shape) -> usize {
        let genre = ask.genre.is_some() && (shape.spec.is_some() || shape.language.is_some());
        if genre {
            INITIAL_ROWS_GENRE
        } else {
            INITIAL_ROWS
        }
    }

    /// One screen for this viewer.
    ///
    /// # Errors
    /// `NotFound` for a name or genre with no page, `Refused` for an event out of season, and `Off`
    /// for a feature the operator left off.
    pub fn screen(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        ask: &ScreenAsk,
    ) -> Result<SafeScreen, PageError> {
        match ask.screen {
            ScreenName::Search | ScreenName::Help | ScreenName::Settings => {
                return Err(PageError::NotFound);
            }
            ScreenName::Games => return Err(PageError::Off("games")),
            ScreenName::Clips => return Err(PageError::Off("clips")),
            ScreenName::MyList | ScreenName::Activity | ScreenName::Notifications => {
                return self.keyed_screen(data, viewer, ask);
            }
            ScreenName::Mine => return Ok(self.mine_screen(data, viewer, ask)),
            _ => {}
        }
        let shape = if ask.screen == ScreenName::New {
            Shape {
                surface: Surface::Both,
                spec: None,
                language: None,
                collection: None,
                tier: viewer.tier(),
            }
        } else {
            Self::shape(data, viewer, ask)?
        };
        let slot = slot_of(ask.now_ms);
        let (rows, shown) = self.rows_of_viewer(data, viewer, ask, &shape, slot);
        let per_page = Self::wanted(ask, &shape);
        let mut sections = Vec::new();
        if ask.screen != ScreenName::New && shape.collection.is_none() {
            sections.extend(self.hero(data, viewer, ask, &shape, slot));
        }
        for row in shown.iter().take(per_page) {
            sections.push(self.section(data, viewer, ask, slot, &rows, row));
        }
        let cursor = (shown.len() > per_page).then(|| self.position(ask, slot, None, per_page));
        Ok(SafeScreen::new(
            ask.screen,
            sections,
            SCREEN_TTL_S,
            refresh_triggers(),
            cursor,
        ))
    }

    fn rows_of_viewer(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        ask: &ScreenAsk,
        shape: &Shape,
        slot: u64,
    ) -> (KeptRows, Vec<Shown>) {
        let rows = if ask.screen == ScreenName::New {
            Arc::new(Self::new_rows(data, shape.tier, ask.now_ms))
        } else {
            self.rows(data, ask, shape, slot, ROWS_CAP.max(MIN_ROWS_FLOOR))
        };
        let composed = self.facts(data).requests.version >= 2;
        let shown = Self::share_out(data, viewer, &rows, ask, composed);
        (rows, shown)
    }

    /// The viewer's own page: a row each for what they are watching, saved and liked.
    fn mine_screen(&self, data: &DataSet, viewer: &Viewer, ask: &ScreenAsk) -> SafeScreen {
        let signals = &ask.personal.signals;
        let parts = [
            (
                "continue",
                "row.continue_watching",
                SectionKind::Continue,
                "continue",
                &signals.recent,
            ),
            (
                "my-list",
                "row.my_list",
                SectionKind::Row,
                "list",
                &signals.saved,
            ),
            (
                "liked",
                "row.liked",
                SectionKind::Row,
                "liked",
                &signals.liked,
            ),
        ];
        let sections = parts
            .into_iter()
            .filter_map(|(id, key, kind, archetype, keys)| {
                let row = Row {
                    title: String::new(),
                    archetype: String::from(archetype),
                    kind: String::new(),
                    items: Vec::new(),
                    meta: None,
                };
                let tiles: Vec<SafeTile> = keys
                    .iter()
                    .filter_map(|title| self.tile(data, viewer, &row, *title, None, ask))
                    .collect();
                (!tiles.is_empty()).then(|| {
                    SafeSection::new(String::from(id), kind, self.words.known(key), tiles, None)
                })
            })
            .collect();
        SafeScreen::new(ask.screen, sections, SCREEN_TTL_S, refresh_triggers(), None)
    }

    /// A screen whose titles come from the library or the fresh list.
    fn keyed_screen(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        ask: &ScreenAsk,
    ) -> Result<SafeScreen, PageError> {
        let keys: Vec<u32> = match ask.screen {
            ScreenName::MyList => ask.personal.signals.saved.clone(),
            ScreenName::Activity => ask.personal.seen.clone(),
            _ => notification_keys(data, ask.now_ms),
        };
        let row = Row {
            title: String::new(),
            archetype: String::from(if ask.screen == ScreenName::Activity {
                "continue"
            } else {
                "grid"
            }),
            kind: String::new(),
            items: Vec::new(),
            meta: None,
        };
        let tiles: Vec<SafeTile> = keys
            .iter()
            .filter_map(|key| self.tile(data, viewer, &row, *key, None, ask))
            .collect();
        let section = SafeSection::new(String::from("grid"), SectionKind::Grid, None, tiles, None);
        Ok(SafeScreen::new(
            ask.screen,
            vec![section],
            SCREEN_TTL_S,
            refresh_triggers(),
            None,
        ))
    }

    /// The next part of a screen or a row. Filtered again at read, so a cursor grants nothing.
    ///
    /// # Errors
    /// `BadCursor` for a forged cursor or one written for another caller.
    pub fn more(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        text: &str,
        ask: &ScreenAsk,
    ) -> Result<SafeMore, PageError> {
        let at = self
            .key
            .read(text, &ask.owner)
            .map_err(|error| match error {
                CursorError::Forged | CursorError::NotYours => PageError::BadCursor,
            })?;
        let screen = screen_of_word(&at.screen).ok_or(PageError::BadCursor)?;
        let ask = ScreenAsk {
            screen,
            genre: at.genre.clone(),
            ..ask.clone()
        };
        let shape = if screen == ScreenName::New {
            Shape {
                surface: Surface::Both,
                spec: None,
                language: None,
                collection: None,
                tier: viewer.tier(),
            }
        } else {
            Self::shape(data, viewer, &ask)?
        };
        let (rows, shown) = self.rows_of_viewer(data, viewer, &ask, &shape, at.slot);
        let offset = at.offset as usize;
        if let Some(section) = &at.section {
            let row = shown
                .iter()
                .find(|row| row.id == *section)
                .ok_or(PageError::BadCursor)?;
            let end = (offset + ROW_PAGE).min(row.keys.len());
            let source = &rows[row.position];
            let ranked = source.archetype == "trending";
            let tiles: Vec<SafeTile> = row
                .keys
                .iter()
                .enumerate()
                .take(end)
                .skip(offset)
                .filter_map(|(n, key)| {
                    let rank = ranked.then(|| u32::try_from(n + 1).unwrap_or(u32::MAX));
                    self.tile(data, viewer, source, *key, rank, &ask)
                })
                .collect();
            let cursor = (row.keys.len() > end)
                .then(|| self.position(&ask, at.slot, Some(section.clone()), end));
            return Ok(SafeMore::new(Vec::new(), tiles, cursor));
        }
        let per_page = Self::wanted(&ask, &shape).min(MIN_ROWS_FLOOR);
        let end = (offset + per_page).min(shown.len());
        let sections = shown[offset.min(end)..end]
            .iter()
            .map(|row| self.section(data, viewer, &ask, at.slot, &rows, row))
            .collect();
        let cursor = (shown.len() > end).then(|| self.position(&ask, at.slot, None, end));
        Ok(SafeMore::new(sections, Vec::new(), cursor))
    }

    /// Search results as a screen.
    #[must_use]
    pub fn search(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        query: &str,
        ask: &ScreenAsk,
    ) -> SafeScreen {
        let set = self.facts(data);
        let keys = search_keys(
            data,
            &set.titles,
            &set.aliases,
            viewer.tier(),
            query,
            SEARCH_TITLES,
        );
        let row = Row {
            title: String::new(),
            archetype: String::from("search"),
            kind: String::new(),
            items: Vec::new(),
            meta: None,
        };
        let tiles: Vec<SafeTile> = keys
            .iter()
            .filter_map(|key| self.tile(data, viewer, &row, *key, None, ask))
            .collect();
        let section = SafeSection::new(
            String::from("results"),
            SectionKind::Grid,
            None,
            tiles,
            None,
        );
        SafeScreen::new(
            ScreenName::Search,
            vec![section],
            SCREEN_TTL_S,
            Vec::new(),
            None,
        )
    }

    /// Search suggestions: names that start with what was typed, best first, or the recommended
    /// mix before anything was typed.
    #[must_use]
    pub fn suggest(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        query: &str,
        ask: &ScreenAsk,
    ) -> SafeSuggestions {
        const LIMIT: usize = 12;
        let keys = if query.trim().is_empty() {
            let today = server_catalog::taxonomy::month_day(ask.now_ms);
            let events = crate::personal::events_in_season(&data.taxonomy, &data.vibe, today);
            crate::suggest::suggest_keys(
                data,
                &crate::suggest::SuggestAsk {
                    tier: viewer.tier(),
                    session: if ask.seed.is_empty() { "-" } else { &ask.seed },
                    slot: slot_of(ask.now_ms),
                    today_days: server_catalog::fresh::days_of_ms(ask.now_ms),
                    limit: LIMIT,
                    taste: &ask.personal.taste,
                    seen: &ask.personal.seen,
                    season_events: &events,
                },
            )
        } else {
            let set = self.facts(data);
            search_keys(data, &set.titles, &set.aliases, viewer.tier(), query, LIMIT)
        };
        let titles: Vec<Visible<'_>> = keys
            .iter()
            .filter_map(|key| viewer.see(data, *key))
            .collect();
        SafeSuggestions::new(&titles)
    }

    /// A title to show beside a profile on the picker, before anyone has entered it. The seed keeps
    /// one choice for each profile.
    #[must_use]
    pub fn feature(&self, data: &DataSet, viewer: &Viewer, seed: &str) -> Option<SafeTile> {
        let keys = crate::hero::hero_keys(
            data,
            &crate::hero::HeroAsk {
                tier: viewer.tier(),
                surface: "both",
                genre: None,
                session: seed,
                limit: 1,
                seen: &[],
            },
        );
        let other = viewer.see(data, *keys.first()?)?;
        let extras = TileExtras {
            tags: item_tags(data.profiles.get(*keys.first()?), &other.item().genres, 3),
            ..TileExtras::default()
        };
        SafeTile::new(other, &self.images, extras)
    }

    /// A title page: its facts and More Like This, hidden when empty.
    #[must_use]
    pub fn title_page(
        &self,
        data: &DataSet,
        viewer: &Viewer,
        title: Visible<'_>,
        system: RatingSystem,
    ) -> Option<SafeTitleDetail> {
        let anchor = title.item().id;
        let age = age_mark(system, title.item().cert_gb, title.item().cert_us);
        let keys = crate::more_like::more_like_keys(
            data,
            viewer.tier(),
            anchor,
            crate::more_like::DEFAULT_LIMIT,
        );
        let more_like_this = keys
            .iter()
            .filter_map(|key| {
                let other = viewer.see(data, *key)?;
                let extras = TileExtras {
                    tags: item_tags(data.profiles.get(*key), &other.item().genres, 3),
                    ..TileExtras::default()
                };
                SafeTile::new(other, &self.images, extras)
            })
            .collect();
        SafeTitleDetail::new(
            title,
            &self.images,
            DetailExtras {
                age,
                seasons: Vec::new(),
                more_like_this,
            },
        )
    }
}

/// One line of `server check pages`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageCheck {
    pub id: String,
    pub surface: &'static str,
    pub cap: usize,
    pub offered: bool,
}

impl Pages {
    /// How every genre page stands for a tier, so an operator sees which pages can fill.
    #[must_use]
    pub fn check_pages(&self, data: &DataSet, tier: Tier) -> Vec<PageCheck> {
        let composed = self.facts(data).requests.version >= 2;
        crate::genre_pages::PAGES
            .iter()
            .filter(|page| page.kids == tier.is_kids())
            .map(|page| {
                let shape = Shape {
                    surface: page.surface,
                    spec: Some(resolve(page.name, page.surface.word(), &data.events)),
                    language: None,
                    collection: None,
                    tier,
                };
                let cap = self.cap(data, &shape, composed);
                PageCheck {
                    id: page.id.to_owned(),
                    surface: page.surface.word(),
                    cap,
                    offered: PagePlan::new(cap, true, false, INITIAL_ROWS_GENRE).supported(),
                }
            })
            .collect()
    }
}

/// One entry of a menu, offered from eight rows, an event only in season.
fn nav_item(id: &str, label: &str, screen: ScreenName, genre: Option<&str>) -> NavItem {
    NavItem {
        id: id.to_owned(),
        label: label.to_owned(),
        screen,
        genre: genre.map(str::to_owned),
    }
}

/// A fixed menu entry. Its words are `nav.{id}` in the strings files.
fn named_item(words: &Words, id: &str, screen: ScreenName) -> NavItem {
    nav_item(id, &words.say(&format!("nav.{id}")), screen, None)
}

impl Pages {
    /// The menu for this tier. Kids get their own home and pages.
    #[must_use]
    pub fn nav(&self, data: &DataSet, tier: Tier) -> Nav {
        let kids = tier.is_kids();
        let mut items = if kids {
            vec![named_item(&self.words, "kids", ScreenName::Kids)]
        } else {
            vec![named_item(&self.words, "home", ScreenName::Home)]
        };
        items.extend([
            named_item(&self.words, "series", ScreenName::Series),
            named_item(&self.words, "films", ScreenName::Films),
            named_item(&self.words, "new", ScreenName::New),
        ]);
        if !kids {
            items.push(named_item(&self.words, "games", ScreenName::Games));
        }
        items.push(named_item(&self.words, "mine", ScreenName::Mine));
        let composed = self.facts(data).requests.version >= 2;
        for page in crate::genre_pages::PAGES
            .iter()
            .filter(|page| page.kids == kids)
        {
            let spec = resolve(page.name, page.surface.word(), &data.events);
            let shape = Shape {
                surface: page.surface,
                spec: Some(spec),
                language: None,
                collection: None,
                tier,
            };
            let plan = PagePlan::new(
                self.cap(data, &shape, composed),
                true,
                false,
                INITIAL_ROWS_GENRE,
            );
            if plan.supported() {
                items.push(nav_item(
                    page.id,
                    page.name,
                    ScreenName::Genre,
                    Some(page.id),
                ));
            }
        }
        Nav {
            items,
            ttl_s: SCREEN_TTL_S,
        }
    }
}

/// Released in the last 14 days, newest first, then due within 14 days.
fn notification_keys(data: &DataSet, now_ms: u64) -> Vec<u32> {
    let today = server_catalog::fresh::days_of_ms(now_ms);
    let mut dated: Vec<(i64, u32)> = data
        .fresh
        .items
        .iter()
        .filter_map(|fresh| {
            let day = server_catalog::fresh::parse_days(&fresh.release_date)?;
            let key = server_catalog::catalog::item_key(&fresh.media_type, fresh.id);
            let art = data.catalog.get(&key)?.has_poster();
            (art && (today - day).abs() <= 14).then_some((day, key))
        })
        .collect();
    dated.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let (past, due): (Vec<_>, Vec<_>) = dated.into_iter().partition(|(day, _)| *day <= today);
    let mut keys: Vec<u32> = past.into_iter().map(|(_, key)| key).collect();
    keys.extend(due.into_iter().rev().map(|(_, key)| key));
    keys.truncate(NOTIFICATIONS_MAX);
    keys
}

fn refresh_triggers() -> Vec<RefreshTrigger> {
    vec![
        RefreshTrigger::Focus,
        RefreshTrigger::ProfileChange,
        RefreshTrigger::ListChange,
        RefreshTrigger::HistoryChange,
    ]
}

/// One row as this viewer gets it.
struct Shown {
    id: String,
    position: usize,
    keys: Vec<u32>,
}
