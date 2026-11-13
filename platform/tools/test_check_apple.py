#!/usr/bin/env python3
"""Tests for tools/check_apple.py: the skip must be loud, and a red run must be red.

Three defects this holds shut. The first is the one it was written for: before 2026-11-13 `./check`
ran no xcodebuild, so 62 Apple unit tests were invisible to it. The second is the one the fix could
introduce: a machine with no Xcode - CI is one, it runs on ubuntu - must still run `./check` to the
end, and must not pass over the Apple side without saying so. The third is the destination, which
is where the Apple side is easiest to get quietly wrong: a written-down simulator id is a path to
one disk, and a device name is not an identity - two televisions here share their first 28
characters - so the destination must be an id this step resolves from simctl on every run.

Run from anywhere with: python3 tools/test_check_apple.py
"""

import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
SCRIPT = TOOLS / "check_apple.py"

spec = importlib.util.spec_from_file_location("check_apple", SCRIPT)
apple = importlib.util.module_from_spec(spec)
spec.loader.exec_module(apple)

# What `xcrun simctl list devices available --json` answers, cut down to what this reads. Two tvOS
# runtimes, so the newest must win; a shut-down device ahead of a booted one in the list, so the
# booted one must win anyway; an unavailable device and a watchOS runtime, which are not answers.
SIMCTL = json.dumps(
    {
        "devices": {
            "com.apple.CoreSimulator.SimRuntime.tvOS-26-5": [
                {
                    "udid": "F6EF99DA-OLD-TV",
                    "name": "Apple TV 4K (3rd generation)",
                    "state": "Shutdown",
                    "isAvailable": True,
                },
                {
                    "udid": "664DD7AC-BOOTED-TV",
                    "name": "Apple TV 4K (3rd generation) (at 1080p)",
                    "state": "Booted",
                    "isAvailable": True,
                },
            ],
            "com.apple.CoreSimulator.SimRuntime.tvOS-18-0": [
                {
                    "udid": "OLDER-RUNTIME-TV",
                    "name": "Apple TV",
                    "state": "Booted",
                    "isAvailable": True,
                }
            ],
            "com.apple.CoreSimulator.SimRuntime.iOS-26-5": [
                {
                    "udid": "BROKEN-PHONE",
                    "name": "iPhone 17",
                    "state": "Shutdown",
                    "isAvailable": False,
                },
                {
                    "udid": "5F7F6A73-PHONE",
                    "name": "iPhone 17 Pro",
                    "state": "Shutdown",
                    "isAvailable": True,
                },
            ],
            "com.apple.CoreSimulator.SimRuntime.watchOS-26-5": [
                {
                    "udid": "A-WATCH",
                    "name": "Apple Watch",
                    "state": "Shutdown",
                    "isAvailable": True,
                }
            ],
        }
    }
)

# What `xcresulttool get test-results summary --format json` answers, cut to what this reads.
GREEN = json.dumps(
    {"result": "Passed", "passedTests": 30, "failedTests": 0, "testFailures": []}
)

RED = json.dumps(
    {
        "result": "Failed",
        "passedTests": 29,
        "failedTests": 1,
        "testFailures": [
            {
                "testName": "testTheBytesDecodeAtTheirOwnSize()",
                "targetName": "PhoneTests",
                "failureText": "XCTAssertEqual failed: the bytes did not decode at their own size",
            }
        ],
    }
)

# A run that failed before any test ran - a compile error is the usual way - writes a bundle with
# nothing in it. That is the case main() must call a failure and not a pass.
EMPTY = json.dumps({"result": "Failed", "passedTests": 0, "failedTests": 0, "testFailures": []})


def run_without(tools, extra=None):
    """The script with the named tools taken away from the environment."""
    environment = {name: value for name, value in os.environ.items() if name not in tools}
    environment.update(extra or {})
    done = subprocess.run(
        [sys.executable, str(SCRIPT)], capture_output=True, text=True, env=environment
    )
    return done.returncode, done.stdout + done.stderr


