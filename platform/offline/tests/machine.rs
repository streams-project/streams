//! The text a host gives `core_open` picks the layer. The core reads none of it.

use contract::core_msg::{CoreIn, CoreInTag, EffectBody};
use contract::version::WIRE_VERSION;
use core_rules::{BadMessage, Core, Step};
use offline::Machine;
use serde_json::{Value, json};

fn config() -> Value {
    json!({
        "contract": "2026-11-05",
        "type": "core_config",
        "class": "web",
        "form": "desktop",
        "mode": "local",
        "app": "sample-web",
        "version": "1.0.0",
        "os": "sample-os",
        "seed": "00112233445566778899aabbccddeeff",
    })
}

fn first_step<S: Step>(machine: &mut S) -> Vec<&'static str> {
    let out = machine.step(CoreIn {
        contract: serde_json::from_value(json!(WIRE_VERSION)).unwrap(),
        tag: CoreInTag::CoreIn,
        now_ms: 1_000,
        mono_ms: 1_000,
        events: Vec::new(),
    });
    out.effects
        .iter()
        .map(|effect| match effect.body {
            EffectBody::Store { .. } => "store",
            EffectBody::Secret { .. } => "secret",
            EffectBody::Http { .. } => "http",
            _ => "other",
        })
        .collect()
}

#[test]
fn no_layer_key_runs_the_core_alone_and_uses_no_store() {
    let mut machine = Machine::open_json(&config().to_string()).unwrap();
    assert!(matches!(machine, Machine::Alone(_)));
    assert!(!first_step(&mut machine).contains(&"store"));
}

#[test]
fn a_layer_key_wraps_the_core_and_the_first_step_reads_the_store() {
    let mut start = config();
    start["layer"] = json!({"boot_id": "boot-1", "storage_bytes": 1_000});
    let mut machine = Machine::open_json(&start.to_string()).unwrap();
    assert!(matches!(machine, Machine::Layered(_)));
    assert!(first_step(&mut machine).contains(&"store"));
}

#[test]
fn the_core_opens_the_same_with_the_layer_key_present() {
    let plain = config();
    let mut start = plain.clone();
    start["layer"] = json!({"boot_id": "boot-1", "storage_bytes": 1_000});
    let mut alone = Core::open_json(&plain.to_string()).unwrap();
    let mut with_key = Core::open_json(&start.to_string()).unwrap();
    assert_eq!(first_step(&mut alone), first_step(&mut with_key));
}

#[test]
fn a_layer_key_that_does_not_read_refuses_the_open() {
    let mut start = config();
    start["layer"] = json!({"boot_id": 7});
    assert!(matches!(
        Machine::open_json(&start.to_string()),
        Err(BadMessage)
    ));
}

#[test]
fn a_config_the_core_refuses_is_refused_here() {
    let mut start = config();
    start["seed"] = json!("short");
    assert!(matches!(
        Machine::open_json(&start.to_string()),
        Err(BadMessage)
    ));
    assert!(matches!(Machine::open_json("not json"), Err(BadMessage)));
}
