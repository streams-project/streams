//! Every row name the page builder makes on a real data set, for the translation tables.

use std::collections::BTreeSet;
use std::path::PathBuf;

use contract::client::DeviceClass;
use contract::screen::ScreenName;
use server_catalog::DataSet;
use server_catalog::manifest::DataTrust;
use server_pages::{Pages, Personal, ScreenAsk};
use server_rules::answer::Images;
use server_rules::{Tier, Viewer};

/// Sessions to build each page for: the rows rotate with the session, so many seeds see most names.
const SEEDS: usize = 12;

const TIERS: [Tier; 5] = [
    Tier::Adult,
    Tier::Fifteen,
    Tier::Twelve,
    Tier::OlderKids,
    Tier::LittleKids,
];

fn ask(screen: ScreenName, genre: Option<String>, seed: usize, now_ms: u64) -> ScreenAsk {
    ScreenAsk {
        screen,
        genre,
        class: DeviceClass::Tv,
        now_ms,
        owner: format!("names-{seed}"),
        seed: format!("names-{seed}"),
        personal: Personal::default(),
        blocked_rows: Vec::new(),
        language: String::new(),
    }
}

#[test]
#[ignore = "needs a data set: ROW_NAMES_DATA and ROW_NAMES_OUT"]
fn every_row_name() {
    let folder = PathBuf::from(std::env::var("ROW_NAMES_DATA").expect("ROW_NAMES_DATA"));
    let out = PathBuf::from(std::env::var("ROW_NAMES_OUT").expect("ROW_NAMES_OUT"));
    let trust = DataTrust::from_key_files(&[folder.join("data_key.pem")]).expect("key");
    let data = DataSet::load(&folder.join("manifest.json"), &trust).expect("data set");
    let pages = Pages::new(Images::new(None));
    // A year of days, so every calendar window opens once.
    let start: u64 = 1_767_225_600_000;
    let day: u64 = 86_400_000;
    let mut names = BTreeSet::new();
    for tier in TIERS {
        let viewer = Viewer::visitor(tier);
        let nav = pages.nav(&data, tier, "");
        let mut asks: Vec<(ScreenName, Option<String>)> = nav
            .items
            .iter()
            .map(|item| (item.screen, item.genre.clone()))
            .collect();
        asks.extend([
            (ScreenName::Home, None),
            (ScreenName::Series, None),
            (ScreenName::Films, None),
            (ScreenName::New, None),
            (ScreenName::Kids, None),
        ]);
        for item in &nav.items {
            names.insert(item.label.clone());
        }
        for (screen, genre) in asks {
            for seed in 0..SEEDS {
                let now = start + (seed as u64 % 12) * 30 * day + seed as u64 * day;
                let Ok(page) = pages.screen(&data, &viewer, &ask(screen, genre.clone(), seed, now))
                else {
                    continue;
                };
                let page = serde_json::to_value(&page).expect("json");
                for section in page["sections"].as_array().into_iter().flatten() {
                    if let Some(title) = section["title"].as_str() {
                        names.insert(title.to_owned());
                    }
                }
            }
        }
    }
    let list: Vec<String> = names
        .into_iter()
        .filter(|name| !name.trim().is_empty())
        .collect();
    std::fs::write(&out, serde_json::to_string_pretty(&list).expect("json")).expect("write");
    println!("{} names written to {}", list.len(), out.display());
}
