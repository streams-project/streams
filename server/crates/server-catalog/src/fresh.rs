//! What is new, upcoming and trending right now. The catalogue knows only a release year; this
//! file carries real release dates and titles too new for the catalogue.
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct FreshItem {
    pub media_type: String,
    pub id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub original_language: Option<String>,
    #[serde(default)]
    pub overview: String,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub vote_average: f32,
    #[serde(default)]
    pub vote_count: u32,
    #[serde(default)]
    pub popularity: f32,
    /// YYYY-MM-DD, empty when unknown
    #[serde(default)]
    pub release_date: String,
    #[serde(default)]
    pub adult: bool,
    #[serde(default)]
    pub lists: Vec<String>,
    /// place of the title in each source list it came from (0 first)
    #[serde(default)]
    pub ranks: BTreeMap<String, usize>,
    /// The next season of a series that has already aired, when its first episode has a date.
    #[serde(default)]
    pub next_season: Option<NextSeason>,
}

/// A season that starts after the list was made.
#[derive(Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct NextSeason {
    pub number: u32,
    /// YYYY-MM-DD
    pub air_date: String,
}

#[derive(Deserialize, Default)]
struct FreshFile {
    #[serde(default)]
    generated: String,
    #[serde(default)]
    items: Vec<FreshItem>,
}

#[derive(Default)]
pub struct FreshStore {
    pub generated: String,
    pub items: Vec<FreshItem>,
    by_key: HashMap<(String, u32), usize>,
}

impl FreshStore {
    pub fn from_json(text: &str) -> Option<Self> {
        let f: FreshFile = serde_json::from_str(text).ok()?;
        let by_key = f
            .items
            .iter()
            .enumerate()
            .map(|(i, e)| ((e.media_type.clone(), e.id), i))
            .collect();
        Some(Self {
            generated: f.generated,
            items: f.items,
            by_key,
        })
    }

    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| Self::from_json(&t))
            .unwrap_or_default()
    }

    pub fn get(&self, media_type: &str, id: u32) -> Option<&FreshItem> {
        self.by_key
            .get(&(media_type.to_string(), id))
            .map(|&i| &self.items[i])
    }

    /// True when the title has a known release date after `today` (YYYY-MM-DD).
    pub fn is_upcoming(&self, media_type: &str, id: u32, today: &str) -> bool {
        self.get(media_type, id)
            .is_some_and(|e| !e.release_date.is_empty() && e.release_date.as_str() > today)
    }
}

// ── Dates without a date library ──────────────────────────────────────────────

/// Days since 1970-01-01 for a civil date (proleptic Gregorian).
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The day of a time in milliseconds since 1970.
#[must_use]
pub fn days_of_ms(ms: u64) -> i64 {
    i64::try_from(ms / 86_400_000).unwrap_or(0)
}

pub fn ymd(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Days since 1970-01-01 for a `YYYY-MM-DD` text.
pub fn parse_days(date: &str) -> Option<i64> {
    let mut p = date.splitn(3, '-');
    let y: i64 = p.next()?.parse().ok()?;
    let m: u32 = p.next()?.parse().ok()?;
    let d: u32 = p.next()?.parse().ok()?;
    ((1..=12).contains(&m) && (1..=31).contains(&d)).then(|| days_from_civil(y, m, d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(ymd(parse_days("2026-10-13").unwrap()), "2026-10-13");
        assert_eq!(ymd(parse_days("2026-10-13").unwrap() + 20), "2026-11-02");
        assert_eq!(ymd(parse_days("2028-02-28").unwrap() + 1), "2028-02-29");
        assert!(parse_days("2026-13-01").is_none());
    }

    #[test]
    fn a_series_carries_its_next_season() {
        let s = FreshStore::from_json(r#"{"items":[{"media_type":"tv","id":5,"release_date":"2020-01-01","next_season":{"number":3,"air_date":"2026-11-21"}},{"media_type":"tv","id":6}]}"#).unwrap();
        let next = s.get("tv", 5).and_then(|item| item.next_season.clone());
        let wanted = NextSeason {
            number: 3,
            air_date: String::from("2026-11-21"),
        };
        assert_eq!(next, Some(wanted));
        let weekly = s.get("tv", 6);
        assert!(weekly.is_some_and(|item| item.next_season.is_none()));
    }

    #[test]
    fn upcoming_means_after_today() {
        let s = FreshStore::from_json(r#"{"generated":"x","items":[{"media_type":"movie","id":1,"release_date":"2026-12-01"},{"media_type":"movie","id":2,"release_date":"2026-01-01"},{"media_type":"tv","id":3}]}"#).unwrap();
        assert!(s.is_upcoming("movie", 1, "2026-10-13"));
        assert!(!s.is_upcoming("movie", 2, "2026-10-13"));
        assert!(!s.is_upcoming("tv", 3, "2026-10-13"));
        assert!(!s.is_upcoming("movie", 99, "2026-10-13"));
    }
}
