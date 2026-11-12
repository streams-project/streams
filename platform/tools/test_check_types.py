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
    """The script with nothing on the PATH and none of its environment variables set.

    JBR_BIN is pointed at the empty folder too, not just unset: Android Studio's bundled JDK is
    where `jdk` looks last, and on a machine that has Android Studio the Java arm would otherwise
    run and this would no longer be the no-compiler case it is named for.
    """
    empty = tempfile.mkdtemp(prefix="no-compilers-")
    environment = {
        name: value
        for name, value in os.environ.items()
        if name not in ("TSC", "SWIFTC", "KOTLINC", "JAVAC", "JAVA", "JBR_BIN", "PATH")
    }
    environment["PATH"] = empty
    environment["JBR_BIN"] = empty
    done = subprocess.run(
        [sys.executable, str(SCRIPT)], capture_output=True, text=True, env=environment
    )
    return done.returncode, done.stdout + done.stderr


class Types(unittest.TestCase):
    def test_a_run_that_compiled_nothing_fails(self):
        code, output = run_with_no_compiler()
        self.assertEqual(code, 1, output)
        self.assertIn("0 of 4 languages compiled (none)", output)
        self.assertIn("nothing about the generated types was proved", output)

    def test_it_still_names_every_language_it_could_not_run(self):
        _, output = run_with_no_compiler()
        for language in ("typescript", "java", "swift", "kotlin"):
            self.assertIn(f"{language}: NOT RUN", output)

    def test_a_jdk_tool_that_does_not_run_is_not_believed(self):
        """macOS ships /usr/bin/javac and /usr/bin/java as stubs.

        They exist, they satisfy `shutil.which`, and with no JDK installed they exit 1 saying
        "Unable to locate a Java Runtime". Measured on the machine this landed on, 2026-11-12.
        Believing `which` there names Java as a language this script proved and then fails the run,
        so `jdk` runs a tool before it uses it. This test is that rule, with a stub of its own.
        """
        folder = tempfile.mkdtemp(prefix="stub-jdk-")
        stub = Path(folder) / "javac"
        stub.write_text("#!/bin/sh\necho 'Unable to locate a Java Runtime' >&2\nexit 1\n")
        stub.chmod(0o755)
        self.assertFalse(types.answers(str(stub)))

        working = Path(folder) / "javac-that-runs"
        working.write_text("#!/bin/sh\necho 'javac 25.0.3'\nexit 0\n")
        working.chmod(0o755)
        self.assertTrue(types.answers(str(working)))

        was_path, was_jbr = os.environ.get("PATH"), types.JBR
        named = os.environ.pop("JAVAC", None)
        try:
            os.environ["PATH"] = folder
            types.JBR = tempfile.mkdtemp(prefix="no-jbr-")
            tool, why = types.jdk("JAVAC", "javac")
            self.assertIsNone(tool)
            self.assertIn("javac", why)
            tool, why = types.jdk("JAVAC", "javac-that-runs")
            self.assertEqual(tool, str(working))
            self.assertIsNone(why)
        finally:
            types.JBR = was_jbr
            if was_path is None:
                os.environ.pop("PATH", None)
            else:
                os.environ["PATH"] = was_path
            if named is not None:
                os.environ["JAVAC"] = named

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
