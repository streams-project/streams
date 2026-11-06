//! Keys of titles: source numbers movies and series separately, so the engine keys every title by `item_key`.
use super::*;

/// source numbers movies and TV series separately, so the same id exists in both. The engine keys every
/// title by `item_key`: movies keep their source id, TV series get the top bit set.
pub const TV_BIT: u32 = 1 << 31;

#[inline]
pub fn item_key(media_type: &str, title_id: u32) -> u32 {
    if media_type == "tv" {
        title_id | TV_BIT
    } else {
        title_id
    }
}

/// Resolve a client supplied (id, media_type) to an internal key that exists in the catalog.
/// Without a media type the movie is tried first, then the series.
pub fn resolve_key(catalog: &CatalogMap, title_id: u32, media_type: Option<&str>) -> Option<u32> {
    match media_type {
        Some(mt) => Some(item_key(mt, title_id)).filter(|k| catalog.contains_key(k)),
        None => [title_id, title_id | TV_BIT]
            .into_iter()
            .find(|k| catalog.contains_key(k)),
    }
}
