#!/usr/bin/env python3
"""The ArkTS of this family cannot be compiled here, so it is held to what text can prove.

No HarmonyOS SDK, no DevEco, no `node`, `npx`, `tsc`, `cmake`, `ohpm`, `hvigorw` or `codelinter`
are on this machine (note 247, `command -v` on each), so not one `.ets` file can be type checked,
linted or run. Nothing below pretends otherwise: **this proves no ArkTS is correct.** What it
proves is the half that is not a language question — that every number, word, colour and corner the
touch surface draws with came from somewhere outside this folder and still agrees with it.

That is the same shape the television's Kotlin is held in by
`apps/android/surfaces/tv/src/test/kotlin/streams/tv/NumbersTest.kt`, which reads its own source as
text and holds each number against the capture or the derivation it came from. Its rule is the rule
here: **a number is never asserted against itself.** Every constant in `surfaces/touch/Sizes.ets`
carries the file and line of the Android phone Kotlin that measured it, and this reads that Kotlin.

Why the Android phone Kotlin and not `reference/netflix-on-the-tv.md`: those two reference files
measure a television at 1920x1080 and a web page at nine viewport widths. Neither measures a phone
app. The touch surface's figures were measured on the Netflix Android app on a Pixel 8 on
2026-11-11, into the **private** reference folder outside this repository, and the in-repo record of
them is the phone surface's Kotlin, each line carrying its own citation. So the Kotlin is this
family's reference file, and `surfaces/touch/README.md` says the same.

Run by `apps/harmony/check.sh` beside the two C suites. Python because every check here is text
against text -- a `.ets` file against a `.kt` file, against `brand.json`, against `strings/en.json`
-- which is regular expression work, and because `platform/tools` belongs to whoever owns it: this
file is written in that folder's shape (a `main() -> int`, a summary line, no arguments) so moving
it there later is a `git mv` and a line in `./check`.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOUCH = ROOT / "apps/harmony/surfaces/touch"
HARMONY = ROOT / "apps/harmony"
PHONE = ROOT / "apps/android/surfaces/phone/src/main/kotlin/streams/phone"
SHARED = ROOT / "apps/android/surfaces/shared/src/main/kotlin/streams/shared"
TV = ROOT / "apps/android/surfaces/tv/src/main/kotlin/streams/tv"

held = 0
failed: list[str] = []


def ok(what: str, good: bool, why: str = "") -> bool:
    """One check. Prints as the C suites in this folder print, and counts the same way."""
    global held
    held += 1
    if good:
        print(f"ok   {what}")
    else:
        print(f"FAIL {what}" + (f": {why}" if why else ""))
        failed.append(what)
    return good


def same(what: str, wanted: object, got: object) -> bool:
    return ok(what, wanted == got, f"wanted {wanted!r}, got {got!r}")


# ---------------------------------------------------------------------------------------------
# Reading


def no_comments(text: str) -> str:
    """The same text with every comment blanked and every line number kept."""
    text = re.sub(r"/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S)
    return re.sub(r"//[^\n]*", "", text)


def touch_files() -> list[Path]:
    return sorted(TOUCH.glob("*.ets"))


def line_of(text: str, at: int) -> int:
    return text.count("\n", 0, at) + 1


# ---------------------------------------------------------------------------------------------
# 1. Every measured number, against the Kotlin that measured it.

# `browse/Rows.kt:84-85`, `Sizes.kt:26-30`, `title/Episodes.kt:98,103,109,111,138`, and the same
# written as a full path from the repository root.
CITATION = re.compile(r"`((?:[A-Za-z0-9_./-]+/)?[A-Za-z0-9_]+\.kt):([0-9]+(?:[-,][0-9]+)*)`")
A_NUMBER = re.compile(r"^export const ([A-Z][A-Z0-9_]*): number = ([^;]+);")
# A right hand side that is arithmetic rather than a measurement: other constants of this file, or
# plain numbers. Held below by its written form rather than by a citation of its own.
DERIVED = re.compile(r"^[A-Z0-9_]+(?:\s*[-+*/]\s*[A-Z0-9_]+)+$")


def kotlin_file(named: str) -> Path | None:
    """The Kotlin a citation names. A full path is taken as written; a short one is searched for."""
    if named.startswith("apps/"):
        found = ROOT / named
        return found if found.is_file() else None
    for tree in (PHONE, SHARED, TV):
        found = tree / named
        if found.is_file():
            return found
    # A citation written from the surface's own package down (`tv/search/Search.kt`), or by the
    # file's name alone. Taken only when exactly one file in one tree ends that way, so an
    # ambiguous name is a failure rather than a guess.
    tail = "/" + named
    matches = [
        found
        for tree in (PHONE, SHARED, TV)
        for found in tree.rglob(Path(named).name)
        if str(found).endswith(tail) and "/build/" not in str(found)
    ]
    return matches[0] if len(matches) == 1 else None


def cited_lines(path: Path, spans: str) -> str | None:
    """The text of the lines a citation names, or None when a span runs off the end of the file."""
    lines = path.read_text(encoding="utf-8").split("\n")
    out: list[str] = []
    for span in spans.split(","):
        first, _, last = span.partition("-")
        start, stop = int(first), int(last or first)
        if start < 1 or stop > len(lines) or stop < start:
            return None
        out += lines[start - 1 : stop]
    return "\n".join(out)


def unwrapped(text: str) -> str:
    """The same text with every doc comment's hand wrapping undone, so a sentence reads as one."""
    text = re.sub(r"\n\s*\*\s?", " ", text)
    return re.sub(r"\n\s*//\s?", " ", text)


