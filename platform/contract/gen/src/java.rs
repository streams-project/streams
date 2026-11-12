//! Java types for the Android surfaces, with a JSON codec that depends on nothing but the JDK.
//!
//! Why this file exists: the Kotlin types wrap every open word and every checked text in an
//! `@JvmInline value class`, so a data class field typed as one gets a getter with a hyphen in its
//! JVM name, and no Java *source* can call it. Note 210 records the count. These are plain classes,
//! so every accessor is callable from Java.
//!
//! The accessor names are protobuf shaped — `getTilesList()`, `getTilesCount()`, `getTiles(int)`,
//! `hasCursor()` — although a JSON reader needs none of that, so note 204's P5 can replace this
//! file with protoc's Java output without touching a call site. A surface reads a value through an
//! accessor and never constructs one; construction stays in `:core` and `:ports`.
//!
//! Java has no kotlinx.serialization, so the codec is emitted into the file: `Json` is a reader, a
//! writer and the handful of helpers the generated `read` and `write` methods call. That is what
//! makes this the one emitted language provable end to end with nothing installed.

use std::collections::BTreeMap;
use std::fmt::Write;

use contract::version::{ABI_VERSION, WIRE_VERSION};

use crate::shape::{Field, Item, Shape, Ty, Variant, lower_camel, upper_camel};

/// Every Java keyword, literal and restricted identifier. Java cannot escape an identifier the way
/// Kotlin's backticks do, so a wire name that lands on one keeps a trailing underscore — in the
/// field *and* in its accessor, because `class` would otherwise ask for `getClass()`, which is
/// final on `Object` and would not compile. This is what protoc's Java output does.
const RESERVED: &[&str] = &[
    "_",
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "permits",
    "private",
    "protected",
    "public",
    "record",
    "return",
    "sealed",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "var",
    "void",
    "volatile",
    "while",
    "yield",
];

