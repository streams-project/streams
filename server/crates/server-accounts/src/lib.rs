//! The door and the accounts: who is calling, on which device session, with which profile.

pub mod auth;
mod clock;
mod door;
mod error;
pub mod flows;
pub mod limits;
mod memory;
mod meter;
pub mod pin;
mod postgres;
pub mod profile;
mod session;
mod store;
mod token;

pub use auth::{AuthError, AuthService, HttpAuth, Identity, MemoryAuth};
pub use clock::{Clock, ManualClock, SystemClock};
pub use door::{Door, Member, ProfileScope, StepUpMethod};
pub use error::DoorError;
pub use flows::{
    AccountView, DevicePollState, DeviceStart, Origin, SignUpAsk, SignedIn, email_shape_ok,
};
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
pub use store::{AccountNames, AccountStore, DeviceCodeRow, StoreError, StoreResult};
pub use token::{TokenHash, hash_token, new_token};
