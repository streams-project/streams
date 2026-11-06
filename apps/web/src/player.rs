//! Calls into `player.js` by name, so all the JavaScript stays in that one small file.

use contract::driver::{DriverCommand, DriverEvent};
use js_sys::{Array, Function, Reflect};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;

use crate::host;
use crate::state::{Clock, Shared};

fn call(name: &str, arguments: &[JsValue]) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(player) = Reflect::get(&window, &"webPlayer".into()) else {
        return;
    };
    let Ok(function) = Reflect::get(&player, &name.into()) else {
        return;
    };
    let Some(function) = function.dyn_ref::<Function>() else {
        return;
    };
    let list = Array::new();
    for argument in arguments {
        list.push(argument);
    }
    let _ = function.apply(&player, &list);
}

/// Put the one video element into the watch page's stage.
pub fn attach(stage: &web_sys::HtmlElement, shared: Shared) {
    let report = Closure::<dyn FnMut(u32, String)>::new(move |generation: u32, text: String| {
        let Ok(event) = serde_json::from_str::<DriverEvent>(&text) else {
            return;
        };
        match &event {
            DriverEvent::Position {
                time_ms,
                duration_ms,
                ..
            } => shared.clock.set(Clock {
                time_ms: *time_ms,
                duration_ms: duration_ms.unwrap_or(0),
            }),
            DriverEvent::Tracks { audio, text } => {
                shared.audio.set(audio.clone());
                shared.text.set(text.clone());
            }
            _ => {}
        }
        host::driver(generation, event);
    });
    call("attach", &[stage.clone().into(), report.as_ref().clone()]);
    report.forget();
}

/// The core's command, as the browser does it.
pub fn run(command: DriverCommand) {
    match command {
        DriverCommand::Open {
            url,
            start_at_ms,
            generation,
            ..
        } => {
            pending_open(url, start_at_ms, generation);
        }
        DriverCommand::Play => call("play", &[]),
        DriverCommand::Pause => call("pause", &[]),
        DriverCommand::Seek { to_ms } => call("seek", &[(to_ms as f64).into()]),
        DriverCommand::Rate { percent } => call("rate", &[percent.into()]),
        DriverCommand::Volume { percent } => call("volume", &[percent.into()]),
        DriverCommand::SelectAudio { track } => call("audio", &[track.into()]),
        DriverCommand::SelectText { track } => {
            let track = track.map_or(JsValue::NULL, JsValue::from);
            call("text", &[track]);
        }
        DriverCommand::Close => call("close", &[]),
        _ => {}
    }
}

/// An open can arrive before the watch page has drawn. The player's element exists from the start
/// of the page, so nothing waits on the drawing.
fn pending_open(url: String, start_at_ms: u64, generation: u32) {
    call(
        "open",
        &[url.into(), (start_at_ms as f64).into(), generation.into()],
    );
}

/// True when the film had to start without sound.
pub fn muted() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    Reflect::get(&window, &"webPlayer".into())
        .and_then(|player| {
            let function: Function = Reflect::get(&player, &"muted".into())?.dyn_into()?;
            function.call0(&player)
        })
        .is_ok_and(|answer| answer.is_truthy())
}

/// Turn the sound on, from a press.
pub fn unmute() {
    call("unmute", &[]);
}

/// Toggle fullscreen.
pub fn fullscreen() {
    call("fullscreen", &[]);
}
