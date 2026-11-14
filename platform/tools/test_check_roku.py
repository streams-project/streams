#!/usr/bin/env python3
"""Tests for tools/check_roku.py: the skip must be loud, and a red run must be red.

The machine this was written on has no node, no npm and no tsc, and the step it tests cannot run a
single Roku test here. That is the condition it was built for, not an excuse: `check_android.py`
and `check_apple.py` are both steps that announce on a machine without their toolchain, and this
is the third. So the runner and the compiler are both faked here, by two stub programs this file
writes, and what is proved is the step's own control flow: the no-toolchain announcement, the
exit code on a red report, on an empty one, and on one that lost a suite.

Three things are proved for real, with no stub at all, because they need no toolchain:

  * every matcher the 130 suites call is one HARNESS implements, which is the whole translation
    from the `describe`/`it`/`expect` shape the suites were written in to `node --test`;
  * every TypeScript file in apps/roku is either inside `tsconfig.check.json` or imports the Roku
    SDK, so no file can quietly fall outside the type check;
  * `overlay` puts the contract fixtures where a compiled suite walks up from `__dirname` to look.

Run from anywhere with: python3 tools/test_check_roku.py
"""

import importlib.util
import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
SCRIPT = TOOLS / "check_roku.py"

spec = importlib.util.spec_from_file_location("check_roku", SCRIPT)
roku = importlib.util.module_from_spec(spec)
spec.loader.exec_module(roku)

GREEN = """<?xml version="1.0" encoding="utf-8"?>
<testsuites>
  <testsuite name="the key list" tests="2" failures="0">
    <testcase name="is exactly the keys Roku documents" classname="keys" time="0.001"/>
    <testcase name="turns a press into a word" classname="keys" time="0.001"/>
  </testsuite>
</testsuites>
"""

RED = """<?xml version="1.0" encoding="utf-8"?>
<testsuites>
  <testsuite name="the key list" tests="2" failures="1">
    <testcase name="is exactly the keys Roku documents" classname="keys" time="0.001"/>
    <testcase name="turns a press into a word" classname="keys" time="0.001">
      <failure type="AssertionError" message="expect('OK').toBe('ok')">at keys.test.ts:52</failure>
    </testcase>
  </testsuite>
</testsuites>
"""

EMPTY = """<?xml version="1.0" encoding="utf-8"?>
<testsuites/>
"""

SKIPPED = """<?xml version="1.0" encoding="utf-8"?>
<testsuites>
  <testsuite name="the key list" tests="1" failures="0">
    <testcase name="is exactly the keys Roku documents" classname="keys" time="0">
      <skipped/>
    </testcase>
  </testsuite>
</testsuites>
"""

# A stub `tsc`. It answers --version, it passes or fails the --noEmit pass on request, and for the
# emit pass it reads the outDir out of the config check_roku.py wrote and puts a file where each
# compiled suite belongs. That last part is what makes the stub worth having: the step then goes on
# to write its wrappers, overlay the tree and read the results, so the whole path is exercised.
#
# Both stubs are shell and not Python because they are started five times per test and this file
# runs inside `./check`: measured on this machine on 2026-11-13, the Python pair cost 1.8s a run
# and the shell pair costs under 0.1s. Each sets its own PATH, because the tests empty it.
STUB_TSC = """
PATH=/usr/bin:/bin; export PATH
for one in "$@"; do
  case "$one" in
    --version) echo "${STUB_TSC_VERSION:-Version 5.6.3}"; exit 0 ;;
    --noEmit) exit "${STUB_TSC_TYPE_CODE:-0}" ;;
  esac
done
previous=""
for one in "$@"; do
  if [ "$previous" = "-p" ]; then config="$one"; fi
  previous="$one"
done
out=$(sed -n 's/.*"outDir": "\\(.*\\)".*/\\1/p' "$config")
root=$(sed -n 's/.*"rootDir": "\\(.*\\)".*/\\1/p' "$config")
mkdir -p "$out/apps/roku/src/tests"
if [ -z "${STUB_TSC_EMIT_NOTHING:-}" ]; then
  for suite in "$root"/apps/roku/src/tests/*.test.ts; do
    name=$(basename "$suite" .ts)
    echo "// compiled by the stub" > "$out/apps/roku/src/tests/$name.js"
  done
fi
exit 0
"""

# A stub `node`. It answers --version with whatever the test wants, and for --test it writes the
# results file the test handed it and exits with the code the test handed it.
STUB_NODE = """
for one in "$@"; do
  case "$one" in
    --version) echo "${STUB_NODE_VERSION:-v22.11.0}"; exit 0 ;;
    --test-reporter-destination=*)
      printf '%s' "$STUB_NODE_RESULTS" > "${one#--test-reporter-destination=}" ;;
  esac
done
exit "${STUB_NODE_CODE:-0}"
"""


