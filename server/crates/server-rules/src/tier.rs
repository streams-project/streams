//! The age tier of a profile.

use server_accounts::{ProfileKind, ProfileScope};
use thiserror::Error;

/// A tier, narrowest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    LittleKids,
    OlderKids,
    Twelve,
    Fifteen,
    Adult,
}

/// A profile whose limit cannot be read gets nothing, and the call answers 503, never Adult.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("this profile's age limit cannot be read")]
pub struct RuleError;

impl Tier {
    /// From the coarse level kept on older rows.
    #[must_use]
    pub fn from_maturity(word: &str) -> Option<Self> {
        match word {
            "little_kids" => Some(Self::LittleKids),
            "older_kids" => Some(Self::OlderKids),
            "teens" | "twelve" => Some(Self::Twelve),
            "fifteen" => Some(Self::Fifteen),
            "adult" => Some(Self::Adult),
            _ => None,
        }
    }

    /// From a British age rating as the profile saves it: U, PG, 12, 15 or 18.
    #[must_use]
    pub fn from_rating(word: &str) -> Option<Self> {
        match word.trim().to_ascii_uppercase().as_str() {
            "U" => Some(Self::LittleKids),
            "PG" => Some(Self::OlderKids),
            "12" | "12A" => Some(Self::Twelve),
            "15" => Some(Self::Fifteen),
            "18" => Some(Self::Adult),
            _ => None,
        }
    }

    /// The tier in force for a profile: the saved limit wins over the older level, and a Kids
    /// profile never rises above PG.
    ///
    /// # Errors
    /// Neither value can be read.
    pub fn of_scope(scope: &ProfileScope) -> Result<Self, RuleError> {
        let saved = Self::from_rating(scope.age_limit());
        let tier = saved
            .or_else(|| Self::from_maturity(scope.maturity()))
            .ok_or(RuleError)?;
        let kids = scope.kind() == ProfileKind::Kids;
        Ok(if kids && tier > Self::OlderKids {
            Self::OlderKids
        } else {
            tier
        })
    }

    /// The highest certificate rank this tier may see (1 U, 2 PG, 3 12, 4 15, 5 18).
    #[must_use]
    pub const fn max_cert(self) -> u8 {
        match self {
            Self::LittleKids => 1,
            Self::OlderKids => 2,
            Self::Twelve => 3,
            Self::Fifteen => 4,
            Self::Adult => 5,
        }
    }

    /// The two tiers of children's profiles.
    #[must_use]
    pub const fn is_kids(self) -> bool {
        matches!(self, Self::LittleKids | Self::OlderKids)
    }

    /// The word used in cache keys.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LittleKids => "little_kids",
            Self::OlderKids => "older_kids",
            Self::Twelve => "twelve",
            Self::Fifteen => "fifteen",
            Self::Adult => "adult",
        }
    }
}
