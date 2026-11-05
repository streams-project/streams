//! Every error answer: a stable code, a label key, the build, and retry time when there is one.

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use contract::error::{ErrorBody, ErrorCode, Notice};
use server_accounts::DoorError;
use server_rules::RuleError;

/// An answer that is not a success.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: ErrorCode,
    pub label: String,
    pub retry_after_s: Option<u32>,
    pub notice: Option<Notice>,
}

impl ApiError {
    /// An error with a label key.
    #[must_use]
    pub fn new(status: StatusCode, code: ErrorCode, label: &str) -> Self {
        Self {
            status,
            code,
            label: format!("error.{label}"),
            retry_after_s: None,
            notice: None,
        }
    }

    /// A bad request, with what was bad.
    #[must_use]
    pub fn bad(label: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, label)
    }

    /// Nothing here.
    #[must_use]
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, "not_found")
    }

    /// Held back by a rule. The same answer for a title that does not exist would leak nothing,
    /// but a client needs to tell refused from gone, so the code stays apart.
    #[must_use]
    pub fn refused(label: &str) -> Self {
        Self::new(StatusCode::FORBIDDEN, ErrorCode::Refused, label)
    }

    /// A feature this deployment left off.
    #[must_use]
    pub fn off(feature: &str) -> Self {
        let mut error = Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::Unsupported,
            &format!("off.{feature}"),
        );
        error.notice = Some(Notice {
            label: format!("notice.off.{feature}"),
            text: format!("This server has {feature} turned off."),
            action_label: None,
            action_address: None,
        });
        error
    }

    /// Too busy, or a part is down. Clients wait and keep their tokens.
    #[must_use]
    pub fn busy(code: ErrorCode, label: &str, retry_after_s: u32) -> Self {
        let mut error = Self::new(StatusCode::SERVICE_UNAVAILABLE, code, label);
        error.retry_after_s = Some(retry_after_s);
        error
    }

    /// A client below the floor, with a notice any old client can draw.
    #[must_use]
    pub fn upgrade(notice: Notice) -> Self {
        let mut error = Self::new(StatusCode::CONFLICT, ErrorCode::UpgradeRequired, "upgrade");
        error.notice = Some(notice);
        error
    }
}

impl From<DoorError> for ApiError {
    fn from(error: DoorError) -> Self {
        use ErrorCode as Code;
        use StatusCode as Status;
        match error {
            DoorError::Unauthorized => {
                Self::new(Status::UNAUTHORIZED, Code::Unauthorized, "sign_in_required")
            }
            DoorError::SessionEnded => {
                Self::new(Status::UNAUTHORIZED, Code::SessionEnded, "session_ended")
            }
            DoorError::ProfileRequired => {
                Self::new(Status::FORBIDDEN, Code::ProfileRequired, "profile_required")
            }
            DoorError::PinRequired => {
                Self::new(Status::FORBIDDEN, Code::PinRequired, "pin_required")
            }
            DoorError::PinWrong => Self::new(Status::FORBIDDEN, Code::PinRequired, "pin_wrong"),
            DoorError::PinLocked { retry_after_s } => {
                let mut error =
                    Self::new(Status::TOO_MANY_REQUESTS, Code::RateLimited, "pin_locked");
                error.retry_after_s = Some(retry_after_s);
                error
            }
            DoorError::StepUpRequired => {
                Self::new(Status::FORBIDDEN, Code::StepUpRequired, "step_up_required")
            }
            DoorError::Refused(rule) => Self::refused(rule),
            DoorError::BadValue(what) => Self::bad(what),
            DoorError::NotFound => Self::not_found(),
            DoorError::RateLimited { retry_after_s } => {
                let mut error =
                    Self::new(Status::TOO_MANY_REQUESTS, Code::RateLimited, "rate_limited");
                error.retry_after_s = Some(retry_after_s);
                error
            }
            DoorError::Unavailable { retry_after_s } => {
                Self::busy(Code::Overloaded, "try_again", retry_after_s)
            }
            _ => Self::new(Status::INTERNAL_SERVER_ERROR, Code::Internal, "internal"),
        }
    }
}

impl From<RuleError> for ApiError {
    /// A profile whose limit cannot be read gets nothing, and never a wider view.
    fn from(_: RuleError) -> Self {
        Self::busy(ErrorCode::Overloaded, "limit_unreadable", 5)
    }
}

/// Carried on a response until the outer layer, which knows the build, writes the body.
#[derive(Clone, Debug)]
pub(crate) struct Failure(pub ErrorBody);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            code: self.code,
            label: self.label,
            build: String::new(),
            retry_after_s: self.retry_after_s,
            notice: self.notice,
        };
        let mut response = self.status.into_response();
        if let Some(seconds) = self.retry_after_s {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(seconds));
        }
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response.extensions_mut().insert(Failure(body));
        response
    }
}

/// Write the body of a failure with the build that answered, so no error goes unnamed.
pub(crate) fn finish_failure(mut response: Response, build: &str) -> Response {
    let Some(Failure(mut body)) = response.extensions_mut().remove::<Failure>() else {
        return response;
    };
    body.build = build.to_owned();
    let (mut parts, _) = response.into_parts();
    let bytes = serde_json::to_vec(&body).unwrap_or_default();
    parts.headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Response::from_parts(parts, axum::body::Body::from(bytes))
}

/// A route result.
pub type ApiResult<T> = Result<T, ApiError>;
