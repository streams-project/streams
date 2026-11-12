//! C++ types for the Xbox surface, with a JSON codec that depends on nothing but the standard
//! library.
//!
//! Why this file exists: note 220 found that Xbox's next blocker is neither hardware nor money but
//! a missing C++ emitter here. A screen reads the view the core returns, and `apps/xbox` had
//! nothing to read it as. Hand-writing the contract in C++ was considered and refused: it would
//! put a second, unchecked copy of the contract in the tree.
//!
//! It follows `java.rs` function for function, so a reviewer can diff them, and differs in four
//! places, each because C++ differs:
//!
//! * **No accessors.** Java emits `getTilesList()` because Kotlin's `@JvmInline value class` makes
//!   a field uncallable from Java source. C++ has no such problem, so a type is a plain aggregate
//!   with public members, as `swift.rs` emits `public var`. A screen reads `view.rows[0].title`.
//! * **Declaration order.** Java and Kotlin do not care where a type is declared; C++ needs one
//!   complete before it is held by value, so `ordered` emits the definitions depth first. The
//!   graph is acyclic today and a cycle panics rather than writing a file that cannot compile.
//! * **No ALL CAPS identifier anywhere.** This emitter exists for Windows, where `<windows.h>`
//!   defines `DELETE` (winnt.h) and `ERROR` (wingdi.h) as object-like macros, and a macro ignores
//!   namespaces and class scope. Of the 185 words the contract publishes, `DELETE`, `delete` and
//!   `error` would have become constants of exactly those names under Java's rule, so the words
//!   are lower case here and the keyword rename catches `continue`, `delete` and `new`.
//! * **A tagged union is a `std::variant`, not a class hierarchy.** Java returns the abstract base
//!   by reference; C++ would need a heap allocation to do the same. The variants are emitted as
//!   their own structs and the union is a struct of the common fields plus a `body`, which is the
//!   shape `swift.rs` already uses.
//!
//! Java has no `kotlinx.serialization` and C++ has no JSON at all, so the codec is emitted into
//! the file the same way: `Json` is a reader, a writer and the handful of helpers the generated
//! `read` and `write` members call. That is what lets `check_types.py` prove this language with
//! nothing installed but a C++ compiler.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use contract::version::{ABI_VERSION, WIRE_VERSION};

use crate::shape::{Field, Item, Shape, Ty, Variant, upper_camel};

/// Every C++ keyword, through C++23, and the four identifiers with special meaning. C++ cannot
/// escape an identifier the way Kotlin's backticks do, so a wire name that lands on one keeps a
/// trailing underscore. `class` is the one wire field name of the 290 that needs it.
const RESERVED: &[&str] = &[
    "alignas",
    "alignof",
    "and",
    "and_eq",
    "asm",
    "atomic_cancel",
    "atomic_commit",
    "atomic_noexcept",
    "auto",
    "bitand",
    "bitor",
    "bool",
    "break",
    "case",
    "catch",
    "char",
    "char16_t",
    "char32_t",
    "char8_t",
    "class",
    "co_await",
    "co_return",
    "co_yield",
    "compl",
    "concept",
    "const",
    "const_cast",
    "consteval",
    "constexpr",
    "constinit",
    "continue",
    "decltype",
    "default",
    "delete",
    "do",
    "double",
    "dynamic_cast",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "final",
    "float",
    "for",
    "friend",
    "goto",
    "if",
    "import",
    "inline",
    "int",
    "long",
    "module",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "not",
    "not_eq",
    "nullptr",
    "operator",
    "or",
    "or_eq",
    "override",
    "private",
    "protected",
    "public",
    "reflexpr",
    "register",
    "reinterpret_cast",
    "requires",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "static_cast",
    "struct",
    "switch",
    "synchronized",
    "template",
    "this",
    "thread_local",
    "throw",
    "true",
    "try",
    "typedef",
    "typeid",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "wchar_t",
    "while",
    "xor",
    "xor_eq",
];

/// The lower case object-like macros `<windows.h>` defines, which would eat an identifier of the
/// same name wherever it appeared. None of the 290 wire field names is one of these today; a name
/// that arrived would compile on this Mac and fail only on the platform this emitter is for, which
/// is why they are renamed here rather than discovered there. The ALL CAPS ones (`DELETE`,
/// `ERROR`, `IN`, `OUT`, `OPTIONAL`) cannot reach a name in this file: it emits none.
const MACROS: &[&str] = &["cdecl", "far", "interface", "near", "pascal", "small"];

/// The names this emitter itself puts in a type's scope. A wire field that landed on one would
/// collide with a member the type needs, so it keeps a trailing underscore like a keyword. None of
/// the 290 wire field names is one of these today.
const TAKEN: &[&str] = &[
    "read",
    "read_body",
    "read_fields",
    "word",
    "write",
    "write_fields",
];

/// The JSON codec, emitted once into the generated header. Nothing here is generated from the
/// contract, so it lives as text: a parser into one `Value` tree, and the helpers the per type
/// `read` and `write` members call.
///
/// It is a `struct` of statics rather than a namespace because unqualified lookup inside a member
/// function finds that type's own members first, so a namespace called `json` would be shadowed by
/// any field of that name. No wire field name is UpperCamel, so `Json` cannot be shadowed.
const CODEC: &str = r#"/// A value the contract refuses, named by what refused it. Every generated `read` throws this and
/// nothing else, so one `catch` covers the contract.
class Refused : public std::runtime_error {
public:
    explicit Refused(const std::string& why) : std::runtime_error(why) {}
};

