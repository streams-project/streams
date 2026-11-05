//! What every request passes before a handler: shed, meter, version, floor, deadline. A failure in
//! any step is one error shape naming the build.

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use contract::error::{ErrorCode, Notice};
use contract::version::{
    API_VERSION_HEADER, VersionFit, VersionRange, WIRE_MIN, WIRE_VERSION, WireDate,
};
use server_accounts::MeterKey;

use crate::caller::{address_of, client_class};
use crate::error::{ApiError, ApiResult, finish_failure};
use crate::live::LiveValues;
use crate::state::Shared;

/// Routes any client may read at any version, so an old client can learn it is old. Open, and not
/// counted, as `/api/health` and `/api/features` were.
fn is_start_route(path: &str) -> bool {
    matches!(path, "/v1/hello" | "/v1/ready")
}

/// The wire dates this build answers.
#[must_use]
pub fn wire_range() -> VersionRange {
    VersionRange {
        min: WireDate::parse(WIRE_MIN).unwrap_or_default(),
        max: WireDate::parse(WIRE_VERSION).unwrap_or_default(),
    }
}

fn upgrade_notice() -> Notice {
    Notice {
        label: String::from("notice.upgrade"),
        text: String::from("This app needs an update."),
        action_label: None,
        action_address: None,
    }
}

/// Check the wire date. A range, never equality: older is refused, newer is served.
///
/// # Errors
/// A missing or unreadable date is a bad request, a date below the range needs an upgrade.
pub fn check_wire(headers: &HeaderMap) -> ApiResult<()> {
    let date = headers
        .get(API_VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::bad("api_version_required"))?;
    let date = WireDate::parse(date).map_err(|_| ApiError::bad("api_version_bad"))?;
    match wire_range().fit(&date) {
        VersionFit::TooOld => Err(ApiError::upgrade(upgrade_notice())),
        _ => Ok(()),
    }
}

/// Numbers of a dotted version, leading digits only, so `1.2.3+bad` reads as 1, 2, 3.
fn version_parts(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| {
            let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
            digits.parse().unwrap_or(0)
        })
        .collect()
}

/// True when `version` sorts below `floor`.
#[must_use]
pub fn below_floor(version: &str, floor: &str) -> bool {
    let (mut have, mut need) = (version_parts(version), version_parts(floor));
    let width = have.len().max(need.len());
    have.resize(width, 0);
    need.resize(width, 0);
    have < need
}

/// Check the client floor and the kill list. A client that states no version is left to the wire
/// check: the floor is a notice for real apps, and rules never rest on what a device states.
///
/// # Errors
/// `upgrade_required` with a notice any old client can draw.
pub fn check_floor(headers: &HeaderMap, live: &LiveValues) -> ApiResult<()> {
    let Some(version) = headers
        .get("client-version")
        .and_then(|value| value.to_str().ok())
    else {
        return Ok(());
    };
    let killed = live.killed_builds.iter().any(|build| build == version);
    let below = live
        .floor_of(client_class(headers))
        .is_some_and(|floor| below_floor(version, floor));
    if killed || below {
        return Err(ApiError::upgrade(upgrade_notice()));
    }
    Ok(())
}

fn address(state: &Shared, request: &Request) -> String {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip());
    address_of(state, request.headers(), peer)
}

fn enter(state: &Shared, request: &Request) -> ApiResult<()> {
    let path = request.uri().path();
    if is_start_route(path) {
        return Ok(());
    }
    state
        .calls
        .check(
            MeterKey::Address(address(state, request)),
            state.clock.now_ms(),
        )
        .map_err(|retry_after_s| {
            ApiError::from(server_accounts::DoorError::RateLimited { retry_after_s })
        })?;
    check_wire(request.headers())?;
    check_floor(request.headers(), &state.live.values())
}

/// The layer around every route.
pub async fn guard(State(state): State<Shared>, request: Request, next: Next) -> Response {
    let build = state.config.build.clone();
    let Ok(_slot) = state.in_flight.clone().try_acquire_owned() else {
        let shed = ApiError::busy(ErrorCode::Overloaded, "overloaded", 2);
        return finish_failure(shed.into_response(), &build);
    };
    if let Err(error) = enter(&state, &request) {
        return finish_failure(error.into_response(), &build);
    }
    let deadline = Duration::from_millis(state.deadline_ms);
    let response = match tokio::time::timeout(deadline, next.run(request)).await {
        Ok(response) => response,
        Err(_) => ApiError::busy(ErrorCode::DeadlineExceeded, "deadline", 2).into_response(),
    };
    finish_failure(response, &build)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h4_a_newer_client_is_served_and_an_older_one_is_told_to_upgrade() {
        let headers = |date: &str| {
            let mut map = HeaderMap::new();
            map.insert(API_VERSION_HEADER, date.parse().unwrap());
            map
        };
        assert!(check_wire(&headers(WIRE_VERSION)).is_ok());
        assert!(check_wire(&headers("2999-01-01")).is_ok());
        let old = check_wire(&headers("2000-01-01")).unwrap_err();
        assert_eq!(old.code, ErrorCode::UpgradeRequired);
        assert_eq!(old.status, 409, "S2 keeps 409");
        assert!(
            old.notice.is_some(),
            "a client below the floor needs text it can draw"
        );
        assert_eq!(
            check_wire(&headers("soon")).unwrap_err().code,
            ErrorCode::BadRequest
        );
        assert_eq!(
            check_wire(&HeaderMap::new()).unwrap_err().code,
            ErrorCode::BadRequest
        );
    }

    #[test]
    fn versions_compare_by_number_not_as_text() {
        assert!(below_floor("1.9.0", "1.10.0"));
        assert!(!below_floor("1.10.0", "1.9.0"));
        assert!(!below_floor("1.2", "1.2.0"));
        assert!(below_floor("1.2.3+bad", "1.2.4"));
    }

    #[test]
    fn c122_a_killed_build_and_a_class_floor_both_refuse() {
        use contract::client::DeviceClass;
        use contract::hello::ClientFloor;
        let live = LiveValues {
            client_min: vec![ClientFloor {
                class: DeviceClass::Tv,
                version: String::from("2.0.0"),
            }],
            killed_builds: vec![String::from("1.0.1+bad")],
            ..LiveValues::default()
        };
        let headers = |class: &str, version: &str| {
            let mut map = HeaderMap::new();
            map.insert("client-class", class.parse().unwrap());
            map.insert("client-version", version.parse().unwrap());
            map
        };
        assert!(check_floor(&headers("tv", "1.9.0"), &live).is_err());
        assert!(check_floor(&headers("tv", "2.0.0"), &live).is_ok());
        assert!(check_floor(&headers("web", "1.9.0"), &live).is_ok());
        assert!(check_floor(&headers("web", "1.0.1+bad"), &live).is_err());
    }
}
