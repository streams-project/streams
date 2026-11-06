//! What a page reads about a profile, in one place, so consent is decided once.

use contract::{Rating, TitleKey, TitleKind};
use server_accounts::ProfileScope;
use server_catalog::catalog::item_key;
use server_library::{Library, LibraryError};

use crate::build::Personal;
use crate::signals::Signals;

/// Titles read from each list.
const LIST_KEYS: usize = 40;
/// Titles read from history.
const RECENT_KEYS: usize = 500;
/// Genres blended into a taste query.
const TASTE_GENRES: usize = 5;

fn packed(key: TitleKey) -> u32 {
    let word = if key.kind() == TitleKind::Tv {
        "tv"
    } else {
        "movie"
    };
    item_key(word, key.id())
}

/// Read a profile for a page. The profile's own list and Continue Watching are its own data and
/// are always drawn. Taste, history and what it liked feed recommendations only with consent on.
///
/// # Errors
/// The store did not answer.
pub async fn personal_of(
    library: &Library,
    scope: &ProfileScope,
) -> Result<Personal, LibraryError> {
    let profile = scope.profile();
    let store = library.store();
    let personal = scope.consent().personal;
    let saved: Vec<u32> = store
        .list(profile, LIST_KEYS)
        .await?
        .into_iter()
        .map(packed)
        .collect();
    let continues = library.continue_watching(profile, LIST_KEYS).await?;
    let mut version = continues.first().map_or(0, |row| row.changed_ms);
    version = version.wrapping_add(saved.len() as u64);
    let progress = continues
        .iter()
        .map(|row| (packed(row.title), (row.percent, row.episode.clone())))
        .collect();
    let mut recent: Vec<u32> = continues.iter().map(|row| packed(row.title)).collect();
    if !personal {
        return Ok(Personal {
            signals: Signals {
                saved,
                recent,
                liked: Vec::new(),
                personal,
            },
            taste: Vec::new(),
            seen: Vec::new(),
            progress,
            version,
        });
    }
    let seen: Vec<u32> = store
        .seen(profile, RECENT_KEYS)
        .await?
        .into_iter()
        .map(packed)
        .collect();
    for key in &seen {
        if !recent.contains(key) && recent.len() < LIST_KEYS {
            recent.push(*key);
        }
    }
    let rated = store.rated(profile, LIST_KEYS).await?;
    let liked: Vec<u32> = rated
        .iter()
        .filter(|(_, rating)| matches!(rating, Rating::Up | Rating::Love))
        .map(|(title, _)| packed(*title))
        .collect();
    let taste = store.taste_of(profile).await?;
    version = version.wrapping_add(u64::from(taste.version));
    Ok(Personal {
        signals: Signals {
            saved,
            recent,
            liked,
            personal,
        },
        taste: taste.top(TASTE_GENRES),
        seen,
        progress,
        version,
    })
}