/// A JSON reader and writer that depends on nothing but the standard library, because C++ has no
/// JSON of its own and a generated header must not add a dependency on one.
struct Json {
    enum class Kind { Null, Bool, Whole, Real, Text, Array, Object };

    /// One JSON value.
    ///
    /// An object holds its keys and its values in two vectors side by side, and not in a
    /// `std::map<std::string, Value>`, because inside its own definition `Value` is incomplete and
    /// `std::vector` is the one standard container allowed to hold an incomplete type. The order
    /// written is the order read, and a repeated key keeps the value written last.
    struct Value {
        Kind kind = Kind::Null;
        bool boolean = false;
        std::int64_t whole = 0;
        double real = 0;
        std::string text;
        std::vector<std::string> keys;
        std::vector<Value> items;
    };

    /// A document as a tree of values.
    static Value parse(const std::string& text) {
        std::size_t index = 0;
        space(text, index);
        Value found = read_value(text, index);
        space(text, index);
        if (index != text.size()) {
            throw Refused("trailing text at " + std::to_string(index));
        }
        return found;
    }

    /// What a value writes itself as.
    template <class T>
    static std::string render(const T& held) {
        std::string out;
        held.write(out);
        return out;
    }

    /// Whether two documents say the same thing. Object fields are compared by key and not by the
    /// order they were written in, which is what `Map.equals` gives the Java arm of the same check.
    static bool same(const Value& left, const Value& right) {
        if (left.kind != right.kind) {
            return false;
        }
        switch (left.kind) {
            case Kind::Null:
                return true;
            case Kind::Bool:
                return left.boolean == right.boolean;
            case Kind::Whole:
                return left.whole == right.whole;
            case Kind::Real:
                return left.real >= right.real && left.real <= right.real;
            case Kind::Text:
                return left.text == right.text;
            case Kind::Array:
                if (left.items.size() != right.items.size()) {
                    return false;
                }
                for (std::size_t index = 0; index < left.items.size(); index++) {
                    if (!same(left.items[index], right.items[index])) {
                        return false;
                    }
                }
                return true;
            case Kind::Object:
                if (left.keys.size() != right.keys.size()) {
                    return false;
                }
                for (std::size_t index = 0; index < left.keys.size(); index++) {
                    const Value* found = held(right, left.keys[index]);
                    if (found == nullptr || !same(left.items[index], *found)) {
                        return false;
                    }
                }
                return true;
        }
        return false;
    }

    /// The object a message is read from.
    static const Value& object(const Value& value) {
        if (value.kind != Kind::Object) {
            throw Refused("not an object");
        }
        return value;
    }

    /// The entry under a key, whether it was written as null or not.
    static const Value* held(const Value& fields, std::string_view key) {
        for (std::size_t index = 0; index < fields.keys.size(); index++) {
            if (fields.keys[index] == key) {
                return &fields.items[index];
            }
        }
        return nullptr;
    }

    /// A field that may be absent. A written null counts as absent, as it does in Kotlin.
    static const Value* at(const Value& fields, std::string_view key) {
        const Value* found = held(fields, key);
        if (found == nullptr || found->kind == Kind::Null) {
            return nullptr;
        }
        return found;
    }

    /// A field that must be there. Never null, so the generated reader dereferences it.
    static const Value* need(const Value& fields, std::string_view key, std::string_view type) {
        const Value* found = at(fields, key);
        if (found == nullptr) {
            throw Refused(std::string(type) + " has no " + std::string(key));
        }
        return found;
    }

    static const std::string& as_text(const Value& value) {
        if (value.kind != Kind::Text) {
            throw Refused("not text");
        }
        return value.text;
    }

    static std::int64_t as_long(const Value& value) {
        if (value.kind != Kind::Whole) {
            throw Refused("not a whole number");
        }
        return value.whole;
    }

    static bool as_bool(const Value& value) {
        if (value.kind != Kind::Bool) {
            throw Refused("not a boolean");
        }
        return value.boolean;
    }

    template <class T, class Each>
    static std::vector<T> list(const Value& value, Each each) {
        if (value.kind != Kind::Array) {
            throw Refused("not a list");
        }
        std::vector<T> found;
        found.reserve(value.items.size());
        for (const Value& item : value.items) {
            found.push_back(each(item));
        }
        return found;
    }

    template <class T, class Each>
    static std::map<std::string, T> map(const Value& value, Each each) {
        const Value& fields = object(value);
        std::map<std::string, T> found;
        for (std::size_t index = 0; index < fields.keys.size(); index++) {
            found.emplace(fields.keys[index], each(fields.items[index]));
        }
        return found;
    }

    /// Opens a field, with the comma the field before it needs.
    static int key(std::string& out, int written, std::string_view name) {
        if (written > 0) {
            out += ',';
        }
        text(out, name);
        out += ':';
        return written + 1;
    }

