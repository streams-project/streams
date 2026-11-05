//! Everything a page builder knows about the request and the catalog.
use super::*;

#[derive(Clone)]
pub struct PageContext<'a> {
    pub data: &'a DataSet,
    pub catalog: &'a CatalogMap,
    pub vectors: &'a VectorStore,
    pub seeds: &'a ClusterSeeds,
    /// World, atom and theme group of every title (single database)
    pub groups: &'a server_catalog::groups::GroupMap,
    pub group_names: &'a server_catalog::groups::GroupNames,
    /// Vibe groups, moods and seasonal sub groups
    pub vibe: &'a server_catalog::vibe::VibeIndex,
    /// The client's local time is between 22:00 and 04:59: late night rows come first
    pub late_night: bool,
    pub vault: &'a CentroidVault,
    pub taxonomy: &'a TaxonomyStore,
    pub profiles: &'a server_catalog::profiles::ProfileStore,
    pub topics: &'a server_catalog::topics::TopicStore,
    /// Release dates of new and upcoming titles, and today's date (YYYY-MM-DD) to compare them with
    pub fresh: &'a server_catalog::fresh::FreshStore,
    pub events: &'a server_catalog::events::EventStore,
    pub today_ymd: String,
    /// Today as days since 1970
    pub today_days: i64,
    /// Original language code (a language page); every row holds only titles in that language
    pub language: Option<&'a str>,
    /// Children's home only: build the page from the animated titles (`Some(true)`) or from the live action ones (`Some(false)`)
    pub animation: Option<bool>,
    /// The Kids home is built from the Kids row vocabulary (`taxonomy` is then that vocabulary): rows chosen by value and variety, none at a fixed place
    pub kids_entries: bool,
    pub surface: &'a str, // "movie" | "tv" | "both"
    /// Optional genre as sent by the client (kept for row titles).
    pub genre: Option<&'a str>,
    /// The rule that genre resolves to, applied to every row (see genre_spec.rs).
    pub genre_spec: Option<&'a crate::genre_spec::GenreSpec>,
    pub taste: Vec<f32>, // blended query vector on unit sphere
    pub target_rows: usize,
    /// Corpus mean vector for centred scoring
    pub mean: &'a [f32],
    /// Today's (month, day) for calendar rows
    pub today: (u8, u8),
    /// Session id: seeds the exploration noise so a session sees a stable page
    pub session: &'a str,
    /// The age tier of this viewer. A page has no way to skip it
    pub tier: Tier,
    /// Fill pass of a thin genre page: the pool is these titles (neighbours of what the page shows) instead of the whole catalog
    pub fill_pool: Option<&'a [u32]>,
    /// Titles a fill pass must not use again (the ones the page already shows)
    pub skip: Option<&'a HashSet<u32>>,
    /// What the client sent about the profile: saved, recent and liked titles (personal rows)
    pub signals: &'a crate::signals::Signals,
    /// Requests rows are made from (empty, or unused while the switch is off)
    pub requests: &'a crate::requests::RequestStore,
}

/// The part of a page context that comes from the request; the rest is the engine's loaded data (`PageContext::new`).
pub struct PageAsk<'a> {
    /// "movie" | "tv" | "both"
    pub surface: &'a str,
    pub genre: Option<&'a str>,
    pub genre_spec: Option<&'a crate::genre_spec::GenreSpec>,
    pub language: Option<&'a str>,
    pub tier: Tier,
    pub late_night: bool,
    /// Blended taste vector on the unit sphere, zeros for a profile without taste
    pub taste: Vec<f32>,
    pub target_rows: usize,
    /// Seeds the exploration noise; empty for an anonymous request
    pub session: &'a str,
    pub signals: &'a crate::signals::Signals,
    /// The time of this page, in milliseconds since 1970
    pub now_ms: u64,
}

impl<'a> PageContext<'a> {
    /// The context of one page: what the data set holds, and what this request asks for.
    #[must_use]
    pub fn new(
        data: &'a DataSet,
        requests: &'a crate::requests::RequestStore,
        ask: PageAsk<'a>,
    ) -> Self {
        // A Kids profile's pages come from the Kids row vocabulary when there is one
        let kids_entries =
            ask.tier.is_kids() && ask.language.is_none() && data.kids_taxonomy.entries.len() >= 60;
        let today_days = server_catalog::fresh::days_of_ms(ask.now_ms);
        Self {
            data,
            catalog: &data.catalog,
            vectors: &data.vectors,
            seeds: &data.seeds,
            groups: &data.groups,
            group_names: &data.group_names,
            vibe: &data.vibe,
            late_night: ask.late_night,
            vault: &data.vault,
            taxonomy: if kids_entries {
                &data.kids_taxonomy
            } else {
                &data.taxonomy
            },
            profiles: &data.profiles,
            topics: &data.topics,
            fresh: &data.fresh,
            events: &data.events,
            today_ymd: server_catalog::fresh::ymd(today_days),
            today_days,
            language: ask.language,
            animation: None,
            kids_entries,
            surface: ask.surface,
            genre: ask.genre,
            genre_spec: ask.genre_spec,
            taste: ask.taste,
            target_rows: ask.target_rows,
            mean: &data.mean,
            today: server_catalog::taxonomy::month_day(ask.now_ms),
            session: ask.session,
            tier: ask.tier,
            fill_pool: None,
            skip: None,
            signals: ask.signals,
            requests,
        }
    }
}
