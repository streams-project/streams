//! The hero: a session seeded weighted draw from the top quality pool of a page.

use server_catalog::catalog::{CatalogItem, FLAG_SUGGESTIVE};

use crate::genre_spec::{GenreSpec, SpecFacts};
use crate::noise::session_uniform;
use crate::page::{Tier, eligible};
use server_catalog::DataSet;

/// Candidates kept per surface and genre before the draw.
const POOL_SIZE: usize = 50;
const MIN_VOTE_AVERAGE: f32 = 6.5;
const MIN_POPULARITY: f32 = 15.0;
/// Formats that make poor hero banners.
const EXCLUDED_GENRES: [&str; 5] = ["News", "Talk", "Soap", "Reality", "Documentary"];
/// Most titles in a hero.
pub const HERO_MAX: usize = 20;
/// Titles in a hero by default.
pub const HERO_DEFAULT: usize = 8;

fn quality_score(item: &CatalogItem) -> f32 {
    item.vote_average * item.popularity.ln_1p()
}

/// A draw with weights (key = u^(1/w), highest keys win). With no session the order is by quality.
fn weighted_draw<'a>(
    pool: Vec<&'a CatalogItem>,
    count: usize,
    session: &str,
) -> Vec<&'a CatalogItem> {
    if session.is_empty() {
        return pool.into_iter().take(count).collect();
    }
    let mut keyed: Vec<(f64, &CatalogItem)> = pool
        .into_iter()
        .map(|item| {
            let weight = f64::from(quality_score(item)).max(0.01);
            (session_uniform(session, item.id).powf(1.0 / weight), item)
        })
        .collect();
    keyed.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
    keyed
        .into_iter()
        .take(count)
        .map(|(_, item)| item)
        .collect()
}

/// Mostly the best of the pool, with about a quarter of discoveries at places 3, 6 and so on.
fn draw_hero<'a>(
    mut pool: Vec<&'a CatalogItem>,
    limit: usize,
    session: &str,
) -> Vec<&'a CatalogItem> {
    pool.sort_by(|a, b| {
        quality_score(b)
            .total_cmp(&quality_score(a))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut rest = if pool.len() > POOL_SIZE {
        pool.split_off(POOL_SIZE)
    } else {
        Vec::new()
    };
    rest.retain(|item| item.vote_average >= 7.2 && item.vote_count >= 150);
    let found_score = |item: &CatalogItem| item.vote_average * item.popularity.ln_1p().sqrt();
    rest.sort_by(|a, b| {
        found_score(b)
            .total_cmp(&found_score(a))
            .then_with(|| a.id.cmp(&b.id))
    });
    rest.truncate(40);
    let discoveries = if session.is_empty() {
        0
    } else {
        (limit / 4).min(rest.len())
    };
    let mut main = weighted_draw(pool, limit - discoveries, session);
    let found = weighted_draw(rest, discoveries, &format!("{session}|discover"));
    for (n, item) in found.into_iter().enumerate() {
        main.insert((2 + 3 * n).min(main.len()), item);
    }
    main
}

/// What a hero is drawn for.
#[derive(Clone, Copy, Debug)]
pub struct HeroAsk<'a> {
    pub tier: Tier,
    /// `movie`, `tv` or `both`.
    pub surface: &'a str,
    pub genre: Option<&'a GenreSpec>,
    pub session: &'a str,
    pub limit: usize,
    pub seen: &'a [u32],
}

/// The hero titles of a page, as item keys.
#[must_use]
pub fn hero_keys(data: &DataSet, ask: &HeroAsk<'_>) -> Vec<u32> {
    let facts = SpecFacts {
        events: &data.events,
        judged: data.catalog.judged,
    };
    let nonscripted_ok = ask.genre.is_some_and(GenreSpec::includes_nonscripted);
    let media = (ask.surface != "both").then_some(ask.surface);
    let limit = ask.limit.clamp(1, HERO_MAX);
    let pool: Vec<&CatalogItem> = data
        .catalog
        .values()
        .filter(|item| {
            item.vote_average >= MIN_VOTE_AVERAGE
                && item.popularity >= MIN_POPULARITY
                && item.has_backdrop()
                && item.has_poster()
                && item.has_overview()
                && !ask.seen.contains(&item.id)
                && (nonscripted_ok
                    || !item
                        .genres
                        .iter()
                        .any(|g| EXCLUDED_GENRES.iter().any(|x| g.eq_ignore_ascii_case(x))))
                && media.is_none_or(|kind| item.media_type == kind)
                && item.flags & FLAG_SUGGESTIVE == 0
                && ask.genre.is_none_or(|spec| spec.matches(item, &facts))
                && eligible(ask.tier, item, &data.profiles)
        })
        .collect();
    let chosen = if media.is_some() {
        draw_hero(pool, limit, ask.session)
    } else {
        draw_both(pool, limit, ask.session)
    };
    chosen.into_iter().map(|item| item.id).collect()
}

/// Films and series are drawn apart, because series have larger popularity numbers and one pool
/// would fill with series, and they alternate.
fn draw_both<'a>(pool: Vec<&'a CatalogItem>, limit: usize, session: &str) -> Vec<&'a CatalogItem> {
    let salt = format!("{session}|home");
    let (films, series): (Vec<&CatalogItem>, Vec<&CatalogItem>) = pool
        .into_iter()
        .partition(|item| item.media_type == "movie");
    let (films, series) = (
        draw_hero(films, limit, &salt),
        draw_hero(series, limit, &salt),
    );
    let series_first = !session.is_empty() && session_uniform(session, 7) < 0.5;
    let (mut first, mut second) = if series_first {
        (series.into_iter(), films.into_iter())
    } else {
        (films.into_iter(), series.into_iter())
    };
    let mut out = Vec::with_capacity(limit);
    while out.len() < limit {
        let (a, b) = (first.next(), second.next());
        if a.is_none() && b.is_none() {
            break;
        }
        out.extend(a);
        if out.len() < limit {
            out.extend(b);
        }
    }
    out
}