/// The JSON codec, emitted once into the generated file. Nothing here is generated from the
/// contract, so it lives as text: a parser into plain `Map`, `List`, `String`, `Long`, `Double`,
/// `Boolean` and null, and the helpers the per type `read` and `write` methods call.
const CODEC: &str = r#"    /** What every generated type can do: write itself back as JSON. */
    public interface Wire {
        void write(StringBuilder out);
    }

    /**
     * A JSON reader and writer that depends on nothing but the JDK, because Java has no
     * kotlinx.serialization and `org.json` is in `android.jar` but not in a desktop JDK, which
     * would leave the Java arm of check_types.py unable to run.
     */
    public static final class Json {
        private final String text;
        private int at;

        private Json(String text) {
            this.text = text;
        }

        /** A document as Map, List, String, Long, Double, Boolean or null. */
        public static Object parse(String text) {
            Json json = new Json(text);
            json.space();
            Object found = json.value();
            json.space();
            if (json.at != text.length()) {
                throw new IllegalArgumentException("trailing text at " + json.at);
            }
            return found;
        }

        /** What a value writes itself as. */
        public static String render(Wire value) {
            StringBuilder out = new StringBuilder();
            value.write(out);
            return out.toString();
        }

        private void space() {
            while (at < text.length()) {
                char here = text.charAt(at);
                if (here != ' ' && here != '\t' && here != '\n' && here != '\r') {
                    return;
                }
                at++;
            }
        }

        private Object value() {
            if (at >= text.length()) {
                throw new IllegalArgumentException("no value at " + at);
            }
            char head = text.charAt(at);
            if (head == '{') {
                return readObject();
            }
            if (head == '[') {
                return readArray();
            }
            if (head == '"') {
                return readText();
            }
            if (head == 't') {
                word("true");
                return Boolean.TRUE;
            }
            if (head == 'f') {
                word("false");
                return Boolean.FALSE;
            }
            if (head == 'n') {
                word("null");
                return null;
            }
            return readNumber();
        }

        private void word(String wanted) {
            if (!text.startsWith(wanted, at)) {
                throw new IllegalArgumentException("not " + wanted + " at " + at);
            }
            at += wanted.length();
        }

        private Map<String, Object> readObject() {
            Map<String, Object> found = new LinkedHashMap<String, Object>();
            at++;
            space();
            if (at < text.length() && text.charAt(at) == '}') {
                at++;
                return found;
            }
            while (true) {
                space();
                String key = readText();
                space();
                if (at >= text.length() || text.charAt(at) != ':') {
                    throw new IllegalArgumentException("no colon at " + at);
                }
                at++;
                space();
                found.put(key, value());
                space();
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed object");
                }
                char next = text.charAt(at++);
                if (next == '}') {
                    return found;
                }
                if (next != ',') {
                    throw new IllegalArgumentException("no comma at " + (at - 1));
                }
            }
        }

        private List<Object> readArray() {
            List<Object> found = new ArrayList<Object>();
            at++;
            space();
            if (at < text.length() && text.charAt(at) == ']') {
                at++;
                return found;
            }
            while (true) {
                space();
                found.add(value());
                space();
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed list");
                }
                char next = text.charAt(at++);
                if (next == ']') {
                    return found;
                }
                if (next != ',') {
                    throw new IllegalArgumentException("no comma at " + (at - 1));
                }
            }
        }

        private String readText() {
            if (at >= text.length() || text.charAt(at) != '"') {
                throw new IllegalArgumentException("no text at " + at);
            }
            at++;
            StringBuilder out = new StringBuilder();
            while (true) {
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed text");
                }
                char here = text.charAt(at++);
                if (here == '"') {
                    return out.toString();
                }
                if (here != '\\') {
                    out.append(here);
                    continue;
                }
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed escape");
                }
                char mark = text.charAt(at++);
                if (mark == 'u') {
                    out.append((char) Integer.parseInt(text.substring(at, at + 4), 16));
                    at += 4;
                } else if (mark == 'n') {
                    out.append('\n');
                } else if (mark == 't') {
                    out.append('\t');
                } else if (mark == 'r') {
                    out.append('\r');
                } else if (mark == 'b') {
                    out.append('\b');
                } else if (mark == 'f') {
                    out.append('\f');
                } else if (mark == '"' || mark == '\\' || mark == '/') {
                    out.append(mark);
                } else {
                    throw new IllegalArgumentException("bad escape at " + (at - 1));
                }
            }
        }

        private Object readNumber() {
            int from = at;
            while (at < text.length() && "+-.eE0123456789".indexOf(text.charAt(at)) >= 0) {
                at++;
            }
            String found = text.substring(from, at);
            if (found.isEmpty()) {
                throw new IllegalArgumentException("no value at " + from);
            }
            if (found.indexOf('.') < 0 && found.indexOf('e') < 0 && found.indexOf('E') < 0) {
                return Long.valueOf(Long.parseLong(found));
            }
            return Double.valueOf(Double.parseDouble(found));
        }

        /** The object a message is read from. */
        @SuppressWarnings("unchecked")
        public static Map<String, Object> object(Object value) {
            if (value instanceof Map) {
                return (Map<String, Object>) value;
            }
            throw new IllegalArgumentException("not an object: " + value);
        }

        /** A field that must be there. A written null counts as absent, as it does in Kotlin. */
        public static Object need(Map<String, Object> from, String key, String type) {
            Object found = from.get(key);
            if (found == null) {
                throw new IllegalArgumentException(type + " has no " + key);
            }
            return found;
        }

        /** A field that may be absent. */
        public static Object at(Map<String, Object> from, String key) {
            return from.get(key);
        }

        public static String asText(Object value) {
            if (value instanceof String) {
                return (String) value;
            }
            throw new IllegalArgumentException("not text: " + value);
        }

        public static long asLong(Object value) {
            if (value instanceof Long) {
                return ((Long) value).longValue();
            }
            throw new IllegalArgumentException("not a whole number: " + value);
        }

        public static boolean asBool(Object value) {
            if (value instanceof Boolean) {
                return ((Boolean) value).booleanValue();
            }
            throw new IllegalArgumentException("not a boolean: " + value);
        }

        public static <T> List<T> list(Object value, Function<Object, T> each) {
            if (!(value instanceof List)) {
                throw new IllegalArgumentException("not a list: " + value);
            }
            List<?> from = (List<?>) value;
            List<T> found = new ArrayList<T>(from.size());
            for (Object item : from) {
                found.add(each.apply(item));
            }
            return Collections.unmodifiableList(found);
        }

        public static <T> Map<String, T> map(Object value, Function<Object, T> each) {
            Map<String, T> found = new LinkedHashMap<String, T>();
            for (Map.Entry<String, Object> entry : object(value).entrySet()) {
                found.put(entry.getKey(), each.apply(entry.getValue()));
            }
            return Collections.unmodifiableMap(found);
        }

        /** Opens a field, with the comma the field before it needs. */
        public static int key(StringBuilder out, int written, String name) {
            if (written > 0) {
                out.append(',');
            }
            text(out, name);
            out.append(':');
            return written + 1;
        }

        /** A JSON string literal. */
        public static void text(StringBuilder out, String value) {
            out.append('"');
            for (int index = 0; index < value.length(); index++) {
                char here = value.charAt(index);
                if (here == '"' || here == '\\') {
                    out.append('\\').append(here);
                } else if (here == '\n') {
                    out.append("\\n");
                } else if (here == '\r') {
                    out.append("\\r");
                } else if (here == '\t') {
                    out.append("\\t");
                } else if (here < 0x20) {
                    out.append(String.format("\\u%04x", Integer.valueOf(here)));
                } else {
                    out.append(here);
                }
            }
            out.append('"');
        }
    }
