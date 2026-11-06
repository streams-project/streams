//! The one eligibility function, and the audience rules beside it.

use server_catalog::catalog::{CatalogItem, FLAG_SUGGESTIVE};
use server_catalog::channels::KidsChannels;
use server_catalog::profiles::{ADULT, FAMILY, KIDS, ProfileStore, TEEN};

use crate::tier::Tier;

/// Genres that keep a title out of both kids tiers.
const KIDS_BLOCKED_GENRES: [&str; 10] = [
    "Horror",
    "Crime",
    "Thriller",
    "Mystery",
    "War",
    "War & Politics",
    "Talk",
    "News",
    "Soap",
    "Western",
];

/// Keywords that mark violent or sexual content. They apply to titles that carry keywords.
const KIDS_BLOCKED_KEYWORDS: [&str; 14] = [
    "gore",
    "violence",
    "blood",
    "murder",
    "serial killer",
    "torture",
    "rape",
    "sex",
    "nudity",
    "drugs",
    "suicide",
    "zombie",
    "slasher",
    "erotic",
];

fn has_genre(item: &CatalogItem, name: &str) -> bool {
    item.genres.iter().any(|g| g.eq_ignore_ascii_case(name))
}

/// May this tier see this title?
#[must_use]
pub fn eligible(tier: Tier, item: &CatalogItem, models: &ProfileStore) -> bool {
    if tier == Tier::Adult {
        return true;
    }
    if item.adult || item.flags & FLAG_SUGGESTIVE != 0 {
        return false;
    }
    if item.cert == 0 {
        let Some(model) = models.get(item.id) else {
            return false;
        };
        return match tier {
            Tier::LittleKids | Tier::OlderKids => model.suits_children(),
            Tier::Twelve => model.suits_teens(),
            Tier::Fifteen | Tier::Adult => !model
                .content
                .iter()
                .any(|c| matches!(c.as_str(), "sex" | "nudity")),
        };
    }
    if item.cert > tier.max_cert() {
        return false;
    }
    if !tier.is_kids() {
        return true;
    }
    if KIDS_BLOCKED_GENRES.iter().any(|g| has_genre(item, g)) {
        return false;
    }
    let blocked_word = item
        .keywords
        .iter()
        .any(|k| KIDS_BLOCKED_KEYWORDS.iter().any(|b| k.contains(b)));
    if blocked_word {
        return false;
    }
    let kids_or_family = has_genre(item, "Kids") || has_genre(item, "Family");
    let kids_reality = has_genre(item, "Reality")
        && !matches!(item.tv_type.as_deref(), Some("Talk Show" | "News"));
    match tier {
        Tier::LittleKids => kids_or_family,
        _ => kids_or_family || has_genre(item, "Animation") || kids_reality,
    }
}

/// A page of an adult genre does not carry children's titles for an adult caller. Kids tiers are
/// limited by eligibility, and pages about kids, family, animation or anime keep them.
#[must_use]
pub fn audience_fits(
    keeps_children: bool,
    tier: Tier,
    models: &ProfileStore,
    item: &CatalogItem,
) -> bool {
    if keeps_children || tier.is_kids() {
        return true;
    }
    if item.cert != 0 {
        return item.cert >= 3;
    }
    if let Some(model) = models.get(item.id) {
        return model.audience & (TEEN | ADULT) != 0;
    }
    !has_genre(item, "Kids") && !has_genre(item, "Family")
}

/// Whether a title is made for children, for a children's home page. A U or PG certificate alone
/// is not enough.
#[must_use]
pub fn kid_interest(models: &ProfileStore, item: &CatalogItem) -> bool {
    match models.get(item.id) {
        Some(model) => model.audience & (KIDS | FAMILY) != 0,
        None => item
            .genres
            .iter()
            .any(|g| matches!(g.as_str(), "Kids" | "Family" | "Animation")),
    }
}