    /// A JSON string literal. Bytes at or above 0x80 pass through, because a `std::string` here
    /// holds UTF-8 and JSON carries UTF-8.
    static void text(std::string& out, std::string_view value) {
        out += '"';
        for (const char here : value) {
            const unsigned char code = static_cast<unsigned char>(here);
            if (here == '"' || here == '\\') {
                out += '\\';
                out += here;
            } else if (here == '\n') {
                out += "\\n";
            } else if (here == '\r') {
                out += "\\r";
            } else if (here == '\t') {
                out += "\\t";
            } else if (code < 0x20) {
                const char* digits = "0123456789abcdef";
                out += "\\u00";
                out += digits[code >> 4];
                out += digits[code & 0x0F];
            } else {
                out += here;
            }
        }
        out += '"';
    }

private:
    static void space(const std::string& text, std::size_t& index) {
        while (index < text.size()) {
            const char here = text[index];
            if (here != ' ' && here != '\t' && here != '\n' && here != '\r') {
                return;
            }
            index++;
        }
    }

    static Value read_value(const std::string& text, std::size_t& index) {
        if (index >= text.size()) {
            throw Refused("no value at " + std::to_string(index));
        }
        Value found;
        const char head = text[index];
        if (head == '{') {
            return read_object(text, index);
        }
        if (head == '[') {
            return read_array(text, index);
        }
        if (head == '"') {
            found.kind = Kind::Text;
            found.text = read_text(text, index);
            return found;
        }
        if (head == 't' || head == 'f') {
            word(text, index, head == 't' ? "true" : "false");
            found.kind = Kind::Bool;
            found.boolean = head == 't';
            return found;
        }
        if (head == 'n') {
            word(text, index, "null");
            return found;
        }
        return read_number(text, index);
    }

    static void word(const std::string& text, std::size_t& index, std::string_view wanted) {
        if (text.compare(index, wanted.size(), wanted) != 0) {
            throw Refused("not " + std::string(wanted) + " at " + std::to_string(index));
        }
        index += wanted.size();
    }

    static Value read_object(const std::string& text, std::size_t& index) {
        Value found;
        found.kind = Kind::Object;
        index++;
        space(text, index);
        if (index < text.size() && text[index] == '}') {
            index++;
            return found;
        }
        while (true) {
            space(text, index);
            std::string name = read_text(text, index);
            space(text, index);
            if (index >= text.size() || text[index] != ':') {
                throw Refused("no colon at " + std::to_string(index));
            }
            index++;
            space(text, index);
            Value value = read_value(text, index);
            const Value* already = held(found, name);
            if (already == nullptr) {
                found.keys.push_back(std::move(name));
                found.items.push_back(std::move(value));
            } else {
                found.items[static_cast<std::size_t>(already - found.items.data())] =
                        std::move(value);
            }
            space(text, index);
            if (index >= text.size()) {
                throw Refused("unclosed object");
            }
            const char next = text[index++];
            if (next == '}') {
                return found;
            }
            if (next != ',') {
                throw Refused("no comma at " + std::to_string(index - 1));
            }
        }
    }

    static Value read_array(const std::string& text, std::size_t& index) {
        Value found;
        found.kind = Kind::Array;
        index++;
        space(text, index);
        if (index < text.size() && text[index] == ']') {
            index++;
            return found;
        }
        while (true) {
            space(text, index);
            found.items.push_back(read_value(text, index));
            space(text, index);
            if (index >= text.size()) {
                throw Refused("unclosed list");
            }
            const char next = text[index++];
            if (next == ']') {
                return found;
            }
            if (next != ',') {
                throw Refused("no comma at " + std::to_string(index - 1));
            }
        }
    }

    static std::string read_text(const std::string& text, std::size_t& index) {
        if (index >= text.size() || text[index] != '"') {
            throw Refused("no text at " + std::to_string(index));
        }
        index++;
        std::string out;
        while (true) {
            if (index >= text.size()) {
                throw Refused("unclosed text");
            }
            const char here = text[index++];
            if (here == '"') {
                return out;
            }
            if (here != '\\') {
                out += here;
                continue;
            }
            if (index >= text.size()) {
                throw Refused("unclosed escape");
            }
            const char mark = text[index++];
            if (mark == 'u') {
                append_point(out, text, index);
            } else if (mark == 'n') {
                out += '\n';
            } else if (mark == 't') {
                out += '\t';
            } else if (mark == 'r') {
                out += '\r';
            } else if (mark == 'b') {
                out += '\b';
            } else if (mark == 'f') {
                out += '\f';
            } else if (mark == '"' || mark == '\\' || mark == '/') {
                out += mark;
            } else {
                throw Refused("bad escape at " + std::to_string(index - 1));
            }
        }
    }

