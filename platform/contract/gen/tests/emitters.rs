//! The exact text the five type writers emit for the cases that have bitten.
//!
//! `gen/tests/generated.rs` is 372 lines about `openapi.json` and names no emitter, which is how
//! `kotlin.rs:107` shipped a pattern escape that left the double quote alone: there was nothing
//! that could fail. These tests build the hostile shapes by hand and compare the emitted text
//! exactly, so changing an emitter's output is a deliberate act and shows up here as a diff.
//!
//! The cases, each named from something real: a field called `class`, which is the only one of the
//! 290 wire field names that is a Java keyword, in a plain message *and* inside a union variant,
//! because those are two code paths; a field with underscores; an optional; a repeated; a map; a
//! nested message; a doc comment holding `*/`; a doc comment opening with `@`; an empty message;
//! an open word, which stays open; a closed tag enum, which does not; a union with common fields
//! and one without; an alias, which Java cannot express; and a pattern carrying `"`, `\\`, `$`, a
//! newline and a tab.
//!
//! The C++ cases are the same shapes again, because the same shapes bite in C++ and two of them
//! bite harder: a doc comment holding `*/` ends the declaration rather than reading oddly, and a
//! parameter the emitter leaves unused is an error, because the C++ arm of `check_types.py` is the
//! one that compiles with `-Werror`. Three more are C++'s alone: the declaration order a header
//! needs, the ALL CAPS names `<windows.h>` would eat, and the two shapes the generator refuses.

use contract_gen::shape::{Field, Item, Shape, Ty, Variant};

fn field(wire: &str, ty: Ty, required: bool) -> Field {
    Field {
        wire: wire.to_string(),
        ty,
        required,
        doc: None,
    }
}

