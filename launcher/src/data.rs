//! Which data set this install runs on.
//!
//! Three rules live here and nowhere else. `data/` belongs to the operator, so a set found there
//! is theirs. The sample's public key, whose private half is a constant in this repository that
//! anyone can read, is never written under the name a real set is checked against, so no real set
//! can inherit the sample's trust root. And a sample this program itself left under the operator's
//! names, before the sample had a folder of its own, is moved aside and written fresh: nobody
//! should have to delete a file by hand to recover from a state this program created.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use server_catalog::manifest::DatasetManifest;

/// The operator's own set: a manifest and the key it was signed with.
const MANIFEST: &str = "manifest.json";
const KEY: &str = "data_key.pem";
/// The sample's folder and its own key name, both apart from the operator's.
const SAMPLE: &str = "data-sample";
const SAMPLE_KEY: &str = "sample_key.pem";
/// What the generated sample writes into its own manifest. A manifest naming both of these was
/// built by this repository's sample generator and by no operator's data build, which is what lets
/// this program account for a sample key it finds at the operator's path instead of asking a
/// person to clear it. The pair is pinned against the generator by
/// `the_sample_still_names_itself_the_way_this_file_reads_it`, so changing it there turns this file
/// red rather than turning the healing quietly off.
const SAMPLE_BUILD: &str = "sample";
const SAMPLE_VERSION: &str = "sample-1";
/// What a file moved out of the operator's way is called afterwards, as for the seed film list.
const ASIDE: &str = ".sample";

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

/// What stands at the operator's path, as one answer.
///
/// It is an enumeration and not an `Option` so that a caller cannot reach the sample without
/// having been told what was found and what was done about it: the reseeded case carries the two
/// paths and exists to be said out loud.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    /// The operator's own set, with the key it is checked against.
    Operator(Data),
    /// Nothing of the operator's there yet, so the sample stands in.
    Nothing,
    /// This program's own older sample, which it had written under the operator's names. Both
    /// files have been moved to these paths and the sample is to be written fresh under its own.
    Reseeded { manifest: PathBuf, key: PathBuf },
}

/// The same name with `.sample` after it, so a second start does not read it as a set.
fn aside(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(OsString::new, std::ffi::OsStr::to_os_string);
    name.push(ASIDE);
    path.with_file_name(name)
}

/// True when this manifest is the generated sample's own.
///
/// It is read with the same type the server checks a set with, so a file this binary cannot parse
/// is not the sample and the key beside it is not accounted for.
fn is_sample_manifest(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<DatasetManifest>(&text).ok())
        .is_some_and(|manifest| {
            manifest.build_commit == SAMPLE_BUILD && manifest.dataset_version == SAMPLE_VERSION
        })
}

