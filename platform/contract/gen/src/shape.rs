//! One small model of the definitions, shared by the three type writers.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::schemas::DEFS_PATH;

/// The type of a field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    Text,
    Int,
    Bool,
    List(Box<Self>),
    Map(Box<Self>),
    Named(String),
}

/// One field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub wire: String,
    pub ty: Ty,
    pub required: bool,
    pub doc: Option<String>,
}

/// One variant of a tagged union.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub word: String,
    pub fields: Vec<Field>,
    pub doc: Option<String>,
}

/// What a definition is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Plain fields.
    Struct(Vec<Field>),
    /// Plain words. Open: a client keeps any word it does not know.
    Words(Vec<String>),
    /// One fixed word, the type tag of a core message.
    Fixed(String),
    /// Variants told apart by `tag`, each with the `common` fields.
    Union {
        tag: String,
        common: Vec<Field>,
        variants: Vec<Variant>,
    },
    /// Another name for a type.
    Alias(Ty),
}

/// One definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub doc: Option<String>,
    pub shape: Shape,
}

fn doc(schema: &Value) -> Option<String> {
    schema
        .get("description")
        .and_then(Value::as_str)
        .map(|text| text.replace('\n', " "))
}

fn ty(schema: &Value) -> Ty {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let prefix = format!("#{DEFS_PATH}/");
        return Ty::Named(reference.trim_start_matches(&prefix).to_string());
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("integer") => Ty::Int,
        Some("boolean") => Ty::Bool,
        Some("array") => Ty::List(Box::new(ty(&schema["items"]))),
        Some("object") => Ty::Map(Box::new(ty(&schema["additionalProperties"]))),
        Some("string") => Ty::Text,
        other => panic!("no type for {other:?} in {schema}"),
    }
}

fn fields(schema: &Value, skip: Option<&str>) -> Vec<Field> {
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    properties
        .iter()
        .filter(|(name, _)| skip != Some(name.as_str()))
        .map(|(name, inner)| Field {
            wire: name.clone(),
            ty: ty(inner),
            required: required.contains(&name.as_str()),
            doc: doc(inner),
        })
        .collect()
}

/// The field that tells variants apart: the one holding a `const` in every option.
fn tag_of(options: &[Value]) -> Option<String> {
    let first = options.first()?.get("properties")?.as_object()?;
    first
        .iter()
        .find(|(name, inner)| {
            inner.get("const").is_some()
                && options
                    .iter()
                    .all(|option| option["properties"][name.as_str()].get("const").is_some())
        })
        .map(|(name, _)| name.clone())
}

fn words(options: &[Value]) -> Vec<String> {
    let mut list = Vec::new();
    for option in options {
        if let Some(word) = option.get("const").and_then(Value::as_str) {
            list.push(word.to_string());
        }
        if let Some(values) = option.get("enum").and_then(Value::as_array) {
            list.extend(values.iter().filter_map(Value::as_str).map(str::to_string));
        }
    }
    list
}

fn shape(name: &str, schema: &Value) -> Shape {
    if let Some(options) = schema.get("oneOf").and_then(Value::as_array) {
        if let Some(tag) = tag_of(options) {
            let variants = options
                .iter()
                .map(|option| Variant {
                    word: option["properties"][tag.as_str()]["const"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    fields: fields(option, Some(&tag)),
                    doc: doc(option),
                })
                .collect();
            return Shape::Union {
                common: fields(schema, None),
                tag,
                variants,
            };
        }
        return Shape::Words(words(options));
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        let list = words(std::slice::from_ref(schema));
        return match (values.len(), name.ends_with("Tag")) {
            (1, true) => Shape::Fixed(list.into_iter().next().unwrap_or_default()),
            _ => Shape::Words(list),
        };
    }
    if schema.get("properties").is_some() {
        return Shape::Struct(fields(schema, None));
    }
    if schema.get("type").and_then(Value::as_str) == Some("object")
        && schema.get("additionalProperties").is_none()
    {
        return Shape::Struct(Vec::new());
    }
    Shape::Alias(ty(schema))
}

/// Every definition, sorted by name.
#[must_use]
pub fn items(defs: &BTreeMap<String, Value>) -> Vec<Item> {
    defs.iter()
        .map(|(name, schema)| Item {
            name: name.clone(),
            doc: doc(schema),
            shape: shape(name, schema),
        })
        .collect()
}

/// `http_done` as `HttpDone`.
#[must_use]
pub fn upper_camel(word: &str) -> String {
    word.split(['_', '-'])
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect()
}

/// `max_height` as `maxHeight`.
#[must_use]
pub fn lower_camel(word: &str) -> String {
    let camel = upper_camel(word);
    let mut chars = camel.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_lowercase().chain(chars).collect()
    })
}
