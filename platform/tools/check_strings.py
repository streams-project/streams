#!/usr/bin/env python3
"""Every language file has English's keys, and the same holes to fill in each of them.

What this reads, and it is the whole of it:

  keys          every key of `strings/en.json` is in every other `strings/<lang>.json`
  placeholders  each translated line carries English's `{placeholders}`, with two exemptions:
                `{brand}` may be dropped, because a language names a thing its own way, and `{n}`
                may be dropped from a `.one` or `.two` plural, which some languages spell out
  rows          `strings/rows/<lang>.json` translates every row name in `strings/rows/names.json`
                -- the English names the page builder makes, written by the row_names test in
                server-pages -- with no blank left standing in for one

What it does NOT read, written down because two separate agents on 2026-11-13 each believed one of
these was already here and built on it:

  * **No source file is opened.** Not a `.kt`, `.swift`, `.rs`, `.ets`, `.ts` or `.java`. An
    English sentence typed straight into a screen is invisible here: this compares one JSON file
    against another and nothing else. The families that have such a guard each wrote their own --
    `touch_check.py` holds the HarmonyOS surface to it, `check_roku.py` the Roku one -- and the
    families that have not got one are not covered by anything.
  * **No key a screen asks for is checked to exist.** A screen naming `title.episodes.missing`
    passes here whether or not `en.json` has it; nothing walks from a call site to a key. The
    reverse is true too: a key in `en.json` that no screen asks for is never reported.
  * **Nothing is said about what a translation MEANS.** A value copied from English, or wrong, is
    a present key with the right placeholders, and that is all this asks of it.

The first of the three is the one worth closing and it is not closed here: it wants a per-family
read of source text, which is where each family already keeps one.
"""

from __future__ import annotations

import json
from pathlib import Path
import re
import sys

PLACEHOLDER_RE = re.compile(r"\{[a-z_]+\}")


def placeholders(text: str) -> set[str]:
    return set(PLACEHOLDER_RE.findall(text))


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    strings_dir = root / "strings"
    en_path = strings_dir / "en.json"
    if not en_path.is_file():
        print(f"missing {en_path}", file=sys.stderr)
        return 1

    en_data: dict[str, str] = json.loads(en_path.read_text(encoding="utf-8"))
    en_keys = list(en_data.keys())
    failed = False

    for path in sorted(strings_dir.glob("*.json")):
        if path.name == "en.json":
            continue
        lang = path.stem
        data: dict[str, str] = json.loads(path.read_text(encoding="utf-8"))

        for key in en_keys:
            if key not in data:
                print(f"missing {lang} {key}")
                failed = True
                continue
            # {brand} may be left out: a language names a thing its own way, and "My {brand}" is not
            # always "my" plus the name. Every other place must match English.
            expected_ph = placeholders(en_data[key])
            actual_ph = placeholders(data[key])
            # The one and two forms of a plural may spell the number out ("موسم واحد", "موسمان").
            optional = {"{brand}"} | ({"{n}"} if key.endswith((".one", ".two")) else set())
            if actual_ph - expected_ph or (expected_ph - optional) - actual_ph:
                print(
                    f"placeholder {lang} {key}: expected {sorted(expected_ph)}, got {sorted(actual_ph)}"
                )
                failed = True

    names_path = strings_dir / "rows" / "names.json"
    names: list[str] = json.loads(names_path.read_text(encoding="utf-8")) if names_path.is_file() else []
    for path in sorted(strings_dir.glob("*.json")):
        lang = path.stem
        if lang == "en" or not names:
            continue
        rows_path = strings_dir / "rows" / f"{lang}.json"
        rows: dict[str, str] = json.loads(rows_path.read_text(encoding="utf-8")) if rows_path.is_file() else {}
        missing = [name for name in names if not str(rows.get(name, "")).strip()]
        if missing:
            print(f"rows {lang}: {len(missing)} of {len(names)} row names untranslated, first {missing[0]!r}")
            failed = True

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
