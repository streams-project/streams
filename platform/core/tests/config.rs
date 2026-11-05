//! What the core holds from the start message and from hello.

use contract::client::{CoreMode, DeviceClass, Form};
use contract::core_msg::{CoreConfig, CoreConfigTag};
use contract::hello::{ClassValue, ClientFloor, Hello};
use contract::version::{VersionRange, WireDate};
use platform_core::config::{Config, DEFAULT_START_DEADLINE_MS, Standing, version_below};

fn start(class: DeviceClass, version: &str) -> CoreConfig {
    CoreConfig {
        contract: WireDate::current(),
        tag: CoreConfigTag::default(),
        class,
        form: Form::Desktop,
        mode: CoreMode::Local,
        app: "sample".to_string(),
        version: version.to_string(),
        os: "sample".to_string(),
        seed: "00112233445566778899aabbccddeeff".to_string(),
        card: None,
    }
}

fn hello() -> Hello {
    let text = include_str!("../../contract/fixtures/2026-11-05/valid/Hello.basic.json");
    serde_json::from_str(text).unwrap()
}

#[test]
fn c108_while_hello_is_unknown_the_answer_is_unknown_and_unreachable_counts_as_required() {
    let mut config = Config::new(start(DeviceClass::Web, "1.0.0"));
    assert_eq!(config.accounts_required(), None);
    config.unreachable();
    assert_eq!(config.accounts_required(), Some(true));
    let mut open = hello();
    open.features.accounts_required = false;
    config.take_hello(open);
    assert_eq!(config.accounts_required(), Some(false));
}

#[test]
fn c108_a_failed_hello_never_drops_a_known_answer() {
    let mut config = Config::new(start(DeviceClass::Web, "1.0.0"));
    let mut open = hello();
    open.features.accounts_required = false;
    config.take_hello(open);
    config.unreachable();
    assert_eq!(config.accounts_required(), Some(false));
}

#[test]
fn c110_limits_arrive_from_hello_and_the_core_holds_them() {
    let mut config = Config::new(start(DeviceClass::Web, "1.0.0"));
    assert!(config.limits().is_none());
    config.take_hello(hello());
    assert_eq!(config.limits().map(|limits| limits.profiles_max), Some(5));
}

#[test]
fn the_start_deadline_is_per_class_from_hello_with_one_fallback() {
    let mut config = Config::new(start(DeviceClass::Tv, "1.0.0"));
    assert_eq!(config.start_deadline_ms(), DEFAULT_START_DEADLINE_MS);
    config.take_hello(hello());
    assert_eq!(config.start_deadline_ms(), 12_000);
    let mut other = hello();
    other.start_deadline_ms = vec![ClassValue {
        class: DeviceClass::Web,
        value: 1,
    }];
    config.take_hello(other);
    assert_eq!(config.start_deadline_ms(), DEFAULT_START_DEADLINE_MS);
}

#[test]
fn versions_compare_as_numbers_and_never_for_equality() {
    assert!(version_below("1.9.0", "1.10.0"));
    assert!(!version_below("1.10.0", "1.9.0"));
    assert!(!version_below("1.2.0", "1.2.0"));
    assert!(!version_below("1.2", "1.2.0"));
    assert!(version_below("1.2", "1.2.1"));
    assert!(!version_below("2", "1.9.9"));
}

#[test]
fn an_old_app_a_killed_build_or_a_contract_too_old_must_upgrade() {
    let mut config = Config::new(start(DeviceClass::Web, "0.9.0"));
    assert_eq!(config.take_hello(hello()), Standing::Upgrade);

    let mut config = Config::new(start(DeviceClass::Web, "1.0.0"));
    assert_eq!(config.take_hello(hello()), Standing::Fine);

    let mut killed = hello();
    killed.killed_builds = vec!["1.0.0".to_string()];
    assert_eq!(config.take_hello(killed), Standing::Upgrade);

    let mut ahead = hello();
    ahead.contract = VersionRange {
        min: WireDate::parse("2099-01-01").unwrap(),
        max: WireDate::parse("2099-01-01").unwrap(),
    };
    assert_eq!(config.take_hello(ahead), Standing::Upgrade);

    let mut newer = hello();
    newer.contract = VersionRange {
        min: WireDate::parse("2000-01-01").unwrap(),
        max: WireDate::parse("2000-01-01").unwrap(),
    };
    assert_eq!(config.take_hello(newer), Standing::Fine);

    let mut other_class = hello();
    other_class.client_min = vec![ClientFloor {
        class: DeviceClass::Tv,
        version: "9.0.0".to_string(),
    }];
    assert_eq!(config.take_hello(other_class), Standing::Fine);
}
