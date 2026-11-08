//! The New and Popular screen. A title the catalogue lacks has no certificate, so it cannot pass
//! the rules and stays off this screen until the data builder holds it.

use server_catalog::DataSet;
use server_catalog::catalog::CatalogItem;
use server_catalog::det::HashSet;
use server_catalog::fresh::parse_days;
use server_catalog::quality::chart_ok;

use crate::page::{Row, RowItem, Tier, eligible, is_childrens_title};

/// Titles in a row of this screen.
const ROW_TITLES: usize = 40;
const MIN_ROW_ITEMS: usize = 6;
/// A row of upcoming titles is shown from three.
const MIN_UPCOMING_ITEMS: usize = 3;
const SCREEN_ROWS: usize = 6;
/// New means released within this many days.
const NEW_DAYS: i64 = 60;

fn items_of<'a>(list: &[&Candidate<'a>]) -> Vec<&'a CatalogItem> {
    list.iter().map(|candidate| candidate.item).collect()
}

struct Candidate<'a> {
    item: &'a CatalogItem,
    release: String,
}

impl Candidate<'_> {
    fn days_since(&self, today: i64) -> Option<i64> {
        parse_days(&self.release).map(|day| today - day)
    }
}

fn chart<'a>(
    data: &'a DataSet,
    tier: Tier,
    kind: &str,
    year: i32,
    today: &str,
) -> Vec<&'a CatalogItem> {
    let mut items: Vec<&CatalogItem> = data
        .catalog
        .values()
        .filter(|i| i.media_type == kind && chart_ok(i) && eligible(tier, i, &data.profiles))
        .filter(|i| i.release_year.is_none_or(|y| y <= year))
        .filter(|i| !data.fresh.is_upcoming(&i.media_type, i.title_id, today))
        .filter(|i| tier.is_kids() || !is_childrens_title(&data.kids_channels, i))
        .collect();
    items.sort_unstable_by(|a, b| {
        b.popularity
            .total_cmp(&a.popularity)
            .then_with(|| a.id.cmp(&b.id))
    });
    items.truncate(10);
    items
}

fn row(title: String, archetype: &str, kind: &str, items: &[&CatalogItem]) -> Row {
    Row {
        title,
        archetype: archetype.to_owned(),
        kind: kind.to_owned(),
        items: items
            .iter()
            .map(|item| RowItem::from_catalog(item))
            .collect(),
        meta: None,
    }
}

/// Today's Top 10 of one kind (`movie` or `tv`) under a tier, first place first: the chart the
/// New and Popular screen shows.
#[must_use]
pub fn top_today<'a>(
    data: &'a DataSet,
    tier: Tier,
    kind: &str,
    today_days: i64,
) -> Vec<&'a CatalogItem> {
    let today = server_catalog::fresh::ymd(today_days);
    let year: i32 = today[..4].parse().unwrap_or(0);
    chart(data, tier, kind, year, &today)
}

/// Today's most watched titles for a visitor, a series and a film in turn, at most `count`. The
/// landing of a television shows them as cards after its showreel.
#[must_use]
pub fn public_trending(data: &DataSet, tier: Tier, today_days: i64, count: usize) -> Vec<&CatalogItem> {
    let today = server_catalog::fresh::ymd(today_days);
    let year: i32 = today[..4].parse().unwrap_or(0);
    let series = chart(data, tier, "tv", year, &today);
    let films = chart(data, tier, "movie", year, &today);
    let mut out = Vec::with_capacity(count);
    for at in 0..series.len().max(films.len()) {
        out.extend(series.get(at).copied());
        out.extend(films.get(at).copied());
    }
    out.truncate(count);
    out
}

/// The rows of the screen. Kids get their own order, and an adult's rows hold no children's titles.
#[must_use]
pub fn new_popular_rows(data: &DataSet, tier: Tier, today_days: i64) -> Vec<Row> {
    let today = server_catalog::fresh::ymd(today_days);
    let year: i32 = today[..4].parse().unwrap_or(0);
    let kids = tier.is_kids();
    let pool: Vec<Candidate<'_>> = data
        .fresh
        .items
        .iter()
        .filter_map(|fresh| {
            let key = server_catalog::catalog::item_key(&fresh.media_type, fresh.id);
            let item = data.catalog.get(&key)?;
            (eligible(tier, item, &data.profiles)
                && item.has_poster()
                && (kids || !is_childrens_title(&data.kids_channels, item)))
            .then(|| Candidate {
                item,
                release: fresh.release_date.clone(),
            })
        })
        .collect();
    let by_popularity = |list: &mut Vec<&Candidate<'_>>, wanted: usize| {
        list.sort_by(|a, b| {
            b.item
                .popularity
                .total_cmp(&a.item.popularity)
                .then_with(|| a.item.id.cmp(&b.item.id))
        });
        list.truncate(wanted);
    };
    let mut new_on: Vec<&Candidate<'_>> = pool
        .iter()
        .filter(|c| {
            c.item.has_backdrop()
                && c.days_since(today_days)
                    .is_some_and(|d| (0..=NEW_DAYS).contains(&d))
        })
        .collect();
    by_popularity(&mut new_on, ROW_TITLES);
    new_on.sort_by(|a, b| b.release.cmp(&a.release));
    let soon = |low: i64, high: i64| {
        let mut list: Vec<&Candidate<'_>> = pool
            .iter()
            .filter(|c| {
                c.days_since(today_days)
                    .is_some_and(|d| (-high..=-low).contains(&d) && d < 0)
            })
            .collect();
        by_popularity(&mut list, ROW_TITLES);
        list.sort_by(|a, b| a.release.cmp(&b.release));
        list
    };
    let (this_week, next_week, later) = (soon(0, 7), soon(8, 14), soon(15, 365));
    let films = chart(data, tier, "movie", year, &today);
    let series = chart(data, tier, "tv", year, &today);
    let (film_name, series_name) = if kids {
        ("Top 10 Kids' Films Today", "Top 10 Kids' Series Today")
    } else {
        ("Top 10 Films Today", "Top 10 Series Today")
    };
    let new_row = row(String::from("New Here"), "new", "new", &items_of(&new_on));
    let film_row = row(film_name.to_owned(), "trending", "", &films);
    let series_row = row(series_name.to_owned(), "trending", "", &series);
    let week = row(
        String::from("Coming This Week"),
        "soon",
        "soon",
        &items_of(&this_week),
    );
    let next = row(
        String::from("Coming Next Week"),
        "soon",
        "soon",
        &items_of(&next_week),
    );
    let wait = row(
        String::from("Worth the Wait"),
        "soon",
        "soon",
        &items_of(&later),
    );
    let order = if kids {
        vec![new_row, series_row, week, wait, film_row, next]
    } else {
        vec![new_row, film_row, series_row, week, next, wait]
    };
    keep_rows(order)
}

/// A title shows once, in the first row that has it, and charts keep all of theirs. A row needs
/// six titles, or three when upcoming.
fn keep_rows(order: Vec<Row>) -> Vec<Row> {
    let mut seen: HashSet<u32> = HashSet::default();
    let mut out = Vec::new();
    for mut row in order {
        let ranked = row.archetype == "trending";
        if !ranked {
            row.items.retain(|item| seen.insert(item.id));
        }
        let min = if row.kind == "soon" {
            MIN_UPCOMING_ITEMS
        } else {
            MIN_ROW_ITEMS
        };
        if !row.items.is_empty() && (ranked || row.items.len() >= min) {
            out.push(row);
        }
        if out.len() >= SCREEN_ROWS {
            break;
        }
    }
    out
}
