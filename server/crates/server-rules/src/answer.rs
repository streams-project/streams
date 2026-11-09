//! Every response type that can hold a title or an image. Wire types are public in the contract,
//! so any crate could fill one in. These wrappers can only be made here, from `Visible`, and the
//! server sends only [`Cleared`] values, so an unchecked title has no way out.

use contract::command::Changes;
use contract::error::Notice;
use contract::play::{AgeCard, Attempt, PlayAnswer};
use contract::screen::{
    AgeMark, Badge, Cursor, Episode, GateLine, GateTitle, Image, ImageRole, More, ProfileGate,
    PublicTitles, RefreshTrigger, Screen, ScreenName, Season, SeasonSummary, Section, SectionKind,
    Suggestions, Tile, TitleDetail,
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

                /// Take the wire value.
                #[must_use]
                pub fn into_inner(self) -> $inner {
                    self.0
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
    /// Titles a visitor may look at, each a visible title page.
    SafePublicTitles(PublicTitles);
    /// Titles over the profile chooser, each a visible tile with its line.
    SafeProfileGate(ProfileGate);
    /// `Changes` naming only visible titles.
    SafeChanges(Changes);
}

/// Where images come from. Each deployment sets its own host; with none, images are left out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Images {
    base: Option<String>,
    art_base: Option<String>,
}

impl Images {
    /// Images under a base address, or none. When the base address contains `/w780`, art URLs
    /// resolve to `/w1280` so the textless portrait poster fills a tall display without upscaling.
    #[must_use]
    pub fn new(base: Option<&str>) -> Self {
        Self::with_art_base(base, None)
    }

    /// Images under a base address, with an optional separate address for art (`ImageRole::Art`).
    /// If no separate art address is given and the base address contains `/w780`, art URLs
    /// resolve with `/w1280` substituted for `/w780`.
    #[must_use]
    pub fn with_art_base(base: Option<&str>, art_base: Option<&str>) -> Self {
        let base = base.map(|text| text.trim_end_matches('/').to_owned());
        let art_base = art_base
            .map(|text| text.trim_end_matches('/').to_owned())
            .or_else(|| {
                base.as_ref().map(|b| {
                    if b.contains("/w780") {
                        b.replace("/w780", "/w1280")
                    } else {
                        b.clone()
                    }
                })
            });
        Self { base, art_base }
    }

    fn image(&self, role: ImageRole, path: Option<&str>) -> Option<Image> {
        let base = if role == ImageRole::Art {
            self.art_base.as_deref().or(self.base.as_deref())?
        } else {
            self.base.as_deref()?
        };
        let path = path.filter(|p| !p.is_empty())?;
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

    /// The title's poster with no words on it, if it has one.
    fn art(&self, item: &CatalogItem) -> Option<Image> {
        self.image(ImageRole::Art, item.art_path.as_deref())
    }

    fn of_title(&self, item: &CatalogItem) -> Vec<Image> {
        [
            self.image(ImageRole::Poster, item.poster_path.as_deref()),
            self.image(ImageRole::Backdrop, item.backdrop_path.as_deref()),
            // Drawings are left out: a client shows a picture, and the name stands in for it.
            self.image(
                ImageRole::Logo,
                item.logo_path
                    .as_deref()
                    .filter(|path| !path.ends_with(".svg")),
            ),
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

    /// The tile with the title's poster with no words on it, for a screen that fills a tall
    /// display. A title with none keeps the pictures it has.
    #[must_use]
    pub fn with_art(mut self, title: Visible<'_>, images: &Images) -> Self {
        self.0.images.extend(images.art(title.item()));
        self
    }

    /// The tile named as the viewer's language names the title. An empty name keeps the catalog's.
    #[must_use]
    pub fn named(mut self, name: &str) -> Self {
        if !name.trim().is_empty() {
            self.0.name = name.trim().to_owned();
        }
        self
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
    pub available_from: Option<contract::screen::CalendarDay>,
    pub reminder_set: bool,
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
            resume: None,
            available_from: extras.available_from,
            reminder_set: extras.reminder_set,
        }))
    }