def stub(folder, name, body):
    """One of the two stub programs, on disk and runnable with an empty PATH.

    Written only when it is not already there with the same text: rewriting a program macOS has
    already seen makes it a program macOS has not seen, and the check it does then costs about a
    second. That is per test, and there are eleven tests that want these two.
    """
    path = Path(folder) / name
    text = "#!/bin/sh%s" % body
    if not path.is_file() or path.read_text() != text:
        path.write_text(text)
        path.chmod(path.stat().st_mode | stat.S_IXUSR)
    return path


def check(**extra):
    """The step, run as its own program, with the environment the test wants.

    PATH is emptied in every case so a machine that does have node or tsc cannot turn a test about
    a machine without them into a test about this one. That is the trap test_check_android.py names
    for JBR_HOME, in the shape this file has.
    """
    folder = tempfile.mkdtemp(prefix="roku-stubs-")
    environment = {
        name: value
        for name, value in os.environ.items()
        if name not in ("PATH", "NODE", "TSC") and not name.startswith("STUB_")
    }
    environment["PATH"] = folder
    environment.update(extra)
    done = subprocess.run(
        [sys.executable, str(SCRIPT)], capture_output=True, text=True, env=environment
    )
    return done.returncode, done.stdout + done.stderr


# The stubs are written once for the whole file, not once per test. Measured on this machine on
# 2026-11-13: writing a fresh executable and running it costs about a second the first time, which
# is macOS checking a program it has not seen before, and 22 of those is most of a minute.
TOOLCHAIN = tempfile.mkdtemp(prefix="roku-stubs-")


def with_tools(results=GREEN, code="0", **extra):
    """The step with both stubs in place, told what the runner should report."""
    return check(
        NODE=str(stub(TOOLCHAIN, "node", STUB_NODE)),
        TSC=str(stub(TOOLCHAIN, "tsc", STUB_TSC)),
        STUB_NODE_RESULTS=results,
        STUB_NODE_CODE=code,
        **extra,
    )


class NoToolchain(unittest.TestCase):
    def test_a_machine_with_no_node_still_passes_and_says_what_it_skipped(self):
        code, output = check()
        self.assertEqual(code, 0, output)
        self.assertIn("roku: NOT RUN", output)
        self.assertIn("no node that runs", output)
        self.assertIn("no tsc that runs", output)
        self.assertIn("nothing about the Roku surface was proved here", output)

    def test_the_skip_names_every_suite_it_did_not_run_and_how_many_tests_that_was(self):
        _, output = check()
        for suite in roku.suites():
            self.assertIn(suite.name, output)
        # Both numbers come from the read, not from a copy of it here. This line said `9` and went
        # red on 2026-11-13 when the folder reached 12 suites: a test asserting a number against a
        # restatement of itself, which is the defect this folder keeps finding and which the test
        # beside it in `test_check_claims.py` had already been fixed for.
        written = roku.suites()
        self.assertIn("%d tests in %d suites" % (roku.cases_written(written), len(written)), output)
        self.assertGreater(len(written), 9)

    def test_the_skip_names_the_files_the_sdk_keeps_out_of_the_type_check(self):
        """Not an aside. Six files in that folder can never be checked here, and a run that did not
        say so would read as if the whole surface had been proved."""
        _, output = check()
        self.assertIn("import the Roku SDK", output)
        for name in roku.sdk_bound():
            self.assertIn(name, output)

    def test_a_node_too_old_for_the_junit_reporter_is_a_skip_and_not_a_run(self):
        """node below 20.12 has no `--test-reporter=junit`. Running anyway would produce no results
        file, which this step would then have to read as "proved nothing"."""
        code, output = with_tools(STUB_NODE_VERSION="v18.20.4")
        self.assertEqual(code, 0, output)
        self.assertIn("older than 20.12", output)
        self.assertIn("NOT RUN", output)


