//! The one binary: `serve`, `migrate`, and `index build` for the index this server owns.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{AccountStore, Door, HttpAuth, PgStore, SystemClock};
use server_api::{AppState, Config};
use server_catalog::DataSlot;
use server_library::{Library, LibraryStore, PgLibrary};
use server_play::{HttpFinder, Play};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

fn open_data(config: &Config) -> Option<Arc<DataSlot>> {
    let manifest = config.data_manifest.as_deref()?;
    match DataSlot::open(manifest) {
        Ok(slot) => Some(Arc::new(slot)),
        Err(error) => {
            warn!("Data set refused, titles stay off: {error}");
            None
        }
    }
}

/// Open the index in the background. Until then the exact scan serves.
fn start_index(config: &Config, data: &Arc<DataSlot>) {
    let Some(path) = config.index_path.clone() else {
        return;
    };
    let set = data.load();
    tokio::task::spawn_blocking(move || {
        if !set.open_index(&path)
            && let Err(error) = set.build_index(&path)
        {
            warn!("Index not built: {error}");
        }
    });
}

async fn open_accounts(config: &Config) -> Result<Option<(Door, Library)>> {
    let Some(url) = &config.database_url else {
        return Ok(None);
    };
    let pool = sqlx::PgPool::connect(url)
        .await
        .context("the database did not answer")?;
    let store: Arc<dyn AccountStore> = Arc::new(PgStore::new(pool.clone()));
    let books: Arc<dyn LibraryStore> = Arc::new(PgLibrary::new(pool));
    let clock = Arc::new(SystemClock);
    let mut door = Door::new(store, clock.clone(), DEFAULT_LIMITS);
    if let (Some(address), Some(token)) = (&config.auth_address, &config.auth_admin_token) {
        let auth = HttpAuth::new(address, token).context("the auth client did not build")?;
        door = door.with_auth(Arc::new(auth));
    }
    Ok(Some((door, Library::new(books, clock))))
}

fn open_play(config: &Config) -> Result<Option<Play>> {
    let Some(token) = config
        .finder_token
        .as_deref()
        .filter(|_| config.playback_ready())
    else {
        return Ok(None);
    };
    let finder =
        HttpFinder::new(&config.finder_urls, token).context("the finder client did not build")?;
    Ok(Some(Play::new(Arc::new(finder))))
}

async fn serve(config: Config) -> Result<()> {
    for off in &config.off {
        warn!("Off: {} ({})", off.feature, off.reason);
    }
    let Some(listen) = config.listen.clone() else {
        bail!("SERVER_LISTEN is not set, so there is nothing to listen on");
    };
    let data = open_data(&config);
    if let Some(data) = &data {
        start_index(&config, data);
    }
    let (door, library) = open_accounts(&config).await?.unzip();
    let play = open_play(&config)?;
    let mut state = AppState::new(config, door, data, library);
    state.play = play;
    let state = Arc::new(state);
    let app = server_api::router(state).into_make_service_with_connect_info::<SocketAddr>();
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    info!("Listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Bring the database up to this build. The chain only goes forward, and each file leaves the
/// previous release working.
async fn migrate(config: &Config) -> Result<()> {
    let url = config
        .database_url
        .as_deref()
        .context("DATABASE_URL is not set")?;
    let pool = sqlx::PgPool::connect(url)
        .await
        .context("the database did not answer")?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    info!("Migrated");
    Ok(())
}

fn build_index(config: &Config) -> Result<()> {
    let manifest = config
        .data_manifest
        .as_deref()
        .context("DATA_MANIFEST is not set")?;
    let path = config
        .index_path
        .as_deref()
        .context("INDEX_PATH is not set")?;
    let slot = DataSlot::open(manifest)?;
    let set = slot.load();
    set.build_index(Path::new(path))?;
    info!("Index built");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let config = Config::from_env();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        [] | ["serve"] => serve(config).await,
        ["migrate"] => migrate(&config).await,
        ["index", "build"] => build_index(&config),
        _ => bail!("use: server [serve | migrate | index build]"),
    }
}
