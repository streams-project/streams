//! Search: titles first, then people, near misses, full text and topics. Every result still passes
//! the rules, so a limited profile looks further and keeps what it may see.

use rusqlite::{Connection, OpenFlags};
use server_catalog::DataSet;
use server_catalog::catalog::CatalogItem;
use server_catalog::det::HashSet;
use server_catalog::manifest::role;
use server_catalog::people::{normalise, within_edits};

use crate::page::{Tier, eligible};

/// Fewer title matches than this and the search reads the full text.
const FEW_TITLES: usize = 5;
/// Normalised titles with their weight, built once per data set.
#[derive(Debug, Default)]
pub struct TitleIndex {
    /// Item key, weight, normalised title with a space at each end.
    rows: Vec<(u32, f32, String)>,
}

/// One continuous score, so a real title stands out among equal matches, and an entry with nothing
/// to show sinks and is still found when it is the only match.
fn search_weight(item: &CatalogItem) -> f32 {
    let has = |path: &Option<String>| path.as_deref().is_some_and(|text| !text.is_empty());
    (1.0 + item.popularity.max(0.0)).ln()
        + if has(&item.poster_path) { 0.8 } else { 0.0 }
        + if has(&item.backdrop_path) { 0.3 } else { 0.0 }
        + if item.overview.chars().count() >= 30 {
            0.4
        } else {
            0.0
        }
        + 0.25 * (1.0 + item.vote_count as f32).ln().min(6.0)
}

impl TitleIndex {
    /// Read every title of a set.
    #[must_use]
    pub fn build(data: &DataSet) -> Self {
        let mut rows: Vec<(u32, f32, String)> = data
            .catalog
            .values()
            .map(|item| {
                (
                    item.id,
                    search_weight(item),
                    format!(" {} ", normalise(&item.title)),
                )
            })
            .collect();
        rows.sort_unstable_by_key(|row| row.0);
        Self { rows }
    }

    fn scored(&self, query: &str) -> Vec<(f32, u32)> {
        let words: Vec<&str> = query.split(' ').filter(|w| !w.is_empty()).collect();
        let Some((last, head)) = words.split_last() else {
            return Vec::new();
        };
        let exact = format!(" {query} ");
        let starts = format!(" {query}");
        let head_pats: Vec<String> = head.iter().map(|w| format!(" {w} ")).collect();
        let last_pat = format!(" {last}");
        self.rows
            .iter()
            .filter(|(_, _, t)| {
                t.contains(&last_pat) && head_pats.iter().all(|p| t.contains(p.as_str()))
            })
            .map(|(key, weight, t)| {
                let tier = if *t == exact {
                    3.0
                } else if t.starts_with(&starts) {
                    1.6
                } else {
                    0.0
                };
                (tier + weight, *key)
            })
            .collect()
    }

    /// Keys of titles whose words all match; the last word may be unfinished. An alias swap
    /// competes on the same score.
    #[must_use]
    pub fn matches(&self, query: &str, aliases: &[Vec<String>], limit: usize) -> Vec<u32> {
        let mut hits = self.scored(query);
        for variant in alias_variants(query, aliases) {
            hits.extend(self.scored(&variant));
        }
        hits.sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut seen = HashSet::default();
        hits.retain(|hit| seen.insert(hit.1));
        hits.into_iter().take(limit).map(|hit| hit.1).collect()
    }
}

/// The groups of words that mean the same in a title, from the data set.
#[must_use]
pub fn load_aliases(data: &DataSet) -> Vec<Vec<String>> {
    let Some(path) = data.files.get(role::ALIASES) else {
        return Vec::new();
    };
    let groups: Vec<Vec<String>> = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    groups
        .into_iter()
        .map(|group| group.iter().map(|word| normalise(word)).collect())
        .collect()
}

/// The query with one word swapped for each other member of its group.
fn alias_variants(query: &str, aliases: &[Vec<String>]) -> Vec<String> {
    let words: Vec<&str> = query.split(' ').filter(|w| !w.is_empty()).collect();
    let mut out = Vec::new();
    for (i, word) in words.iter().enumerate() {
        for group in aliases
            .iter()
            .filter(|group| group.iter().any(|m| m == word))
        {
            for other in group.iter().filter(|m| m.as_str() != *word) {
                let mut variant = words.clone();
                variant[i] = other;
                out.push(variant.join(" "));
            }
        }
    }
    out
}

