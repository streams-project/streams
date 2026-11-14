#!/usr/bin/env python3
"""Every file on this disk that git does not hold, for the checks that would otherwise miss it.

The hole this closes, measured on 2026-11-13. `launcher/tests/one_name.rs` walks `git ls-files` and
refuses the brand's name in a program file. Two new types carrying that name were written, the test
was run, and it was green — because the files were not committed yet. It went red the instant they
were added, by which time the author had been told nothing. Every check in this folder that reads
`git ls-files` had the same hole: `check_private.py` read no uncommitted file at all, so a home
folder or a credential in one passed, and `check_rings.py` could not see an uncommitted code file
sitting outside every ring.

The other answer was to make `./check` refuse to run while `check_tracked.py` is red. That is
already what happens — `./check` is `set -eu` and `check_tracked.py` is its first line — and it did
not close this, because the defect reached its author through one check run on its own, which is how
a check is run while the work is being done. It also costs the wrong thing: it ties "I have not
committed yet" to "no rule in this repository is proved", which teaches an agent to commit in order
to get a green run.

So each check reads the working tree beside the index, and the escape is the one `check_tracked.py`
already names: a scratch file of your own goes in `.gitignore` or `.git/info/exclude`, and git stops
reporting it, so this stops reading it. A file that is on disk and not ignored is held to the rules
whether or not anybody has added it yet.

What this is NOT: a file git has been told to ignore is not read here, and neither is anything
inside a folder a build wrote, because git ignores those too. So a rule can still be hidden by
`.gitignore`, on purpose, and that is the one escape there is.
"""

from __future__ import annotations

import subprocess
from pathlib import Path


def loose(root: Path, under: str = "") -> list[str]:
    """Every path git does not hold and has not been told to ignore, newest state of the disk.

    `under` narrows it to one folder, the way `git ls-files -- <path>` narrows the index.
    """
    args = ["git", "status", "--porcelain", "--untracked-files=all", "-z"]
    if under:
        args += ["--", under]
    done = subprocess.run(args, cwd=root, capture_output=True, text=True, check=True)
    return [line[3:] for line in done.stdout.split("\0") if line.startswith("?? ")]
