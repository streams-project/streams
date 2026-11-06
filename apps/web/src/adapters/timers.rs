//! The timer port, on the browser's timeouts.

use contract::core_msg::{EventBody, TimerOp};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use crate::host::{answer, step, window, with_host};

pub fn perform(id: String, op: TimerOp, after_ms: Option<u64>) {
    if op == TimerOp::Clear {
        let old = with_host(|host| host.timers.remove(&id)).flatten();
        if let Some(old) = old {
            window().clear_timeout_with_handle(old);
        }
        return;
    }
    let key = id.clone();
    let fire = Closure::once_into_js(move || {
        with_host(|host| host.timers.remove(&key));
        step(vec![answer(key, EventBody::TimerFired)]);
    });
    let after = after_ms.unwrap_or(0).min(i32::MAX as u64) as i32;
    let handle = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(fire.unchecked_ref(), after)
        .ok();
    if let Some(handle) = handle {
        with_host(|host| host.timers.insert(id, handle));
    }
}
