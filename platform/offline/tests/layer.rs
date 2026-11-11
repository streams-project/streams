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
use offline::queue::SESSION_MAX;
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

/// One command batch the core hands down, as the text of its ops.
fn command(id: &str, scope: &str, ops: &str) -> Effect {
    Effect {
        id: id.to_string(),
        body: EffectBody::Http {
            method: HttpMethod::Post,
            path: "/v1/commands".to_string(),
            headers: BTreeMap::new(),
            body: Some(format!("{{\"ops\":[{ops}]}}")),
            deadline_ms: 60_000,
            class: HttpClass::Command,
            to: BodyTarget::Core,
            scope: Some(scope.to_string()),
        },
    }
}

/// One position op of this run, as the core would write it in a batch.
fn position_op(n: u64) -> String {
    position_at(n, n, 30)
}

/// A position op of this run on a named title, at a named second.
fn position_at(n: u64, title: u64, seconds: u32) -> String {
    json!({
        "op_id": format!("op-{n}"),
        "age_ms": 0,
        "command": "position",
        "title": format!("movie:{title}"),
        "position_s": seconds,
        "duration_s": 600,
    })
    .to_string()
}

/// One list op of this run. Nothing re-sends a list change: there is no heartbeat for it.
fn list_op(n: u64) -> String {
    json!({
        "op_id": format!("op-{n}"),
        "age_ms": 0,
        "command": "list",
        "title": format!("movie:{n}"),
        "in_list": true,
    })
    .to_string()
}

/// The text of the queue item this step wrote to the store.
fn queue_put(out: &CoreOut) -> Option<String> {
    out.effects.iter().find_map(|effect| match &effect.body {
        EffectBody::Store {
            op: StoreOp::Put,
            key: Some(key),
            value: Some(value),
        } if key.starts_with("q/") => Some(value.clone()),
        _ => None,
    })
}

/// True when this step asked the store to delete this key.
fn deleted(out: &CoreOut, key: &str) -> bool {
    out.effects.iter().any(|effect| {
        matches!(
            &effect.body,
            EffectBody::Store { op: StoreOp::Delete, key: Some(held), .. } if held == key
        )
    })
}

/// The ops of the batch this layer sent, in order.
fn sent_ops(tape: &Rc<RefCell<Tape>>) -> String {
    requests(tape)
        .iter()
        .find(|(method, path, _)| *method == HttpMethod::Post && path == "/v1/commands")
        .map(|(_, _, body)| body.clone().unwrap_or_default())
        .unwrap_or_default()
}

/// Bytes one queue item takes in the store, measured from a layer with room to spare.
fn item_bytes(ops: &str) -> u64 {
    let (mut layer, tape) = with_cap(100_000);
    tape.borrow_mut().effects = vec![command("m1", "u1/p1", ops)];
    let out = step(&mut layer, 2_000, Vec::new());
    let text = queue_put(&out).expect("the op was written");
    text.len() as u64
}

/// True when this step wrote a queue item to the store.
fn wrote_queue_item(out: &CoreOut) -> bool {
    out.effects.iter().any(|effect| {
        matches!(
            &effect.body,
            EffectBody::Store { op: StoreOp::Put, key: Some(key), .. } if key.starts_with("q/")
        )
    })
}

