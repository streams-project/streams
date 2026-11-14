//! The protobuf schema is the same contract, and no field number ever moves.
//!
//! `generated.rs` beside this file holds the 110 generated files to the contract by
//! writing them again and comparing bytes. `platform/contract-proto` had no such
//! tie: its fourteen `.proto` files are hand written, and on 2026-11-13 the contract
//! change in `cc1f607` left them behind - a whole message, a whole enum, an enum
//! word and four fields - while `./check` stayed green and the one consumer,
//! `platform/core/src/proto.rs` behind `--features proto`, stopped compiling.
//!
//! WHY THIS IS NOT AN EMITTER, WHICH IS THE OBVIOUS SHAPE
//! -----------------------------------------------------
//! The obvious tie is `generated.rs`'s: emit the `.proto` from the contract and
//! compare bytes. It is the wrong shape here, for one reason that is about
//! protobuf and one that is about these files.
//!
//! **A field number is forever, and nothing in the contract source states one.** An
//! emitter has to put a number on every field, and the only thing it could derive
//! one from is declaration order. Declaration order moves: `cc1f607` declared
//! `AttemptKind::Dash` SECOND, between `Hls` and `File`, and `Attempt::bitrate`
//! between `height` and `audio`. An order-numbering emitter would have handed
//! `Dash` the 2 that `File` holds and `bitrate` the 10 that `audio` holds, and every
//! deployed reader would then have read a file attempt as a dash attempt and an
//! audio list as a bitrate. The schema gives them 4 and 12 instead, which no
//! derivation from the contract can produce. So the numbers must be hand written in
//! the `.proto`, and a tool that writes the file cannot be the tool that keeps them.
//! This test never computes a number. It only reads them, and that is what makes
//! renumbering impossible here rather than merely unlikely.
//!
//! **The `.proto` files are not generated prose.** 2,240 lines of which the
//! declarations are about a third; the rest is the reasoning for every reshape - why
//! `TitleKey.id` is `uint64` and not `uint32`, why `CalendarDay` is packed, why
//! `Headers` exists at all. An emitter would own all of it, so every one of those
//! paragraphs would have to move into the Rust contract source, where it does not
//! belong: they explain protobuf decisions, not contract decisions.
//!
//! So: the contract states the shape, the schema states the numbers, and this file
//! holds the first against the second. What it cannot hold - that a number which was
//! never in a commit is still the right one - is `buf breaking`'s job, which
//! `buf.yaml` already configures at `FILE` level and which no machine here can run.
//!
//! WHERE EACH FACT COMES FROM, SO NO NUMBER HAS TWO HOMES
//! ------------------------------------------------------
//! * The shape: `contract_gen::shape::items`, the same call `generated.rs` and all
//!   five type emitters make. Not a second reading of the contract - the same one.
//! * The schema: `contract-proto/generated/contract.descriptor.binpb`, which
//!   `build.rs` writes from the `.proto` files and `./check` holds with
//!   `git diff --exit-code`.
//! * The value rules: `contract-proto/generated/rules.json`, written by
//!   `contract-validate-gen` out of the same descriptor.
//! * The history: `git show HEAD:` of that descriptor. Git already keeps every past
//!   numbering, so holding one commit against its parent holds every commit against
//!   every ancestor by induction, and nothing in the tree has to carry a second copy
//!   of 646 field numbers.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use contract_gen::shape::{Field, Shape, Ty, items, upper_camel};
use prost_reflect::{DescriptorPool, Kind};

const PACKAGE: &str = "contract.v1";
const DESCRIPTOR: &str = "platform/contract-proto/generated/contract.descriptor.binpb";
const RULES: &str = "platform/contract-proto/generated/rules.json";

/// Deliberate reshapes, each one a decision rather than a derivation. A reshape
/// that is in the schema and not in one of these tables fails a test below, so
/// none of them can be added quietly.
mod decided {
    /// A contract type whose message carries another name.
    /// `WireDate` is `ContractDate` so the generated type differs from
    /// `CalendarDay`'s the way the two Rust newtypes do (`common.proto`).
    pub const RENAMED: &[(&str, &str)] = &[("WireDate", "ContractDate")];

