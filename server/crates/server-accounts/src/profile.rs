//! Account and profile rows, and the rules that need no store.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DoorError;
use crate::limits::AccountLimits;

macro_rules! id_type {
    ($name:ident, $about:literal) => {
        #[doc = $about]
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        pub struct $name(pub Uuid);

        impl $name {
            /// A new random id.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Read an id from text.
            ///
            /// # Errors
            /// Text that is not a UUID, answered as not found so a guess learns nothing.
            pub fn parse(text: &str) -> Result<Self, DoorError> {
                Uuid::try_parse(text)
                    .map(Self)
                    .map_err(|_| DoorError::NotFound)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(AccountId, "An account.");
id_type!(ProfileId, "A profile of an account.");
id_type!(SessionId, "One device session.");

/// The two kinds of profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    Adult,
    Kids,
}

impl ProfileKind {
    /// The word stored in the row.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Adult => "adult",
            Self::Kids => "kids",
        }
    }

    /// The kind for a stored word. Anything else reads as kids, the narrower kind.
    #[must_use]
    pub fn from_word(word: &str) -> Self {
        if word == "adult" {
            Self::Adult
        } else {
            Self::Kids
        }
    }
}

/// One profile row. The age limit and the blocked names are opaque text here: the rules crate
/// reads them, and this crate stays free of what they mean.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub id: ProfileId,
    pub account: AccountId,
    pub name: String,
    pub avatar: String,
    pub kind: ProfileKind,
    pub is_main: bool,
    pub auto_kids: bool,
    pub has_pin: bool,
    pub sort_order: i16,
    /// The age limit word as saved. Empty when the row has none.
    pub age_limit: String,
    /// The older coarse level, kept for rows with no saved limit.
    pub maturity: String,
    /// Names this profile never sees, as typed.
    pub blocked_names: Vec<String>,
}

/// The limits row, written only by the step up command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileLimitsRow {
    pub age_limit: String,
    pub blocked_names: Vec<String>,
}

/// A change to a profile. Age limit and blocked names are absent on purpose.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfilePatch {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub kind: Option<ProfileKind>,
}

/// What sign up knows about a person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewAccount {
    pub id: AccountId,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

/// A trimmed profile name within limits.
///
/// # Errors `BadValue` outside 1 to the limit after
/// trim.
pub fn clean_profile_name(name: &str, limits: &AccountLimits) -> Result<String, DoorError> {
    let name = name.trim();
    let length = u32::try_from(name.chars().count()).unwrap_or(u32::MAX);
    if length == 0 || length > limits.profile_name_max {
        return Err(DoorError::BadValue("profile_name"));
    }
    Ok(name.to_owned())
}

/// The key two names clash on: trimmed, lower case.
#[must_use]
pub fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// The first profile's name: first name, else the mail's local part, else Me, cut to the
/// limit. The word kids is left to the Kids profile.
#[must_use]
pub fn main_profile_name(first_name: &str, email: &str, limits: &AccountLimits) -> String {
    let first = first_name.trim();
    let local = email.split('@').next().unwrap_or("").trim();
    let base = [first, local, "Me"]
        .into_iter()
        .find(|text| !text.is_empty())
        .unwrap_or("Me");
    let cut: String = base
        .chars()
        .take(limits.profile_name_max as usize)
        .collect();
    if name_key(&cut) == "kids" {
        String::from("Me")
    } else {
        cut
    }
}

/// The two profiles of a new account: main named from the person, then Kids.
#[must_use]
pub fn starting_profiles(account: &NewAccount, limits: &AccountLimits) -> [Profile; 2] {
    [
        Profile {
            id: ProfileId::new(),
            account: account.id,
            name: main_profile_name(&account.first_name, &account.email, limits),
            avatar: String::from("default"),
            kind: ProfileKind::Adult,
            is_main: true,
            auto_kids: false,
            has_pin: false,
            sort_order: 0,
            age_limit: String::from("18"),
            maturity: String::from("adult"),
            blocked_names: Vec::new(),
        },
        Profile {
            id: ProfileId::new(),
            account: account.id,
            name: String::from("Kids"),
            avatar: String::from("kids"),
            kind: ProfileKind::Kids,
            is_main: false,
            auto_kids: true,
            has_pin: false,
            sort_order: 1,
            age_limit: String::from("PG"),
            maturity: String::from("older_kids"),
            blocked_names: Vec::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::DEFAULT_LIMITS;

    #[test]
    fn s37_main_name_falls_back_in_order_and_kids_becomes_me() {
        let l = &DEFAULT_LIMITS;
        assert_eq!(main_profile_name(" Ada ", "x@y.test", l), "Ada");
        assert_eq!(main_profile_name("", "grace@y.test", l), "grace");
        assert_eq!(main_profile_name("", "", l), "Me");
        assert_eq!(main_profile_name("KIDS", "x@y.test", l), "Me");
        assert_eq!(
            main_profile_name(&"a".repeat(40), "", l).chars().count(),
            30
        );
    }

    #[test]
    fn s39_names_are_trimmed_and_limited() {
        let l = &DEFAULT_LIMITS;
        assert_eq!(clean_profile_name("  Ann ", l).unwrap(), "Ann");
        assert!(clean_profile_name("   ", l).is_err());
        assert!(clean_profile_name(&"a".repeat(31), l).is_err());
        assert_eq!(name_key(" ANN "), name_key("ann"));
    }

    #[test]
    fn s123_a_new_account_gets_main_and_kids() {
        let account = NewAccount {
            id: AccountId::new(),
            email: "a@b.test".into(),
            first_name: "Ada".into(),
            last_name: "L".into(),
        };
        let [main, kids] = starting_profiles(&account, &DEFAULT_LIMITS);
        assert!(main.is_main && !main.auto_kids && main.kind == ProfileKind::Adult);
        assert!(kids.auto_kids && !kids.is_main && kids.kind == ProfileKind::Kids);
        assert_eq!(
            (kids.name.as_str(), kids.age_limit.as_str()),
            ("Kids", "PG")
        );
    }
}