class Verdict(unittest.TestCase):
    def test_a_red_report_is_red_and_names_what_failed(self):
        code, output = with_tools(results=RED, code="1")
        self.assertNotEqual(code, 0)
        self.assertIn("roku: FAILED 1 of 2", output)
        self.assertIn("turns a press into a word", output)

    def test_a_failure_in_the_report_outranks_a_happy_runner(self):
        """If the runner ever exits 0 with a failure in its own report, the report wins."""
        code, output = with_tools(results=RED, code="0")
        self.assertEqual(code, 1, output)
        self.assertIn("roku: FAILED", output)

    def test_a_run_that_wrote_no_result_is_not_a_pass(self):
        code, output = with_tools(results=EMPTY, code="0")
        self.assertEqual(code, 1, output)
        self.assertIn("proved nothing", output)

    def test_a_run_that_skipped_everything_is_not_a_pass(self):
        code, output = with_tools(results=SKIPPED, code="0")
        self.assertEqual(code, 1, output)
        self.assertIn("were skipped", output)

    def test_a_run_that_lost_a_suite_is_not_a_pass(self):
        """GREEN carries 2 cases and the suites declare 130. A glob that stopped matching a file,
        or a suite that failed to load before registering anything, looks exactly like this."""
        code, output = with_tools(results=GREEN, code="0")
        self.assertEqual(code, 1, output)
        self.assertIn("a suite did not run", output)

    def test_a_full_green_report_passes_and_counts_what_it_proved(self):
        wanted = roku.cases_written(roku.suites())
        cases = "\n".join(
            '    <testcase name="case %d" classname="keys" time="0.001"/>' % index
            for index in range(wanted)
        )
        green = '<testsuites>\n  <testsuite name="all" tests="%d">\n%s\n  </testsuite>\n</testsuites>'
        code, output = with_tools(results=green % (wanted, cases), code="0")
        self.assertEqual(code, 0, output)
        self.assertIn("roku: %d tests passed" % wanted, output)

    def test_a_type_error_fails_the_run_before_a_single_test(self):
        code, output = with_tools(STUB_TSC_TYPE_CODE="2")
        self.assertNotEqual(code, 0)
        self.assertIn("the type check FAILED", output)
        self.assertNotIn("tests passed", output)

    def test_a_suite_that_did_not_compile_is_named_rather_than_skipped(self):
        code, output = with_tools(STUB_TSC_EMIT_NOTHING="1")
        self.assertEqual(code, 1, output)
        self.assertIn("nothing was emitted for", output)


class Matchers(unittest.TestCase):
    def test_every_matcher_the_suites_use_is_one_the_harness_implements(self):
        """The translation itself. `node:test` has no `expect`, so every assertion in those 130
        tests runs through the twelve matchers in HARNESS; one the suites call and HARNESS has not
        got would be `undefined is not a function` at run time, inside a test, reported as that
        test failing for a reason that has nothing to do with the code under test."""
        wanted = roku.matchers_wanted(roku.suites())
        self.assertTrue(wanted)
        self.assertEqual(wanted - roku.MATCHERS, set(), "HARNESS is missing these")

    def test_a_thirteenth_matcher_is_caught_before_it_runs(self):
        folder = Path(tempfile.mkdtemp(prefix="roku-suite-"))
        suite = folder / "made_up.test.ts"
        suite.write_text('it("x", () => {\n  expect(a).toHaveProperty("b");\n});\n')
        self.assertEqual(roku.matchers_wanted([suite]), {"toHaveProperty"})
        self.assertNotIn("toHaveProperty", roku.MATCHERS)

    def test_a_string_method_inside_an_expect_is_not_read_as_a_matcher(self):
        folder = Path(tempfile.mkdtemp(prefix="roku-suite-"))
        suite = folder / "cased.test.ts"
        suite.write_text('expect(word.toUpperCase()).toBe("OK");\n')
        self.assertEqual(roku.matchers_wanted([suite]), {"toBe"})

    def test_a_matcher_on_its_own_line_is_still_found(self):
        folder = Path(tempfile.mkdtemp(prefix="roku-suite-"))
        suite = folder / "wrapped.test.ts"
        suite.write_text("expect(\n  value\n)\n  .toBeNull();\n")
        self.assertIn("toBeNull", roku.matchers_wanted([suite]))

    def test_the_harness_puts_all_three_names_where_a_suite_looks_for_them(self):
        for name in ("describe", "it", "expect"):
            self.assertIn("globalThis.%s = " % name, roku.HARNESS)

    def test_every_matcher_is_declared_for_the_type_checker_too(self):
        """A matcher HARNESS implements and ambient.d.ts does not declare type-checks as an error
        on every call site; the other way round is a green type check and a crash."""
        declared = (roku.ROKU / "types" / "ambient.d.ts").read_text()
        for matcher in sorted(roku.MATCHERS):
            self.assertIn("  %s(" % matcher, declared, matcher)


class Reports(unittest.TestCase):
    def test_a_failure_in_the_xml_is_named(self):
        folder = Path(tempfile.mkdtemp(prefix="roku-red-"))
        path = folder / "results.xml"
        path.write_text(RED)
        self.assertEqual(roku.read_results(path), (2, 0, ["keys: turns a press into a word"]))

    def test_a_green_report_counts_its_cases(self):
        folder = Path(tempfile.mkdtemp(prefix="roku-green-"))
        path = folder / "results.xml"
        path.write_text(GREEN)
        self.assertEqual(roku.read_results(path), (2, 0, []))

    def test_a_report_that_was_never_written_counts_nothing(self):
        folder = Path(tempfile.mkdtemp(prefix="roku-none-"))
        self.assertEqual(roku.read_results(folder / "results.xml"), (0, 0, []))

    def test_a_report_that_is_not_xml_is_a_failure_and_not_an_empty_run(self):
        folder = Path(tempfile.mkdtemp(prefix="roku-torn-"))
        path = folder / "results.xml"
        path.write_text("<testsuites><testsuite>")
        _, _, bad = roku.read_results(path)
        self.assertTrue(bad)


