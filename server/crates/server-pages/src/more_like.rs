//! More Like This: nearest titles by the centred cosine, with a small push for what people watch
//! and rate well. The title page has no other source, and an empty answer hides the section.

use server_catalog::DataSet;
use server_catalog::catalog::CatalogItem;
use server_catalog::det::{HashMap, HashSet};
use server_catalog::people::normalise;
use server_catalog::vectors::l2_normalize;

use crate::page::{Tier, eligible};

const POPULARITY_WEIGHT: f32 = 0.3;
const RATING_WEIGHT: f32 = 0.06;
/// Titles a title page lists by default.
pub const DEFAULT_LIMIT: usize = 20;

fn centred(data: &DataSet, row: usize) -> Vec<f32> {
    let mut vector: Vec<f32> = data
        .vectors
        .get_f32(row)
        .iter()
        .zip(&data.mean)
        .map(|(x, m)| x - m)
        .collect();
    l2_normalize(&mut vector);
    vector
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// One title once, the best of its versions, and at most three from a franchise.
fn one_of_each(scored: Vec<(f32, &CatalogItem)>, limit: usize) -> Vec<&CatalogItem> {
    let mut seen: HashSet<(String, String)> = HashSet::default();
    let mut franchise: HashMap<String, usize> = HashMap::default();
    scored
        .into_iter()
        .map(|(_, item)| item)
        .filter(|item| {
            let name = normalise(&item.title);
            if !seen.insert((item.media_type.clone(), name.clone())) {
                return false;
            }
            let first = name.split(' ').next().unwrap_or("").to_owned();
            if first.chars().count() >= 4 && first != "the" {
                let count = franchise.entry(first).or_insert(0);
                *count += 1;
                return *count <= 3;
            }
            true
        })
        .take(limit)
        .collect()
}

/// The titles like this one, as item keys.
#[must_use]
pub fn more_like_keys(data: &DataSet, tier: Tier, anchor: u32, limit: usize) -> Vec<u32> {
    let Some(item) = data.catalog.get(&anchor) else {
        return Vec::new();
    };
    let Some(row) = data.vectors.index_of(anchor) else {
        return Vec::new();
    };
    let limit = limit.clamp(1, 100);
    let fetch = (limit * 8).max(80) * if tier == Tier::Adult { 1 } else { 2 } + 1;
    let vector = data.vectors.get_f32(row);
    let centre = centred(data, row);
    let anchor_name = normalise(&item.title);
    let mut scored: Vec<(f32, &CatalogItem)> = data
        .nearest(&vector, fetch)
        .into_iter()
        .filter(|(key, _)| *key != anchor)
        .filter_map(|(key, _)| {
            let other = data.catalog.get(&key)?;
            if !eligible(tier, other, &data.profiles) {
                return None;
            }
            let short = other.media_type == "movie"
                && other.runtime > 0
                && other.runtime < 40
                && item.runtime >= 60;
            let same_name =
                other.media_type == item.media_type && normalise(&other.title) == anchor_name;
            if short || same_name {
                return None;
            }
            let near = dot(&centre, &centred(data, data.vectors.index_of(key)?));
            let known = (other.popularity.ln_1p() / 100f32.ln_1p()).clamp(0.0, 1.0);
            let rated = ((other.vote_average - 5.0) / 4.0).clamp(0.0, 1.0);
            Some((
                near + POPULARITY_WEIGHT * known + RATING_WEIGHT * rated,
                other,
            ))
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.id.cmp(&b.1.id)));
    one_of_each(scored, limit)
        .into_iter()
        .map(|other| other.id)
        .collect()
}
