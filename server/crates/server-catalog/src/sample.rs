//! A small invented data set for tests. Every title, name and number is generated here; nothing
//! comes from a real catalogue. Written as the real files, so tests go through the manifest.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use half::f16;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

use crate::catalog::item_key;
use crate::manifest::{DatasetManifest, FileEntry, SCHEMA_VERSION, role, sha256_file};

/// Width of the sample vectors.
pub const SAMPLE_DIMS: usize = 16;

/// One invented title.
#[derive(Clone, Debug)]
pub struct SampleTitle {
    pub id: u32,
    pub kind: &'static str,
    pub name: String,
    pub genres: Vec<&'static str>,
    /// The certificate cell, as the country map.
    pub certs: Option<String>,
    pub keywords: Vec<&'static str>,
    pub adult: bool,
    pub vote: f64,
    pub popularity: f64,
    pub votes: u32,
    pub year: u32,
    pub other_names: Vec<String>,
}

const FIRST: [&str; 7] = ["Cop", "Har", "Vel", "Qui", "Am", "Nor", "Pap"];
const SECOND: [&str; 11] = [
    "ber", "den", "lin", "mor", "pen", "ric", "sun", "tor", "vex", "wyn", "zal",
];
const THIRD: [&str; 5] = ["Lan", "Orch", "Stat", "Mead", "Comp"];
const FOURTH: [&str; 13] = [
    "ard", "bel", "cot", "dun", "eth", "fel", "gan", "hol", "ion", "jar", "kel", "lom", "mir",
];

/// An invented name. Its words differ from title to title, as real names do.
fn invented_name(n: u32) -> String {
    let at = |count: usize| n as usize % count;
    format!(
        "{}{} {}{}",
        FIRST[at(FIRST.len())],
        SECOND[at(SECOND.len())],
        THIRD[at(THIRD.len())],
        FOURTH[at(FOURTH.len())]
    )
}

const GENRES: [(&str, &str); 12] = [
    ("Drama", "15"),
    ("Comedy", "12"),
    ("Action", "15"),
    ("Thriller", "18"),
    ("Science Fiction", "12"),
    ("Romance", "15"),
    ("Family", "U"),
    ("Animation", "PG"),
    ("Crime", "15"),
    ("History", "12"),
    ("Music", "PG"),
    ("Mystery", "12"),
];

/// Invented titles of the regular kind.
const REGULAR_TITLES: u32 = 480;

/// Fixed key numbers of the titles with a special part in tests.
pub mod special {
    /// Rated 18, adult flag.
    pub const ADULT_FLAG: u32 = 9001;
    /// Series and film share this id.
    pub const SHARED_ID: u32 = 9002;
    /// No certificate and no model profile.
    pub const UNRATED_NO_PROFILE: u32 = 9003;
    /// No certificate, a children's model profile.
    pub const UNRATED_KIDS_PROFILE: u32 = 9004;
    /// Rated PG, a Reality show for children.
    pub const KIDS_REALITY: u32 = 9005;
    /// Rated U, with an accent in its name and a second name in another language.
    pub const ACCENT_NAME: u32 = 9006;
}

