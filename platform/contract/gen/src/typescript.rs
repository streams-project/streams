//! TypeScript types, also read by ArkTS.

use std::fmt::Write;

use contract::version::{ABI_VERSION, WIRE_VERSION};

use crate::shape::{Field, Item, Shape, Ty};

fn ty(ty: &Ty) -> String {
    match ty {
        Ty::Text => "string".into(),
        Ty::Int => "number".into(),
        Ty::Bool => "boolean".into(),
        Ty::List(inner) => format!("{}[]", wrap(inner)),
        Ty::Map(inner) => format!("Record<string, {}>", self::ty(inner)),
        Ty::Named(name) => name.clone(),
    }
}

fn wrap(inner: &Ty) -> String {
    self::ty(inner)
}

fn comment(out: &mut String, indent: &str, doc: Option<&String>) {
    if let Some(doc) = doc {
        let _ = writeln!(out, "{indent}/** {} */", doc.replace("*/", "* /"));
    }
}

fn field_list(out: &mut String, indent: &str, fields: &[Field]) {
    for field in fields {
        comment(out, indent, field.doc.as_ref());
        let mark = if field.required { "" } else { "?" };
        let _ = writeln!(out, "{indent}{}{mark}: {};", field.wire, ty(&field.ty));
    }
}

/// The whole file.
#[must_use]
pub fn write(items: &[Item]) -> String {
    let mut out = String::from("// Generated from the contract. Do not edit.\n\n");
    let _ = writeln!(out, "export const WIRE_VERSION = \"{WIRE_VERSION}\";");
    let _ = writeln!(out, "export const ABI_VERSION = {ABI_VERSION};");
    for item in items {
        out.push('\n');
        comment(&mut out, "", item.doc.as_ref());
        match &item.shape {
            Shape::Struct(fields) => {
                let _ = writeln!(out, "export interface {} {{", item.name);
                field_list(&mut out, "  ", fields);
                out.push_str("}\n");
            }
            Shape::Words(words) => {
                let list: Vec<String> = words.iter().map(|word| format!("\"{word}\"")).collect();
                let _ = writeln!(
                    out,
                    "export type {} = {} | (string & {{}});",
                    item.name,
                    list.join(" | ")
                );
            }
            Shape::Fixed(word) => {
                let _ = writeln!(out, "export type {} = \"{word}\";", item.name);
            }
            Shape::Alias(inner) => {
                let _ = writeln!(out, "export type {} = {};", item.name, ty(inner));
            }
            Shape::Union {
                tag,
                common,
                variants,
            } => {
                let _ = writeln!(out, "export type {} =", item.name);
                for variant in variants {
                    let _ = writeln!(out, "  | {{");
                    comment(&mut out, "      ", variant.doc.as_ref());
                    let _ = writeln!(out, "      {tag}: \"{}\";", variant.word);
                    field_list(&mut out, "      ", common);
                    field_list(&mut out, "      ", &variant.fields);
                    let _ = writeln!(out, "    }}");
                }
                out.push_str("  ;\n");
            }
        }
    }
    out
}
