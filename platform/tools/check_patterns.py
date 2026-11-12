#!/usr/bin/env python3
"""The generated clients refuse exactly what the core refuses.

A string type on the wire carries two statements of one rule: the Rust reader (`TitleKey::parse`,
`CalendarDay::parse`, `WireDate::parse`) and a regular expression the schema publishes and every
generated client compiles into itself. Written by two hands, they drift, and they had: the Kotlin
and Swift `TitleKey` took `movie:9999999999` while Rust refuses anything above `MAX_TITLE_ID`, so
an id too large for the core could be built on a device and only the server would say no.

This proves the two agree, three ways:

  one copy      every patterned type's expression is the same text in the standalone schema, in
                the OpenAPI document, in Contract.java, in Contract.kt, in Contract.swift and in
                Contract.hpp
  one corpus    every fixture of a patterned type, which Rust reads in `contract`'s own tests, gets
                the same verdict from the expression as the folder it sits in
  one bound     the expression accepts the largest id the Rust constant allows and refuses the next
                one, with the constant read from the Rust source and nothing written down here

It also refuses an expression using anything outside the subset Python, Java, ICU and ECMAScript
all read the same way, because this file judges those expressions with Python's engine and must not
be judging a different language from the one the client compiles.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "platform" / "contract"
GENERATED = CONTRACT / "generated"
TYPES = GENERATED / "types"

# The wire date of the fixtures this contract is at, read from the Rust source so this file holds
# no second copy of it.
WIRE_VERSION = re.compile(r'pub const WIRE_VERSION: &str = "([0-9-]+)"')

# A Rust `pub const NAME: u32 = <expression>;`, where the expression is shifts, arithmetic and
# numbers. Nothing else is accepted, so this can never evaluate something surprising.
RUST_CONST = r"pub const {name}: u32 = ([-+*() 0-9<>]+);"

# Each patterned type with a numeric bound, the Rust constant that is the bound, the file that
# declares it, and how a value of that size is written on the wire. The largest allowed value must
# match and the next one must not.
BOUNDS = (("TitleKey", "MAX_TITLE_ID", "src/title.rs", "movie:{id}"),)

# What a pattern may be made of. These four engines agree on anchors, classes, bounded repeats,
# alternation and a non-capturing group; they do not agree on `\d` inside unicode input, on
# lookbehind, or on inline flags, so a pattern may not use them.
BANNED_IN_PATTERN = {
    r"\d": "a digit class reads differently per engine, write [0-9]",
    r"\w": "a word class reads differently per engine, write the characters",
    r"\s": "a space class reads differently per engine, write the characters",
    "(?i": "an inline flag is not read by every engine",
    "(?<": "a lookbehind is not read by every engine",
    "(?=": "a lookahead is not read by every engine",
    "(?!": "a lookahead is not read by every engine",
    "\\1": "a backreference is not read by every engine",
}

# A scan that found nothing must not read as a pass. Three string types carry a pattern today
# (TitleKey, CalendarDay, WireDate) and two of them have fixtures; raise these when a fourth
# arrives, lower them only when a type really goes.
LEAST_PATTERNED = 3
LEAST_PROVED_BY_FIXTURES = 2
LEAST_FIXTURE_STRINGS = 10


def wire_version() -> str:
    text = (CONTRACT / "src" / "version.rs").read_text("utf-8")
    found = WIRE_VERSION.search(text)
    if not found:
        raise SystemExit("check_patterns: no WIRE_VERSION in contract/src/version.rs")
    return found.group(1)


def rust_const(name: str, where: str) -> int:
    """A `u32` constant as the Rust source declares it, evaluated as the arithmetic it is."""
    text = (CONTRACT / where).read_text("utf-8")
    found = re.search(RUST_CONST.format(name=name), text)
    if not found:
        raise SystemExit(f"check_patterns: no {name} in contract/{where}")
    return int(eval(found.group(1), {"__builtins__": {}}, {}))  # noqa: S307 - digits and shifts


def matches(pattern: str, text: str) -> bool:
    """Whether the whole text is the pattern, the way a client's engine asks it.

    The patterns are anchored, and `$` in Python also matches before a final newline, which Java's
    `matches` and Swift's full-range test do not. Asking for the whole string closes that gap.
    """
    return re.fullmatch(pattern.removeprefix("^").removesuffix("$"), text) is not None


def patterned_types() -> dict[str, str]:
    """Every string type whose standalone schema publishes a pattern, and that pattern."""
    found = {}
    for path in sorted((GENERATED / "schemas").glob("*.json")):
        schema = json.loads(path.read_text("utf-8"))
        if isinstance(schema, dict) and "pattern" in schema:
            found[path.stem] = schema["pattern"]
    openapi = json.loads((GENERATED / "openapi.json").read_text("utf-8"))
    for name, schema in openapi["components"]["schemas"].items():
        if isinstance(schema, dict) and "pattern" in schema:
            found.setdefault(name, schema["pattern"])
    return found


def one_copy(name: str, pattern: str) -> list[str]:
    """The same expression in every place a client reads it from."""
    found = []
    places = {
        "generated/openapi.json": pattern,
        # Java has no string interpolation, so `$` is left exactly as the schema writes it, which
        # is the opposite of Kotlin below. A backslash doubles, as it does in any Java literal.
        # No pattern carries a double quote today; one that did would need `"` escaped here too,
        # and the generated file already does it, so this line would be what went stale.
        "generated/types/Contract.java": pattern.replace("\\", "\\\\"),
        "generated/types/Contract.kt": pattern.replace("$", "\\$"),
        "generated/types/Contract.swift": pattern,
        # C++ has no interpolation either, so `$` stays exactly as the schema writes it and the
        # backslash doubles, which is the Java line again. The two differ nowhere today; they are
        # written out separately because the day a pattern carries a double quote they stop being
        # the same line, and a shared one would have to be split under pressure instead.
        "generated/types/Contract.hpp": pattern.replace("\\", "\\\\"),
    }
    standalone = GENERATED / "schemas" / f"{name}.json"
    if standalone.is_file():
        places["generated/schemas/%s.json" % name] = pattern
    for where, wanted in places.items():
        text = (GENERATED / Path(where).relative_to("generated")).read_text("utf-8")
        if wanted not in text:
            found.append(f"{where}: {name} does not carry the pattern the schema publishes")
    return found


def dialect(name: str, pattern: str) -> list[str]:
    return [
        f"{name}: the pattern uses {bad!r}: {why}"
        for bad, why in BANNED_IN_PATTERN.items()
        if bad in pattern
    ]


def corpus(version: str, types: dict[str, str]) -> tuple[list[str], dict[str, set[str]], int]:
    """Every fixture that is a plain string of a patterned type, judged by that type's pattern."""
    found: list[str] = []
    proved: dict[str, set[str]] = {}
    read = 0
    for kind in ("valid", "tolerated", "refused"):
        folder = CONTRACT / "fixtures" / version / kind
        for path in sorted(folder.glob("*.json")):
            name = path.stem.split(".")[0]
            if name not in types:
                continue
            value = json.loads(path.read_text("utf-8"))
            if not isinstance(value, str):
                continue
            read += 1
            proved.setdefault(name, set()).add("refused" if kind == "refused" else "accepted")
            hit = matches(types[name], value)
            if kind == "refused" and hit:
                found.append(
                    f"fixtures/{version}/{kind}/{path.name}: the {name} pattern takes {value!r}, "
                    "which the core refuses: the generated clients are more forgiving than the core"
                )
            if kind != "refused" and not hit:
                found.append(
                    f"fixtures/{version}/{kind}/{path.name}: the {name} pattern refuses {value!r}, "
                    "which the core reads: the generated clients are stricter than the core"
                )
    return found, proved, read