def citations_of(comment: str) -> list[tuple[str, str]]:
    """Every citation in a doc comment, with the comment's own wrapping undone.

    A path too long for one line is wrapped by hand across ` * ` continuations, so the leaders come
    off and the lines are joined with nothing between them before the paths are read out.
    """
    joined = "".join(re.sub(r"^\s*\*\s?", "", line) for line in comment.split("\n"))
    return CITATION.findall(joined)


def blocks_of_sizes() -> list[tuple[str, list[tuple[str, str, int]]]]:
    """Every run of constants in `Sizes.ets` with the comment that stands over it."""
    text = (TOUCH / "Sizes.ets").read_text(encoding="utf-8")
    out: list[tuple[str, list[tuple[str, str, int]]]] = []
    comment: list[str] = []
    run: list[tuple[str, str, int]] = []
    for at, line in enumerate(text.split("\n"), start=1):
        bare = line.strip()
        if bare.startswith(("/*", "*", "//")):
            if run:
                out.append(("\n".join(comment), run))
                run = []
                comment = []
            comment.append(line)
            continue
        found = A_NUMBER.match(line)
        if found:
            run.append((found.group(1), found.group(2).strip(), at))
        elif bare and not found:
            if run:
                out.append(("\n".join(comment), run))
                run = []
            comment = []
    if run:
        out.append(("\n".join(comment), run))
    return out


def written(value: str) -> re.Pattern[str]:
    """The value as the Kotlin writes it: `119.dp`, `0.712f`, `3850L`, and never inside 1119."""
    return re.compile(r"(?<![0-9.])" + re.escape(value) + r"(?![0-9])")


# The numbers in `Sizes.ets` that are this surface's own and not a port of a measurement, each one
# named in that file with what it replaces and why a citation would be false. The set is exact: a
# fourth cannot join it without a line here, and giving one of these a citation fails this too.
NOT_MEASURED = {"CLOSE_GLYPH_TYPE", "TITLE_CLOSE_GLYPH_TYPE", "SEASON_CLOSE_GLYPH_TYPE"}


