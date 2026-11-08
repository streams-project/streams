//! The age tier of a profile.

use server_accounts::{Profile, ProfileKind, ProfileScope};
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
        Self::of_parts(scope.age_limit(), scope.maturity(), scope.kind())
    }

    /// The tier of a profile row, by the same rule as [`Tier::of_scope`], for answers given before
    /// anyone has entered a profile.
    ///
    /// # Errors
    /// Neither value can be read.
    pub fn of_profile(row: &Profile) -> Result<Self, RuleError> {
        Self::of_parts(&row.age_limit, &row.maturity, row.kind)
    }

    fn of_parts(age_limit: &str, maturity: &str, kind: ProfileKind) -> Result<Self, RuleError> {
        let tier = Self::from_rating(age_limit)
            .or_else(|| Self::from_maturity(maturity))
            .ok_or(RuleError)?;
        Ok(if kind == ProfileKind::Kids && tier > Self::OlderKids {
            Self::OlderKids
        } else {
            tier
        })
    }

    /// The tier a screen shown to a whole account keeps to: the most open tier among its grown up
    /// profiles, or among its Kids profiles when it has no other. Profiles whose limit cannot be
    /// read are left out, and an account with no readable profile gets none.
    #[must_use]
    pub fn of_account(rows: &[Profile]) -> Option<Self> {
        let readable = || {
            rows.iter()
                .filter_map(|row| Self::of_profile(row).ok().map(|tier| (row.kind, tier)))
        };
        readable()
            .filter(|(kind, _)| *kind != ProfileKind::Kids)
            .map(|(_, tier)| tier)
            .max()
            .or_else(|| readable().map(|(_, tier)| tier).max())
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

#[cfg(test)]
mod tests {
    use server_accounts::{AccountId, Profile, ProfileId, ProfileKind};

    use super::Tier;

    fn row(kind: ProfileKind, limit: &str) -> Profile {
        Profile {
            id: ProfileId::new(),
            account: AccountId::new(),
            name: String::from("P"),
            avatar: String::from("default"),
            kind,
            is_main: false,
            auto_kids: false,
            has_pin: false,
            sort_order: 0,
            age_limit: limit.to_owned(),
            maturity: String::new(),
            blocked_names: Vec::new(),
        }
    }

    #[test]
    fn an_account_keeps_to_its_most_open_grown_up_profile() {
        let rows = [
            row(ProfileKind::Adult, "12"),
            row(ProfileKind::Adult, "15"),
            row(ProfileKind::Kids, "PG"),
        ];
        assert_eq!(Tier::of_account(&rows), Some(Tier::Fifteen));
    }

    #[test]
    fn an_account_of_kids_profiles_keeps_to_the_kids_tier() {
        let rows = [row(ProfileKind::Kids, "U"), row(ProfileKind::Kids, "18")];
        assert_eq!(Tier::of_account(&rows), Some(Tier::OlderKids));
        let little = [row(ProfileKind::Kids, "U")];
        assert_eq!(Tier::of_account(&little), Some(Tier::LittleKids));
    }

    #[test]
    fn an_unreadable_limit_is_left_out_and_never_reads_as_adult() {
        let rows = [row(ProfileKind::Adult, "??"), row(ProfileKind::Kids, "PG")];
        assert_eq!(Tier::of_account(&rows), Some(Tier::OlderKids));
        assert_eq!(Tier::of_account(&[row(ProfileKind::Adult, "??")]), None);
    }
}
