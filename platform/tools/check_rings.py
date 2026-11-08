#!/usr/bin/env python3
"""The rings: every app family is its own hexagon, and this makes that true instead of a claim.

A family has `ports`, `adapters` and `surfaces`. A surface draws, and reaches the world only through
a port. So a word that names a browser or system feature behind a port may not appear in a surface.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# A family's folder, what its code ends in, where its parts live, and what its surfaces may not name.
FAMILIES = {
    "tv": {
        "code": "apps/tv/page",
        "ends": {".js"},
        "skip": set(),
        "banned": {
            "the keys port": r"\bkeyCode\b|\baddEventListener\(\"keydown\"",
            "the network port": r"\bXMLHttpRequest\b|\bfetch\(",
            "the secrets port": r"\blocalStorage\b|\bsessionStorage\b",
            "the player port": r"\bHls\b|\bcreateElement\(\"video\"\)",
        },
    },
    "web": {
        "code": "apps/web/src",
        "ends": {".rs"},
        "skip": set(),
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bfetch_with|\bRequestInit\b|\breqwest\b",
            "the secrets port": r"\blocal_storage\b|\bsession_storage\b",
            "the address port": r"\bhistory\(\)|\bpush_state",
            "the player port": r"\bwebPlayer\b|\bHtmlVideoElement\b",
            "a port, and adapters are not for surfaces": r"crate::adapters",
        },
    },
    "android": {
        "code": "apps/android",
        "ends": {".kt"},
        "skip": set(),
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bjava\.net\b|\bokhttp3\b|\bHttpURLConnection\b",
            "the secrets port": r"\bSharedPreferences\b",
            "the player port": r"\bandroidx\.media3\b|\bExoPlayer\b",
        },
    },
    "apple": {
        "code": "apps/apple",
        "ends": {".swift"},
        "skip": set(),
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bURLSession\b",
            "the secrets port": r"\bSecItem\w*\b",
            "the player port": r"\bAVPlayer\b",
        },
    },
}
PARTS = ("adapters", "surfaces")


def check(name: str, family: dict) -> list[str]:
    found = []
    code = ROOT / family["code"]
    ports = sorted(code.glob("ports*"))
    if not ports:
        found.append(f"{name}: no ports")
    for part in PARTS:
        if not (code / part).is_dir():
            found.append(f"{name}: no {part} folder")
    surfaces = code / "surfaces"
    for path in sorted(surfaces.rglob("*")) if surfaces.is_dir() else []:
        if path.suffix not in family["ends"] or path.name in family["skip"]:
            continue
        text = path.read_text("utf-8")
        for port, words in family["banned"].items():
            for match in re.finditer(words, text):
                line = text[: match.start()].count("\n") + 1
                found.append(f"{path.relative_to(ROOT)}:{line}: reach this through {port}: {match.group()}")
    return found


def main() -> int:
    found = [line for name, family in FAMILIES.items() for line in check(name, family)]
    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
