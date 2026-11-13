//! The step boundary on the binary wire decides what the JSON boundary decides.
//!
//! Only built with `--features proto`. The comparison is byte equality of one
//! `contract.v1.CoreOut`: the JSON path's answer is read back into the core's own
//! out message and written to the schema, and the proto path's answer is written
//! by the core itself. A difference anywhere in effects, view, wake or unsupported
//! moves a byte.
#![cfg(feature = "proto")]

use contract::core_msg::CoreOut;
use contract::version::WIRE_VERSION;
use contract_proto::contract as p;
use contract_proto::prost::Message as _;
use platform_core::proto::{in_to_bytes, out_from_bytes, out_to_bytes, step_proto};
use platform_core::{BadMessage, Core, step_json};
use serde_json::{Value, json};

const CONFIG: &str = r#"{"app":"a","class":"web","contract":"2026-11-05","form":"desktop",
  "mode":"local","os":"o","seed":"00112233445566778899aabbccddeeff","type":"core_config",
  "version":"1.0.0"}"#;

fn core() -> Core {
    Core::open_json(CONFIG).unwrap()
}

fn now(mono_ms: u64) -> u64 {
    1_793_836_800_000_u64 + mono_ms
}

fn json_in(mono_ms: u64, events: &Value) -> String {
    json!({
        "contract": WIRE_VERSION,
        "type": "core_in",
        "now_ms": now(mono_ms),
        "mono_ms": mono_ms,
        "events": events,
    })
    .to_string()
}

/// One step on each path, from two cores built from the same seed, and the two
/// answers as the same bytes.
fn both(
    json_core: &mut Core,
    proto_core: &mut Core,
    mono_ms: u64,
    events: (&Value, Vec<p::Event>),
) -> Vec<u8> {
    let text = step_json(json_core, &json_in(mono_ms, events.0)).unwrap();
    let read: CoreOut = serde_json::from_str(&text).unwrap();
    let from_json = out_to_bytes(read).unwrap();
    let from_proto = step_proto(
        proto_core,
        &in_to_bytes(WIRE_VERSION, now(mono_ms), mono_ms, events.1),
    )
    .unwrap();
    assert_eq!(
        out_from_bytes(&from_json).unwrap(),
        out_from_bytes(&from_proto).unwrap(),
        "step {mono_ms}: the two paths answered differently"
    );
    assert_eq!(
        from_json, from_proto,
        "step {mono_ms}: same message, different bytes"
    );
    from_proto
}

fn event(id: &str, body: p::event::Body) -> p::Event {
    p::Event {
        id: Some(id.to_string()),
        body: Some(body),
    }
}

/// Signs in on both paths at once, and gives back the last answer's bytes.
fn member(json_core: &mut Core, proto_core: &mut Core) -> Vec<u8> {
    let first = both(json_core, proto_core, 0, (&json!([]), Vec::new()));
    let out = out_from_bytes(&first).unwrap();
    let kept = out.effects[1].id.clone().unwrap();
    both(
        json_core,
        proto_core,
        1,
        (
            &json!([{"id": kept, "type": "secret_done"}]),
            vec![event(
                &kept,
                p::event::Body::SecretDone(p::event::SecretDone { value: None }),
            )],
        ),
    );
    let action = json!({"action": "sign_in", "email": "a@b.example", "password": "x"});
    let out = both(
        json_core,
        proto_core,
        2,
        (
            &json!([{"id": "s1", "type": "action", "action": action}]),
            vec![event(
                "s1",
                p::event::Body::Action(p::event::ActionEvent {
                    action: Some(p::Action {
                        action: Some(p::action::Action::SignIn(p::action::SignIn {
                            email: Some("a@b.example".to_string()),
                            password: Some("x".to_string()),
                        })),
                    }),
                }),
            )],
        ),
    );
    let out = out_from_bytes(&out).unwrap();
    let sign_in = out
        .effects
        .iter()
        .find(|effect| match &effect.body {
            Some(p::effect::Body::Http(http)) => http.path.as_deref() == Some("/v1/auth/password"),
            _ => false,
        })
        .and_then(|effect| effect.id.clone())
        .expect("a password request");
    let tokens = json!({
        "user_id": "u1",
        "email": "a@b.example",
        "access_token": "at",
        "refresh_token": "rt",
        "expires_in": 3600,
    })
    .to_string();
    both(
        json_core,
        proto_core,
        3,
        (
            &json!([{"id": sign_in, "type": "http_done", "status": 200, "body": tokens}]),
            vec![event(
                &sign_in,
                p::event::Body::HttpDone(p::event::HttpDone {
                    status: Some(200),
                    headers: alloc_map(),
                    body: Some(tokens),
                }),
            )],
        ),
    )
}