/// What one item adds to a language's output, exactly.
///
/// Each emitter writes a header, then one block per item, then whatever closes the file, so what an
/// item contributes is what is left of `write(&[item])` once `write(&[])` is taken off both ends.
/// The assertion is what keeps this honest: if the two halves did not account for every byte of the
/// no-item output, the emitter would be weaving the item into its header — which `kotlin.rs` does
/// do, for a union, in its `contractJson` block — and this fails loudly instead of quietly
/// comparing the wrong text.
fn body(language: fn(&[Item]) -> String, item: &Item) -> String {
    let head = language(&[]);
    let whole = language(std::slice::from_ref(item));
    let front = head
        .bytes()
        .zip(whole.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let back = head
        .bytes()
        .rev()
        .zip(whole.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    assert!(
        whole.len() >= head.len() && front + back >= head.len(),
        "{}: the no-item output is not this output's head and tail",
        item.name
    );
    whole[front..front + (whole.len() - head.len())].to_string()
}

/// The hostile case of that name.
fn named(name: &str) -> Item {
    hostile()
        .into_iter()
        .find(|one| one.name == name)
        .expect("a case of that name")
}

fn hostile() -> Vec<Item> {
    vec![
        Item {
            name: "Hostile".to_string(),
            doc: Some("Holds */ which would end this comment early.".to_string()),
            shape: Shape::Struct(vec![
                field("class", Ty::Named("DeviceClass".to_string()), true),
                field("max_height", Ty::Int, true),
                field("cursor", Ty::Text, false),
                field(
                    "tiles",
                    Ty::List(Box::new(Ty::Named("Tile".to_string()))),
                    true,
                ),
                field("headers", Ty::Map(Box::new(Ty::Text)), false),
                field("live", Ty::Bool, false),
            ]),
        },
        Item {
            name: "Tagged".to_string(),
            doc: Some("@tag would read as a javadoc tag.".to_string()),
            shape: Shape::Struct(Vec::new()),
        },
        Item {
            name: "Rating".to_string(),
            doc: None,
            shape: Shape::Words(vec!["liked".to_string(), "two-thumbs".to_string()]),
        },
        Item {
            name: "Checked".to_string(),
            doc: None,
            shape: Shape::Checked("^a\"b\\c$d\n\t$".to_string()),
        },
        Item {
            name: "Quoted".to_string(),
            doc: None,
            shape: Shape::Checked("^a\"b\\c$d$".to_string()),
        },
        Item {
            name: "CoreInTag".to_string(),
            doc: None,
            shape: Shape::Fixed("core_in".to_string()),
        },
        Item {
            name: "Effect".to_string(),
            doc: None,
            shape: Shape::Union {
                tag: "kind".to_string(),
                common: vec![field("id", Ty::Text, true), field("note", Ty::Text, false)],
                variants: vec![Variant {
                    word: "http".to_string(),
                    doc: Some("A path.".to_string()),
                    fields: vec![
                        field("class", Ty::Named("HttpClass".to_string()), true),
                        field("deadline_ms", Ty::Int, false),
                    ],
                }],
            },
        },
        Item {
            name: "Bare".to_string(),
            doc: None,
            shape: Shape::Union {
                tag: "kind".to_string(),
                common: Vec::new(),
                variants: vec![Variant {
                    word: "one".to_string(),
                    doc: None,
                    fields: Vec::new(),
                }],
            },
        },
        Item {
            name: "Cursor".to_string(),
            doc: Some("An opaque position.".to_string()),
            shape: Shape::Alias(Ty::Text),
        },
    ]
}

/// A field named `class` is the one wire name of the 290 that is a Java keyword. Java cannot
/// escape an identifier, and `getClass()` is final on `Object`, so both the field and the accessor
/// keep a trailing underscore. This case also carries the underscored, optional, repeated, map and
/// nested fields, and a doc comment holding `*/`.
#[test]
fn java_renames_a_keyword_field_in_a_message_and_in_its_accessor() {
    assert_eq!(
        body(contract_gen::java::write, &named("Hostile")),
        r#"
    /** Holds * / which would end this comment early. */
    public static final class Hostile implements Wire {
        private final DeviceClass class_;
        private final long maxHeight;
        private final String cursor;
        private final List<Tile> tiles;
        private final Map<String, String> headers;
        private final Boolean live;

        public Hostile(DeviceClass class_, long maxHeight, String cursor, List<Tile> tiles, Map<String, String> headers, Boolean live) {
            this.class_ = class_;
            this.maxHeight = maxHeight;
            this.cursor = cursor;
            this.tiles = tiles;
            this.headers = headers;
            this.live = live;
        }

        public DeviceClass getClass_() {
            return class_;
        }

        public long getMaxHeight() {
            return maxHeight;
        }

        public boolean hasCursor() {
            return cursor != null;
        }

        public String getCursor() {
            return cursor == null ? "" : cursor;
        }

        public List<Tile> getTilesList() {
            return tiles == null ? Collections.<Tile>emptyList() : tiles;
        }

        public int getTilesCount() {
            return tiles == null ? 0 : tiles.size();
        }

        public Tile getTiles(int $index) {
            return tiles.get($index);
        }

        public Map<String, String> getHeadersMap() {
            return headers == null ? Collections.<String, String>emptyMap() : headers;
        }

        public int getHeadersCount() {
            return headers == null ? 0 : headers.size();
        }

        public boolean hasLive() {
            return live != null;
        }

        public boolean getLive() {
            return live != null && live;
        }

        public static Hostile read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "class", "Hostile");
            Object $raw1 = Json.need($fields, "max_height", "Hostile");
            Object $raw2 = Json.at($fields, "cursor");
            Object $raw3 = Json.need($fields, "tiles", "Hostile");
            Object $raw4 = Json.at($fields, "headers");
            Object $raw5 = Json.at($fields, "live");
            return new Hostile(
                    DeviceClass.read($raw0),
                    Json.asLong($raw1),
                    $raw2 == null ? null : Json.asText($raw2),
                    Json.list($raw3, $x0 -> Tile.read($x0)),
                    $raw4 == null ? null : Json.map($raw4, $x0 -> Json.asText($x0)),
                    $raw5 == null ? null : Boolean.valueOf(Json.asBool($raw5)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $count = Json.key($out, $count, "max_height");
            $out.append(maxHeight);
            if (cursor != null) {
                $count = Json.key($out, $count, "cursor");
                Json.text($out, cursor);
            }
            $count = Json.key($out, $count, "tiles");
            $out.append('[');
            for (int $i0 = 0; $i0 < tiles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                tiles.get($i0).write($out);
            }
            $out.append(']');
            if (headers != null) {
                $count = Json.key($out, $count, "headers");
                $out.append('{');
                int $n0 = 0;
                for (Map.Entry<String, String> $e0 : headers.entrySet()) {
                    $n0 = Json.key($out, $n0, $e0.getKey());
                    Json.text($out, $e0.getValue());
                }
                $out.append('}');
            }
            if (live != null) {
                $count = Json.key($out, $count, "live");
                $out.append(live);
            }
            $out.append('}');
        }
    }
"#
    );
}
/// An empty message still has to be an object on the wire, and a doc comment opening with `@`
/// would read as a javadoc block tag.
#[test]
fn java_writes_an_empty_message_and_defuses_a_javadoc_tag() {
    assert_eq!(
        body(contract_gen::java::write, &named("Tagged")),
        r#"
    /** {@literal @}tag would read as a javadoc tag. */
    public static final class Tagged implements Wire {
        public Tagged() {
        }

        public static Tagged read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            return new Tagged();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append("{}");
        }
    }
"#
    );
}
/// An open word keeps any word the server sends, so this is a class with constants and not an
/// enum. It is the shape that `@JvmInline` made uncallable from Java.
#[test]
fn java_keeps_an_open_word_open() {
    assert_eq!(
        body(contract_gen::java::write, &named("Rating")),
        r#"
    public static final class Rating implements Wire {
        public static final Rating LIKED = new Rating("liked");
        public static final Rating TWO_THUMBS = new Rating("two-thumbs");

        private final String value;

        public Rating(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static Rating read(Object $json) {
            return new Rating(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof Rating && (((Rating) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }
"#
    );
}
/// The pattern is compiled once into a static, not per value, because a row of twenty tiles would
/// otherwise pay for it twenty times. The literal escapes the backslash, the quote, the newline and
/// the tab, and leaves `$` alone: Java has no interpolation and escaping it would corrupt the
/// expression. The constructor throws, which is what makes a refused fixture fail to parse.
#[test]
fn java_compiles_a_checked_pattern_once_and_escapes_the_literal() {
    assert_eq!(
        body(contract_gen::java::write, &named("Checked")),
        r#"
    public static final class Checked implements Wire {
        private static final Pattern SHAPE = Pattern.compile("^a\"b\\c$d\n\t$");

        private final String value;

        public Checked(String value) {
            if (!SHAPE.matcher(value).matches()) {
                throw new IllegalArgumentException("not a Checked");
            }
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static Checked read(Object $json) {
            return new Checked(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof Checked && (((Checked) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }
"#
    );
}
/// A fixed tag is closed, unlike an open word: a value this build does not know is refused. That
/// is what the `CoreIn.wrong_type` fixture asks for, a core_in message whose type says core_out.
#[test]
fn java_refuses_a_word_a_closed_tag_does_not_know() {
    assert_eq!(
        body(contract_gen::java::write, &named("CoreInTag")),
        r#"
    public enum CoreInTag implements Wire {
        VALUE("core_in");

        private final String wire;

        CoreInTag(String wire) {
            this.wire = wire;
        }

        public String wire() {
            return wire;
        }

        public static CoreInTag read(Object $json) {
            String $word = Json.asText($json);
            if (!"core_in".equals($word)) {
                throw new IllegalArgumentException("not a CoreInTag: " + $word);
            }
            return VALUE;
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, wire);
        }
    }
"#
    );
}
/// A union reaches its fields through a different path from a plain message, so the keyword rename
/// is proved again here: two of the seven schemas carrying a `class` field are union variants. The
/// unknown variant keeps the tag it could not place and falls back on each common field, which is
/// what lets a tolerated fixture read on a build that predates a new variant.
#[test]
fn java_carries_a_keyword_field_through_a_union_variant_and_keeps_an_unknown_tag() {
    assert_eq!(
        body(contract_gen::java::write, &named("Effect")),
        r#"
    public abstract static class Effect implements Wire {
        public abstract String getId();

        public abstract boolean hasNote();

        public abstract String getNote();

        public static Effect read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "kind", "Effect"));
            if ("http".equals($tag)) {
                return EffectHttp.read($json);
            }
            return EffectUnknown.read($json);
        }
    }

    /** A path. */
    public static final class EffectHttp extends Effect {
        private final String id;
        private final String note;
        private final HttpClass class_;
        private final Long deadlineMs;

        public EffectHttp(String id, String note, HttpClass class_, Long deadlineMs) {
            this.id = id;
            this.note = note;
            this.class_ = class_;
            this.deadlineMs = deadlineMs;
        }

        @Override
        public String getId() {
            return id;
        }

        @Override
        public boolean hasNote() {
            return note != null;
        }

        @Override
        public String getNote() {
            return note == null ? "" : note;
        }

        public HttpClass getClass_() {
            return class_;
        }

        public boolean hasDeadlineMs() {
            return deadlineMs != null;
        }

        public long getDeadlineMs() {
            return deadlineMs == null ? 0L : deadlineMs;
        }

        public static EffectHttp read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"http".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectHttp");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.at($fields, "note");
            Object $raw2 = Json.need($fields, "class", "Effect");
            Object $raw3 = Json.at($fields, "deadline_ms");
            return new EffectHttp(
                    Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    HttpClass.read($raw2),
                    $raw3 == null ? null : Long.valueOf(Json.asLong($raw3)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "http");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            if (note != null) {
                $count = Json.key($out, $count, "note");
                Json.text($out, note);
            }
            $count = Json.key($out, $count, "class");
            class_.write($out);
            if (deadlineMs != null) {
                $count = Json.key($out, $count, "deadline_ms");
                $out.append(deadlineMs);
            }
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class EffectUnknown extends Effect {
        private final String tag;
        private final String rawId;
        private final String rawNote;

        public EffectUnknown(String tag, String rawId, String rawNote) {
            this.tag = tag;
            this.rawId = rawId;
            this.rawNote = rawNote;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        @Override
        public String getId() {
            return rawId == null ? "" : rawId;
        }

        @Override
        public boolean hasNote() {
            return rawNote != null;
        }

        @Override
        public String getNote() {
            return rawNote == null ? "" : rawNote;
        }

        public static EffectUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "kind");
            Object $raw1 = Json.at($fields, "id");
            Object $raw2 = Json.at($fields, "note");
            return new EffectUnknown(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    $raw2 == null ? null : Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, tag);
            if (rawId != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, rawId);
            }
            if (rawNote != null) {
                $count = Json.key($out, $count, "note");
                Json.text($out, rawNote);
            }
            $out.append('}');
        }
    }
"#
    );
}
/// A union with no common fields has nothing to declare on its base and nothing to keep on its
/// unknown variant but the tag.
#[test]
fn java_writes_a_union_with_no_common_fields() {
    assert_eq!(
        body(contract_gen::java::write, &named("Bare")),
        r#"
    public abstract static class Bare implements Wire {
        public static Bare read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "kind", "Bare"));
            if ("one".equals($tag)) {
                return BareOne.read($json);
            }
            return BareUnknown.read($json);
        }
    }

    public static final class BareOne extends Bare {
        public BareOne() {
        }

        public static BareOne read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"one".equals(Json.asText(Json.need($fields, "kind", "Bare")))) {
                throw new IllegalArgumentException("not a BareOne");
            }
            return new BareOne();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "one");
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class BareUnknown extends Bare {
        private final String tag;

        public BareUnknown(String tag) {
            this.tag = tag;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        public static BareUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "kind");
            return new BareUnknown(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, tag);
            $out.append('}');
        }
    }
"#
    );
}
/// Java has no type alias. Every use of the alias is emitted as the type it names, and the
/// definition becomes a line comment, which also keeps the name present in the file for the test
/// that every definition is named. A javadoc here would attach to nothing and javac says so.
#[test]
fn java_writes_an_alias_as_a_comment_because_java_has_no_alias() {
    assert_eq!(
        body(contract_gen::java::write, &named("Cursor")),
        r#"
    // An opaque position.
    // Cursor is String. Java has no type alias, so every use of it reads String.
"#
    );
}
/// One pattern, five languages, five different right answers, and this is the test that would have
/// caught `kotlin.rs:107`. Java escapes the quote and the backslash and leaves `$`; Kotlin escapes
/// all three, because `$` interpolates there; Swift escapes the quote and the backslash, and
/// doubling the backslash is also what stops `\(` reading as interpolation; C++ escapes the quote
/// and the backslash and leaves `$`, which is Java's answer reached a different way, and it is the
/// only one of the five that also escapes a control character as octal, because `\x1b` in C++
/// swallows a following hex digit and `\033` cannot; TypeScript carries no pattern at all, which
/// is why `check_patterns.py` has no TypeScript line to keep in step.
#[test]
fn the_five_languages_each_escape_one_pattern_their_own_way() {
    assert_eq!(
        body(contract_gen::java::write, &named("Quoted")),
        r#"
    public static final class Quoted implements Wire {
        private static final Pattern SHAPE = Pattern.compile("^a\"b\\c$d$");

        private final String value;

        public Quoted(String value) {
            if (!SHAPE.matcher(value).matches()) {
                throw new IllegalArgumentException("not a Quoted");
            }
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static Quoted read(Object $json) {
            return new Quoted(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof Quoted && (((Quoted) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }
"#
    );
    assert_eq!(
        body(contract_gen::kotlin::write, &named("Quoted")),
        r#"
@Serializable
@JvmInline
value class Quoted(val value: String) {
    init {
        require(Regex("^a\"b\\c\$d\$").matches(value)) { "not a Quoted" }
    }
}
"#
    );
    assert_eq!(
        body(contract_gen::swift::write, &named("Quoted")),
        r#"
public struct Quoted: Codable, Hashable, Sendable {
    public let rawValue: String

    public init(_ rawValue: String) throws {
        guard rawValue.range(of: "^a\"b\\c$d$", options: .regularExpression) != nil else {
            throw ContractError.refused("Quoted")
        }
        self.rawValue = rawValue
    }

    public init(from decoder: Decoder) throws {
        try self.init(try decoder.singleValueContainer().decode(String.self))
    }

    public func encode(to encoder: Encoder) throws {
        var single = encoder.singleValueContainer()
        try single.encode(rawValue)
    }
}
"#
    );
    assert_eq!(
        body(contract_gen::cpp::write, &named("Quoted")),
        r#"struct Quoted {
    std::string value;

    explicit Quoted(std::string text) : value(std::move(text)) {
        if (!std::regex_match(this->value, shape())) {
            throw Refused("not a Quoted");
        }
    }

    static Quoted read(const Json::Value& document) {
        return Quoted(Json::as_text(document));
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }

private:
    static const std::regex& shape() {
        static const std::regex compiled("^a\"b\\c$d$", std::regex::ECMAScript);
        return compiled;
    }
};

"#
    );
    assert_eq!(
        body(contract_gen::typescript::write, &named("Quoted")),
        r#"
export type Quoted = string;
"#
    );
}

/// `class` is the one wire field name of the 290 that is a C++ keyword, the same one Java
/// trips on and for a different reason: C++ cannot escape an identifier at all. Every other name is
/// carried through untouched, because the wire names are snake case and so is C++, so `max_height`
/// is `max_height` here and `maxHeight` in the other three languages.
///
/// This case also carries the optional, the repeated, the map and the nested fields, and a doc
/// comment holding `*/`. The comment is a `///` line and not a `/** */` block for exactly that
/// text: in C++ a comment that closes itself early does not merely read oddly, it ends the
/// declaration, and what follows is parsed as code.
#[test]
fn cpp_renames_a_keyword_field_in_a_message_and_keeps_every_other_wire_name() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("Hostile")),
        r#"/// Holds */ which would end this comment early.
struct Hostile {
    DeviceClass class_;
    std::int64_t max_height = 0;
    std::optional<std::string> cursor;
    std::vector<Tile> tiles;
    std::optional<std::map<std::string, std::string>> headers;
    std::optional<bool> live;

    static Hostile read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "class", "Hostile");
        const Json::Value* raw1 = Json::need(fields, "max_height", "Hostile");
        const Json::Value* raw2 = Json::at(fields, "cursor");
        const Json::Value* raw3 = Json::need(fields, "tiles", "Hostile");
        const Json::Value* raw4 = Json::at(fields, "headers");
        const Json::Value* raw5 = Json::at(fields, "live");
        return Hostile{
                DeviceClass::read(*raw0),
                Json::as_long(*raw1),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2)),
                Json::list<Tile>(*raw3, [](const Json::Value& x0) { return Tile::read(x0); }),
                raw4 == nullptr ? std::optional<std::map<std::string, std::string>>() : std::optional<std::map<std::string, std::string>>(Json::map<std::string>(*raw4, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw5 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw5))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "class");
        this->class_.write(out);
        count = Json::key(out, count, "max_height");
        out += std::to_string(this->max_height);
        if (this->cursor.has_value()) {
            count = Json::key(out, count, "cursor");
            Json::text(out, (*this->cursor));
        }
        count = Json::key(out, count, "tiles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->tiles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->tiles[index0].write(out);
        }
        out += ']';
        if (this->headers.has_value()) {
            count = Json::key(out, count, "headers");
            out += '{';
            int n0 = 0;
            for (const auto& entry0 : (*this->headers)) {
                n0 = Json::key(out, n0, entry0.first);
                Json::text(out, entry0.second);
            }
            out += '}';
        }
        if (this->live.has_value()) {
            count = Json::key(out, count, "live");
            out += (*this->live) ? "true" : "false";
        }
        out += '}';
    }
};

"#
    );
}
/// An empty message still has to be an object on the wire, so the reader asks for one and binds
/// nothing: an unused local is an error under -Wall and the Java emitter's `Map $fields = ...` has
/// no counterpart here. A doc comment opening with `@` needs no defusing, unlike Java's.
#[test]
fn cpp_writes_an_empty_message_and_opens_the_document_without_binding_it() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("Tagged")),
        r#"/// @tag would read as a javadoc tag.
