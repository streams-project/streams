//! The host: it steps the core and performs each effect with what a browser has.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use contract::core_msg::{
    Action, BodyTarget, CoreConfig, CoreIn, CoreOut, Effect, EffectBody, Event, EventBody,
    HttpMethod, NoAnswerReason, RouteTarget, SecretOp, Target, TimerOp, View,
};
use contract::screen::ScreenName;
use contract::version::WIRE_VERSION;
use core_rules::{Core, Step};
use js_sys::{Date, Math};
use leptos::prelude::*;
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{AbortController, RequestInit, Response};

use crate::player;
use crate::state::Shared;

/// A form's answer: the status, then the body.
type Reply = Box<dyn FnOnce(u32, String)>;

struct Host {
    machine: Core,
    shared: Shared,
    timers: HashMap<String, i32>,
    wake: Option<i32>,
    replies: HashMap<String, Reply>,
    next_ask: u64,
}

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

fn window() -> web_sys::Window {
    web_sys::window().expect("a page runs in a window")
}

fn mono_ms() -> u64 {
    window().performance().map_or(0, |clock| clock.now() as u64)
}

fn seed() -> String {
    (0..32)
        .map(|_| {
            let digit = (Math::random() * 16.0) as u32;
            char::from_digit(digit.min(15), 16).unwrap_or('0')
        })
        .collect()
}

/// Open the core once and start it at the address the page was opened on.
pub fn start(shared: Shared) {
    let config: CoreConfig = serde_json::from_value(json!({
        "contract": WIRE_VERSION, "type": "core_config", "class": "web", "form": "desktop",
        "mode": "local", "app": "web", "version": env!("CARGO_PKG_VERSION"), "os": "browser",
        "seed": seed(),
    }))
    .expect("the config is well formed");
    let machine = Core::open(config).expect("the seed is 32 hex digits");
    HOST.with_borrow_mut(|host| {
        *host = Some(Host {
            machine,
            shared,
            timers: HashMap::new(),
            wake: None,
            replies: HashMap::new(),
            next_ask: 0,
        });
    });
    let back = Closure::<dyn FnMut()>::new(go_silently);
    window().set_onpopstate(Some(back.as_ref().unchecked_ref()));
    back.forget();
    // Read first: the core can finish starting within this step and rewrite the bar.
    let opened_at = current_address();
    step(Vec::new());
    go(&opened_at);
    ask(
        HttpMethod::Get,
        "/v1/strings/en",
        None,
        move |status, text| {
            if !(200..300).contains(&status) {
                return;
            }
            if let Ok(strings) = serde_json::from_str::<contract::hello::Strings>(&text) {
                shared.strings.set(strings.texts);
            }
        },
    );
}

fn current_address() -> String {
    let location = window().location();
    let path = location.pathname().unwrap_or_else(|_| "/".to_owned());
    let query = location.search().unwrap_or_default();
    format!("{path}{query}")
}

fn go_silently() {
    step(vec![event(EventBody::Address {
        address: current_address(),
    })]);
}

/// Ask the core to open an address.
pub fn go(address: &str) {
    step(vec![event(EventBody::Address {
        address: address.to_owned(),
    })]);
}

/// Hand the core something the viewer asked for.
pub fn act(action: Action) {
    step(vec![event(EventBody::Action { action })]);
}

/// Send a prepared request, and hand its answer to `reply`, with status 0 when no answer came.
/// A 401 on the session's token is the core's to retry, so `reply` hears how that ended. If the
/// session ended instead it hears nothing, because the core has moved to sign in.
pub fn ask(
    method: HttpMethod,
    path: &str,
    body: Option<String>,
    reply: impl FnOnce(u32, String) + 'static,
) {
    let id = HOST.with_borrow_mut(|host| {
        let host = host.as_mut().expect("the host started");
        host.next_ask += 1;
        let id = format!("ask-{}", host.next_ask);
        host.replies.insert(id.clone(), Box::new(reply));
        id
    });
    step(vec![Event {
        id: Some(id),
        body: EventBody::Action {
            action: Action::Request {
                method,
                path: path.to_owned(),
                body,
            },
        },
    }]);
}

/// Hand the core something the driver saw.
pub fn driver(generation: u32, event_body: contract::driver::DriverEvent) {
    step(vec![event(EventBody::Driver {
        generation,
        event: event_body,
    })]);
}

fn event(body: EventBody) -> Event {
    Event { id: None, body }
}

fn answer(id: String, body: EventBody) -> Event {
    Event { id: Some(id), body }
}

/// Run the core until nothing more can be answered at once.
pub fn step(mut events: Vec<Event>) {
    for _ in 0..64 {
        let input = CoreIn {
            contract: Default::default(),
            tag: Default::default(),
            now_ms: Date::now() as u64,
            mono_ms: mono_ms(),
            events,
        };
        let out = HOST.with_borrow_mut(|host| host.as_mut().map(|host| host.machine.step(input)));
        let Some(out) = out else { return };
        events = apply(out);
        if events.is_empty() {
            return;
        }
    }
}

