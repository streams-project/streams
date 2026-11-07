#!/usr/bin/env python3
"""Check that every strings/<lang>.json file has all keys and matching placeholders from en.json."""

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
            expected_ph = placeholders(en_data[key])
            actual_ph = placeholders(data[key])
            if expected_ph != actual_ph:
                print(
                    f"placeholder {lang} {key}: expected {sorted(expected_ph)}, got {sorted(actual_ph)}"
                )
                failed = True

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