struct Tagged {
    static Tagged read(const Json::Value& document) {
        Json::object(document);
        return Tagged{};
    }

    void write(std::string& out) const {
        out += "{}";
    }
};

"#
    );
}
/// An open word keeps any word the server sends, so this is a struct with constants and not an
/// enum. The constants are lower case, which is the one place this emitter departs from Java on
/// purpose: `<windows.h>` defines `DELETE` and `ERROR` as object-like macros, a macro ignores both
/// namespace and class scope, and of the 185 words the contract publishes `DELETE`, `delete` and
/// `error` would have become constants of exactly those names.
#[test]
fn cpp_keeps_an_open_word_open_and_writes_no_all_caps_constant() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("Rating")),
        r#"struct Rating {
    std::string value;

    static constexpr std::string_view liked = "liked";
    static constexpr std::string_view two_thumbs = "two-thumbs";

    static Rating read(const Json::Value& document) {
        return Rating{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

"#
    );
}
/// The pattern is compiled on first use into a function local static, and not once per value,
/// because a row of twenty tiles would otherwise pay for it twenty times. The literal escapes the
/// backslash, the quote, the newline and the tab and leaves `$` alone, as Java does and Kotlin must
/// not. The constructor throws and is the only way in, so there is no default one and no value of
/// this type exists that the pattern has not seen.
#[test]
fn cpp_compiles_a_checked_pattern_once_and_escapes_the_literal() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("Checked")),
        r#"struct Checked {
    std::string value;

    explicit Checked(std::string text) : value(std::move(text)) {
        if (!std::regex_match(this->value, shape())) {
            throw Refused("not a Checked");
        }
    }

    static Checked read(const Json::Value& document) {
        return Checked(Json::as_text(document));
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }

