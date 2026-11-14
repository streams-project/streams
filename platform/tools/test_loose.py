#!/usr/bin/env python3
"""Tests for tools/loose.py, against the hole of 2026-11-13.

The defect: `launcher/tests/one_name.rs` walks `git ls-files` and refuses the brand's name in a
program file. Two new types carrying that name were written, the test was run, it was green, and it
turned red on the commit that added them -- by which time its author had been told nothing. Every
check in this folder built on `git ls-files` had the same hole.

So each of them reads the working tree beside the index, and these are the tests that say so. They
plant a real file on the real tree, because the thing being proved is that a file on disk and not
in git cannot hide a rule violation, and a fixture tree would prove the opposite of that.

Run from anywhere with: python3 tools/test_loose.py
"""

import importlib.util
import io
import unittest
from contextlib import redirect_stdout
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
ROOT = TOOLS.parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, TOOLS / f"{name}.py")
    one = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(one)
    return one


loose = module("loose")
rings = module("check_rings")
private = module("check_private")

# A home path, built from its halves: written out whole it would be a finding in this very file,
# which is `check_private.py` working. The same trick `test_check_private.py` uses.
A_HOME_PATH = 'AT = "/%s/somebody/code/platform"\n' % "Users"


class Planted:
    """One real file on the real tree, removed again whatever happens."""

    def __init__(self, name: str, text: str):
        self.path = ROOT / name
        self.name = name
        self.text = text

    def __enter__(self):
        if self.path.exists():
            raise AssertionError(f"{self.name} is already here: this test would eat it")
        self.path.write_text(self.text, encoding="utf-8")
        return self

    def __exit__(self, *_):
        self.path.unlink(missing_ok=True)
        return False


class Loose(unittest.TestCase):
    def test_a_file_written_and_not_added_is_reported(self):
        name = "platform/tools/zz_loose_probe.py"
        with Planted(name, "# nothing\n"):
            self.assertIn(name, loose.loose(ROOT))
            self.assertIn(name, loose.loose(ROOT, "platform/tools"))
            self.assertNotIn(name, loose.loose(ROOT, "apps"))
        self.assertNotIn(name, loose.loose(ROOT))

    def test_nothing_git_holds_is_reported_as_loose(self):
        # Two files this repository has held for weeks. Not this file and not `loose.py`: both are
        # new, and on the run that writes them they are exactly what this reports, which is the
        # point of it.
        here = set(loose.loose(ROOT))
        self.assertNotIn("README.md", here)
        self.assertNotIn("platform/tools/check_rings.py", here)

    def test_a_build_folder_is_never_reported(self):
        # git is told to ignore them, which is also the one escape a person has.
        for name in loose.loose(ROOT):
            with self.subTest(name=name):
                parts = set(Path(name).parts)
                self.assertFalse(parts & {"target", "build", "node_modules", ".build"}, name)


class ThroughTheChecks(unittest.TestCase):
    def test_a_new_file_in_no_ring_is_a_failure_before_anybody_adds_it(self):
        name = "apps/desktop/src/zz_stray_probe.rs"
        with Planted(name, "//! a probe\npub fn nothing() {}\n"):
            found = rings.check("desktop", rings.FAMILIES["desktop"])
            self.assertTrue(
                any("zz_stray_probe.rs" in line and "in no ring" in line for line in found), found
            )
        self.assertEqual(rings.check("desktop", rings.FAMILIES["desktop"]), [])

    def test_a_leak_in_a_file_nobody_added_is_refused(self):
        name = "platform/tools/zz_private_probe.py"
        with Planted(name, A_HOME_PATH):
            self.assertIn(name, private.files())
            text = io.StringIO()
            with redirect_stdout(text):
                code = private.main()
            self.assertEqual(code, 1)
            self.assertIn(f"{name}:1: names a home folder", text.getvalue())
        self.assertNotIn(name, private.files())

    def test_the_index_is_still_read_and_is_most_of_what_is_read(self):
        # The disk half is an addition, not a replacement: a check that stopped reading git would
        # pass this folder's floors on an empty working tree otherwise.
        held = set(private.tracked())
        self.assertGreater(len(held), 1000)
        self.assertTrue(set(private.files()) >= held)


if __name__ == "__main__":
    unittest.main()
