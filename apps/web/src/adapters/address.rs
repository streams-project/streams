//! The address port: the bar, the back button and what they mean to the core.

use contract::address;
use contract::core_msg::{RouteTarget, Target};

use crate::host::window;

pub fn current() -> String {
    let location = window().location();
    let path = location.pathname().unwrap_or_else(|_| "/".to_owned());
    let query = location.search().unwrap_or_default();
    format!("{path}{query}")
}

/// One step back, as the viewer's own back button would.
pub fn back() {
    if let Ok(history) = window().history() {
        let _ = history.back();
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
            Some(next) => format!("{}?next={}", address::SIGN_IN, in_address(next)),
            None => address::SIGN_IN.to_owned(),
        },
        Target::Activate => match &target.code {
            Some(code) => format!("/activate?code={}", in_address(code)),
            None => "/activate".to_owned(),
        },
        Target::Profiles => address::PROFILES.to_owned(),
        Target::Home => address::HOME.to_owned(),
        Target::KidsHome => "/kids".to_owned(),
        Target::Title => {
            let (kind, id) = pair?;
            format!("{}{kind}-{id}", address::TITLE)
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
        Target::Screen => address::of_screen(target.screen?, target.genre.as_deref()),
        _ => return None,
    };
    Some(address)
}

pub fn write(target: &RouteTarget) {
    let Some(address) = address_of(target) else {
        return;
    };
    if address == current() {
        return;
    }
    if let Ok(history) = window().history() {
        let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&address));
    }
}
