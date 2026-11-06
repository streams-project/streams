//! The layer around a core that only records what it is handed and answers what the test says.
//! Whole sessions run in kit/core; these rules are the layer's own, so each one is held here.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use contract::core_msg::{
    Action, BodyTarget, CoreIn, CoreInTag, CoreOut, CoreOutTag, Effect, EffectBody, Event,
    EventBody, HttpClass, HttpMethod, SessionState, SessionView, StoreOp, View,
};
use contract::version::WireDate;
use core_rules::Step;
use offline::{Config, Layer};
use serde_json::json;

const BOOT: &str = "boot-1";
const PAGE: &str = "/v1/screens/home";
const BODY: &str = r#"{"ttl_s":60}"#;

/// What the core was handed, and what it answers with on its next step.
#[derive(Default)]
struct Tape {
    seen: Vec<Event>,
    effects: Vec<Effect>,
    view: Option<View>,
}

/// A core that decides nothing.
#[derive(Clone, Default)]
struct Fake {
    tape: Rc<RefCell<Tape>>,
}

impl Step for Fake {
    fn step(&mut self, input: CoreIn) -> CoreOut {
        let mut tape = self.tape.borrow_mut();
        tape.seen.extend(input.events);
        CoreOut {
            contract: WireDate::current(),
            tag: CoreOutTag::default(),
            effects: std::mem::take(&mut tape.effects),
            view: tape.view.take(),
            next_wake_ms: None,
            unsupported: Vec::new(),
        }
    }
}

fn config() -> Config {
    Config {
        boot_id: BOOT.to_string(),
        storage_bytes: 100_000,
    }
}

fn step(layer: &mut Layer<Fake>, mono_ms: u64, events: Vec<Event>) -> CoreOut {
    layer.step(CoreIn {
        contract: WireDate::current(),
        tag: CoreInTag::default(),
        now_ms: 1_793_836_800_000 + mono_ms,
        mono_ms,
        events,
    })
}

fn store_done(id: &str, value: Option<&str>, keys: &[&str]) -> Event {
    Event {
        id: Some(id.to_string()),
        body: EventBody::StoreDone {
            value: value.map(str::to_string),
            keys: keys.iter().map(|key| (*key).to_string()).collect(),
        },
    }
}

fn http_done(id: &str, status: u32, headers: &[(&str, &str)], body: Option<&str>) -> Event {
    Event {
        id: Some(id.to_string()),
        body: EventBody::HttpDone {
            status,
            headers: headers
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
            body: body.map(str::to_string),
        },
    }
}

fn cacheable(id: &str, path: &str) -> Effect {
    Effect {
        id: id.to_string(),
        body: EffectBody::Http {
            method: HttpMethod::Get,
            path: path.to_string(),
            headers: BTreeMap::new(),
            body: None,
            deadline_ms: 60_000,
            class: HttpClass::Cacheable,
            to: BodyTarget::Core,
            scope: None,
        },
    }
}

