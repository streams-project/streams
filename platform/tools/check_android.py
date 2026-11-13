#!/usr/bin/env python3
"""Run the Android modules' unit tests, or say out loud that they were not run.

Until 2026-11-13 `./check` ran no Gradle at all. The Android surfaces held 219 passing unit tests
that CI could not see, so every compile error and every test regression on that side was caught by
a person or not at all. `apps/android/surfaces/tv/build.gradle.kts` said so in a comment.

The difficulty is that `./check` must stay runnable on a machine with no JDK and no Android SDK -
CI is one such machine today, because the workflow deletes /usr/local/lib/android to free disk. A
step that cannot run everywhere and exits 1 stops every push; a step that skips in silence is the
defect class this repository exists to prevent. So this follows check_types.py: it probes for the
tools, runs one before believing it, and when it cannot run it prints what it did NOT check, by
name, on every run.

What it runs, and what it does not. `testDebugUnitTest` only: the release variant compiles the same
sources and would double the wall clock for nothing. The instrumented tests, the APK build and the
lint tasks are not here either - they need a device, an emulator or minutes of this machine.

Where the tools are looked for, in order, each believed only once it has answered:

  JAVA_HOME          a JDK, taken as given when it is set, because setting it is a deliberate act
  java on the PATH   probed by running it: /usr/bin/java is a macOS stub that exits 1 with
                     "Unable to locate a Java Runtime", and satisfies `shutil.which` anyway
  JBR_HOME           Android Studio's bundled JDK, which on this project's machines is the only
                     real one. Pointing JBR_HOME at a folder that is not a JDK is how a test, and
                     a person, simulates a machine with no Java at all.

  ANDROID_HOME, then ANDROID_SDK_ROOT, then sdk.dir in apps/android/local.properties, then
  ~/Library/Android/sdk. A folder is believed when it holds a `platforms` directory.

The exit code is Gradle's, and 1 when Gradle passed but no test result was written: a run that
proved nothing is a failure, not a pass. A machine missing either tool exits 0 and says so.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import xml.etree.ElementTree as ElementTree
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ANDROID = ROOT / "apps" / "android"
SETTINGS = ANDROID / "settings.gradle.kts"

# Android Studio's bundled JDK, the last place a JDK is looked for and the only real one on this
# project's machines. It is read from the environment so a test can point it at an empty folder and
# simulate a machine with no JDK, which this fallback would otherwise defeat. The same rule, for
# the same reason, as JBR_BIN in check_types.py.
JBR = os.environ.get(
    "JBR_HOME", "/Applications/Android Studio.app/Contents/jbr/Contents/Home"
)

# Where the Android SDK is looked for last, when nothing in the environment names one.
SDK_GUESSES = (Path.home() / "Library" / "Android" / "sdk", Path.home() / "Android" / "Sdk")

# Every module, read from the build rather than written here: a module must not be able to arrive
# without its tests arriving with it.
#
# This read "..surfaces.." until 2026-11-13, and the day it was written that covered every module
# holding tests. It stopped being true the same day: `:ports` gained four of them, proving Java
# can read the generated contract, and they ran in no check at all -- so the build line that
# opened that gate could have closed again and nothing would have said so. A check that names a
# subset by spelling rather than by the property it cares about is the defect this file exists to
# end, and it had the defect.
A_MODULE = re.compile(r'"(:[a-z0-9:-]+)"')

# An Android module's tests live behind a build variant; a plain JVM module's do not. The plugin
# in the module's own build file decides, so neither name is written here.
VARIANT = "testDebugUnitTest"
PLAIN = "test"
ANDROID_PLUGIN = re.compile(r"com\.android\.(?:library|application)")


def answers(java: Path) -> bool:
    """Whether a `java` really runs, asked by running it.

    macOS ships /usr/bin/java as a stub: it exists, `shutil.which` finds it, and with no JDK
    installed it exits 1 saying "Unable to locate a Java Runtime". Measured on this machine on
    2026-11-13. Believing the PATH there hands Gradle a JDK that is not one, and the run fails for
    a reason that reads nothing like "this machine has no Java".
    """
    try:
        done = subprocess.run(
            [str(java), "-version"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )
    except OSError:
        return False
    return done.returncode == 0


def java_home() -> tuple[str | None, str | None]:
    """A JDK for Gradle to run on, and what is missing when there is none."""
    named = os.environ.get("JAVA_HOME")
    if named:
        return named, None
    on_path = os.environ.get("PATH", "")
    # parents[1] because _java_on answers with `<home>/bin/java`, and Gradle wants `<home>`.
    for folder in [Path(part).parents[1] for part in _java_on(on_path)] + [Path(JBR)]:
        if answers(folder / "bin" / "java"):
            return str(folder), None
    return None, (
        "no java that runs, on the PATH or in Android Studio's JDK at "
        f"{JBR}, and JAVA_HOME is not set"
    )


def _java_on(path: str) -> list[str]:
    """Every `java` on the PATH that is executable, in PATH order, as bin/java paths."""
    found = []
    for part in path.split(os.pathsep):
        if not part:
            continue
        candidate = Path(part) / "java"
        if candidate.is_file() and os.access(candidate, os.X_OK):
            found.append(str(candidate))
    return found


def sdk_home() -> tuple[str | None, str | None]:
    """The Android SDK, and what is missing when there is none.

    A folder counts as an SDK when it holds `platforms`: an empty ~/Library/Android/sdk is left
    behind by an Android Studio that was opened once and never used, and handing that to Gradle
    fails deep inside the Android plugin instead of here.
    """
    for variable in ("ANDROID_HOME", "ANDROID_SDK_ROOT"):
        named = os.environ.get(variable)
        if named:
            return named, None
    guesses = list(SDK_GUESSES)
    local = ANDROID / "local.properties"
    if local.is_file():
        for line in local.read_text().splitlines():
            if line.startswith("sdk.dir="):
                guesses.insert(0, Path(line.split("=", 1)[1].strip()))
    for folder in guesses:
        if (folder / "platforms").is_dir():
            return str(folder), None
    return None, (
        "no Android SDK: ANDROID_HOME and ANDROID_SDK_ROOT are not set and no `platforms` folder "
        "in " + " or ".join(str(folder) for folder in guesses)
    )


def surfaces() -> list[tuple[str, Path, str]]:
    """Every module that holds unit tests: its gradle path, its folder, and the task that runs them.

    A module with no `src/test` is not named, because asking gradle for a task that does not exist
    fails the whole run. A module that grows one is picked up with no edit here.
    """
    found = []
    for path in sorted(set(A_MODULE.findall(SETTINGS.read_text()))):
        folder = ANDROID / path.strip(":").replace(":", "/")
        if not (folder / "src" / "test").is_dir():
            continue
        build = folder / "build.gradle.kts"
        android = build.is_file() and ANDROID_PLUGIN.search(build.read_text())
        found.append((path, folder, VARIANT if android else PLAIN))
    return found


def results(folder: Path, task: str) -> tuple[int, list[str]]:
    """How many test cases that module's last run wrote, and the names that did not pass."""
    return read_reports(folder / "build" / "test-results" / task)