/// Titles whose overview, genres or director hold the query as a phrase. Only letters and digits
/// reach the query, so nothing in it reads as full text syntax. The table is checked at load.
fn full_text(data: &DataSet, query: &str, limit: usize) -> Vec<u32> {
    let Some(path) = data.files.get(role::CATALOG) else {
        return Vec::new();
    };
    let Ok(conn) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return Vec::new();
    };
    let Ok(mut statement) = conn.prepare(
        "SELECT c.id FROM catalog_fts f JOIN catalog c ON c.id = f.id WHERE catalog_fts MATCH ?1 \
         ORDER BY rank, c.popularity DESC LIMIT ?2",
    ) else {
        return Vec::new();
    };
    let phrase = format!("\"{query}\"");
    let cap = i64::try_from(limit).unwrap_or(i64::MAX);
    let Ok(rows) = statement.query_map(rusqlite::params![phrase, cap], |row| row.get::<_, i64>(0))
    else {
        return Vec::new();
    };
    rows.filter_map(Result::ok)
        .filter_map(|key| u32::try_from(key).ok())
        .collect()
}

/// Near misses, only when few titles were found. Compared on the normalised title, as the old code
/// meant to do when it compared raw title lengths.
fn near_misses(data: &DataSet, query: &str) -> Vec<u32> {
    let length = query.chars().count();
    let max = if length <= 7 { 1 } else { 2 };
    let mut hits: Vec<&CatalogItem> = data
        .catalog
        .values()
        .filter(|item| item.popularity >= 1.0)
        .filter(|item| {
            let name = normalise(&item.title);
            name.chars().count().abs_diff(length) <= max && within_edits(query, &name, max)
        })
        .collect();
    hits.sort_unstable_by(|a, b| b.popularity.total_cmp(&a.popularity).then(a.id.cmp(&b.id)));
    hits.into_iter().take(12).map(|item| item.id).collect()
}

/// The keys of a search, best first, filtered to what this tier may see.
#[must_use]
pub fn search_keys(
    data: &DataSet,
    index: &TitleIndex,
    aliases: &[Vec<String>],
    tier: Tier,
    raw: &str,
    wanted: usize,
) -> Vec<u32> {
    let query = normalise(raw);
    if query.is_empty() {
        return Vec::new();
    }
    let catalog = &data.catalog;
    let limit = if tier == Tier::Adult {
        wanted
    } else {
        (wanted * 4).min(200)
    };
    let title_keys = index.matches(&query, aliases, limit);
    let text_keys = if title_keys.len() < FEW_TITLES {
        full_text(data, &query, limit)
    } else {
        Vec::new()
    };
    let (exact_person, person_keys) = data.people.titles_for(raw, 24);
    let mut near = Vec::new();
    if title_keys.len() < 4 && person_keys.is_empty() && query.chars().count() >= 4 {
        near = near_misses(data, &query);
    }
    let popularity = |key: &u32| catalog.get(key).map_or(0.0, |item| item.popularity);
    let (title_keys, near) = if near.is_empty() {
        (title_keys, near)
    } else {
        let mut all: Vec<u32> = title_keys.iter().chain(near.iter()).copied().collect();
        all.sort_by(|a, b| popularity(b).total_cmp(&popularity(a)).then(a.cmp(b)));
        all.dedup();
        (all, Vec::new())
    };
    let word_person = if !exact_person && person_keys.is_empty() {
        data.people.person_by_word(raw, 24)
    } else {
        None
    };
    let best_title = title_keys.iter().map(popularity).fold(0.0f32, f32::max);
    let word_first = word_person
        .as_ref()
        .is_some_and(|(_, weight, _)| *weight > 60.0 && best_title < 40.0);
    let word_keys = word_person.map(|(_, _, keys)| keys).unwrap_or_default();
    let topic = data.topics.match_query(&query).into_iter().next();
    let topic_keys: Vec<u32> = topic
        .and_then(|(slug, _)| catalog.topic_keys.get(slug))
        .map(|keys| {
            keys.iter()
                .filter(|key| popularity(key) >= 1.0)
                .take(30)
                .copied()
                .collect()
        })
        .unwrap_or_default();
    let named_title = title_keys
        .iter()
        .filter_map(|key| catalog.get(key))
        .any(|item| normalise(&item.title) == query);
    let person_first = exact_person && (query.contains(' ') || !named_title);
    let ordered: Vec<u32> = if person_first {
        person_keys
            .iter()
            .take(12)
            .chain(&title_keys)
            .chain(person_keys.iter().skip(12))
            .chain(&text_keys)
            .chain(&topic_keys)
            .copied()
            .collect()
    } else if word_first {
        word_keys
            .iter()
            .take(12)
            .chain(&title_keys)
            .chain(word_keys.iter().skip(12))
            .chain(&person_keys)
            .chain(&text_keys)
            .chain(&topic_keys)
            .copied()
            .collect()
    } else {
        title_keys
            .iter()
            .chain(&person_keys)
            .chain(&near)
            .chain(word_keys.iter().take(8))
            .chain(&text_keys)
            .chain(&topic_keys)
            .copied()
            .collect()
    };
    let mut seen = HashSet::default();
    ordered
        .into_iter()
        .filter(|key| seen.insert(*key))
        .filter(|key| {
            catalog
                .get(key)
                .is_some_and(|item| eligible(tier, item, &data.profiles))
        })
        .take(wanted)
        .collect()
}
