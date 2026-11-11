#!/usr/bin/env python3
"""Tests for tools/check_patterns.py, including the drift it was written to catch.

The pattern that shipped until 2026-11-11 is planted here by hand, so these tests fail if anybody
ever makes the check tolerant of it again.

Run from anywhere with: python3 tools/test_check_patterns.py
"""

import importlib.util
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location("check_patterns", TOOLS / "check_patterns.py")
patterns = importlib.util.module_from_spec(spec)
spec.loader.exec_module(patterns)

# What the generated clients carried before this check existed: ten digits, so `movie:9999999999`
# was a title key to Kotlin and Swift and not to the core.
DRIFTED = "^(movie|tv):[1-9][0-9]{0,9}$"


class Patterns(unittest.TestCase):
    def test_the_repository_agrees_with_itself(self):
        self.assertEqual(patterns.main(), 0)

    def test_the_bound_is_read_from_the_rust_source(self):
        self.assertEqual(patterns.rust_const("MAX_TITLE_ID", "src/title.rs"), 2147483647)

    def test_the_old_pattern_fails_the_bound(self):
        found = patterns.bounds({"TitleKey": DRIFTED})
        self.assertTrue(found)
        self.assertIn("2147483648", " ".join(found))

    def test_the_old_pattern_fails_the_corpus(self):
        version = patterns.wire_version()
        found, proved, read = patterns.corpus(version, {"TitleKey": DRIFTED})
        self.assertTrue(any("id_over_max" in line for line in found))
        self.assertTrue(any("id_ten_digits" in line for line in found))
        self.assertGreater(read, 5)

    def test_the_pattern_in_the_tree_passes_the_corpus(self):
        version = patterns.wire_version()
        types = patterns.patterned_types()
        found, proved, read = patterns.corpus(version, types)
        self.assertEqual(found, [])
        self.assertEqual(proved["TitleKey"], {"accepted", "refused"})

    def test_a_pattern_that_is_stricter_than_the_core_also_fails(self):
        found, _, _ = patterns.corpus(patterns.wire_version(), {"TitleKey": "^movie:[0-9]+$"})
        self.assertTrue(any("series_same_id" in line for line in found))

    def test_a_pattern_in_a_dialect_this_cannot_judge_is_refused(self):
        self.assertTrue(patterns.dialect("TitleKey", r"^(movie|tv):\d+$"))
        self.assertTrue(patterns.dialect("X", "^(?i)a$"))
        self.assertEqual(patterns.dialect("TitleKey", patterns.patterned_types()["TitleKey"]), [])

    def test_a_pattern_missing_from_one_client_is_found(self):
        self.assertTrue(patterns.one_copy("TitleKey", "^never-generated$"))
        self.assertEqual(
            patterns.one_copy("TitleKey", patterns.patterned_types()["TitleKey"]), []
        )

    def test_the_whole_string_is_asked_for(self):
        self.assertTrue(patterns.matches("^[0-9]{2}$", "12"))
        self.assertFalse(patterns.matches("^[0-9]{2}$", "12\n"))
        self.assertFalse(patterns.matches("^[0-9]{2}$", "123"))


if __name__ == "__main__":
    unittest.main()