/// The titles of the sample.
#[must_use]
pub fn sample_titles() -> Vec<SampleTitle> {
    let mut titles = Vec::new();
    for n in 0..REGULAR_TITLES {
        let (genre, cert) = GENRES[n as usize % GENRES.len()];
        let kind = if n % 3 == 2 { "tv" } else { "movie" };
        let cert_cell = if kind == "tv" {
            serde_json::json!({ "GB": cert, "US": "TV-14" })
        } else {
            serde_json::json!({ "GB": cert })
        };
        titles.push(SampleTitle {
            id: 100 + n,
            kind,
            name: invented_name(n),
            genres: if genre == "Animation" {
                vec![genre, "Family"]
            } else {
                vec![genre]
            },
            certs: Some(cert_cell.to_string()),
            keywords: Vec::new(),
            adult: false,
            vote: 6.2 + f64::from(n % 27) / 10.0,
            popularity: 3.0 + f64::from(REGULAR_TITLES - n),
            votes: 40 + n * 25,
            year: 1975 + n % 50,
            other_names: Vec::new(),
        });
    }
    let special = |id, kind, name: &str, genres, certs: Option<&str>| SampleTitle {
        id,
        kind,
        name: name.to_owned(),
        genres,
        certs: certs.map(str::to_owned),
        keywords: Vec::new(),
        adult: false,
        vote: 7.0,
        popularity: 40.0,
        votes: 500,
        year: 2020,
        other_names: Vec::new(),
    };
    let mut adult = special(
        special::ADULT_FLAG,
        "movie",
        "Grown Up Only",
        vec!["Drama"],
        Some(r#"{"GB":"18"}"#),
    );
    adult.adult = true;
    titles.push(adult);
    titles.push(special(
        special::SHARED_ID,
        "movie",
        "Twin Film",
        vec!["Drama"],
        Some(r#"{"GB":"15"}"#),
    ));
    titles.push(special(
        special::SHARED_ID,
        "tv",
        "Twin Series",
        vec!["Comedy"],
        Some(r#"{"GB":"U"}"#),
    ));
    titles.push(special(
        special::UNRATED_NO_PROFILE,
        "movie",
        "Nobody Rated This",
        vec!["Family"],
        None,
    ));
    titles.push(special(
        special::UNRATED_KIDS_PROFILE,
        "movie",
        "Gentle Unrated Friends",
        vec!["Family"],
        None,
    ));
    titles.push(special(
        special::KIDS_REALITY,
        "tv",
        "Little Bakers",
        vec!["Reality"],
        Some(r#"{"GB":"PG"}"#),
    ));
    let mut accent = special(
        special::ACCENT_NAME,
        "movie",
        "Café Münster",
        vec!["Family"],
        Some(r#"{"GB":"U"}"#),
    );
    accent.other_names = vec![String::from("Cat Cafe")];
    titles.push(accent);
    titles
}

fn unit_vector(title: &SampleTitle) -> Vec<f32> {
    let digest = Sha256::digest(format!("{}{}", title.kind, title.id));
    let mut row: Vec<f32> = digest[..SAMPLE_DIMS]
        .iter()
        .map(|byte| f32::from(*byte) / 255.0 - 0.5)
        .collect();
    let axis = GENRES
        .iter()
        .position(|(genre, _)| title.genres.contains(genre))
        .unwrap_or(GENRES.len());
    row[axis % SAMPLE_DIMS] += 2.5;
    crate::vectors::l2_normalize(&mut row);
    row
}

fn write_catalog(path: &Path, titles: &[SampleTitle]) -> rusqlite::Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE catalog (id INTEGER PRIMARY KEY, title_id INTEGER, media_type TEXT, title TEXT,
           release_year TEXT, genres TEXT, overview TEXT, poster_path TEXT, backdrop_path TEXT,
           vote_average REAL, popularity REAL, is_anime INTEGER, tagline TEXT, director TEXT,
           vote_count INTEGER, adult INTEGER, original_language TEXT, tv_type TEXT,
           origin_country TEXT, keywords TEXT, runtime INTEGER, certs TEXT, networks TEXT,
           other_names TEXT);
         CREATE VIRTUAL TABLE catalog_fts USING fts5(id UNINDEXED, overview, genres, director);",
    )?;
    let mut insert = conn.prepare(
        "INSERT INTO catalog VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,0,'',?12,?13,?14,'en',?15,
         '[\"GB\"]',?16,?17,?18,'[]',?19)",
    )?;
    for title in titles {
        let genres = title.genres.join(", ");
        let tv_type = (title.genres.contains(&"Reality")).then_some("Reality");
        let tv_type = tv_type.or((title.kind == "tv").then_some("Scripted"));
        insert.execute(params![
            item_key(title.kind, title.id),
            title.id,
            title.kind,
            title.name,
            title.year.to_string(),
            genres,
            format!(
                "An invented story about {} that only exists for tests of this server.",
                title.name
            ),
            format!("/poster/{}.jpg", title.id),
            format!("/backdrop/{}.jpg", title.id),
            title.vote,
            title.popularity,
            format!("Director {}", title.id % 7),
            title.votes,
            i64::from(title.adult),
            tv_type,
            serde_json::to_string(&title.keywords).unwrap_or_default(),
            if title.kind == "tv" { 45 } else { 100 },
            title.certs,
            serde_json::to_string(&title.other_names).unwrap_or_default(),
        ])?;
    }
    conn.execute_batch(
        "INSERT INTO catalog_fts (id, overview, genres, director)
         SELECT id, overview, genres, director FROM catalog;",
    )?;
    Ok(())
}

/// One seed per genre, as the offline grouping would make.
fn seed_bytes(titles: &[SampleTitle]) -> Vec<u8> {
    let mut groups: Vec<(Vec<f32>, Vec<u32>)> = Vec::new();
    for (genre, _) in GENRES {
        let members: Vec<&SampleTitle> = titles
            .iter()
            .filter(|title| title.id < 9000 && title.genres.first() == Some(&genre))
            .collect();
        let mut centre = vec![0.0f32; SAMPLE_DIMS];
        for title in &members {
            for (sum, value) in centre.iter_mut().zip(unit_vector(title)) {
                *sum += value;
            }
        }
        crate::vectors::l2_normalize(&mut centre);
        let keys = members
            .iter()
            .map(|title| item_key(title.kind, title.id))
            .collect();
        groups.push((centre, keys));
    }
    let mut bytes = Vec::new();
    let width = |value: usize| u32::try_from(value).unwrap_or(0).to_le_bytes();
    bytes.extend_from_slice(&width(groups.len()));
    bytes.extend_from_slice(&width(SAMPLE_DIMS));
    for (centre, keys) in groups {
        for value in centre {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&width(keys.len()));
        for key in keys {
            bytes.extend_from_slice(&key.to_le_bytes());
        }
    }
    bytes
}

/// Release dates near the sample's own time: 2023-11-14.
fn fresh_json(titles: &[SampleTitle]) -> String {
    let items: Vec<serde_json::Value> = titles
        .iter()
        .take(48)
        .enumerate()
        .map(|(n, title)| {
            let day = crate::fresh::parse_days("2023-11-14").unwrap_or(0) + 12 - n as i64;
            serde_json::json!({
                "media_type": title.kind,
                "id": title.id,
                "title": title.name,
                "release_date": crate::fresh::ymd(day),
                "popularity": title.popularity,
                "lists": ["trending_day"],
            })
        })
        .collect();
    serde_json::json!({ "generated": "sample", "items": items }).to_string()
}

const EVENTS_JSON: &str = r#"{"events":{
  "autumn":{"label":"Autumn Evenings","season":{"start":"10-01","end":"12-01"},"genres":["Drama"],"languages":["en"]},
  "midwinter":{"label":"Midwinter Feast","season":{"start":"12-20","end":"01-05"},"keywords":["feast"]}
}}"#;