fn alloc_map() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::new()
}

#[test]
fn the_two_paths_answer_the_same_bytes_from_the_first_step_to_a_signed_in_member() {
    let last = member(&mut core(), &mut core());
    let out = out_from_bytes(&last).unwrap();
    let session = out.view.and_then(|view| view.session).expect("a session");
    assert_eq!(session.state, Some(p::SessionState::SignedIn as i32));
    assert_eq!(session.email.as_deref(), Some("a@b.example"));
}

#[test]
fn the_first_answer_carries_the_four_effects_and_the_wait_route() {
    let out = out_from_bytes(&both(&mut core(), &mut core(), 0, (&json!([]), Vec::new()))).unwrap();
    assert_eq!(out.effects.len(), 4);
    let route = out.view.and_then(|view| view.route).expect("a route");
    assert_eq!(route.target, Some(p::Target::Wait as i32));
}

#[test]
fn an_event_arm_this_build_does_not_know_is_answered_unsupported() {
    let mut core = core();
    step_proto(&mut core, &in_to_bytes(WIRE_VERSION, now(0), 0, Vec::new())).unwrap();
    // Field 90 is in no arm of the oneof, so prost keeps nothing and the body
    // arrives as no arm at all - the wire's form of an unknown `type`.
    let mut unknown = p::Event {
        id: Some("x1".to_string()),
        body: None,
    }
    .encode_to_vec();
    unknown.extend_from_slice(&[0xd0, 0x05, 0x01]);
    let mut message = p::CoreIn {
        contract: Some(p::ContractDate {
            ymd: Some(20_261_105),
        }),
        now_ms: Some(now(1)),
        mono_ms: Some(1),
        events: Vec::new(),
    }
    .encode_to_vec();
    message.push(0x22);
    message.push(u8::try_from(unknown.len()).unwrap());
    message.extend_from_slice(&unknown);
    let out = out_from_bytes(&step_proto(&mut core, &message).unwrap()).unwrap();
    assert_eq!(out.unsupported, ["x1"]);
}

#[test]
fn a_title_id_the_domain_cannot_hold_is_refused_and_changes_nothing() {
    let mut core = core();
    step_proto(&mut core, &in_to_bytes(WIRE_VERSION, now(0), 0, Vec::new())).unwrap();
    let play = p::action::Play {
        title: Some(p::TitleKey {
            kind: p::TitleKind::Tv as i32,
            id: 9_999_999_999,
        }),
        episode: None,
    };
    let message = in_to_bytes(
        WIRE_VERSION,
        now(1),
        1,
        vec![event(
            "p1",
            p::event::Body::Action(p::event::ActionEvent {
                action: Some(p::Action {
                    action: Some(p::action::Action::Play(play)),
                }),
            }),
        )],
    );
    assert_eq!(step_proto(&mut core, &message), Err(BadMessage));
    // The step was refused whole, so the next one still answers.
    let after = out_from_bytes(
        &step_proto(&mut core, &in_to_bytes(WIRE_VERSION, now(2), 2, Vec::new())).unwrap(),
    )
    .unwrap();
    assert_eq!(
        after.contract,
        Some(p::ContractDate {
            ymd: Some(20_261_105)
        })
    );
}

#[test]
fn an_answer_with_no_status_is_refused_rather_than_given_one() {
    let mut core = core();
    let first = out_from_bytes(
        &step_proto(&mut core, &in_to_bytes(WIRE_VERSION, now(0), 0, Vec::new())).unwrap(),
    )
    .unwrap();
    let hello = first.effects[2].id.clone().unwrap();
    let message = in_to_bytes(
        WIRE_VERSION,
        now(1),
        1,
        vec![event(
            &hello,
            p::event::Body::HttpDone(p::event::HttpDone {
                status: None,
                headers: alloc_map(),
                body: Some("{}".to_string()),
            }),
        )],
    );
    assert_eq!(step_proto(&mut core, &message), Err(BadMessage));
}

#[test]
fn bytes_that_are_no_core_in_are_refused() {
    let mut core = core();
    assert_eq!(
        step_proto(&mut core, b"not a message at all"),
        Err(BadMessage)
    );
    // A CoreIn with no contract date, no now and no mono: every one is required.
    let empty = p::CoreIn::default().encode_to_vec();
    assert_eq!(step_proto(&mut core, &empty), Err(BadMessage));
}
