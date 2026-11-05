//! What search offers before anything is typed: groups taken in turn, each with its own measure
//! and a little noise per session and slot, at most three titles of a genre.

use server_catalog::DataSet;
use server_catalog::catalog::CatalogItem;
use server_catalog::det::{HashMap, HashSet};
use server_catalog::quality::chart_ok;

use crate::page::{Tier, eligible, entry_noise};

/// What a suggestion list is for.
#[derive(Clone, Copy, Debug)]
pub struct SuggestAsk<'a> {
    pub tier: Tier,
    pub session: &'a str,
    pub slot: u64,
    pub today_days: i64,
    pub limit: usize,
    /// Genre weights of the profile, empty with consent off.
    pub taste: &'a [(String, f32)],
    pub seen: &'a [u32],
    pub season_events: &'a [&'a str],
}

fn best(mut scored: Vec<(f32, &CatalogItem)>) -> Vec<&CatalogItem> {
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
    scored.into_iter().take(60).map(|(_, item)| item).collect()
}

/// Suggested titles, as item keys.
#[must_use]
pub fn suggest_keys(data: &DataSet, ask: &SuggestAsk<'_>) -> Vec<u32> {
    let catalog = &data.catalog;
    let slot = usize::try_from(ask.slot).unwrap_or(0);
    let year: i32 = server_catalog::fresh::ymd(ask.today_days)[..4]
        .parse()
        .unwrap_or(0);
    let noise = |item: &CatalogItem, salt: usize| {
        entry_noise(ask.session, item.id as usize + slot * 7 + salt)
    };
    let affinity = |item: &CatalogItem| -> f32 {
        item.genres
            .iter()
            .map(|g| {
                ask.taste
                    .iter()
                    .find(|(name, _)| name == g)
                    .map_or(0.0, |(_, w)| *w)
            })
            .sum()
    };
    let ok = |item: &CatalogItem| {
        chart_ok(item)
            && item.release_year.is_none_or(|y| y <= year)
            && !ask.seen.contains(&item.id)
            && eligible(ask.tier, item, &data.profiles)
    };
    let mut pool: Vec<&CatalogItem> = catalog
        .values()
        .filter(|i| i.popularity >= 6.0 && ok(i))
        .collect();
    pool.sort_unstable_by_key(|item| item.id);
    let mut sources: Vec<(&str, Vec<&CatalogItem>)> = Vec::new();
    let score_all = |score: &dyn Fn(&CatalogItem) -> Option<f32>| -> Vec<&CatalogItem> {
        best(
            pool.iter()
                .filter_map(|item| Some((score(item)?, *item)))
                .collect(),
        )
    };
    if !ask.taste.is_empty() {
        sources.push((
            "for_you",
            score_all(&|i| {
                let fit = affinity(i);
                (fit > 0.0).then(|| {
                    (1.0 + i.popularity).ln()
                        + 0.25 * i.vote_average
                        + 5.0 * fit
                        + 1.4 * noise(i, 1)
                })
            }),
        ));
    }
    sources.push((
        "top10",
        score_all(&|i| Some(i.popularity + 25.0 * noise(i, 2))),
    ));
    sources.push((
        "new",
        score_all(&|i| {
            i.release_year
                .is_some_and(|y| y >= year)
                .then(|| (1.0 + i.popularity).ln() + 1.4 * noise(i, 3))
        }),
    ));
    sources.push((
        "acclaimed",
        score_all(&|i| {
            (i.vote_average >= 7.6 && i.vote_count >= 500)
                .then(|| i.vote_average + 0.3 * (i.vote_count as f32).ln() + 1.4 * noise(i, 4))
        }),
    ));
    let season: Vec<&CatalogItem> = ask
        .season_events
        .iter()
        .filter_map(|event| data.vibe.seasons.get(*event))
        .flat_map(|subs| subs.iter().flat_map(|sub| sub.items.iter()))
        .filter_map(|key| catalog.get(key))
        .filter(|item| ok(item))
        .take(40)
        .collect();
    if !season.is_empty() {
        sources.push(("season", season));
    }
    let mut slugs: Vec<&String> = catalog
        .topic_keys
        .iter()
        .filter(|(_, keys)| keys.len() >= 30)
        .map(|(slug, _)| slug)
        .collect();
    slugs.sort();
    if !slugs.is_empty() {
        let pick =
            (entry_noise(ask.session, 41 + slot) * slugs.len() as f32) as usize % slugs.len();
        let items: Vec<&CatalogItem> = catalog.topic_keys[slugs[pick]]
            .iter()
            .filter_map(|key| catalog.get(key))
            .filter(|item| item.popularity >= 3.0 && ok(item))
            .take(40)
            .collect();
        if !items.is_empty() {
            sources.push(("topic", items));
        }
    }
    sources.sort_by(|a, b| {
        let weight = |name: &str| entry_noise(ask.session, name.len() * 13 + slot);
        weight(a.0).total_cmp(&weight(b.0))
    });
    take_in_turn(&sources, ask.limit)
}

/// One title from each group in turn, never one twice, at most three of a first genre.
fn take_in_turn(sources: &[(&str, Vec<&CatalogItem>)], limit: usize) -> Vec<u32> {
    let mut cursors = vec![0usize; sources.len()];
    let mut used: HashSet<u32> = HashSet::default();
    let mut per_genre: HashMap<&str, usize> = HashMap::default();
    let mut picked = Vec::new();
    'fill: loop {
        let mut progressed = false;
        for (n, (_, list)) in sources.iter().enumerate() {
            while cursors[n] < list.len() {
                let item = list[cursors[n]];
                cursors[n] += 1;
                let genre = per_genre
                    .entry(item.genres.first().map_or("", String::as_str))
                    .or_insert(0);
                if used.contains(&item.id) || *genre >= 3 {
                    continue;
                }
                *genre += 1;
                used.insert(item.id);
                picked.push(item.id);
                progressed = true;
                break;
            }
            if picked.len() >= limit {
                break 'fill;
            }
        }
        if !progressed {
            break;
        }
    }
    picked
}