fn shared() -> Option<Shared> {
    HOST.with_borrow(|host| host.as_ref().map(|host| host.shared))
}

fn apply(out: CoreOut) -> Vec<Event> {
    let CoreOut {
        effects,
        view,
        next_wake_ms,
        ..
    } = out;
    if let (Some(view), Some(shared)) = (view, shared()) {
        show(view, shared);
    }
    schedule_wake(next_wake_ms);
    let mut results = Vec::new();
    for effect in effects {
        perform(effect, &mut results);
    }
    results
}

fn show(view: View, shared: Shared) {
    if let Some(session) = view.session {
        shared.session.set(Some(session));
    }
    if let Some(route) = view.route {
        // Search starts empty, and the last page stays up elsewhere until the next one arrives.
        if route.screen == Some(ScreenName::Search) {
            shared.screen.set(None);
        }
        shared.route.set(Some(route));
    }
    if let Some(gate) = view.gate {
        shared.gate.set(gate.ask);
    }
    if let Some(play) = view.play {
        shared.play.set(Some(play));
    }
    if let Some(list) = view.profiles {
        shared.profiles.set(list.profiles);
    }
    if let Some(hello) = view.hello {
        if !hello.brand.name.is_empty() {
            let document = window().document().expect("a document");
            document.set_title(&hello.brand.name);
        }
        shared.hello.set(Some(hello));
    }
}

fn schedule_wake(next_wake_ms: Option<u64>) {
    let old = HOST.with_borrow_mut(|host| host.as_mut().and_then(|host| host.wake.take()));
    if let Some(old) = old {
        window().clear_timeout_with_handle(old);
    }
    let Some(at) = next_wake_ms else { return };
    let after = at.saturating_sub(mono_ms()).min(60_000) as i32;
    let tick = Closure::once_into_js(|| step(Vec::new()));
    let handle = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(tick.unchecked_ref(), after)
        .ok();
    HOST.with_borrow_mut(|host| {
        if let Some(host) = host.as_mut() {
            host.wake = handle;
        }
    });
}

fn perform(effect: Effect, results: &mut Vec<Event>) {
    let Effect { id, body } = effect;
    match body {
        EffectBody::Http {
            method,
            path,
            headers,
            body,
            deadline_ms,
            to,
            ..
        } => spawn_local(fetch_it(id, method, path, headers, body, deadline_ms, to)),
        EffectBody::Secret { op, name, value } => {
            results.push(answer(id, secret(op, &name, value)));
        }
        EffectBody::Timer { op, after_ms } => timer(id, op, after_ms),
        EffectBody::Player { command } => player::run(command),
        EffectBody::View { notice, .. } => {
            if let (Some(notice), Some(shared)) = (notice, shared()) {
                shared.notice.set(Some(notice));
            }
        }
        EffectBody::Open { target } => write_address(&target),
        EffectBody::Log { code, detail, .. } => {
            let line = format!("{code} {}", detail.unwrap_or_default());
            web_sys::console::log_1(&line.into());
        }
        _ => results.push(answer(id, EventBody::Unsupported)),
    }
}

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

fn secret(op: SecretOp, name: &str, value: Option<String>) -> EventBody {
    let key = format!("secret:{name}");
    let Some(store) = storage() else {
        return EventBody::Unsupported;
    };
    let mut held = None;
    match op {
        SecretOp::Get => held = store.get_item(&key).ok().flatten(),
        SecretOp::Put => {
            let _ = store.set_item(&key, &value.unwrap_or_default());
        }
        _ => {
            let _ = store.remove_item(&key);
        }
    }
    EventBody::SecretDone { value: held }
}

fn timer(id: String, op: TimerOp, after_ms: Option<u64>) {
    if op == TimerOp::Clear {
        let old =
            HOST.with_borrow_mut(|host| host.as_mut().and_then(|host| host.timers.remove(&id)));
        if let Some(old) = old {
            window().clear_timeout_with_handle(old);
        }
        return;
    }
    let key = id.clone();
    let fire = Closure::once_into_js(move || {
        HOST.with_borrow_mut(|host| {
            if let Some(host) = host.as_mut() {
                host.timers.remove(&key);
            }
        });
        step(vec![answer(key, EventBody::TimerFired)]);
    });
    let after = after_ms.unwrap_or(0).min(i32::MAX as u64) as i32;
    let handle = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(fire.unchecked_ref(), after)
        .ok();
    if let Some(handle) = handle {
        HOST.with_borrow_mut(|host| {
            if let Some(host) = host.as_mut() {
                host.timers.insert(id, handle);
            }
        });
    }
}

fn in_address(text: &str) -> String {
    js_sys::encode_uri_component(text).into()
}

