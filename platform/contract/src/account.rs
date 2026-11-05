//! Sign in, devices, account and profiles. Tokens are opaque and the server alone reads them.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::client::{CapabilityCard, DeviceClass};

words! {
    /// Answers while a device waits for approval.
    pub enum DevicePoll {
        AuthorizationPending,
        SlowDown,
        AccessDenied,
        ExpiredToken,
    }
}

wire! {
    /// Answer with no data.
    pub struct Done {
        pub done: bool,
    }

    pub struct PasswordSignIn {
        pub email: String,
        pub password: String,
    }

    pub struct CodeSend {
        pub email: String,
    }

    /// A code was sent, or would have been. The answer never says whether the account exists.
    pub struct CodeSent {
        pub expires_in: u32,
    }

    pub struct CodeSignIn {
        pub email: String,
        pub code: String,
    }

    pub struct SignUp {
        pub email: String,
        pub password: String,
        pub first_name: String,
        pub last_name: String,
    }

    /// Opaque tokens. `expires_in` is seconds, so no device clock decides anything.
    pub struct SignedIn {
        pub user_id: String,
        pub email: String,
        pub access_token: String,
        pub refresh_token: String,
        pub expires_in: u32,
    }

    pub struct RefreshRequest {
        pub refresh_token: String,
    }

    pub struct SignOut {
        /// True ends every session of this account.
        pub everywhere: bool,
    }

    /// Step up. Send a password or an emailed code.
    pub struct Confirm {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub password: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub code: Option<String>,
    }

    /// A step up holds for this many seconds and a refresh never renews it.
    pub struct Confirmed {
        pub valid_s: u32,
    }

    pub struct RecoveryStart {
        pub email: String,
    }

    pub struct RecoveryFinish {
        pub email: String,
        pub code: String,
        pub new_password: String,
    }

    pub struct DeviceCodeRequest {
        pub device_name: String,
    }

    /// Device sign in. The screen shows `user_code` even when it shows a QR code.
    pub struct DeviceCode {
        pub device_code: String,
        pub user_code: String,
        pub verification_uri: String,
        pub verification_uri_complete: String,
        pub expires_in: u32,
        pub interval: u32,
    }

    pub struct DeviceTokenRequest {
        pub device_code: String,
    }

    /// Either tokens or a word saying to wait or stop.
    pub struct DeviceTokenAnswer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub signed_in: Option<SignedIn>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub poll: Option<DevicePoll>,
    }

    pub struct DeviceDecision {
        pub user_code: String,
    }

    /// Sent at each start. The card travels only when the server asks or the app or OS changed.
    pub struct DeviceCardRequest {
        pub device_key: String,
        pub card_hash: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub card: Option<CapabilityCard>,
    }

    /// True when the server wants the whole card.
    pub struct DeviceCardAnswer {
        pub send_card: bool,
    }

    /// One signed in device.
    pub struct DeviceSession {
        pub id: String,
        pub name: String,
        pub class: DeviceClass,
        pub last_seen_ms: u64,
        pub current: bool,
    }

    pub struct DeviceList {
        pub devices: Vec<DeviceSession>,
    }

    pub struct Account {
        pub id: String,
        pub email: String,
        pub first_name: String,
        pub last_name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub phone: Option<String>,
    }

    pub struct AccountUpdate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub first_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub last_name: Option<String>,
    }

    pub struct PasswordChange {
        pub new_password: String,
    }

    pub struct EmailChange {
        pub email: String,
    }

    pub struct PhoneChange {
        pub phone: String,
    }

    /// Consent for personal rows.
    pub struct Consent {
        pub personal: bool,
    }

    /// One profile.
    pub struct Profile {
        pub id: String,
        pub name: String,
        pub kids: bool,
        pub main: bool,
        pub has_pin: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub avatar: Option<String>,
    }

    pub struct ProfileList {
        pub profiles: Vec<Profile>,
    }

    pub struct ProfileCreate {
        pub name: String,
        #[serde(default)]
        pub kids: bool,
    }

    pub struct ProfileUpdate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub kids: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub avatar: Option<String>,
    }

    pub struct ProfileEnter {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub pin: Option<String>,
    }

    /// The profile answer names its home, so the core can map it to a target.
    pub struct ProfileEntered {
        pub profile: Profile,
        pub home: crate::screen::ScreenName,
    }

    /// Absent clears the PIN.
    pub struct PinSet {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub pin: Option<String>,
    }

    pub struct ProfileTransfer {
        pub to_account_email: String,
    }

    /// Needs a step up. Never reachable through a queued command.
    pub struct ProfileLimits {
        pub age_rating: String,
        pub blocked_titles: Vec<String>,
    }

    pub struct ProfileSettings {
        pub autoplay_next: bool,
        pub autoplay_previews: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub audio_language: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub subtitle_language: Option<String>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub subtitle_style: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub ui_language: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub handle: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub gender: Option<String>,
    }

    /// Everything held about an account, in plain JSON.
    pub struct AccountExport {
        pub account: Account,
        pub profiles: Vec<Profile>,
        pub devices: Vec<DeviceSession>,
    }
}