private:
    static const std::regex& shape() {
        static const std::regex compiled("^a\"b\\c$d\n\t$", std::regex::ECMAScript);
        return compiled;
    }
};

"#
    );
}
/// A fixed tag is closed, unlike an open word: a value this build does not know is refused. That
/// is what the `CoreIn.wrong_type` fixture asks for, a core_in message whose type says core_out.
#[test]
fn cpp_refuses_a_word_a_closed_tag_does_not_know() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("CoreInTag")),
        r#"struct CoreInTag {
    static constexpr std::string_view word = "core_in";

    static CoreInTag read(const Json::Value& document) {
        const std::string& found = Json::as_text(document);
        if (found != word) {
            throw Refused("not a CoreInTag: " + found);
        }
        return CoreInTag{};
    }

    void write(std::string& out) const {
        Json::text(out, word);
    }
};

"#
    );
}
/// A union reaches its fields through a different path from a plain message, so the keyword rename
/// is proved again here: two of the seven schemas carrying a `class` field are union variants.
///
/// The shape is not Java's. Java returns an abstract base and C++ would need a heap allocation to
/// do the same, so the variants are their own structs and the union is the common fields plus a
/// `std::variant` body, which is what `swift.rs` already does. The wrapper writes the tag out of
/// whichever variant it holds, which is how the unknown one carries a tag this build cannot place
/// and still writes the document back unchanged.
#[test]
fn cpp_carries_a_keyword_field_through_a_union_variant_and_keeps_an_unknown_tag() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("Effect")),
        r#"/// A path.
