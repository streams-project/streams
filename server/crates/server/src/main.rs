//! The one binary: `serve`, and `index build` for the index this server owns.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{AccountStore, Door, PgStore, SystemClock};
use server_api::{AppState, Config};
use server_catalog::DataSlot;
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

async fn open_door(config: &Config) -> Result<Option<Door>> {
    let Some(url) = &config.database_url else {
        return Ok(None);
    };
    let pool = sqlx::PgPool::connect(url)
        .await
        .context("the database did not answer")?;
    let store: Arc<dyn AccountStore> = Arc::new(PgStore::new(pool));
    Ok(Some(Door::new(
        store,
        Arc::new(SystemClock),
        DEFAULT_LIMITS,
    )))
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
    let door = open_door(&config).await?;
    let state = Arc::new(AppState::new(config, door, data));
    let app = server_api::router(state).into_make_service_with_connect_info::<SocketAddr>();
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    info!("Listening");
    axum::serve(listener, app).await?;
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
        ["index", "build"] => build_index(&config),
        _ => bail!("use: server [serve | index build]"),
    }
}
