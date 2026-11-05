//! The door and the accounts: who is calling, on which device session, with which profile.

mod clock;
mod door;
mod error;
pub mod limits;
mod memory;
mod meter;
pub mod pin;
mod postgres;
pub mod profile;
mod session;
mod store;
mod token;

pub use clock::{Clock, ManualClock, SystemClock};
pub use door::{Door, Member, ProfileScope, StepUpMethod};
pub use error::DoorError;
pub use limits::AccountLimits;
pub use memory::MemoryStore;
pub use meter::{Meter, MeterKey};
pub use pin::{PinCheck, PinOutcome, PinRow};
pub use postgres::PgStore;
pub use profile::{
    AccountId, NewAccount, Profile, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch,
    SessionId, main_profile_name,
};
pub use session::{Consent, DeviceSession, IssuedTokens, RefreshOutcome};
pub use store::{AccountStore, StoreError, StoreResult};
pub use token::{TokenHash, hash_token, new_token};
