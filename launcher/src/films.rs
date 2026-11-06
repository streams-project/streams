//! A generated sample has invented titles and no films of its own, so a fresh install gives each
//! of them one open short film. Then Play works before anyone has set up a thing.

use std::path::Path;

use anyhow::Result;
use serde_json::{Value, json};

/// Big Buck Bunny by the Blender Foundation, as a public test stream.
const OPEN_FILM: &str = "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8";
const OPEN_FILM_LICENCE: &str = "CC-BY-3.0";
/// More than the sample holds, so an id it adds later still plays.
const SAMPLE_IDS: u32 = 999;

fn list() -> Value {
    let entries: Vec<Value> = ["movie", "tv"]
        .into_iter()
        .flat_map(|kind| {
            (1..=SAMPLE_IDS).map(move |id| {
                json!({
                    "title": format!("{kind}:{id}"),
                    "url": OPEN_FILM,
                    "kind": "hls",
                    "licence": OPEN_FILM_LICENCE,
                    "every_episode": kind == "tv",
                })
            })
        })
        .collect();
    Value::Array(entries)
}

/// Writes the list unless this install has films of its own.
///
/// # Errors
/// The file could not be read or written.
pub fn seed(path: &Path) -> Result<()> {
    if path.exists() && std::fs::read_to_string(path)?.trim() != "[]" {
        return Ok(());
    }
    std::fs::write(path, list().to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_install_gets_a_film_for_every_sample_title() {
        let folder = std::env::temp_dir().join(format!("films-seed-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("films.json");
        std::fs::write(&path, "[]").unwrap();
        seed(&path).unwrap();
        let written: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let entries = written.as_array().unwrap();
        assert_eq!(entries.len(), 2 * SAMPLE_IDS as usize);
        assert!(
            entries
                .iter()
                .all(|entry| entry["licence"] == OPEN_FILM_LICENCE)
        );
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn an_owners_own_list_is_never_replaced() {
        let folder = std::env::temp_dir().join(format!("films-keep-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("films.json");
        std::fs::write(&path, r#"[{"title":"movie:7"}]"#).unwrap();
        seed(&path).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains("movie:7"));
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