/// The address a target lives at, so the bar and the back button stay true.
fn address_of(target: &RouteTarget) -> Option<String> {
    let title = target.title.as_ref();
    let pair = title.map(|title| (title.kind().word(), title.id()));
    let address = match target.target {
        Target::Start => "/".to_owned(),
        Target::SignIn => match &target.next {
            Some(next) => format!("/sign-in?next={}", in_address(next)),
            None => "/sign-in".to_owned(),
        },
        Target::Activate => match &target.code {
            Some(code) => format!("/activate?code={}", in_address(code)),
            None => "/activate".to_owned(),
        },
        Target::Profiles => "/profiles".to_owned(),
        Target::Home => "/home".to_owned(),
        Target::KidsHome => "/kids".to_owned(),
        Target::Title => {
            let (kind, id) = pair?;
            format!("/title/{kind}/{id}")
        }
        Target::Watch => {
            let (kind, id) = pair?;
            match &target.episode {
                Some(episode) => {
                    format!("/watch/{kind}/{id}/{}/{}", episode.season, episode.episode)
                }
                None => format!("/watch/{kind}/{id}"),
            }
        }
        Target::Screen => {
            let word = serde_json::to_value(target.screen?).ok()?;
            match &target.genre {
                Some(genre) => format!("/genre/{genre}"),
                None => format!("/{}", word.as_str()?),
            }
        }
        _ => return None,
    };
    Some(address)
}

fn write_address(target: &RouteTarget) {
    let Some(address) = address_of(target) else {
        return;
    };
    if address == current_address() {
        return;
    }
    if let Ok(history) = window().history() {
        let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&address));
    }
}

async fn fetch_it(
    id: String,
    method: HttpMethod,
    path: String,
    headers: BTreeMap<String, String>,
    body: Option<String>,
    deadline_ms: u32,
    to: BodyTarget,
) {
    let carried_token = headers.contains_key("Authorization");
    let result = send(method, &path, &headers, body, deadline_ms).await;
    let event = match result {
        Err(reason) => {
            hand_reply(&id, 0, String::new());
            EventBody::HttpFailed { reason }
        }
        Ok((status, etag, text)) => {
            let mut headers = BTreeMap::new();
            if let Some(etag) = etag {
                headers.insert("etag".to_owned(), etag);
            }
            let good = (200..300).contains(&status);
            let to_screen = to == BodyTarget::Screen && good;
            if to_screen {
                keep_page(&path, &text);
            }
            // The core answers this 401 by refreshing and sending the same id again, so the reply
            // waits for that answer. Sending again from a screen would be a second request.
            if status != 401 || !carried_token {
                hand_reply(&id, status, text.clone());
            }
            EventBody::HttpDone {
                status,
                headers,
                body: (!to_screen && !text.is_empty()).then_some(text),
            }
        }
    };
    step(vec![answer(id, event)]);
}

fn hand_reply(id: &str, status: u32, text: String) {
    let waiting =
        HOST.with_borrow_mut(|host| host.as_mut().and_then(|host| host.replies.remove(id)));
    if let Some(reply) = waiting {
        reply(status, text);
    }
}

fn keep_page(path: &str, text: &str) {
    let Some(shared) = shared() else { return };
    if path.starts_with("/v1/nav") {
        if let Ok(nav) = serde_json::from_str::<contract::screen::Nav>(text) {
            shared.nav.set(nav.items);
        }
    } else if path.starts_with("/v1/screens/") {
        shared.screen.set(serde_json::from_str(text).ok());
    } else if path.contains("/titles/") {
        shared.detail.set(serde_json::from_str(text).ok());
    }
}

fn method_word(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        _ => "GET",
    }
}

async fn send(
    method: HttpMethod,
    path: &str,
    headers: &BTreeMap<String, String>,
    body: Option<String>,
    deadline_ms: u32,
) -> Result<(u32, Option<String>, String), NoAnswerReason> {
    let abort = AbortController::new().map_err(|_| NoAnswerReason::Offline)?;
    let init = RequestInit::new();
    init.set_method(method_word(method));
    init.set_signal(Some(&abort.signal()));
    let list = web_sys::Headers::new().map_err(|_| NoAnswerReason::Offline)?;
    for (name, value) in headers {
        let _ = list.set(name, value);
    }
    if let Some(body) = body {
        let _ = list.set("content-type", "application/json");
        init.set_body(&body.into());
    }
    init.set_headers(&list);
    let stop = abort.clone();
    let timeout = Closure::once_into_js(move || stop.abort());
    let handle = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            timeout.unchecked_ref(),
            deadline_ms as i32,
        )
        .ok();
    let answer = JsFuture::from(window().fetch_with_str_and_init(path, &init)).await;
    if let Some(handle) = handle {
        window().clear_timeout_with_handle(handle);
    }
    let response: Response = answer
        .map_err(|_| NoAnswerReason::Offline)?
        .dyn_into()
        .map_err(|_| NoAnswerReason::Offline)?;
    let status = u32::from(response.status());
    let etag = response.headers().get("etag").ok().flatten();
    let reading = response.text().map_err(|_| NoAnswerReason::Offline)?;
    let text = JsFuture::from(reading)
        .await
        .map_err(|_| NoAnswerReason::Timeout)?
        .as_string()
        .unwrap_or_default();
    Ok((status, etag, text))
}
