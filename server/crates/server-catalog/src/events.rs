//! Calendar events with a page of their own. An event is its label, its window (MM-DD, which may
//! wrap the year), the keywords a title needs to belong to it and the genres of its rows.
use crate::taxonomy::Season;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct EventDef {
    pub id: String,
    pub label: String,
    pub season: Season,
    pub keywords: Vec<String>,
    pub genres: Vec<String>,
    /// Original languages whose titles belong when the event has no keywords of its own, kept to
    /// the event's genres
    pub languages: Vec<String>,
}

#[derive(Deserialize)]
struct RawSeason {
    start: String,
    end: String,
}

#[derive(Deserialize)]
struct RawEvent {
    label: String,
    season: RawSeason,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(default)]
    languages: Vec<String>,
}

#[derive(Deserialize)]
struct RawFile {
    events: std::collections::BTreeMap<String, RawEvent>,
}

/// The events of a data set.
#[derive(Debug, Default)]
pub struct EventStore {
    events: Vec<EventDef>,
}

fn parse_md(s: &str) -> Option<(u8, u8)> {
    let (m, d) = s.split_once('-')?;
    Some((m.trim().parse().ok()?, d.trim().parse().ok()?))
}

impl EventStore {
    /// Read the events file. An unreadable file leaves no events, so no event page is offered.
    #[must_use]
    pub fn load(path: &std::path::Path) -> Self {
        let file = std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str::<RawFile>(&text).ok());
        let events = file.map_or_else(Vec::new, |file| {
            file.events
                .into_iter()
                .filter_map(|(id, e)| {
                    let season = Season {
                        start: parse_md(&e.season.start)?,
                        end: parse_md(&e.season.end)?,
                    };
                    Some(EventDef {
                        id,
                        label: e.label,
                        season,
                        keywords: e.keywords.into_iter().map(|k| k.to_lowercase()).collect(),
                        genres: e.genres,
                        languages: e.languages,
                    })
                })
                .collect()
        });
        Self { events }
    }

    /// Events from a list, for tests.
    #[must_use]
    pub fn from_list(events: Vec<EventDef>) -> Self {
        Self { events }
    }

    /// Every event.
    #[must_use]
    pub fn all(&self) -> &[EventDef] {
        &self.events
    }

    /// One event by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&EventDef> {
        self.events.iter().find(|e| e.id == id)
    }

    /// The event whose label is `name`; case does not matter, and "and" equals "&".
    #[must_use]
    pub fn by_label(&self, name: &str) -> Option<&EventDef> {
        let key = name.trim().to_lowercase().replace(" and ", " & ");
        self.events.iter().find(|e| e.label.to_lowercase() == key)
    }

    /// Whether the event's window holds a day.
    #[must_use]
    pub fn in_season(&self, id: &str, today: (u8, u8)) -> bool {
        self.get(id).is_some_and(|e| e.season.active_on(today))
    }

    /// Whether a title belongs to the event: it carries one of its keywords, or it is in one of
    /// its languages and genres, for an event with no keywords of its own.
    #[must_use]
    pub fn title_belongs(&self, id: &str, item: &crate::catalog::CatalogItem) -> bool {
        self.get(id).is_some_and(|e| {
            item.keywords
                .iter()
                .any(|k| e.keywords.iter().any(|w| w == k))
                || (e.languages.contains(&item.original_language)
                    && (e.genres.is_empty()
                        || item
                            .genres
                            .iter()
                            .any(|g| e.genres.iter().any(|x| same_genre(x, g)))))
        })
    }
}

/// Whether two genre names mean the same genre across films and series.
#[must_use]
pub fn same_genre(a: &str, b: &str) -> bool {
    fn family(g: &str) -> Option<&'static str> {
        match g.to_ascii_lowercase().as_str() {
            "action" | "adventure" | "action & adventure" => Some("action"),
            "science fiction" | "fantasy" | "sci-fi & fantasy" => Some("scifi"),
            "war" | "war & politics" => Some("war"),
            _ => None,
        }
    }
    a.eq_ignore_ascii_case(b) || matches!((family(a), family(b)), (Some(x), Some(y)) if x == y)
}