"#;

/// A Java string literal body. `$` is left alone: Java has no interpolation, and escaping it would
/// corrupt a pattern. This is the defect `kotlin.rs:107` shipped, from the other side.
fn literal(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// A field or local name. A Java keyword keeps a trailing underscore.
fn name(wire: &str) -> String {
    let camel = lower_camel(wire);
    if RESERVED.contains(&camel.as_str()) {
        format!("{camel}_")
    } else {
        camel
    }
}

/// What an accessor is named after: `Title` from `title`, `Class_` from `class`.
fn stem(wire: &str) -> String {
    let camel = upper_camel(wire);
    if RESERVED.contains(&lower_camel(wire).as_str()) {
        format!("{camel}_")
    } else {
        camel
    }
}

/// A constant name for an open word. A word opening with a digit keeps a leading underscore.
fn constant(word: &str) -> String {
    let upper = word.replace('-', "_").to_uppercase();
    if upper.starts_with(|first: char| first.is_ascii_digit()) {
        format!("_{upper}")
    } else {
        upper
    }
}

fn ty(ty: &Ty) -> String {
    match ty {
        Ty::Text => "String".into(),
        Ty::Int => "long".into(),
        Ty::Bool => "boolean".into(),
        Ty::List(inner) => format!("List<{}>", boxed(inner)),
        Ty::Map(inner) => format!("Map<String, {}>", boxed(inner)),
        Ty::Named(name) => name.clone(),
    }
}

/// The form a generic argument takes, where a primitive cannot go.
fn boxed(ty: &Ty) -> String {
    match ty {
        Ty::Int => "Long".into(),
        Ty::Bool => "Boolean".into(),
        other => self::ty(other),
    }
}

/// How a field is stored: an optional scalar is boxed, so absent and zero stay apart.
fn field_ty(field: &Field) -> String {
    if field.required {
        ty(&field.ty)
    } else {
        boxed(&field.ty)
    }
}

/// What an absent value reads as, which is protobuf's default and not null.
fn fallback(ty: &Ty) -> &'static str {
    match ty {
        Ty::Text => "\"\"",
        Ty::Int => "0L",
        Ty::Bool => "false",
        _ => "null",
    }
}

/// A javadoc comment. `*/` would close it early, and a leading `@` would read as a block tag.
fn comment(out: &mut String, indent: &str, doc: Option<&String>) {
    if let Some(doc) = doc {
        let safe = doc.replace("*/", "* /");
        let safe = match safe.strip_prefix('@') {
            Some(rest) => format!("{{@literal @}}{rest}"),
            None => safe,
        };
        let _ = writeln!(out, "{indent}/** {safe} */");
    }
}

/// Every alias, so a use of one resolves to the type it names: Java has no type alias.
fn aliases(items: &[Item]) -> BTreeMap<String, Ty> {
    items
        .iter()
        .filter_map(|item| match &item.shape {
            Shape::Alias(inner) => Some((item.name.clone(), inner.clone())),
            _ => None,
        })
        .collect()
}

fn resolve(ty: &Ty, aliases: &BTreeMap<String, Ty>) -> Ty {
    match ty {
        Ty::Named(name) => aliases
            .get(name)
            .map_or_else(|| ty.clone(), |target| resolve(target, aliases)),
        Ty::List(inner) => Ty::List(Box::new(resolve(inner, aliases))),
        Ty::Map(inner) => Ty::Map(Box::new(resolve(inner, aliases))),
        other => other.clone(),
    }
}

fn resolve_field(field: &Field, aliases: &BTreeMap<String, Ty>) -> Field {
    Field {
        ty: resolve(&field.ty, aliases),
        ..field.clone()
    }
}

