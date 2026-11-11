#!/usr/bin/env python3
"""The Android app really strips the core, and keeps the symbols somewhere it can read them.

Measured on 2026-11-11: `lib/arm64-v8a/libstreamscore.so` in the APK was 9,439,528 bytes, "with
debug_info, not stripped", and Android Gradle's strip task wrote a file with the same SHA1 as its
input — a strip that was a no-op. The reason: `ndkVersion` was declared on `:core` and not on
`:app`, and `:app` is the module that packages native libraries, so it had no NDK to find
`llvm-strip` in and passed the library through untouched. Native libraries are stored in an APK
without compression, so the whole of the debug information was shipped at full size:
`llvm-strip --strip-unneeded` from NDK 27.2 gives 3,302,872 (arm64) and 2,568,236 (armv7), which is
11.9 MB off the two of them.

Cargo's `strip = true` cannot do this: `bridge-c` is a staticlib, and the DWARF rides into the
shared library when the NDK links it.

Nothing here runs Gradle: the facts are read out of the build files, because a build is minutes of
one machine and this is the kind of fact an edit undoes silently.

  one version   `apps/android/core/ndk-version.txt` is the only place the NDK version is written
  it is set     every module that builds or packages native code declares `ndkVersion` from it
  it is kept    the application module asks for the symbols to be kept for symbolising
  not undone    nothing in the build asks for the debug symbols to stay in the package
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ANDROID = ROOT / "apps" / "android"
VERSION_FILE = ANDROID / "core" / "ndk-version.txt"

# An installed NDK is named like 27.2.12479018.
A_VERSION = re.compile(r"^\d+\.\d+\.\d+$")

# How the one declaration is read in a build file.
READS_THE_FILE = 'rootDir.resolve("core/ndk-version.txt").readText().trim()'

# A module that holds native code or packages it needs the NDK: the first builds with CMake, the
# second runs the strip task. The plugin counts only where it is really applied: the root build
# file names every plugin with `apply false`, and applies none of them.
APPLIES_THE_APP_PLUGIN = re.compile(r'id\("com\.android\.application"\)(?!.*apply false)')

# Settings that put the debug information back into the package, or keep the strip from running.
PUTS_IT_BACK = {
    "keepDebugSymbols": "this keeps the debug information inside the package",
    "doNotStrip": "this keeps the debug information inside the package",
    'debugSymbolLevel = "NONE"': "this throws the symbols away instead of keeping them",
}

LEAST_FILES = 3


def build_files() -> list[Path]:
    return sorted(ANDROID.rglob("build.gradle.kts"))


def scan(name: str, text: str) -> tuple[list[str], bool]:
    """Every failing fact in one build file, and whether it is the module that packages the app."""
    found = []
    for number, line in enumerate(text.splitlines(), 1):
        if re.search(r'ndkVersion\s*=\s*"', line):
            found.append(
                f"{name}:{number}: the NDK version is written here as well as in "
                f"{VERSION_FILE.relative_to(ROOT)}: read it with {READS_THE_FILE}"
            )
        for bad, why in PUTS_IT_BACK.items():
            if bad in line:
                found.append(f"{name}:{number}: {bad}: {why}")
    is_application = bool(APPLIES_THE_APP_PLUGIN.search(text))
    if not (is_application or "externalNativeBuild" in text):
        return found, False
    if is_application and "debugSymbolLevel" not in text:
        found.append(
            f"{name}: no debugSymbolLevel: the app would ship stripped with nothing kept to read "
            "a crash address with"
        )
    if "ndkVersion" not in text:
        found.append(
            f"{name}: builds or packages native code and declares no ndkVersion: the strip task "
            "has no llvm-strip to run and copies the library through unstripped"
        )
    elif READS_THE_FILE not in text:
        found.append(
            f"{name}: declares ndkVersion from somewhere other than "
            f"{VERSION_FILE.relative_to(ROOT)}"
        )
    return found, is_application


def main() -> int:
    found: list[str] = []
    if not VERSION_FILE.is_file():
        print(f"{VERSION_FILE.relative_to(ROOT)}: missing: the NDK version has no home")
        return 1
    version = VERSION_FILE.read_text("utf-8").strip()
    if not A_VERSION.match(version):
        found.append(f"{VERSION_FILE.relative_to(ROOT)}: {version!r} is not an NDK version")

    files = [path for path in build_files() if "/build/" not in path.as_posix()]
    application = []
    for path in files:
        name = path.relative_to(ROOT).as_posix()
        lines, is_application = scan(name, path.read_text("utf-8"))
        found += lines
        if is_application:
            application.append(name)

    print(
        f"android native: NDK {version}, read {len(files)} build files, "
        f"application modules: {', '.join(application) or 'none'}"
    )
    if len(files) < LEAST_FILES:
        found.append(
            f"read {len(files)} build files, expected at least {LEAST_FILES}: this check is not "
            "reading the android family"
        )
    if not application:
        found.append("no application module found: the module that packages the library is the one that strips it")
    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
