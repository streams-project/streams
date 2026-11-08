//! The titles over the profile chooser. At most three for the whole account, each with the facts
//! of the line under its logo: titles coming within 30 days first, then series whose new season
//! starts within 30 days, then today's Top 10, then the most watched with no line. Each needs a
//! logo, a poster and a backdrop, and the banner formats of the hero. The draw turns each day, so
//! the chooser shows other titles tomorrow.

use server_catalog::DataSet;
use server_catalog::catalog::{CatalogItem, FLAG_SUGGESTIVE, item_key};
use server_catalog::det::{HashMap, HashSet};
use server_catalog::fresh::ymd;
use server_catalog::seasons::SeasonStart;

use crate::hero::is_banner_format;
use crate::new_popular::top_today;
use crate::noise::session_uniform;
use crate::page::{Tier, eligible};

/// Titles on the chooser.
pub const GATE_TITLES: usize = 3;
/// A dated line looks this many days ahead.
pub const COMING_DAYS: i64 = 30;
/// The most popular titles of each kind that the day's draw picks from.
const POOL: usize = 12;

/// The facts of the line under a title's logo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// Comes out on this day, YYYY-MM-DD.
    Coming(String),
    /// A new season starts on this day, YYYY-MM-DD.
    SeasonsComing(String),
    /// Place in today's Top 10 of its kind, `movie` or `tv`.
    Ranked { rank: u32, kind: &'static str },
    /// No line.
    None,
}

/// One chosen title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePick {
    pub key: u32,
    pub line: Line,
}

/// True when the title can fill the chooser: a logo that is a picture, a poster and a backdrop, a
/// banner format, nothing suggestive, and within the tier.
#[must_use]
pub fn fits(data: &DataSet, tier: Tier, item: &CatalogItem) -> bool {
    let logo = item
        .logo_path
        .as_deref()
        .is_some_and(|path| !path.is_empty() && !path.ends_with(".svg"));
    logo && item.has_poster()
        && item.has_backdrop()
        && is_banner_format(item)
        && item.flags & FLAG_SUGGESTIVE == 0
        && eligible(tier, item, &data.profiles)
}

/// The most popular `POOL` of a list, then a draw weighted by popularity and seeded by the day,
/// titles with art first.
fn draw<'a, T>(mut pool: Vec<(&'a CatalogItem, T)>, seed: &str) -> Vec<(&'a CatalogItem, T)> {
    pool.sort_by(|a, b| {
        b.0.popularity
            .total_cmp(&a.0.popularity)
            .then_with(|| a.0.id.cmp(&b.0.id))
    });
    pool.truncate(POOL);
    let mut keyed: Vec<(f64, (&'a CatalogItem, T))> = pool
        .into_iter()
        .map(|entry| {
            let weight = f64::from(entry.0.popularity.ln_1p()).max(0.01);
            (session_uniform(seed, entry.0.id).powf(1.0 / weight), entry)
        })
        .collect();
    keyed.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.0.id.cmp(&b.1.0.id)));
    let mut drawn: Vec<(&'a CatalogItem, T)> = keyed.into_iter().map(|(_, entry)| entry).collect();
    // A title with a poster free of words fills a tall screen; the draw's order holds within each.
    drawn.sort_by_key(|(item, _)| item.art_path.is_none());
    drawn
}

/// The chooser's titles for a tier on a day. `starts` are the seasons the catalogue dates.
#[must_use]
pub fn gate_picks(
    data: &DataSet,
    tier: Tier,
    today_days: i64,
    starts: &[SeasonStart],
) -> Vec<GatePick> {
    let today = ymd(today_days);
    let until = ymd(today_days + COMING_DAYS);
    let soon = |date: &str| date.len() == 10 && date > today.as_str() && date <= until.as_str();
    let seed = format!("gate|{today}");
    let fit = |key: u32| data.catalog.get(&key).filter(|item| fits(data, tier, item));
    let mut out: Vec<GatePick> = Vec::with_capacity(GATE_TITLES);
    let mut used: HashSet<u32> = HashSet::default();
    let mut take = |pool: Vec<(&CatalogItem, Line)>, out: &mut Vec<GatePick>| {
        for (item, line) in pool {
            if out.len() >= GATE_TITLES {
                return;
            }
            if used.insert(item.id) {
                out.push(GatePick { key: item.id, line });
            }
        }
    };

    let coming: Vec<(&CatalogItem, Line)> = data
        .fresh
        .items
        .iter()
        .filter(|fresh| soon(&fresh.release_date))
        .filter_map(|fresh| {
            let item = fit(item_key(&fresh.media_type, fresh.id))?;
            Some((item, Line::Coming(fresh.release_date.clone())))
        })
        .collect();
    take(draw(coming, &format!("{seed}|coming")), &mut out);

    let mut season_days: HashMap<u32, String> = HashMap::default();
    let dated = data.fresh.items.iter().filter_map(|fresh| {
        let next = fresh.next_season.as_ref()?;
        (fresh.media_type == "tv").then(|| (fresh.id, next.air_date.clone()))
    });
    let catalogued = starts
        .iter()
        .map(|start| (start.series, start.air_date.clone()));
    for (series, day) in dated.chain(catalogued).filter(|(_, day)| soon(day)) {
        let key = item_key("tv", series);
        let earliest = season_days.entry(key).or_insert_with(|| day.clone());
        if day < *earliest {
            *earliest = day;
        }
    }
    let seasons: Vec<(&CatalogItem, Line)> = season_days
        .into_iter()
        .filter_map(|(key, day)| Some((fit(key)?, Line::SeasonsComing(day))))
        .collect();
    take(draw(seasons, &format!("{seed}|seasons")), &mut out);

    let charts: Vec<(&'static str, Vec<&CatalogItem>)> = ["tv", "movie"]
        .into_iter()
        .map(|kind| (kind, top_today(data, tier, kind, today_days)))
        .collect();
    // A child's chart is a list of titles for children, so its place is not a place in "Today".
    if !tier.is_kids() {
        let ranked: Vec<(&CatalogItem, Line)> = charts
            .iter()
            .flat_map(|(kind, items)| {
                let kind: &'static str = kind;
                items.iter().zip(1u32..).filter_map(move |(item, rank)| {
                    fits(data, tier, item).then_some((*item, Line::Ranked { rank, kind }))
                })
            })
            .collect();
        take(draw(ranked, &format!("{seed}|ranked")), &mut out);
    }

    let mut most_watched: Vec<(&CatalogItem, Line)> = Vec::new();
    let longest = charts
        .iter()
        .map(|(_, items)| items.len())
        .max()
        .unwrap_or(0);
    for at in 0..longest {
        for (_, items) in &charts {
            if let Some(item) = items.get(at).filter(|item| fits(data, tier, item)) {
                most_watched.push((*item, Line::None));
            }
        }
    }
    take(most_watched, &mut out);
    out
}