struct EffectHttp {
    HttpClass class_;
    std::optional<std::int64_t> deadline_ms;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "http";
    }

    static EffectHttp read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "class", "Effect");
        const Json::Value* raw1 = Json::at(fields, "deadline_ms");
        return EffectHttp{
                HttpClass::read(*raw0),
                raw1 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw1))};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "class");
        this->class_.write(out);
        if (this->deadline_ms.has_value()) {
            count = Json::key(out, count, "deadline_ms");
            out += std::to_string((*this->deadline_ms));
        }
    }
};

/// A tag this build does not know.
struct EffectUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using EffectBody = std::variant<EffectHttp, EffectUnknown>;

struct Effect {
    std::string id;
    std::optional<std::string> note;
    EffectBody body;

    static Effect read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "id", "Effect");
        const Json::Value* raw1 = Json::at(fields, "note");
        return Effect{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "kind");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        if (this->note.has_value()) {
            count = Json::key(out, count, "note");
            Json::text(out, (*this->note));
        }
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static EffectBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "kind", "Effect"));
        if (word == "http") {
            return EffectHttp::read_fields(fields);
        }
        return EffectUnknown{word};
    }
};

"#
    );
}
/// A union with no common fields is the body alone, and a variant with no fields of its own reads
/// nothing out of the object it is handed: that parameter goes unnamed, because an unused one is an
/// error under -Wextra and this is the only emitter whose output is compiled with -Werror.
#[test]
fn cpp_writes_a_union_with_no_common_fields() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("Bare")),
        r#"struct BareOne {
    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "one";
    }

    static BareOne read_fields(const Json::Value&) {
        return BareOne{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// A tag this build does not know.
struct BareUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using BareBody = std::variant<BareOne, BareUnknown>;

struct Bare {
    BareBody body;

    static Bare read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        return Bare{
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "kind");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static BareBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "kind", "Bare"));
        if (word == "one") {
            return BareOne::read_fields(fields);
        }
        return BareUnknown{word};
    }
};

