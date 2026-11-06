//! What every request shares.

use std::sync::Arc;

use server_accounts::mail::MailSender;
use server_accounts::{Clock, Door, Meter, SystemClock};
use server_catalog::DataSlot;
use server_library::Library;
use server_pages::Pages;
use server_play::{Play, PlayLog, Subtitles};
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
        let live = LiveFile::new(config.live_settings.clone());
        let images = Images::new(config.image_base.as_deref());
        let brand = config.brand_name.clone();
        Self {
            config,
            clock: Arc::new(SystemClock),
            door,
            data,
            library,
            pages: Pages::new(images.clone()).branded(&brand),
            play: None,
            play_log: None,
            subtitles: None,
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
