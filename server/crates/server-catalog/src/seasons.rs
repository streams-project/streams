//! Seasons and episodes of a series. They stay in the catalogue file, and one series is read at a
//! time, because the full list is far larger than anything a page needs at once.

use std::path::Path;

use rusqlite::{Connection, OpenFlags, params};

/// A season a title page names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonFact {
    pub number: u32,
    pub episodes: u32,
}

/// One episode as the data set holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpisodeFact {
    pub number: u32,
    pub name: String,
    pub overview: Option<String>,
    pub still: Option<String>,
    pub runtime: Option<u32>,
}

fn open(db: &Path) -> Option<Connection> {
    Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
}

fn filled(text: Option<String>) -> Option<String> {
    text.filter(|text| !text.is_empty())
}

/// The numbered seasons of a series. Season 0 holds specials, which no menu lists. A set built
/// without episodes answers none.
#[must_use]
pub fn seasons_of(db: &Path, series: u32) -> Vec<SeasonFact> {
    let read = || -> rusqlite::Result<Vec<SeasonFact>> {
        let Some(conn) = open(db) else {
            return Ok(Vec::new());
        };
        let mut statement = conn.prepare(
            "SELECT season_number, count(*) FROM episodes
             WHERE media_type = 'tv' AND title_id = ?1 AND season_number > 0
             GROUP BY season_number ORDER BY season_number",
        )?;
        let rows = statement.query_map(params![series], |row| {
            Ok(SeasonFact {
                number: row.get(0)?,
                episodes: row.get(1)?,
            })
        })?;
        rows.collect()
    };
    read().unwrap_or_default()
}

/// One season's name and episodes, or none when the set does not have it.
#[must_use]
pub fn episodes_of(
    db: &Path,
    series: u32,
    season: u32,
) -> Option<(Option<String>, Vec<EpisodeFact>)> {
    let conn = open(db)?;
    let name = conn
        .query_row(
            "SELECT name FROM seasons WHERE media_type = 'tv' AND title_id = ?1 AND season_number = ?2",
            params![series, season],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .and_then(filled);
    let mut statement = conn
        .prepare(
            "SELECT episode_number, name, overview, still_path, runtime FROM episodes
             WHERE media_type = 'tv' AND title_id = ?1 AND season_number = ?2
             ORDER BY episode_number",
        )
        .ok()?;
    let rows = statement
        .query_map(params![series, season], |row| {
            Ok(EpisodeFact {
                number: row.get(0)?,
                name: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                overview: filled(row.get(2)?),
                still: filled(row.get(3)?),
                runtime: row.get(4)?,
            })
        })
        .ok()?;
    let episodes: Vec<EpisodeFact> = rows.collect::<rusqlite::Result<_>>().ok()?;
    (!episodes.is_empty()).then_some((name, episodes))
}