def suite_numbers() -> None:
    """Every number against the Kotlin that measured it, and what that does and does not catch.

    CAUGHT: a figure invented in this folder, a figure edited on one side of the port and not the
    other, a citation naming a Kotlin file that does not exist, and a citation whose line runs past
    the end of the file it names.

    NOT CAUGHT, and measured rather than guessed: the citation's LINE NUMBER. The number has to be
    in the cited file and the cited lines have to exist; the number does not have to be on them. On
    2026-11-13, 48 of the 171 figures here sat somewhere other than their cited line, drifting by
    one line in 24 cases and by as much as 67 in `title/Title.kt`, because `apps/android` belongs
    to another family and is edited daily. Holding the line exactly would have this suite fail on
    every unrelated edit over there, which is churn and not a check, and the first thing anyone
    would do is widen it. So it is stated here instead: a line number in a citation is a reader's
    convenience and the file is the part this holds.
    """
    print("== every measured number against the Android phone Kotlin that measured it")
    blocks = blocks_of_sizes()
    counted = sum(len(run) for _, run in blocks)
    ok(
        f"Sizes.ets declares {counted} measured numbers",
        counted > 150,
        f"read {counted}, which is too few to be the whole file",
    )
    names = {name for _, run in blocks for name, _, _ in run}
    for comment, run in blocks:
        cites = citations_of(comment)
        read: dict[tuple[str, str], str] = {}
        broken = False
        for named, spans in cites:
            path = kotlin_file(named)
            if path is None:
                for name, _, _ in run:
                    ok(f"{name} cites {named}, which is a file", False, "no such Kotlin file")
                broken = True
                break
            if cited_lines(path, spans) is None:
                for name, _, _ in run:
                    ok(
                        f"{name} cites {named}:{spans}, which is inside that file",
                        False,
                        "the span runs off the end",
                    )
                broken = True
                break
            read[(named, spans)] = path.read_text(encoding="utf-8")
        whole = "\n".join(read.values())
        for name, value, _ in run:
            if DERIVED.match(value):
                wanted = {w for w in re.findall(r"[A-Z][A-Z0-9_]*", value)}
                ok(
                    f"{name} is written as {value}, and every name in it is one of this file's",
                    wanted <= names,
                    f"{sorted(wanted - names)} are not declared here",
                )
                continue
            if name in NOT_MEASURED:
                continue
            if not cites:
                ok(f"{name} carries the Kotlin that measured it", False, "no citation over it")
                continue
            if broken:
                continue
            where = ", ".join(f"{f}:{s}" for f, s in read)
            ok(
                f"{name} = {value} is in {where}",
                bool(written(value).search(whole)),
                "that Kotlin does not hold that number",
            )

    # The derivations the file states in prose, held to the arithmetic they claim. A number that is
    # a sum of two others is the one kind that can drift silently: both halves still cite a real
    # line and the total stops being their total.
    text = (TOUCH / "Sizes.ets").read_text(encoding="utf-8")
    ok(
        "the tile pitch stays written as the tile plus the gap beside it",
        "export const TILE_PITCH: number = TILE_WIDE + TILE_GAP;" in text,
        "a pitch written as its answer loses the reason a resized tile has to move it",
    )
    ok(
        "a More Like This tile keeps the row tile's own ratio",
        "export const TILE_RATIO: number = TILE_WIDE / TILE_HIGH;" in text,
    )
    ok(
        "the title page's artwork stays written as 16 / 9",
        "export const TITLE_ART_RATIO: number = 16 / 9;" in text,
    )
    # The three that are this surface's own stay three, and stay named in the file as well as here.
    declared = {name for _, run in blocks for name, _, _ in run}
    same("the numbers this surface chose rather than measured", NOT_MEASURED, NOT_MEASURED & declared)
    ok(
        "Sizes.ets still says in its own words which numbers it chose",
        "NOT MEASURED." in text,
        "the section naming them is gone, so a reader has only this file's word for it",
    )
    # `browse/Billboard.kt` derives the billboard's share of the screen in prose and the number is
    # the result: 536.2 / 936.6 = 0.572. Held to the arithmetic, not to the comment.
    card = unwrapped((PHONE / "browse/Billboard.kt").read_text(encoding="utf-8"))
    sides = re.search(r"card is ([0-9.]+) x ([0-9.]+) dp on a ([0-9.]+) dp screen", card)
    if ok("Billboard.kt still states where the billboard's share came from", sides is not None):
        high, screen = float(sides.group(2)), float(sides.group(3))
        share = float(re.search(r"CARD_SHARE: number = ([0-9.]+)", text).group(1))
        ok(
            f"the billboard's {share} of the screen is {high} over {screen}",
            abs(high / screen - share) < 0.001,
            f"{high} / {screen} = {high / screen:.4f}",
        )
    # The one figure the README states as a ratio rather than a size.
    wide = float(re.search(r"TILE_WIDE: number = ([0-9.]+)", text).group(1))
    tall = float(re.search(r"TILE_HIGH: number = ([0-9.]+)", text).group(1))
    readme = (TOUCH / "README.md").read_text(encoding="utf-8")
    claimed = re.search(r"\| Tile \| ([0-9.]+) x ([0-9.]+) dp, ratio 1 : ([0-9.]+) \|", readme)
    if ok("the touch README still states the tile it draws", claimed is not None):
        same("the README's tile width is the one Sizes.ets draws", wide, float(claimed.group(1)))
        same("the README's tile height is the one Sizes.ets draws", tall, float(claimed.group(2)))
        ok(
            f"the README's 1 : {claimed.group(3)} is {tall} over {wide}",
            abs(tall / wide - float(claimed.group(3))) < 0.001,
            f"{tall} / {wide} = {tall / wide:.4f}",
        )
    # Tiles across a row: the README claims 3.29 visible. That is the reference phone's width,
    # less the row's start, over the pitch -- and the width is read out of `browse/Empty.kt`, which
    # states it ("On the reference's 421.5 dp phone that is four -- three full and a cut fourth,
    # which is the measured 3.29 visible"), so no second copy of it lives here.
    across = re.search(r"\| Tiles across a row \| ([0-9.]+) visible \|", readme)
    empty = unwrapped((PHONE / "browse/Empty.kt").read_text(encoding="utf-8"))
    phone = re.search(r"reference's ([0-9.]+) dp phone", empty)
    if ok("Empty.kt still states the phone the reference was measured on", phone is not None) and ok(
        "the touch README still states how many tiles a row shows", across is not None
    ):
        gap = float(re.search(r"TILE_GAP: number = ([0-9.]+)", text).group(1))
        start = float(re.search(r"ROW_START: number = ([0-9.]+)", text).group(1))
        pitch = wide + gap
        visible = (float(phone.group(1)) - start) / pitch
        ok(
            f"{across.group(1)} tiles is a {phone.group(1)} dp phone less the row's start over a {pitch} pitch",
            abs(visible - float(across.group(1))) < 0.02,
            f"({phone.group(1)} - {start}) / {pitch} = {visible:.4f}",
        )


# ---------------------------------------------------------------------------------------------
# 2. Every colour and corner, against `brand.json`.

