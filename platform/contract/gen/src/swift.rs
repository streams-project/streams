//! Swift types. Words stay open and a union keeps the tag of a variant it does not know.

use std::fmt::Write;

use contract::version::{ABI_VERSION, WIRE_VERSION};

use crate::shape::{Field, Item, Shape, Ty, lower_camel, upper_camel};

const RESERVED: &[&str] = &[
    "as",
    "case",
    "class",
    "continue",
    "default",
    "extension",
    "for",
    "func",
    "in",
    "init",
    "internal",
    "is",
    "let",
    "operator",
    "private",
    "protocol",
    "public",
    "repeat",
    "return",
    "self",
    "static",
    "struct",
    "switch",
    "true",
    "try",
    "var",
    "where",
    "while",
];

fn name(word: &str) -> String {
    let camel = lower_camel(word);
    if RESERVED.contains(&camel.as_str()) {
        format!("`{camel}`")
    } else {
        camel
    }
}

fn ty(ty: &Ty) -> String {
    match ty {
        Ty::Text => "String".into(),
        Ty::Int => "Int".into(),
        Ty::Bool => "Bool".into(),
        Ty::List(inner) => format!("[{}]", self::ty(inner)),
        Ty::Map(inner) => format!("[String: {}]", self::ty(inner)),
        Ty::Named(name) => name.clone(),
    }
}

fn field_ty(field: &Field) -> String {
    let base = ty(&field.ty);
    if field.required {
        base
    } else {
        format!("{base}?")
    }
}

fn comment(out: &mut String, indent: &str, doc: Option<&String>) {
    if let Some(doc) = doc {
        let _ = writeln!(out, "{indent}/// {doc}");
    }
}

fn plain_struct(out: &mut String, type_name: &str, fields: &[Field]) {
    let _ = writeln!(
        out,
        "public struct {type_name}: Codable, Equatable, Sendable {{"
    );
    for field in fields {
        comment(out, "    ", field.doc.as_ref());
        let _ = writeln!(
            out,
            "    public var {}: {}",
            name(&field.wire),
            field_ty(field)
        );
    }
    if !fields.is_empty() {
        out.push_str("\n    enum CodingKeys: String, CodingKey {\n");
        for field in fields {
            let _ = writeln!(
                out,
                "        case {} = \"{}\"",
                name(&field.wire),
                field.wire
            );
        }
        out.push_str("    }\n");
    }
    let params: Vec<String> = fields
        .iter()
        .map(|field| {
            let fallback = if field.required { "" } else { " = nil" };
            format!("{}: {}{fallback}", name(&field.wire), field_ty(field))
        })
        .collect();
    let _ = writeln!(out, "\n    public init({}) {{", params.join(", "));
    for field in fields {
        let bare = name(&field.wire);
        let _ = writeln!(out, "        self.{bare} = {bare}");
    }
    out.push_str("    }\n}\n");
}

fn words(out: &mut String, type_name: &str, list: &[String]) {
    let _ = writeln!(
        out,
        "public struct {type_name}: RawRepresentable, Codable, Hashable, Sendable {{"
    );
    out.push_str("    public var rawValue: String\n");
    out.push_str("    public init(rawValue: String) { self.rawValue = rawValue }\n");
    for word in list {
        let _ = writeln!(
            out,
            "    public static let {} = {type_name}(rawValue: \"{word}\")",
            name(word)
        );
    }
    out.push_str("}\n");
}

fn checked(out: &mut String, type_name: &str, pattern: &str) {
    let _ = writeln!(
        out,
        "public struct {type_name}: Codable, Hashable, Sendable {{"
    );
    out.push_str("    public let rawValue: String\n\n");
    out.push_str("    public init(_ rawValue: String) throws {\n");
    let _ = writeln!(
        out,
        "        guard rawValue.range(of: \"{pattern}\", options: .regularExpression) != nil else {{"
    );
    let _ = writeln!(
        out,
        "            throw ContractError.refused(\"{type_name}\")"
    );
    out.push_str("        }\n        self.rawValue = rawValue\n    }\n\n");
    out.push_str("    public init(from decoder: Decoder) throws {\n");
    out.push_str("        try self.init(try decoder.singleValueContainer().decode(String.self))\n");
    out.push_str("    }\n\n");
    out.push_str("    public func encode(to encoder: Encoder) throws {\n");
    out.push_str("        var single = encoder.singleValueContainer()\n");
    out.push_str("        try single.encode(rawValue)\n    }\n}\n");
}

