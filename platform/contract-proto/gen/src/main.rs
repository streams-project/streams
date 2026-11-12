//! Writes `generated/validate.rs` from the field options in the schema, and
//! `generated/rules.json` for the ledger tie check.
//!
//! The point of this program: the rule that refused `movie:2147483648` in Rust and
//! accepted it in Kotlin was written twice. Here it is written once, as a field
//! option, and every language's check is emitted from the descriptor. Nobody writes
//! a bound or a presence check by hand again.
//!
//! The rule that makes that true is in `rule_for`: an option this program cannot
//! emit is a HARD ERROR, never a skip. A generator that quietly ignores what it
//! does not understand recreates the defect it was built to close.
//!
//!     cargo run --offline -- write     # writes the files
//!     cargo run --offline -- check     # compares, writes nothing, exits 1 on drift
//!
//! `./check` runs `write` and then `git diff --exit-code`, which is this
//! repository's existing habit for generated files.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use prost_reflect::{
    Cardinality, ExtensionDescriptor, FieldDescriptor, Kind, MessageDescriptor, OneofDescriptor,
    Value,
};

/// The package whose messages get a validator. Anything else in the pool
/// (google.protobuf, the options themselves) is vocabulary, not a wire type.
const PACKAGE: &str = "contract.v1";

/// The one option. Its full name, so a rename in the schema fails here loudly.
const EXTENSION: &str = "contract.constraints.v1.field";

/// `ValueRule` numbers this generator can emit. One, today.
const VALUE_CALENDAR_DAY: i32 = 1;

/// What one field option says, read out of the descriptor.
#[derive(Default, Debug, Clone)]
struct Rule {
    required: bool,
    min: Option<u64>,
    max: Option<u64>,
    closed: bool,
    value: i32,
    presence_lost: bool,
}

impl Rule {
    /// A rule that produces a line of Rust. `presence_lost` alone does not: it is a
    /// declared loss, and the generator's refusal to emit anything for it is the
    /// point.
    fn enforceable(&self) -> bool {
        self.required || self.closed || self.min.is_some() || self.max.is_some() || self.value != 0
    }

    fn is_empty(&self) -> bool {
        !self.enforceable() && !self.presence_lost
    }
}

/// Counts the generator reports, so the schema's rules and losses are numbers in a
/// file rather than a claim in a comment.
#[derive(Default)]
struct Counts {
    messages: usize,
    validators: usize,
    fields: usize,
    required: usize,
    bounds: usize,
    closed: usize,
    value: usize,
    presence_lost: usize,
}