    /// Messages with no contract type of their own. Both restore presence to a map,
    /// which proto3 has not got: `refused/Attempt.no_headers.json` is byte-identical
    /// to its valid twin without the wrapper (`common.proto`).
    pub const WRAPPERS: &[&str] = &["Headers", "TextMap"];

    /// The contract's type tag on a core message. A binary wire needs no tag: the
    /// arm is the oneof case. `shape::Shape::Fixed` is the whole of each one.
    pub const TAG_FIELDS: &[(&str, &str)] = &[
        ("CoreIn", "type"),
        ("CoreOut", "type"),
        ("CoreConfig", "type"),
    ];

    /// Union arms whose nested message is not `upper_camel(word)`. Both are name
    /// collisions in a generated language, named at the message in `core.proto` and
    /// `driver.proto`.
    pub const ARMS: &[(&str, &str, &str)] = &[
        ("DriverEvent", "error", "ErrorEvent"),
        ("Action", "command", "CommandAction"),
        ("Effect", "view", "ViewEffect"),
        ("Event", "action", "ActionEvent"),
    ];

    /// Unions the Rust flattens into a carrier struct and the schema nests properly.
    /// `CommandOp` holds `op_id`, `age_ms` and a `Command`; the arms are `Command`'s
    /// (`command.proto`). The arms are still checked, under `Command`.
    pub const FLATTENED: &[&str] = &["CommandOp"];
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the repository root is three above platform/contract/gen")
        .to_path_buf()
}

fn pool_from(bytes: &[u8]) -> DescriptorPool {
    DescriptorPool::decode(bytes).expect("the descriptor decodes")
}

fn pool() -> DescriptorPool {
    pool_from(&std::fs::read(root().join(DESCRIPTOR)).expect("the descriptor is on disk"))
}

/// `http_done` as `HTTP_DONE`, `TitleKind` as `TITLE_KIND`.
fn screaming(name: &str) -> String {
    let mut out = String::new();
    for (index, letter) in name.char_indices() {
        if letter.is_ascii_uppercase() && index != 0 {
            out.push('_');
        }
        out.push(letter.to_ascii_uppercase());
    }
    out
}

/// The value name a word gets: the enum's own name, then the word.
fn value_name(enum_name: &str, word: &str) -> String {
    format!(
        "{}_{}",
        screaming(enum_name),
        word.replace('-', "_").to_ascii_uppercase()
    )
}

fn renamed(name: &str) -> String {
    decided::RENAMED
        .iter()
        .find(|(from, _)| *from == name)
        .map_or_else(|| name.to_string(), |(_, to)| (*to).to_string())
}

fn full(name: &str) -> String {
    format!("{PACKAGE}.{}", renamed(name))
}

fn arm_name(union: &str, word: &str) -> String {
    decided::ARMS
        .iter()
        .find(|(owner, arm, _)| *owner == union && *arm == word)
        .map_or_else(|| upper_camel(word), |(_, _, to)| (*to).to_string())
}

/// What the schema may hold in a message beyond the contract's own fields.
#[derive(Clone, Debug)]
enum Extra {
    /// Nothing. A plain struct is a plain message.
    None,
    /// The oneof cases of a union: one field per arm, named for the arm's word.
    /// The arms themselves are checked as messages of their own.
    Cases(BTreeSet<String>),
    /// A reshape: the message is made of parts the contract states as one value.
    /// `TitleKey` is `{kind, id}` where the contract has `movie:550`. The parts are
    /// decided in `common.proto` and what each one refuses is held by
    /// `platform/contract-proto/tests/refusals.rs`, not here.
    Reshaped,
}

/// One message the contract asks for: the fields it must hold, and what else the
/// schema is allowed to hold. Nothing here is a number - this is the shape only.
#[derive(Clone, Debug)]
struct Expect {
    fields: Vec<Field>,
    extra: Extra,
}

