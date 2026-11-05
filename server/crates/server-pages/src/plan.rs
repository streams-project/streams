//! How many rows a page may have: the policy constants, and the row caps of genre and language
//! pages, from what their pool of titles can fill.

use server_catalog::DataSet;

use crate::genre_spec::{GenreSpec, SpecFacts};
use crate::page::{
    Tier, audience_fits, eligible, genre_ok_with, language_matches, meets_vote_floor,
    member_quality_for,
};

/// The most rows any page shows.
pub const ROWS_CAP: usize = 60;
pub const INITIAL_ROWS: usize = 12;
pub const INITIAL_ROWS_GENRE: usize = 10;
pub const MIN_ROWS_FLOOR: usize = 8;
const ROW_TITLES: usize = 15;
const SESSION_TURNOVER: usize = 2;
/// Below this a genre has too few rows to be its own page.
pub const MIN_GENRE_PAGE_ROWS: usize = 8;

/// How many rows a page can have and how many are built for this request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagePlan {
    /// Rows the page's own titles can fill; unlimited with no genre.
    pub own_cap: usize,
    /// Rows the page may show in total.
    pub cap: usize,
    /// Rows to build now.
    pub target_rows: usize,
}

/// The cap of a page, which only changes when the data set does.
#[must_use]
pub fn page_cap(
    data: &DataSet,
    surface: &str,
    spec: Option<&GenreSpec>,
    language: Option<&str>,
    tier: Tier,
    composed: bool,
) -> usize {
    let facts = SpecFacts {
        events: &data.events,
        judged: data.catalog.judged,
    };
    let kinds = |kind: &str| surface == "both" || kind == surface;
    match (spec, language) {
        (_, Some(language)) => {
            let pool = data
                .catalog
                .values()
                .filter(|i| {
                    kinds(&i.media_type)
                        && language_matches(language, &i.original_language)
                        && audience_fits(true, tier, &data.profiles, i)
                        && eligible(tier, i, &data.profiles)
                        && server_catalog::quality::passes_member_quality(i)
                        && meets_vote_floor(i, true)
                })
                .count();
            (pool / (ROW_TITLES * SESSION_TURNOVER)).min(ROWS_CAP)
        }
        (Some(rule), None) => {
            let keeps = rule.keeps_children();
            let pool = data
                .catalog
                .values()
                .filter(|i| {
                    kinds(&i.media_type)
                        && genre_ok_with(Some(rule), &data.profiles, i, &facts)
                        && audience_fits(keeps, tier, &data.profiles, i)
                        && eligible(tier, i, &data.profiles)
                        && member_quality_for(Some(rule), i, &facts)
                        && meets_vote_floor(i, true)
                })
                .count();
            // a composed row costs the pool its first screen, and a judged niche or a child's
            // page may repeat titles across two visits
            let per_row = if composed {
                crate::page::FIRST_SCREEN * SESSION_TURNOVER
            } else if rule.is_judged(&facts) || tier.is_kids() {
                ROW_TITLES
            } else {
                ROW_TITLES * SESSION_TURNOVER
            };
            (pool / per_row).min(ROWS_CAP)
        }
        (None, None) => usize::MAX,
    }
}

impl PagePlan {
    /// The plan of a page of this cap for this many wanted rows.
    #[must_use]
    pub fn new(own_cap: usize, genre: bool, language: bool, wanted_rows: usize) -> Self {
        let cap = match (genre, language) {
            (true, false) => own_cap.max(crate::page::THIN_PAGE_FLOOR),
            (_, true) => own_cap,
            (false, false) => ROWS_CAP,
        };
        let target_rows = wanted_rows.min(cap.max(MIN_ROWS_FLOOR)).max(MIN_ROWS_FLOOR);
        Self {
            own_cap,
            cap,
            target_rows,
        }
    }

    /// Whether a menu should offer this page.
    #[must_use]
    pub const fn supported(&self) -> bool {
        self.own_cap >= MIN_GENRE_PAGE_ROWS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s75_a_plain_page_is_capped_at_60_and_never_below_the_floor() {
        let plan = PagePlan::new(usize::MAX, false, false, 12);
        assert_eq!((plan.cap, plan.target_rows), (ROWS_CAP, 12));
        assert_eq!(
            PagePlan::new(usize::MAX, false, false, 2).target_rows,
            MIN_ROWS_FLOOR
        );
        assert_eq!(
            PagePlan::new(usize::MAX, false, false, 900).target_rows,
            ROWS_CAP
        );
    }

    #[test]
    fn s76_a_genre_page_is_offered_from_eight_rows_and_topped_up_to_the_floor() {
        let thin = PagePlan::new(3, true, false, 10);
        assert!(!thin.supported());
        assert_eq!(thin.cap, crate::page::THIN_PAGE_FLOOR);
        assert!(PagePlan::new(8, true, false, 10).supported());
    }
}