    /// Four hex digits as UTF-8, joining a surrogate pair when one follows. A `std::string` holds
    /// bytes, not UTF-16 units, so Java's `(char) point` has no counterpart here.
    static void append_point(std::string& out, const std::string& text, std::size_t& index) {
        unsigned long point = hex(text, index);
        if (point >= 0xD800 && point <= 0xDBFF) {
            if (text.compare(index, 2, "\\u") != 0) {
                throw Refused("a lone high surrogate at " + std::to_string(index));
            }
            index += 2;
            const unsigned long low = hex(text, index);
            if (low < 0xDC00 || low > 0xDFFF) {
                throw Refused("not a low surrogate at " + std::to_string(index));
            }
            point = 0x10000 + ((point - 0xD800) << 10) + (low - 0xDC00);
        } else if (point >= 0xDC00 && point <= 0xDFFF) {
            throw Refused("a lone low surrogate at " + std::to_string(index));
        }
        if (point < 0x80) {
            out += static_cast<char>(point);
            return;
        }
        if (point < 0x800) {
            out += static_cast<char>(0xC0 | (point >> 6));
        } else if (point < 0x10000) {
            out += static_cast<char>(0xE0 | (point >> 12));
            out += static_cast<char>(0x80 | ((point >> 6) & 0x3F));
        } else {
            out += static_cast<char>(0xF0 | (point >> 18));
            out += static_cast<char>(0x80 | ((point >> 12) & 0x3F));
            out += static_cast<char>(0x80 | ((point >> 6) & 0x3F));
        }
        out += static_cast<char>(0x80 | (point & 0x3F));
    }

    static unsigned long hex(const std::string& text, std::size_t& index) {
        if (index + 4 > text.size()) {
            throw Refused("a short escape at " + std::to_string(index));
        }
        unsigned long found = 0;
        for (std::size_t step = 0; step < 4; step++) {
            const char here = text[index + step];
            const std::size_t digit = std::string_view("0123456789abcdef").find(
                    static_cast<char>(here >= 'A' && here <= 'F' ? here + ('a' - 'A') : here));
            if (digit == std::string_view::npos) {
                throw Refused("not a hex digit at " + std::to_string(index + step));
            }
            found = (found << 4) + digit;
        }
        index += 4;
        return found;
    }

    static Value read_number(const std::string& text, std::size_t& index) {
        const std::size_t from = index;
        while (index < text.size()
                && std::string_view("+-.eE0123456789").find(text[index])
                        != std::string_view::npos) {
            index++;
        }
        const std::string found = text.substr(from, index - from);
        if (found.empty()) {
            throw Refused("no value at " + std::to_string(from));
        }
        Value out;
        const bool whole = found.find('.') == std::string::npos
                && found.find('e') == std::string::npos
                && found.find('E') == std::string::npos;
        try {
            if (whole) {
                out.kind = Kind::Whole;
                out.whole = static_cast<std::int64_t>(std::stoll(found));
            } else {
                out.kind = Kind::Real;
                out.real = std::stod(found);
            }
        } catch (const std::exception&) {
            throw Refused("not a number: " + found);
        }
        return out;
    }
};
"#;

/// A C++ string literal body. `$` is left alone, as in Java: C++ has no interpolation and escaping
/// it would corrupt a pattern. A control character becomes a three digit octal escape and not a
/// hex one, because `\x1b` would swallow a following hex digit and `\033` cannot.
fn literal(text: &str) -> String {
    let mut out = String::new();
    for found in text.chars() {
        match found {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if (other as u32) < 0x20 => {
                let _ = write!(out, "\\{:03o}", other as u32);
            }
            other => out.push(other),
        }
    }
    out
}

/// A member name. The wire name is kept as it is, because it is already snake case and so is C++;
/// a name that lands on a keyword, on a Windows macro or on a member this emitter needs keeps a
/// trailing underscore. This one helper is reached from the struct path and from the union path,
/// which is what stops a rename landing in only one of them.
fn name(wire: &str) -> String {
    if RESERVED.contains(&wire) || MACROS.contains(&wire) || TAKEN.contains(&wire) {
        format!("{wire}_")
    } else {
        wire.to_string()
    }
}

/// A constant name for a word or an address. Lower case, never ALL CAPS: see the header of this
/// file for `DELETE` and `ERROR`. A word opening with a digit keeps a leading underscore.
fn constant(word: &str) -> String {
    let lower = word.replace('-', "_").to_lowercase();
    if lower.starts_with(|first: char| first.is_ascii_digit()) {
        format!("_{lower}")
    } else {
        name(&lower)
    }
}

fn ty(ty: &Ty) -> String {
    match ty {
        Ty::Text => "std::string".into(),
        Ty::Int => "std::int64_t".into(),
        Ty::Bool => "bool".into(),
        Ty::List(inner) => format!("std::vector<{}>", self::ty(inner)),
        Ty::Map(inner) => format!("std::map<std::string, {}>", self::ty(inner)),
        Ty::Named(name) => name.clone(),
    }
}

/// How a field is stored: an optional one is a `std::optional`, so absent and empty stay apart.
fn field_ty(field: &Field) -> String {
    if field.required {
        ty(&field.ty)
    } else {
        format!("std::optional<{}>", ty(&field.ty))
    }
}

/// What a member starts as when nobody says. A string, a list, a map and an optional are empty
/// already; a number and a boolean would be indeterminate, so they are written down.
fn initial(field: &Field) -> &'static str {
    if !field.required {
        return "";
    }
    match &field.ty {
        Ty::Int => " = 0",
        Ty::Bool => " = false",
        _ => "",
    }
}

/// A doc comment. `///` and not `/** */`: a line comment cannot be closed early by a `*/` in the
/// text, which in C++ is a compile error at best and swallows the declaration after it at worst.
/// A newline or a carriage return would end the comment and leave the rest as code, so both go.
fn comment(out: &mut String, indent: &str, doc: Option<&String>) {
    if let Some(doc) = doc {
        let safe = doc.replace(['\n', '\r'], " ");
        let _ = writeln!(out, "{indent}/// {safe}");
    }
}

