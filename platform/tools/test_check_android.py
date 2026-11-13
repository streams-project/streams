#!/usr/bin/env python3
"""Tests for tools/check_android.py: the skip must be loud, and a red report must be red.

Two defects this holds shut. The first is the one it was written for: before 2026-11-13 `./check`
ran no Gradle, so 219 Android unit tests were invisible to it. The second is the one the fix could
introduce: a machine with no JDK or no Android SDK must still run `./check` to the end, and must
not be allowed to pass over the Android side without saying so.

Run from anywhere with: python3 tools/test_check_android.py
"""

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
SCRIPT = TOOLS / "check_android.py"

spec = importlib.util.spec_from_file_location("check_android", SCRIPT)
android = importlib.util.module_from_spec(spec)
spec.loader.exec_module(android)

GREEN = """<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="streams.tv.PageTest" tests="2" failures="0">
  <testcase name="rows" classname="streams.tv.PageTest" time="0.01"/>
  <testcase name="focus" classname="streams.tv.PageTest" time="0.01"/>
</testsuite>
"""

RED = """<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="streams.tv.PageTest" tests="2" failures="1">
  <testcase name="rows" classname="streams.tv.PageTest" time="0.01"/>
  <testcase name="focus" classname="streams.tv.PageTest" time="0.01">
    <failure message="expected 6 rows" type="java.lang.AssertionError">at PageTest.kt:40</failure>
  </testcase>
</testsuite>
"""


def run_without(tools, extra=None):
    """The script with the named tools taken away from the environment.

    JBR_HOME is pointed at an empty folder and not merely unset, the way test_check_types.py points
    JBR_BIN: Android Studio's JDK is where this looks last, and on a machine that has Android
    Studio the no-JDK case would otherwise quietly become the JDK case and prove nothing.
    """
    empty = tempfile.mkdtemp(prefix="no-android-tools-")
    environment = {
        name: value for name, value in os.environ.items() if name not in tools
    }
    environment.update(extra or {})
    done = subprocess.run(
        [sys.executable, str(SCRIPT)], capture_output=True, text=True, env=environment
    )
    return done.returncode, done.stdout + done.stderr, empty


class NoTools(unittest.TestCase):
    def test_a_machine_with_no_jdk_still_passes_and_says_what_it_skipped(self):
        empty = tempfile.mkdtemp(prefix="no-jdk-")
        code, output, _ = run_without(
            ("JAVA_HOME", "PATH", "JBR_HOME"), {"PATH": empty, "JBR_HOME": empty}
        )
        self.assertEqual(code, 0, output)
        self.assertIn("android: NOT RUN", output)
        self.assertIn("no java that runs", output)
        self.assertIn("were not run on this machine", output)

    def test_a_machine_with_no_sdk_still_passes_and_says_what_it_skipped(self):
        empty = tempfile.mkdtemp(prefix="no-sdk-")
        code, output, _ = run_without(
            ("ANDROID_HOME", "ANDROID_SDK_ROOT", "HOME"), {"HOME": empty}
        )
        self.assertEqual(code, 0, output)
        self.assertIn("android: NOT RUN", output)
        self.assertIn("no Android SDK", output)

    def test_the_skip_names_every_module_it_did_not_test(self):
        empty = tempfile.mkdtemp(prefix="no-jdk-")
        _, output, _ = run_without(
            ("JAVA_HOME", "PATH", "JBR_HOME"), {"PATH": empty, "JBR_HOME": empty}
        )
        for module in android.surfaces():
            self.assertIn(module, output)


class Probes(unittest.TestCase):
    def test_a_java_that_does_not_run_is_not_believed(self):
        """macOS ships /usr/bin/java as a stub that exits 1 with no JDK installed.

        It exists and it is executable, so a probe that only looked would hand Gradle a JDK that is
        not one. Measured on this machine on 2026-11-13: `/usr/bin/java -version` exits 1 with
        "Unable to locate a Java Runtime".
        """
        folder = Path(tempfile.mkdtemp(prefix="stub-jdk-"))
        stub = folder / "java"
        stub.write_text("#!/bin/sh\necho 'Unable to locate a Java Runtime' >&2\nexit 1\n")
        stub.chmod(0o755)
        self.assertFalse(android.answers(stub))

        working = folder / "java-that-runs"
        working.write_text("#!/bin/sh\necho 'openjdk 25.0.3'\nexit 0\n")
        working.chmod(0o755)
        self.assertTrue(android.answers(working))

    def test_an_empty_sdk_folder_is_not_an_sdk(self):
        """Android Studio opened once and never used leaves ~/Library/Android/sdk with no
        platforms in it. Handing that to Gradle fails deep inside the Android plugin, in a message
        that reads nothing like "this machine has no SDK"."""
        folder = Path(tempfile.mkdtemp(prefix="hollow-sdk-"))
        was = {name: os.environ.pop(name, None) for name in ("ANDROID_HOME", "ANDROID_SDK_ROOT")}
        guesses = android.SDK_GUESSES
        try:
            android.SDK_GUESSES = (folder,)
            sdk, why = android.sdk_home()
            self.assertIsNone(sdk)
            self.assertIn("no Android SDK", why)
            (folder / "platforms").mkdir()
            sdk, why = android.sdk_home()
            self.assertEqual(sdk, str(folder))
            self.assertIsNone(why)
        finally:
            android.SDK_GUESSES = guesses
            for name, value in was.items():
                if value is not None:
                    os.environ[name] = value


class Reports(unittest.TestCase):
    def test_a_failure_in_the_xml_is_named(self):
        folder = Path(tempfile.mkdtemp(prefix="red-reports-"))
        (folder / "TEST-streams.tv.PageTest.xml").write_text(RED)
        cases, bad = android.read_reports(folder)
        self.assertEqual(cases, 2)
        self.assertEqual(bad, ["streams.tv.PageTest.focus"])

    def test_a_green_report_counts_its_cases(self):
        folder = Path(tempfile.mkdtemp(prefix="green-reports-"))
        (folder / "TEST-streams.tv.PageTest.xml").write_text(GREEN)
        cases, bad = android.read_reports(folder)
        self.assertEqual(cases, 2)
        self.assertEqual(bad, [])

    def test_a_module_that_wrote_nothing_counts_nothing(self):
        """Which main() turns into a failure: Gradle happy and no report written is a step that
        proved nothing, and a step that proved nothing is not a pass."""
        folder = Path(tempfile.mkdtemp(prefix="no-reports-"))
        self.assertEqual(android.read_reports(folder), (0, []))


class Modules(unittest.TestCase):
    def test_every_module_it_will_test_is_a_folder_that_is_there(self):
        modules = android.surfaces()
        self.assertTrue(modules)
        for module in modules:
            self.assertTrue((android.ANDROID / "surfaces" / module).is_dir(), module)

    def test_every_surface_folder_with_tests_is_a_module_it_will_test(self):
        """A surface whose tests this step does not name is a surface CI cannot see."""
        modules = set(android.surfaces())
        for folder in sorted((android.ANDROID / "surfaces").iterdir()):
            if (folder / "src" / "test").is_dir():
                self.assertIn(folder.name, modules)


if __name__ == "__main__":
    unittest.main()