"#
    );
}
/// C++ has a type alias, so unlike Java this is a declaration and not a comment. It is still
/// resolved away in every field that uses it: `using Cursor = std::string` is transparent, so a
/// field typed `Cursor` would ask for `Cursor::read`, which is `std::string::read` and does not
/// exist. The alias is declared for anyone who wants to name it and used by nothing.
#[test]
fn cpp_writes_an_alias_as_a_using_declaration() {
    assert_eq!(
        body(contract_gen::cpp::write, &named("Cursor")),
        r#"/// An opaque position.
using Cursor = std::string;

"#
    );
}

/// Every named type a definition holds, read from the shapes and not from the emitter, so this
/// asks the question rather than repeating the answer.
fn holds(item: &Item) -> Vec<String> {
    fn named(ty: &Ty, found: &mut Vec<String>) {
        match ty {
            Ty::Named(name) => found.push(name.clone()),
            Ty::List(inner) | Ty::Map(inner) => named(inner, found),
            _ => {}
        }
    }
    let mut found = Vec::new();
    match &item.shape {
        Shape::Struct(fields) => fields.iter().for_each(|field| named(&field.ty, &mut found)),
        Shape::Alias(inner) => named(inner, &mut found),
        Shape::Union {
            common, variants, ..
        } => {
            common.iter().for_each(|field| named(&field.ty, &mut found));
            variants
                .iter()
                .flat_map(|variant| &variant.fields)
                .for_each(|field| named(&field.ty, &mut found));
        }
        _ => {}
    }
    found
}

