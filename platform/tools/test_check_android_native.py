#!/usr/bin/env python3
"""Tests for tools/check_android_native.py, against the silent no-op strip of 2026-11-11.

The build file as it was that morning is planted here, so the day somebody takes `ndkVersion` back
off the application module, a test says which 11.9 MB went back into the APK.

Run from anywhere with: python3 tools/test_check_android_native.py
"""

import importlib.util
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location(
    "check_android_native", TOOLS / "check_android_native.py"
)
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)

# The application module as it was: no ndkVersion, so no strip, so the DWARF shipped.
AS_IT_WAS = """
plugins {
    id("com.android.application")
}
android {
    namespace = "streams.app"
    compileSdk = 37
    buildTypes { release { isMinifyEnabled = true } }
}
"""

# The version written out in a second place, which is how the two modules came to disagree.
A_SECOND_COPY = """
plugins { id("com.android.library") }
android {
    ndkVersion = "27.2.12479018"
    externalNativeBuild { cmake { path = file("src/main/cpp/CMakeLists.txt") } }
}
"""

ROOT_FILE = """
plugins {
    id("com.android.application") version "9.3.1" apply false
    id("com.android.library") version "9.3.1" apply false
}
"""


class Native(unittest.TestCase):
    def test_the_tree_strips_and_keeps_the_symbols(self):
        self.assertEqual(native.main(), 0)

    def test_an_application_module_with_no_ndk_version_is_a_failure(self):
        found, is_application = native.scan("app/build.gradle.kts", AS_IT_WAS)
        self.assertTrue(is_application)
        self.assertTrue(any("no llvm-strip to run" in line for line in found))
        self.assertTrue(any("no debugSymbolLevel" in line for line in found))

    def test_a_second_copy_of_the_version_is_a_failure(self):
        found, _ = native.scan("core/build.gradle.kts", A_SECOND_COPY)
        self.assertTrue(any("written here as well as in" in line for line in found))

    def test_the_root_build_file_is_not_a_module(self):
        found, is_application = native.scan("build.gradle.kts", ROOT_FILE)
        self.assertFalse(is_application)
        self.assertEqual(found, [])

    def test_putting_the_symbols_back_into_the_package_is_a_failure(self):
        for bad in native.PUTS_IT_BACK:
            with self.subTest(bad=bad):
                found, _ = native.scan(
                    "app/build.gradle.kts",
                    AS_IT_WAS.replace("isMinifyEnabled = true", bad + " = true"),
                )
                self.assertTrue(any(bad in line for line in found))

    def test_the_version_is_declared_once_and_read_by_both_modules(self):
        self.assertTrue(native.A_VERSION.match(native.VERSION_FILE.read_text().strip()))
        for name in ("app/build.gradle.kts", "core/build.gradle.kts"):
            with self.subTest(name=name):
                text = (native.ANDROID / name).read_text("utf-8")
                self.assertIn(native.READS_THE_FILE, text)

    def test_the_declared_version_is_the_one_the_native_build_uses(self):
        # `:core` builds the Rust staticlib and CMake links it; both run under this NDK.
        version = native.VERSION_FILE.read_text("utf-8").strip()
        self.assertIn(version.split(".")[0], ("27", "28", "29"), "an NDK this old or newer")


if __name__ == "__main__":
    unittest.main()