/// Every item with its field types resolved through the aliases.
fn resolved(items: &[Item]) -> Vec<Item> {
    let table = aliases(items);
    let field = |list: &[Field]| -> Vec<Field> {
        list.iter()
            .map(|one| resolve_field(one, &table))
            .collect::<Vec<Field>>()
    };
    items
        .iter()
        .map(|item| Item {
            shape: match &item.shape {
                Shape::Struct(fields) => Shape::Struct(field(fields)),
                Shape::Union {
                    tag,
                    common,
                    variants,
                } => Shape::Union {
                    tag: tag.clone(),
                    common: field(common),
                    variants: variants
                        .iter()
                        .map(|variant| Variant {
                            fields: field(&variant.fields),
                            ..variant.clone()
                        })
                        .collect(),
                },
                other => other.clone(),
            },
            ..item.clone()
        })
        .collect()
}

/// How a field's accessors are emitted.
struct Reading<'a> {
    /// `public`, or `public abstract` on the base of a union.
    head: &'a str,
    /// The field a body reads. `None` emits the signature alone, for an abstract one.
    body: Option<&'a str>,
    /// Whether every method carries `@Override`.
    overriding: bool,
    /// Whether the field read may be null although the wire field is required, which is the
    /// unknown variant: it keeps the common fields as raw optionals, so a required one still
    /// needs the fallback Kotlin's `raw ?: ""` gives it.
    nullable: bool,
}

/// The accessors for one field, protobuf shaped: `getTilesList`, `getTilesCount`, `getTiles(int)`
/// for a repeated field, `hasCursor` for an optional one.
fn accessors(out: &mut String, field: &Field, how: &Reading) {
    let stem = stem(&field.wire);
    let bare = how.body.unwrap_or("");
    let mut methods: Vec<(String, Option<String>)> = Vec::new();
    match &field.ty {
        Ty::List(inner) => {
            let empty = format!("Collections.<{}>emptyList()", boxed(inner));
            methods.push((
                format!("List<{}> get{stem}List()", boxed(inner)),
                how.body
                    .map(|_| format!("{bare} == null ? {empty} : {bare}")),
            ));
            methods.push((
                format!("int get{stem}Count()"),
                how.body
                    .map(|_| format!("{bare} == null ? 0 : {bare}.size()")),
            ));
            methods.push((
                format!("{} get{stem}(int $index)", ty(inner)),
                how.body.map(|_| format!("{bare}.get($index)")),
            ));
        }
        Ty::Map(inner) => {
            let empty = format!("Collections.<String, {}>emptyMap()", boxed(inner));
            methods.push((
                format!("Map<String, {}> get{stem}Map()", boxed(inner)),
                how.body
                    .map(|_| format!("{bare} == null ? {empty} : {bare}")),
            ));
            methods.push((
                format!("int get{stem}Count()"),
                how.body
                    .map(|_| format!("{bare} == null ? 0 : {bare}.size()")),
            ));
        }
        plain => {
            if !field.required {
                methods.push((
                    format!("boolean has{stem}()"),
                    how.body.map(|_| format!("{bare} != null")),
                ));
            }
            let read = if field.required && !how.nullable {
                bare.to_string()
            } else if matches!(plain, Ty::Bool) {
                format!("{bare} != null && {bare}")
            } else {
                format!("{bare} == null ? {} : {bare}", fallback(plain))
            };
            methods.push((format!("{} get{stem}()", ty(plain)), how.body.map(|_| read)));
        }
    }
    comment(out, "    ", field.doc.as_ref());
    for (index, (signature, body)) in methods.into_iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        if how.overriding {
            out.push_str("    @Override\n");
        }
        match body {
            Some(text) => {
                let _ = writeln!(
                    out,
                    "    {} {signature} {{\n        return {text};\n    }}",
                    how.head
                );
            }
            None => {
                let _ = writeln!(out, "    {} {signature};", how.head);
            }
        }
    }
}

/// `Json.asText($raw0)`, and the loops a list or a map needs.
fn read_expr(raw: &str, ty: &Ty, depth: usize) -> String {
    match ty {
        Ty::Text => format!("Json.asText({raw})"),
        Ty::Int => format!("Json.asLong({raw})"),
        Ty::Bool => format!("Json.asBool({raw})"),
        Ty::Named(name) => format!("{name}.read({raw})"),
        Ty::List(inner) => format!(
            "Json.list({raw}, $x{depth} -> {})",
            read_expr(&format!("$x{depth}"), inner, depth + 1)
        ),
        Ty::Map(inner) => format!(
            "Json.map({raw}, $x{depth} -> {})",
            read_expr(&format!("$x{depth}"), inner, depth + 1)
        ),
    }
}

