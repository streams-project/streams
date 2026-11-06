//! Pages. The server decides what appears and in what order; the device decides how it looks.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Notice;
use crate::title::{EpisodeRef, TitleKey};

/// An opaque position in a list. Clients hand it back and never read it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct Cursor(pub String);

words! {
    /// The screens the server builds.
    pub enum ScreenName as "kebab-case" {
        Home,
        Films,
        Series,
        Genre,
        New,
        MyList,
        Kids,
        Search,
        Notifications,
        Activity,
        Games,
        Clips,
        Language,
    }
}

words! {
    /// Section kinds. A kind this build does not draw falls back to a plain row.
    pub enum SectionKind {
        Hero,
        Row,
        Ranked,
        Continue,
        Grid,
        Notice,
    }
}

impl SectionKind {
    /// The kind to draw. Unknown kinds become a plain row.
    #[must_use]
    pub const fn drawn_as(self) -> Self {
        match self {
            Self::Unknown => Self::Row,
            known => known,
        }
    }
}

words! {
    /// What an image is for.
    pub enum ImageRole {
        Poster,
        Backdrop,
        Logo,
        Still,
        Avatar,
    }
}

words! {
    /// The colour family of an age mark. The server picks it and each app paints it.
    pub enum AgeTone {
        Green,
        Yellow,
        Orange,
        Pink,
        Red,
        DarkRed,
        Blue,
        Grey,
    }
}

words! {
    /// Marks on a tile.
    pub enum BadgeKind {
        Ranked,
        New,
    }
}

words! {
    /// Events after which a screen should be read again.
    pub enum RefreshTrigger {
        Focus,
        SignIn,
        ProfileChange,
        ListChange,
        HistoryChange,
    }
}

wire! {
    /// One image address. Images go straight from the host to the viewer.
    pub struct Image {
        pub role: ImageRole,
        pub url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub width: Option<u32>,
    }

    /// A mark on a tile.
    pub struct Badge {
        pub kind: BadgeKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rank: Option<u32>,
    }

    /// One title in a section.
    pub struct Tile {
        pub title: TitleKey,
        pub name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub images: Vec<Image>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub badges: Vec<Badge>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub tags: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub episode: Option<EpisodeRef>,
        /// Whole percent watched.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub percent: Option<u32>,
    }

    /// One row, hero, grid or notice of a screen.
    pub struct Section {
        pub id: String,
        pub kind: SectionKind,
        /// Translated title.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub tiles: Vec<Tile>,
        /// Reads more of this section through `/v1/more/{cursor}`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<Cursor>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub notice: Option<Notice>,
    }

    /// A built page.
    pub struct Screen {
        pub name: ScreenName,
        pub sections: Vec<Section>,
        /// Seconds this answer stays good.
        pub ttl_s: u32,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub refresh_on: Vec<RefreshTrigger>,
        /// Reads more sections through `/v1/more/{cursor}`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<Cursor>,
    }

    /// The next part of a screen or a section.
    pub struct More {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub sections: Vec<Section>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub tiles: Vec<Tile>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<Cursor>,
    }

    /// One entry of the menu.
    pub struct NavItem {
        pub id: String,
        pub label: String,
        pub screen: ScreenName,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub genre: Option<String>,
    }

    /// The menu for this class and profile.
    pub struct Nav {
        pub items: Vec<NavItem>,
        pub ttl_s: u32,
    }

    /// One episode.
    pub struct Episode {
        pub number: u32,
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub overview: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub runtime_min: Option<u32>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub images: Vec<Image>,
        /// Whole percent watched.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub percent: Option<u32>,
    }

    /// One season with its episodes.
    pub struct Season {
        pub number: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        pub episodes: Vec<Episode>,
    }

    /// A season named on a title page.
    pub struct SeasonSummary {
        pub number: u32,
        pub episode_count: u32,
    }

    /// An age mark, as this viewer reads it. Each screen picks its shape.
    pub struct AgeMark {
        pub label: String,
        pub tone: AgeTone,
    }

    /// Facts of one title.
    pub struct TitleDetail {
        pub title: TitleKey,
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub overview: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub year: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub runtime_min: Option<u32>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub genres: Vec<String>,
        /// Absent when the title has no rating in either system.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub age: Option<AgeMark>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub images: Vec<Image>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub seasons: Vec<SeasonSummary>,
        /// Hidden when empty.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub more_like_this: Vec<Tile>,
    }

    /// Search suggestions.
    pub struct Suggestions {
        pub items: Vec<String>,
    }
}
