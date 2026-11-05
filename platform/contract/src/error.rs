//! Error codes and notices. Every error names a stable code, a label key and the build.

use alloc::string::String;

words! {
    /// Stable error code. A code this build does not know reads as `Unknown`.
    pub enum ErrorCode {
        BadRequest,
        Unauthorized,
        SessionEnded,
        ProfileRequired,
        PinRequired,
        StepUpRequired,
        Refused,
        NotFound,
        NoLongerAvailable,
        Conflict,
        RateLimited,
        Overloaded,
        DeadlineExceeded,
        UpgradeRequired,
        Unsupported,
        NoSource,
        ClockWrong,
        Internal,
    }
}

wire! {
    /// Text a client can draw with no other knowledge, including a client below the floor.
    pub struct Notice {
        /// Label key, for clients with the strings.
        pub label: String,
        /// Plain text in the viewer's language.
        pub text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub action_label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub action_address: Option<String>,
    }

    /// Body of every error answer.
    pub struct ErrorBody {
        pub code: ErrorCode,
        /// Label key for this error.
        pub label: String,
        /// The build that answered.
        pub build: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub retry_after_s: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub notice: Option<Notice>,
    }
}