/// One constructor argument, read out of the raw local. An absent optional stays null, so the
/// writer can leave it out rather than write a null, which `kotlin.rs:234` does with explicitNulls.
fn read_arg(raw: &str, field: &Field) -> String {
    let found = read_expr(raw, &field.ty, 0);
    if field.required {
        return found;
    }
    match &field.ty {
        Ty::Int => format!("{raw} == null ? null : Long.valueOf({found})"),
        Ty::Bool => format!("{raw} == null ? null : Boolean.valueOf({found})"),
        _ => format!("{raw} == null ? null : {found}"),
    }
}

/// The `read` method of a plain message, a union variant or the unknown variant.
fn read_method(
    out: &mut String,
    type_name: &str,
    owner: &str,
    fields: &[Field],
    raw_names: bool,
    tag: Option<(&str, &str)>,
) {
    let _ = writeln!(
        out,
        "    public static {type_name} read(Object $json) {{\n        \
         Map<String, Object> $fields = Json.object($json);"
    );
    if let Some((key, word)) = tag {
        let _ = writeln!(
            out,
            "        if (!\"{word}\".equals(Json.asText(Json.need($fields, \"{key}\", \"{owner}\")))) {{\n            \
             throw new IllegalArgumentException(\"not a {type_name}\");\n        \
             }}"
        );
    }
    for (index, field) in fields.iter().enumerate() {
        let getter = if field.required && !raw_names {
            format!("Json.need($fields, \"{}\", \"{owner}\")", field.wire)
        } else {
            format!("Json.at($fields, \"{}\")", field.wire)
        };
        let _ = writeln!(out, "        Object $raw{index} = {getter};");
    }
    let args: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let loose = if raw_names {
                Field {
                    required: false,
                    ..field.clone()
                }
            } else {
                field.clone()
            };
            read_arg(&format!("$raw{index}"), &loose)
        })
        .collect();
    if args.is_empty() {
        let _ = writeln!(out, "        return new {type_name}();\n    }}");
    } else {
        let _ = writeln!(
            out,
            "        return new {type_name}(\n                {});\n    }}",
            args.join(",\n                ")
        );
    }
}

/// How one value is written. Recursive, so a list of maps nests without a helper per shape.
fn write_value(out: &mut String, indent: &str, expr: &str, ty: &Ty, depth: usize) {
    match ty {
        Ty::Text => {
            let _ = writeln!(out, "{indent}Json.text($out, {expr});");
        }
        Ty::Int | Ty::Bool => {
            let _ = writeln!(out, "{indent}$out.append({expr});");
        }
        Ty::Named(_) => {
            let _ = writeln!(out, "{indent}{expr}.write($out);");
        }
        Ty::List(inner) => {
            let _ = writeln!(out, "{indent}$out.append('[');");
            let _ = writeln!(
                out,
                "{indent}for (int $i{depth} = 0; $i{depth} < {expr}.size(); $i{depth}++) {{"
            );
            let _ = writeln!(out, "{indent}    if ($i{depth} > 0) {{");
            let _ = writeln!(out, "{indent}        $out.append(',');");
            let _ = writeln!(out, "{indent}    }}");
            write_value(
                out,
                &format!("{indent}    "),
                &format!("{expr}.get($i{depth})"),
                inner,
                depth + 1,
            );
            let _ = writeln!(out, "{indent}}}");
            let _ = writeln!(out, "{indent}$out.append(']');");
        }
        Ty::Map(inner) => {
            let _ = writeln!(out, "{indent}$out.append('{{');");
            let _ = writeln!(out, "{indent}int $n{depth} = 0;");
            let _ = writeln!(
                out,
                "{indent}for (Map.Entry<String, {}> $e{depth} : {expr}.entrySet()) {{",
                boxed(inner)
            );
            let _ = writeln!(
                out,
                "{indent}    $n{depth} = Json.key($out, $n{depth}, $e{depth}.getKey());"
            );
            write_value(
                out,
                &format!("{indent}    "),
                &format!("$e{depth}.getValue()"),
                inner,
                depth + 1,
            );
            let _ = writeln!(out, "{indent}}}");
            let _ = writeln!(out, "{indent}$out.append('}}');");
        }
    }
}

