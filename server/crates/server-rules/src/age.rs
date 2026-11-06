//! What an age mark says, for a reader of one rating system. A title shows its real certificate in
//! that system. When it has none there, a neutral age stands in, taken from the other system.
//! Nothing is converted into a certificate it does not hold, and nothing is guessed.

use contract::screen::{AgeMark, AgeTone};

use server_catalog::catalog::{gb_label, us_label};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingSystem {
    British,
    American,
}

/// The system a country reads: Britain and Ireland read the British one, everyone else the American.
#[must_use]
pub fn system_of_country(country: Option<&str>) -> RatingSystem {
    match country
        .map(|code| code.trim().to_ascii_uppercase())
        .as_deref()
    {
        Some("GB" | "UK" | "IE") => RatingSystem::British,
        _ => RatingSystem::American,
    }
}

/// The age band of a US code, 1 to 5, on the same scale as the British ranks.
fn us_band(code: u8) -> Option<u8> {
    Some(match code {
        1 | 6..=8 => 1,
        2 | 9 => 2,
        3 => 3,
        4 | 10 => 4,
        5 | 11 => 5,
        _ => return None,
    })
}

fn band_tone(band: u8) -> AgeTone {
    match band {
        1 => AgeTone::Green,
        2 => AgeTone::Yellow,
        3 => AgeTone::Orange,
        4 => AgeTone::Pink,
        5 => AgeTone::Red,
        _ => AgeTone::Grey,
    }
}

/// American certificates have no official colours, so they follow one logical scale.
fn us_tone(code: u8) -> AgeTone {
    match code {
        1 | 6 | 8 => AgeTone::Green,
        2 | 7 | 9 => AgeTone::Yellow,
        3 | 10 => AgeTone::Orange,
        4 | 11 => AgeTone::Red,
        5 => AgeTone::DarkRed,
        _ => AgeTone::Grey,
    }
}

fn neutral(band: u8) -> AgeMark {
    let label = match band {
        1 => "All",
        2 => "8+",
        3 => "12+",
        4 => "15+",
        _ => "18+",
    };
    AgeMark {
        label: label.to_owned(),
        tone: band_tone(band),
    }
}

/// The mark for a title's British rank and US code, either of which may be 0, or none at all.
#[must_use]
pub fn age_mark(system: RatingSystem, gb: u8, us: u8) -> Option<AgeMark> {
    let british = gb_label(gb).map(|label| AgeMark {
        label: label.to_owned(),
        tone: band_tone(gb),
    });
    let american = us_label(us).map(|label| AgeMark {
        label: label.to_owned(),
        tone: us_tone(us),
    });
    match system {
        RatingSystem::British => british.or_else(|| us_band(us).map(neutral)),
        RatingSystem::American => american.or_else(|| (gb > 0).then(|| neutral(gb))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_shows_its_own_certificate_in_the_readers_system() {
        let mark = age_mark(RatingSystem::British, 4, 11).unwrap();
        assert_eq!((mark.label.as_str(), mark.tone), ("15", AgeTone::Pink));
        let mark = age_mark(RatingSystem::American, 4, 3).unwrap();
        assert_eq!((mark.label.as_str(), mark.tone), ("PG-13", AgeTone::Orange));
    }

    #[test]
    fn a_title_rated_only_elsewhere_gets_a_neutral_age() {
        assert_eq!(age_mark(RatingSystem::American, 3, 0).unwrap().label, "12+");
        assert_eq!(age_mark(RatingSystem::American, 1, 0).unwrap().label, "All");
        assert_eq!(age_mark(RatingSystem::British, 0, 11).unwrap().label, "18+");
    }

    #[test]
    fn a_title_with_no_rating_has_no_mark() {
        assert_eq!(age_mark(RatingSystem::British, 0, 0), None);
        assert_eq!(age_mark(RatingSystem::American, 0, 0), None);
    }

    #[test]
    fn american_tones_follow_their_scale() {
        assert_eq!(
            age_mark(RatingSystem::American, 0, 11).unwrap().tone,
            AgeTone::Red
        );
        assert_eq!(
            age_mark(RatingSystem::American, 0, 5).unwrap().tone,
            AgeTone::DarkRed
        );
    }

    #[test]
    fn britain_and_ireland_read_british_and_everyone_else_american() {
        assert_eq!(system_of_country(Some("gb")), RatingSystem::British);
        assert_eq!(system_of_country(Some("IE")), RatingSystem::British);
        assert_eq!(system_of_country(Some("NG")), RatingSystem::American);
        assert_eq!(system_of_country(None), RatingSystem::American);
    }
}
