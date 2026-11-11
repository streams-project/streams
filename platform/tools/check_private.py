#!/usr/bin/env python3
"""Refuses a private reference in any file of this repository, not only in the data builder's own.

The data builder scans the folder it builds from before every build. Nothing scanned the rest, so a
tool under platform/ carried its author's home folder into a public repository and no check saw it.
This reads every tracked file with that same scanner, and adds the one rule that needs no private
list: a file may not name a person's home folder.

The scanner's word list is private, named by an environment variable, so a copy of this project has
none. That is why this says out loud which rules it ran and how many files it read: a scan that
quietly found nothing is the fault being closed here, not a pass.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "data" / "src"))

from databuild import scan  # noqa: E402

# A path under somebody's home folder. Nothing that ships may hold one: it names a person, and it
# names a machine nobody else has. The folder has to start the path, so a temporary folder that
# merely ends in the word (a test's /tmp/home/data) is not a person's home.
HOME = re.compile(r"(?<![A-Za-z0-9._-])/(?:Users|home)/[A-Za-z][A-Za-z0-9._-]*")

# Where a home folder belongs: the build runner's own temporary folder, which is the runner's name
# and not a person's.
ALLOWED_HOME = ("/home/runner",)

# A web address of this product is not a machine's path, and the two are written the same: an
# address of `/home/<page>` on a site reads exactly like the folder of a person called `<page>` on
# Linux. So the addresses are read from the two files that declare them, and a match that is one of
# those addresses, or inside one, is not a home folder. No address is written down here: the day a
# home page address is added to the table, this stops reading it as a person, with no edit.
ADDRESS_TABLE = Path("platform") / "contract" / "src" / "address.rs"
ROUTE_TABLE = Path("platform") / "contract" / "generated" / "openapi.json"
RUST_ADDRESS = re.compile(r'pub const [A-Z_0-9]+: &str = "(/[^"]*)"')

# A link is an address too, and these are how a link is written. A match inside one of these tokens
# is a place on a site, not a folder on a machine.
LINK_STARTS = ("http://", "https://", "//")

# Read as bytes, never as text. `scan.BINARY_SUFFIXES` is the data builder's list, for the files a
# build touches; these are the kinds this repository also tracks, and reading one as text is how a
# font or a jar comes to look like a refused word.
MORE_BINARY_SUFFIXES = frozenset({".jar", ".ttf", ".otf", ".woff", ".woff2", ".wasm", ".ico", ".webp", ".mp4", ".zip"})

# Which of the scanner's findings apply outside a built data set. An address and a long hex string
# are refused inside data/, where a stray address means a capture leaked into a build; across the
# repository a local address, a licence link and a fixture digest all belong, so those two kinds are
# read here through KEY_SHAPES instead of being refused outright.
KINDS = ("refused word", "refused file name", "capture tag prefix")

# The shapes of a real credential, as opposed to the digests this repository writes by the hundred.
# A line the scanner called key shaped is kept only when it looks like one of these.
KEY_SHAPES = re.compile(r"\beyJ[A-Za-z0-9_-]{20,}|\bgh[pousr]_[A-Za-z0-9]{20,}|\bhf_[A-Za-z0-9]{20,}|\bsk-[A-Za-z0-9_-]{20,}")

# The scan must read at least this many files under each of these folders. The numbers are the proof
# that it reached the repository and not a corner of it: lower one only when files really went.
LEAST = {"platform": 400, "apps": 300, "server": 150, "data": 80, "strings": 20}

# Files whose whole point is to hold the shape of a credential: this one and the scanner it uses.
# They are exempt from that one rule and from no other. Exempting them from every rule, as this did
# until 2026-11-11, made the two files holding the patterns the only two files where the leak this
# check exists to catch could not be caught.
KEY_SHAPE_FILES = (
    "platform/tools/check_private.py",
    "data/src/databuild/scan.py",
)


def tracked() -> list[str]:
    """Every file git knows about, so build output and ignored files are never read."""
    done = subprocess.run(
        ["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return [name for name in done.stdout.split("\0") if name]


def addresses() -> tuple[str, ...]:
    """Every address this product publishes that a home folder could be confused with."""
    found = set()
    text = (ROOT / ADDRESS_TABLE).read_text(encoding="utf-8")
    found |= {match.group(1).rstrip("/") for match in RUST_ADDRESS.finditer(text)}
    routes = json.loads((ROOT / ROUTE_TABLE).read_text(encoding="utf-8"))
    found |= {path.rstrip("/") for path in routes.get("paths", {})}
    # Only an address that starts the same way as a home folder can be mistaken for one, so no
    # other address can widen this.
    return tuple(sorted(path for path in found if path.startswith(("/home", "/Users"))))


def in_a_link(line: str, start: int) -> bool:
    """Whether the match sits inside a written out link."""
    token = line[:start].rsplit(" ", 1)[-1]
    for opener in ('"', "'", "(", "[", "<", "=", "`"):
        token = token.rsplit(opener, 1)[-1]
    return token.startswith(LINK_STARTS)


def home_folders(text: str, path: str, published: tuple[str, ...]) -> list[str]:
    found = []
    for number, line in enumerate(text.splitlines(), 1):
        for match in HOME.finditer(line):
            hit = match.group()
            if hit in ALLOWED_HOME or in_a_link(line, match.start()):
                continue
            if any(hit == address or hit.startswith(address + "/") for address in published):
                continue
            found.append(f"{path}:{number}: names a home folder: {hit}")
    return found


def keep(finding: scan.Finding, text: str, name: str) -> bool:
    if finding.what in KINDS:
        return True
    if finding.what != "text shaped like a key":
        return False
    if name in KEY_SHAPE_FILES:
        return False
    lines = text.splitlines()
    line = lines[finding.line - 1] if 0 < finding.line <= len(lines) else ""
    return bool(KEY_SHAPES.search(line))


def main() -> int:
    found: list[str] = []
    read: dict[str, int] = {}
    published = addresses()
    for name in tracked():
        path = ROOT / name
        if not path.is_file():
            continue
        read[name.split("/")[0]] = read.get(name.split("/")[0], 0) + 1
        if scan.scan_name(path.name):
            found.append(f"{name}:0: refused file name")
        if path.suffix in scan.BINARY_SUFFIXES or path.suffix in MORE_BINARY_SUFFIXES:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        found += home_folders(text, name, published)
        found += [
            f"{name}:{f.line}: {f.what}" for f in scan.scan_text(text) if keep(f, text, name)
        ]

    # Which rules really ran. A file name is refused only by the private word list, exactly as a
    # private word is, so it belongs on the same side of this sentence: saying it ran without the
    # list was the overstatement this check was written to stop making.
    rules = "home folders, credential shapes"
    if os.environ.get(scan.REFUSED_FILE):
        rules += ", file names, private words, capture tags"
    else:
        rules += (
            f"; NOT file names, private words or capture tags, {scan.REFUSED_FILE} is not set"
        )
    print(f"private: read {sum(read.values())} files, refusing {rules}")

    for folder, least in sorted(LEAST.items()):
        if read.get(folder, 0) < least:
            found.append(
                f"{folder}: read {read.get(folder, 0)} files, expected at least {least}: "
                "this scan is not reading the repository"
            )

    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