fn expected_messages() -> BTreeMap<String, Expect> {
    let (defs, _) = contract_gen::schemas::definitions();
    let mut out: BTreeMap<String, Expect> = BTreeMap::new();
    for item in items(&defs) {
        let here = full(&item.name);
        let untagged = |fields: &[Field]| -> Vec<Field> {
            fields
                .iter()
                .filter(|field| {
                    !decided::TAG_FIELDS
                        .iter()
                        .any(|(owner, name)| *owner == item.name && *name == field.wire)
                })
                .cloned()
                .collect()
        };
        match &item.shape {
            Shape::Struct(fields) => {
                out.insert(
                    here,
                    Expect {
                        fields: untagged(fields),
                        extra: Extra::None,
                    },
                );
            }
            Shape::Checked(_) => {
                out.insert(
                    here,
                    Expect {
                        fields: Vec::new(),
                        extra: Extra::Reshaped,
                    },
                );
            }
            Shape::Union {
                common, variants, ..
            } => {
                // A flattened union keeps its carrier fields and holds the arms in
                // a message of their own, so its one extra case is that message.
                let flattened = decided::FLATTENED.contains(&item.name.as_str());
                let cases: BTreeSet<String> = if flattened {
                    core::iter::once("command".to_string()).collect()
                } else {
                    variants
                        .iter()
                        .map(|variant| variant.word.clone())
                        .collect()
                };
                out.insert(
                    here.clone(),
                    Expect {
                        fields: untagged(common),
                        extra: Extra::Cases(cases),
                    },
                );
                if flattened {
                    continue;
                }
                for variant in variants {
                    let arm = arm_name(&item.name, &variant.word);
                    out.insert(
                        format!("{here}.{arm}"),
                        Expect {
                            fields: variant.fields.clone(),
                            extra: Extra::None,
                        },
                    );
                }
            }
            Shape::Words(_) | Shape::Fixed(_) | Shape::Alias(_) => {}
        }
    }
    out
}

/// Every enum the contract asks for, as a full name and its words in order.
fn expected_enums() -> BTreeMap<String, Vec<String>> {
    let (defs, _) = contract_gen::schemas::definitions();
    items(&defs)
        .into_iter()
        .filter_map(|item| match item.shape {
            Shape::Words(words) => Some((full(&item.name), words)),
            _ => None,
        })
        .collect()
}

/// Contract names that reach the schema as a scalar rather than a type of their own.
fn aliases() -> BTreeSet<String> {
    let (defs, _) = contract_gen::schemas::definitions();
    items(&defs)
        .into_iter()
        .filter(|item| matches!(item.shape, Shape::Alias(_)))
        .map(|item| item.name)
        .collect()
}

fn real_messages(pool: &DescriptorPool) -> Vec<prost_reflect::MessageDescriptor> {
    pool.all_messages()
        .filter(|message| message.full_name().starts_with(PACKAGE) && !message.is_map_entry())
        .collect()
}

// ---------------------------------------------------------------------------
// The shape: the schema says what the contract says, and nothing else.
// ---------------------------------------------------------------------------

