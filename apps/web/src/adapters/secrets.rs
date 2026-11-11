//! The secrets port, on local storage.

use contract::core_msg::{EventBody, SecretOp};

use crate::host::window;

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

pub fn perform(op: SecretOp, name: &str, value: Option<String>) -> EventBody {
    let key = format!("secret:{name}");
    let Some(store) = storage() else {
        return EventBody::Unsupported;
    };
    let mut held = None;
    match op {
        SecretOp::Get => held = store.get_item(&key).ok().flatten(),
        SecretOp::Put => {
            // A refused write is not a write: a full or blocked store would otherwise sign the
            // viewer out at the next load, where the core keeps the tokens in memory instead.
            if store.set_item(&key, &value.unwrap_or_default()).is_err() {
                return EventBody::Unsupported;
            }
        }
        _ => {
            let _ = store.remove_item(&key);
        }
    }
    EventBody::SecretDone { value: held }
}
