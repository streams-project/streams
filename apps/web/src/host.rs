//! The host: it steps the core and performs each effect with what a browser has.

use std::cell::RefCell;
use std::collections::HashMap;

use contract::core_msg::{
    Action, CoreConfig, CoreIn, CoreOut, Effect, EffectBody, Event, EventBody, HttpMethod, View,
};
use contract::screen::ScreenName;
use contract::version::WIRE_VERSION;
use core_rules::{Core, Step};
use js_sys::{Date, Math};
use leptos::prelude::*;
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::{address, network, player, secrets, timers};
use crate::state::Shared;

/// A form's answer: the status, then the body.
type Reply = Box<dyn FnOnce(u32, String)>;

/// Activity counts at most this often, the same rate as the other three hosts.
const TOUCH_EVERY_MS: u64 = 10_000;

/// Where a viewport stops being a phone, and where it stops being a tablet.
const PHONE_WIDEST_PX: f64 = 600.0;
const TABLET_WIDEST_PX: f64 = 1024.0;

pub struct Host {
    machine: Core,
    shared: Shared,
    pub timers: HashMap<String, i32>,
    wake: Option<i32>,
    replies: HashMap<String, Reply>,
    next_ask: u64,
    last_touch_ms: Option<u64>,
}

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

/// The host, for an adapter that keeps its own part of it.
pub fn with_host<T>(work: impl FnOnce(&mut Host) -> T) -> Option<T> {
    HOST.with_borrow_mut(|host| host.as_mut().map(work))
}

pub fn window() -> web_sys::Window {
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

/// The form this browser draws in. The contract says a browser on a phone is class web, form
/// phone, and the core's idle limit before the profile gate asks again keys on the form.
fn form_of(width_px: f64) -> &'static str {
    if width_px <= PHONE_WIDEST_PX {
        "phone"
    } else if width_px <= TABLET_WIDEST_PX {
        "tablet"
    } else {
        "desktop"
    }
}

/// A width a browser will not give reads as a desktop, which is the shortest idle limit of the
/// three, so an unknown device is never left open for longer than a known one.
fn viewport_width() -> f64 {
    window()
        .inner_width()
        .ok()
        .and_then(|width| width.as_f64())
        .unwrap_or(f64::INFINITY)
}

/// Open the core once and start it at the address the page was opened on.
pub fn start(shared: Shared) {
    let config: CoreConfig = serde_json::from_value(json!({
        "contract": WIRE_VERSION, "type": "core_config", "class": "web",
        "form": form_of(viewport_width()),
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
            last_touch_ms: None,
        });
    });
    let back = Closure::<dyn FnMut()>::new(go_silently);
    window().set_onpopstate(Some(back.as_ref().unchecked_ref()));
    back.forget();
    listen();
    // Read first: the core can finish starting within this step and rewrite the bar.
    let opened_at = address::current();
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

fn go_silently() {
    step(vec![event(EventBody::Address {
        address: address::current(),
    })]);
}

/// What the page can tell the core about its own life. Without these the core never hears that the
/// viewer is still here, so the profile gate asks in the middle of a film, and it never hears the
/// network come back, so a waiting call sits out its whole backoff. The other three hosts send the
/// same three events.
fn listen() {
    let document = window().document().expect("a document");
    let shown = Closure::<dyn FnMut()>::new(|| {
        let document = window().document().expect("a document");
        step(vec![event(EventBody::Visibility {
            visible: !document.hidden(),
        })]);
    });
    let _ = document
        .add_event_listener_with_callback("visibilitychange", shown.as_ref().unchecked_ref());
    shown.forget();

    for (name, online) in [("online", true), ("offline", false)] {
        let moved = Closure::<dyn FnMut()>::new(move || {
            step(vec![event(EventBody::Network { online })]);
        });
        let _ = window().add_event_listener_with_callback(name, moved.as_ref().unchecked_ref());
        moved.forget();
    }

    for name in ["pointerdown", "keydown"] {
        let used = Closure::<dyn FnMut()>::new(touched);
        let _ = document.add_event_listener_with_callback(name, used.as_ref().unchecked_ref());
        used.forget();
    }
}

/// A press or a tap. The core only needs to hear this now and then.
fn touched() {
    let now = mono_ms();
    let due = HOST.with_borrow_mut(|host| match host.as_mut() {
        Some(host) => {
            let due = host
                .last_touch_ms
                .is_none_or(|last| now.saturating_sub(last) >= TOUCH_EVERY_MS);
            if due {
                host.last_touch_ms = Some(now);
            }
            due
        }
        None => false,
    });
    if due {
        step(vec![event(EventBody::Activity)]);
    }
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

pub fn answer(id: String, body: EventBody) -> Event {
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
        } => spawn_local(network::perform(
            id,
            method,
            path,
            headers,
            body,
            deadline_ms,
            to,
        )),
        EffectBody::Secret { op, name, value } => {
            results.push(answer(id, secrets::perform(op, &name, value)));
        }
        EffectBody::Timer { op, after_ms } => timers::perform(id, op, after_ms),
        EffectBody::Player { command } => player::run(command),
        EffectBody::View { notice, .. } => {
            if let (Some(notice), Some(shared)) = (notice, shared()) {
                shared.notice.set(Some(notice));
            }
        }
        EffectBody::Open { target } => address::write(&target),
        EffectBody::Log { code, detail, .. } => {
            let line = format!("{code} {}", detail.unwrap_or_default());
            web_sys::console::log_1(&line.into());
        }
        _ => results.push(answer(id, EventBody::Unsupported)),
    }
}

pub fn hand_reply(id: &str, status: u32, text: String) {
    let waiting =
        HOST.with_borrow_mut(|host| host.as_mut().and_then(|host| host.replies.remove(id)));
    if let Some(reply) = waiting {
        reply(status, text);
    }
}

pub fn keep_page(path: &str, text: &str) {
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

#[cfg(test)]
mod tests {
    use super::form_of;

    #[test]
    fn a_browser_on_a_phone_draws_in_a_phone_s_form() {
        assert_eq!(form_of(390.0), "phone");
        assert_eq!(form_of(600.0), "phone");
        assert_eq!(form_of(601.0), "tablet");
        assert_eq!(form_of(1024.0), "tablet");
        assert_eq!(form_of(1440.0), "desktop");
        // A width a browser would not give reads as the form with the shortest idle limit.
        assert_eq!(form_of(f64::INFINITY), "desktop");
    }
}
