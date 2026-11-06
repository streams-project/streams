//! The one binary: `serve`, `migrate`, and `index build` for the index this server owns.

use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::mail::{HttpMail, MailSender};
use server_accounts::{AccountStore, BuiltInAuth, Door, MailPost, PgStore, SystemClock};
use server_api::{AppState, Config};
use server_catalog::{DataSlot, DataTrust};
use server_library::{Library, LibraryStore, PgLibrary};
use server_pages::Pages;
use server_play::{HttpFinder, Play};
use server_rules::Tier;
use server_rules::answer::Images;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

/// The keys every command checks a set with. With `DATA_PUBLIC_KEYS` unset there is no key, so
/// every set is refused and nothing reads one unsigned.
fn data_trust(config: &Config) -> Result<DataTrust> {
    DataTrust::from_key_files(&config.data_keys)
        .context("DATA_PUBLIC_KEYS names the PEM keys a data set is signed with")
}

fn open_data(config: &Config) -> Option<Arc<DataSlot>> {
    let manifest = config.data_manifest.as_deref()?;
    let trust = match data_trust(config) {
        Ok(trust) => trust,
        Err(error) => {
            warn!("Data signatures cannot be checked, titles stay off: {error:#}");
            return None;
        }
    };
    match DataSlot::open(manifest, trust) {
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
    let books: Arc<dyn LibraryStore> = Arc::new(PgLibrary::new(pool.clone()));
    let clock = Arc::new(SystemClock);
    let site = config.site_address.clone().unwrap_or_default();
    let post = MailPost {
        mail: open_mail(config)?,
        name: config.brand_name.clone(),
        help_email: config.support_email.clone(),
        help_url: config.help_address.clone().unwrap_or_else(|| site.clone()),
        site_url: site,
    };
    let door = Door::new(store, clock.clone(), DEFAULT_LIMITS)
        .with_auth(Arc::new(BuiltInAuth::new(pool, Arc::new(post))));
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

fn open_mail(config: &Config) -> Result<Option<Arc<dyn MailSender>>> {
    let (Some(address), Some(token), Some(sender)) = (
        &config.mail_address,
        &config.mail_token,
        &config.mail_sender,
    ) else {
        return Ok(None);
    };
    let mail = HttpMail::new(address, token, sender).map_err(|error| anyhow!("{error}"))?;
    Ok(Some(Arc::new(mail)))
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
    let mail = open_mail(&config)?;
    let mut state = AppState::new(config, door, data, library);
    state.play = play;
    state.mail = mail;
    let state = Arc::new(state);
    server_api::jobs::start(&state);
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

/// Print how every genre page stands for each tier: `check pages`.
fn check_pages(config: &Config) -> Result<()> {
    let manifest = config
        .data_manifest
        .as_deref()
        .context("DATA_MANIFEST is not set")?;
    let slot = DataSlot::open(manifest, data_trust(config)?)?;
    let set = slot.load();
    let pages = Pages::new(Images::new(None));
    let tiers = [
        Tier::Adult,
        Tier::Fifteen,
        Tier::Twelve,
        Tier::OlderKids,
        Tier::LittleKids,
    ];
    let mut out = std::io::stdout().lock();
    for tier in tiers {
        for line in pages.check_pages(&set, tier) {
            writeln!(
                out,
                "{} {} {} cap={} {}",
                tier.as_str(),
                line.surface,
                line.id,
                line.cap,
                if line.offered { "offered" } else { "too thin" }
            )?;
        }
    }
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
    let slot = DataSlot::open(manifest, data_trust(config)?)?;
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
        ["check", "pages"] => check_pages(&config),
        _ => bail!("use: server [serve | migrate | index build | check pages]"),
    }
}