/// Every alias, so a field typed as one reads and writes as the type it names. C++ *has* a type
/// alias and emits one, but `using Cursor = std::string;` is transparent: `Cursor::read` is
/// `std::string::read`, which does not exist. So a field's type is resolved the way `java.rs`
/// resolves it, and the alias is still declared for anyone who wants to name it.
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

/// Every named type a definition must have complete before it: C++ holds a member by value.
fn needs(ty: &Ty, found: &mut BTreeSet<String>) {
    match ty {
        Ty::Named(name) => {
            found.insert(name.clone());
        }
        Ty::List(inner) | Ty::Map(inner) => needs(inner, found),
        Ty::Text | Ty::Int | Ty::Bool => {}
    }
}

fn depends(item: &Item) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    match &item.shape {
        Shape::Struct(fields) => {
            fields.iter().for_each(|field| needs(&field.ty, &mut found));
        }
        Shape::Alias(inner) => needs(inner, &mut found),
        Shape::Union {
            common, variants, ..
        } => {
            common.iter().for_each(|field| needs(&field.ty, &mut found));
            variants
                .iter()
                .flat_map(|variant| &variant.fields)
                .for_each(|field| needs(&field.ty, &mut found));
        }
        Shape::Words(_) | Shape::Fixed(_) | Shape::Checked(_) => {}
    }
    found
}

fn walk<'a>(
    item: &'a Item,
    table: &BTreeMap<&'a str, &'a Item>,
    open: &mut Vec<&'a str>,
    done: &mut BTreeSet<&'a str>,
    out: &mut Vec<&'a Item>,
) {
    let here = item.name.as_str();
    if done.contains(here) {
        return;
    }
    assert!(
        !open.contains(&here),
        "the contract holds a cycle, which C++ cannot declare by value: {} -> {here}",
        open.join(" -> ")
    );
    open.push(here);
    for wanted in depends(item) {
        if let Some(next) = table.get(wanted.as_str()) {
            walk(next, table, open, done, out);
        }
    }
    open.pop();
    done.insert(here);
    out.push(item);
}

/// The definitions in an order C++ can declare: every type a definition holds comes first. Walking
/// the names in order keeps the output the same bytes for the same input.
fn ordered(items: &[Item]) -> Vec<&Item> {
    let table: BTreeMap<&str, &Item> = items
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let mut open = Vec::new();
    let mut done = BTreeSet::new();
    let mut out = Vec::new();
    for item in items {
        walk(item, &table, &mut open, &mut done, &mut out);
    }
    out
}

/// One data member, with its doc comment.
fn member(out: &mut String, field: &Field) {
    comment(out, "    ", field.doc.as_ref());
    let _ = writeln!(
        out,
        "    {} {}{};",
        field_ty(field),
        name(&field.wire),
        initial(field)
    );
}

/// `Json::as_text(*raw0)`, and the loops a list or a map needs.
fn read_expr(raw: &str, ty: &Ty, depth: usize) -> String {
    match ty {
        Ty::Text => format!("Json::as_text({raw})"),
        Ty::Int => format!("Json::as_long({raw})"),
        Ty::Bool => format!("Json::as_bool({raw})"),
        Ty::Named(name) => format!("{name}::read({raw})"),
        Ty::List(inner) => format!(
            "Json::list<{}>({raw}, [](const Json::Value& x{depth}) {{ return {}; }})",
            self::ty(inner),
            read_expr(&format!("x{depth}"), inner, depth + 1)
        ),
        Ty::Map(inner) => format!(
            "Json::map<{}>({raw}, [](const Json::Value& x{depth}) {{ return {}; }})",
            self::ty(inner),
            read_expr(&format!("x{depth}"), inner, depth + 1)
        ),
    }
}

/// One member of the braced initialiser, read out of the raw pointer. An absent optional stays
/// empty, so the writer leaves the field out rather than writing a null.
fn read_arg(raw: &str, field: &Field) -> String {
    let found = read_expr(&format!("*{raw}"), &field.ty, 0);
    if field.required {
        return found;
    }
    let held = field_ty(field);
    format!("{raw} == nullptr ? {held}() : {held}({found})")
}

/// How a type is read.
///
/// This stands where `java.rs` keeps its accessor-shaped `Reading`: there are no accessors here,
/// and what differs between a message and a union variant is only whether the reader opens the
/// document itself or is handed the object the union wrapper already opened.
struct Reading<'a> {
    /// `read` for a type read from a document, `read_fields` for a union variant.
    method: &'a str,
    /// Whether the body opens the document into an object first.
    opens: bool,
    /// One more initialiser after the fields, which is how a union wrapper reads its body.
    tail: Option<&'a str>,
}

