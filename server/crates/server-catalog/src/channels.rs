//! The channels a data set names as children's channels. The names are data the builder writes,
//! never code: this binary holds no channel name of its own. With no file the list is empty and
//! the channel part of the rule simply does not fire, like every other optional file.

use std::path::Path;

use tracing::warn;

use crate::det::HashSet;

/// Channel names that mark a programme as made for children, folded to lowercase so a set written
/// with another capitalisation still matches.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KidsChannels {
    names: HashSet<String>,
}

impl KidsChannels {
    /// The list from a JSON array of names. A file that does not read leaves the list empty, and
    /// the reason is logged.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        let read = std::fs::read_to_string(path)
            .map_err(|error| error.to_string())
            .and_then(|text| {
                serde_json::from_str::<Vec<String>>(&text).map_err(|error| error.to_string())
            });
        match read {
            Ok(names) => Self::from_names(names),
            Err(error) => {
                warn!("Children's channels left out: {error}");
                Self::default()
            }
        }
    }

    /// The list from names in memory, for a fixture or a test.
    #[must_use]
    pub fn from_names<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            names: names
                .into_iter()
                .map(|name| name.as_ref().trim().to_lowercase())
                .filter(|name| !name.is_empty())
                .collect(),
        }
    }

    /// Does the set name this channel a children's channel?
    #[must_use]
    pub fn holds(&self, name: &str) -> bool {
        !self.names.is_empty() && self.names.contains(&name.trim().to_lowercase())
    }

    /// How many channels the set names.
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// True when the set names none, so the channel part of the rule cannot fire.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_matches_whatever_its_case_and_an_empty_list_matches_nothing() {
        let list = KidsChannels::from_names(["Some Kids Channel", " other kids channel "]);
        assert_eq!(list.len(), 2);
        assert!(list.holds("some kids channel"));
        assert!(list.holds("Other Kids Channel"));
        assert!(!list.holds("Some Other Channel"));
        let empty = KidsChannels::default();
        assert!(empty.is_empty());
        assert!(!empty.holds("Some Kids Channel"));
    }

    #[test]
    fn a_file_that_is_not_a_list_of_names_leaves_the_list_empty() {
        let dir = std::env::temp_dir().join(format!("channels_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("folder");
        let path = dir.join("channels.json");
        std::fs::write(&path, r#"{"channels": ["x"]}"#).expect("file");
        assert!(KidsChannels::load(&path).is_empty());
        std::fs::write(&path, r#"["A Channel", "B Channel"]"#).expect("file");
        assert_eq!(KidsChannels::load(&path).len(), 2);
        assert!(KidsChannels::load(&dir.join("missing.json")).is_empty());
    }
}