fn main() {
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "check".to_string());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("gen/ sits inside the crate")
        .to_path_buf();

    // The same list build.rs reads, so the descriptor this program validates is
    // the descriptor the crate compiled.
    let text = std::fs::read_to_string(root.join("proto/FILES")).expect("proto/FILES is readable");
    let files: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    // The Compiler, not `protox::compile`: only this path keeps the custom options.
    // `prost_types::FieldOptions` is a prost struct, prost drops unknown fields on
    // decode, and a FileDescriptorSet that has been through it has lost every
    // option this program exists to read.
    let mut compiler = match protox::Compiler::new([root.join("proto")]) {
        Ok(compiler) => compiler,
        Err(error) => {
            eprintln!("the include path does not open:\n{error}");
            std::process::exit(2);
        }
    };
    compiler.include_imports(true);
    if let Err(error) = compiler.open_files(&files) {
        eprintln!("the schema does not compile:\n{error}");
        std::process::exit(2);
    }
    let pool = compiler.descriptor_pool();
    let extension = pool
        .get_extension_by_name(EXTENSION)
        .unwrap_or_else(|| panic!("the option {EXTENSION} is not in the schema"));

    let mut messages: Vec<MessageDescriptor> = pool
        .all_messages()
        .filter(|m| m.package_name() == PACKAGE && !m.is_map_entry())
        .collect();
    messages.sort_by(|a, b| a.full_name().cmp(b.full_name()));

    // Which messages need a validator: the ones with an enforceable rule, plus
    // everything that can reach one. Fixed point, so a rule three messages deep
    // still refuses at the top - which is what refused/Screen.bare_id_in_a_tile.json
    // tests.
    let needs = needs_validator(&messages, &extension);

    let mut counts = Counts::default();
    let mut bodies = String::new();
    let mut index: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for message in &messages {
        counts.messages += 1;
        if !needs.contains(message.full_name()) {
            // Still walk the fields, so the option checks and the counts see every
            // field in the contract and not only the ones inside a validator.
            for field in message.fields() {
                counts.fields += 1;
                let rule = rule_for(&field, &extension);
                if rule.presence_lost {
                    counts.presence_lost += 1;
                    index
                        .entry(field.full_name().to_string())
                        .or_default()
                        .push("presence_lost".to_string());
                }
            }
            continue;
        }
        counts.validators += 1;
        bodies.push_str(&emit_message(
            message,
            &extension,
            &needs,
            &mut counts,
            &mut index,
        ));
    }

    let validate = render(&bodies, &counts);
    let rules = render_index(&index);
    let validate_path = root.join("generated/validate.rs");
    let rules_path = root.join("generated/rules.json");

    match mode.as_str() {
        "write" => {
            std::fs::write(&validate_path, &validate).expect("generated/validate.rs is writable");
            std::fs::write(&rules_path, &rules).expect("generated/rules.json is writable");
            println!("wrote {}", validate_path.display());
            println!("wrote {}", rules_path.display());
        }
        "check" => {
            compare(&validate_path, &validate);
            compare(&rules_path, &rules);
            println!("generated files are current");
        }
        _ => {
            eprintln!("usage: contract-validate-gen [write|check]");
            std::process::exit(2);
        }
    }

    println!(
        "messages {}  validators {}  fields {}",
        counts.messages, counts.validators, counts.fields
    );
    println!(
        "rules: required {}  bounds {}  closed {}  value {}",
        counts.required, counts.bounds, counts.closed, counts.value
    );
    println!(
        "presence lost: {} repeated fields refuse absence today and cannot on this wire",
        counts.presence_lost
    );
}

fn compare(path: &Path, want: &str) {
    match std::fs::read_to_string(path) {
        Ok(have) if have == want => {}
        Ok(_) => {
            eprintln!(
                "{} is out of date: run `cargo run -- write`",
                path.display()
            );
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("{} is missing: run `cargo run -- write`", path.display());
            std::process::exit(1);
        }
    }
}

/// Reads the one option off a field. An option the generator cannot enforce is a
/// hard error, which is the whole reason the option set is ours and closed.
fn rule_for(field: &FieldDescriptor, extension: &ExtensionDescriptor) -> Rule {
    let options = field.options();
    if !options.has_extension(extension) {
        return Rule::default();
    }
    let declared = options.get_extension(extension);
    let Value::Message(declared) = declared.as_ref() else {
        die(field, "the option is not a message, which cannot happen")
    };

    let mut rule = Rule::default();
    for (descriptor, item) in declared.fields() {
        match descriptor.name() {
            "required" => rule.required = item.as_bool().unwrap_or(false),
            "min" => rule.min = item.as_u64(),
            "max" => rule.max = item.as_u64(),
            "closed" => rule.closed = item.as_bool().unwrap_or(false),
            "value" => rule.value = item.as_enum_number().unwrap_or(0),
            "presence_lost" => rule.presence_lost = item.as_bool().unwrap_or(false),
            other => die(
                field,
                &format!(
                    "the option field `{other}` was added to FieldRule and this generator \
                     does not emit it. Add it here or take it out of the schema: a rule the \
                     generator ignores is a rule the core does not enforce, which is the \
                     defect this package closes."
                ),
            ),
        }
    }
    if rule.value != 0 && rule.value != VALUE_CALENDAR_DAY {
        die(
            field,
            &format!(
                "value rule number {} has no implementation. Every ValueRule needs one \
                 predicate in src/rules.rs and one arm here.",
                rule.value
            ),
        );
    }
    rule
}