/// The id of the store effect for this key.
fn store_effect(out: &CoreOut, key: &str) -> String {
    out.effects
        .iter()
        .find_map(|effect| match &effect.body {
            EffectBody::Store {
                key: Some(held), ..
            } if held == key => Some(effect.id.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no store effect for {key}"))
}

/// The id of the store read of a key under this prefix.
fn store_read(out: &CoreOut, prefix: &str) -> String {
    out.effects
        .iter()
        .find_map(|effect| match &effect.body {
            EffectBody::Store {
                op: StoreOp::Get,
                key: Some(key),
                ..
            } if key.starts_with(prefix) => Some(effect.id.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no store read under {prefix}"))
}

fn header(out: &CoreOut, id: &str, name: &str) -> Option<String> {
    out.effects
        .iter()
        .find(|effect| effect.id == id)
        .and_then(|effect| match &effect.body {
            EffectBody::Http { headers, .. } => headers.get(name).cloned(),
            _ => None,
        })
}

/// Every prepared request the layer sent through the core.
fn requests(tape: &Rc<RefCell<Tape>>) -> Vec<(HttpMethod, String, Option<String>)> {
    tape.borrow()
        .seen
        .iter()
        .filter_map(|event| match &event.body {
            EventBody::Action {
                action: Action::Request { method, path, body },
            } => Some((*method, path.clone(), body.clone())),
            _ => None,
        })
        .collect()
}

/// Every answer the layer handed back to the core.
fn answers(tape: &Rc<RefCell<Tape>>) -> Vec<(String, u32, Option<String>)> {
    tape.borrow()
        .seen
        .iter()
        .filter_map(|event| match &event.body {
            EventBody::HttpDone { status, body, .. } => {
                Some((event.id.clone().unwrap_or_default(), *status, body.clone()))
            }
            _ => None,
        })
        .collect()
}

fn signed_in(scope: &str) -> View {
    View {
        session: Some(SessionView {
            state: SessionState::SignedIn,
            user_id: Some("u1".to_string()),
            email: None,
            expires_at_ms: None,
            profile: Some("p1".to_string()),
            scope: Some(scope.to_string()),
            ended: None,
        }),
        ..View::default()
    }
}

/// One kept position command of this scope, as the store held it.
fn kept(scope: &str) -> String {
    json!({
        "seq": 0,
        "scope": scope,
        "op": {
            "op_id": "run-1",
            "age_ms": 0,
            "command": "position",
            "title": "movie:7",
            "position_s": 30,
            "duration_s": 600,
        },
        "mono_ms": 500,
        "boot": BOOT,
    })
    .to_string()
}

/// A layer that has read the store, with the kept queue and index it was given.
fn loaded(queue: &[&str], items: &[&str]) -> (Layer<Fake>, Rc<RefCell<Tape>>) {
    let fake = Fake::default();
    let tape = fake.tape.clone();
    let mut layer = Layer::new(fake, config());
    let out = step(&mut layer, 1_000, Vec::new());
    let list = store_effect(&out, "q/");
    let index = store_effect(&out, "index");
    let out = step(
        &mut layer,
        1_100,
        vec![
            store_done(&list, None, queue),
            store_done(&index, None, &[]),
        ],
    );
    for (key, item) in queue.iter().copied().zip(items.iter().copied()) {
        let id = store_effect(&out, key);
        step(&mut layer, 1_200, vec![store_done(&id, Some(item), &[])]);
    }
    (layer, tape)
}

#[test]
fn c2_c19_ops_kept_for_a_profile_go_out_as_soon_as_that_profile_is_in_use() {
    let item = kept("u1/p1");
    let (mut layer, tape) = loaded(&["q/0000000000000000"], &[item.as_str()]);
    assert!(
        requests(&tape).is_empty(),
        "with no profile in use nothing can go out"
    );
    // the profile that owns the kept op comes into use
    tape.borrow_mut().view = Some(signed_in("u1/p1"));
    step(&mut layer, 2_000, Vec::new());
    let sent = requests(&tape);
    let batch = sent
        .iter()
        .find(|(method, path, _)| *method == HttpMethod::Post && path == "/v1/commands");
    let Some((_, _, body)) = batch else {
        panic!("the kept op never went out: {sent:?}")
    };
    assert!(
        body.as_deref().unwrap_or_default().contains("run-1"),
        "the batch holds the kept op: {body:?}"
    );
}

#[test]
fn c100_c122_a_kept_page_past_its_ttl_is_read_again_with_its_etag() {
    let (mut layer, tape) = loaded(&[], &[]);
    // the core asks for a page that may be kept
    tape.borrow_mut().effects = vec![cacheable("e9", PAGE)];
    let out = step(&mut layer, 2_000, Vec::new());
    assert_eq!(
        header(&out, "e9", "If-None-Match"),
        None,
        "nothing is kept yet"
    );
    // the answer carries an etag and a one minute ttl
    let out = step(
        &mut layer,
        2_100,
        vec![http_done("e9", 200, &[("ETag", "\"v1\"")], Some(BODY))],
    );
    assert!(
        !store_effect(&out, "index").is_empty(),
        "the index is written, so the etag survives a restart"
    );
    // past the ttl the same page is asked for again
    tape.borrow_mut().effects = vec![cacheable("e10", PAGE)];
    let out = step(&mut layer, 62_101, Vec::new());
    assert_eq!(
        header(&out, "e10", "If-None-Match").as_deref(),
        Some("\"v1\""),
        "the server cannot answer 304 without being told what is held"
    );
    // a 304 serves the kept body, which only works because the etag went out
    let out = step(&mut layer, 62_102, vec![http_done("e10", 304, &[], None)]);
    let read = store_read(&out, "c/");
    step(&mut layer, 62_103, vec![store_done(&read, Some(BODY), &[])]);
    assert!(
        answers(&tape).iter().any(|(id, status, body)| id == "e10"
            && *status == 200
            && body.as_deref() == Some(BODY)),
        "the kept body answers the request: {:?}",
        answers(&tape)
    );
}
