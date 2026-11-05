//! The one router. Handlers live in this library so route tests run against the real thing.

use std::time::Duration;

use axum::Router;
use axum::http::{HeaderName, Method, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post, put};
use contract::error::ErrorCode;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::error::ApiError;
use crate::guard::guard;
use crate::routes::{account, auth, commands, devices, pages, profiles, start, titles};
use crate::state::Shared;

/// Bearer only and no cookies, so any origin may call, and `Origin: null` from a packaged TV app is
/// safe. Preflight is answered before routing.
fn cors() -> CorsLayer {
    let headers = [
        "authorization",
        "content-type",
        "if-none-match",
        "api-version",
        "client-app",
        "client-version",
        "client-class",
        "client-os",
    ]
    .map(HeaderName::from_static);
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(headers)
        .max_age(Duration::from_secs(3600))
}

/// A panic ends one request, never the process.
fn panicked(_: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let error = ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::Internal,
        "internal",
    );
    error.into_response()
}

async fn nothing_here() -> ApiError {
    ApiError::not_found()
}

/// Every route, with the layers every route shares.
pub fn router(state: Shared) -> Router {
    Router::new()
        .route("/v1/hello", get(start::hello))
        .route("/v1/ready", get(start::ready))
        .route("/v1/strings/{lang}", get(start::strings))
        .route("/v1/public/titles/{kind}/{id}", get(titles::public_title))
        .route("/v1/auth/password", post(auth::password))
        .route("/v1/auth/code", post(auth::send_code))
        .route("/v1/auth/code/verify", post(auth::verify_code))
        .route("/v1/auth/signup", post(auth::sign_up))
        .route("/v1/auth/refresh", post(auth::refresh))
        .route("/v1/auth/signout", post(devices::sign_out))
        .route("/v1/auth/confirm", post(auth::confirm))
        .route("/v1/auth/recovery/start", post(auth::recovery_start))
        .route("/v1/auth/recovery/finish", post(auth::recovery_finish))
        .route("/v1/auth/device/code", post(auth::device_code))
        .route("/v1/auth/device/token", post(auth::device_token))
        .route("/v1/auth/device/approve", post(auth::device_approve))
        .route("/v1/auth/device/deny", post(auth::device_deny))
        .route(
            "/v1/account",
            get(account::account)
                .patch(account::update)
                .delete(account::delete),
        )
        .route("/v1/account/export", get(account::export))
        .route("/v1/account/password", post(account::password))
        .route("/v1/account/email", post(account::email))
        .route("/v1/account/phone", post(account::phone))
        .route("/v1/devices/me", post(devices::register))
        .route("/v1/devices", get(devices::list).delete(devices::remove))
        .route("/v1/consent", put(devices::set_consent))
        .route("/v1/profiles", get(profiles::list).post(profiles::create))
        .route(
            "/v1/profiles/{id}",
            patch(profiles::update).delete(profiles::delete),
        )
        .route("/v1/profiles/{id}/enter", post(profiles::enter))
        .route("/v1/profiles/{id}/pin", put(profiles::set_pin))
        .route("/v1/profiles/{id}/main", post(profiles::set_main))
        .route("/v1/profiles/{id}/transfer", post(profiles::transfer))
        .route(
            "/v1/profiles/{id}/limits",
            get(profiles::limits).put(profiles::set_limits),
        )
        .route("/v1/profiles/me/settings", get(commands::settings))
        .route("/v1/titles/{kind}/{id}", get(titles::title))
        .route("/v1/nav", get(pages::nav))
        .route("/v1/screens/{name}", get(pages::screen))
        .route("/v1/more/{cursor}", get(pages::more))
        .route("/v1/search", get(pages::search))
        .route("/v1/search/suggest", get(pages::suggest))
        .route("/v1/commands", post(commands::commands))
        .route("/v1/changes", get(commands::changes))
        .fallback(nothing_here)
        .layer(CatchPanicLayer::custom(panicked))
        .layer(from_fn_with_state(state.clone(), guard))
        .layer(CompressionLayer::new())
        .layer(cors())
        .with_state(state)
}