/// The `write` method. An absent optional is left out rather than written as null.
fn write_method(out: &mut String, fields: &[(String, Field)], tag: Option<(&str, &str)>) {
    let _ = writeln!(
        out,
        "    @Override\n    public void write(StringBuilder $out) {{"
    );
    if fields.is_empty() && tag.is_none() {
        let _ = writeln!(out, "        $out.append(\"{{}}\");\n    }}");
        return;
    }
    let _ = writeln!(out, "        $out.append('{{');\n        int $count = 0;");
    if let Some((key, word)) = tag {
        let _ = writeln!(
            out,
            "        $count = Json.key($out, $count, \"{key}\");\n        \
             Json.text($out, \"{}\");",
            literal(word)
        );
    }
    for (bare, field) in fields {
        if field.required {
            let _ = writeln!(
                out,
                "        $count = Json.key($out, $count, \"{}\");",
                field.wire
            );
            write_value(out, "        ", bare, &field.ty, 0);
        } else {
            let _ = writeln!(out, "        if ({bare} != null) {{");
            let _ = writeln!(
                out,
                "            $count = Json.key($out, $count, \"{}\");",
                field.wire
            );
            write_value(out, "            ", bare, &field.ty, 0);
            let _ = writeln!(out, "        }}");
        }
    }
    let _ = writeln!(out, "        $out.append('}}');\n    }}");
}

/// The stored fields and the constructor.
fn state(out: &mut String, type_name: &str, fields: &[(String, Field)]) {
    for (bare, field) in fields {
        let _ = writeln!(out, "    private final {} {bare};", field_ty(field));
    }
    if !fields.is_empty() {
        out.push('\n');
    }
    let params: Vec<String> = fields
        .iter()
        .map(|(bare, field)| format!("{} {bare}", field_ty(field)))
        .collect();
    let _ = writeln!(out, "    public {type_name}({}) {{", params.join(", "));
    for (bare, _) in fields {
        let _ = writeln!(out, "        this.{bare} = {bare};");
    }
    let _ = writeln!(out, "    }}");
}

/// A plain message: fields, constructor, accessors, reader, writer.
fn message(out: &mut String, type_name: &str, fields: &[Field]) {
    let held: Vec<(String, Field)> = fields
        .iter()
        .map(|field| (name(&field.wire), field.clone()))
        .collect();
    let _ = writeln!(
        out,
        "public static final class {type_name} implements Wire {{"
    );
    state(out, type_name, &held);
    for (bare, field) in &held {
        out.push('\n');
        accessors(
            out,
            field,
            &Reading {
                head: "public",
                body: Some(bare),
                overriding: false,
                nullable: false,
            },
        );
    }
    out.push('\n');
    read_method(out, type_name, type_name, fields, false, None);
    out.push('\n');
    write_method(out, &held, None);
    out.push_str("}\n");
}

/// An open word: any word the server sends is kept, and the ones this build knows are constants.
fn words(out: &mut String, type_name: &str, list: &[String]) {
    let _ = writeln!(
        out,
        "public static final class {type_name} implements Wire {{"
    );
    for word in list {
        let _ = writeln!(
            out,
            "    public static final {type_name} {} = new {type_name}(\"{}\");",
            constant(word),
            literal(word)
        );
    }
    if !list.is_empty() {
        out.push('\n');
    }
    let _ = writeln!(out, "    private final String value;\n");
    let _ = writeln!(
        out,
        "    public {type_name}(String value) {{\n        this.value = value;\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    public String value() {{\n        return value;\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    public static {type_name} read(Object $json) {{\n        \
         return new {type_name}(Json.asText($json));\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public void write(StringBuilder $out) {{\n        \
         Json.text($out, value);\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public boolean equals(Object other) {{\n        \
         return other instanceof {type_name} && ((({type_name}) other).value).equals(value);\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public int hashCode() {{\n        return value.hashCode();\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public String toString() {{\n        return value;\n    }}\n}}"
    );
}

/// Checked text: the pattern is compiled once, not per value, because a row of 20 tiles would pay
/// for it twenty times. The message matches Kotlin's `require(...) { "not a X" }`.
fn checked(out: &mut String, type_name: &str, pattern: &str) {
    let _ = writeln!(
        out,
        "public static final class {type_name} implements Wire {{"
    );
    let _ = writeln!(
        out,
        "    private static final Pattern SHAPE = Pattern.compile(\"{}\");\n",
        literal(pattern)
    );
    let _ = writeln!(out, "    private final String value;\n");
    let _ = writeln!(
        out,
        "    public {type_name}(String value) {{\n        \
         if (!SHAPE.matcher(value).matches()) {{\n            \
         throw new IllegalArgumentException(\"not a {type_name}\");\n        }}\n        \
         this.value = value;\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    public String value() {{\n        return value;\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    public static {type_name} read(Object $json) {{\n        \
         return new {type_name}(Json.asText($json));\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public void write(StringBuilder $out) {{\n        \
         Json.text($out, value);\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public boolean equals(Object other) {{\n        \
         return other instanceof {type_name} && ((({type_name}) other).value).equals(value);\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public int hashCode() {{\n        return value.hashCode();\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    @Override\n    public String toString() {{\n        return value;\n    }}\n}}"
    );
}