/// The `read` of a plain message or the `read_fields` of a union variant.
fn read_method(out: &mut String, type_name: &str, owner: &str, fields: &[Field], how: &Reading) {
    // A variant with no fields of its own reads nothing out of the object it is handed, and an
    // unused parameter is an error under -Wextra, so that one parameter goes unnamed.
    let head = if how.opens {
        " document"
    } else if fields.is_empty() {
        ""
    } else {
        " fields"
    };
    let _ = writeln!(
        out,
        "    static {type_name} {}(const Json::Value&{head}) {{",
        how.method
    );
    let wanted = !fields.is_empty() || how.tail.is_some();
    if how.opens {
        if wanted {
            let _ = writeln!(
                out,
                "        const Json::Value& fields = Json::object(document);"
            );
        } else {
            let _ = writeln!(out, "        Json::object(document);");
        }
    }
    for (index, field) in fields.iter().enumerate() {
        let getter = if field.required {
            format!("Json::need(fields, \"{}\", \"{owner}\")", field.wire)
        } else {
            format!("Json::at(fields, \"{}\")", field.wire)
        };
        let _ = writeln!(out, "        const Json::Value* raw{index} = {getter};");
    }
    let mut args: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(index, field)| read_arg(&format!("raw{index}"), field))
        .collect();
    args.extend(how.tail.map(str::to_string));
    if args.is_empty() {
        let _ = writeln!(out, "        return {type_name}{{}};\n    }}");
        return;
    }
    let _ = writeln!(
        out,
        "        return {type_name}{{\n                {}}};\n    }}",
        args.join(",\n                ")
    );
}

/// How one value is written. Recursive, so a list of maps nests without a helper per shape.
fn write_value(out: &mut String, indent: &str, expr: &str, ty: &Ty, depth: usize) {
    match ty {
        Ty::Text => {
            let _ = writeln!(out, "{indent}Json::text(out, {expr});");
        }
        Ty::Int => {
            let _ = writeln!(out, "{indent}out += std::to_string({expr});");
        }
        Ty::Bool => {
            let _ = writeln!(out, "{indent}out += {expr} ? \"true\" : \"false\";");
        }
        Ty::Named(_) => {
            let _ = writeln!(out, "{indent}{expr}.write(out);");
        }
        Ty::List(inner) => {
            let _ = writeln!(out, "{indent}out += '[';");
            let _ = writeln!(
                out,
                "{indent}for (std::size_t index{depth} = 0; index{depth} < {expr}.size(); index{depth}++) {{"
            );
            let _ = writeln!(out, "{indent}    if (index{depth} > 0) {{");
            let _ = writeln!(out, "{indent}        out += ',';");
            let _ = writeln!(out, "{indent}    }}");
            write_value(
                out,
                &format!("{indent}    "),
                &format!("{expr}[index{depth}]"),
                inner,
                depth + 1,
            );
            let _ = writeln!(out, "{indent}}}");
            let _ = writeln!(out, "{indent}out += ']';");
        }
        Ty::Map(inner) => {
            let _ = writeln!(out, "{indent}out += '{{';");
            let _ = writeln!(out, "{indent}int n{depth} = 0;");
            let _ = writeln!(out, "{indent}for (const auto& entry{depth} : {expr}) {{");
            let _ = writeln!(
                out,
                "{indent}    n{depth} = Json::key(out, n{depth}, entry{depth}.first);"
            );
            write_value(
                out,
                &format!("{indent}    "),
                &format!("entry{depth}.second"),
                inner,
                depth + 1,
            );
            let _ = writeln!(out, "{indent}}}");
            let _ = writeln!(out, "{indent}out += '}}';");
        }
    }
}

/// The field by field body both `write` and `write_fields` are made of. A member is reached
/// through `this->` so that a local of the same name cannot be written instead: five wire field
/// names are also names this emitter uses for a local or a parameter.
fn write_fields(out: &mut String, indent: &str, fields: &[Field]) {
    for field in fields {
        let bare = name(&field.wire);
        let (expr, inner) = if field.required {
            (format!("this->{bare}"), indent.to_string())
        } else {
            (format!("(*this->{bare})"), format!("{indent}    "))
        };
        if !field.required {
            let _ = writeln!(out, "{indent}if (this->{bare}.has_value()) {{");
        }
        let _ = writeln!(
            out,
            "{inner}count = Json::key(out, count, \"{}\");",
            field.wire
        );
        write_value(out, &inner, &expr, &field.ty, 0);
        if !field.required {
            let _ = writeln!(out, "{indent}}}");
        }
    }
}

/// The `write` of a plain message. An absent optional is left out rather than written as null.
fn write_method(out: &mut String, fields: &[Field]) {
    let _ = writeln!(out, "    void write(std::string& out) const {{");
    if fields.is_empty() {
        let _ = writeln!(out, "        out += \"{{}}\";\n    }}");
        return;
    }
    let _ = writeln!(out, "        out += '{{';\n        int count = 0;");
    write_fields(out, "        ", fields);
    let _ = writeln!(out, "        out += '}}';\n    }}");
}

/// A plain message: members, reader, writer. An aggregate, so a screen writes `Tile{...}` and
/// `Tile tile;` leaves every member at the value its declaration gives it.
fn message(out: &mut String, type_name: &str, fields: &[Field]) {
    let _ = writeln!(out, "struct {type_name} {{");
    for field in fields {
        member(out, field);
    }
    if !fields.is_empty() {
        out.push('\n');
    }
    read_method(
        out,
        type_name,
        type_name,
        fields,
        &Reading {
            method: "read",
            opens: true,
            tail: None,
        },
    );
    out.push('\n');
    write_method(out, fields);
    out.push_str("};\n");
}

