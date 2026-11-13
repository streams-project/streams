#!/usr/bin/env python3
"""Run the Apple family's unit tests on simulators, or say out loud that they were not run.

Until 2026-11-13 `./check` ran no xcodebuild. The Apple surfaces held 62 passing unit tests - 34
in PhoneTests, 28 in TelevisionTests, counted at 13:20 on 2026-11-13 and rising that afternoon -
that CI could not see, and `apps/apple/README.md` said so.
Worse than invisible, they were believed unrunnable here: `swift build` fails with "no such module
'UIKit'", and that error was read as "this machine cannot build them". It was wrong. `swift build`
has no simulator SDK unless it is handed one; xcodebuild picks one from the destination, and both
simulator SDKs have been installed all along. Measured on this machine on 2026-11-13: 34 phone
tests and 28 television tests, 0 failures.

The difficulty is the same one check_android.py has: `./check` must stay runnable where there is no
Xcode - CI is one such machine, because it runs on ubuntu-24.04 and always will. A step that cannot
run everywhere and exits 1 stops every push; a step that skips in silence is the defect class this
repository exists to prevent. So this follows check_types.py and check_android.py: it probes, runs
a candidate before believing it, and when it cannot run it prints what it did NOT check, by name,
on every run.

The destination is resolved at run time and never written down. A simulator id differs per machine,
so pinning one here would be a path to this disk, and a device name is not an identity either:
this machine lists two televisions whose names agree for their first 28 characters, and which of
them a `name=` destination means is xcodebuild's business and not something to rely on. One such
destination was refused here earlier today with "no available devices matched" while simctl listed
that device available; the same destination resolved an hour later, so what decides it is not
understood. `id=<udid>` has never failed, so that is what this uses: `xcrun simctl list devices
available --json` is asked on every run and the newest runtime wins. A simulator that is already
booted is preferred over one that is not: booting is most of the wall clock, and a developer with
one open should not pay it twice.

What it runs, and what it does not. The test targets only, each on the one platform it belongs to.
Each run still compiles the whole package for its platform - `-only-testing` chooses what runs,
not what is built - so a phone-only compile error turns the television run red as well, and the
second run is nearly as long as the first. The two apps under App/ and PhoneApp/ are not built, the
UI tests are not here, and nothing is archived or signed. The package has no remote dependencies,
so unlike the Gradle step this one never wants the network.

Where the tools are looked for, each believed only once it has answered:

  XCODEBUILD         an xcodebuild, taken as given when it is set, because setting it is a
                     deliberate act
  xcodebuild on the  probed by running `-version`: /usr/bin/xcodebuild is a macOS stub that exits
  PATH               1 with "requires Xcode, but active developer directory is a CommandLineTools
                     instance", and satisfies `shutil.which` anyway. The same trap as /usr/bin/java
                     in check_android.py, in a different coat.

  A simulator per platform, from `xcrun simctl list devices available --json`. A platform with no
  available simulator is named as not run; the other half still runs.

Two generated files must be in place before any of this compiles, and neither is in git:
Sources/Contract/Contract.swift and Sources/CoreBridge/include/core.h. Both are copies of tracked
files that `apps/apple/build-core.sh` makes. A clone that has not run it is not a broken clone and
must not be a red run, so a missing one is a loud not-run like a missing simulator.

The exit code is xcodebuild's, and 1 when xcodebuild passed but a target wrote no test result: a
run that proved nothing is a failure, not a pass. A machine without Xcode exits 0 and says so.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APPLE = ROOT / "apps" / "apple"
PACKAGE = APPLE / "Package.swift"

# The one scheme that carries every target. `xcodebuild -list` also offers a scheme per library,
# but none of them carries a test target, so none of them can run a test.
SCHEME = "AppleFamily-Package"

# The two files the package needs and git has not got, each with the tracked file it is copied
# from, so the message can say what is missing without saying "run a script and hope".
GENERATED = (
    (
        APPLE / "Sources" / "Contract" / "Contract.swift",
        "platform/contract/generated/types/Contract.swift",
    ),
    (
        APPLE / "Sources" / "CoreBridge" / "include" / "core.h",
        "platform/bridges/c/core.h",
    ),
)

# A test target is read from the build rather than written here: a third test target must not be
# able to arrive without these tests arriving with it. The declaration is found by counting
# parentheses rather than by one expression over the whole call, because the first version of this
# demanded `name:` and `dependencies:` with nothing but whitespace between them, and the afternoon
# it landed somebody wrote a comment between those two lines. A target silently stopped being
# tested and only test_check_apple.py noticed. Hence `declared` below, which counts what is there
# and makes a disagreement a red run.
A_TEST_TARGET = re.compile(r"\.testTarget\(")
A_NAME = re.compile(r"name:\s*\"([A-Za-z0-9_]+)\"")
A_DEPENDENCIES = re.compile(r"dependencies:\s*\[([^\]]*)\]")

# Which simulator a test target wants. A target that reaches for the television surface is a
# television target; everything else builds and runs on the phone. The rule is a rule and not a
# table of two names for the same reason the targets are read and not listed: a new target gets an
# answer, and `main` says which platform it chose on every run.
TV_SURFACE = "SurfacesTV"

# The runtime identifiers simctl answers with, for example
# `com.apple.CoreSimulator.SimRuntime.tvOS-26-5`.
A_RUNTIME = re.compile(r"SimRuntime\.([A-Za-z]+)-([0-9][0-9-]*)$")

# What each platform is called in a runtime identifier, and what to call it in a message.
PLATFORMS = {"ios": "iOS", "tvos": "tvOS"}


def answers(program: str) -> bool:
    """Whether an xcodebuild really runs, asked by running it.

    macOS ships /usr/bin/xcodebuild whether Xcode is installed or not: it is a shim that forwards
    to whatever `xcode-select` points at, so it exists and `shutil.which` finds it on a machine
    that has only the Command Line Tools, where it exits 1 with a message about requiring Xcode.
    That half is not measured here - this machine has Xcode, so the shim works - but it is the same
    trap /usr/bin/java sets in check_android.py, where it was measured, and the probe is one
    process either way. Believing the PATH without it turns a machine with no Xcode into a red run
    whose message reads nothing like "this machine has no Xcode".
    """
    try:
        done = subprocess.run(
            [program, "-version"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )
    except OSError:
        return False
    return done.returncode == 0


def xcodebuild() -> tuple[str | None, str | None]:
    """An xcodebuild to run the tests with, and what is missing when there is none."""
    named = os.environ.get("XCODEBUILD")
    if named:
        return named, None
    found = shutil.which("xcodebuild")
    if found and answers(found):
        return found, None
    return None, (
        "no xcodebuild that runs on the PATH and XCODEBUILD is not set"
        + ("" if sys.platform == "darwin" else ", and this is not a mac")
    )


def version(program: str) -> str:
    """What this xcodebuild calls itself, for the last line. Never used as a probe.

    It asks the xcodebuild that ran the tests and not whatever is on the PATH, because XCODEBUILD
    may name a second Xcode and a line that credits the wrong one is worse than no line.
    """
    try:
        done = subprocess.run(
            [program, "-version"], capture_output=True, text=True, check=False
        )
    except OSError:
        return "an unknown Xcode"
    first = done.stdout.strip().splitlines()
    return first[0] if first else "an unknown Xcode"


def missing_generated() -> list[str]:
    """The generated files that are not there, said as what is missing and where it comes from."""
    return [
        f"{said(path)} has not been generated (it is a copy of {source}, which "
        "apps/apple/build-core.sh makes)"
        for path, source in GENERATED
        if not path.is_file()
    ]


def said(path: Path) -> str:
    """A path as this repository names it, or in full when it is somewhere else."""
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def bare(text: str) -> str:
    """The same Swift with its comments taken out, so parentheses can be counted in it.

    A comment is where the surprises live: `// one thing: (above)` holds a parenthesis that no
    declaration opened, and a `"//"` inside a string is not a comment at all. Both are handled
    here rather than by an expression that would get one of them wrong quietly.
    """
    out, index, end, quoted = [], 0, len(text), False
    while index < end:
        here = text[index]
        if quoted:
            out.append(here)
            if here == "\\" and index + 1 < end:
                out.append(text[index + 1])
                index += 2
                continue
            quoted = here != '"'
            index += 1
        elif here == '"':
            quoted, index = True, index + 1
            out.append(here)
        elif text.startswith("//", index):
            while index < end and text[index] != "\n":
                index += 1
        elif text.startswith("/*", index):
            shut = text.find("*/", index + 2)
            index = end if shut < 0 else shut + 2
        else:
            out.append(here)
            index += 1
    return "".join(out)


def declared(text: str) -> int:
    """How many test targets the package declares, however they are written inside."""
    return len(A_TEST_TARGET.findall(bare(text)))


def targets(text: str | None = None) -> list[tuple[str, str]]:
    """Every test target in the package that can be read, with the platform it belongs on.

    `main` compares the length of this against `declared`, so a target this cannot read stops the
    run instead of quietly not being tested.
    """
    swift = bare(PACKAGE.read_text() if text is None else text)
    found = []
    for opened in (match.end() for match in A_TEST_TARGET.finditer(swift)):
        depth, index, end = 1, opened, len(swift)
        while index < end and depth:
            if swift[index] == "(":
                depth += 1
            elif swift[index] == ")":
                depth -= 1
            index += 1
        block = swift[opened : index - 1]
        name = A_NAME.search(block)
        dependencies = A_DEPENDENCIES.search(block)
        if name and dependencies:
            platform = "tvos" if TV_SURFACE in dependencies.group(1) else "ios"
            found.append((name.group(1), platform))
    return sorted(found)


def devices(out: str) -> dict[str, list[tuple[tuple[int, ...], str, dict]]]:
    """Simulators that can be booted, by platform, newest runtime first.

    Reads `simctl list devices available --json` rather than its human output: the television
    devices differ only by a parenthesis in their names, and picking one by eye off a text list is
    how a destination ends up matching nothing.
    """
    by_platform: dict[str, list[tuple[tuple[int, ...], str, dict]]] = {}
    for runtime, listed in json.loads(out).get("devices", {}).items():
        found = A_RUNTIME.search(runtime)
        if not found:
            continue
        platform = found.group(1).lower()
        if platform not in PLATFORMS:
            continue
        numbers = tuple(int(part) for part in found.group(2).split("-"))
        for device in listed:
            if device.get("isAvailable") and device.get("udid"):
                by_platform.setdefault(platform, []).append((numbers, runtime, device))
    for listed in by_platform.values():
        # Newest runtime first, and a simulator that is already booted ahead of one that is not:
        # booting is most of the wall clock of this step.
        listed.sort(key=lambda row: (row[0], row[2].get("state") == "Booted"), reverse=True)
    return by_platform


def simulators() -> tuple[dict[str, list], str | None]:
    """Every usable simulator on this machine, and what is missing when simctl cannot be asked."""
    try:
        done = subprocess.run(
            ["xcrun", "simctl", "list", "devices", "available", "--json"],
            capture_output=True,
            text=True,
        )
    except OSError as error:
        return {}, f"simctl could not be run ({error})"
    if done.returncode != 0:
        return {}, "simctl could not list the simulators on this machine"
    try:
        return devices(done.stdout), None
    except (json.JSONDecodeError, ValueError) as error:
        return {}, f"simctl answered something that is not device JSON ({error})"


def destination(found: dict[str, list], platform: str) -> tuple[str | None, str]:
    """The id of the simulator to run a platform's tests on, and what to call it in a message,
    or no id and what is missing when that platform has no simulator here."""
    listed = found.get(platform)
    if not listed:
        return None, f"no {PLATFORMS[platform]} simulator is installed"
    _, runtime, device = listed[0]
    release = runtime.rsplit(".", 1)[-1].split("-", 1)[-1].replace("-", ".")
    return device["udid"], f"{device['name']} ({PLATFORMS[platform]} {release} simulator)"


def command(program: str, target: str, udid: str, bundle: Path) -> list[str]:
    """The xcodebuild to run, as a list, so a test can read it without running Xcode.

    `-quiet` because the useful output is the bundle, not three thousand lines of build log; a
    failure still prints, and the count and the failing names are read from the bundle below.
    """
    return [
        program,
        "test",
        "-scheme",
        SCHEME,
        "-destination",
        f"id={udid}",
        f"-only-testing:{target}",
        "-resultBundlePath",
        str(bundle),
        "-quiet",
    ]


def run(program: str, target: str, udid: str, bundle: Path) -> int:
    """One test target, on one simulator, with its result bundle written where this can read it."""
    return subprocess.run(command(program, target, udid, bundle), cwd=APPLE).returncode


def results(bundle: Path) -> tuple[int, list[str]]:
    """How many test cases that run wrote, and the names that did not pass.

    Read from the result bundle xcodebuild writes, not from its console output, for the reason
    check_android.py reads Gradle's XML: a run that did nothing prints no counts, and the count is
    what says this step proved something.
    """
    try:
        done = subprocess.run(
            [
                "xcrun",
                "xcresulttool",
                "get",
                "test-results",
                "summary",
                "--path",
                str(bundle),
                "--format",
                "json",
            ],
            capture_output=True,
            text=True,
        )
    except OSError as error:
        return 0, [f"the result bundle could not be read ({error})"]
    if done.returncode != 0:
        return 0, [f"{bundle.name} is not a result bundle this Xcode can read"]
    return read_summary(done.stdout)


def read_summary(out: str) -> tuple[int, list[str]]:
    """The same, from the JSON, so a test can hand this one it wrote."""
    try:
        summary = json.loads(out)
    except json.JSONDecodeError as error:
        return 0, [f"the result summary is not readable JSON: {error}"]
    cases = sum(
        int(summary.get(kind, 0) or 0)
        for kind in ("passedTests", "failedTests", "expectedFailures", "skippedTests")
    )
    bad = []
    for failure in summary.get("testFailures") or []:
        where = failure.get("targetName") or ""
        what = failure.get("testName") or failure.get("failureText") or "an unnamed test"
        bad.append(f"{where}.{what}" if where else what)
    if not bad and int(summary.get("failedTests", 0) or 0):
        # A failed count with no named failure is still a failure, and must not read as a pass.
        bad.append(f"{summary.get('failedTests')} unnamed failures")
    return cases, bad


def main() -> int:
    program, why_xcode = xcodebuild()
    found, why_simctl = ({}, None) if why_xcode else simulators()
    wanted = targets()
    if not wanted:
        print(f"apple: no test target in {PACKAGE}, so there is nothing to run")
        return 1
    says = declared(PACKAGE.read_text())
    if says != len(wanted):
        # Not a skip and not a pass: a target this could not read is a target nothing tests, and
        # the whole point of this step is that it cannot go quiet about one.
        print(
            f"apple: {said(PACKAGE)} declares {says} test targets and this step could read "
            f"{len(wanted)} ({', '.join(name for name, _ in wanted)}): it cannot say what it "
            "would be leaving out, so it is running none of them"
        )
        return 1
    named = ", ".join(name for name, _ in wanted)
    stopped = [why for why in (why_xcode, why_simctl) if why] + missing_generated()
    if stopped:
        # Loud, every run, with what is missing and what that cost. A skip nobody reads is the
        # same as no check at all, which is what this file was written to end.
        for why in stopped:
            print(f"apple: NOT RUN, {why}")
        print(
            f"apple: the unit tests of {len(wanted)} targets ({named}) were not run on this "
            "machine, so nothing about the Apple surfaces was proved here"
        )
        return 0

    code, total, failed, ran = 0, 0, False, []
    with tempfile.TemporaryDirectory(prefix="apple-check-") as folder:
        for target, platform in wanted:
            udid, where = destination(found, platform)
            if not udid:
                print(f"apple: NOT RUN, {where}, so {target} was not run")
                continue
            bundle = Path(folder) / f"{target}.xcresult"
            # `--continue` has no counterpart here, so the worst code of the runs is kept and
            # every target still runs: the point of this step is the whole list of what is broken.
            worst = run(program, target, udid, bundle)
            code = code or worst
            cases, bad = results(bundle)
            total += cases
            ran.append(target)
            if cases == 0:
                # No test ran at all, which is what a build that did not compile looks like from
                # here. xcodebuild has already printed the errors; this says what it cost.
                failed = True
                print(
                    f"apple: {target} ran no test on {where}, so it proved nothing"
                    + (f" - {bad[0]}" if bad else "")
                )
            elif bad:
                failed = True
                print(
                    "apple: %s FAILED %d of %d on %s - %s"
                    % (target, len(bad), cases, where, ", ".join(bad[:10]))
                )
            else:
                print(f"apple: {target} {cases} tests passed on {where}")
    if not ran:
        print(
            f"apple: none of the {len(wanted)} test targets ({named}) could be run on this "
            "machine, so nothing about the Apple surfaces was proved here"
        )
        return 0
    print(
        # "1 of 2" and not "1", because a half that had no simulator is the case a reader must
        # not mistake for the whole.
        "apple: %d unit tests in %d of %d test targets (%s), on %s"
        % (total, len(ran), len(wanted), ", ".join(ran), version(program))
    )
    if code == 0 and (failed or total == 0):
        # xcodebuild was happy and the bundle is not: an empty or unreadable result is not a pass.
        return 1
    return code


if __name__ == "__main__":
    sys.exit(main())