fn union(out: &mut String, item: &Item, tag: &str, variants: &[crate::shape::Variant]) {
    let enum_name = union_name(item);
    for variant in variants {
        comment(out, "", variant.doc.as_ref());
        let type_name = format!("{}{}", item.name, upper_camel(&variant.word));
        plain_struct(out, &type_name, &variant.fields);
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "public enum {enum_name}: Codable, Equatable, Sendable {{"
    );
    for variant in variants {
        let _ = writeln!(
            out,
            "    case {}({}{})",
            name(&variant.word),
            item.name,
            upper_camel(&variant.word)
        );
    }
    out.push_str("    /// A tag this build does not know.\n    case unknown(String)\n\n");
    let _ = writeln!(
        out,
        "    private enum TagKey: String, CodingKey {{ case tag = \"{tag}\" }}\n"
    );
    out.push_str("    public init(from decoder: Decoder) throws {\n");
    out.push_str("        let tagged = try decoder.container(keyedBy: TagKey.self)\n");
    out.push_str("        let tag = try tagged.decode(String.self, forKey: .tag)\n");
    out.push_str("        switch tag {\n");
    for variant in variants {
        let _ = writeln!(
            out,
            "        case \"{}\": self = .{}(try {}{}(from: decoder))",
            variant.word,
            name(&variant.word),
            item.name,
            upper_camel(&variant.word)
        );
    }
    out.push_str("        default: self = .unknown(tag)\n        }\n    }\n\n");
    out.push_str("    public func encode(to encoder: Encoder) throws {\n");
    out.push_str("        var tagged = encoder.container(keyedBy: TagKey.self)\n");
    out.push_str("        switch self {\n");
    for variant in variants {
        let _ = writeln!(
            out,
            "        case .{}(let value):\n            try tagged.encode(\"{}\", forKey: .tag)\n            try value.encode(to: encoder)",
            name(&variant.word),
            variant.word
        );
    }
    out.push_str(
        "        case .unknown(let tag):\n            try tagged.encode(tag, forKey: .tag)\n",
    );
    out.push_str("        }\n    }\n}\n");
}

fn union_name(item: &Item) -> String {
    match &item.shape {
        Shape::Union { common, .. } if !common.is_empty() => format!("{}Body", item.name),
        _ => item.name.clone(),
    }
}

/// A union with common fields becomes a struct around the union of its variants.
fn wrapper(out: &mut String, item: &Item, common: &[Field]) {
    let _ = writeln!(
        out,
        "public struct {}: Codable, Equatable, Sendable {{",
        item.name
    );
    for field in common {
        comment(out, "    ", field.doc.as_ref());
        let _ = writeln!(
            out,
            "    public var {}: {}",
            name(&field.wire),
            field_ty(field)
        );
    }
    let _ = writeln!(out, "    public var body: {}Body\n", item.name);
    out.push_str("    private enum CodingKeys: String, CodingKey {\n");
    for field in common {
        let _ = writeln!(
            out,
            "        case {} = \"{}\"",
            name(&field.wire),
            field.wire
        );
    }
    out.push_str("    }\n\n");
    let params: Vec<String> = common
        .iter()
        .map(|field| {
            let fallback = if field.required { "" } else { " = nil" };
            format!("{}: {}{fallback}", name(&field.wire), field_ty(field))
        })
        .collect();
    let _ = writeln!(
        out,
        "    public init({}, body: {}Body) {{",
        params.join(", "),
        item.name
    );
    for field in common {
        let bare = name(&field.wire);
        let _ = writeln!(out, "        self.{bare} = {bare}");
    }
    out.push_str("        self.body = body\n    }\n\n");
    out.push_str("    public init(from decoder: Decoder) throws {\n");
    out.push_str("        let keyed = try decoder.container(keyedBy: CodingKeys.self)\n");
    for field in common {
        let bare = name(&field.wire);
        let method = if field.required {
            "decode"
        } else {
            "decodeIfPresent"
        };
        let _ = writeln!(
            out,
            "        \
             self.{bare} = try keyed.{method}({}.self, forKey: .{bare})",
            ty(&field.ty)
        );
    }
    let _ = writeln!(
        out,
        "        self.body = try {}Body(from: decoder)",
        item.name
    );
    out.push_str("    }\n\n");
    out.push_str("    public func encode(to encoder: Encoder) throws {\n");
    out.push_str("        var keyed = encoder.container(keyedBy: CodingKeys.self)\n");
    for field in common {
        let bare = name(&field.wire);
        let method = if field.required {
            "encode"
        } else {
            "encodeIfPresent"
        };
        let _ = writeln!(out, "        try keyed.{method}({bare}, forKey: .{bare})");
    }
    out.push_str("        try body.encode(to: encoder)\n    }\n}\n");
}

/// The whole file.
#[must_use]
pub fn write(items: &[Item]) -> String {
    let mut out = String::from("// Generated from the contract. Do not edit.\n\n");
    out.push_str("import Foundation\n\n");
    let _ = writeln!(out, "public let wireVersion = \"{WIRE_VERSION}\"");
    let _ = writeln!(out, "public let abiVersion = {ABI_VERSION}");
    out.push_str("\n/// A value the contract refuses, named by its type.\n");
    out.push_str("public enum ContractError: Error { case refused(String) }\n");
    for item in items {
        out.push('\n');
        comment(&mut out, "", item.doc.as_ref());
        match &item.shape {
            Shape::Struct(fields) => plain_struct(&mut out, &item.name, fields),
            Shape::Words(list) => words(&mut out, &item.name, list),
            Shape::Fixed(word) => {
                let _ = writeln!(
                    out,
                    "public enum {}: String, Codable, Sendable {{ case value = \"{word}\" }}",
                    item.name
                );
            }
            Shape::Checked(pattern) => checked(&mut out, &item.name, pattern),
            Shape::Alias(inner) => {
                let _ = writeln!(out, "public typealias {} = {}", item.name, ty(inner));
            }
            Shape::Union {
                tag,
                common,
                variants,
            } => {
                if !common.is_empty() {
                    wrapper(&mut out, item, common);
                    out.push('\n');
                }
                union(&mut out, item, tag, variants);
            }
        }
    }
    out
}
