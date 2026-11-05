//! What every request shares.

use std::sync::Arc;

use server_accounts::{Clock, Door, Meter, SystemClock};
use server_catalog::DataSlot;
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
    pub live: LiveFile,
    pub images: Images,
    pub calls: Meter,
    pub in_flight: Arc<Semaphore>,
    pub deadline_ms: u64,
}

impl AppState {
    /// State from settings, with the parts the settings opened.
    #[must_use]
    pub fn new(config: Config, door: Option<Door>, data: Option<Arc<DataSlot>>) -> Self {
        let live = LiveFile::new(config.live_settings.clone());
        let images = Images::new(config.image_base.as_deref());
        Self {
            config,
            clock: Arc::new(SystemClock),
            door,
            data,
            live,
            images,
            calls: Meter::new(CALLS_PER_MINUTE),
            in_flight: Arc::new(Semaphore::new(IN_FLIGHT_MAX)),
            deadline_ms: REQUEST_DEADLINE_MS,
        }
    }
}

/// The handle routes receive.
pub type Shared = Arc<AppState>;
