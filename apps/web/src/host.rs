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

pub struct Host {
    machine: Core,
    shared: Shared,
    pub timers: HashMap<String, i32>,
    wake: Option<i32>,
    replies: HashMap<String, Reply>,
    next_ask: u64,
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
