//! Open routes: `/v1/hello`, `/v1/ready`, strings.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use contract::hello::{BrandValues, Features, Hello, Limits, Ready, ServiceState, Strings};

use crate::answer::{Keep, send};
use crate::error::{ApiError, ApiResult};
use crate::guard::wire_range;
use crate::state::{AppState, Shared};

/// Seconds a client may keep hello.
const HELLO_TTL_S: u32 = 300;

fn limits(state: &AppState) -> Limits {
    let values = state
        .door
        .as_ref()
        .map_or(server_accounts::limits::DEFAULT_LIMITS, |door| {
            *door.limits()
        });
    Limits {
        profiles_max: values.profiles_max,
        profile_name_max: values.profile_name_max,
        pin_digits: values.pin_digits,
        password_min: values.password_min,
        step_up_s: values.step_up_s,
        phone_digits_min: values.phone_digits_min,
        phone_digits_max: values.phone_digits_max,
        blocked_title_max: values.blocked_name_max,
        blocked_titles_max: values.blocked_names_max,
        name_max: values.name_max,
        handle_min: values.handle_min,
        handle_max: values.handle_max,
        code_digits: values.code_digits,
        code_valid_s: values.code_valid_s,
        command_batch_max: values.command_batch_max,
    }
}

/// The first answer a client reads.
#[must_use]
pub fn hello_value(state: &AppState) -> Hello {
    let live = state.live.values();
    let visitor = (!state.config.accounts_required)
        .then_some(state.config.visitor_tier)
        .flatten();
    Hello {
        contract: wire_range(),
        client_min: live.client_min,
        features: Features {
            accounts_required: state.config.accounts_required,
            games: live.games,
            clips: live.clips,
            original_audio: live.original_audio,
            visitor_tier: visitor.map(|tier| tier.as_str().to_owned()),
        },
        limits: limits(state),
        playback: live.playback && state.config.playback_ready(),
        killed_builds: live.killed_builds,
        rollouts: live.rollouts,
        moved: live.moved,
        start_deadline_ms: live.start_deadline_ms,
        brand: BrandValues {
            name: state.config.brand_name.clone(),
            support_email: state.config.support_email.clone(),
        },
        ttl_s: HELLO_TTL_S,
    }
}

/// `GET /v1/hello`.
pub async fn hello(State(state): State<Shared>, headers: HeaderMap) -> ApiResult<Response> {
    send(&headers, &hello_value(&state), Keep::Public(HELLO_TTL_S))
}

/// The state of the index: ready, building (the exact scan serves), or no data at all.
fn index_state(state: &AppState) -> ServiceState {
    match &state.data {
        None => ServiceState::Down,
        Some(slot) if slot.load().index.load().is_some() => ServiceState::Ready,
        Some(_) => ServiceState::Degraded,
    }
}

/// `/v1/ready`: plain state words only, so an outsider reads no provider or meter.
#[must_use]
pub fn ready_value(state: &AppState) -> Ready {
    let data = state.data.as_ref().map(|slot| slot.load());
    let now = state.clock.now_ms();
    let (data_hash, data_age_s) = data.map_or((String::new(), 0), |set| {
        let age_ms = now.saturating_sub(set.manifest.built_at_ms);
        (set.data_hash.clone(), age_ms / 1000)
    });
    Ready {
        build: state.config.build.clone(),
        data_hash,
        data_age_s,
        index: index_state(state),
        finder: if state.config.playback_ready() {
            ServiceState::Ready
        } else {
            ServiceState::Down
        },
    }
}

/// `GET /v1/ready`. Always 200 while the process answers, so warming is told apart from broken.
pub async fn ready(State(state): State<Shared>, headers: HeaderMap) -> ApiResult<Response> {
    send(&headers, &ready_value(&state), Keep::Never)
}

/// True for a language code this route will open as a file.
fn plain_lang(lang: &str) -> bool {
    (2..=12).contains(&lang.len()) && lang.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// `GET /v1/strings/{lang}`: one file of text, with the brand filled in at render time.
pub async fn strings(
    State(state): State<Shared>,
    Path(lang): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let dir = state
        .config
        .strings_dir
        .as_ref()
        .ok_or_else(|| ApiError::off("strings"))?;
    if !plain_lang(&lang) {
        return Err(ApiError::not_found());
    }
    let bytes =
        std::fs::read(dir.join(format!("{lang}.json"))).map_err(|_| ApiError::not_found())?;
    let texts = serde_json::from_slice(&bytes).map_err(|_| ApiError::not_found())?;
    send(
        &headers,
        &Strings { lang, texts },
        Keep::Public(HELLO_TTL_S),
    )
}
