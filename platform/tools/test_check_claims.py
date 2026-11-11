#!/usr/bin/env python3
"""Tests for tools/check_claims.py: the wrong numbers of 2026-11-11 must not come back.

Run from anywhere with: python3 tools/test_check_claims.py
"""

import importlib.util
import io
import unittest
from contextlib import redirect_stdout
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location("check_claims", TOOLS / "check_claims.py")
claims = importlib.util.module_from_spec(spec)
spec.loader.exec_module(claims)

# The three claims that were wrong in the Status table, written the way they were written.
WERE_WRONG = ("1,744 lines of Rust in 15 files", "10,994 lines of Swift", "14,906 lines of Kotlin")


# The row as it stood in README.md, inside the Status table: that is where every wrong count was,
# so a rule that passed over table rows would have passed over all of them.
THE_WRONG_ROW = (
    "| Website (`apps/web`) | Runs | 1,744 lines of Rust in 15 files. It is a workspace crate, so "
    "`./check` builds, lints and tests it |"
)


class Claims(unittest.TestCase):
    def test_the_row_that_was_wrong_is_caught_where_it_lived(self):
        found = claims.claims_in(THE_WRONG_ROW, "README.md")
        self.assertEqual(len(found), 2, found)

    def test_a_number_inside_a_code_block_is_not_a_claim(self):
        self.assertEqual(claims.claims_in("```\n1,744 lines\n```\n"), [])
        self.assertEqual(claims.claims_in("    1,744 lines of output"), [])

    def test_a_count_in_prose_is_found(self):
        for text in WERE_WRONG:
            with self.subTest(text=text):
                self.assertTrue(claims.A_COUNT.search(text))

    def test_a_version_a_date_and_a_port_are_not_counts(self):
        for text in ("version 0.1.0", "on 2026-11-07", "port 8080", "NDK 27.2.12479018"):
            with self.subTest(text=text):
                self.assertIsNone(claims.A_COUNT.search(text))

    def test_the_public_files_hold_no_count_today(self):
        for name in claims.PUBLIC_FILES:
            with self.subTest(name=name):
                self.assertEqual(claims.counts(name), [])

    def test_the_whole_check_passes_and_names_what_it_does_not_hold(self):
        text = io.StringIO()
        with redirect_stdout(text):
            code = claims.main()
        self.assertEqual(code, 0, text.getvalue())
        self.assertIn("read 2 public files", text.getvalue())

    def test_a_file_in_the_list_that_git_does_not_hold_is_a_failure(self):
        held = claims.PUBLIC_FILES
        claims.PUBLIC_FILES = ("README.md", "NO-SUCH-FILE.md")
        try:
            text = io.StringIO()
            with redirect_stdout(text):
                code = claims.main()
        finally:
            claims.PUBLIC_FILES = held
        self.assertEqual(code, 1)
        self.assertIn("NO-SUCH-FILE.md: in this rule's list and not in git", text.getvalue())


if __name__ == "__main__":
    unittest.main()
