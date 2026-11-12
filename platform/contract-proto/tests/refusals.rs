//! Every fixture in `platform/contract/fixtures/2026-11-05/refused`, classified,
//! and each one that a generated rule covers actually refused.
//!
//! The first test is the one that matters most and it is the cheapest: it reads the
//! folder and fails when a file is there that this table does not name. The spike's
//! classification covered 25 of the 27 files in that folder, and the two it missed
//! were `TitleKey.id_over_max` ("movie:2147483648") and `TitleKey.id_ten_digits`
//! ("tv:9999999999") - the pair a patterned string gets wrong. A table that is not
//! checked against the folder is how that happens.
//!
//! CORRECTION, 2026-11-12, verified: this file used to say those two were "the exact
//! pair the generated Kotlin accepts and the core refuses". That was true before
//! `5db018f` and is not true now. That commit replaced
//! `"^(movie|tv):[1-9][0-9]{0,9}$"` with the ten branch alternation at
//! `platform/contract/src/title.rs:23-25` and added the 230 line
//! `platform/tools/check_patterns.py`, and the generated Kotlin at
//! `platform/contract/generated/types/Contract.kt:2444` refuses both keys today
//! (checked by running the expression against both strings, and by
//! `python3 tools/check_patterns.py`, which passes). The case for declaring the
//! bound is therefore not that a client is wrong; it is that the rule is still
//! written twice - as `id == 0 || id > MAX_TITLE_ID` at `title.rs:98-103` and as a
//! hand expanded ten branch regular expression at `title.rs:23-25` - and the two
//! agree only because a check compares them on twelve fixture strings. Declared on
//! the field there is no second statement to police.

// A packed day is written `2026_11_32`, grouped year_month_day, because that is
// what the number means and because every assertion here is about which days and
// which ids the contract accepts. Clippy wants `20_261_132`, grouped in threes from
// the right, which hides the month and the day - and `mistyped_literal_suffixes`
// reads the `_32` of `2026_11_32` as a mistyped `i32`. Both are right about the
// shape of the literal and wrong about this file.
#![expect(
    clippy::inconsistent_digit_grouping,
    clippy::mistyped_literal_suffixes,
    reason = "a packed YYYYMMDD is grouped year_month_day, which is what it means"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use prost_reflect::{DescriptorPool, Kind};

use contract_proto::contract as c;
use contract_proto::rules::{Refused, Rule};
use contract_proto::validate as v;

/// The descriptor build.rs writes and checks in. Read here to pin what the schema
/// DECLARES, next to the tests that pin what it refuses.
const DESCRIPTOR: &[u8] = include_bytes!("../generated/contract.descriptor.binpb");

/// The dated corpus. `platform/contract/src/version.rs` WIRE_VERSION.
const WIRE_VERSION: &str = "2026-11-05";