/// The ids the layer handed back no answer for.
fn no_answer(tape: &Rc<RefCell<Tape>>) -> Vec<String> {
    tape.borrow()
        .seen
        .iter()
        .filter_map(|event| match &event.body {
            EventBody::HttpFailed { .. } => event.id.clone(),
            _ => None,
        })
        .collect()
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

/// A layer whose store holds this many bytes, with the first store reads already answered.
fn with_cap(bytes: u64) -> (Layer<Fake>, Rc<RefCell<Tape>>) {
    let fake = Fake::default();
    let tape = fake.tape.clone();
    let mut layer = Layer::new(
        fake,
        Config {
            boot_id: BOOT.to_string(),
            storage_bytes: bytes,
        },
    );
    let out = step(&mut layer, 1_000, Vec::new());
    let list = store_effect(&out, "q/");
    let index = store_effect(&out, "index");
    step(
        &mut layer,
        1_100,
        vec![store_done(&list, None, &[]), store_done(&index, None, &[])],
    );
    (layer, tape)
}

#[test]
fn c2_h20_an_op_with_no_room_in_the_store_is_refused_and_does_not_wait_in_the_queue() {
    let (mut layer, tape) = with_cap(600);
    let mut persisted: Vec<u64> = Vec::new();
    for n in 1..=8 {
        let id = format!("e{n}");
        tape.borrow_mut().effects = vec![command(&id, "u1/p1", &position_op(n))];
        let out = step(&mut layer, 2_000 + n, Vec::new());
        let written = wrote_queue_item(&out);
        let taken = answers(&tape)
            .iter()
            .any(|(answered, status, _)| *answered == id && *status == 202);
        assert_eq!(
            written,
            taken,
            "op-{n} is written and taken, or neither: {:?}",
            answers(&tape)
        );
        if written {
            persisted.push(n);
        } else {
            assert!(no_answer(&tape).contains(&id), "the refusal is reported");
        }
    }
    assert!(!persisted.is_empty(), "the cap fits some ops");
    assert!(persisted.len() < 8, "a full store refuses the rest");

    // What goes out is exactly what the store holds, so no unpersisted op sits in the queue
    // counting as nothing against the cap.
    tape.borrow_mut().view = Some(signed_in("u1/p1"));
    step(&mut layer, 9_000, Vec::new());
    let sent = requests(&tape);
    let (_, _, body) = sent
        .iter()
        .find(|(method, path, _)| *method == HttpMethod::Post && path == "/v1/commands")
        .expect("the kept ops go out");
    let batch = body.clone().unwrap_or_default();
    for n in 1..=8 {
        assert_eq!(
            batch.contains(&format!("\"op-{n}\"")),
            persisted.contains(&n),
            "op-{n} goes out only if it was kept: {batch}"
        );
    }
}

#[test]
fn c2_h20_a_refused_position_leaves_the_one_it_would_have_replaced_in_the_store() {
    // The store holds exactly one op. A newer position of the same title is two bytes longer, so
    // it is refused even after the older one's bytes are credited back. Nothing may be deleted:
    // the op that is still in the queue is the only copy of that position anywhere.
    let first = position_at(1, 7, 30);
    let (mut layer, tape) = with_cap(item_bytes(&first));
    tape.borrow_mut().effects = vec![command("e1", "u1/p1", &first)];
    let out = step(&mut layer, 2_000, Vec::new());
    let key = String::from("q/0000000000000000");
    assert!(queue_put(&out).is_some(), "the first position is written");

    tape.borrow_mut().effects = vec![command("e2", "u1/p1", &position_at(2, 7, 3_000))];
    let out = step(&mut layer, 2_100, Vec::new());
    assert!(
        queue_put(&out).is_none(),
        "there is no room for the newer one"
    );
    assert!(
        !deleted(&out, &key),
        "the position still in the queue keeps its store key: {:?}",
        out.effects
    );
    assert!(
        no_answer(&tape).contains(&String::from("e2")),
        "the refusal is reported"
    );

    tape.borrow_mut().view = Some(signed_in("u1/p1"));
    step(&mut layer, 9_000, Vec::new());
    let batch = sent_ops(&tape);
    assert!(
        batch.contains("\"op-1\""),
        "the kept position goes out: {batch}"
    );
    assert!(
        !batch.contains("\"op-2\""),
        "the refused one does not: {batch}"
    );
}

#[test]
fn c2_h20_one_refused_op_fails_the_answer_for_the_whole_batch() {
    // One answer per batch, so a batch where one op is refused is answered as taken by none.
    // The ops that did fit stay queued and go out, which is what this pins: the core logs every
    // op of this batch as dropped while the first is in fact kept and will be applied.
    let (mut layer, tape) = with_cap(item_bytes(&position_op(1)));
    let two = format!("{},{}", position_op(1), position_op(2));
    tape.borrow_mut().effects = vec![command("e1", "u1/p1", &two)];
    let out = step(&mut layer, 2_000, Vec::new());
    assert!(queue_put(&out).is_some(), "the first op fits");
    assert!(
        answers(&tape)
            .iter()
            .all(|(id, status, _)| !(id == "e1" && *status == 202)),
        "a batch with a refused op is not answered as taken: {:?}",
        answers(&tape)
    );
    assert!(no_answer(&tape).contains(&String::from("e1")));

    tape.borrow_mut().view = Some(signed_in("u1/p1"));
    step(&mut layer, 9_000, Vec::new());
    let batch = sent_ops(&tape);
    assert!(
        batch.contains("\"op-1\""),
        "the op that fit goes out: {batch}"
    );
    assert!(
        !batch.contains("\"op-2\""),
        "the refused one does not: {batch}"
    );
}

#[test]
fn c2_h20_a_command_nothing_resends_is_held_for_the_session_when_the_store_is_full() {
    // A full store must not lose what a device with no layer at all would have delivered. A
    // position is refused, because the next heartbeat carries a newer one; a list change is sent
    // once and nothing re-sends it, so it is held for this session, written nowhere, counted
    // nowhere, and gone on the next start.
    let (mut layer, tape) = with_cap(item_bytes(&position_op(1)));
    tape.borrow_mut().effects = vec![command("e1", "u1/p1", &position_op(1))];
    step(&mut layer, 2_000, Vec::new());

    tape.borrow_mut().effects = vec![command("e2", "u1/p1", &list_op(2))];
    let out = step(&mut layer, 2_100, Vec::new());
    assert!(
        queue_put(&out).is_none(),
        "there is no room to write it: {:?}",
        out.effects
    );
    assert!(
        answers(&tape)
            .iter()
            .any(|(id, status, _)| id == "e2" && *status == 202),
        "it is taken, because it will go out: {:?}",
        answers(&tape)
    );
    assert!(!no_answer(&tape).contains(&String::from("e2")));

    tape.borrow_mut().view = Some(signed_in("u1/p1"));
    step(&mut layer, 9_000, Vec::new());
    let batch = sent_ops(&tape);
    assert!(
        batch.contains("\"op-2\""),
        "the list change goes out: {batch}"
    );
}

#[test]
fn c2_h20_ops_held_for_the_session_are_capped_so_a_full_store_is_not_unbounded_memory() {
    let (mut layer, tape) = with_cap(item_bytes(&position_op(1)));
    tape.borrow_mut().effects = vec![command("e0", "u1/p1", &position_op(1))];
    step(&mut layer, 2_000, Vec::new());
    for n in 1..=SESSION_MAX as u64 {
        let id = format!("e{n}");
        tape.borrow_mut().effects = vec![command(&id, "u1/p1", &list_op(100 + n))];
        step(&mut layer, 3_000 + n, Vec::new());
        assert!(!no_answer(&tape).contains(&id), "op {n} is held");
    }
    let last = String::from("e-over");
    tape.borrow_mut().effects = vec![command(&last, "u1/p1", &list_op(999))];
    step(&mut layer, 4_000, Vec::new());
    assert!(
        no_answer(&tape).contains(&last),
        "past the cap the op is refused rather than held"
    );
}
