//! The exact text the four type writers emit for the cases that have bitten.
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
/// One pattern, four languages, four different right answers, and this is the test that would have
/// caught `kotlin.rs:107`. Java escapes the quote and the backslash and leaves `$`; Kotlin escapes
/// all three, because `$` interpolates there; Swift escapes the quote and the backslash, and
/// doubling the backslash is also what stops `\(` reading as interpolation; TypeScript carries no
/// pattern at all, which is why `check_patterns.py` has no TypeScript line to keep in step.
#[test]
fn the_four_languages_each_escape_one_pattern_their_own_way() {
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
        body(contract_gen::typescript::write, &named("Quoted")),
        r#"
export type Quoted = string;
"#
    );
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