fn die(field: &FieldDescriptor, message: &str) -> ! {
    eprintln!("{}: {message}", field.full_name());
    std::process::exit(2);
}

/// Messages with an enforceable rule, plus every message that can reach one.
fn needs_validator(
    messages: &[MessageDescriptor],
    extension: &ExtensionDescriptor,
) -> BTreeSet<String> {
    let mut needs: BTreeSet<String> = BTreeSet::new();
    for message in messages {
        for field in message.fields() {
            if rule_for(&field, extension).enforceable() {
                needs.insert(message.full_name().to_string());
                break;
            }
        }
    }
    loop {
        let mut grew = false;
        for message in messages {
            if needs.contains(message.full_name()) {
                continue;
            }
            for field in message.fields() {
                if field.is_map() {
                    continue;
                }
                if let Kind::Message(child) = field.kind()
                    && needs.contains(child.full_name())
                {
                    needs.insert(message.full_name().to_string());
                    grew = true;
                    break;
                }
            }
        }
        if !grew {
            return needs;
        }
    }
}

/// The Rust path of a message, as prost names it: a nested message lives in a
/// module named after its parent, in snake case.
fn rust_path(message: &MessageDescriptor) -> String {
    let parts = relative(message);
    let mut path = String::from("c::");
    for parent in &parts[..parts.len() - 1] {
        path.push_str(&snake(parent));
        path.push_str("::");
    }
    path.push_str(parts[parts.len() - 1]);
    path
}

/// The Rust path of a oneof's generated enum: prost puts it in the module named
/// after the message, under the oneof's own name in PascalCase. `Event.body`
/// becomes `c::event::Body`, `DriverEvent.kind` becomes `c::driver_event::Kind`.
fn union_path(message: &MessageDescriptor, union: &OneofDescriptor) -> String {
    let mut path = String::from("c::");
    for part in relative(message) {
        path.push_str(&snake(part));
        path.push_str("::");
    }
    path.push_str(&pascal(union.name()));
    path
}

/// The validator's name: the message path, flattened, in snake case.
fn fn_name(message: &MessageDescriptor) -> String {
    relative(message)
        .iter()
        .map(|part| snake(part))
        .collect::<Vec<_>>()
        .join("_")
}

fn relative(message: &MessageDescriptor) -> Vec<&str> {
    message
        .full_name()
        .strip_prefix(&format!("{PACKAGE}."))
        .expect("message is in the contract package")
        .split('.')
        .collect()
}

fn snake(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (index, ch) in chars.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            let previous_lower = index > 0
                && (chars[index - 1].is_ascii_lowercase() || chars[index - 1].is_ascii_digit());
            let previous_upper = index > 0 && chars[index - 1].is_ascii_uppercase();
            let next_lower = index + 1 < chars.len() && chars[index + 1].is_ascii_lowercase();
            if previous_lower || (previous_upper && next_lower) {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(*ch);
        }
    }
    out
}

