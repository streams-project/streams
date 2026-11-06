//! Who is watching after time away. Each test names the behaviour it covers.

use contract::client::Form;
use platform_core::gate::{Gate, idle_limit_ms};

const MINUTE: u64 = 60_000;
const START: u64 = 1_000_000;

fn gate(form: Form) -> Gate {
    let mut gate = Gate::new(form);
    gate.set_profiles(2);
    gate.set_active(true);
    gate.seen(START);
    gate
}

#[test]
fn c74_c103_c130_p11_the_limit_is_120_minutes_on_a_phone_and_40_on_a_tablet_or_desktop() {
    assert_eq!(idle_limit_ms(Form::Phone), Some(120 * MINUTE));
    assert_eq!(idle_limit_ms(Form::Tablet), Some(40 * MINUTE));
    assert_eq!(idle_limit_ms(Form::Desktop), Some(40 * MINUTE));
}

#[test]
fn the_tv_value_waits_for_a_reference_capture_so_a_tv_is_not_asked() {
    assert_eq!(idle_limit_ms(Form::Tv), None);
    let mut gate = gate(Form::Tv);
    assert!(!gate.activity(START + 600 * MINUTE));
}

#[test]
fn c130_asked_when_used_after_the_limit_and_not_at_the_limit() {
    let mut desktop = gate(Form::Desktop);
    assert!(!desktop.activity(START + 40 * MINUTE));
    let mut desktop = gate(Form::Desktop);
    assert!(desktop.activity(START + 40 * MINUTE + 1));
    assert!(desktop.asking());
    let mut phone = gate(Form::Phone);
    assert!(!phone.activity(START + 100 * MINUTE));
}

#[test]
fn c130_activity_counts_at_most_every_30_s() {
    let mut desktop = gate(Form::Desktop);
    // 20 s after the last write: not written, so the idle time still runs from START
    desktop.activity(START + 20_000);
    assert_eq!(desktop.last_seen(), Some(START));
    desktop.activity(START + 30_001);
    assert_eq!(desktop.last_seen(), Some(START + 30_001));
}

#[test]
fn c103_leaving_counts_from_now_and_coming_back_checks() {
    let mut desktop = gate(Form::Desktop);
    desktop.left(START + 10 * MINUTE);
    assert_eq!(desktop.last_seen(), Some(START + 10 * MINUTE));
    assert!(!desktop.activity(START + 50 * MINUTE));
    let mut desktop = gate(Form::Desktop);
    desktop.left(START + 10 * MINUTE);
    assert!(desktop.activity(START + 50 * MINUTE + 1));
}

#[test]
fn c130_not_asked_with_fewer_than_two_profiles_or_no_active_profile() {
    let mut one = gate(Form::Desktop);
    one.set_profiles(1);
    assert!(!one.activity(START + 100 * MINUTE));
    let mut none = gate(Form::Desktop);
    none.set_active(false);
    assert!(!none.activity(START + 100 * MINUTE));
}

#[test]
fn entering_a_profile_clears_the_question_and_starts_the_count_again() {
    let mut desktop = gate(Form::Desktop);
    assert!(desktop.activity(START + 50 * MINUTE));
    desktop.answered(START + 51 * MINUTE);
    assert!(!desktop.asking());
    assert!(!desktop.activity(START + 60 * MINUTE));
}