/// An open word: any word the server sends is kept, and the ones this build knows are constants.
fn words(out: &mut String, type_name: &str, list: &[String]) {
    // `value` is not in TAKEN, because it is a wire field name on real messages and renaming it
    // in all of them to protect this one shape would be noise. A word of that name would declare
    // the member twice, so the generator refuses instead.
    assert!(
        !list.iter().any(|word| constant(word) == "value"),
        "{type_name} publishes a word called value, which is the member an open word keeps"
    );
    let _ = writeln!(out, "struct {type_name} {{");
    let _ = writeln!(out, "    std::string value;\n");
    for word in list {
        let _ = writeln!(
            out,
            "    static constexpr std::string_view {} = \"{}\";",
            constant(word),
            literal(word)
        );
    }
    if !list.is_empty() {
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "    static {type_name} read(const Json::Value& document) {{\n        \
         return {type_name}{{Json::as_text(document)}};\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    void write(std::string& out) const {{\n        \
         Json::text(out, this->value);\n    }}\n}};"
    );
}

/// Checked text: the pattern is compiled once, on first use, and not once per value, because a row
/// of twenty tiles would otherwise pay for it twenty times. The constructor throws, which is what
/// makes a refused fixture fail to read, and it is the only way in, so there is no default one.
fn checked(out: &mut String, type_name: &str, pattern: &str) {
    let _ = writeln!(out, "struct {type_name} {{");
    let _ = writeln!(out, "    std::string value;\n");
    let _ = writeln!(
        out,
        "    explicit {type_name}(std::string text) : value(std::move(text)) {{\n        \
         if (!std::regex_match(this->value, shape())) {{\n            \
         throw Refused(\"not a {type_name}\");\n        }}\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    static {type_name} read(const Json::Value& document) {{\n        \
         return {type_name}(Json::as_text(document));\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    void write(std::string& out) const {{\n        \
         Json::text(out, this->value);\n    }}\n"
    );
    let _ = writeln!(out, "private:");
    let _ = writeln!(
        out,
        "    static const std::regex& shape() {{\n        \
         static const std::regex compiled(\"{}\", std::regex::ECMAScript);\n        \
         return compiled;\n    }}\n}};",
        literal(pattern)
    );
}

/// One fixed word, the type tag of a core message. Closed, unlike an open word: a tag this build
/// does not know is refused, which is what `CoreIn.wrong_type` asks for.
fn fixed(out: &mut String, type_name: &str, word: &str) {
    let _ = writeln!(out, "struct {type_name} {{");
    let _ = writeln!(
        out,
        "    static constexpr std::string_view word = \"{}\";\n",
        literal(word)
    );
    let _ = writeln!(
        out,
        "    static {type_name} read(const Json::Value& document) {{\n        \
         const std::string& found = Json::as_text(document);\n        \
         if (found != word) {{\n            \
         throw Refused(\"not a {type_name}: \" + found);\n        }}\n        \
         return {type_name}{{}};\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    void write(std::string& out) const {{\n        \
         Json::text(out, word);\n    }}\n}};"
    );
}

/// A tagged union: one struct per variant, one for a tag this build does not know, a `std::variant`
/// of the three, and a wrapper holding the common fields and that body.
fn union(out: &mut String, item: &Item, tag: &str, common: &[Field], variants: &[Variant]) {
    let base = &item.name;
    assert!(
        common.iter().all(|field| name(&field.wire) != "body"),
        "{base} has a common field called body, which is the member the wrapper keeps its variant in"
    );
    for variant in variants {
        comment(out, "", variant.doc.as_ref());
        let type_name = format!("{base}{}", upper_camel(&variant.word));
        let _ = writeln!(out, "struct {type_name} {{");
        for field in &variant.fields {
            member(out, field);
        }
        if !variant.fields.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "    /// The word this variant carries in `{tag}`.\n    \
             std::string_view word() const {{\n        return \"{}\";\n    }}\n",
            literal(&variant.word)
        );
        read_method(
            out,
            &type_name,
            base,
            &variant.fields,
            &Reading {
                method: "read_fields",
                opens: false,
                tail: None,
            },
        );
        out.push('\n');
        if variant.fields.is_empty() {
            // The parameters are unnamed, because a body that writes nothing would leave both
            // unused and -Wunused-parameter is in -Wextra.
            let _ = writeln!(
                out,
                "    void write_fields(std::string&, int&) const {{\n    }}"
            );
        } else {
            let _ = writeln!(
                out,
                "    void write_fields(std::string& out, int& count) const {{"
            );
            write_fields(out, "        ", &variant.fields);
            let _ = writeln!(out, "    }}");
        }
        out.push_str("};\n\n");
    }

    out.push_str("/// A tag this build does not know.\n");
    let _ = writeln!(out, "struct {base}Unknown {{");
    out.push_str("    /// The tag that was read.\n    std::string tag;\n\n");
    let _ = writeln!(
        out,
        "    std::string_view word() const {{\n        return this->tag;\n    }}\n"
    );
    let _ = writeln!(
        out,
        "    void write_fields(std::string&, int&) const {{\n    }}\n}};\n"
    );
    let names: Vec<String> = variants
        .iter()
        .map(|variant| format!("{base}{}", upper_camel(&variant.word)))
        .chain(std::iter::once(format!("{base}Unknown")))
        .collect();
    let _ = writeln!(
        out,
        "using {base}Body = std::variant<{}>;\n",
        names.join(", ")
    );
    comment(out, "", item.doc.as_ref());
    let _ = writeln!(out, "struct {base} {{");
    for field in common {
        member(out, field);
    }
    let _ = writeln!(out, "    {base}Body body;\n");
    read_method(
        out,
        base,
        base,
        common,
        &Reading {
            method: "read",
            opens: true,
            tail: Some("read_body(fields)"),
        },
    );
    out.push('\n');
    let _ = writeln!(out, "    void write(std::string& out) const {{");
    let _ = writeln!(out, "        out += '{{';\n        int count = 0;");
    let _ = writeln!(out, "        count = Json::key(out, count, \"{tag}\");");
    let _ = writeln!(
        out,
        "        Json::text(out, std::visit([](const auto& held) -> std::string_view {{ return held.word(); }}, this->body));"
    );
    write_fields(out, "        ", common);
    let _ = writeln!(
        out,
        "        std::visit([&out, &count](const auto& held) {{ held.write_fields(out, count); }}, this->body);"
    );
    let _ = writeln!(out, "        out += '}}';\n    }}\n");
    out.push_str("private:\n");
    let _ = writeln!(
        out,
        "    static {base}Body read_body(const Json::Value& fields) {{"
    );
    let _ = writeln!(
        out,
        "        const std::string& word = Json::as_text(*Json::need(fields, \"{tag}\", \"{base}\"));"
    );
    for variant in variants {
        let _ = writeln!(
            out,
            "        if (word == \"{}\") {{\n            return {base}{}::read_fields(fields);\n        }}",
            literal(&variant.word),
            upper_camel(&variant.word)
        );
    }
    let _ = writeln!(out, "        return {base}Unknown{{word}};\n    }}\n}};");
}

