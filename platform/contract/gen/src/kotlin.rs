//! Kotlin types for kotlinx.serialization. Read with `contractJson`, which ignores unknown fields
//! and turns a tag this build does not know into the `Unknown` variant.

use std::fmt::Write;

use contract::version::{ABI_VERSION, WIRE_VERSION};

use crate::shape::{Field, Item, Shape, Ty, lower_camel, upper_camel};

const RESERVED: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
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
        Ty::Int => "Long".into(),
        Ty::Bool => "Boolean".into(),
        Ty::List(inner) => format!("List<{}>", self::ty(inner)),
        Ty::Map(inner) => format!("Map<String, {}>", self::ty(inner)),
        Ty::Named(name) => name.clone(),
    }
}

fn comment(out: &mut String, indent: &str, doc: Option<&String>) {
    if let Some(doc) = doc {
        let _ = writeln!(out, "{indent}/** {} */", doc.replace("*/", "* /"));
    }
}

fn parameter(out: &mut String, field: &Field, overriding: bool) {
    comment(out, "    ", field.doc.as_ref());
    let lead = if overriding { "override val" } else { "val" };
    let tail = if field.required { "" } else { "? = null" };
    let _ = writeln!(
        out,
        "    @SerialName(\"{}\") {lead} {}: {}{tail},",
        field.wire,
        name(&field.wire),
        ty(&field.ty)
    );
}

fn data_class(out: &mut String, head: &str, common: &[Field], fields: &[Field], tail: &str) {
    if common.is_empty() && fields.is_empty() {
        let _ = writeln!(out, "{head}{tail}");
        return;
    }
    let _ = writeln!(out, "{head}(");
    for field in common {
        parameter(out, field, true);
    }
    for field in fields {
        parameter(out, field, false);
    }
    let _ = writeln!(out, "){tail}");
}

fn words(out: &mut String, type_name: &str, list: &[String]) {
    let _ = writeln!(out, "@Serializable\n@JvmInline");
    let _ = writeln!(out, "value class {type_name}(val value: String) {{");
    out.push_str("    companion object {\n");
    for word in list {
        let constant = word.replace('-', "_").to_uppercase();
        let _ = writeln!(out, "        val {constant} = {type_name}(\"{word}\")");
    }
    out.push_str("    }\n}\n");
}

fn checked(out: &mut String, type_name: &str, pattern: &str) {
    let escaped = pattern.replace('\\', "\\\\").replace('$', "\\$");
    let _ = writeln!(out, "@Serializable\n@JvmInline");
    let _ = writeln!(out, "value class {type_name}(val value: String) {{");
    let _ = writeln!(
        out,
        "    init {{\n        require(Regex(\"{escaped}\").matches(value)) {{ \"not a {type_name}\" }}\n    }}\n}}"
    );
}

fn union(
    out: &mut String,
    item: &Item,
    tag: &str,
    common: &[Field],
    variants: &[crate::shape::Variant],
) {
    let _ = writeln!(out, "@Serializable\n@JsonClassDiscriminator(\"{tag}\")");
    if common.is_empty() {
        let _ = writeln!(out, "sealed interface {}", item.name);
    } else {
        let _ = writeln!(out, "sealed interface {} {{", item.name);
        for field in common {
            let tail = if field.required { "" } else { "?" };
            let _ = writeln!(
                out,
                "    val {}: {}{tail}",
                name(&field.wire),
                ty(&field.ty)
            );
        }
        out.push_str("}\n");
    }
    for variant in variants {
        out.push('\n');
        comment(out, "", variant.doc.as_ref());
        let type_name = format!("{}{}", item.name, upper_camel(&variant.word));
        let _ = writeln!(out, "@Serializable\n@SerialName(\"{}\")", variant.word);
        let empty = common.is_empty() && variant.fields.is_empty();
        let head = if empty {
            format!("data object {type_name}")
        } else {
            format!("data class {type_name}")
        };
        data_class(
            out,
            &head,
            common,
            &variant.fields,
            &format!(" : {}", item.name),
        );
    }
    out.push_str("\n/** A tag this build does not know. */\n@Serializable\n");
    let head = if common.is_empty() {
        format!("data object {}Unknown", item.name)
    } else {
        format!("data class {}Unknown", item.name)
    };
    let optional: Vec<Field> = common
        .iter()
        .map(|field| Field {
            required: false,
            ..field.clone()
        })
        .collect();
    let tail = format!(" : {}", item.name);
    if common.is_empty() {
        let _ = writeln!(out, "{head}{tail}");
    } else {
        let _ = writeln!(out, "{head}(");
        for field in &optional {
            comment(out, "    ", field.doc.as_ref());
            let _ = writeln!(
                out,
                "    @SerialName(\"{}\") val raw{}: {}? = null,",
                field.wire,
                upper_camel(&field.wire),
                ty(&field.ty)
            );
        }
        let _ = writeln!(out, "){tail} {{");
        for field in common {
            let bang = if field.required { " ?: \"\"" } else { "" };
            let tail = if field.required { "" } else { "?" };
            let _ = writeln!(
                out,
                "    override val {}: {}{tail} get() = raw{}{bang}",
                name(&field.wire),
                ty(&field.ty),
                upper_camel(&field.wire)
            );
        }
        out.push_str("}\n");
    }
}

