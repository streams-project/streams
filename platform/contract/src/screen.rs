//! Pages. The server decides what appears and in what order; the device decides how it looks.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::Notice;
use crate::title::{EpisodeRef, Rating, TitleKey, TitleKind};

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
        /// The viewer's own page: what they are watching, saved and liked.
        Mine,
        Kids,
        Search,
        Notifications,
        Activity,
        Games,
        Clips,
        Language,
        /// Help. Its text comes from the help routes, so it has no page of rows.
        Help,
        /// The viewer's settings, which also have no page of rows.
        Settings,
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
        /// A tall picture of the title with no words on it, to fill a portrait screen.
        Art,
    }
}

/// A day of the calendar, `YYYY-MM-DD`, as the place a title comes from writes it. The app writes
/// it out in the viewer's own words.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct CalendarDay(String);

/// A text that is not a calendar day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadCalendarDay;

impl fmt::Display for BadCalendarDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a calendar day is YYYY-MM-DD")
    }
}

impl core::error::Error for BadCalendarDay {}

impl CalendarDay {
    /// Read a day: four digits, a month from 01 to 12 and a day from 01 to 31.
    ///
    /// # Errors
    /// Text of another shape.
    pub fn parse(text: &str) -> Result<Self, BadCalendarDay> {
        let two = |a: u8, b: u8| {
            (a.is_ascii_digit() && b.is_ascii_digit()).then(|| (a - b'0') * 10 + (b - b'0'))
        };
        let fits = match text.as_bytes() {
            [y1, y2, y3, y4, b'-', m1, m2, b'-', d1, d2] => {
                [y1, y2, y3, y4].iter().all(|b| b.is_ascii_digit())
                    && two(*m1, *m2).is_some_and(|month| (1..=12).contains(&month))
                    && two(*d1, *d2).is_some_and(|day| (1..=31).contains(&day))
            }
            _ => false,
        };
        if fits {
            Ok(Self(String::from(text)))
        } else {
            Err(BadCalendarDay)
        }
    }

    /// The day as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for CalendarDay {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for CalendarDay {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "CalendarDay".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": "^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$",
            "description": "A day of the calendar, YYYY-MM-DD. The app writes it out in the viewer's own words."
        })
    }
}

words! {
    /// What the line under a title's logo on the profile chooser tells. The app writes the words.
    pub enum GateLineKind {
        /// The title comes out on `date`.
        Coming,
        /// A new season of the series starts on `date`.
        SeasonsComing,
        /// The title is number `rank` among today's most watched of its `media`.
        Ranked,
        /// No line.
        None,
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
        /// True when this profile has saved the title.
        #[serde(default, skip_serializing_if = "core::ops::Not::not")]
        pub in_list: bool,
        /// What this profile said about the title.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rating: Option<Rating>,
        /// Where this profile left off. Absent when there is nothing to go back to.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub resume: Option<Resume>,
    }

    /// Where play picks up: the episode of a series, and 0 for one not started yet.
    pub struct Resume {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub episode: Option<EpisodeRef>,
        pub position_s: u32,
    }

    /// Search suggestions.
    pub struct Suggestions {
        pub items: Vec<String>,
    }

    /// Titles a visitor may look at before signing in, most watched first. A series also carries
    /// pictures of its first episodes, so a television can show the title as a slideshow.
    pub struct PublicTitles {
        pub titles: Vec<TitleDetail>,
    }

    /// The facts of the line under a title's logo on the profile chooser. Never words: the app
    /// writes them in the viewer's language, with the date in its own month names and digits.
    pub struct GateLine {
        pub kind: GateLineKind,
        /// The day, for `coming` and `seasons_coming`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub date: Option<CalendarDay>,
        /// The place from 1, for `ranked`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rank: Option<u32>,
        /// Films or series, for `ranked`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub media: Option<TitleKind>,
    }

    /// One title shown over the profile chooser: its tile, with art, logo and poster, and its line.
    pub struct GateTitle {
        pub tile: Tile,
        pub line: GateLine,
    }

    /// Titles shown over the profile chooser before anyone has chosen a profile, at most three, in
    /// order. Chosen for the whole account: never above the most open tier among its profiles.
    pub struct ProfileGate {
        pub titles: Vec<GateTitle>,
        /// Seconds this answer stays good.
        pub ttl_s: u32,
    }
}
