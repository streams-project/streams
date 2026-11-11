#!/usr/bin/env python3
"""Tests for tools/check_rings.py, against the blindness found on 2026-11-11.

The old check asserted a count and nothing else, so two things passed green: a `surfaces` pattern
that matched no folder at all, and a new folder of screens that no pattern named. Both are planted
here on the real tree, by handing `check` a family whose patterns are wrong, so no fixture tree is
needed and nothing is written anywhere.

Run from anywhere with: python3 tools/test_check_rings.py
"""

import importlib.util
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location("check_rings", TOOLS / "check_rings.py")
rings = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rings)


def family(name, **changes):
    """The real family, with something changed, so a planted fault is one difference from truth."""
    one = dict(rings.FAMILIES[name])
    one.update(changes)
    return one


class Rings(unittest.TestCase):
    def test_every_family_is_in_its_rings_today(self):
        self.assertEqual(rings.main(), 0)

    def test_a_surfaces_pattern_that_names_no_folder_is_a_failure(self):
        folders, blind = rings.surface_folders(
            rings.ROOT / "apps/apple", family("apple", surfaces=("surfaces", "Sources/Screens*"))
        )
        self.assertTrue(blind)
        self.assertIn("Sources/Screens*", blind[0])
        found = rings.check("apple", family("apple", surfaces=("surfaces", "Sources/Screens*")))
        self.assertTrue(any("is no folder of this family" in line for line in found))

    def test_the_apple_blindness_of_2026_11_11_is_caught_now(self):
        # What shipped for weeks: only the folder at the family root was read, so the phone
        # surfaces were never scanned and the count still cleared its floor.
        found = rings.check("apple", family("apple", surfaces=("surfaces",)))
        strays = [line for line in found if "in no ring of this family" in line]
        self.assertTrue(strays)
        self.assertTrue(any("SurfacesPhone" in line for line in strays))

    def test_a_new_folder_of_screens_nobody_globbed_is_a_failure(self):
        # The same shape as the defect above, from the other end: a real folder of surface files
        # that the patterns do not name is a failure even though the count went up, not down.
        one = family("apple", surfaces=("Sources/SurfacesTV",), least=1)
        found = rings.check("apple", one)
        self.assertTrue(any("SurfacesShared" in line for line in found))
        self.assertTrue(any("in no ring of this family" in line for line in found))

    def test_the_surfaces_folder_at_the_root_must_be_in_the_read_set(self):
        found = rings.check("apple", family("apple", surfaces=("Sources/SurfacesPhone",)))
        self.assertTrue(
            any("surfaces folder at the family root is not in the read set" in line for line in found)
        )

    def test_every_floor_keeps_a_margin_under_the_real_count(self):
        # `least` equal to the count makes an ordinary deletion an architecture failure, which
        # invites lowering the number, which is how the first blindness was hidden.
        for name, one in rings.FAMILIES.items():
            with self.subTest(name=name):
                read = len(rings.surface_files(rings.ROOT / one["code"], one))
                self.assertGreater(read, one["least"], f"{name}: least has no margin")

    def test_the_floor_still_fires_on_a_read_of_almost_nothing(self):
        found = rings.check("web", family("web", least=1000))
        self.assertTrue(any("under the floor of 1000" in line for line in found))

    def test_the_family_files_are_read_from_git_and_not_from_this_disk(self):
        for name, one in rings.FAMILIES.items():
            with self.subTest(name=name):
                held = rings.tracked_code(rings.ROOT / one["code"], one)
                self.assertTrue(held)
                for path in held:
                    self.assertFalse(path.is_symlink())
                    self.assertIn(path.suffix, one["ends"])


if __name__ == "__main__":
    unittest.main()