    /// The title page in the viewer's language: its name and story where a translation has them
    /// (empty keeps the catalog's English), its genres as the language names them, and the titles
    /// of More Like This named by `name_of`.
    #[must_use]
    pub fn speaking(
        mut self,
        name: &str,
        overview: &str,
        genre: impl Fn(&str) -> String,
        name_of: impl Fn(&TitleKey) -> Option<String>,
    ) -> Self {
        if !name.trim().is_empty() {
            self.0.name = name.trim().to_owned();
        }
        if !overview.trim().is_empty() {
            self.0.overview = Some(overview.trim().to_owned());
        }
        self.0.genres = self.0.genres.iter().map(|g| genre(g)).collect();
        for tile in &mut self.0.more_like_this {
            if let Some(name) = name_of(&tile.title).filter(|name| !name.trim().is_empty()) {
                tile.name = name;
            }
        }
        self
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

    /// Adds where this profile left off.
    #[must_use]
    pub fn resuming(mut self, resume: Option<contract::screen::Resume>) -> Self {
        self.0.resume = resume;
        self
    }

    /// Adds what this profile said about the title.
    #[must_use]
    pub fn rated(mut self, rating: Option<contract::Rating>) -> Self {
        self.0.rating = rating;
        self
    }

    /// Says when this title becomes available.
    #[must_use]
    pub fn coming(mut self, day: Option<contract::screen::CalendarDay>) -> Self {
        self.0.available_from = day;
        self
    }

    /// Says whether this profile set a reminder.
    #[must_use]
    pub fn reminded(mut self, reminded: bool) -> Self {
        self.0.reminder_set = reminded;
        self
    }

    /// Adds pictures of the title's own episodes, after its other pictures.
    #[must_use]
    pub fn with_stills(mut self, stills: Vec<Image>) -> Self {
        self.0.images.extend(
            stills
                .into_iter()
                .filter(|image| image.role == ImageRole::Still),
        );
        self
    }
}

impl SafePublicTitles {
    /// A list of visible title pages, in their order.
    #[must_use]
    pub fn new(titles: Vec<SafeTitleDetail>) -> Self {
        Self(PublicTitles {
            titles: titles.into_iter().map(|title| title.0).collect(),
        })
    }
}

impl SafeProfileGate {
    /// Visible tiles with their lines, in their order.
    #[must_use]
    pub fn new(titles: Vec<(SafeTile, GateLine)>, ttl_s: u32) -> Self {
        Self(ProfileGate {
            titles: titles
                .into_iter()
                .map(|(tile, line)| GateTitle { tile: tile.0, line })
                .collect(),
            ttl_s,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_resolves_art_size_from_poster_base() {
        let images = Images::new(Some("https://images.example/t/p/w780"));
        let poster = images.image(ImageRole::Poster, Some("/p.jpg")).unwrap();
        assert_eq!(poster.url, "https://images.example/t/p/w780/p.jpg");
        let art = images.image(ImageRole::Art, Some("/art.jpg")).unwrap();
        assert_eq!(art.url, "https://images.example/t/p/w1280/art.jpg");
    }

    #[test]
    fn art_resolves_explicit_art_base() {
        let images = Images::with_art_base(
            Some("https://images.example/t/w780"),
            Some("https://custom.example/w1280"),
        );
        let poster = images.image(ImageRole::Poster, Some("/p.jpg")).unwrap();
        assert_eq!(poster.url, "https://images.example/t/w780/p.jpg");
        let art = images.image(ImageRole::Art, Some("/art.jpg")).unwrap();
        assert_eq!(art.url, "https://custom.example/w1280/art.jpg");
    }

    #[test]
    fn art_falls_back_to_base_without_replacement() {
        let images = Images::new(Some("https://images.example/t"));
        let art = images.image(ImageRole::Art, Some("/art.jpg")).unwrap();
        assert_eq!(art.url, "https://images.example/t/art.jpg");
    }
}