fn entry(dir: &Path, name: &str, required: bool) -> std::io::Result<FileEntry> {
    let path = dir.join(name);
    Ok(FileEntry {
        path: name.to_owned(),
        sha256: sha256_file(&path)?,
        bytes: std::fs::metadata(&path)?.len(),
        required,
    })
}

/// One row request per genre, as a data set with composed rows holds.
fn requests_json() -> String {
    let requests: Vec<serde_json::Value> = GENRES
        .iter()
        .enumerate()
        .map(|(n, (genre, _))| {
            serde_json::json!({
                "id": format!("sample-{n}"),
                "name": format!("{genre} Stories"),
                "kind": "theme",
                "rules": { "genres_any": [genre] },
                "page": "home",
                "format": "both",
                "profile": "any",
            })
        })
        .collect();
    serde_json::json!({ "version": 2, "requests": requests }).to_string()
}

/// The sample, with composed row requests. Deep rows page through cursors.
///
/// # Panics
/// A test folder cannot be written.
#[must_use]
pub fn write_composed_sample(dir: &Path) -> PathBuf {
    let path = write_sample(dir);
    std::fs::write(dir.join("requests.json"), requests_json()).expect("sample requests");
    let text = std::fs::read(&path).expect("manifest");
    let mut manifest: DatasetManifest = serde_json::from_slice(&text).expect("manifest");
    manifest.files.insert(
        role::REQUESTS.to_owned(),
        entry(dir, "requests.json", false).expect("sample entry"),
    );
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&manifest).expect("manifest"),
    )
    .expect("manifest file");
    path
}