class NoXcode(unittest.TestCase):
    def test_a_machine_with_no_xcode_still_passes_and_says_what_it_skipped(self):
        """CI is this machine and always will be: the workflow runs on ubuntu-24.04."""
        empty = tempfile.mkdtemp(prefix="no-xcode-")
        code, output = run_without(("XCODEBUILD", "PATH"), {"PATH": empty})
        self.assertEqual(code, 0, output)
        self.assertIn("apple: NOT RUN", output)
        self.assertIn("no xcodebuild", output)
        self.assertIn("were not run on this machine", output)

    def test_the_skip_names_every_target_it_did_not_run(self):
        empty = tempfile.mkdtemp(prefix="no-xcode-")
        _, output = run_without(("XCODEBUILD", "PATH"), {"PATH": empty})
        for target, _ in apple.targets():
            self.assertIn(target, output)

    def test_an_xcodebuild_that_does_not_run_is_not_believed(self):
        """macOS ships /usr/bin/xcodebuild whether Xcode is installed or not.

        It is a shim that forwards to whatever `xcode-select` points at: it exists, it is
        executable, and on a machine with only the Command Line Tools it exits 1 asking for Xcode.
        A probe that only looked would run the Apple step there and call what came back a
        regression. The stub below is that machine, written out, because this one has Xcode and
        cannot be asked.
        """
        folder = Path(tempfile.mkdtemp(prefix="stub-xcode-"))
        stub = folder / "xcodebuild"
        stub.write_text("#!/bin/sh\necho 'requires Xcode' >&2\nexit 1\n")
        stub.chmod(0o755)
        self.assertFalse(apple.answers(str(stub)))

        working = folder / "xcodebuild-that-runs"
        working.write_text("#!/bin/sh\necho 'Xcode 26.5'\nexit 0\n")
        working.chmod(0o755)
        self.assertTrue(apple.answers(str(working)))

    def test_a_clone_that_has_not_built_the_core_is_a_loud_skip_and_not_a_red_run(self):
        """Contract.swift and core.h are copies of tracked files that build-core.sh makes, and
        neither is in git. A clone without them is not broken, so it must not be a failure - but
        it must not be silence either."""
        folder = Path(tempfile.mkdtemp(prefix="unbuilt-clone-"))
        was = apple.GENERATED
        try:
            apple.GENERATED = ((folder / "Contract.swift", "a tracked file somewhere"),)
            missing = apple.missing_generated()
            self.assertEqual(len(missing), 1)
            self.assertIn("has not been generated", missing[0])
            self.assertIn("build-core.sh", missing[0])
            (folder / "Contract.swift").write_text("// generated\n")
            self.assertEqual(apple.missing_generated(), [])
        finally:
            apple.GENERATED = was


class Destinations(unittest.TestCase):
    def test_the_newest_runtime_wins_and_a_booted_simulator_wins_inside_it(self):
        found = apple.devices(SIMCTL)
        udid, where = apple.destination(found, "tvos")
        self.assertEqual(udid, "664DD7AC-BOOTED-TV")
        self.assertIn("tvOS 26.5", where)

    def test_a_simulator_that_is_not_available_is_not_a_destination(self):
        found = apple.devices(SIMCTL)
        udid, where = apple.destination(found, "ios")
        self.assertEqual(udid, "5F7F6A73-PHONE")
        self.assertIn("iPhone 17 Pro", where)

    def test_a_platform_with_no_simulator_is_named_and_not_guessed(self):
        udid, why = apple.destination({}, "tvos")
        self.assertIsNone(udid)
        self.assertIn("tvOS", why)
        self.assertIn("no", why)

    def test_the_destination_is_an_id_and_never_a_name(self):
        """A name is not an identity. This machine lists two televisions whose names agree for
        their first 28 characters, and on 2026-11-13 a `platform=tvOS Simulator,name=...`
        destination was refused with "no available devices matched" while simctl listed that
        device available - then resolved an hour later from the same shell. An id resolved from
        simctl at run time has never failed, and resolved rather than written down because ids
        differ per machine."""
        argv = apple.command("xcodebuild", "TelevisionTests", "AN-ID", Path("/tmp/x.xcresult"))
        self.assertIn("id=AN-ID", argv)
        for part in argv:
            self.assertNotIn("name=", part)
        self.assertIn("-only-testing:TelevisionTests", argv)

    def test_no_simulator_id_is_written_down_in_the_script(self):
        """A pinned id is a path to one disk. Every id this step uses comes from simctl."""
        udid = re.compile(r"\b[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}\b")
        self.assertEqual(udid.findall(SCRIPT.read_text()), [])


