//! The open film finder, started from the environment.

use std::process::ExitCode;
use std::sync::Arc;

use finder_open_films::{OpenFilms, parse_films};
use finder_service::Provider;

fn films() -> Vec<Arc<dyn Provider>> {
    let Ok(path) = std::env::var("FINDER_FILMS") else {
        eprintln!("no films: FINDER_FILMS is not set");
        return Vec::new();
    };
    let read = std::fs::read_to_string(&path).map_err(|e| e.to_string());
    match read.and_then(|text| parse_films(&text).map_err(|e| e.to_string())) {
        Ok(films) if films.is_empty() => Vec::new(),
        Ok(films) => vec![Arc::new(OpenFilms(films))],
        Err(why) => {
            eprintln!("no films: {why}");
            Vec::new()
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    finder_service::run(films()).await
}