def read_reports(folder: Path) -> tuple[int, list[str]]:
    """The same, from a folder, so a test can hand this one it wrote.

    Read from the XML Gradle writes, not from its console output: a task reported UP-TO-DATE prints
    no counts, and the count is what says this step proved something.
    """
    cases, bad = 0, []
    for report in sorted(folder.glob("*.xml")):
        try:
            suite = ElementTree.parse(report).getroot()
        except ElementTree.ParseError as error:
            bad.append(f"{report.name} is not readable XML: {error}")
            continue
        for case in suite.iter("testcase"):
            cases += 1
            for kind in ("failure", "error"):
                if case.find(kind) is not None:
                    bad.append(f"{case.get('classname')}.{case.get('name')}")
    return cases, bad


def gradle(home: str, sdk: str, modules: list[str]) -> int:
    """Run the unit tests of every surface module, and let each one report.

    `--continue` because the point of this step is the list of what is broken: without it the first
    failing module hides the other two, and the next run only uncovers the next one.
    """
    wrapper = ANDROID / "gradlew"
    if not os.access(wrapper, os.X_OK):
        # Not a skip: the wrapper is a tracked file, so a clone that has not got it is broken and
        # this says which file rather than raising OSError out of subprocess.
        print(f"android: {wrapper} is not there to run")
        return 1
    environment = dict(os.environ, JAVA_HOME=home, ANDROID_HOME=sdk)
    command = ["./gradlew", "--continue"] + [f"{path}:{task}" for path, _, task in modules]
    done = subprocess.run(command, cwd=ANDROID, env=environment)
    return done.returncode


def main() -> int:
    home, why_java = java_home()
    sdk, why_sdk = sdk_home()
    modules = surfaces()
    if not modules:
        print(f"android: no module in {SETTINGS} holds tests, so there is nothing to run")
        return 1
    named = ", ".join(path for path, _, _ in modules)
    if why_java or why_sdk:
        # Loud, every run, with what is missing and what that cost. A skip nobody reads is the
        # same as no check at all, which is what this file was written to end.
        for why in (why_java, why_sdk):
            if why:
                print(f"android: NOT RUN, {why}")
        print(
            f"android: the unit tests of {len(modules)} modules ({named}) were not run on this "
            "machine, so nothing about the Android surfaces was proved here"
        )
        return 0

    code = gradle(home, sdk, modules)
    total, failed = 0, False
    for path, folder, task in modules:
        cases, bad = results(folder, task)
        total += cases
        if bad:
            failed = True
            print(
                "android: %s FAILED %d of %d - %s" % (path, len(bad), cases, ", ".join(bad[:10]))
            )
        elif cases == 0:
            failed = True
            print(f"android: {path} wrote no test result, so it proved nothing")
        else:
            print(f"android: {path} {cases} tests passed")
    print("android: %d unit tests in %d modules (%s), on %s" % (total, len(modules), named, home))
    if code == 0 and (failed or total == 0):
        # Gradle was happy and the reports are not: a stale or empty report is not a pass.
        return 1
    return code


if __name__ == "__main__":
    sys.exit(main())
