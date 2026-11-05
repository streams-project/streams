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
use crate::routes::{devices, profiles, start, titles};
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
        .route("/v1/auth/signout", post(devices::sign_out))
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
        .route(
            "/v1/profiles/{id}/limits",
            get(profiles::limits).put(profiles::set_limits),
        )
        .route("/v1/titles/{kind}/{id}", get(titles::title))
        .fallback(nothing_here)
        .layer(CatchPanicLayer::custom(panicked))
        .layer(from_fn_with_state(state.clone(), guard))
        .layer(CompressionLayer::new())
        .layer(cors())
        .with_state(state)
}