/// Where the header declares a name. A union is declared across several structs and the first of
/// them is where its block starts, because everything it holds has to come before that.
fn declared(text: &str, item: &Item) -> usize {
    let at = |name: &str| {
        text.find(&format!("\nstruct {name} {{"))
            .or_else(|| text.find(&format!("\nusing {name} = ")))
            .unwrap_or_else(|| panic!("the header declares no {name}"))
    };
    let mut found = at(&item.name);
    if let Shape::Union { variants, .. } = &item.shape {
        for variant in variants {
            let name = format!(
                "{}{}",
                item.name,
                contract_gen::shape::upper_camel(&variant.word)
            );
            found = found.min(at(&name));
        }
    }
    found
}

/// The one thing C++ asks for that no other emitted language does: a type must be complete before
/// it is held by value, and `shape::items` hands them over in alphabetical order. A header in that
/// order does not compile, so `cpp.rs` walks the definitions depth first. This asserts the result
/// over the real contract: every type a definition holds is declared earlier in the file.
///
/// `check_types.py` proves the same thing far harder, by compiling the header, but only on a
/// machine that has a C++ compiler. This holds on every machine.
#[test]
fn every_type_the_generated_header_holds_is_declared_before_the_type_holding_it() {
    let text = contract_gen::generate(&["ignored".to_string()])["types/Contract.hpp"].clone();
    let (defs, _) = contract_gen::schemas::definitions();
    let items = contract_gen::shape::items(&defs);
    // An alias is resolved away in every field that uses it, so where it sits is nobody's
    // business: `using Cursor = std::string` is transparent and no field is typed by that name.
    let aliases: Vec<&str> = items
        .iter()
        .filter(|item| matches!(item.shape, Shape::Alias(_)))
        .map(|item| item.name.as_str())
        .collect();
    let mut proved = 0;
    for item in &items {
        let here = declared(&text, item);
        for wanted in holds(item) {
            if aliases.contains(&wanted.as_str()) {
                continue;
            }
            let there = items
                .iter()
                .find(|other| other.name == wanted)
                .map(|other| declared(&text, other))
                .unwrap_or_else(|| panic!("no definition called {wanted}"));
            assert!(
                there < here,
                "{}: {wanted} is declared after it, at {there} and {here}",
                item.name
            );
            proved += 1;
        }
    }
    assert!(proved > 200, "{proved}");
}

/// Not one name this header declares is ALL CAPS, and that is a rule about Windows rather than a
/// matter of taste. `<windows.h>` defines `DELETE` (winnt.h), `ERROR` (wingdi.h), `IN`, `OUT` and
/// `OPTIONAL` as object-like macros, and a macro ignores namespaces and class scope: a constant of
/// that name is replaced by a number wherever the header is included after it. Three of the 185
/// words the contract publishes — `DELETE`, `delete` and `error` — land on two of those under
/// Java's uppercasing rule, and nothing on this Mac would ever have said so.
///
/// The include guard is the one ALL CAPS name in the file, because a guard must be one.
#[test]
fn no_name_the_generated_header_declares_is_all_caps() {
    let text = contract_gen::generate(&["ignored".to_string()])["types/Contract.hpp"].clone();
    let mut read = 0;
    for line in text.lines() {
        if line.starts_with("#ifndef") || line.starts_with("#define") || line.starts_with("#endif")
        {
            continue;
        }
        // Comments and string literals carry words like JSON and UTF-8 and are not declarations.
        let code = line.split("//").next().unwrap_or_default();
        let code = code.split("/*").next().unwrap_or_default();
        let code = code.split('"').next().unwrap_or_default();
        read += 1;
        let mut word = String::new();
        for here in code.chars().chain(std::iter::once(' ')) {
            if here.is_ascii_alphanumeric() || here == '_' {
                word.push(here);
                continue;
            }
            let shouting = word.len() > 1
                && word.starts_with(|first: char| first.is_ascii_uppercase())
                && !word.chars().any(|each| each.is_ascii_lowercase());
            assert!(!shouting, "{word} in: {line}");
            word.clear();
        }
    }
    assert!(read > 5000, "{read}");
}