/// Write the sample into `dir` and return the manifest path.
///
/// # Panics
/// A test folder cannot be written.
#[must_use]
pub fn write_sample(dir: &Path) -> PathBuf {
    std::fs::create_dir_all(dir).expect("sample folder");
    let titles = sample_titles();
    let catalog = dir.join("catalog.db");
    let _ = std::fs::remove_file(&catalog);
    write_catalog(&catalog, &titles).expect("sample catalogue");
    let mut vectors = Vec::new();
    let mut keys = Vec::new();
    for title in &titles {
        for value in unit_vector(title) {
            vectors.extend_from_slice(&f16::from_f32(value).to_le_bytes());
        }
        keys.extend_from_slice(&item_key(title.kind, title.id).to_le_bytes());
    }
    std::fs::write(dir.join("vectors.bin"), vectors).expect("sample vectors");
    std::fs::write(dir.join("vectors.keys"), keys).expect("sample keys");
    let profile = serde_json::json!({
        "k": format!("movie:{}", special::UNRATED_KIDS_PROFILE),
        "p": { "audience": "kids, family", "content": [], "themes": ["friendship"], "tone": ["gentle"] }
    });
    std::fs::write(dir.join("profiles.jsonl"), format!("{profile}\n")).expect("sample profiles");
    std::fs::write(dir.join("seeds.bin"), seed_bytes(&titles)).expect("sample seeds");
    let names: Vec<String> = GENRES
        .iter()
        .map(|(genre, _)| format!("{genre} Evenings"))
        .collect();
    std::fs::write(
        dir.join("seeds.names"),
        serde_json::to_vec(&names).expect("names"),
    )
    .expect("sample names");
    std::fs::write(dir.join("fresh.json"), fresh_json(&titles)).expect("sample fresh list");
    std::fs::write(dir.join("events.json"), EVENTS_JSON).expect("sample events");
    let mut files = BTreeMap::new();
    for (name, file, required) in [
        (role::CATALOG, "catalog.db", true),
        (role::VECTORS, "vectors.bin", true),
        (role::VECTOR_KEYS, "vectors.keys", true),
        (role::PROFILES, "profiles.jsonl", false),
        (role::SEEDS, "seeds.bin", false),
        (role::SEED_NAMES, "seeds.names", false),
        (role::FRESH, "fresh.json", false),
        (role::EVENTS, "events.json", false),
    ] {
        files.insert(
            name.to_owned(),
            entry(dir, file, required).expect("sample entry"),
        );
    }
    let manifest = DatasetManifest {
        schema_version: SCHEMA_VERSION,
        dataset_version: String::from("sample-1"),
        build_commit: String::from("sample"),
        built_at_ms: 1_700_000_000_000,
        model: String::from("sample-hash"),
        dimensions: SAMPLE_DIMS as u32,
        quantisation: String::from("f16"),
        row_count: titles.len() as u64,
        files,
    };
    let path = dir.join("manifest.json");
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&manifest).expect("manifest"),
    )
    .expect("manifest file");
    path
}
