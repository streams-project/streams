//! Every response type that can hold a title or an image. Wire types are public in the contract,
//! so any crate could fill one in. These wrappers can only be made here, from `Visible`, and the
//! server sends only [`Cleared`] values, so an unchecked title has no way out.

use contract::command::Changes;
use contract::error::Notice;
use contract::play::{AgeCard, Attempt, PlayAnswer};
use contract::screen::{
    AgeMark, Badge, Cursor, Episode, Image, ImageRole, More, RefreshTrigger, Screen, ScreenName,
    Season, SeasonSummary, Section, SectionKind, Suggestions, Tile, TitleDetail,
};
use contract::{EpisodeRef, TitleKey, TitleKind};
use serde::Serialize;
use server_catalog::catalog::CatalogItem;

use crate::viewer::Visible;

mod sealed {
    pub trait Sealed {}
}

/// A value that passed the rules and may leave the server.
pub trait Cleared: Serialize + sealed::Sealed {}

macro_rules! cleared {
    ($($(#[$meta:meta])* $name:ident($inner:ty);)*) => {
        $(
            $(#[$meta])*
            #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
            #[serde(transparent)]
            pub struct $name($inner);

            impl sealed::Sealed for $name {}
            impl Cleared for $name {}

            impl $name {
                /// The wire value, read only.
                #[must_use]
                pub const fn wire(&self) -> &$inner {
                    &self.0
                }
            }
        )*
    };
}

cleared! {
    /// A `Tile` built from a visible title.
    SafeTile(Tile);
    /// A `Section` of safe tiles.
    SafeSection(Section);
    /// A `Screen` of safe sections.
    SafeScreen(Screen);
    /// A `More` of safe sections or tiles.
    SafeMore(More);
    /// A `TitleDetail` of a visible title.
    SafeTitleDetail(TitleDetail);
    /// A `Season` of a visible series.
    SafeSeason(Season);
    /// A `PlayAnswer` for a visible title.
    SafePlayAnswer(PlayAnswer);
    /// Suggestions drawn from visible titles.
    SafeSuggestions(Suggestions);
    /// `Changes` naming only visible titles.
    SafeChanges(Changes);
}

/// Where images come from. Each deployment sets its own host; with none, images are left out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Images {
    base: Option<String>,
}

impl Images {
    /// Images under a base address, or none.
    #[must_use]
    pub fn new(base: Option<&str>) -> Self {
        Self {
            base: base.map(|text| text.trim_end_matches('/').to_owned()),
        }
    }

    fn image(&self, role: ImageRole, path: Option<&str>) -> Option<Image> {
        let (base, path) = (self.base.as_deref()?, path.filter(|p| !p.is_empty())?);
        Some(Image {
            role,
            url: format!("{base}{path}"),
            width: None,
        })
    }

    /// The picture of one episode, if it has one.
    #[must_use]
    pub fn still(&self, path: Option<&str>) -> Vec<Image> {
        self.image(ImageRole::Still, path).into_iter().collect()
    }

    fn of_title(&self, item: &CatalogItem) -> Vec<Image> {
        [
            self.image(ImageRole::Poster, item.poster_path.as_deref()),
            self.image(ImageRole::Backdrop, item.backdrop_path.as_deref()),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// The wire key of a title.
#[must_use]
pub fn title_key(item: &CatalogItem) -> Option<TitleKey> {
    let kind = if item.media_type == "tv" {
        TitleKind::Tv
    } else {
        TitleKind::Movie
    };
    TitleKey::new(kind, item.title_id).ok()
}

/// What a page adds to a tile.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileExtras {
    pub badges: Vec<Badge>,
    pub tags: Vec<String>,
    pub episode: Option<EpisodeRef>,
    pub percent: Option<u32>,
}

impl SafeTile {
    /// A tile for a visible title.
    #[must_use]
    pub fn new(title: Visible<'_>, images: &Images, extras: TileExtras) -> Option<Self> {
        let item = title.item();
        Some(Self(Tile {
            title: title_key(item)?,
            name: item.title.clone(),
            images: images.of_title(item),
            badges: extras.badges,
            tags: extras.tags,
            episode: extras.episode,
            percent: extras.percent,
        }))
    }
}

fn wires(tiles: Vec<SafeTile>) -> Vec<Tile> {
    tiles.into_iter().map(|tile| tile.0).collect()
}

impl SafeSection {
    /// A row, hero, grid or ranked section.
    #[must_use]
    pub fn new(
        id: String,
        kind: SectionKind,
        title: Option<String>,
        tiles: Vec<SafeTile>,
        cursor: Option<Cursor>,
    ) -> Self {
        Self(Section {
            id,
            kind,
            title,
            tiles: wires(tiles),
            cursor,
            notice: None,
        })
    }

    /// A notice section, which holds no title.
    #[must_use]
    pub fn notice(id: String, notice: Notice) -> Self {
        Self(Section {
            id,
            kind: SectionKind::Notice,
            title: None,
            tiles: Vec::new(),
            cursor: None,
            notice: Some(notice),
        })
    }
}

impl SafeScreen {
    /// A built page.
    #[must_use]
    pub fn new(
        name: ScreenName,
        sections: Vec<SafeSection>,
        ttl_s: u32,
        refresh_on: Vec<RefreshTrigger>,
        cursor: Option<Cursor>,
    ) -> Self {
        Self(Screen {
            name,
            sections: sections.into_iter().map(|section| section.0).collect(),
            ttl_s,
            refresh_on,
            cursor,
        })
    }
}

impl SafeMore {
    /// The next part of a screen or section.
    #[must_use]
    pub fn new(sections: Vec<SafeSection>, tiles: Vec<SafeTile>, cursor: Option<Cursor>) -> Self {
        Self(More {
            sections: sections.into_iter().map(|section| section.0).collect(),
            tiles: wires(tiles),
            cursor,
        })
    }
}

/// Facts of a title page that are not on the title row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DetailExtras {
    pub age: Option<AgeMark>,
    pub seasons: Vec<SeasonSummary>,
    pub more_like_this: Vec<SafeTile>,
}

impl SafeTitleDetail {
    /// The title page of a visible title.
    #[must_use]
    pub fn new(title: Visible<'_>, images: &Images, extras: DetailExtras) -> Option<Self> {
        let item = title.item();
        Some(Self(TitleDetail {
            title: title_key(item)?,
            name: item.title.clone(),
            overview: (!item.overview.is_empty()).then(|| item.overview.clone()),
            year: item.release_year.and_then(|year| u32::try_from(year).ok()),
            runtime_min: (item.runtime > 0).then_some(item.runtime),
            genres: item.genres.clone(),
            age: extras.age,
            images: images.of_title(item),
            seasons: extras.seasons,
            more_like_this: wires(extras.more_like_this),
            in_list: false,
            rating: None,
        }))
    }

    /// Names the seasons a series has.
    #[must_use]
    pub fn with_seasons(mut self, seasons: Vec<SeasonSummary>) -> Self {
        self.0.seasons = seasons;
        self
    }

    /// Says whether this profile has saved the title.
    #[must_use]
    pub fn saved(mut self, in_list: bool) -> Self {
        self.0.in_list = in_list;
        self
    }

    /// Adds what this profile said about the title.
    #[must_use]
    pub fn rated(mut self, rating: Option<contract::Rating>) -> Self {
        self.0.rating = rating;
        self
    }
}

impl SafeSeason {
    /// One season of a visible series.
    #[must_use]
    pub fn new(
        _series: Visible<'_>,
        number: u32,
        name: Option<String>,
        episodes: Vec<Episode>,
    ) -> Self {
        Self(Season {
            number,
            name,
            episodes,
        })
    }
}

/// The parts of a play answer that are not attempts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayExtras {
    pub play_id: String,
    pub start_at_ms: u64,
    pub age_card: Option<AgeCard>,
    pub audio_default: Option<String>,
    pub subtitle_default: Option<String>,
    pub next_episode: Option<EpisodeRef>,
    pub autoplay_next: bool,
    pub dropped: u32,
    pub more: bool,
}

impl SafePlayAnswer {
    /// A play answer for a title that passed the gate.
    #[must_use]
    pub fn new(_title: Visible<'_>, attempts: Vec<Attempt>, extras: PlayExtras) -> Self {
        Self(PlayAnswer {
            play_id: extras.play_id,
            attempts,
            start_at_ms: extras.start_at_ms,
            age_card: extras.age_card,
            audio_default: extras.audio_default,
            subtitle_default: extras.subtitle_default,
            next_episode: extras.next_episode,
            autoplay_next: extras.autoplay_next,
            dropped: extras.dropped,
            more: extras.more,
        })
    }
}

impl SafeSuggestions {
    /// Names of visible titles.
    #[must_use]
    pub fn new(titles: &[Visible<'_>]) -> Self {
        Self(Suggestions {
            items: titles
                .iter()
                .map(|title| title.item().title.clone())
                .collect(),
        })
    }
}

impl SafeChanges {
    /// Changes that passed a filter. The `keep` closure lives in the pages crate and holds the
    /// viewer; history changes name no title.
    #[must_use]
    pub fn new(mut changes: Changes, keep: impl Fn(&TitleKey) -> bool) -> Self {
        use contract::command::Change;
        changes.changes.retain(|change| match change {
            Change::Title { title, .. } | Change::Hidden { title } => keep(title),
            Change::Progress { state } => keep(&state.title),
            _ => true,
        });
        Self(changes)
    }
}