class Results(unittest.TestCase):
    def test_a_failure_in_the_summary_is_named(self):
        cases, bad = apple.read_summary(RED)
        self.assertEqual(cases, 30)
        self.assertEqual(bad, ["PhoneTests.testTheBytesDecodeAtTheirOwnSize()"])

    def test_a_green_summary_counts_its_cases(self):
        cases, bad = apple.read_summary(GREEN)
        self.assertEqual(cases, 30)
        self.assertEqual(bad, [])

    def test_a_failed_count_with_no_named_failure_is_still_a_failure(self):
        """Whatever a future xcresulttool calls the list, a non-zero failed count must not read as
        a pass: this step's whole value is that it cannot go quiet."""
        cases, bad = apple.read_summary(
            json.dumps({"passedTests": 29, "failedTests": 1, "testFailures": []})
        )
        self.assertEqual(cases, 30)
        self.assertTrue(bad)

    def test_a_run_that_wrote_nothing_counts_nothing(self):
        """Which main() turns into a failure. A compile error writes a bundle with no test in it,
        and a step that proved nothing is not a pass."""
        self.assertEqual(apple.read_summary(EMPTY), (0, []))

    def test_a_bundle_that_is_not_there_is_named_and_not_believed(self):
        cases, bad = apple.results(Path(tempfile.mkdtemp(prefix="no-bundle-")) / "gone.xcresult")
        self.assertEqual(cases, 0)
        self.assertTrue(bad)

    def test_a_summary_that_is_not_json_is_named(self):
        cases, bad = apple.read_summary("xcresulttool: error: unknown subcommand")
        self.assertEqual(cases, 0)
        self.assertTrue(bad)


class Targets(unittest.TestCase):
    def test_every_test_target_in_the_package_is_one_it_will_run(self):
        found = dict(apple.targets())
        self.assertEqual(found.get("PhoneTests"), "ios")
        self.assertEqual(found.get("TelevisionTests"), "tvos")

    def test_every_folder_of_xctest_cases_is_a_target_it_will_run(self):
        """A test target this step does not name is a target CI cannot see, which is the whole
        defect. Tests/Heard and Tests/Stand hold no XCTestCase: they are support targets."""
        known = {name for name, _ in apple.targets()}
        for folder in sorted((apple.APPLE / "Tests").iterdir()):
            if not folder.is_dir():
                continue
            cases = any(
                re.search(r"class\s+\w+\s*:\s*XCTestCase", source.read_text())
                for source in folder.glob("*.swift")
            )
            if cases:
                self.assertIn(folder.name, known, folder.name)

    def test_a_comment_inside_a_declaration_does_not_lose_the_target(self):
        """The defect that actually happened, on the afternoon this landed. The first version of
        this parser wanted `name:` and `dependencies:` with nothing but whitespace between them;
        somebody wrote a comment between those two lines and TelevisionTests silently stopped
        being tested. Nothing but this file noticed."""
        written = """
            .testTarget(
                name: "TelevisionTests",
                // `Adapters` is here for one thing: the real decode (not a stand-in).
                dependencies: ["Heard", "SurfacesTV"],
                path: "Tests/TelevisionTests"
            ),
            .testTarget(
                name: "PhoneTests",
                dependencies: ["Heard", "SurfacesPhone"],
                path: "Tests/PhoneTests"
            ),
        """
        self.assertEqual(
            apple.targets(written), [("PhoneTests", "ios"), ("TelevisionTests", "tvos")]
        )
        self.assertEqual(apple.declared(written), 2)

    def test_a_declaration_this_cannot_read_is_counted_anyway(self):
        """Which main() turns into a red run. The count and the read must agree, or this step
        cannot say what it would be leaving out - and leaving one out in silence is the defect."""
        written = """
            .testTarget(
                name: "PhoneTests",
                dependencies: ["Heard"],
            ),
            .testTarget(
                theName: "WrittenSomeOtherWay",
                dependencies: ["Heard"],
            ),
        """
        self.assertEqual(apple.declared(written), 2)
        self.assertEqual(apple.targets(written), [("PhoneTests", "ios")])

    def test_a_comment_with_a_parenthesis_in_it_is_not_a_declaration(self):
        """The blocks are found by counting parentheses, so a comment holding one that nothing
        opened would swallow the next declaration whole."""
        written = """
            .testTarget(
                name: "PhoneTests",
                // a box one pixel over a rung decodes at the next one :-) ((
                dependencies: ["Heard"]
            ),
        """
        self.assertEqual(apple.declared(written), 1)
        self.assertEqual(apple.targets(written), [("PhoneTests", "ios")])

    def test_a_double_slash_inside_a_string_is_not_a_comment(self):
        written = '.testTarget(name: "PhoneTests", dependencies: ["https://x"], path: "a")'
        self.assertEqual(apple.declared(written), 1)
        self.assertEqual(apple.targets(written), [("PhoneTests", "ios")])

    def test_the_scheme_is_the_one_the_package_defines(self):
        """Swift Package Manager names the all-targets scheme `<package>-Package`. A package
        renamed without this line changed fails inside xcodebuild, in a message about a scheme."""
        name = re.search(r'Package\(\s*name:\s*"([A-Za-z0-9_]+)"', apple.PACKAGE.read_text())
        self.assertIsNotNone(name)
        self.assertEqual(apple.SCHEME, f"{name.group(1)}-Package")


if __name__ == "__main__":
    unittest.main()
