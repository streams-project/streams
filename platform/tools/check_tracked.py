#!/usr/bin/env python3
"""The checks prove the tree in git, not the tree on this disk.

A file that exists here and not in git breaks a fresh clone, and nothing noticed: on 2026-11-11
five files were untracked and three of them were declared by tracked code — a `pub mod data;` with
no `data.rs`, a `mod conformance;` with no folder, and an Android manifest pointing at two XML
files that were not there. Every check passed on the machine that wrote them.

Two rules, both about the same thing:

  referenced   an untracked file whose name is declared by a tracked file in its own part of the
               tree, or by `check` or a workflow, is a failure: a clone cannot build
  named        every path written in `check` itself is a path git holds

A scratch file of your own is silenced the way everything else is silenced: name it in `.gitignore`
or in `.git/info/exclude`, and `git status` stops reporting it, so this stops reading it. That is
the only escape, on purpose — a list inside this file would be the next thing to go stale.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# A file whose own name says nothing, so the folder holding it is what tracked code declares.
FOLDER_NAMES = {"mod.rs", "__init__.py", "index.js", "index.ts", "build.gradle.kts", "Cargo.toml"}

# Where a reference to a file may be written: its own top folder, and the two places that run
# everything regardless of folder.
ALWAYS = ("check", ".github")

# A path inside `check`: a word with a slash in it and a suffix, which is how every path in that
# file is written.
IN_CHECK = re.compile(r"(?<![\w/.-])([\w.-]+/[\w./-]*\.[a-z]{1,4})\b")

# `check` names this many paths today, with a margin: the floor is here to catch this file failing
# to read `check` at all, not to count the lines of it.
LEAST_NAMED = 12


def git(*args: str) -> str:
    done = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return done.stdout


def untracked() -> list[str]:
    """Every file on this disk that git does not hold and has not been told to ignore."""
    lines = git("status", "--porcelain", "--untracked-files=all", "-z").split("\0")
    return [line[3:] for line in lines if line.startswith("?? ")]


def tracked() -> set[str]:
    return {name for name in git("ls-files", "-z").split("\0") if name}


def token(name: str) -> str:
    """The word tracked code would use to declare this file."""
    path = Path(name)
    if path.name in FOLDER_NAMES and path.parent.name:
        return path.parent.name
    return path.stem


def references(name: str) -> list[str]:
    """Tracked files that declare this untracked file, as `path:line: text`."""
    top = name.split("/")[0]
    where = [top, *ALWAYS]
    done = subprocess.run(
        ["git", "grep", "-I", "-n", "-w", "-F", token(name), "--", *where],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return [line for line in done.stdout.splitlines() if line.strip()]


def named_in_check(held: set[str]) -> tuple[list[str], int]:
    """Every path `check` names is a path git holds."""
    found = []
    text = (ROOT / "check").read_text("utf-8")
    seen = set()
    for match in IN_CHECK.finditer(text):
        name = match.group(1)
        if name in seen:
            continue
        seen.add(name)
        if name in held:
            continue
        # `check` runs some paths from inside a folder, so a plain name is tried there too.
        if any(held_name.endswith("/" + name) for held_name in held):
            continue
        if not (ROOT / name).exists() and not any(
            (ROOT / top / name).exists() for top in ("platform", "server", "data")
        ):
            continue
        found.append(f"check: runs {name}, which git does not hold: a clone cannot run ./check")
    return found, len(seen)


def main() -> int:
    held = tracked()
    found: list[str] = []
    loose = untracked()
    for name in loose:
        where = references(name)
        if where:
            found.append(
                f"{name}: not in git, and {len(where)} tracked file(s) declare it, first at "
                f"{where[0].split(':', 2)[0]}:{where[0].split(':', 2)[1]}: `git add` it, or name "
                "it in .gitignore if it is yours alone"
            )
    lines, named = named_in_check(held)
    found += lines
    print(
        f"tracked: git holds {len(held)} files, {len(loose)} on this disk it does not, "
        f"{named} paths named in check"
    )
    if named < LEAST_NAMED:
        found.append(
            f"check: read {named} paths out of it, expected at least {LEAST_NAMED}: this check is "
            "not reading the file it is about"
        )
    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
