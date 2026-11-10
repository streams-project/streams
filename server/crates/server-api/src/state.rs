//! What every request shares.

use std::sync::Arc;

use server_accounts::mail::MailSender;
use server_accounts::{Clock, Door, Meter, SystemClock};
use server_catalog::DataSlot;
use server_help::Help;
use server_library::Library;
use server_pages::Pages;
use server_play::{HttpSubtitles, Play, PlayLog, Subtitles};
use server_rules::answer::Images;
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::live::LiveFile;

/// Calls per minute from one address, before any token lookup.
const CALLS_PER_MINUTE: u32 = 600;
/// Requests in flight before new ones are shed with Retry-After.
const IN_FLIGHT_MAX: usize = 256;
/// Longest time one request may take.
pub const REQUEST_DEADLINE_MS: u64 = 15_000;

/// The state behind the router.
pub struct AppState {
    pub config: Config,
    pub clock: Arc<dyn Clock>,
    /// Off with no database.
    pub door: Option<Door>,
    /// Off with no manifest.
    pub data: Option<Arc<DataSlot>>,
    /// Off with no database.
    pub library: Option<Library>,
    pub pages: Pages,
    /// Off with no help folder.
    pub help: Option<Help>,
    /// Off with no finder.
    pub play: Option<Play>,
    /// Off with no database.
    pub play_log: Option<Arc<dyn PlayLog>>,
    /// Off with no subtitle source.
    pub subtitles: Option<Subtitles>,
    /// Off with no mail provider.
    pub mail: Option<Arc<dyn MailSender>>,
    pub live: LiveFile,
    pub images: Images,
    pub calls: Meter,
    pub in_flight: Arc<Semaphore>,
    /// Page work running at once. The rest wait, so page work cannot take every thread.
    pub page_work: Arc<Semaphore>,
    pub deadline_ms: u64,
}

impl AppState {
    /// State from settings, with the parts the settings opened.
    #[must_use]
    pub fn new(
        config: Config,
        door: Option<Door>,
        data: Option<Arc<DataSlot>>,
        library: Option<Library>,
    ) -> Self {
        let subtitles = subtitle_url_and_token(&config)
            .and_then(|(url, token)| HttpSubtitles::new(url, token).ok())
            .map(|source| Subtitles::new(Arc::new(source)));
        let live = LiveFile::new(config.live_settings.clone());
        let mut sizes = server_rules::answer::ImageSizes::default();
        if let Some(s) = &config.image_size_poster {
            sizes.poster = s.clone();
        }
        if let Some(s) = &config.image_size_backdrop {
            sizes.backdrop = s.clone();
        }
        if let Some(s) = &config.image_size_still {
            sizes.still = s.clone();
        }
        if let Some(s) = &config.image_size_logo {
            sizes.logo = s.clone();
        }
        if let Some(s) = &config.image_size_avatar {
            sizes.avatar = s.clone();
        }
        let images = Images::with_options(
            config.image_base.as_deref(),
            config.image_base_art.as_deref(),
            sizes,
        );
        let brand = config.brand_name.clone();
        let help = config
            .help_dir
            .as_deref()
            .and_then(|dir| match Help::open(dir) {
                Ok(help) => Some(help),
                Err(error) => {
                    tracing::warn!("help is off: {error}");
                    None
                }
            });
        // Every language's words for the pages, from the same folder `/v1/strings` reads.
        let books = Arc::new(config.strings_dir.as_deref().map_or_else(
            server_pages::words::Books::default,
            server_pages::words::Books::read,
        ));
        Self {
            help,
            config,
            clock: Arc::new(SystemClock),
            door,
            data,
            library,
            pages: Pages::new(images.clone())
                .speaking_from(books)
                .branded(&brand),
            play: None,
            play_log: None,
            subtitles,
            mail: None,
            live,
            images,
            calls: Meter::new(CALLS_PER_MINUTE),
            in_flight: Arc::new(Semaphore::new(IN_FLIGHT_MAX)),
            page_work: Arc::new(Semaphore::new(page_workers())),
            deadline_ms: REQUEST_DEADLINE_MS,
        }
    }
}

/// Page builds at once: the machine's threads, less one for everything else.
fn page_workers() -> usize {
    std::thread::available_parallelism().map_or(2, |count| count.get().saturating_sub(1).max(1))
}

/// The handle routes receive.
pub type Shared = Arc<AppState>;

fn subtitle_url_and_token(config: &Config) -> Option<(&str, &str)> {
    Some((
        config.subtitle_url.as_deref()?,
        config.finder_token.as_deref()?,
    ))
}
