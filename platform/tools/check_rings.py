#!/usr/bin/env python3
"""The rings: every app family is its own hexagon, and this makes that true instead of a claim.

A family has `ports`, `adapters` and `surfaces`. A surface draws, and reaches the world only through
a port. So a word that names a browser or system feature behind a port may not appear in a surface.

A family may keep its surfaces in more than one folder, so each one names them by pattern and says
how many files the scan must read. Without that count a scan that reached half the surfaces would
pass green: the apple family kept its phone surfaces outside the one folder this read, and nineteen
files went unscanned for weeks.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# A family's folder, what its code ends in, where its surfaces live, how many surface files the scan
# must read, and what those surfaces may not name. Raise `least` when surfaces are added, lower it
# when they go: it is the only thing that tells a whole scan from half of one.
FAMILIES = {
    "tv": {
        "code": "apps/tv/page",
        "ends": {".js"},
        "surfaces": ("surfaces",),
        "least": 19,
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
        "surfaces": ("surfaces",),
        "least": 5,
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
        "surfaces": ("surfaces",),
        "least": 84,
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
        # `surfaces` at the family root is a link to one of these folders, so the pattern reads them
        # all: a phone, a television and the parts they share are each a surface.
        "surfaces": ("surfaces", "Sources/Surfaces*"),
        "least": 71,
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


def surface_files(code: Path, family: dict) -> list[Path]:
    """Every surface file of a family, each one once.

    A folder may be reached twice, once by its own name and once through a link to it, so the files
    are gathered by the path they really have.
    """
    folders = {
        folder.resolve()
        for pattern in family["surfaces"]
        for folder in code.glob(pattern)
        if folder.is_dir()
    }
    files = {
        path.resolve()
        for folder in folders
        for path in folder.rglob("*")
        if path.is_file() and path.suffix in family["ends"] and path.name not in family["skip"]
    }
    return sorted(files)


def check(name: str, family: dict) -> list[str]:
    found = []
    code = ROOT / family["code"]
    ports = sorted(code.glob("ports*"))
    if not ports:
        found.append(f"{name}: no ports")
    for part in PARTS:
        if not (code / part).is_dir():
            found.append(f"{name}: no {part} folder")
    files = surface_files(code, family)
    # A scan that read too few files proves nothing, so too few is itself a failure.
    if len(files) < family["least"]:
        found.append(
            f"{name}: read {len(files)} surface files, expected at least {family['least']}: "
            f"a surfaces folder is missing from `surfaces`, or lower `least` because files went"
        )
    for path in files:
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