/// C++ cannot declare a cycle by value, and nothing in the emitter can paper over that: a type
/// holding one that holds it back needs an indirection the contract does not describe. The
/// definitions are acyclic today. The day they are not, the generator says so by name instead of
/// writing a header that cannot compile.
#[test]
#[should_panic(expected = "holds a cycle")]
fn cpp_refuses_a_contract_that_holds_a_cycle() {
    let ring = vec![
        Item {
            name: "Left".to_string(),
            doc: None,
            shape: Shape::Struct(vec![field("right", Ty::Named("Right".to_string()), true)]),
        },
        Item {
            name: "Right".to_string(),
            doc: None,
            shape: Shape::Struct(vec![field("left", Ty::Named("Left".to_string()), true)]),
        },
    ];
    let _ = contract_gen::cpp::write(&ring);
}

/// The union wrapper keeps its variant in a member called `body`, and there is a wire field of
/// that name in the contract today — on `ScreenBody`, which is a plain message, and on
/// `EffectHttp`, which is a variant. Either is harmless. A *common* field of a union called `body`
/// would declare the member twice, so the generator refuses rather than emitting it.
#[test]
#[should_panic(expected = "common field called body")]
fn cpp_refuses_a_union_whose_common_field_is_called_body() {
    let clash = vec![Item {
        name: "Clash".to_string(),
        doc: None,
        shape: Shape::Union {
            tag: "kind".to_string(),
            common: vec![field("body", Ty::Text, true)],
            variants: vec![Variant {
                word: "one".to_string(),
                doc: None,
                fields: Vec::new(),
            }],
        },
    }];
    let _ = contract_gen::cpp::write(&clash);
}

/// The one escape C++ needs that no other emitted language does. A hex escape is greedy: `"\x1bf"`
/// is one character and not two, because the `f` joins the escape, so a pattern carrying a control
/// character followed by a hex digit would compile into a different expression without a word from
/// the compiler. Three digit octal cannot run on. No description and no pattern in the contract
/// carries a control character today, so this case is built by hand.
#[test]
fn cpp_escapes_a_control_character_as_octal_because_hex_would_run_on() {
    let item = Item {
        name: "Escaped".to_string(),
        doc: None,
        shape: Shape::Checked("^\u{1b}f$".to_string()),
    };
    let text = body(contract_gen::cpp::write, &item);
    let line = text
        .lines()
        .find(|line| line.contains("std::regex compiled"))
        .expect("a compiled pattern");
    assert_eq!(
        line.trim(),
        r#"static const std::regex compiled("^\033f$", std::regex::ECMAScript);"#
    );
}

/// An open word keeps its word in a member called `value`, so a word of that name would declare
/// the member twice. There is a wire *field* called `value` in the contract today and it is
/// harmless: it is a field of a plain message, which keeps no such member.
#[test]
#[should_panic(expected = "publishes a word called value")]
fn cpp_refuses_an_open_word_called_value() {
    let clash = vec![Item {
        name: "Clash".to_string(),
        doc: None,
        shape: Shape::Words(vec!["value".to_string()]),
    }];
    let _ = contract_gen::cpp::write(&clash);
}

/// The gate this whole emitter exists to open, asserted over the real contract rather than a hand
/// built case: no name a Java source file would have to type carries a hyphen. `javap` finds 184
/// such getters in the Kotlin output (note 210) and a hyphen is not a legal Java identifier, so
/// each one is a line that cannot be written. javac proves this too, when there is a javac.
#[test]
fn no_declaration_in_the_generated_java_carries_a_hyphen() {
    let text = contract_gen::generate(&["ignored".to_string()])["types/Contract.java"].clone();
    let mut declarations = 0;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("public ") && !trimmed.starts_with("private ") {
            continue;
        }
        declarations += 1;
        let upto = trimmed.split('"').next().unwrap_or_default();
        let upto = upto.split('(').next().unwrap_or_default();
        assert!(!upto.contains('-'), "{line}");
    }
    assert!(declarations > 2000, "{declarations}");
}