A_COLOUR = re.compile(r"^export const ([A-Z][A-Z0-9_]*): string = '(#[0-9a-f]{6})';", re.M)
A_CORNER = re.compile(r"^export const (CORNER_[A-Z_]*): number = ([0-9]+);", re.M)
A_HEX = re.compile(r"'#[0-9a-fA-F]{3,8}'")

# `Theme.ets`'s name for each `brand.json` value. The names differ on purpose -- the brand file's
# are roles and this surface's are what the thing is called where it is drawn -- so the pairing is
# written once, here, and a value that stops matching fails.
BRAND_COLOURS = {
    "BACK": "background",
    "PLATE": "surface",
    "RAISED": "raised",
    "INK": "text",
    "MUTED": "muted",
    "ACCENT": "accent",
    "JOIN": "join",
}
BRAND_CORNERS = {
    "CORNER_BUTTON": "button_radius",
    "CORNER_CARD": "card_radius",
    "CORNER_CHIP": "chip_radius",
    "CORNER_CHIP_END": "chip_end_radius",
    "CORNER_FIELD": "field_radius",
    "CORNER_PANEL": "panel_radius",
}


def suite_brand() -> None:
    print()
    print("== every colour and corner against brand.json, and nowhere else")
    brand = json.loads((ROOT / "brand.json").read_text(encoding="utf-8"))
    theme = (TOUCH / "Theme.ets").read_text(encoding="utf-8")
    colours = dict(A_COLOUR.findall(theme))
    corners = dict(A_CORNER.findall(theme))
    for name, role in BRAND_COLOURS.items():
        same(f"{name} is brand.json colors.{role}", brand["colors"][role], colours.get(name))
    for name, role in BRAND_CORNERS.items():
        same(f"{name} is brand.json shape.{role}", str(brand["shape"][role]), corners.get(name))
    ok(
        f"Theme.ets names all {len(BRAND_CORNERS)} of the brand's corners and no seventh",
        set(corners) == set(BRAND_CORNERS),
        f"found {sorted(corners)}",
    )

    # One home for each value. A brand colour spelled again anywhere else in this family is the
    # drift `one_name.rs` catches for the brand's name and nothing catches for its look.
    values = set(brand["colors"].values())
    for path in touch_files() + [HARMONY / "core/Host.ets", HARMONY / "ports/Ports.ets"]:
        if path.name == "Theme.ets":
            continue
        text = no_comments(path.read_text(encoding="utf-8"))
        spelled = [found for found in A_HEX.findall(text) if found.strip("'").lower() in values]
        ok(
            f"{path.name} reaches the brand's colours through Theme.ets",
            not spelled,
            f"it spells {sorted(set(spelled))}",
        )
    # And no file but `Theme.ets` carries a colour at all: a grey written at a call site is the
    # same drift one step further from the brand file.
    for path in touch_files():
        if path.name == "Theme.ets":
            continue
        text = no_comments(path.read_text(encoding="utf-8"))
        ok(
            f"{path.name} writes no colour of its own",
            not A_HEX.search(text),
            f"it writes {sorted(set(A_HEX.findall(text)))}",
        )
    # Every grey `Theme.ets` carries that is NOT a brand value is one the reference was measured
    # at, so each one answers to the Kotlin that named it, exactly as a size does.
    for comment, run in blocks_of_theme():
        cites = citations_of(comment)
        for name, value in run:
            if name in BRAND_COLOURS or value in values:
                continue
            if name == "BLACK":
                ok("BLACK is black and cites nothing, because it is not a measurement", value == "#000000")
                continue
            if not cites:
                ok(f"{name} carries the Kotlin that measured it", False, "no citation over it")
                continue
            whole = ""
            good = True
            for named, spans in cites:
                path = kotlin_file(named)
                if path is None:
                    good = ok(f"{name} cites {named}, which is a file", False)
                    break
                if cited_lines(path, spans) is None:
                    good = ok(f"{name} cites {named}:{spans}, which is inside that file", False)
                    break
                whole += path.read_text(encoding="utf-8") + "\n"
            if not good:
                continue
            where = ", ".join(f"{f}:{s}" for f, s in cites)
            # The Kotlin writes a colour as `Color(0xFFd4d4d1)`, so the six digits are what both
            # sides have in common. The line is not held, for the reason `suite_numbers` gives.
            ok(
                f"{name} = {value} is in {where}",
                re.search(re.escape(value.lstrip("#")), whole, re.I) is not None,
                "that Kotlin does not hold that colour",
            )


A_THEME_COLOUR = re.compile(r"^export const ([A-Z][A-Z0-9_]*): string = '(#[0-9a-f]{6})';")


