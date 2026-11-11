#!/usr/bin/env python3
"""Tests for tools/check_types.py: a run that proved no language is a failure.

Before 2026-11-11 this script exited 0 on a machine with no TypeScript, no Swift and no Kotlin,
which is what CI was, so the generated clients were never compiled by anything and the step still
read as green.

Run from anywhere with: python3 tools/test_check_types.py
"""

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
SCRIPT = TOOLS / "check_types.py"

spec = importlib.util.spec_from_file_location("check_types", SCRIPT)
types = importlib.util.module_from_spec(spec)
spec.loader.exec_module(types)


def run_with_no_compiler():
    """The script with nothing on the PATH and none of its environment variables set."""
    empty = tempfile.mkdtemp(prefix="no-compilers-")
    environment = {
        name: value
        for name, value in os.environ.items()
        if name not in ("TSC", "SWIFTC", "KOTLINC", "JAVA", "PATH")
    }
    environment["PATH"] = empty
    done = subprocess.run(
        [sys.executable, str(SCRIPT)], capture_output=True, text=True, env=environment
    )
    return done.returncode, done.stdout + done.stderr


class Types(unittest.TestCase):
    def test_a_run_that_compiled_nothing_fails(self):
        code, output = run_with_no_compiler()
        self.assertEqual(code, 1, output)
        self.assertIn("0 of 3 languages compiled (none)", output)
        self.assertIn("nothing about the generated types was proved", output)

    def test_it_still_names_every_language_it_could_not_run(self):
        _, output = run_with_no_compiler()
        for language in ("typescript", "swift", "kotlin"):
            self.assertIn(f"{language}: NOT RUN", output)

    def test_every_fixture_the_type_system_cannot_refuse_is_a_real_fixture(self):
        refused = set(types.names("refused"))
        self.assertTrue(refused)
        stale = sorted(types.TS_CANNOT_SEE - refused)
        self.assertEqual(stale, [], "TS_CANNOT_SEE names fixtures that are not there")

    def test_the_two_ids_above_the_maximum_are_refused_fixtures(self):
        refused = set(types.names("refused"))
        self.assertIn("TitleKey.id_over_max.json", refused)
        self.assertIn("TitleKey.id_ten_digits.json", refused)


if __name__ == "__main__":
    unittest.main()
