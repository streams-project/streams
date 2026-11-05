//! Account limits, served from one place through `/v1/hello` so no client holds a copy.

/// Every limit a client may need to know, plus the windows the door applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccountLimits {
    pub profiles_max: u32,
    pub profile_name_max: u32,
    pub pin_digits: u32,
    pub password_min: u32,
    pub step_up_s: u32,
    pub phone_digits_min: u32,
    pub phone_digits_max: u32,
    pub blocked_name_max: u32,
    pub blocked_names_max: u32,
    pub name_max: u32,
    pub handle_min: u32,
    pub handle_max: u32,
    pub code_digits: u32,
    pub code_valid_s: u32,
    pub command_batch_max: u32,
    /// Opaque access token lifetime.
    pub access_s: u32,
    /// A session with no use for this long ends.
    pub idle_s: u32,
    /// How long a refresh token stays reusable after its first use, for a lost reply.
    pub refresh_reuse_s: u32,
}

/// The values carried over from the old account rules.
pub const DEFAULT_LIMITS: AccountLimits = AccountLimits {
    profiles_max: 5,
    profile_name_max: 30,
    pin_digits: 4,
    password_min: 8,
    step_up_s: 300,
    phone_digits_min: 6,
    phone_digits_max: 15,
    blocked_name_max: 100,
    blocked_names_max: 200,
    name_max: 60,
    handle_min: 3,
    handle_max: 16,
    code_digits: 8,
    code_valid_s: 600,
    command_batch_max: 400,
    access_s: 900,
    idle_s: 30 * 86_400,
    // Target value. A device that sleeps through a reply for longer signs in again; a thief who
    // keeps in step with the real device has to do so inside this window (threat T1).
    refresh_reuse_s: 300,
};

impl Default for AccountLimits {
    fn default() -> Self {
        DEFAULT_LIMITS
    }
}