/// One fixed word, the type tag of a core message. Closed, unlike an open word: a tag this build
/// does not know is refused, which is what `CoreIn.wrong_type` asks for.
fn fixed(out: &mut String, type_name: &str, word: &str) {
    let _ = writeln!(out, "public enum {type_name} implements Wire {{");
    let _ = writeln!(out, "    VALUE(\"{}\");\n", literal(word));
    let _ = writeln!(out, "    private final String wire;\n");
    let _ = writeln!(
        out,
        "    {type_name}(String wire) {{\n        this.wire = wire;\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    public String wire() {{\n        return wire;\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    public static {type_name} read(Object $json) {{\n        \
         String $word = Json.asText($json);\n        \
         if (!\"{}\".equals($word)) {{\n            \
         throw new IllegalArgumentException(\"not a {type_name}: \" + $word);\n        }}\n        \
         return VALUE;\n    }}\n",
        literal(word)
    );
    let _ = writeln!(
        out,
        "    @Override\n    public void write(StringBuilder $out) {{\n        \
         Json.text($out, wire);\n    }}\n}}"
    );
}

/// A tagged union: an abstract base carrying the common fields, one class per variant, and one for
/// a tag this build does not know.
fn union(out: &mut String, item: &Item, tag: &str, common: &[Field], variants: &[Variant]) {
    let base = &item.name;
    let _ = writeln!(
        out,
        "public abstract static class {base} implements Wire {{"
    );
    for (index, field) in common.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        accessors(
            out,
            field,
            &Reading {
                head: "public abstract",
                body: None,
                overriding: false,
                nullable: false,
            },
        );
    }
    if !common.is_empty() {
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "    public static {base} read(Object $json) {{\n        \
         String $tag = Json.asText(Json.need(Json.object($json), \"{tag}\", \"{base}\"));"
    );
    for variant in variants {
        let _ = writeln!(
            out,
            "        if (\"{}\".equals($tag)) {{\n            return {base}{}.read($json);\n        }}",
            literal(&variant.word),
            upper_camel(&variant.word)
        );
    }
    let _ = writeln!(out, "        return {base}Unknown.read($json);\n    }}\n}}");

    for variant in variants {
        out.push('\n');
        comment(out, "", variant.doc.as_ref());
        let type_name = format!("{base}{}", upper_camel(&variant.word));
        let fields: Vec<Field> = common
            .iter()
            .chain(variant.fields.iter())
            .cloned()
            .collect();
        let held: Vec<(String, Field)> = fields
            .iter()
            .map(|field| (name(&field.wire), field.clone()))
            .collect();
        let _ = writeln!(
            out,
            "public static final class {type_name} extends {base} {{"
        );
        state(out, &type_name, &held);
        for (bare, field) in &held {
            out.push('\n');
            accessors(
                out,
                field,
                &Reading {
                    head: "public",
                    body: Some(bare),
                    overriding: common.iter().any(|one| one.wire == field.wire),
                    nullable: false,
                },
            );
        }
        out.push('\n');
        read_method(
            out,
            &type_name,
            base,
            &fields,
            false,
            Some((tag, &variant.word)),
        );
        out.push('\n');
        write_method(out, &held, Some((tag, &variant.word)));
        out.push_str("}\n");
    }

    out.push_str("\n/** A tag this build does not know. */\n");
    let type_name = format!("{base}Unknown");
    let raw: Vec<(String, Field)> = common
        .iter()
        .map(|field| {
            (
                format!("raw{}", stem(&field.wire)),
                Field {
                    required: false,
                    ..field.clone()
                },
            )
        })
        .collect();
    let mut held = vec![(
        "tag".to_string(),
        Field {
            wire: tag.to_string(),
            ty: Ty::Text,
            required: true,
            doc: Some("The tag that was read.".to_string()),
        },
    )];
    held.extend(raw.iter().cloned());
    let _ = writeln!(
        out,
        "public static final class {type_name} extends {base} {{"
    );
    state(out, &type_name, &held);
    out.push_str("\n    /** The tag that was read. */\n");
    let _ = writeln!(
        out,
        "    public String getTag() {{\n        return tag;\n    }}"
    );
    for (bare, field) in &raw {
        out.push('\n');
        let known = common
            .iter()
            .find(|one| one.wire == field.wire)
            .expect("a raw field comes from a common one");
        accessors(
            out,
            known,
            &Reading {
                head: "public",
                body: Some(bare),
                overriding: true,
                nullable: true,
            },
        );
    }
    out.push('\n');
    let fields: Vec<Field> = held.iter().map(|(_, field)| field.clone()).collect();
    read_method(out, &type_name, base, &fields, true, None);
    out.push('\n');
    let writing: Vec<(String, Field)> = held
        .iter()
        .map(|(bare, field)| {
            (
                bare.clone(),
                Field {
                    required: bare == "tag",
                    ..field.clone()
                },
            )
        })
        .collect();
    write_method(out, &writing, None);
    out.push_str("}\n");
}