/// How a refused fixture is refused on a protobuf wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum By {
    /// A rule generated from a field option. The validator names this rule and this
    /// field.
    Rule(Rule, &'static str),
    /// The decoder, before any rule runs: a wire type clash, or a message that
    /// cannot be read as the message it was handed to.
    Decoder(&'static str),
    /// Nothing, because the thing it refused was text and there is no text. Each of
    /// these is a deliberate, named change to what the contract refuses.
    GoneWithTheText(&'static str),
}

/// All 27. Counted by `ls`, checked against the folder by the first test below.
const CLASSIFIED: &[(&str, By)] = &[
    (
        "Attempt.no_group",
        By::Rule(Rule::Required, "contract.v1.Attempt.group"),
    ),
    (
        "Attempt.no_headers",
        By::Rule(Rule::Required, "contract.v1.Attempt.headers"),
    ),
    (
        "Attempt.no_verified",
        By::Rule(Rule::Required, "contract.v1.Attempt.verified"),
    ),
    (
        "CalendarDay.day_32",
        By::Rule(Rule::CalendarDay, "contract.v1.CalendarDay.ymd"),
    ),
    (
        "CalendarDay.month_13",
        By::Rule(Rule::CalendarDay, "contract.v1.CalendarDay.ymd"),
    ),
    (
        "CalendarDay.no_dashes",
        By::GoneWithTheText(
            "\"20261120\" is refused today for having no dashes. Packed it is 20261120, \
             which is 2026-11-20, a day the project accepts. The refusal was about \
             punctuation.",
        ),
    ),
    (
        "Command.list_bare_id",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "Command.remind_bare_id",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "CommandBatch.bare_id",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "CoreIn.bad_date",
        By::Decoder(
            "\"contract\": \"soon\". ContractDate.ymd is a varint, so a length \
             delimited string in that field does not decode.",
        ),
    ),
    (
        "CoreIn.no_contract",
        By::Rule(Rule::Required, "contract.v1.CoreIn.contract"),
    ),
    (
        "CoreIn.no_mono_ms",
        By::Rule(Rule::Required, "contract.v1.CoreIn.mono_ms"),
    ),
    (
        "CoreIn.wrong_type",
        By::Decoder(
            "there is no `type` field. A CoreOut handed to a CoreIn reader clashes on \
             field 2, varint against length delimited, and an empty one is caught by \
             `required` on now_ms.",
        ),
    ),
    (
        "CoreOut.wrong_type",
        By::Decoder("the same, in the other direction."),
    ),
    (
        "GateLine.bad_date",
        By::Decoder("\"date\": \"20 November\" against a varint ymd."),
    ),
    (
        "Hello.no_range",
        By::Rule(Rule::Required, "contract.v1.Hello.contract"),
    ),
    (
        "PlayRequest.bare_id_number",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "PlayRequest.bare_id_text",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "Screen.bare_id_in_a_tile",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "Tile.bare_id",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "TitleKey.bare_id_number",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "TitleKey.bare_id_text",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "TitleKey.id_over_max",
        By::Rule(Rule::Max, "contract.v1.TitleKey.id"),
    ),
    (
        "TitleKey.id_ten_digits",
        By::Rule(Rule::Max, "contract.v1.TitleKey.id"),
    ),
    (
        "TitleKey.id_zero",
        By::Rule(Rule::Min, "contract.v1.TitleKey.id"),
    ),
    (
        "TitleKey.unknown_kind",
        By::Rule(Rule::Closed, "contract.v1.TitleKey.kind"),
    ),
    (
        "TitleKey.upper_case",
        By::GoneWithTheText(
            "\"Movie:1\" is refused today for its capital letter. There is no kind word \
             on the wire, only a number, so there is no case to get wrong.",
        ),
    ),
];

/// Where the refused corpus lives: `../contract/fixtures`, because this crate now
/// sits in the checkout beside platform/contract.
///
/// There is deliberately NO environment override. While this crate lived in the
/// spike worktree it read the corpus through `CONTRACT_FIXTURES`, because that tree
/// is pinned at cefb33c (2026-11-10) and has 25 refused fixtures where
/// tasks/boundaries has 27 - `TitleKey.id_over_max` and `TitleKey.id_ten_digits`
/// were added afterwards in 5db018f, and they are the pair that proves the defect
/// this package closes. The whole job of the first test below is to fail when the
/// corpus and the table disagree, and a redirectable corpus is a way to make it
/// pass against the wrong folder. One path, no escape.
fn refused_folder() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate sits under platform/")
        .join("contract/fixtures")
        .join(WIRE_VERSION)
        .join("refused")
}

#[test]
fn the_table_names_every_refused_fixture_and_no_others() {
    let on_disk: BTreeSet<String> = fs::read_dir(refused_folder())
        .expect("the refused corpus is where the contract keeps it")
        .map(|entry| entry.expect("the folder reads").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            path.file_stem()
                .expect("a fixture has a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let in_table: BTreeSet<String> = CLASSIFIED
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect();

    let missing: Vec<&String> = on_disk.difference(&in_table).collect();
    let extra: Vec<&String> = in_table.difference(&on_disk).collect();
    assert!(
        missing.is_empty(),
        "these refused fixtures are not classified: {missing:?}"
    );
    assert!(
        extra.is_empty(),
        "these classified names have no fixture: {extra:?}"
    );
    assert_eq!(on_disk.len(), 27, "the corpus changed size");
}

#[test]
fn the_counts_are_what_the_schema_claims() {
    let mut by_rule = 0;
    let mut by_decoder = 0;
    let mut gone = 0;
    for (_, how) in CLASSIFIED {
        match how {
            By::Rule(..) => by_rule += 1,
            By::Decoder(_) => by_decoder += 1,
            By::GoneWithTheText(_) => gone += 1,
        }
    }
    assert_eq!(by_rule, 21, "refused by a generated rule");
    assert_eq!(by_decoder, 4, "refused by the decoder");
    assert_eq!(
        gone, 2,
        "no longer refused, because the text they refused is gone"
    );
    assert_eq!(by_rule + by_decoder + gone, 27);
    // Nothing is accepted with nothing to catch it. That is the whole claim.
}

/// The one rule that needs a lookup: which rule and field the validator names.
fn expect(result: Result<(), Refused>, rule: Rule, path: &str) {
    let refusal = result.expect_err("this message must be refused");
    assert_eq!(
        refusal.rule(),
        rule,
        "refused by the wrong rule: {refusal:?}"
    );
    assert_eq!(
        refusal.path(),
        path,
        "refused on the wrong field: {refusal:?}"
    );
}

fn bare_id(id: u64) -> c::TitleKey {
    // A bare id on this wire: no kind, which is the zero value. There is no way to
    // send `1001` and have it mean a title, which is the point of the reshape.
    c::TitleKey { kind: 0, id }
}

fn movie(id: u64) -> c::TitleKey {
    c::TitleKey { kind: 1, id }
}

fn attempt() -> c::Attempt {
    c::Attempt {
        id: Some("a2".into()),
        url: Some("https://media2.example/b.mp4".into()),
        kind: Some(2),
        group: Some("g2".into()),
        headers: Some(c::Headers::default()),
        verified: Some(false),
        ..Default::default()
    }
}

#[test]
fn a_missing_required_field_is_refused_by_name() {
    let mut no_group = attempt();
    no_group.group = None;
    expect(
        v::attempt(&no_group),
        Rule::Required,
        "contract.v1.Attempt.group",
    );

    // The sharpest case in the spike: on a JSON-shaped schema this fixture and its
    // valid twin encoded to the same 40 bytes, because a proto3 map has no
    // presence. The wrapper message restores it, for two bytes.
    let mut no_headers = attempt();
    no_headers.headers = None;
    expect(
        v::attempt(&no_headers),
        Rule::Required,
        "contract.v1.Attempt.headers",
    );

    let mut no_verified = attempt();
    no_verified.verified = None;
    expect(
        v::attempt(&no_verified),
        Rule::Required,
        "contract.v1.Attempt.verified",
    );
}

#[test]
fn an_empty_headers_map_is_not_a_missing_one() {
    // The twin of Attempt.no_headers: present and empty must still be accepted, or
    // the wrapper has bought a refusal the contract does not make.
    assert_eq!(v::attempt(&attempt()), Ok(()));
}

#[test]
fn a_bare_id_is_refused_wherever_a_title_key_appears() {
    expect(
        v::title_key(&bare_id(1001)),
        Rule::Closed,
        "contract.v1.TitleKey.kind",
    );
    expect(
        v::play_request(&c::PlayRequest {
            title: Some(bare_id(1001)),
            episode: None,
        }),
        Rule::Closed,
        "contract.v1.TitleKey.kind",
    );
    expect(
        v::tile(&c::Tile {
            title: Some(bare_id(1001)),
            name: Some("x".into()),
            ..Default::default()
        }),
        Rule::Closed,
        "contract.v1.TitleKey.kind",
    );
}

#[test]
fn a_bad_key_three_messages_deep_is_still_refused() {
    // refused/Screen.bare_id_in_a_tile.json. The generated validator walks into
    // every message field and every repeated message, so the rule declared once on
    // TitleKey.kind reaches a tile inside a section inside a screen.
    let screen = c::Screen {
        name: Some(1),
        sections: vec![c::Section {
            id: Some("s".into()),
            kind: Some(2),
            tiles: vec![c::Tile {
                title: Some(bare_id(1001)),
                name: Some("x".into()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ttl_s: Some(1),
        ..Default::default()
    };
    expect(
        v::screen(&screen),
        Rule::Closed,
        "contract.v1.TitleKey.kind",
    );
    // The refusal names the field the RULE is declared on, not the route the
    // instance took to get there. That is deliberate: there is one declaration, so
    // there is one name. A message for a viewer needs the route, and building it is
    // the conversion layer's job (note 204 P2), not the validator's.
}

#[test]
fn the_two_keys_a_patterned_string_gets_wrong_are_refused_by_a_bound_here() {
    // The two keys that needed a ten branch regular expression to refuse, refused by
    // `{min: 1, max: 2147483647}` on the field instead. Before `5db018f` the emitted
    // expression was `[1-9][0-9]{0,9}` and really did accept both of these while
    // title.rs refused them; that is fixed, and what is left is one rule written
    // twice - the comparison in title.rs and the alternation beside it - held
    // together by check_patterns.py. Here it is written once and every language's
    // check comes out of it, so there is nothing to hold together.
    expect(
        v::title_key(&movie(2_147_483_648)),
        Rule::Max,
        "contract.v1.TitleKey.id",
    );
    expect(
        v::title_key(&movie(0)),
        Rule::Min,
        "contract.v1.TitleKey.id",
    );
    // And the edge stays where title.rs:17 puts it.
    assert_eq!(v::title_key(&movie(2_147_483_647)), Ok(()));
    assert_eq!(v::title_key(&movie(1)), Ok(()));
}

#[test]
fn an_unknown_title_kind_is_refused_and_every_other_unknown_word_is_kept() {
    // TitleKind is the one closed enum (platform/contract/src/title.rs:23), and the
    // refusal is on the field, not the enum: proto3 enums are open over numbers.
    expect(
        v::title_key(&c::TitleKey { kind: 7, id: 1 }),
        Rule::Closed,
        "contract.v1.TitleKey.kind",
    );
    // The other 39 enums stay open, which is what the tolerated corpus needs. An
    // unknown section kind is carried, not refused: tolerated/Section.unknown_kind.
    let section = c::Section {
        id: Some("s".into()),
        kind: Some(99),
        ..Default::default()
    };
    assert_eq!(v::section(&section), Ok(()));
}

#[test]
fn a_packed_day_that_is_not_a_day_is_refused() {
    expect(
        v::calendar_day(&c::CalendarDay {
            ymd: Some(2026_11_32),
        }),
        Rule::CalendarDay,
        "contract.v1.CalendarDay.ymd",
    );
    expect(
        v::calendar_day(&c::CalendarDay {
            ymd: Some(2026_13_01),
        }),
        Rule::CalendarDay,
        "contract.v1.CalendarDay.ymd",
    );
    assert_eq!(
        v::calendar_day(&c::CalendarDay {
            ymd: Some(2026_11_05)
        }),
        Ok(())
    );
    // refused/CalendarDay.no_dashes.json, which stops being refused on purpose.
    assert_eq!(
        v::calendar_day(&c::CalendarDay {
            ymd: Some(2026_11_20)
        }),
        Ok(())
    );
}

#[test]
fn a_contract_date_is_not_held_to_the_calendar_or_to_wire_min() {
    // WireDate::parse (platform/contract/src/version.rs:1132-1144) checks eight
    // digits in YYYY-MM-DD shape and nothing else, and the WIRE_MIN floor lives in
    // VersionRange::fit, a comparison the core makes. The spike's `contract_date`
    // check applied both, which would have added two refusals the contract does not
    // make. This test is here so nobody adds them back by tidying.
    assert_eq!(
        v::contract_date(&c::ContractDate {
            ymd: Some(2026_13_01)
        }),
        Ok(())
    );
    assert_eq!(
        v::contract_date(&c::ContractDate {
            ymd: Some(2020_01_01)
        }),
        Ok(())
    );
    expect(
        v::contract_date(&c::ContractDate { ymd: None }),
        Rule::Required,
        "contract.v1.ContractDate.ymd",
    );
}

#[test]
fn the_core_messages_refuse_what_they_refuse_today() {
    let date = c::ContractDate {
        ymd: Some(2026_11_05),
    };
    expect(
        v::core_in(&c::CoreIn {
            contract: None,
            now_ms: Some(1),
            mono_ms: Some(1),
            events: vec![],
        }),
        Rule::Required,
        "contract.v1.CoreIn.contract",
    );
    expect(
        v::core_in(&c::CoreIn {
            contract: Some(date),
            now_ms: Some(1),
            mono_ms: None,
            events: vec![],
        }),
        Rule::Required,
        "contract.v1.CoreIn.mono_ms",
    );
    assert_eq!(
        v::core_in(&c::CoreIn {
            contract: Some(date),
            now_ms: Some(1),
            mono_ms: Some(1),
            events: vec![],
        }),
        Ok(())
    );
}

#[test]
fn a_hello_with_no_version_range_is_refused() {
    // A message field has presence, so a missing range is absent and not an
    // all-default VersionRange. refused/Hello.no_range.json.
    let hello = c::Hello {
        contract: None,
        ..Default::default()
    };
    expect(
        v::hello(&hello),
        Rule::Required,
        "contract.v1.Hello.contract",
    );
}

/// Hand build `TitleKey { kind, id }` with the id as a varint of any width, which
/// is what a sender in another language would put on the wire.
fn encoded_key(kind: u8, id: u64) -> Vec<u8> {
    let mut bytes = vec![0x08, kind, 0x10];
    let mut value = id;
    while value >= 0x80 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "one varint group at a time"
        )]
        bytes.push((value as u8) | 0x80);
        value >>= 7;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the last group is under 0x80"
    )]
    bytes.push(value as u8);
    bytes
}

#[test]
fn a_ten_digit_id_arrives_whole_and_is_refused_rather_than_truncated() {
    // This is why TitleKey.id is uint64 and not uint32, and it is the sharpest thing
    // measured in this package.
    //
    // refused/TitleKey.id_ten_digits.json is "tv:9999999999". The generated Kotlin
    // regular expression accepts it; title.rs refuses it. Declared as uint32 the
    // schema does something worse than either: protobuf truncates a varint that does
    // not fit the declared width, so these exact bytes decoded as
    // `TitleKey { kind: Tv, id: 1410065407 }` - a different and entirely valid title.
    // A viewer would be shown the wrong film and no fault would be reported anywhere.
    //
    // As uint64 the value arrives whole and the bound refuses it by name.
    use prost::Message as _;

    let bytes = encoded_key(0x02, 9_999_999_999);
    let decoded = c::TitleKey::decode(bytes.as_slice()).expect("the varint fits a uint64");
    assert_eq!(decoded.id, 9_999_999_999, "the id must not be truncated");
    expect(v::title_key(&decoded), Rule::Max, "contract.v1.TitleKey.id");

    // The largest id the schema allows still round trips, so the ceiling is the
    // bound's job and not an accident of the field width.
    let largest = c::TitleKey {
        kind: 1,
        id: 2_147_483_647,
    };
    let mut round = Vec::new();
    largest.encode(&mut round).expect("encoding cannot fail");
    assert_eq!(c::TitleKey::decode(round.as_slice()), Ok(largest));
}

#[test]
fn the_three_bounded_numbers_are_uint64_in_the_schema_because_uint32_truncates() {
    // The test above proves the BEHAVIOUR. This one pins the DECLARATION, read out
    // of the checked in descriptor, so the schema edit that would reintroduce the
    // defect fails here by name rather than somewhere downstream.
    //
    // `uint32` on any of these three is not a smaller field, it is a silent one. A
    // varint that does not fit the declared width is truncated, not refused: the
    // bytes for "tv:9999999999" decode cleanly as TitleKey { kind: Tv, id:
    // 1410065407 }, a different and entirely valid title, with no fault reported
    // anywhere. A viewer would be shown the wrong film. `uint64` makes the illegal
    // value representable so the bound in common.proto can refuse it, and a varint
    // costs the same bytes for every value that fits, so a real key is not a byte
    // larger.
    //
    // ContractDate.ymd is in the list for the same reason at a different scale:
    // WireDate::parse (platform/contract/src/version.rs) takes eight digits and
    // nothing else, so "123456789" is refused today, and that refusal survives the
    // reshape only because the value arrives whole and `{max: 99999999}` sees it.
    let pool = DescriptorPool::decode(DESCRIPTOR).expect("the checked in descriptor loads");
    for (message, field) in [
        ("contract.v1.TitleKey", "id"),
        ("contract.v1.CalendarDay", "ymd"),
        ("contract.v1.ContractDate", "ymd"),
    ] {
        let kind = pool
            .get_message_by_name(message)
            .unwrap_or_else(|| panic!("{message} is in the schema"))
            .get_field_by_name(field)
            .unwrap_or_else(|| panic!("{message}.{field} is in the schema"))
            .kind();
        assert_eq!(
            kind,
            Kind::Uint64,
            "{message}.{field} is {kind:?} and must be uint64: a narrower varint is \
             truncated on decode, which turns a refusal into a different valid value"
        );
    }
}

#[test]
fn a_nine_digit_contract_date_is_refused() {
    // WireDate::parse takes eight digits and nothing else, so "123456789" is refused
    // today. Packed, that refusal survives only because ContractDate.ymd has a
    // ceiling; without it the reshape would have quietly dropped it.
    expect(
        v::contract_date(&c::ContractDate {
            ymd: Some(123_456_789),
        }),
        Rule::Max,
        "contract.v1.ContractDate.ymd",
    );
    assert_eq!(
        v::contract_date(&c::ContractDate {
            ymd: Some(2026_11_05)
        }),
        Ok(())
    );
}

#[test]
fn a_bad_key_inside_a_command_batch_is_refused() {
    // refused/CommandBatch.bare_id.json: through a oneof arm, inside a repeated
    // field, inside a message.
    let batch = c::CommandBatch {
        ops: vec![c::CommandOp {
            op_id: Some("op1".into()),
            age_ms: None,
            command: Some(c::Command {
                command: Some(c::command::Command::List(c::command::List {
                    title: Some(bare_id(1001)),
                    in_list: Some(true),
                })),
            }),
        }],
    };
    expect(
        v::command_batch(&batch),
        Rule::Closed,
        "contract.v1.TitleKey.kind",
    );
}
