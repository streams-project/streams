//! What happens to the 271 JSON fixtures: they become protobuf TEXT FORMAT, and
//! this test is the proof that the format works here, with no protoc and no
//! network.
//!
//! The question it answers is narrow and it is the one that decides the corpus:
//! can a fixture be stored as something a person can read and `git diff` can show,
//! and does it come back as the same bytes? Two facts make that non-obvious:
//!
//!   1. `prost` has no text format at all - no reader, no writer, nothing. A
//!      corpus in text format cannot be loaded by the crate the core links.
//!   2. `prost-reflect` does have both, over a `DynamicMessage` driven by the
//!      checked in descriptor. So the corpus is readable by a std tool and not by
//!      the core, which is the right way round: the core reads bytes.
//!
//! What this test does NOT do is convert the existing JSON fixtures. That needs the
//! shape mapping - a title key's text to {kind, id}, a date's text to a packed
//! number, a flattened union to a oneof - which is the conversion layer, note 204's
//! P2, and the corpus itself is P4. This package's job was to say what the format
//! is and prove it holds.

use prost::Message as _;
use prost_reflect::{DescriptorPool, DynamicMessage};

use contract_proto::contract as c;

const DESCRIPTOR: &[u8] = include_bytes!("../generated/contract.descriptor.binpb");

fn pool() -> DescriptorPool {
    DescriptorPool::decode(DESCRIPTOR).expect("the checked in descriptor loads")
}

fn as_text(pool: &DescriptorPool, full_name: &str, bytes: &[u8]) -> String {
    let descriptor = pool
        .get_message_by_name(full_name)
        .unwrap_or_else(|| panic!("{full_name} is in the schema"));
    let message = DynamicMessage::decode(descriptor, bytes).expect("the bytes decode");
    // `{:#}` and not `{:#?}`: Display is the text format writer, Debug prints the
    // internal map keyed by field number. An easy and silent mistake, worth the
    // comment.
    format!("{message:#}")
}

#[test]
fn a_fixture_round_trips_through_text_format_with_identical_bytes() {
    let pool = pool();
    let attempt = c::Attempt {
        id: Some("a2".into()),
        url: Some("https://media2.example/b.mp4".into()),
        kind: Some(2),
        group: Some("g2".into()),
        headers: Some(c::Headers::default()),
        verified: Some(false),
        ..Default::default()
    };
    let mut bytes = Vec::new();
    attempt.encode(&mut bytes).expect("encoding cannot fail");

    let text = as_text(&pool, "contract.v1.Attempt", &bytes);
    let descriptor = pool
        .get_message_by_name("contract.v1.Attempt")
        .expect("Attempt is in the schema");
    let parsed = DynamicMessage::parse_text_format(descriptor, &text)
        .expect("what the writer wrote, the reader reads");
    let mut again = Vec::new();
    parsed.encode(&mut again).expect("encoding cannot fail");

    assert_eq!(bytes, again, "text format lost or reordered something");
    assert_eq!(
        c::Attempt::decode(again.as_slice()),
        Ok(attempt),
        "the generated struct reads it back unchanged"
    );
}

#[test]
fn text_format_is_readable_and_diffable() {
    // The two properties the corpus needs, stated as assertions rather than as a
    // claim in a comment: field names appear, and one field is one line, so a diff
    // points at the field that changed.
    let pool = pool();
    let screen = c::Screen {
        name: Some(6), // SCREEN_NAME_MY_LIST, the word no proto3 JSON encoder could write
        sections: vec![c::Section {
            id: Some("s".into()),
            kind: Some(2),
            tiles: vec![c::Tile {
                title: Some(c::TitleKey { kind: 1, id: 550 }),
                name: Some("Fight Club".into()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ttl_s: Some(60),
        ..Default::default()
    };
    let mut bytes = Vec::new();
    screen.encode(&mut bytes).expect("encoding cannot fail");
    let text = as_text(&pool, "contract.v1.Screen", &bytes);

    // The writer puts the enum VALUE NAME in, not the number, which is what makes a
    // diff of this corpus mean something: `kind: SECTION_KIND_ROW`, not `kind: 2`.
    assert!(
        text.contains("name: SCREEN_NAME_MY_LIST"),
        "the enum word is readable:\n{text}"
    );
    assert!(
        text.contains("kind: TITLE_KIND_MOVIE"),
        "a nested enum word is readable:\n{text}"
    );
    assert!(text.contains("ttl_s: 60"), "a scalar is readable:\n{text}");
    assert!(text.contains("Fight Club"), "the text is readable:\n{text}");
    assert!(
        text.lines().count() > 8,
        "pretty form is one field per line:\n{text}"
    );
}

#[test]
fn the_empty_headers_wrapper_survives_the_format() {
    // The case the whole wrapper exists for. `headers {}` present and empty must
    // read back as present and empty, through bytes AND through text, or the
    // corpus cannot hold refused/Attempt.no_headers.json apart from its valid twin.
    let pool = pool();
    let with = c::Attempt {
        id: Some("a".into()),
        url: Some("u".into()),
        kind: Some(2),
        group: Some("g".into()),
        headers: Some(c::Headers::default()),
        verified: Some(true),
        ..Default::default()
    };
    let mut without = with.clone();
    without.headers = None;

    let mut with_bytes = Vec::new();
    with.encode(&mut with_bytes).expect("encoding cannot fail");
    let mut without_bytes = Vec::new();
    without
        .encode(&mut without_bytes)
        .expect("encoding cannot fail");
    assert_ne!(
        with_bytes, without_bytes,
        "present-and-empty must differ from absent on the wire"
    );

    let with_text = as_text(&pool, "contract.v1.Attempt", &with_bytes);
    let without_text = as_text(&pool, "contract.v1.Attempt", &without_bytes);
    assert!(
        with_text.contains("headers"),
        "the wrapper is visible:\n{with_text}"
    );
    assert!(
        !without_text.contains("headers"),
        "an absent wrapper must not appear:\n{without_text}"
    );
}

#[test]
fn the_checked_in_descriptor_still_carries_the_options() {
    // The descriptor is what `buf breaking` compares and what the tie check reads,
    // and it only carries the custom options because build.rs uses
    // `Compiler::encode_file_descriptor_set` rather than going through
    // prost_types. If someone simplifies that line, this test is what notices:
    // prost drops unknown fields, so the options would silently vanish and the
    // generator would emit an empty validator.
    let pool = pool();
    let extension = pool
        .get_extension_by_name("contract.constraints.v1.field")
        .expect("the option is in the descriptor");
    let title_key = pool
        .get_message_by_name("contract.v1.TitleKey")
        .expect("TitleKey is in the schema");
    let id = title_key
        .get_field_by_name("id")
        .expect("TitleKey.id is in the schema");
    assert!(
        id.options().has_extension(&extension),
        "TitleKey.id lost its bounds in the checked in descriptor"
    );
}