/// What this install runs on, healing the one state this program is known to have created.
///
/// # Errors
/// `data/data_key.pem` holds the published sample key, which would accept a set signed by anyone,
/// and the manifest beside it is not the sample's own. That pair is not one this program wrote, so
/// the start stops rather than moving an operator's file. Or a file could not be read or moved.
pub fn read(home: &Path, sample_key: &str) -> Result<Found> {
    let data = home.join("data");
    let (manifest, key) = (data.join(MANIFEST), data.join(KEY));
    if !(manifest.exists() && key.exists()) {
        return Ok(Found::Nothing);
    }
    if std::fs::read_to_string(&key)?.trim() != sample_key.trim() {
        return Ok(Found::Operator(Data {
            manifest,
            key,
            films: home.join("films.json"),
            sample: false,
        }));
    }
    if !is_sample_manifest(&manifest) {
        bail!(
            "{} holds the published sample key, so it would accept a set signed by anyone, and \
             the manifest beside it is not the sample's own. Put this install's own key there.",
            key.display()
        );
    }
    // Moved, not deleted: it is the operator's folder, and whoever wants to see what stood there
    // can read it back. The key goes first, so a start interrupted between the two moves leaves a
    // manifest with no key, which reads as no set at all, and never a sample key on its own that
    // a later real manifest would be checked against.
    let (manifest_aside, key_aside) = (aside(&manifest), aside(&key));
    let moved = |from: &Path, to: &Path| {
        std::fs::rename(from, to)
            .with_context(|| format!("{} could not be moved aside", from.display()))
    };
    moved(&key, &key_aside)?;
    moved(&manifest, &manifest_aside)?;
    Ok(Found::Reseeded {
        manifest: manifest_aside,
        key: key_aside,
    })
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

    fn sample_pem() -> String {
        server_catalog::sample::sample_key_pem()
    }

    fn home(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("data-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(folder.join("data")).expect("a home");
        folder
    }

    /// The sample's own manifest, written by the generator this program calls.
    fn sample_manifest_text(home: &Path) -> String {
        let built = home.join("built");
        let path = server_catalog::sample::write_sample(&built);
        std::fs::read_to_string(path).expect("the sample manifest")
    }

    #[test]
    fn with_nothing_in_the_data_folder_there_is_no_operator_set() {
        let home = home("empty");
        assert_eq!(
            read(&home, &sample_pem()).expect("read"),
            Found::Nothing,
            "a manifest alone is not a set"
        );
        std::fs::write(home.join("data/manifest.json"), "{}").expect("a manifest");
        assert_eq!(read(&home, &sample_pem()).expect("read"), Found::Nothing);
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
        let Found::Operator(found) = read(&home, &sample_pem()).expect("read") else {
            panic!("the operator's own set was not used");
        };
        assert_eq!(found.key, home.join("data/data_key.pem"));
        assert_eq!(found.films, home.join("films.json"));
        assert!(!found.sample);
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn an_install_left_holding_the_sample_under_the_operators_names_heals_itself() {
        let home = home("left-holding");
        // Exactly the state an older version of this program left behind, and the state the
        // owner's own install had been in since 8 November: the sample's manifest and the
        // published sample key, both under the names a real set is read from.
        let text = sample_manifest_text(&home);
        std::fs::write(home.join("data/manifest.json"), &text).expect("a manifest");
        std::fs::write(home.join("data/data_key.pem"), sample_pem()).expect("a key");
        let Found::Reseeded { manifest, key } = read(&home, &sample_pem()).expect("read") else {
            panic!("the start refused a state this program created");
        };
        assert_eq!(manifest, home.join("data/manifest.json.sample"));
        assert_eq!(key, home.join("data/data_key.pem.sample"));
        assert!(
            !home.join("data/data_key.pem").exists(),
            "the published sample key is still the operator's trust root"
        );
        assert!(!home.join("data/manifest.json").exists());
        assert!(manifest.exists() && key.exists(), "both were deleted");
        // And the next start finds nothing of the operator's, with nothing left to move.
        assert_eq!(read(&home, &sample_pem()).expect("read"), Found::Nothing);
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn the_published_sample_key_over_a_set_this_program_did_not_write_is_refused() {
        let home = home("sample-key");
        // A real build's manifest, or anything this binary cannot read as the sample: the key is
        // then one this program cannot account for, and moving an operator's file is not its call.
        for manifest in [
            String::from("{}"),
            sample_manifest_text(&home).replace("\"sample-1\"", "\"2026-11-11-1\""),
        ] {
            std::fs::write(home.join("data/manifest.json"), &manifest).expect("a manifest");
            std::fs::write(home.join("data/data_key.pem"), sample_pem()).expect("a key");
            let why = read(&home, &sample_pem()).expect_err("refused").to_string();
            assert!(why.contains("published sample key"), "{why}");
            assert!(
                home.join("data/data_key.pem").exists(),
                "a refusal moved the operator's file anyway"
            );
        }
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn the_sample_keeps_its_own_folder_and_key_name() {
        let (folder, key) = sample_at(Path::new("/tmp/home"));
        assert_ne!(folder, Path::new("/tmp/home/data"));
        assert_ne!(key.file_name(), Some(std::ffi::OsStr::new(KEY)));
    }

    /// The two words above are copied from the sample generator, which this crate does not own.
    /// If they ever stop matching, the healing above would quietly stop recognising the state it
    /// exists for and installs would go back to refusing to start, so it is pinned here.
    #[test]
    fn the_sample_still_names_itself_the_way_this_file_reads_it() {
        let home = home("pinned");
        let path = home.join("data/manifest.json");
        std::fs::write(&path, sample_manifest_text(&home)).expect("a manifest");
        assert!(
            is_sample_manifest(&path),
            "the sample generator no longer writes build_commit {SAMPLE_BUILD} and \
             dataset_version {SAMPLE_VERSION}"
        );
        std::fs::remove_dir_all(&home).expect("cleaned");
    }
}
