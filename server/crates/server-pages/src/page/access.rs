//! Who may see which title on which page: genre rule, age limit, audience, news and seasonal nets, vote and quality gates.
use super::*;

/// The page's genre rule. The Pride page trusts the model's profile when the title has one: LGBTQ+ must be central to the story (measured
/// 51 of 53 correct on titles graded by hand, against 18 of 50 for "present"); a title without a profile keeps the keyword rule.
pub fn genre_ok_with(
    spec: Option<&crate::genre_spec::GenreSpec>,
    profiles: &server_catalog::profiles::ProfileStore,
    item: &CatalogItem,
    facts: &crate::genre_spec::SpecFacts<'_>,
) -> bool {
    let Some(spec) = spec else { return true };
    if matches!(spec, crate::genre_spec::GenreSpec::Lgbtq)
        && let Some(p) = profiles.get(item.id)
    {
        return p.lgbtq == server_catalog::profiles::Lgbtq::Central;
    }
    spec.matches(item, facts)
}

/// Whether a title's original language belongs to a language page: source writes Chinese as `zh` or `cn` and Norwegian as `no` or `nb`.
pub fn language_matches(page: &str, original: &str) -> bool {
    page == original || matches!((page, original), ("zh", "cn") | ("no", "nb"))
}

/// Vote count floor (see `PageContext::reliable`): 10 votes on default pages, 5 on genre pages.
pub fn meets_vote_floor(item: &CatalogItem, genre_page: bool) -> bool {
    item.vote_count == 0 || item.vote_count >= if genre_page { 5 } else { 10 }
}

/// The quality gate of a page: a page whose members the judge vetted (Stand-up, Cult, Independent ...) accepts less known titles (rating 5.5, popularity 0.5), since
/// the niche is made of them; every other page uses `passes_member_quality`.
pub fn member_quality_for(
    spec: Option<&crate::genre_spec::GenreSpec>,
    item: &CatalogItem,
    facts: &crate::genre_spec::SpecFacts<'_>,
) -> bool {
    // reality and children's pages take the lower gate: reality shows and children's series rarely collect the votes scripted dramas do
    let lower = spec.is_some_and(|s| {
        s.is_judged(facts)
            || matches!(
                s,
                crate::genre_spec::GenreSpec::Reality | crate::genre_spec::GenreSpec::Kids(_)
            )
    });
    if lower {
        item.vote_average >= 5.5
            && item.popularity >= 0.5
            && item.has_overview()
            && item.has_poster()
            && (item.has_backdrop())
    } else {
        server_catalog::quality::passes_member_quality(item)
    }
}

impl<'a> PageContext<'a> {
    /// A title this page may show in a row made from a request or a group: the page's filters, its quality gate and its vote floor (also served by /api/check/page-pool).
    pub fn page_pool_member(&self, item: &CatalogItem) -> bool {
        self.matches_filters(item) && self.member_ok(item) && self.reliable(item)
    }

    /// What genre rules read.
    pub(crate) fn facts(&self) -> crate::genre_spec::SpecFacts<'_> {
        crate::genre_spec::SpecFacts {
            events: self.events,
            judged: self.catalog.judged,
        }
    }

    pub(crate) fn matches_filters(&self, item: &CatalogItem) -> bool {
        (self.surface == "both" || item.media_type == self.surface || item.media_type == "both")
            && self
                .language
                .is_none_or(|l| language_matches(l, &item.original_language))
            && self.genre_ok(item)
            && self.audience_fits(item)
            && self.maturity_allows(item)
            && (self.allowed_by_nets(item))
    }

    pub(crate) fn genre_ok(&self, item: &CatalogItem) -> bool {
        genre_ok_with(self.genre_spec, self.profiles, item, &self.facts())
    }

    /// The one eligibility function for this viewer's tier.
    pub(crate) fn maturity_allows(&self, item: &CatalogItem) -> bool {
        eligible(self.tier, item, self.profiles)
    }

    pub(crate) fn audience_fits(&self, item: &CatalogItem) -> bool {
        let keeps = self
            .genre_spec
            .is_none_or(crate::genre_spec::GenreSpec::keeps_children);
        audience_fits(keeps, self.tier, self.profiles, item)
    }

    /// Vote count floor: a rating from a handful of votes says nothing. Known only for pulled titles (all of the
    /// quality pool); genre pages use a lower floor so small collections survive.
    pub(crate) fn reliable(&self, item: &CatalogItem) -> bool {
        meets_vote_floor(item, self.genre_spec.is_some() || self.language.is_some())
    }

    /// News, talk and soap never fill default rows (a page that asks for them keeps them); seasonal titles only
    /// appear inside their window (the Halloween page lifts the Halloween rule).
    pub(crate) fn allowed_by_nets(&self, item: &CatalogItem) -> bool {
        use server_catalog::catalog::{
            FLAG_CHRISTMAS, FLAG_HALLOWEEN, FLAG_NONSCRIPTED, FLAG_SUGGESTIVE, FLAG_WRESTLING_EVENT,
        };
        let f = item.flags & server_catalog::catalog::FLAGS_RULES;
        if f == 0 {
            return true;
        }
        if f & FLAG_SUGGESTIVE != 0 {
            return false;
        }
        if f & FLAG_NONSCRIPTED != 0 && !self.genre_spec.is_some_and(|s| s.includes_nonscripted()) {
            return false;
        }
        // pay per view wrestling events are not default home content (the Sports page keeps them)
        if f & FLAG_WRESTLING_EVENT != 0
            && !matches!(self.genre_spec, Some(crate::genre_spec::GenreSpec::Sports))
        {
            return false;
        }
        let (m, d) = self.today;
        let md = (m as u32) * 100 + d as u32;
        // a seasonal title stays off the pages outside its season, except the page of its own event
        let event_page = self.genre_spec.and_then(|s| s.event_id());
        if f & FLAG_CHRISTMAS != 0 && event_page != Some("christmas") && !(md >= 1120 || md <= 106)
        {
            return false;
        }
        if f & FLAG_HALLOWEEN != 0
            && event_page != Some("halloween")
            && !(1001..=1101).contains(&md)
        {
            return false;
        }
        true
    }

    /// Filters of the page without its genre rule (a seasonal page holds titles that are near the season without carrying its keyword).
    pub(crate) fn matches_filters_any_genre(&self, item: &CatalogItem) -> bool {
        (self.surface == "both" || item.media_type == self.surface || item.media_type == "both")
            && self
                .language
                .is_none_or(|l| language_matches(l, &item.original_language))
            && self.audience_fits(item)
            && self.maturity_allows(item)
            && (self.allowed_by_nets(item))
    }

    /// A title made for children (source Kids genre, or a US TV-Y or TV-Y7 rating): on a page that is not a child's it stays out of the personal rows, the popular row and
    /// the composed rows that are not about children or animation (Miraculous next to Breaking Bad in Top Picks, iCarly next to The Boys in US Series).
    pub(crate) fn childrens_title(&self, item: &CatalogItem) -> bool {
        is_childrens_title(&self.data.kids_channels, item)
    }

    /// The quality gate of this page (see `member_quality_for`).
    pub(crate) fn member_ok(&self, item: &CatalogItem) -> bool {
        member_quality_for(self.genre_spec, item, &self.facts())
    }
}
