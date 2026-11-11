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

# Files whose whole point is to hold a pattern that these rules refuse.
ALLOWED_FILES = (
    "platform/tools/check_private.py",
    "data/src/databuild/scan.py",
)


def tracked() -> list[str]:
    """Every file git knows about, so build output and ignored files are never read."""
    done = subprocess.run(
        ["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return [name for name in done.stdout.split("\0") if name]


def home_folders(text: str, path: str) -> list[str]:
    found = []
    for number, line in enumerate(text.splitlines(), 1):
        for match in HOME.finditer(line):
            if match.group() not in ALLOWED_HOME:
                found.append(f"{path}:{number}: names a home folder: {match.group()}")
    return found


def keep(finding: scan.Finding, text: str) -> bool:
    if finding.what in KINDS:
        return True
    if finding.what != "text shaped like a key":
        return False
    lines = text.splitlines()
    line = lines[finding.line - 1] if 0 < finding.line <= len(lines) else ""
    return bool(KEY_SHAPES.search(line))


def main() -> int:
    found: list[str] = []
    read: dict[str, int] = {}
    for name in tracked():
        path = ROOT / name
        if not path.is_file() or name in ALLOWED_FILES:
            continue
        read[name.split("/")[0]] = read.get(name.split("/")[0], 0) + 1
        if scan.scan_name(path.name):
            found.append(f"{name}:0: refused file name")
        if path.suffix in scan.BINARY_SUFFIXES:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        found += home_folders(text, name)
        found += [f"{name}:{f.line}: {f.what}" for f in scan.scan_text(text) if keep(f, text)]

    rules = "home folders, file names, credential shapes"
    if os.environ.get(scan.REFUSED_FILE):
        rules += ", private words, capture tags"
    else:
        rules += f"; NOT private words or capture tags, {scan.REFUSED_FILE} is not set"
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