/// A title made for children: Kids genre, a US TV-Y or TV-Y7 rating, a children's channel, or an
/// animated series rated U with no US rating.
#[must_use]
pub fn is_childrens_title(channels: &KidsChannels, item: &CatalogItem) -> bool {
    item.genres.iter().any(|g| g == "Kids")
        || matches!(item.cert_us, 6 | 7)
        || item.networks.iter().any(|n| channels.holds(n))
        || (item.media_type == "tv"
            && item.cert_us == 0
            && item.cert_gb == 1
            && item.genres.iter().any(|g| g == "Animation"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(genres: &[&str], keywords: &[&str], adult: bool, flags: u8) -> CatalogItem {
        CatalogItem {
            id: 1,
            title_id: 1,
            media_type: "tv".into(),
            title: "t".into(),
            release_year: None,
            genres: genres.iter().map(ToString::to_string).collect(),
            overview: String::new(),
            tagline: None,
            director: None,
            poster_path: None,
            backdrop_path: None,
            vote_average: 7.0,
            popularity: 10.0,
            is_anime: false,
            vote_count: 100,
            adult,
            original_language: "en".into(),
            tv_type: None,
            countries: vec![],
            keywords: keywords.iter().map(ToString::to_string).collect(),
            runtime: 0,
            networks: vec![],
            other_names: vec![],
            flags,
            cert: 0,
            cert_gb: 0,
            cert_us: 0,
        }
    }

    fn with_cert(mut it: CatalogItem, cert: u8) -> CatalogItem {
        it.cert = cert;
        it
    }

    fn rated(cert: u8) -> CatalogItem {
        with_cert(item(&["Drama"], &[], false, 0), cert)
    }

    fn allows(tier: Tier, it: &CatalogItem) -> bool {
        eligible(tier, it, &ProfileStore::empty())
    }

    fn with_model(line: &str) -> ProfileStore {
        let dir = std::env::temp_dir().join(format!("rules_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.jsonl", line.len()));
        std::fs::write(&path, line).unwrap();
        ProfileStore::load(&path)
    }

    #[test]
    fn s67_a_known_age_rating_decides_what_each_tier_sees() {
        let kids_title = |c| with_cert(item(&["Kids", "Family"], &[], false, 0), c);
        assert!(
            allows(Tier::LittleKids, &kids_title(1)) && !allows(Tier::LittleKids, &kids_title(2))
        );
        assert!(
            allows(Tier::OlderKids, &kids_title(2)) && !allows(Tier::OlderKids, &kids_title(3))
        );
        assert!(!allows(Tier::OlderKids, &rated(2)));
        let mystery = with_cert(item(&["Mystery", "Drama", "Crime"], &[], false, 0), 2);
        assert!(!allows(Tier::OlderKids, &mystery));
        assert!(!allows(
            Tier::OlderKids,
            &with_cert(item(&["Family"], &[], false, 0), 0)
        ));
        assert!(allows(Tier::Twelve, &rated(3)) && !allows(Tier::Twelve, &rated(4)));
        assert!(allows(Tier::Fifteen, &rated(4)) && !allows(Tier::Fifteen, &rated(5)));
        assert!(allows(Tier::Adult, &rated(5)));
    }

    #[test]
    fn s67_policy_by_genre_keyword_and_flag() {
        let kids_show = with_cert(item(&["Kids", "Animation"], &[], false, 0), 1);
        let family_film = with_cert(item(&["Family", "Comedy"], &["friendship"], false, 0), 1);
        let adult_cartoon = with_cert(item(&["Animation", "Comedy"], &[], false, 0), 2);
        let horror = with_cert(item(&["Horror", "Family"], &[], false, 0), 1);
        let violent = with_cert(
            item(&["Animation", "Action"], &["gore", "war"], false, 0),
            2,
        );
        let drama = with_cert(item(&["Drama"], &[], false, 0), 2);
        let adult_flag = item(&["Drama"], &[], true, 0);
        let suggestive = item(&["Animation"], &[], false, FLAG_SUGGESTIVE);
        for tier in [
            Tier::LittleKids,
            Tier::OlderKids,
            Tier::Twelve,
            Tier::Fifteen,
            Tier::Adult,
        ] {
            assert!(allows(tier, &kids_show), "{tier:?} kids show");
        }
        assert!(allows(Tier::LittleKids, &family_film));
        assert!(!allows(Tier::LittleKids, &adult_cartoon));
        assert!(allows(Tier::OlderKids, &adult_cartoon));
        for tier in [Tier::LittleKids, Tier::OlderKids] {
            assert!(!allows(tier, &horror), "{tier:?} horror");
            assert!(!allows(tier, &violent), "{tier:?} violent keywords");
            assert!(!allows(tier, &drama), "{tier:?} plain drama");
        }
        assert!(allows(Tier::Fifteen, &drama) && allows(Tier::Fifteen, &horror));
        assert!(!allows(Tier::Fifteen, &adult_flag) && !allows(Tier::Fifteen, &suggestive));
        assert!(allows(Tier::Adult, &adult_flag));
    }

    #[test]
    fn h33_s196_kids_reality_follows_the_server_rule() {
        let show = with_cert(item(&["Reality"], &[], false, 0), 2);
        assert!(allows(Tier::OlderKids, &show));
        assert!(!allows(Tier::LittleKids, &with_cert(show.clone(), 1)));
        let mut talk = show;
        talk.tv_type = Some("Talk Show".into());
        assert!(!allows(Tier::OlderKids, &talk));
    }

    #[test]
    fn s67_h32_an_unrated_title_with_no_profile_is_refused_below_18() {
        let unrated = item(&["Drama"], &[], false, 0);
        for tier in [
            Tier::LittleKids,
            Tier::OlderKids,
            Tier::Twelve,
            Tier::Fifteen,
        ] {
            assert!(!allows(tier, &unrated), "{tier:?}");
        }
        assert!(allows(Tier::Adult, &unrated));
    }

    #[test]
    fn s68_an_unrated_title_uses_the_model_profile() {
        let kids = with_model(r#"{"k":"movie:1","p":{"audience":"kids, family","content":[]}}"#);
        let unrated = {
            let mut it = item(&["Drama"], &[], false, 0);
            it.media_type = "movie".into();
            it
        };
        assert!(eligible(Tier::OlderKids, &unrated, &kids));
        assert!(eligible(Tier::Twelve, &unrated, &kids));
        let grown = with_model(r#"{"k":"movie:1","p":{"audience":"adult","content":["nudity"]}}"#);
        assert!(!eligible(Tier::OlderKids, &unrated, &grown));
        assert!(!eligible(Tier::Twelve, &unrated, &grown));
        assert!(!eligible(Tier::Fifteen, &unrated, &grown));
        let adult_flag = item(&["Drama"], &[], true, 0);
        assert!(!eligible(Tier::OlderKids, &adult_flag, &kids));
    }

    #[test]
    fn s70_audience_on_an_adult_genre_page() {
        let none = ProfileStore::empty();
        let family = with_cert(item(&["Family"], &[], false, 0), 2);
        assert!(!audience_fits(false, Tier::Adult, &none, &family));
        assert!(audience_fits(true, Tier::Adult, &none, &family));
        assert!(audience_fits(false, Tier::OlderKids, &none, &family));
        assert!(audience_fits(false, Tier::Adult, &none, &rated(3)));
        assert!(!audience_fits(
            false,
            Tier::Adult,
            &none,
            &item(&["Kids"], &[], false, 0)
        ));
    }

    /// The channel names belong to the data set, so a test names its own.
    fn channels() -> KidsChannels {
        KidsChannels::from_names(["A Kids Channel"])
    }

    #[test]
    fn s71_childrens_titles() {
        let listed = channels();
        let mut series = item(&["Comedy"], &[], false, 0);
        assert!(!is_childrens_title(&listed, &series));
        series.networks = vec!["A Kids Channel".into()];
        assert!(is_childrens_title(&listed, &series));
        let mut young = item(&["Drama"], &[], false, 0);
        young.cert_us = 6;
        assert!(is_childrens_title(&listed, &young));
        let mut cartoon = item(&["Animation"], &[], false, 0);
        cartoon.cert_gb = 1;
        assert!(is_childrens_title(&listed, &cartoon));
        assert!(kid_interest(&ProfileStore::empty(), &cartoon));
        assert!(!kid_interest(&ProfileStore::empty(), &rated(1)));
    }

    /// Why a set with no channel list may fail open on this clause: the age gate does not read it,
    /// and every reader asks `tier.is_kids() || !is_childrens_title(..)`, so an empty list changes
    /// an adult page and no child's page. A reader that ever uses the clause to let a title
    /// through to a child breaks this test, and the decision has to be taken again.
    #[test]
    fn s71_with_no_channel_list_only_an_adult_page_changes() {
        let listed = channels();
        let empty = KidsChannels::default();
        let mut series = with_cert(item(&["Family"], &[], false, 0), 1);
        series.networks = vec!["A Kids Channel".into()];
        assert!(is_childrens_title(&listed, &series));
        assert!(!is_childrens_title(&empty, &series), "the clause is off");
        // the question every reader of the rule asks: may this surface show the title?
        let shown = |tier: Tier, channels: &KidsChannels| {
            tier.is_kids() || !is_childrens_title(channels, &series)
        };
        for tier in [Tier::LittleKids, Tier::OlderKids] {
            assert!(allows(tier, &series), "{tier:?} may see it by its rating");
            assert_eq!(shown(tier, &listed), shown(tier, &empty), "{tier:?}");
        }
        assert!(!shown(Tier::Adult, &listed));
        assert!(
            shown(Tier::Adult, &empty),
            "with no list the adult page loses the exclusion"
        );
    }
}