def blocks_of_theme() -> list[tuple[str, list[tuple[str, str]]]]:
    text = (TOUCH / "Theme.ets").read_text(encoding="utf-8")
    out: list[tuple[str, list[tuple[str, str]]]] = []
    comment: list[str] = []
    run: list[tuple[str, str]] = []
    for line in text.split("\n"):
        bare = line.strip()
        if bare.startswith(("/*", "*", "//")):
            if run:
                out.append(("\n".join(comment), run))
                run = []
                comment = []
            comment.append(line)
            continue
        found = A_THEME_COLOUR.match(line)
        if found:
            run.append((found.group(1), found.group(2)))
        elif bare:
            if run:
                out.append(("\n".join(comment), run))
                run = []
            comment = []
    if run:
        out.append(("\n".join(comment), run))
    return out


# ---------------------------------------------------------------------------------------------
# 3. Every word, against `strings/en.json`.

A_KEY = re.compile(r"""\b(?:word|counted)\(\s*(?:this\.)?seen\s*,\s*(.+?)\)""")
A_LITERAL_KEY = re.compile(r"'([a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)+)'")
A_FILL = re.compile(r"fill\(\s*word\(\s*(?:this\.)?seen\s*,\s*'([a-z0-9_.]+)'\s*\)\s*,\s*'([a-z_]+)'")

# Keys this surface asks for that no language has yet, each one a gap with an owner. The set is
# exact, as `apps/android/surfaces/phone/src/test/kotlin/streams/phone/WordsTest.kt`'s is: a new
# one fails this, and filling one of these fails it too, so the list cannot outlive the gap.
#
# It holds exactly what the phone's holds, and for the same reason. `app.title` is the header's
# fallback when the menu has no label for the page; the Apple surface falls through it to the brand
# and, since 2026-11-13, so do the Android phone (`browse/Home.kt:106`) and this one, so the key is
# named by all three and wanted by none of them. Whether it should exist at all is the owner's, and
# `WordsTest.kt` says so at length.
#
# Until 2026-11-13 this surface ended on `word(seen, 'app.title')` with no brand behind it and drew
# an EMPTY header for any page the menu had no label for -- the exact defect commit edfaf28 had
# already fixed on the Kotlin side the day before this folder was written from it.
GAPS: set[str] = {"app.title"}

# The call sites that take a key from an expression rather than a literal, as `<file> <expression>`
# so the set survives a line moving. Each one's literals are spelled in the same file and are swept
# up by the second reading below; this list exists so a sixth cannot appear unnoticed.
CHOSEN_ABOVE = {
    "Billboard.ets isSeries(this.tile",
    "Gate.ets key",
    "Notifications.ets kindKey(item.kind",
    "Title.ets key",
}


def suite_words() -> None:
    print()
    print("== every word against strings/en.json")
    book = json.loads((ROOT / "strings/en.json").read_text(encoding="utf-8"))
    asked: dict[str, str] = {}
    unread: list[str] = []
    for path in touch_files():
        text = no_comments(path.read_text(encoding="utf-8"))
        for found in A_KEY.finditer(text):
            inside = found.group(1)
            keys = A_LITERAL_KEY.findall(inside)
            if not keys:
                unread.append(f"{path.name} {inside.strip()}")
                continue
            for key in keys:
                asked[key] = f"{path.name}:{line_of(text, found.start())}"
    ok(f"the touch surface asks for {len(asked)} words", len(asked) > 25, f"read {len(asked)}")
    same(
        "every word asked for through `word` is in strings/en.json",
        set(),
        {key for key in asked if key not in book and f"{key}.one" not in book},
    )
    # The keys chosen above the call site rather than written in it -- a `kindKey(...)` or a
    # ternary too wide to read out of the call. Each one's literals are spelled somewhere in the
    # same file, so the sweep below catches them; this names them so neither list is a silence.
    same("the call sites that choose their key above themselves", CHOSEN_ABOVE, set(unread))
    # Every key-shaped literal in the folder, wherever it is written. This is what holds the five
    # `notification.*` keys that `Notifications.ets` picks in `kindKey` and never writes inside a
    # `word(...)` call.
    loose: dict[str, str] = {}
    for path in touch_files():
        text = no_comments(path.read_text(encoding="utf-8"))
        for found in A_LITERAL_KEY.finditer(text):
            key = found.group(1)
            if key.startswith("app.media."):
                continue
            loose.setdefault(key, f"{path.name}:{line_of(text, found.start())}")
    for key, where in sorted(loose.items()):
        if key in GAPS:
            continue
        ok(f"{key} is in strings/en.json ({where})", key in book or f"{key}.one" in book)
    same("every word asked for by name is one of those", set(), set(asked) - set(loose))
    missing_loose = {key for key in loose if key not in book and f"{key}.one" not in book}
    same("the words this folder names that no language has", GAPS, missing_loose)
    # Every place a word is filled is a place that word has.
    for path in touch_files():
        text = no_comments(path.read_text(encoding="utf-8"))
        for key, place in A_FILL.findall(text):
            value = book.get(key)
            if value is None:
                continue
            ok(
                f"{key} has a {{{place}}} for {path.name} to fill",
                "{" + place + "}" in value,
                f"en.json says {value!r}",
            )


