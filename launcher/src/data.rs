//! Which data set this install runs on.
//!
//! Two rules live here and nowhere else. `data/` belongs to the operator and this program never
//! writes in it, so a set found there is theirs. And the sample's public key, whose private half
//! is a constant in this repository that anyone can read, is never written under the name a real
//! set is checked against, so no real set can inherit the sample's trust root.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

/// The operator's own set: a manifest and the key it was signed with.
const MANIFEST: &str = "manifest.json";
const KEY: &str = "data_key.pem";
/// The sample's folder and its own key name, both apart from the operator's.
const SAMPLE: &str = "data-sample";
const SAMPLE_KEY: &str = "sample_key.pem";

/// The data set in use and the film list that goes with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    pub manifest: PathBuf,
    pub key: PathBuf,
    /// The film list the finders read.
    pub films: PathBuf,
    /// True when this is the generated sample.
    pub sample: bool,
}

/// The operator's set, if they have put one in `data/`.
///
/// # Errors
/// `data/data_key.pem` holds the sample's published key, which would trust any set at all. An
/// install that once started without a catalogue was left with exactly that file under exactly
/// that name, so finding it is a stop and not a warning.
pub fn operator_set(home: &Path, sample_key: &str) -> Result<Option<Data>> {
    let data = home.join("data");
    let (manifest, key) = (data.join(MANIFEST), data.join(KEY));
    if !(manifest.exists() && key.exists()) {
        return Ok(None);
    }
    if std::fs::read_to_string(&key)?.trim() == sample_key.trim() {
        bail!(
            "{} holds the published sample key, so it would accept a set signed by anyone.\n\
             Put this install's own key there, or delete the file and the manifest beside it to \
             run on the sample again.",
            key.display()
        );
    }
    Ok(Some(Data {
        manifest,
        key,
        films: home.join("films.json"),
        sample: false,
    }))
}

/// The sample's folder and the name its key goes under. Nothing here shares a name with the
/// operator's set, which is what makes the two impossible to confuse.
#[must_use]
pub fn sample_at(home: &Path) -> (PathBuf, PathBuf) {
    let folder = home.join(SAMPLE);
    let key = folder.join(SAMPLE_KEY);
    (folder, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PEM: &str = "-----BEGIN PUBLIC KEY-----\nsample\n-----END PUBLIC KEY-----\n";

    fn home(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("data-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(folder.join("data")).expect("a home");
        folder
    }

    #[test]
    fn with_nothing_in_the_data_folder_there_is_no_operator_set() {
        let home = home("empty");
        assert_eq!(
            operator_set(&home, SAMPLE_PEM).expect("read"),
            None,
            "a manifest alone is not a set"
        );
        std::fs::write(home.join("data/manifest.json"), "{}").expect("a manifest");
        assert_eq!(operator_set(&home, SAMPLE_PEM).expect("read"), None);
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn the_operators_own_key_is_used_with_their_own_film_list() {
        let home = home("own");
        std::fs::write(home.join("data/manifest.json"), "{}").expect("a manifest");
        std::fs::write(
            home.join("data/data_key.pem"),
            "-----BEGIN PUBLIC KEY-----\nmine\n",
        )
        .expect("a key");
        let found = operator_set(&home, SAMPLE_PEM)
            .expect("read")
            .expect("a set");
        assert_eq!(found.key, home.join("data/data_key.pem"));
        assert_eq!(found.films, home.join("films.json"));
        assert!(!found.sample);
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn the_published_sample_key_is_never_accepted_as_a_trust_root() {
        let home = home("sample-key");
        std::fs::write(home.join("data/manifest.json"), "{}").expect("a manifest");
        std::fs::write(home.join("data/data_key.pem"), SAMPLE_PEM).expect("a key");
        let why = operator_set(&home, SAMPLE_PEM)
            .expect_err("refused")
            .to_string();
        assert!(why.contains("published sample key"), "{why}");
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn the_sample_keeps_its_own_folder_and_key_name() {
        let (folder, key) = sample_at(Path::new("/tmp/home"));
        assert_ne!(folder, Path::new("/tmp/home/data"));
        assert_ne!(key.file_name(), Some(std::ffi::OsStr::new(KEY)));
    }
}
