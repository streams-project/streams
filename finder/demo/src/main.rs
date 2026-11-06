//! Start the demo finder from the environment.

use finder_demo::{Settings, parse_films, router};

fn load() -> Settings {
    let mut settings = Settings {
        build: option_env!("FINDER_BUILD").unwrap_or("dev").to_owned(),
        ..Settings::default()
    };
    for name in ["FINDER_SERVICE_TOKEN", "FINDER_SERVICE_TOKEN_NEXT"] {
        if let Some(token) = std::env::var(name).ok().filter(|t| !t.is_empty()) {
            settings.tokens.push(token);
        }
    }
    if settings.tokens.is_empty() {
        settings
            .off
            .push("finding is off: FINDER_SERVICE_TOKEN is not set".to_owned());
    }
    match std::env::var("FINDER_DEMO_FILMS") {
        Err(_) => settings
            .off
            .push("no films: FINDER_DEMO_FILMS is not set".to_owned()),
        Ok(path) => {
            let read = std::fs::read_to_string(&path).map_err(|e| e.to_string());
            match read.and_then(|text| parse_films(&text).map_err(|e| e.to_string())) {
                Ok(films) => settings.films = films,
                Err(why) => settings.off.push(format!("no films: {why}")),
            }
        }
    }
    settings
}

#[tokio::main]
async fn main() {
    let Ok(bind) = std::env::var("FINDER_BIND") else {
        eprintln!("not started: FINDER_BIND is not set");
        std::process::exit(2);
    };
    let settings = load();
    for line in &settings.off {
        eprintln!("{line}");
    }
    let listener = match tokio::net::TcpListener::bind(&bind).await {
        Ok(listener) => listener,
        Err(why) => {
            eprintln!("not started: {why}");
            std::process::exit(2);
        }
    };
    if let Err(why) = axum::serve(listener, router(settings)).await {
        eprintln!("stopped: {why}");
        std::process::exit(1);
    }
}