# A literal that would be a word on a screen: a capitalised word, or two words with a space between
# them, with the punctuation a sentence carries. `'9+'`, `'100%'`, `'home'` (a route) and
# `'#0b0b10'` are not. The trailing stop matters: a first pass left it out and
# `Text('No notifications yet.')` walked straight through, which is the exact shape of the Kotlin
# fallbacks this surface refuses to carry.
AN_ENGLISH_LITERAL = re.compile(
    r"'([A-Z][a-z]+(?:[ '’][A-Za-z]+)*[.!?,]?|[a-z]+(?: [a-z]+)+[.!?,]?)'"
)
# Words that are addresses, keys, attribute values and type names rather than anything a viewer
# reads. Every one of them is a value ArkUI or the contract defines.
NOT_A_WORD = {
    "px", "vp", "fp", "lpx", "auto", "none", "top", "bottom", "left", "right", "center",
    "start", "end", "home", "kids", "search", "genre", "title", "screen", "profiles",
    "notifications", "latn", "arab", "arabext", "en", "reply", "no_reply", "poster",
    "backdrop", "logo", "still", "hero", "row", "rank", "name", "date", "percent", "n", "m",
    "h", "movie", "series", "film", "GET", "POST", "PUT", "DELETE", "kids_home", "Rtl", "Ltr",
}


def suite_no_english() -> None:
    print()
    print("== no English on a screen, because every word comes from the server")
    for path in touch_files():
        text = no_comments(path.read_text(encoding="utf-8"))
        found = [
            m.group(1)
            for m in AN_ENGLISH_LITERAL.finditer(text)
            if m.group(1) not in NOT_A_WORD
        ]
        ok(
            f"{path.name} spells no English of its own",
            not found,
            f"it spells {sorted(set(found))}",
        )


# ---------------------------------------------------------------------------------------------
# 4. The brand's name, and the ring.


def suite_brand_name() -> None:
    print()
    print("== the brand's name is in brand.json and in no program file of this family")
    brand = json.loads((ROOT / "brand.json").read_text(encoding="utf-8"))
    name = brand["name"]
    # `launcher/tests/one_name.rs` walks `git ls-files`, so a file written and not yet committed is
    # invisible to it. This walks the folder, which is what catches a new file the day it is
    # written rather than the day it is added.
    for path in sorted(HARMONY.rglob("*")):
        if not path.is_file() or path.suffix not in (".ets", ".ts", ".c", ".h", ".py", ".json5"):
            continue
        ok(
            f"{path.relative_to(HARMONY)} does not state the name {name}",
            name not in path.read_text(encoding="utf-8", errors="ignore"),
        )


# What reaches a device directly. `surfaces/touch/README.md` holds this list and says why each one
# is a call and not an import: `@kit.ImageKit` is imported for the TYPE `image.PixelMap`, which
# ArkTS needs because it has no structural typing, so an import is not a ring crossing and a call
# is. `platform/tools/check_rings.py` has no `harmony` family, so until it does this is the only
# thing holding this folder to its ports.
BANNED = {
    "the network port": r"\bhttp\.createHttp\b|\bcreateHttp\(|@ohos\.net\.|@kit\.NetworkKit",
    "the secrets port": r"\basset\.(add|query|remove|update)\b|@kit\.AssetStoreKit",
    "the store port": r"\bfileIo\.|@kit\.CoreFileKit|@ohos\.file\.fs",
    "the player port": r"\bAVPlayer\b|createAVPlayer|createMediaSourceWithUrl",
    "the pictures port": r"createImageSource|createPixelMap",
}


def suite_rings() -> None:
    print()
    print("== a surface reaches the world only through a port")
    for path in touch_files():
        text = no_comments(path.read_text(encoding="utf-8"))
        for port, pattern in BANNED.items():
            found = re.search(pattern, text)
            ok(
                f"{path.name} does not reach past {port}",
                found is None,
                f"line {line_of(text, found.start())} has {found.group(0)!r}" if found else "",
            )
    # The one call this folder makes that argues with that list, named so it is never silent.
    pictures = no_comments((TOUCH / "Pictures.ets").read_text(encoding="utf-8"))
    ok(
        "the colour picker in Pictures.ets is still the one argued exception",
        pictures.count("effectKit.createColorPicker") == 1,
        "it reads a colour out of a PixelMap the port already handed over, which is what "
        "androidx.palette does inside browse/Billboard.kt:134; if it is to be banned the colour "
        "moves into the Pictures port for both families at once",
    )
    # Addresses are the core's, not a screen's: a path spelled in a surface is a second routing
    # table. `Pick.ets` is where this family keeps the one it has.
    table = re.compile(r"""'/(?:home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)(?:[/'?])""")
    for path in touch_files():
        if path.name == "Pick.ets":
            continue
        text = no_comments(path.read_text(encoding="utf-8"))
        found = table.search(text)
        ok(
            f"{path.name} spells no address of its own",
            found is None,
            f"line {line_of(text, found.start())} has {found.group(0)!r}" if found else "",
        )


