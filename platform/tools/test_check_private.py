#!/usr/bin/env python3
"""Tests for tools/check_private.py, one per defect found on 2026-11-11.

Nothing here writes to git: `tracked` is the only door to the repository and the tests that need a
different tree replace it.

Run from anywhere with: python3 tools/test_check_private.py
"""

import importlib.util
import io
import os
import unittest
from contextlib import redirect_stdout
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location("check_private", TOOLS / "check_private.py")
private = importlib.util.module_from_spec(spec)
spec.loader.exec_module(private)

# Built at run time from its two halves, because this file is read by the check it tests: a home
# path written out here would be a finding in this very file, which is the rule working.
A_HOME_PATH = 'let folder = "/%s/somebody/code/platform";' % "Users"
A_LINUX_HOME_PATH = "cd /%s/somebody/platform && ./check" % "home"


def run_main():
    text = io.StringIO()
    with redirect_stdout(text):
        code = private.main()
    return code, text.getvalue()


class Home(unittest.TestCase):
    def test_a_home_path_in_a_file_is_refused(self):
        self.assertTrue(private.home_folders(A_HOME_PATH, "a.rs", ()))
        self.assertTrue(private.home_folders(A_LINUX_HOME_PATH, "a.sh", ()))

    def test_a_folder_merely_ending_in_the_word_is_not_a_home(self):
        self.assertEqual(private.home_folders("/tmp/home/data", "a.rs", ()), [])

    def test_the_runner_is_not_a_person(self):
        self.assertEqual(private.home_folders("TMPDIR=/home/runner/tmp", "a.yml", ()), [])

    def test_an_address_of_this_product_is_not_a_home_folder(self):
        # The defect: a route table holding a home page address reddened the first step of ./check
        # on a file with nothing private in it.
        published = ("/" + "home",)
        a_home_page = 'pub const HOME: &str = "/%s/latest";' % "home"
        self.assertEqual(private.home_folders(a_home_page, "address.rs", published), [])
        # ... and a real home path is still refused on the same line shape.
        self.assertTrue(private.home_folders(A_HOME_PATH, "address.rs", published))

    def test_a_link_is_an_address_and_not_a_folder(self):
        for line in (
            "see https://example.test/home/getting-started for more",
            '<a href="//example.test/home/account">account</a>',
        ):
            with self.subTest(line=line):
                self.assertEqual(private.home_folders(line, "help.md", ()), [])

    def test_the_addresses_come_from_the_tables_and_are_not_written_down(self):
        published = private.addresses()
        self.assertIsInstance(published, tuple)
        for address in published:
            self.assertTrue(address.startswith(("/home", "/Users")), address)


class Exemptions(unittest.TestCase):
    def test_the_two_pattern_files_are_exempt_from_the_key_rule_only(self):
        finding = private.scan.Finding("", 1, "text shaped like a key")
        # Spelt at run time for the same reason as the home path above: written out, it would be a
        # finding in this file, and this file is not one of the two that may hold one.
        line = "gh" + "p_" + "0123456789" * 4
        for name in private.KEY_SHAPE_FILES:
            with self.subTest(name=name):
                self.assertFalse(private.keep(finding, line, name))
        self.assertTrue(private.keep(finding, line, "server/src/mail.rs"))

    def test_nothing_is_exempt_from_the_home_rule(self):
        # Every file is read, including the two that hold the patterns: before this, those two were
        # the only files in the repository where a leaked home path could not be caught.
        self.assertFalse(hasattr(private, "ALLOWED_FILES"))
        for name in private.KEY_SHAPE_FILES:
            with self.subTest(name=name):
                self.assertTrue(private.home_folders(A_HOME_PATH, name, ()))


class Floors(unittest.TestCase):
    def test_a_scan_of_one_file_fails_every_floor(self):
        held = private.tracked
        private.tracked = lambda: ["README.md"]
        try:
            code, output = run_main()
        finally:
            private.tracked = held
        self.assertEqual(code, 1, output)
        for folder in private.LEAST:
            self.assertIn(f"{folder}: read", output)
            self.assertIn("this scan is not reading the repository", output)

    def test_the_whole_repository_passes_and_says_what_it_proved(self):
        code, output = run_main()
        self.assertEqual(code, 0, output)
        self.assertIn("home folders, credential shapes", output)
        if not os.environ.get(private.scan.REFUSED_FILE):
            # A rule that cannot have run may not be claimed to have run.
            self.assertIn("NOT file names, private words or capture tags", output)
            self.assertNotIn("refusing home folders, credential shapes, file names", output)

    def test_a_binary_this_repository_tracks_is_never_read_as_text(self):
        for suffix in (".jar", ".ttf", ".woff2", ".wasm"):
            with self.subTest(suffix=suffix):
                self.assertIn(
                    suffix, private.MORE_BINARY_SUFFIXES | private.scan.BINARY_SUFFIXES
                )


if __name__ == "__main__":
    unittest.main()