/// The whole file: one header, because a type set a screen reads needs no translation unit of its
/// own, and every function here is `inline` by being a member or a template.
#[must_use]
pub fn write(items: &[Item]) -> String {
    let items = resolved(items);
    let mut out = String::from("// Generated from the contract. Do not edit.\n\n");
    out.push_str("#ifndef STREAMS_CONTRACT_HPP\n#define STREAMS_CONTRACT_HPP\n\n");
    out.push_str("#include <cstddef>\n");
    out.push_str("#include <cstdint>\n");
    out.push_str("#include <map>\n");
    out.push_str("#include <optional>\n");
    out.push_str("#include <regex>\n");
    out.push_str("#include <stdexcept>\n");
    out.push_str("#include <string>\n");
    out.push_str("#include <string_view>\n");
    out.push_str("#include <utility>\n");
    out.push_str("#include <variant>\n");
    out.push_str("#include <vector>\n\n");
    out.push_str(
        "/// Every type the contract publishes, as plain aggregates a screen reads, each\n",
    );
    out.push_str(
        "/// with the reader and the writer the wire needs. Construction stays in :core\n",
    );
    out.push_str("/// and :ports; a surface reads what the core returns.\n");
    out.push_str("namespace contract {\n\n");
    let _ = writeln!(
        out,
        "inline constexpr std::string_view wire_version = \"{}\";",
        literal(WIRE_VERSION)
    );
    let _ = writeln!(out, "inline constexpr int abi_version = {ABI_VERSION};\n");
    out.push_str("/// Every address the apps open.\nstruct Address {\n");
    for (name, value) in contract::address::ALL {
        let _ = writeln!(
            out,
            "    static constexpr std::string_view {} = \"{}\";",
            constant(name),
            literal(value)
        );
    }
    out.push_str("\n    /// Where a tab lives, by the screen's word.\n");
    out.push_str("    static std::string_view of(std::string_view screen) {\n");
    for (name, value) in contract::address::SCREENS {
        let _ = writeln!(
            out,
            "        if (screen == \"{}\") {{\n            return \"{}\";\n        }}",
            literal(name),
            literal(value)
        );
    }
    let _ = writeln!(out, "        return {};\n    }}\n}};\n", constant("HOME"));
    out.push_str(CODEC);
    for item in ordered(&items) {
        out.push('\n');
        // A union writes its own doc comment onto the wrapper, which comes after its variants.
        if !matches!(item.shape, Shape::Union { .. }) {
            comment(&mut out, "", item.doc.as_ref());
        }
        match &item.shape {
            Shape::Struct(fields) => message(&mut out, &item.name, fields),
            Shape::Words(list) => words(&mut out, &item.name, list),
            Shape::Checked(pattern) => checked(&mut out, &item.name, pattern),
            Shape::Fixed(word) => fixed(&mut out, &item.name, word),
            Shape::Union {
                tag,
                common,
                variants,
            } => union(&mut out, item, tag, common, variants),
            Shape::Alias(inner) => {
                let _ = writeln!(out, "using {} = {};", item.name, ty(inner));
            }
        }
    }
    out.push_str("\n}  // namespace contract\n\n#endif  // STREAMS_CONTRACT_HPP\n");
    out
}