/// The whole file. Every type is a nested class of one outer `Contract`, because a Java file holds
/// one public top level class, and because it keeps a name like `ClassValue` away from
/// `java.lang.ClassValue`.
#[must_use]
pub fn write(items: &[Item]) -> String {
    let items = resolved(items);
    let mut out = String::from("// Generated from the contract. Do not edit.\n\n");
    out.push_str("package contract;\n\n");
    out.push_str("import java.util.ArrayList;\n");
    out.push_str("import java.util.Collections;\n");
    out.push_str("import java.util.LinkedHashMap;\n");
    out.push_str("import java.util.List;\n");
    out.push_str("import java.util.Map;\n");
    out.push_str("import java.util.function.Function;\n");
    out.push_str("import java.util.regex.Pattern;\n\n");
    out.push_str("/**\n");
    out.push_str(" * Every type the contract publishes, with protobuf shaped accessor names so\n");
    out.push_str(" * note 204's P5 can swap protoc's output in without touching a call site.\n");
    out.push_str(" * Read a value through an accessor; construction stays in :core and :ports.\n");
    out.push_str(" */\n");
    out.push_str("public final class Contract {\n");
    out.push_str("    private Contract() {\n    }\n\n");
    let _ = writeln!(
        out,
        "    public static final String WIRE_VERSION = \"{WIRE_VERSION}\";"
    );
    let _ = writeln!(
        out,
        "    public static final int ABI_VERSION = {ABI_VERSION};\n"
    );
    out.push_str("    /** Every address the apps open. */\n");
    out.push_str("    public static final class Address {\n");
    out.push_str("        private Address() {\n        }\n\n");
    for (name, value) in contract::address::ALL {
        let _ = writeln!(
            out,
            "        public static final String {name} = \"{}\";",
            literal(value)
        );
    }
    out.push_str("\n        /** Where a tab lives, by the screen's word. */\n");
    out.push_str("        public static String of(String screen) {\n");
    out.push_str("            switch (screen) {\n");
    for (name, value) in contract::address::SCREENS {
        let _ = writeln!(
            out,
            "                case \"{}\":\n                    return \"{}\";",
            literal(name),
            literal(value)
        );
    }
    out.push_str("                default:\n                    return HOME;\n");
    out.push_str("            }\n        }\n    }\n\n");
    out.push_str(CODEC);
    for item in &items {
        let mut body = String::new();
        // An alias emits no declaration, so a javadoc would dangle: it goes in the line comment.
        if !matches!(item.shape, Shape::Alias(_)) {
            comment(&mut body, "", item.doc.as_ref());
        }
        match &item.shape {
            Shape::Struct(fields) => message(&mut body, &item.name, fields),
            Shape::Words(list) => words(&mut body, &item.name, list),
            Shape::Checked(pattern) => checked(&mut body, &item.name, pattern),
            Shape::Fixed(word) => fixed(&mut body, &item.name, word),
            Shape::Union {
                tag,
                common,
                variants,
            } => union(&mut body, item, tag, common, variants),
            Shape::Alias(inner) => {
                if let Some(doc) = item.doc.as_ref() {
                    let _ = writeln!(body, "// {doc}");
                }
                let _ = writeln!(
                    body,
                    "// {} is {}. Java has no type alias, so every use of it reads {}.",
                    item.name,
                    boxed(inner),
                    boxed(inner)
                );
            }
        }
        out.push('\n');
        for line in body.lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                let _ = writeln!(out, "    {line}");
            }
        }
    }
    out.push_str("}\n");
    out
}