# ---------------------------------------------------------------------------------------------
# 5. The icons, against the Android phone surface that names them.

AN_ICON = re.compile(r"^export const (ICON_[A-Z0-9_]*): Resource = \$r\('app\.media\.(icon_[a-z0-9_]+)'\);", re.M)


# Icons this family names and no screen here draws yet, each with the screen that would. The set
# is exact: an icon that stops being drawn fails this, and writing the screen that draws one fails
# it too, which is how the list cannot outlive the gap.
NOT_YET_DRAWN = {
    # `profiles/EditProfile.kt:453` draws the open padlock beside the locked one. Editing a
    # profile is 1,194 lines of Kotlin and has no screen in this folder.
    "ICON_LOCK_OPEN",
    # `title/Title.kt:587` draws a Share control whose word is typed into the surface in English
    # and whose click body is empty in both families. `Title.ets` leaves it out rather than
    # drawing a control that does nothing; the icon stays named for when there is a port for it.
    "ICON_SHARE",
}


def suite_icons() -> None:
    print()
    print("== the icons are the Android phone surface's, name for name")
    text = (TOUCH / "Icons.ets").read_text(encoding="utf-8")
    named = dict(AN_ICON.findall(text))
    drawn = set()
    for tree in (PHONE, SHARED):
        for path in sorted((tree).rglob("*.kt")):
            if "/build/" in str(path):
                continue
            drawn |= set(re.findall(r"R\.drawable\.(icon_[a-z0-9_]+)", path.read_text(encoding="utf-8")))
    ok(f"Icons.ets names {len(named)} icons", len(named) > 20)
    same("every icon the Android phone surface draws is named here", set(), drawn - set(named.values()))
    same("and this family invents none of its own", set(), set(named.values()) - drawn)
    for const, name in sorted(named.items()):
        ok(f"{const} is {name}, spelled the ArkTS way", const == "ICON_" + name[len("icon_"):].upper())
    # Every icon named is an icon drawn, so the file does not grow a tail nothing uses.
    used = "".join(no_comments(p.read_text(encoding="utf-8")) for p in touch_files() if p.name != "Icons.ets")
    waiting = {const for const in named if not re.search(r"\b" + const + r"\b", used)}
    same("the icons named here that no screen in this folder draws yet", NOT_YET_DRAWN, waiting)


# ---------------------------------------------------------------------------------------------
# 6. Every number a screen draws with comes from Sizes.ets or Theme.ets.

# The attributes and the object keys that carry a length. A number written at one of these is a
# measurement, and `surfaces/touch/README.md` says a measurement this surface may not hold is a
# number it may not draw with.
A_LENGTH = re.compile(
    r"\.(?:width|height|fontSize|borderRadius|lineHeight|size|strokeWidth|constraintSize)\(\s*(-?[0-9]+(?:\.[0-9]+)?)\s*[,)]"
    r"|\b(?:padding|margin|offset|border|space)\s*[:(]\s*\{[^{}]*?\b(?:top|bottom|left|right|start|end|x|y|width|radius)\s*:\s*(-?[0-9]+(?:\.[0-9]+)?)"
    r"|\bspace\s*:\s*(-?[0-9]+(?:\.[0-9]+)?)"
)

# Lengths that are not measurements, each with why. The set is exact: a new one fails this check,
# and taking one away without naming its number fails it too.
NOT_A_MEASUREMENT = {
    # A hairline. One device unit is the thinnest line ArkUI draws and is not a figure off a
    # capture; the Kotlin writes `1.dp` at the same three places.
    ("Billboard.ets", "border", "1"),
    ("BottomBar.ets", "border", "1"),
    ("Header.ets", "border", "1"),
    ("Title.ets", "height", "1"),
    # Mirroring: `scale({ x: -1 })` turns a glyph for a right to left language. A direction, not a
    # length, and the only two values it can take.
    ("Header.ets", "scale", "-1"),
    ("Header.ets", "scale", "1"),
    ("Rows.ets", "scale", "-1"),
    ("Rows.ets", "scale", "1"),
    ("Search.ets", "scale", "-1"),
    ("Search.ets", "scale", "1"),
    ("Title.ets", "scale", "-1"),
    ("Title.ets", "scale", "1"),
}


def suite_lengths() -> None:
    print()
    print("== every length a screen draws with is a named constant")
    for path in touch_files():
        if path.name in ("Sizes.ets", "Theme.ets"):
            continue
        text = no_comments(path.read_text(encoding="utf-8"))
        loose: list[str] = []
        for found in A_LENGTH.finditer(text):
            value = next(g for g in found.groups() if g is not None)
            loose.append(f"line {line_of(text, found.start())}: {found.group(0).strip()[:48]} = {value}")
        ok(
            f"{path.name} draws with named lengths alone",
            not loose,
            "; ".join(loose),
        )


AN_EXPORT = re.compile(r"^export (?:const|function|class|enum|interface|type|struct) ([A-Za-z_][A-Za-z0-9_]*)", re.M)
AN_IMPORT = re.compile(r"import\s*\{([^{}]*)\}\s*from\s*'(\.[^']*)';")


