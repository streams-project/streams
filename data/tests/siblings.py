"""The folders this one reads from the same repository, and the only place their paths are built.

A cross check that cannot see one fails and says which is missing. It does not skip: a skip once
made the ledger gate and the two cross checks silently absent from every CI run.
"""

from __future__ import annotations

from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
BESIDE = HERE.parent
WORKFLOW = BESIDE / ".github" / "workflows" / "check.yml"
CHECK = BESIDE / "check"

# Each name, and what this repository reads from it.
NEEDED = {
    "platform": "the contract: the ledger, the generated schemas and the refused fixtures",
    "server": "the type the server reads a data set with",
}


def beside(name: str) -> Path:
    """The folder of a repository next to this one, or a failure naming what is missing."""
    assert name in NEEDED, f"{name} is not a repository this one reads"
    folder = BESIDE / name
    if not folder.is_dir():
        raise AssertionError(
            f"the {name} folder must sit at {folder}, because this folder reads "
            f"{NEEDED[name]}. It is part of this repository; a run by hand needs it "
            f"cloned next to this folder."
        )
    return folder


def within(name: str, *parts: str) -> Path:
    """A file or folder inside a sibling repository, or a failure naming what is missing."""
    path = beside(name).joinpath(*parts)
    if not path.exists():
        raise AssertionError(f"{path} is missing from the {name} repository")
    return path
