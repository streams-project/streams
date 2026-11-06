"""The workflow runs every check, and no cross check can go quiet when a folder is missing."""

from pathlib import Path

import pytest

from . import siblings
from .siblings import CHECK, NEEDED, WORKFLOW

TESTS = Path(__file__).resolve().parent


def test_the_workflow_runs_the_one_check_and_it_reaches_this_folder():
    assert "./check" in WORKFLOW.read_text()
    assert "cd data && ./scripts/check" in CHECK.read_text()
    for name in NEEDED:
        assert (siblings.BESIDE / name).is_dir(), name


def test_no_test_answers_a_missing_sibling_with_a_skip():
    """A skip hid the ledger gate and the two cross checks from every CI run, so none is allowed."""
    # Written in parts so this file does not match its own check.
    call = "skip" + "("
    about = ("reposi" + "tor", "next to " + "this one")
    for path in sorted(TESTS.glob("test_*.py")):
        for number, line in enumerate(path.read_text().splitlines(), 1):
            quiet = call in line and any(word in line for word in about)
            assert not quiet, f"{path.name}:{number} skips instead of failing"


def test_only_one_place_builds_the_path_of_a_sibling():
    # Written in parts so this file does not match its own check.
    folder_above = "parents[" + "2]"
    for path in sorted(TESTS.glob("*.py")):
        if path.name == "siblings.py":
            continue
        assert folder_above not in path.read_text(), path.name


def test_a_missing_sibling_fails_and_names_itself(tmp_path, monkeypatch):
    monkeypatch.setattr(siblings, "BESIDE", tmp_path)
    for name in NEEDED:
        with pytest.raises(AssertionError, match=f"the {name} folder must sit at"):
            siblings.beside(name)
        (tmp_path / name).mkdir()
        assert siblings.beside(name) == tmp_path / name
    with pytest.raises(AssertionError, match="is missing from the platform repository"):
        siblings.within("platform", "contract", "ledger.csv")