def suite_imports() -> None:
    """The one thing here that stands in for a compiler, and the only one that can.

    ArkTS has no implicit anything: a name a file uses from another file in this folder has to be
    imported by name, and an import that nothing uses is an error under `noUnusedLocals`, which a
    DevEco project turns on. Neither can be checked here by a tool, so both are checked as text.

    TWO OF THE THREE DIRECTIONS, and the third is stated rather than faked. An import that nothing
    in the body names is dead, and a name imported from a sibling that the sibling does not export
    will not resolve: both are sound to read off the text. The direction that is NOT held is a name
    USED and not imported, because telling that apart from a local of the same name wants a parser
    and not a regular expression -- `wide` is a parameter in six files and an export of `Pick.ets`,
    `number` is an export of `Words.ets` and the type of every length here. A first pass that tried
    it reported thirteen such collisions and not one real defect. That is what the SDK's own
    compiler is for, and it is on the list of what this cannot do.

    The same collision WEAKENS the dead-import half, and it is measured: an unused `wide` added to
    `Notifications.ets`'s import of `Pick.ets` survived the mutation run of 2026-11-13, because
    `wide:` is also a field of the `Art` component's arguments in that file. An unused `logo` was
    caught. So this half catches a dead import whose name is not also an argument name or a local
    somewhere in the same file, and nothing more than that.
    """
    print()
    print("== every import a screen takes is exported by the file it names, and is used")
    exported = {path.stem: set(AN_EXPORT.findall(path.read_text(encoding="utf-8"))) for path in touch_files()}
    for path in touch_files():
        text = path.read_text(encoding="utf-8")
        body = no_comments(AN_IMPORT.sub("", re.sub(r"^import[^\n]*;$", "", text, flags=re.M)))
        spoken = set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", body))
        for found in AN_IMPORT.finditer(text):
            module = Path(found.group(2)).stem
            if module not in exported:
                continue
            taken = {name.strip() for name in found.group(1).replace("\n", " ").split(",") if name.strip()}
            same(f"{module}.ets exports everything {path.name} takes from it", set(), taken - exported[module])
            same(f"and {path.name} uses all of it", set(), taken - spoken)


# The brand's shape values, which belong in `Theme.ets` and nowhere else. `CORNER_BUTTON` is 999,
# which reads as a plain number at a call site and is the one most likely to be spelled again --
# `Tiles.ets:266` had `round(999, CONTINUE_BUTTON)` until 2026-11-13.
def suite_corners() -> None:
    print()
    print("== a brand corner is never spelled again at a call site")
    brand = json.loads((ROOT / "brand.json").read_text(encoding="utf-8"))
    corners = {str(value) for value in brand["shape"].values()}
    for path in touch_files():
        if path.name in ("Sizes.ets", "Theme.ets"):
            continue
        text = no_comments(path.read_text(encoding="utf-8"))
        spelled = [
            f"line {line_of(text, m.start())}: {m.group(0)}"
            for m in re.finditer(r"\b(?:round|borderRadius)\(\s*(-?[0-9]+)", text)
            if m.group(1) in corners
        ]
        ok(f"{path.name} corners through Theme.ets", not spelled, "; ".join(spelled))


def suite_claims() -> None:
    print()
    print("== no file in this family claims a count nothing measures")
    # `platform/tools/check_claims.py` holds four public files to this and names neither of this
    # family's READMEs. Until one of them joins that list, this holds them to the same rule, with
    # that file's own expression so there is one statement of what a claim is.
    source = (ROOT / "platform/tools/check_claims.py").read_text(encoding="utf-8")
    pattern = re.search(r"A_COUNT = re\.compile\(\n\s*(r\"[^\"]+\")", source)
    if not ok("check_claims.py still states what a size claim is", pattern is not None):
        return
    a_count = re.compile(eval(pattern.group(1)), re.IGNORECASE)  # noqa: S307 - a literal from that file
    in_code = re.compile(r"^\s{4,}|^```")
    for path in sorted(HARMONY.rglob("*.md")):
        text = path.read_text(encoding="utf-8")
        inside = False
        found = []
        for at, line in enumerate(text.split("\n"), start=1):
            if line.startswith("```"):
                inside = not inside
                continue
            if inside or in_code.match(line):
                continue
            hit = a_count.search(line)
            if hit:
                found.append(f"{at}: {hit.group(0)}")
        ok(
            f"{path.relative_to(HARMONY)} names a command rather than a count",
            not found,
            "; ".join(found),
        )


def main() -> int:
    suite_numbers()
    suite_brand()
    suite_words()
    suite_no_english()
    suite_brand_name()
    suite_rings()
    suite_icons()
    suite_lengths()
    suite_corners()
    suite_imports()
    suite_claims()
    print()
    print(f"{held - len(failed)} of {held} held")
    if failed:
        print(f"{len(failed)} failed: {failed[:8]}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
