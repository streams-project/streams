#!/usr/bin/env python3
"""A public file may not claim a number that nothing measures.

`README.md` said the website was 1,744 lines of Rust in 15 files, the apple family 10,994 lines of
Swift, the android family 14,906 lines of Kotlin. Three of the four were already wrong the day
after they were written, and the same wrong numbers had been copied into two more README files. A
count in prose is a promise with no enforcement: nobody can tell a stale number from a lie, and no
check can tell either.

So the rule: in the files this reads, a size claim is refused. Say what a thing is and what proves
it — a command a reader can run — and leave the counting to the command.

The scope is the files whose claims are this repository's own front matter. `apps/android/README.md`
and `apps/apple/README.md` held the same stale counts and were named in the output every run until
they joined the list; on 2026-11-13 their counts were measured, found wrong in all four figures
(Kotlin 14,906 lines in 93 files against 19,215 in 122; Swift 10,994 in 75 against 14,963 in 87),
and removed rather than refreshed. The hole is closed, so `NOT_YET` is empty and both files are
held to the rule like the rest.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# The files this rule holds. A file joins this list once its counts are gone.
PUBLIC_FILES = (
    "README.md",
    "CONTRIBUTING.md",
    "apps/android/README.md",
    "apps/apple/README.md",
    # The HarmonyOS family's two front pages, added 2026-11-13. Both were written clean -- they
    # name `ls apps/harmony/surfaces/touch` and `./apps/harmony/check.sh` where a count would have
    # gone -- and both were outside this rule, so the next number written into either would have
    # been as unenforced as the four that started this check. They were measured before being
    # added: 0 claims in each.
    "apps/harmony/README.md",
    "apps/harmony/surfaces/touch/README.md",
    "apps/desktop/README.md",
)

NOT_YET: tuple[str, ...] = (
    "apps/roku/README.md",
)

# "1,744 lines", "15 files", "93 files of Kotlin", "2,185 lines of ES5". A version number, a date
# and a port are not size claims, so the unit has to be there.
#
# What that costs, named rather than left silent: a count whose unit is implied slips past. The
# root README said "152 in the television module alone" of its tests, which this never saw and
# which had drifted to 160 by 2026-11-13. The sentence was rewritten to name the command instead.
# Dropping the unit requirement is not the fix -- every port, year and version number would
# become a claim -- so the hole stays, and this comment is the only thing standing in it.
A_COUNT = re.compile(
    r"\b\d[\d,]*\s+(?:lines|files|crates|screens|words|languages|tests)\b", re.IGNORECASE
)

# Where a number with a unit is the point rather than a claim: a command's output or a code block.
# A table row is prose here — the counts this rule exists for were all inside one — so a table is
# held to the rule like any other line.
IN_CODE = re.compile(r"^\s{4,}|^```")

LEAST_READ = 6


def tracked(name: str) -> bool:
    done = subprocess.run(
        ["git", "ls-files", "--error-unmatch", "--", name],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return done.returncode == 0


def claims_in(text: str, name: str = "") -> list[str]:
    """Every size claim in a text, as `path:line: the claim`."""
    found = []
    fenced = False
    for number, line in enumerate(text.splitlines(), 1):
        if line.startswith("```"):
            fenced = not fenced
            continue
        if fenced or IN_CODE.match(line):
            continue
        for match in A_COUNT.finditer(line):
            found.append(f"{name}:{number}: {match.group()}")
    return found


def counts(name: str) -> list[str]:
    """Every size claim in a tracked file."""
    path = ROOT / name
    return claims_in(path.read_text(encoding="utf-8"), name) if path.is_file() else []


def main() -> int:
    found: list[str] = []
    read = 0
    for name in PUBLIC_FILES:
        if not tracked(name):
            found.append(f"{name}: in this rule's list and not in git")
            continue
        read += 1
        found += [
            f"{line}: a size claim nothing measures: say what proves it, not how much of it there is"
            for line in counts(name)
        ]
    elsewhere = {name: counts(name) for name in NOT_YET}
    waiting = ", ".join(
        f"{name} ({len(lines)})" for name, lines in elsewhere.items() if lines
    )
    print(
        f"claims: read {read} public files"
        + (f"; NOT held to this rule yet, and still holding counts: {waiting}" if waiting else "")
    )
    if read < LEAST_READ:
        found.append(
            f"read {read} files, expected at least {LEAST_READ}: this check is not reading the "
            "files it is about"
        )
    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
