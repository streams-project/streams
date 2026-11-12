#!/usr/bin/env python3
"""Tests for tools/check_tracked.py, against the three files that were untracked on 2026-11-11.

Nothing here writes to git. The planted cases are names handed to the rules, not files made on
disk, so the test says the same thing on a dirty tree and a clean one.

Run from anywhere with: python3 tools/test_check_tracked.py
"""

import importlib.util
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location("check_tracked", TOOLS / "check_tracked.py")
check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check)

# The three that broke a fresh clone. They are tracked now; the rules are asked about the names.
BROKE_A_CLONE = (
    "launcher/src/data.rs",
    "server/crates/server-accounts/tests/conformance/mod.rs",
    "apps/android/app/src/main/res/xml/network_security_config.xml",
)


class Tracked(unittest.TestCase):
    def test_a_file_whose_name_says_nothing_is_known_by_its_folder(self):
        self.assertEqual(check.token("server/x/tests/conformance/mod.rs"), "conformance")
        self.assertEqual(check.token("launcher/src/data.rs"), "data")
        self.assertEqual(
            check.token("apps/android/app/src/main/res/xml/network_security_config.xml"),
            "network_security_config",
        )

    def test_every_file_that_broke_a_clone_is_declared_by_tracked_code(self):
        for name in BROKE_A_CLONE:
            with self.subTest(name=name):
                where = check.references(name)
                self.assertTrue(where, f"{name}: nothing declares it, so the rule would miss it")

    def test_a_scratch_file_nobody_declares_is_not_a_failure(self):
        # The probe's stem is assembled here rather than written out whole. `references` is a
        # `git grep` over tracked files and this file is tracked, so a stem written as one literal
        # finds this very line and the test asserting that nothing declares the file becomes the
        # thing that declares it. That is how this test failed from the day it was added: the
        # failure was hidden behind an earlier one in `./check` and only showed when that was fixed.
        stem = "unreferenced" + "_scratch_" + "zz"
        self.assertEqual(check.references(f"platform/tools/{stem}.py"), [])

    def test_the_paths_check_names_must_be_held_by_git(self):
        found, named = check.named_in_check(set())
        self.assertGreaterEqual(named, check.LEAST_NAMED)
        self.assertTrue(found)
        self.assertTrue(all("a clone cannot run ./check" in line for line in found))

    def test_a_path_check_names_that_is_not_a_file_here_is_not_claimed(self):
        # `check` also names things this repository does not hold, such as a target triple or a
        # cargo profile. The rule is about files, so a name that is no file is passed over.
        found, _ = check.named_in_check(set())
        for line in found:
            name = line.split("runs ", 1)[1].split(",", 1)[0]
            self.assertTrue(
                (check.ROOT / name).exists()
                or any((check.ROOT / top / name).exists() for top in ("platform", "server", "data")),
                line,
            )

    def test_untracked_is_what_git_says_and_nothing_ignored(self):
        for name in check.untracked():
            with self.subTest(name=name):
                self.assertNotIn(name, check.tracked())


if __name__ == "__main__":
    unittest.main()
