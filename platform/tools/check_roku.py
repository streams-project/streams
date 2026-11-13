#!/usr/bin/env python3
"""Type-check the Roku surface and run its unit tests, or say out loud that they were not run.

Until 2026-11-13 nothing here ran a line of that folder. It holds 130 tests in 9 suites, written
against the contract's own fixtures, and not one of them had ever been executed; every `strict`
flag in `apps/roku/tsconfig.json` was a promise that nothing had ever asked a compiler to keep.
The folder was written on a machine with no node, so there was no runner to ask with.

The runner is `node --test`, chosen over Jest because Jest means a `package.json`, a lockfile and a
`node_modules` - a dependency tree in a repository whose whole discipline is not having one. The
cost of that choice is paid here, in three mechanical translations that live in this file and not
in the suites:

  1. The suites call `describe`, `it` and `expect(...)`. `node:test` exports `describe` and `it`
     and has no `expect` at all, so HARNESS below re-exports the first two onto `globalThis` and
     implements the twelve matchers the suites use on top of `node:assert`. What the suites assert
     is theirs; the spelling of the assertion is this file's.
  2. The suites are TypeScript with extensionless relative imports and `__dirname`, which is the
     CommonJS shape. Node's own type stripping would resolve neither, so `tsc` emits CommonJS into
     a temporary folder and node runs that. `tsc` has to be here for step 3 regardless.
  3. The emitted tree is rooted at the repository root, because every suite reaches its fixtures
     with `join(__dirname, "..", "..", "..", "..")`. `overlay` fills the gaps in that tree with
     links to the real thing, so `platform/contract/fixtures`, `strings` and `server` are where the
     suites look for them.

What is NOT checked, and is named on every run: `src/main.tsx`, the four screens under
`src/screens`, and `src/screens/remote.ts` import `@roku-sdk/solid` and `@roku-sdk/navigation`.
Those packages are not here and cannot be without the dependency tree the runner choice exists to
avoid. Writing declarations for them by hand would mean inventing an API and a set of JSX intrinsic
elements nobody has read, and a type check against an invented API is worse than none because it
reads green. They are found by their imports, not by a list here.

Where the tools are looked for, each believed only once it has answered `--version`:

  NODE / node on the PATH    node 20.12 or newer, for `--test-reporter=junit`
  TSC  / tsc  on the PATH    a TypeScript compiler

A tool named by its variable is taken as given, because naming one is a deliberate act. One found
on the PATH is run before it is believed: a version manager's shim satisfies `shutil.which` and
then exits non-zero when the version it points at is not installed, and handing that to the rest of
this file fails in a message that reads nothing like "this machine has no node". The same rule, for
the same reason, as `jdk` in check_types.py.

The exit code is node's, and 1 when node passed and the run wrote no results, or fewer cases than
there are `it(` in the suites: a step that proved nothing, or that quietly lost a suite, is not a
pass. A machine with no node or no tsc exits 0 and prints what it could not prove.

One thing is proved on every machine, node or no node: that the matchers the suites use are the
matchers HARNESS implements. That check needs no runner, and it is what stops a thirteenth matcher
arriving in a suite and failing at run time with `undefined is not a function`.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ElementTree
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ROKU = ROOT / "apps" / "roku"
CONFIG = ROKU / "tsconfig.check.json"
SUITES = ROKU / "src" / "tests"

# The patterns `tsconfig.check.json` type-checks, repeated here so this file can say which source
# files fall outside them. test_check_roku.py fails if the two ever disagree, and fails again if a
# file that does not import the Roku SDK is outside both.
INCLUDE = ("types/*.d.ts", "src/*.ts", "src/tests/*.ts", "roku-config.ts")

# `--test-reporter=junit` arrived in node 20.12. Below that the run would be unreadable rather than
# absent, which is the worse failure: it would pass.
LEAST = (20, 12)

SDK = re.compile(r'from "@roku-sdk/')
A_TEST = re.compile(r"^\s*it\(", re.MULTILINE)
A_MATCHER = re.compile(r"\.(to[A-Z][A-Za-z]*)\(")

# Names that look like a matcher and are not. A string or an array answers to these, and a suite
# that calls one inside `expect(...)` would otherwise be read as asking for a matcher nobody wrote.
NOT_MATCHERS = frozenset(
    {
        "toUpperCase",
        "toLowerCase",
        "toLocaleUpperCase",
        "toLocaleLowerCase",
        "toString",
        "toFixed",
        "toPrecision",
        "toJSON",
        "toISOString",
        "toLocaleString",
        "toSorted",
        "toReversed",
        "toSpliced",
    }
)

# The translation layer, and the only JavaScript in this repository that is not generated.
#
# It is here rather than in `apps/roku` because it belongs to the check and not to the surface: the
# day that folder gets a real toolchain, this goes and the suites do not move. `node --test` runs
# each file in its own child process, so the globals are set by a wrapper that requires this first.
#
# `toEqual` ignores own properties whose value is `undefined`, on both sides, because that is what
# the suites were written against: `assert.deepStrictEqual` alone would call `{ kind: "back" }` and
# `{ kind: "back", rating: undefined }` different, and with `exactOptionalPropertyTypes` on, both
# shapes are things this code returns. Prototypes are still compared, so a class instance is never
# equal to a plain object.
HARNESS = r'''"use strict";
// Written by platform/tools/check_roku.py. Do not edit here; edit HARNESS there.
//
// `node:test` gives `describe` and `it` and no `expect`. The twelve matchers below are every
// matcher the suites in apps/roku/src/tests use, and check_roku.py fails the run if a suite ever
// asks for a thirteenth.
const assert = require("node:assert");
const { inspect } = require("node:util");
const { describe, it } = require("node:test");

/** Whether this is an object literal, rather than an instance of something. */
function plain(value) {
  if (value === null || typeof value !== "object") return false;
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

/** The same value with own properties whose value is `undefined` dropped, recursively. */
function pruned(value, seen) {
  if (value !== null && typeof value === "object") {
    if (seen.has(value)) return value;
    seen.add(value);
  }
  if (Array.isArray(value)) return value.map((one) => pruned(one, seen));
  if (!plain(value)) return value;
  const out = {};
  for (const key of Object.keys(value)) {
    if (value[key] === undefined) continue;
    out[key] = pruned(value[key], seen);
  }
  return out;
}

function deeply(actual, expected) {
  try {
    assert.deepStrictEqual(pruned(actual, new Set()), pruned(expected, new Set()));
    return true;
  } catch (error) {
    if (error instanceof assert.AssertionError) return false;
    throw error;
  }
}

function length(actual) {
  return actual === null || actual === undefined ? NaN : actual.length;
}

function contains(actual, expected) {
  if (typeof actual === "string") return actual.includes(String(expected));
  if (Array.isArray(actual)) return actual.includes(expected);
  throw new TypeError("toContain wants a string or an array, got " + inspect(actual));
}

function matches(actual, expected) {
  return expected instanceof RegExp
    ? expected.test(String(actual))
    : String(actual).includes(String(expected));
}

function matchers(actual, want) {
  const show = (value) => inspect(value, { depth: 4, breakLength: 100 });
  function say(name, expected, pass, arity) {
    if (pass === want) return;
    throw new assert.AssertionError({
      message:
        "expect(" + show(actual) + ")" + (want ? "." : ".not.") + name +
        "(" + (arity ? show(expected) : "") + ")",
      actual: actual,
      expected: expected,
      operator: name,
      stackStartFn: say,
    });
  }
  return {
    toBe: (expected) => say("toBe", expected, Object.is(actual, expected), 1),
    toEqual: (expected) => say("toEqual", expected, deeply(actual, expected), 1),
    toBeNull: () => say("toBeNull", null, actual === null, 0),
    toBeDefined: () => say("toBeDefined", undefined, actual !== undefined, 0),
    toBeUndefined: () => say("toBeUndefined", undefined, actual === undefined, 0),
    toBeGreaterThan: (expected) => say("toBeGreaterThan", expected, actual > expected, 1),
    toBeGreaterThanOrEqual: (expected) =>
      say("toBeGreaterThanOrEqual", expected, actual >= expected, 1),
    toBeLessThan: (expected) => say("toBeLessThan", expected, actual < expected, 1),
    toBeLessThanOrEqual: (expected) => say("toBeLessThanOrEqual", expected, actual <= expected, 1),
    toContain: (expected) => say("toContain", expected, contains(actual, expected), 1),
    toMatch: (expected) => say("toMatch", expected, matches(actual, expected), 1),
    toHaveLength: (expected) => say("toHaveLength", expected, length(actual) === expected, 1),
  };
}

function expect(actual) {
  const yes = matchers(actual, true);
  yes.not = matchers(actual, false);
  return yes;
}

globalThis.describe = describe;
globalThis.it = it;
globalThis.expect = expect;
'''

# Read out of the harness itself, so the two cannot drift: a matcher is implemented when its line
# is there, not when someone remembered to name it twice.
MATCHERS = frozenset(re.findall(r"^    (to[A-Za-z]+): ", HARNESS, re.MULTILINE))


def runs(tool: str, flag: str = "--version") -> str | None:
    """What the tool says when asked its version, or None when it is not one.

    Asked by running it, never by looking. A version manager's `node` shim - nvm, volta, asdf, mise
    - is a file on the PATH that satisfies `shutil.which` and then exits non-zero when the version
    it points at is not installed. Believing the PATH there hands `tsc` and `--test` a node that is
    not one, and the run fails for a reason that reads nothing like "this machine has no node".
    """
    try:
        done = subprocess.run(
            [tool, flag], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
        )
    except OSError:
        return None
    return done.stdout.strip() if done.returncode == 0 else None


def tool(variable: str, program: str) -> tuple[str | None, str | None, str]:
    """The program to use, what is missing when there is none, and what it said about itself."""
    named = os.environ.get(variable)
    if named:
        return named, None, runs(named) or ""
    found = shutil.which(program)
    if found:
        said = runs(found)
        if said is not None:
            return found, None, said
    return None, f"no {program} that runs on the PATH and {variable} is not set", ""


def version(said: str) -> tuple[int, ...]:
    """The version out of what a tool printed, as numbers. () when it printed nothing like one."""
    found = re.search(r"(\d+)\.(\d+)\.(\d+)", said)
    return tuple(int(part) for part in found.groups()) if found else ()


def suites() -> list[Path]:
    """Every test suite in the Roku folder, read from the folder and never listed here."""
    return sorted(SUITES.glob("*.test.ts"))


def cases_written(files: list[Path]) -> int:
    """How many tests those suites declare, counted from their source.

    This is the floor the run must clear. A suite that fails to load, or a glob that stops matching
    one, otherwise shows up as a smaller green number, which is the silence this file exists to end.
    """
    return sum(len(A_TEST.findall(path.read_text())) for path in files)


def matchers_wanted(files: list[Path]) -> set[str]:
    """Every matcher those suites call."""
    found: set[str] = set()
    for path in files:
        for line in path.read_text().splitlines():
            if "expect(" in line or line.lstrip().startswith("."):
                found.update(A_MATCHER.findall(line))
    return found - NOT_MATCHERS


def sdk_bound() -> list[str]:
    """The files in the Roku folder that import the Roku SDK, so cannot be checked without it."""
    found = []
    for path in sorted(ROKU.rglob("*.ts")) + sorted(ROKU.rglob("*.tsx")):
        if SDK.search(path.read_text()):
            found.append(str(path.relative_to(ROOT)))
    return sorted(found)


def covered() -> set[Path]:
    """The files `tsconfig.check.json` type-checks, inside the Roku folder."""
    found: set[Path] = set()
    for pattern in INCLUDE:
        found.update(ROKU.glob(pattern))
    return found


def emit_config(work: Path) -> Path:
    """A config that extends the checked one and emits CommonJS, written outside the repository.

    Outside because `check_tracked.py` runs first and an untracked file in the tree is a failure of
    its own. `extends` is relative because TypeScript resolves a bare name as a package.

    `rootDir` is the repository root and not `apps/roku`: the suites import the generated contract
    types and `brand.json`, both of which are above that folder, and the emitted layout has to keep
    the same shape for `join(__dirname, "..", "..", "..", "..")` to land on a root.
    """
    path = work / "tsconfig.emit.json"
    path.write_text(
        json.dumps(
            {
                "extends": os.path.relpath(CONFIG, work),
                "compilerOptions": {
                    "noEmit": False,
                    # CommonJS, because the suites use `__dirname` and extensionless relative
                    # imports. Both are the CommonJS shape and neither resolves under node's ESM.
                    "module": "commonjs",
                    "moduleResolution": "node",
                    "rootDir": str(ROOT),
                    "outDir": str(work / "out"),
                    "declaration": False,
                    "sourceMap": False,
                },
            },
            indent=2,
        )
        + "\n"
    )
    return path


def overlay(real: Path, built: Path) -> None:
    """Link everything the real tree has that the built one has not, descending only where it must.

    The built tree holds the compiled `.js` and nothing else. The suites read the contract fixtures,
    the strings and `server/crates/server-rules/src/tier.rs` by walking up from `__dirname`, so the
    built tree has to answer for the whole repository. Links, not copies: the corpus is large and
    nothing here writes into it.
    """
    for entry in sorted(real.iterdir()):
        if entry.name == ".git":
            continue
        target = built / entry.name
        if not target.exists() and not target.is_symlink():
            target.symlink_to(entry)
        elif entry.is_dir() and target.is_dir() and not target.is_symlink():
            overlay(entry, target)


def wrappers(out: Path, files: list[Path]) -> list[Path]:
    """One runnable file per suite, each loading the harness before the suite it runs.

    A wrapper rather than `NODE_OPTIONS=--require`: node's test runner starts a child process per
    file, and a wrapper is a file on disk that cannot fail to arrive there.
    """
    harness = out / "harness.cjs"
    harness.write_text(HARNESS)
    made = []
    for path in files:
        compiled = out / path.relative_to(ROOT).with_suffix(".js")
        runner = compiled.with_name(path.name.replace(".test.ts", ".run.js"))
        runner.write_text(
            'require(%s);\nrequire("./%s");\n' % (json.dumps(str(harness)), compiled.name)
        )
        made.append(runner)
    return made


def read_results(path: Path) -> tuple[int, int, list[str]]:
    """How many cases the run wrote, how many it skipped, and the names that did not pass.

    Read from the JUnit XML node writes rather than from its console output, for the reason
    check_android.py reads Gradle's: the count is the thing that says this step proved something,
    and a console line is not a count.
    """
    if not path.is_file():
        return 0, 0, []
    try:
        root = ElementTree.parse(path).getroot()
    except ElementTree.ParseError as error:
        return 0, 0, [f"{path.name} is not readable XML: {error}"]
    cases, skipped, bad = 0, 0, []
    for case in root.iter("testcase"):
        cases += 1
        if case.find("skipped") is not None:
            skipped += 1
        for kind in ("failure", "error"):
            found = case.find(kind)
            if found is not None:
                where = case.get("classname") or ""
                bad.append(f"{where}: {case.get('name')}" if where else str(case.get("name")))
    return cases, skipped, bad


def verdict(code: int, cases: int, skipped: int, bad: list[str], wanted: int) -> tuple[int, str]:
    """The exit code and the one line that says what the run proved.

    node's own code when it is unhappy. 1 when node is happy and the results are not: no cases at
    all, fewer cases than the suites declare `it(`, every case skipped, or a named failure that
    somehow did not reach the exit code. Each of those is a run that proved less than it claims.
    """
    if bad:
        return (code or 1), "roku: FAILED %d of %d - %s" % (len(bad), cases, ", ".join(bad[:10]))
    if cases == 0:
        return (code or 1), "roku: the runner wrote no result at all, so it proved nothing"
    if cases - skipped == 0:
        return (code or 1), "roku: all %d cases were skipped, so it proved nothing" % cases
    if cases < wanted:
        return (code or 1), (
            "roku: %d cases ran and the suites declare %d tests, so a suite did not run"
            % (cases, wanted)
        )
    passed = cases - skipped
    return code, "roku: %d tests passed%s" % (
        passed,
        "" if skipped == 0 else ", %d skipped" % skipped,
    )


def announce(why: list[str], files: list[Path], wanted: int, unchecked: list[str]) -> int:
    """What a machine without the toolchain prints, every run, loud.

    A skip nobody reads is the same as no check at all, which is what this file was written to end.
    """
    for one in why:
        print(f"roku: NOT RUN, {one}")
    named = ", ".join(path.name for path in files)
    print(
        "roku: %d tests in %d suites (%s) were not run on this machine, and the TypeScript they "
        "cover was not type-checked, so nothing about the Roku surface was proved here"
        % (wanted, len(files), named)
    )
    print("roku: %s are never type-checked here, they import the Roku SDK" % ", ".join(unchecked))
    return 0


def main() -> int:
    files = suites()
    if not files:
        print(f"roku: no suite in {SUITES}, so there is nothing to run")
        return 1
    wanted = cases_written(files)
    if wanted == 0:
        print(f"roku: the {len(files)} suites in {SUITES} declare no test at all")
        return 1

    # First, and on every machine: the suites and the harness must agree about what an assertion
    # is. This needs no node, so it is the one thing this step proves everywhere.
    gap = sorted(matchers_wanted(files) - MATCHERS)
    if gap:
        print(
            "roku: the suites use %s, which HARNESS in %s does not implement - add it there, "
            "and to apps/roku/types/ambient.d.ts"
            % (", ".join(gap), Path(__file__).name)
        )
        return 1

    unchecked = sdk_bound()
    node, why_node, said_node = tool("NODE", "node")
    tsc, why_tsc, _ = tool("TSC", "tsc")
    if node and version(said_node) and version(said_node) < LEAST:
        why_node = "node %s is older than %s, which `--test-reporter=junit` needs" % (
            said_node,
            ".".join(str(part) for part in LEAST),
        )
    if why_node or why_tsc:
        return announce([one for one in (why_node, why_tsc) if one], files, wanted, unchecked)

    assert node and tsc
    with tempfile.TemporaryDirectory(prefix="roku-check-") as folder:
        # Resolved, because on a mac TMPDIR is under a symlink and `extends` below is written as
        # a relative path: a lexical `..` out of /var/folders and a physical one disagree.
        return run(Path(folder).resolve(), node, tsc, files, wanted, unchecked)


def run(
    work: Path, node: str, tsc: str, files: list[Path], wanted: int, unchecked: list[str]
) -> int:
    """Type-check, emit, run, and say what each of the three cost."""
    started = time.monotonic()
    typed = subprocess.run([tsc, "--noEmit", "-p", str(CONFIG)], cwd=ROKU)
    checking = time.monotonic() - started
    if typed.returncode != 0:
        print(
            "roku: the type check FAILED - every strict flag in apps/roku/tsconfig.json is above, "
            "and until today none of them had ever been asked"
        )
        return typed.returncode
    print(
        "roku: %d files type-checked against tsconfig.check.json in %.0fs"
        % (len(covered()), checking)
    )
    print("roku: %s were NOT type-checked, they import the Roku SDK" % ", ".join(unchecked))

    out = work / "out"
    built = subprocess.run([tsc, "-p", str(emit_config(work))], cwd=ROKU)
    if built.returncode != 0:
        print("roku: the suites type-check and will not emit, which is a bug in the emit config")
        return built.returncode
    missing = [
        str(path.relative_to(ROOT))
        for path in files
        if not (out / path.relative_to(ROOT).with_suffix(".js")).is_file()
    ]
    if missing:
        print("roku: nothing was emitted for %s, so those suites cannot run" % ", ".join(missing))
        return 1

    runners = wrappers(out, files)
    overlay(ROOT, out)
    results = work / "results.xml"
    before = time.monotonic()
    done = subprocess.run(
        [
            node,
            "--test",
            "--test-reporter=junit",
            f"--test-reporter-destination={results}",
            *[str(path) for path in runners],
        ],
        cwd=out,
    )
    running = time.monotonic() - before

    cases, skipped, bad = read_results(results)
    code, line = verdict(done.returncode, cases, skipped, bad, wanted)
    print(line)
    print(
        "roku: %d tests in %d suites, on %s, in %.0fs - %.0fs type check, %.0fs emit, %.0fs run"
        % (
            cases,
            len(files),
            node,
            time.monotonic() - started,
            checking,
            before - checking - started,
            running,
        )
    )
    return code


if __name__ == "__main__":
    sys.exit(main())