#[test]
fn every_contract_type_has_a_message_or_an_enum_in_the_schema() {
    let pool = pool();
    let mut missing = Vec::new();
    for name in expected_messages().keys() {
        if pool.get_message_by_name(name).is_none() {
            missing.push(format!("message {name}"));
        }
    }
    for name in expected_enums().keys() {
        if pool.get_enum_by_name(name).is_none() {
            missing.push(format!("enum {name}"));
        }
    }
    assert!(
        missing.is_empty(),
        "the contract has {} type(s) the protobuf schema has not got. Add each one to \
         platform/contract-proto/proto/contract/v1/, giving every field a NEW number \
         (one above the highest the message already uses) and never a number taken \
         from declaration order:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

#[test]
fn every_message_and_enum_in_the_schema_comes_from_the_contract() {
    let pool = pool();
    let messages = expected_messages();
    let enums = expected_enums();
    let wrappers: BTreeSet<String> = decided::WRAPPERS.iter().map(|name| full(name)).collect();
    let mut extra = Vec::new();
    for message in real_messages(&pool) {
        let name = message.full_name().to_string();
        if !messages.contains_key(&name) && !wrappers.contains(&name) {
            extra.push(format!("message {name}"));
        }
    }
    for item in pool.all_enums() {
        let name = item.full_name().to_string();
        if name.starts_with(PACKAGE) && !enums.contains_key(&name) {
            extra.push(format!("enum {name}"));
        }
    }
    assert!(
        extra.is_empty(),
        "the protobuf schema holds {} shape(s) the contract does not. A shape nobody \
         specified is a shape nothing refuses; either give it a contract type or name it \
         in `decided::WRAPPERS` with the reason:\n  {}",
        extra.len(),
        extra.join("\n  ")
    );
}

fn kind_matches(
    ty: &Ty,
    field: &prost_reflect::FieldDescriptor,
    aliased: &BTreeSet<String>,
) -> bool {
    let inner = match ty {
        Ty::List(inner) => {
            if !field.is_list() {
                return false;
            }
            inner.as_ref()
        }
        Ty::Map(_) => {
            // A map the contract requires needs a wrapper message, because proto3
            // has no presence for a map. An optional one is a bare map.
            return field.is_map()
                || matches!(field.kind(), Kind::Message(ref message)
                    if decided::WRAPPERS.iter().any(|name| full(name) == message.full_name()));
        }
        other => {
            if field.is_list() || field.is_map() {
                return false;
            }
            other
        }
    };
    match inner {
        Ty::Text => matches!(field.kind(), Kind::String),
        Ty::Bool => matches!(field.kind(), Kind::Bool),
        Ty::Int => matches!(
            field.kind(),
            Kind::Uint32 | Kind::Uint64 | Kind::Int32 | Kind::Int64
        ),
        Ty::Named(name) if aliased.contains(name) => !matches!(field.kind(), Kind::Message(_)),
        Ty::Named(name) => match field.kind() {
            Kind::Message(message) => message.full_name() == full(name),
            Kind::Enum(item) => item.full_name() == full(name),
            _ => false,
        },
        Ty::List(_) | Ty::Map(_) => false,
    }
}

#[test]
fn every_field_the_contract_has_is_in_the_schema_with_the_same_name_and_type() {
    let pool = pool();
    let aliased = aliases();
    let mut problems = Vec::new();
    for (name, expect) in expected_messages() {
        let Some(message) = pool.get_message_by_name(&name) else {
            continue; // named by the test above
        };
        for field in expect.fields {
            match message.get_field_by_name(&field.wire) {
                None => problems.push(format!(
                    "{name}.{} is in the contract and not in the schema",
                    field.wire
                )),
                Some(found) if !kind_matches(&field.ty, &found, &aliased) => {
                    problems.push(format!(
                        "{name}.{} is {:?} in the contract and {:?} in the schema",
                        field.wire,
                        field.ty,
                        found.kind()
                    ))
                }
                Some(_) => {}
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} field(s) differ. A field added to the schema takes the NEXT FREE number in \
         its message - one above the highest already there - and never the number its \
         position in the Rust would suggest:\n  {}",
        problems.len(),
        problems.join("\n  ")
    );
}

#[test]
fn no_field_is_in_the_schema_that_the_contract_does_not_have() {
    let pool = pool();
    let mut extra = Vec::new();
    for (name, expect) in expected_messages() {
        let Some(message) = pool.get_message_by_name(&name) else {
            continue;
        };
        if matches!(expect.extra, Extra::Reshaped) {
            continue;
        }
        let mut known: BTreeSet<String> = expect
            .fields
            .iter()
            .map(|field| field.wire.clone())
            .collect();
        if let Extra::Cases(cases) = &expect.extra {
            known.extend(cases.iter().cloned());
        }
        for field in message.fields() {
            if !known.contains(field.name()) {
                extra.push(format!("{name}.{}", field.name()));
            }
        }
    }
    assert!(
        extra.is_empty(),
        "{} field(s) are in the schema and not in the contract:\n  {}",
        extra.len(),
        extra.join("\n  ")
    );
}

#[test]
fn a_field_the_contract_requires_is_required_in_the_schema_too() {
    let rules: BTreeMap<String, Vec<String>> =
        serde_json::from_str(&std::fs::read_to_string(root().join(RULES)).expect("rules.json"))
            .expect("rules.json is a map of path to rule names");
    let pool = pool();
    let mut problems = Vec::new();
    for (name, expect) in expected_messages() {
        let Some(message) = pool.get_message_by_name(&name) else {
            continue;
        };
        for field in expect.fields {
            let path = format!("{name}.{}", field.wire);
            let Some(found) = message.get_field_by_name(&field.wire) else {
                continue;
            };
            let declared = rules.get(&path).is_some_and(|rules| {
                // `presence_lost` is the honest answer where proto3 cannot carry
                // presence at all: a repeated field, and a map that is required and
                // wrapped carries its presence in the wrapper instead.
                rules
                    .iter()
                    .any(|rule| rule == "required" || rule == "presence_lost")
            });
            let wrapped = matches!(field.ty, Ty::Map(_))
                && matches!(found.kind(), Kind::Message(ref message)
                    if decided::WRAPPERS.iter().any(|name| full(name) == message.full_name()));
            if field.required && !(declared || wrapped) {
                problems.push(format!(
                    "{path} is required in the contract and the schema refuses nothing. \
                     Declare `required: true` on the field, or `presence_lost: true` if \
                     proto3 cannot carry its presence"
                ));
            }
            if !field.required && declared && rules[&path].iter().any(|rule| rule == "required") {
                problems.push(format!(
                    "{path} is optional in the contract and `required: true` in the schema, \
                     which is a refusal the contract does not make"
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} field(s) refuse different things:\n  {}",
        problems.len(),
        problems.join("\n  ")
    );
}

#[test]
fn every_word_has_a_value_and_the_schema_invents_none() {
    let pool = pool();
    let mut problems = Vec::new();
    for (name, words) in expected_enums() {
        let Some(item) = pool.get_enum_by_name(&name) else {
            continue;
        };
        let simple = name.rsplit('.').next().unwrap_or(&name);
        let want: BTreeSet<String> = words.iter().map(|word| value_name(simple, word)).collect();
        let have: BTreeSet<String> = item
            .values()
            .map(|value| value.name().to_string())
            .collect();
        let zero = format!("{}_UNSPECIFIED", screaming(simple));
        for missing in want.difference(&have) {
            problems.push(format!(
                "{name}.{missing} is a contract word with no value. Give it the NEXT number \
                 after the highest in the enum, never the number its position suggests"
            ));
        }
        for found in have.difference(&want) {
            if *found != zero {
                problems.push(format!(
                    "{name}.{found} is a value the contract has no word for"
                ));
            }
        }
        if !have.contains(&zero) {
            problems.push(format!(
                "{name} has no {zero}, so an unknown number has nowhere to go"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "{} word(s) differ:\n  {}",
        problems.len(),
        problems.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// The numbers: forever, and never derived.
// ---------------------------------------------------------------------------

/// Every `message.field` and `enum.VALUE` of a descriptor, with its number.
fn numbering(pool: &DescriptorPool) -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    for message in real_messages(pool) {
        for field in message.fields() {
            out.insert(
                format!("{}.{}", message.full_name(), field.name()),
                i64::from(field.number()),
            );
        }
    }
    for item in pool.all_enums() {
        if !item.full_name().starts_with(PACKAGE) {
            continue;
        }
        for value in item.values() {
            out.insert(
                format!("{}.{}", item.full_name(), value.name()),
                i64::from(value.number()),
            );
        }
    }
    out
}

/// The descriptor as the last commit has it, or `None` when it is not in HEAD yet.
fn descriptor_at_head() -> Option<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root())
        .args(["show", &format!("HEAD:{DESCRIPTOR}")])
        .output()
        .expect("git runs; platform/tools/check_tracked.py needs it too");
    output.status.success().then_some(output.stdout)
}

#[test]
fn the_number_of_every_field_and_word_is_the_number_it_had_in_the_last_commit() {
    let Some(before) = descriptor_at_head() else {
        panic!(
            "{DESCRIPTOR} is not in HEAD, so there is nothing to hold this commit against. \
             Commit the descriptor before relying on this test."
        );
    };
    let old = numbering(&pool_from(&before));
    let new = numbering(&pool());

    let mut moved = Vec::new();
    for (path, number) in &old {
        match new.get(path) {
            Some(now) if now != number => moved.push(format!(
                "{path} was {number} and is now {now}. A field number is forever: every \
                 reader already deployed decodes {number} as {path}"
            )),
            Some(_) => {}
            // A removal is the other half of the same rule and belongs to
            // `buf breaking`; what matters here is that its number is not reused,
            // which the reuse pass below catches.
            None => {}
        }
    }

    // A number freed by a rename must not be handed to a different field of the
    // same message, which is renumbering wearing a second name.
    let owner = |path: &str| path.rsplit_once('.').map(|(head, _)| head.to_string());
    let mut taken: BTreeMap<(String, i64), String> = BTreeMap::new();
    for (path, number) in &old {
        if let Some(head) = owner(path) {
            taken.insert((head, *number), path.clone());
        }
    }
    for (path, number) in &new {
        let Some(head) = owner(path) else { continue };
        if let Some(was) = taken.get(&(head, *number))
            && was != path
            && !old.contains_key(path)
        {
            moved.push(format!(
                "{path} takes {number}, which {was} held in the last commit"
            ));
        }
    }

    moved.sort();
    assert!(
        moved.is_empty(),
        "{} number(s) moved between HEAD and this tree:\n  {}",
        moved.len(),
        moved.join("\n  ")
    );
}

#[test]
fn the_schema_is_not_numbered_in_declaration_order_and_must_never_be() {
    // The witness. If anyone ever replaces this file with an emitter that derives
    // numbers from the contract, this test is what has to be deleted first - and
    // deleting it is the visible act that deriving numbers really is.
    let pool = pool();
    let mut witnesses = Vec::new();
    for (name, words) in expected_enums() {
        let Some(item) = pool.get_enum_by_name(&name) else {
            continue;
        };
        let simple = name.rsplit('.').next().unwrap_or(&name);
        for (position, word) in words.iter().enumerate() {
            let wanted = value_name(simple, word);
            if let Some(value) = item.values().find(|value| value.name() == wanted)
                && i64::try_from(position + 1) != Ok(i64::from(value.number()))
            {
                witnesses.push(format!(
                    "{name}.{wanted} is {} and stands {} in the contract",
                    value.number(),
                    position + 1
                ));
            }
        }
    }
    for (name, expect) in expected_messages() {
        let Some(message) = pool.get_message_by_name(&name) else {
            continue;
        };
        for (position, field) in expect.fields.iter().enumerate() {
            if let Some(found) = message.get_field_by_name(&field.wire)
                && u32::try_from(position + 1) != Ok(found.number())
            {
                witnesses.push(format!(
                    "{name}.{} is {} and stands {} in the contract",
                    field.wire,
                    found.number(),
                    position + 1
                ));
            }
        }
    }
    assert!(
        !witnesses.is_empty(),
        "every number in the schema happens to equal its position in the contract. That \
         is a coincidence this test exists to notice losing, because the day it holds is \
         the day somebody believes numbers can be derived."
    );
    // `AttemptKind::Dash` is the one that cost something: declared second in
    // platform/contract/src/play.rs and numbered 4, because 2 is `File`'s forever.
    assert!(
        witnesses
            .iter()
            .any(|line| line.contains("AttemptKind.ATTEMPT_KIND_DASH is 4")),
        "AttemptKind.ATTEMPT_KIND_DASH should be 4 while standing 2nd in the contract. \
         Witnesses:\n  {}",
        witnesses.join("\n  ")
    );
}