def bounds(types: dict[str, str]) -> list[str]:
    """The pattern stops exactly where the Rust constant stops."""
    found = []
    for type_name, name, where, shape in BOUNDS:
        if type_name not in types:
            found.append(f"{type_name}: no pattern, so the {name} bound is proved on nothing")
            continue
        pattern = types[type_name]
        most = rust_const(name, where)
        if not matches(pattern, shape.format(id=most)):
            found.append(
                f"{type_name}: the pattern refuses {shape.format(id=most)}, and "
                f"{name} in contract/{where} is {most}"
            )
        for over in (most + 1, int("9" * len(str(most + 1)))):
            if matches(pattern, shape.format(id=over)):
                found.append(
                    f"{type_name}: the pattern takes {shape.format(id=over)}, above "
                    f"{name} = {most} in contract/{where}: a client would build an id the core "
                    "cannot hold"
                )
    return found


def main() -> int:
    version = wire_version()
    types = patterned_types()
    found: list[str] = []
    for name, pattern in sorted(types.items()):
        found += dialect(name, pattern)
        found += one_copy(name, pattern)
    lines, proved, read = corpus(version, types)
    found += lines
    found += bounds(types)

    whole = sorted(name for name, kinds in proved.items() if len(kinds) == 2)
    bare = sorted(set(types) - set(proved))
    print(
        f"patterns: {len(types)} patterned types, {read} fixture strings judged, "
        f"proved both ways by fixtures: {', '.join(whole) or 'none'}"
        + (f"; NO fixture of their own: {', '.join(bare)}" if bare else "")
    )
    if len(types) < LEAST_PATTERNED:
        found.append(
            f"read {len(types)} patterned types, expected at least {LEAST_PATTERNED}: "
            "this check is not reading the generated schemas"
        )
    if len(whole) < LEAST_PROVED_BY_FIXTURES:
        found.append(
            f"{len(whole)} patterned types are proved by a fixture both ways, expected at least "
            f"{LEAST_PROVED_BY_FIXTURES}: a pattern with no refused fixture is not proved"
        )
    if read < LEAST_FIXTURE_STRINGS:
        found.append(
            f"judged {read} fixture strings, expected at least {LEAST_FIXTURE_STRINGS}: "
            "the fixtures this reads are not the fixtures the core reads"
        )
    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