class Config(unittest.TestCase):
    def test_the_patterns_this_file_checks_are_the_patterns_the_config_type_checks(self):
        """INCLUDE is read by `covered` to say which files fall outside the type check. If it drifts
        from the config, that sentence stops being true and the test below stops meaning anything."""
        text = (roku.ROKU / "tsconfig.check.json").read_text()
        for pattern in roku.INCLUDE:
            self.assertIn('"%s"' % pattern, text)

    def test_every_typescript_file_in_the_folder_is_checked_or_is_held_out_by_the_sdk(self):
        """The hole this closes: a new `src/screens/helper.ts` with no SDK import would fall outside
        both the `include` globs and the announced list, and nothing would ever look at it."""
        held = set(roku.sdk_bound())
        checked = roku.covered()
        for path in sorted(roku.ROKU.rglob("*.ts")) + sorted(roku.ROKU.rglob("*.tsx")):
            name = str(path.relative_to(roku.ROOT))
            self.assertTrue(path in checked or name in held, name)

    def test_the_sdk_list_is_not_empty_and_is_not_everything(self):
        """A regular expression that matched nothing, or matched every file, would make the two
        tests above pass while proving nothing."""
        held = roku.sdk_bound()
        self.assertTrue(held)
        self.assertTrue(roku.covered())
        self.assertNotIn("apps/roku/src/keys.ts", held)

    def test_the_emit_config_reaches_the_checked_one_by_a_relative_path(self):
        """TypeScript resolves a bare name in `extends` as a package, so a path that does not start
        with a dot would be looked for in a `node_modules` this repository does not have."""
        # Resolved the way main() resolves it: on a mac the temporary folder is under a symlink,
        # and a relative `extends` out of it only lands on the right file if both ends agree
        # about which path the folder has.
        folder = Path(tempfile.mkdtemp(prefix="roku-emit-")).resolve()
        written = json.loads(roku.emit_config(folder).read_text())
        self.assertTrue(written["extends"].startswith("."))
        self.assertEqual(
            (folder / written["extends"]).resolve(), (roku.ROKU / "tsconfig.check.json").resolve()
        )
        self.assertEqual(written["compilerOptions"]["module"], "commonjs")
        self.assertFalse(written["compilerOptions"]["noEmit"])


class Overlay(unittest.TestCase):
    def test_a_compiled_suite_finds_the_fixtures_it_walks_up_to(self):
        """Every suite reads `join(__dirname, "..", "..", "..", "..")` and expects a repository
        root. The emitted tree holds `apps/roku/src/tests` and nothing else, so without this the
        corpus, the strings and `server-rules/src/tier.rs` are all missing and nine suites throw."""
        out = Path(tempfile.mkdtemp(prefix="roku-out-"))
        (out / "apps" / "roku" / "src" / "tests").mkdir(parents=True)
        (out / "apps" / "roku" / "src" / "tests" / "keys.test.js").write_text("//\n")
        roku.overlay(roku.ROOT, out)
        reached = out / "apps" / "roku" / "src" / "tests" / ".." / ".." / ".." / ".."
        self.assertTrue((reached / "platform" / "contract" / "fixtures").is_dir())
        self.assertTrue((reached / "strings").is_dir())
        self.assertTrue(
            (reached / "server" / "crates" / "server-rules" / "src" / "tier.rs").is_file()
        )

    def test_it_never_writes_over_what_was_compiled(self):
        out = Path(tempfile.mkdtemp(prefix="roku-out-"))
        (out / "apps" / "roku" / "src" / "tests").mkdir(parents=True)
        compiled = out / "apps" / "roku" / "src" / "tests" / "keys.test.js"
        compiled.write_text("// compiled\n")
        roku.overlay(roku.ROOT, out)
        self.assertEqual(compiled.read_text(), "// compiled\n")
        self.assertFalse(compiled.is_symlink())
        # And the real source is linked in beside it, where CommonJS resolution will never reach
        # it: `require("../keys")` finds `keys.js` first and has no idea `.ts` is a thing.
        self.assertTrue((out / "apps" / "roku" / "src" / "keys.ts").is_symlink())


if __name__ == "__main__":
    unittest.main()