/// The whole file.
#[must_use]
pub fn write(items: &[Item]) -> String {
    let mut out = String::from("// Generated from the contract. Do not edit.\n\n");
    out.push_str("@file:OptIn(ExperimentalSerializationApi::class)\n\n");
    out.push_str("package contract\n\n");
    out.push_str("import kotlinx.serialization.ExperimentalSerializationApi\n");
    out.push_str("import kotlinx.serialization.SerialName\n");
    out.push_str("import kotlinx.serialization.Serializable\n");
    out.push_str("import kotlinx.serialization.json.Json\n");
    out.push_str("import kotlinx.serialization.json.JsonClassDiscriminator\n");
    out.push_str("import kotlinx.serialization.modules.SerializersModule\n");
    out.push_str("import kotlinx.serialization.modules.polymorphic\n\n");
    let _ = writeln!(out, "const val WIRE_VERSION = \"{WIRE_VERSION}\"");
    let _ = writeln!(out, "const val ABI_VERSION = {ABI_VERSION}\n");
    let _ = writeln!(
        out,
        "/** Every address the apps open. */\nobject Address {{"
    );
    for (name, value) in contract::address::ALL {
        let _ = writeln!(out, "    const val {name} = \"{value}\"");
    }
    let _ = writeln!(out, "\n    /** Where a tab lives, by the screen's word. */");
    let _ = writeln!(out, "    fun of(screen: String): String = when (screen) {{");
    for (name, value) in contract::address::SCREENS {
        let _ = writeln!(out, "        \"{name}\" -> \"{value}\"");
    }
    let _ = writeln!(out, "        else -> HOME\n    }}\n}}\n");
    out.push_str(
        "/** Ignores unknown fields and reads an unknown tag as the Unknown variant. */\n",
    );
    out.push_str(
        "val contractJson = Json {\n    ignoreUnknownKeys = true\n    explicitNulls = false\n",
    );
    out.push_str("    serializersModule = SerializersModule {\n");
    for item in items {
        if matches!(item.shape, Shape::Union { .. }) {
            let _ = writeln!(
                out,
                "        polymorphic({0}::class) {{ defaultDeserializer {{ {0}Unknown.serializer() }} }}",
                item.name
            );
        }
    }
    out.push_str("    }\n}\n");
    for item in items {
        out.push('\n');
        comment(&mut out, "", item.doc.as_ref());
        match &item.shape {
            Shape::Struct(fields) => {
                out.push_str("@Serializable\n");
                if fields.is_empty() {
                    let _ = writeln!(out, "class {}", item.name);
                } else {
                    let head = format!("data class {}", item.name);
                    data_class(&mut out, &head, &[], fields, "");
                }
            }
            Shape::Words(list) => words(&mut out, &item.name, list),
            Shape::Fixed(word) => {
                let _ = writeln!(
                    out,
                    "@Serializable\nenum class {} {{\n    @SerialName(\"{word}\")\n    VALUE,\n}}",
                    item.name
                );
            }
            Shape::Checked(pattern) => checked(&mut out, &item.name, pattern),
            Shape::Alias(inner) => {
                let _ = writeln!(out, "typealias {} = {}", item.name, ty(inner));
            }
            Shape::Union {
                tag,
                common,
                variants,
            } => union(&mut out, item, tag, common, variants),
        }
    }
    out
}
