"""The repositories that must sit next to this one, and the only place their paths are built.

A cross check that cannot see its sibling fails and says which one is missing. It does not skip:
a skip made the ledger gate and the two cross checks silently absent from every CI run, which
checked out this repository alone.
"""

from __future__ import annotations

from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
BESIDE = HERE.parent
WORKFLOW = HERE / ".github" / "workflows" / "check.yml"

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
            f"the {name} repository must sit at {folder}, because this repository reads "
            f"{NEEDED[name]}. .github/workflows/check.yml checks it out; a run by hand needs it "
            f"cloned next to this folder."
        )
    return folder


def within(name: str, *parts: str) -> Path:
    """A file or folder inside a sibling repository, or a failure naming what is missing."""
    path = beside(name).joinpath(*parts)
    if not path.exists():
        raise AssertionError(f"{path} is missing from the {name} repository")
    return path