fn pascal(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = true;
    for ch in name.chars() {
        if ch == '_' {
            upper = true;
        } else if upper {
            out.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// A oneof protobuf invented for a proto3 `optional` field, which must not be read
/// as a union. prost-reflect has no `is_synthetic`, and the generated name is
/// always an underscore followed by the field's own name.
fn is_synthetic(oneof: &OneofDescriptor) -> bool {
    let fields: Vec<FieldDescriptor> = oneof.fields().collect();
    fields.len() == 1 && oneof.name() == format!("_{}", fields[0].name())
}

/// Rust keywords a field of this contract could collide with. prost escapes them
/// with `r#`; the list is short because the contract's field names are words.
fn field_ident(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "type", "match", "move", "ref", "box", "in", "fn", "mod", "as", "if", "for", "loop",
    ];
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

fn emit_message(
    message: &MessageDescriptor,
    extension: &ExtensionDescriptor,
    needs: &BTreeSet<String>,
    counts: &mut Counts,
    index: &mut BTreeMap<String, Vec<String>>,
) -> String {
    let unions: Vec<OneofDescriptor> = message.oneofs().filter(|o| !is_synthetic(o)).collect();
    let union_members: BTreeSet<String> = unions
        .iter()
        .flat_map(|o| o.fields().map(|f| f.name().to_string()).collect::<Vec<_>>())
        .collect();

    let mut lines = String::new();
    for field in message.fields() {
        counts.fields += 1;
        if union_members.contains(field.name()) {
            continue;
        }
        let rule = rule_for(&field, extension);
        lines.push_str(&emit_field(&field, &rule, needs, counts, index));
    }

    for union in &unions {
        let mut arms = String::new();
        for arm in union.fields() {
            let Kind::Message(child) = arm.kind() else {
                die(&arm, "a union arm must be a message in this contract")
            };
            let rule = rule_for(&arm, extension);
            if !rule.is_empty() {
                die(
                    &arm,
                    "a rule on a union arm: presence is the oneof's job, so there is nothing \
                     for this option to say",
                );
            }
            if needs.contains(child.full_name()) {
                let _ = writeln!(
                    arms,
                    "            {}::{}(inner) => {}(inner)?,",
                    union_path(message, union),
                    pascal(arm.name()),
                    fn_name(&child)
                );
            }
        }
        if !arms.is_empty() {
            let _ = writeln!(
                lines,
                "    if let Some(arm) = &m.{} {{\n        #[allow(unreachable_patterns)]\n        match arm {{\n{arms}            _ => {{}}\n        }}\n    }}",
                field_ident(union.name())
            );
        }
    }

    format!(
        "/// `{}`.\n///\n/// # Errors\n/// The first rule the message breaks, naming the field.\npub fn {}(m: &{}) -> Result<(), Refused> {{\n{lines}    Ok(())\n}}\n\n",
        message.full_name(),
        fn_name(message),
        rust_path(message)
    )
}

fn emit_field(
    field: &FieldDescriptor,
    rule: &Rule,
    needs: &BTreeSet<String>,
    counts: &mut Counts,
    index: &mut BTreeMap<String, Vec<String>>,
) -> String {
    let path = field.full_name().to_string();
    let name = field_ident(field.name());
    let mut out = String::new();
    let note = |kind: &str, index: &mut BTreeMap<String, Vec<String>>| {
        index
            .entry(path.clone())
            .or_default()
            .push(kind.to_string());
    };

    // A map field. proto3 gives it no presence and this option set has no rule for
    // one; a map that must be present takes a wrapper message instead.
    if field.is_map() {
        if !rule.is_empty() {
            die(
                field,
                "a map field can carry no rule: proto3 forbids `optional` on a map and a map \
                 has no presence. Wrap it in a message, as Headers and TextMap do.",
            );
        }
        if let Kind::Message(entry) = field.kind()
            && let Some(value) = entry.get_field_by_name("value")
            && matches!(value.kind(), Kind::Message(_))
        {
            die(
                field,
                "a map with message values needs a loop this generator does not write",
            );
        }
        return out;
    }

    // A repeated field.
    if field.cardinality() == Cardinality::Repeated {
        if rule.required {
            die(
                field,
                "`required` on a repeated field is a lie: proto3 has no presence for one, so \
                 absent and empty are the same bytes. Declare `presence_lost: true` instead.",
            );
        }
        if rule.closed || rule.min.is_some() || rule.max.is_some() || rule.value != 0 {
            die(
                field,
                "this generator writes no element-wise rule, and nothing in the contract \
                 needs one",
            );
        }
        if rule.presence_lost {
            counts.presence_lost += 1;
            note("presence_lost", index);
        }
        if let Kind::Message(child) = field.kind() {
            if child.package_name() != PACKAGE {
                die(field, "a repeated field of a foreign package");
            }
            if needs.contains(child.full_name()) {
                let _ = writeln!(
                    out,
                    "    for item in &m.{name} {{\n        {}(item)?;\n    }}",
                    fn_name(&child)
                );
            }
        }
        return out;
    }

    // A singular message field. prost gives it `Option<T>` whether or not the
    // schema writes `optional`, because a message always has presence.
    if let Kind::Message(child) = field.kind() {
        if child.package_name() != PACKAGE {
            die(field, "a message field of a foreign package");
        }
        if rule.closed || rule.min.is_some() || rule.max.is_some() || rule.value != 0 {
            die(field, "a bound or an enum rule on a message field");
        }
        let recurse = needs.contains(child.full_name());
        if rule.required {
            counts.required += 1;
            note("required", index);
            let _ = write!(
                out,
                "    match &m.{name} {{\n        None => return Err(Refused::new(Rule::Required, \"{path}\")),\n"
            );
            if recurse {
                let _ = writeln!(out, "        Some(inner) => {}(inner)?,", fn_name(&child));
            } else {
                let _ = writeln!(out, "        Some(_) => {{}}");
            }
            let _ = writeln!(out, "    }}");
            return out;
        }
        if recurse {
            let _ = writeln!(
                out,
                "    if let Some(inner) = &m.{name} {{\n        {}(inner)?;\n    }}",
                fn_name(&child)
            );
        }
        return out;
    }

    // A singular enum field.
    if let Kind::Enum(enumeration) = field.kind() {
        if rule.min.is_some() || rule.max.is_some() || rule.value != 0 {
            die(field, "a numeric bound or a value rule on an enum field");
        }
        let pattern = enumeration
            .values()
            .map(|value| value.number())
            .filter(|number| *number != 0)
            .map(|number| number.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        let presence = field.supports_presence();
        if rule.required && !presence {
            die(
                field,
                "`required` on a field with no presence. Write `optional` in the schema, or \
                 drop the rule.",
            );
        }
        if rule.required {
            counts.required += 1;
            note("required", index);
        }
        if rule.closed {
            counts.closed += 1;
            note("closed", index);
        }
        match (presence, rule.required, rule.closed) {
            (true, true, true) => {
                let _ = writeln!(
                    out,
                    "    match m.{name} {{\n        None => return Err(Refused::new(Rule::Required, \"{path}\")),\n        Some(value) if !matches!(value, {pattern}) => {{\n            return Err(Refused::new(Rule::Closed, \"{path}\"));\n        }}\n        Some(_) => {{}}\n    }}"
                );
            }
            (true, true, false) => {
                let _ = writeln!(
                    out,
                    "    if m.{name}.is_none() {{\n        return Err(Refused::new(Rule::Required, \"{path}\"));\n    }}"
                );
            }
            (true, false, true) => {
                let _ = writeln!(
                    out,
                    "    if let Some(value) = m.{name} {{\n        if !matches!(value, {pattern}) {{\n            return Err(Refused::new(Rule::Closed, \"{path}\"));\n        }}\n    }}"
                );
            }
            (false, false, true) => {
                let _ = writeln!(
                    out,
                    "    if !matches!(m.{name}, {pattern}) {{\n        return Err(Refused::new(Rule::Closed, \"{path}\"));\n    }}"
                );
            }
            _ => {}
        }
        return out;
    }

    // A singular scalar.
    let numeric = matches!(
        field.kind(),
        Kind::Uint32 | Kind::Uint64 | Kind::Int32 | Kind::Int64 | Kind::Fixed32 | Kind::Fixed64
    );
    if (rule.min.is_some() || rule.max.is_some() || rule.value != 0) && !numeric {
        die(
            field,
            "a numeric bound or a value rule on a field that is not a whole number",
        );
    }
    if rule.closed {
        die(field, "`closed` on a field that is not an enum");
    }
    let presence = field.supports_presence();
    if rule.required && !presence {
        die(
            field,
            "`required` on a field with no presence. Write `optional` in the schema, or drop \
             the rule.",
        );
    }

    let mut checks = String::new();
    if let Some(min) = rule.min {
        counts.bounds += 1;
        note("min", index);
        let _ = write!(
            checks,
            "        if value < {min} {{\n            return Err(Refused::new(Rule::Min, \"{path}\"));\n        }}\n"
        );
    }
    if let Some(max) = rule.max {
        counts.bounds += 1;
        note("max", index);
        let _ = write!(
            checks,
            "        if value > {max} {{\n            return Err(Refused::new(Rule::Max, \"{path}\"));\n        }}\n"
        );
    }
    if rule.value == VALUE_CALENDAR_DAY {
        counts.value += 1;
        note("calendar_day", index);
        let _ = write!(
            checks,
            "        if !is_calendar_day(value) {{\n            return Err(Refused::new(Rule::CalendarDay, \"{path}\"));\n        }}\n"
        );
    }

    if presence {
        if rule.required {
            counts.required += 1;
            note("required", index);
            if checks.is_empty() {
                // `is_none` and not a match: `match m.field` on an `Option<String>`
                // behind a shared reference would try to move the String out.
                let _ = writeln!(
                    out,
                    "    if m.{name}.is_none() {{\n        return Err(Refused::new(Rule::Required, \"{path}\"));\n    }}"
                );
            } else {
                // Only a whole number can carry a bound or a value rule, and those
                // are Copy, so matching by value is sound here.
                let _ = writeln!(
                    out,
                    "    match m.{name} {{\n        None => return Err(Refused::new(Rule::Required, \"{path}\")),\n        Some(value) => {{\n{}        }}\n    }}",
                    indent(&checks)
                );
            }
            return out;
        }
        if !checks.is_empty() {
            let _ = writeln!(out, "    if let Some(value) = m.{name} {{\n{checks}    }}");
        }
        return out;
    }

    if !checks.is_empty() {
        let _ = writeln!(out, "    {{\n        let value = m.{name};\n{checks}    }}");
    }
    out
}

fn indent(text: &str) -> String {
    text.lines().map(|line| format!("    {line}\n")).collect()
}

fn render(bodies: &str, counts: &Counts) -> String {
    format!(
        "// Generated by `cargo run -p contract-validate-gen -- write`. DO NOT EDIT.\n\
         //\n\
         // One validator per message that carries a rule, or that holds something which\n\
         // does. Every line below comes from a field option in proto/contract/v1, so no\n\
         // rule can be stated in one language and not another.\n\
         //\n\
         // {} messages in the contract, {} with a validator, {} fields read.\n\
         // Rules: required {}, bounds {}, closed {}, value {}.\n\
         // Declared losses: {} repeated fields refuse absence today and cannot on a\n\
         // proto3 wire.\n\
         \n\
         use crate::contract as c;\n\
         use crate::rules::{{is_calendar_day, Refused, Rule}};\n\
         \n{bodies}",
        counts.messages,
        counts.validators,
        counts.fields,
        counts.required,
        counts.bounds,
        counts.closed,
        counts.value,
        counts.presence_lost,
    )
}

fn render_index(index: &BTreeMap<String, Vec<String>>) -> String {
    let mut out = String::from("{\n");
    for (position, (path, rules)) in index.iter().enumerate() {
        if position > 0 {
            out.push_str(",\n");
        }
        let list: Vec<String> = rules.iter().map(|rule| format!("\"{rule}\"")).collect();
        let _ = write!(out, "  \"{path}\": [{}]", list.join(", "));
    }
    out.push_str("\n}\n");
    out
}
