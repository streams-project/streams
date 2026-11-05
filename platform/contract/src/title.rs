//! Title identity: a kind plus an id. The two number ranges overlap, so a bare id means nothing.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

/// Most keys read from one comma separated list.
pub const MAX_KEYS_PER_LIST: usize = 40;

/// Highest title id. The top bit of a packed key holds the kind.
pub const MAX_TITLE_ID: u32 = (1 << 31) - 1;

const TV_BIT: u32 = 1 << 31;

/// Completed means this percentage of the duration or more.
const COMPLETED_PERCENT: u64 = 95;

/// The two kinds of title. Closed on purpose: a new kind reaches old clients only through a
/// dated transform, so a key always names a kind this build can open.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TitleKind {
    Movie,
    Tv,
}

impl TitleKind {
    /// The word used in keys and addresses.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Tv => "tv",
        }
    }

    fn from_word(word: &str) -> Option<Self> {
        match word {
            "movie" => Some(Self::Movie),
            "tv" => Some(Self::Tv),
            _ => None,
        }
    }
}

/// Why a title key was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TitleKeyError {
    /// No `kind:` part, which includes a bare id.
    NoKind,
    /// The kind word is not one of the known kinds.
    BadKind,
    /// The id is not plain digits from 1 to `MAX_TITLE_ID`.
    BadId,
}

impl fmt::Display for TitleKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoKind => "a title key needs a kind, as in movie:123",
            Self::BadKind => "the kind of a title key is movie or tv",
            Self::BadId => "the id of a title key is digits from 1 to 2147483647",
        })
    }
}

impl core::error::Error for TitleKeyError {}

/// `movie:<id>` or `tv:<id>`. The fields are private, so a key with no kind or id 0 cannot exist.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TitleKey {
    kind: TitleKind,
    id: u32,
}

impl TitleKey {
    /// Build a key from its two parts.
    ///
    /// # Errors
    /// `BadId` when the id is 0 or above `MAX_TITLE_ID`.
    pub const fn new(kind: TitleKind, id: u32) -> Result<Self, TitleKeyError> {
        if id == 0 || id > MAX_TITLE_ID {
            return Err(TitleKeyError::BadId);
        }
        Ok(Self { kind, id })
    }

    /// Read one key, strictly.
    ///
    /// # Errors
    /// Any text other than `movie:<id>` or `tv:<id>`, which includes a bare id.
    pub fn parse(text: &str) -> Result<Self, TitleKeyError> {
        let (word, digits) = text.split_once(':').ok_or(TitleKeyError::NoKind)?;
        let kind = TitleKind::from_word(word).ok_or(TitleKeyError::BadKind)?;
        let plain = !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
            && !digits.starts_with('0');
        if !plain {
            return Err(TitleKeyError::BadId);
        }
        let id = digits.parse().map_err(|_| TitleKeyError::BadId)?;
        Self::new(kind, id)
    }

    /// Read a comma separated list: spaces and case around a key are forgiven, junk is skipped,
    /// repeats are removed with the first place kept, and at most `MAX_KEYS_PER_LIST` are read.
    #[must_use]
    pub fn parse_list(list: &str) -> Vec<Self> {
        let mut keys: Vec<Self> = Vec::new();
        for part in list.split(',') {
            let Ok(key) = Self::parse(&part.trim().to_ascii_lowercase()) else {
                continue;
            };
            if !keys.contains(&key) {
                keys.push(key);
            }
            if keys.len() >= MAX_KEYS_PER_LIST {
                break;
            }
        }
        keys
    }

    /// The kind of this title.
    #[must_use]
    pub const fn kind(self) -> TitleKind {
        self.kind
    }

    /// The id of this title, which means nothing without its kind.
    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }

    /// The key as one number for server indexes: the id, with the top bit set for tv.
    /// Never sent on the wire.
    #[must_use]
    pub const fn packed(self) -> u32 {
        match self.kind {
            TitleKind::Tv => self.id | TV_BIT,
            TitleKind::Movie => self.id,
        }
    }

    /// Read a number made by `packed`.
    ///
    /// # Errors
    /// `BadId` when the id part is 0.
    pub const fn from_packed(packed: u32) -> Result<Self, TitleKeyError> {
        let kind = if packed & TV_BIT == 0 {
            TitleKind::Movie
        } else {
            TitleKind::Tv
        };
        Self::new(kind, packed & !TV_BIT)
    }

    /// Key of the watch record of this title: `movie_<id>` or `tv_<id>`.
    #[must_use]
    pub fn record_key(self) -> String {
        format!("{}_{}", self.kind.word(), self.id)
    }

    /// Key of one episode of this series: `tv_<series>_<season>_<episode>`.
    #[must_use]
    pub fn episode_key(self, season: u32, episode: u32) -> String {
        format!("{}_{season}_{episode}", self.record_key())
    }
}

impl fmt::Display for TitleKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind.word(), self.id)
    }
}

impl Serialize for TitleKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for TitleKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyText;

        impl Visitor<'_> for KeyText {
            type Value = TitleKey;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a title key as text, movie:<id> or tv:<id>")
            }

            fn visit_str<E: de::Error>(self, text: &str) -> Result<TitleKey, E> {
                TitleKey::parse(text).map_err(|error| E::custom(error.to_string()))
            }
        }

        deserializer.deserialize_str(KeyText)
    }
}

wire! {
    /// One episode of a series. A film has none.
    pub struct EpisodeRef {
        pub season: u32,
        pub episode: u32,
    }
}

words! {
    /// Rating words on the wire. Screens use the words like, love and dislike.
    pub enum Rating {
        Down,
        Up,
        Love,
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for TitleKey {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "TitleKey".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": "^(movie|tv):[1-9][0-9]{0,9}$",
            "description": "movie:<id> or tv:<id>. A bare id does not parse."
        })
    }
}

impl Rating {
    /// The wire value for a word a screen uses.
    #[must_use]
    pub fn from_client_word(word: &str) -> Option<Self> {
        match word {
            "dislike" => Some(Self::Down),
            "like" => Some(Self::Up),
            "love" => Some(Self::Love),
            _ => None,
        }
    }

    /// The word a screen uses for this value.
    #[must_use]
    pub const fn client_word(self) -> Option<&'static str> {
        match self {
            Self::Down => Some("dislike"),
            Self::Up => Some("like"),
            Self::Love => Some("love"),
            Self::Unknown => None,
        }
    }
}

/// Whole percent watched, clamped to 0 to 100. `None` when the duration is 0.
#[must_use]
pub fn watch_percent(position_ms: u64, duration_ms: u64) -> Option<u8> {
    if duration_ms == 0 {
        return None;
    }
    let percent = u128::from(position_ms) * 100 / u128::from(duration_ms);
    Some(u8::try_from(percent.min(100)).unwrap_or(100))
}

/// Completed means 95 percent or more of a known duration.
#[must_use]
pub fn is_completed(position_ms: u64, duration_ms: u64) -> bool {
    duration_ms > 0
        && u128::from(position_ms) * 100 >= u128::from(duration_ms) * u128::from(COMPLETED_PERCENT)
}
