//! A generated sample has invented titles and no films of its own, so a fresh install gives each
//! of them one open short film. Then Play works before anyone has set up a thing.
//!
//! The list covers exactly the titles the sample holds. It used to cover every id up to a
//! thousand, which stood the one test film in front of real titles an operator's own catalogue
//! numbers the same way.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{Value, json};

/// Big Buck Bunny by the Blender Foundation, as a public test stream.
const OPEN_FILM: &str = "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8";
const OPEN_FILM_LICENCE: &str = "CC-BY-3.0";

/// One title of the sample: its kind and its id.
pub type Title = (String, u32);

fn list(titles: &[Title]) -> Value {
    let entries: Vec<Value> = titles
        .iter()
        .map(|(kind, id)| {
            json!({
                "title": format!("{kind}:{id}"),
                "url": OPEN_FILM,
                "kind": "hls",
                "licence": OPEN_FILM_LICENCE,
                "every_episode": kind == "tv",
            })
        })
        .collect();
    Value::Array(entries)
}

/// Writes a film for each of these titles, unless this install has films of its own.
///
/// # Errors
/// The file could not be read or written.
pub fn seed(path: &Path, titles: &[Title]) -> Result<()> {
    if path.exists() && std::fs::read_to_string(path)?.trim() != "[]" {
        return Ok(());
    }
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder)?;
    }
    std::fs::write(path, list(titles).to_string())?;
    Ok(())
}

/// True when this list holds nothing but rows this file wrote: the one open film, under a title
/// made of a kind and an id. Nothing else writes that shape, so a list of them is the seed and not
/// an operator's own work. A list with one row of their own is theirs and is left alone.
fn only_seed(text: &str) -> bool {
    let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(text) else {
        return false;
    };
    if rows.is_empty() {
        return false;
    }
    rows.iter().all(|row| {
        let seeded_title = row["title"].as_str().is_some_and(|title| {
            title.split_once(':').is_some_and(|(kind, id)| {
                matches!(kind, "movie" | "tv")
                    && !id.is_empty()
                    && id.bytes().all(|b| b.is_ascii_digit())
            })
        });
        seeded_title && row["url"] == OPEN_FILM && row["licence"] == OPEN_FILM_LICENCE
    })
}

/// Moves a film list aside when it holds nothing but this file's seed, and says where it went.
///
/// An install that once started without a catalogue was left with one test film under every id an
/// older version seeded. Those ids are the numbers a real catalogue gives its own titles, and the
/// finder that reads this list answers before the catalogue does, so until the file goes the test
/// film stands in front of real films. Deciding it fresh each start is what makes this reach an
/// install that already has the file, rather than only a new one.
///
/// # Errors
/// The file could not be read, or could not be moved aside.
pub fn retire_seed(path: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    if !only_seed(&std::fs::read_to_string(path)?) {
        return Ok(None);
    }
    // Moved, not deleted: it is the operator's folder, and a list that was written there once can
    // be read back by whoever wants to see what it held.
    let aside = path.with_extension("json.sample");
    std::fs::rename(path, &aside)?;
    Ok(Some(aside))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles() -> Vec<Title> {
        vec![
            (String::from("movie"), 100),
            (String::from("tv"), 102),
            (String::from("movie"), 9001),
        ]
    }

    #[test]
    fn a_fresh_install_gets_a_film_for_every_sample_title_and_no_others() {
        let folder = std::env::temp_dir().join(format!("films-seed-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("films.json");
        std::fs::write(&path, "[]").unwrap();
        seed(&path, &titles()).unwrap();
        let written: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let entries = written.as_array().unwrap();
        let named: Vec<&str> = entries
            .iter()
            .filter_map(|entry| entry["title"].as_str())
            .collect();
        assert_eq!(named, ["movie:100", "tv:102", "movie:9001"]);
        assert!(entries.iter().all(|e| e["licence"] == OPEN_FILM_LICENCE));
        // An id the sample does not hold gets no film, so this list cannot stand in front of a
        // real catalogue's own title with that number.
        assert!(!named.contains(&"movie:1"));
        assert_eq!(entries[1]["every_episode"], true, "a series stands in once");
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn an_install_that_already_holds_the_old_seed_has_it_moved_aside() {
        let folder = std::env::temp_dir().join(format!("films-retire-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("films.json");
        // What an older version left behind: one test film for every id up to a thousand, which is
        // how a real catalogue numbers its own titles.
        let old: Vec<Title> = (1..=600u32)
            .map(|id| (String::from("movie"), id))
            .chain((1..=600u32).map(|id| (String::from("tv"), id)))
            .collect();
        std::fs::write(&path, list(&old).to_string()).unwrap();
        let aside = retire_seed(&path).unwrap().expect("moved aside");
        assert!(
            !path.exists(),
            "the seed still stands in front of the catalogue"
        );
        assert_eq!(aside, folder.join("films.json.sample"));
        assert!(aside.exists(), "the old list was deleted rather than kept");
        // Nothing to do the next time, and nothing to say.
        assert_eq!(retire_seed(&path).unwrap(), None);
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_list_with_any_row_of_the_operators_own_is_left_where_it_is() {
        let folder = std::env::temp_dir().join(format!("films-own-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("films.json");
        let mut rows = list(&titles());
        rows.as_array_mut().unwrap().push(json!({
            "title": "movie:7",
            "url": "https://media.example.invalid/mine.m3u8",
            "kind": "hls",
            "licence": "CC0-1.0",
            "every_episode": false,
        }));
        std::fs::write(&path, rows.to_string()).unwrap();
        assert_eq!(retire_seed(&path).unwrap(), None, "the operator's own list");
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("mine.m3u8")
        );
        // An empty list is what a start writes when there is nothing to serve, not a seed.
        std::fs::write(&path, "[]").unwrap();
        assert_eq!(retire_seed(&path).unwrap(), None);
        assert!(path.exists());
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn an_owners_own_list_is_never_replaced() {
        let folder = std::env::temp_dir().join(format!("films-keep-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("films.json");
        std::fs::write(&path, r#"[{"title":"movie:7"}]"#).unwrap();
        seed(&path, &titles()).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains("movie:7"));
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
