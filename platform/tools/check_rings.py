#!/usr/bin/env python3
"""The rings: every app family is its own hexagon, and this makes that true instead of a claim.

A family has `ports`, `adapters` and `surfaces`. A surface draws, and reaches the world only through
a port. So a word that names a browser or system feature behind a port may not appear in a surface.

A family may keep its surfaces in more than one folder, so each one names them by pattern. The
thing that must be true is not a number: it is that **every file of the family is in a ring this
read**. A count alone cannot say that — the apple family kept its phone surfaces outside the one
folder this read and nineteen files went unscanned for weeks, and a count of 71 against a floor of
71 would have passed just the same the day somebody added a new folder of screens. So three things
are asserted, in this order:

  patterns    every `surfaces` pattern of a family matched at least one real folder
  parts       the `surfaces` and `adapters` folders at the family root are inside the read set,
              whether they are folders or links to folders
  no strays   every code file the family has in git is inside a ring, or inside a folder the
              family declares as `rest`, which is the part of the app that is not a ring
  depends     no surface module DECLARES a dependency that reaches the device. The read above is a
              read of words in source, which is a different assertion: the android family's
              `okhttp3` ban passed on every run while okhttp3 was linked into all three surface
              modules, arriving under `io.coil-kt.coil3:coil-network-okhttp`, which those modules
              declared themselves. 78 okhttp3 and 89 okio classes were in the APK while this read
              said the network was behind a port. A dependency is not a word anybody types

`least` stays, with a margin under today's count, and it is now only a floor against a scan that
read nothing like the tree: a deleted file is bookkeeping, a missing folder is caught above.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# A family's folder, what its code ends in, where its surfaces live, the floor under the number of
# surface files, the parts of the family that are not rings, and what a surface may not name.
# `least` carries a margin under the real count, so an ordinary deletion is bookkeeping; `rest` is
# the whole of the rest of the family, so a folder that is in neither is a failure with a name.
# What a surface module may not DEPEND ON, read from the coordinates its own build file declares.
#
# Only the android family has a build file per surface. The apple family's surface targets declare
# in-package targets alone (`Package.swift`: Contract, Ports, SurfacesShared), the web family is one
# crate with one manifest, and the television page was stopped, so for those three there is nothing
# of this kind to read and `builds` is empty on purpose rather than by omission.
#
# What is read is what a build file DECLARES, not the transitive graph: the whole graph needs a
# Gradle run, and this is a guard that has to stay seconds long -- `./check` starts no Gradle at
# all. So a surface that declares something harmless which drags a banned library in behind it is
# NOT caught here. That hole is the smaller one of the two: a declaration is a line somebody chose
# to type, and this read now sees every one of them.
FAMILIES = {
    # The television page was deleted on 2026-11-12. The owner stopped it on 2026-11-07 (note 179):
    # it was a DOM page served by the server, and every television now gets a native app. Its
    # measurements survive in `pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md`, which other families cite, until they are
    # rehoused in a note. There is no family here because there is no code here.
    "web": {
        "code": "apps/web/src",
        "ends": {".rs"},
        "surfaces": ("surfaces",),
        "least": 4,
        "rest": ("host.rs", "main.rs", "state.rs"),
        "skip": set(),
        "builds": (),
        "banned_deps": {},
        "allowed_deps": frozenset(),
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bfetch_with|\bRequestInit\b|\breqwest\b",
            "the secrets port": r"\blocal_storage\b|\bsession_storage\b",
            "the address port": r"\bhistory\(\)|\bpush_state",
            "the player port": r"\bwebPlayer\b|\bHtmlVideoElement\b",
            "a port, and adapters are not for surfaces": r"crate::adapters",
        },
    },
    "android": {
        "code": "apps/android",
        # Java as well as Kotlin, and not because any Java is here: with `.kt` alone a `.java` file
        # was in no ring and in no `rest`, and both the stray read and the banned-word read skipped
        # it on its suffix, so the hexagon simply stopped being enforced for it. The same shape as
        # the apple family's nineteen unscanned files, one suffix earlier. `apple` keeps `.swift`
        # alone for now; if Objective-C is ever admitted there, `.m` and `.h` belong in it the same
        # day, for the same reason.
        "ends": {".kt", ".java"},
        "surfaces": ("surfaces",),
        "least": 78,
        # The one activity that plugs adapters into ports, and the core in its JNI shell.
        "rest": ("app", "core"),
        "skip": set(),
        "builds": ("surfaces/*/build.gradle.kts",),
        "banned_deps": {
            "the pictures port": r"^io\.coil-kt|^com\.github\.bumptech\.glide:|^com\.squareup\.picasso:",
            "the network port": r"^com\.squareup\.(okhttp3|okio|retrofit2):|^io\.ktor:",
            "the player port": r"^androidx\.media3:|^com\.google\.android\.exoplayer:",
            "the store port": r"^androidx\.(room|datastore|security|sqlite):",
            "a port, and the adapters and the core are not a surface's to link": r"^:(adapters|core)$",
        },
        # The one exemption, and it is named here rather than left out of the ban above, so that
        # deleting it is what closes this. Coil's Compose binding is still declared by the three
        # surface modules because 15 surface files draw through `AsyncImage`, and those files are
        # rewritten in Java anyway; what reaches the device -- the network fetcher and the decoders
        # -- moved to `:adapters` behind the `Pictures` port on 2026-11-12. This line goes when the
        # last `AsyncImage` does.
        "allowed_deps": frozenset({"io.coil-kt.coil3:coil-compose"}),
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bjava\.net\b|\bokhttp3\b|\bHttpURLConnection\b",
            "the secrets port": r"\bSharedPreferences\b",
            "the player port": r"\bandroidx\.media3\b|\bExoPlayer\b",
        },
    },
    "apple": {
        "code": "apps/apple",
        "ends": {".swift"},
        # `surfaces` at the family root is a link to one of these folders, so the pattern reads them
        # all: a phone, a television and the parts they share are each a surface.
        "surfaces": ("surfaces", "Sources/Surfaces*"),
        "least": 64,
        # The two app entry points, the package file, the core's host and the C bridge beside it,
        # and the tests, which are in no ring because they stand outside the app looking in.
        "rest": (
            "App",
            "PhoneApp",
            "Package.swift",
            "Sources/Contract",
            "Sources/CoreBridge",
            "Sources/CoreHost",
            "Tests",
        ),
        "skip": set(),
        "builds": (),
        "banned_deps": {},
        "allowed_deps": frozenset(),
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bURLSession\b",
            "the secrets port": r"\bSecItem\w*\b",
            "the player port": r"\bAVPlayer\b",
        },
    },
}
PARTS = ("adapters", "surfaces")

# A line's comment, so prose about a library is not mistaken for a declaration of one. It is read
# per line and knows nothing about quotes, so a `//` inside a string takes the rest of that line
# with it; a build file with a url and a dependency on one line would hide the dependency, and no
# build file here has one.
ASIDE = re.compile(r"//.*$")
# `group:artifact` or `group:artifact:version` inside quotes, which is how every build system this
# project uses names an outside library.
COORDINATE = re.compile(r'"([a-z][\w.\-]*:[\w.\-]+)(?::[^"]*)?"')
# A module of this same build, named the way Gradle names one.
NEARBY = re.compile(r'project\(\s*"(:[\w:.\-]+)"\s*\)')


def surface_folders(code: Path, family: dict) -> tuple[set[Path], list[str]]:
    """Every folder a family's `surfaces` patterns name, and a line per pattern that named none.

    A folder may be reached twice, once by its own name and once through a link to it, so they are
    gathered by the path they really have. A pattern matching nothing is the failure this check
    exists for: it means the surfaces moved and the read went on passing.
    """
    folders: set[Path] = set()
    blind = []
    for pattern in family["surfaces"]:
        matched = {folder.resolve() for folder in code.glob(pattern) if folder.is_dir()}
        if not matched:
            blind.append(
                f"`surfaces` names {pattern!r}, which is no folder of this family: the surfaces "
                "moved, and this read would pass without them"
            )
        folders |= matched
    return folders, blind


def surface_builds(code: Path, family: dict) -> tuple[list[Path], list[str]]:
    """Every build file a family's `builds` patterns name, and a line per pattern that named none.

    A pattern matching nothing is the same failure as a `surfaces` pattern matching nothing: the
    module moved and the read went on passing.
    """
    files: list[Path] = []
    blind = []
    for pattern in family["builds"]:
        matched = sorted(path for path in code.glob(pattern) if path.is_file())
        if not matched:
            blind.append(
                f"`builds` names {pattern!r}, which is no build file of this family: a surface "
                "module moved, and this read would pass without it"
            )
        files += matched
    return files, blind


def declared(path: Path) -> list[tuple[int, str]]:
    """Every outside library and nearby module a build file names, as (line, what it named)."""
    found = []
    for number, line in enumerate(path.read_text("utf-8").splitlines(), 1):
        bare = ASIDE.sub("", line)
        found += [(number, match.group(1)) for match in NEARBY.finditer(bare)]
        found += [(number, match.group(1)) for match in COORDINATE.finditer(bare)]
    return found


def dependencies(code: Path, family: dict) -> list[str]:
    """Every dependency a surface module declares that reaches the device behind a port."""
    files, blind = surface_builds(code, family)
    found = list(blind)
    for path in files:
        for number, named in declared(path):
            if named in family["allowed_deps"]:
                continue
            for port, words in family["banned_deps"].items():
                if re.search(words, named):
                    found.append(
                        f"{path.relative_to(ROOT)}:{number}: this module declares {named}, which "
                        f"puts it on this surface's classpath: reach it through {port}"
                    )
    return found


# Folders a build writes into. `tracked_code` below reads from git and so never sees these; this
# read walks the disk, because a family may reach its surfaces through a link that git does not
# follow, and so it has to refuse them by name. It went wrong the moment `.java` was admitted to the
# android family: two generated `BuildConfig.java` counted toward `least` and were scanned for
# banned words, which is a check reading its own build output and calling it the app.
BUILT = frozenset({"build", ".gradle", "target", "dist", "node_modules", "DerivedData", ".build"})

# A test is in no ring. It stands outside the app looking in, so it may name a port's own library and
# a page's own address without that being a surface reaching past a port - and a test that scans the
# surfaces for a banned word has to be able to write the word down. The apple family has always said
# this, by naming `Tests` in its `rest`; these are the folder names the other families use for the
# same thing, and without them the eleven preservation tests written on 2026-11-12 counted as
# surfaces and were scanned as if they drew something.
OUTSIDE = frozenset({"test", "androidTest", "testFixtures", "Tests"})


def surface_files(code: Path, family: dict) -> list[Path]:
    """Every surface file of a family, each one once, and nothing a build wrote."""
    folders, _ = surface_folders(code, family)
    files = {
        path.resolve()
        for folder in folders
        for path in folder.rglob("*")
        if path.is_file()
        and path.suffix in family["ends"]
        and path.name not in family["skip"]
        and not BUILT.intersection(path.relative_to(folder).parts[:-1])
        and not OUTSIDE.intersection(path.relative_to(folder).parts[:-1])
    }
    return sorted(files)


def tracked_code(code: Path, family: dict) -> list[Path]:
    """Every file of the family's language that git holds, by the path it really has.

    Read from git, so a build folder, a download or anything else this disk happens to hold is not
    mistaken for part of the app, and a file nobody committed cannot make a ring look full.
    """
    done = subprocess.run(
        ["git", "ls-files", "-z", "--", str(code.relative_to(ROOT))],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    files = []
    for name in done.stdout.split("\0"):
        if not name:
            continue
        path = ROOT / name
        if path.is_symlink() or not path.is_file() or path.suffix not in family["ends"]:
            continue
        if path.name in family["skip"]:
            continue
        files.append(path)
    return files


def strays(code: Path, family: dict, rings: set[Path]) -> list[str]:
    """Every file of the family that is in no ring and in no part the family declares."""
    found = []
    for path in tracked_code(code, family):
        real = path.resolve()
        if any(real.is_relative_to(ring) for ring in rings):
            continue
        rest = path.relative_to(code).as_posix()
        if any(rest == part or rest.startswith(part + "/") for part in family["rest"]):
            continue
        found.append(
            f"{path.relative_to(ROOT)}: in no ring of this family: put it under `surfaces`, "
            "`adapters` or a `ports` folder, add its folder to that family's `surfaces` patterns, "
            "or name it in `rest` because it is not a ring"
        )
    return found


def check(name: str, family: dict) -> list[str]:
    found = []
    code = ROOT / family["code"]
    ports = sorted(code.glob("ports*"))
    if not ports:
        found.append(f"{name}: no ports")
    for part in PARTS:
        if not (code / part).is_dir():
            found.append(f"{name}: no {part} folder")
    folders, blind = surface_folders(code, family)
    found += [f"{name}: {line}" for line in blind]
    # The folder at the family root is the one a person looks in, so it has to be one of the
    # folders really read, whether it is that folder or a link to it.
    read_surfaces = (code / "surfaces").resolve()
    if (code / "surfaces").is_dir() and read_surfaces not in folders:
        found.append(
            f"{name}: the surfaces folder at the family root is not in the read set: "
            f"`surfaces` names {family['surfaces']}, and none of them is {read_surfaces}"
        )
    rings = set(folders)
    # A family small enough keeps its ports in one file rather than a folder, so both count.
    rings |= {part.resolve() for part in ports}
    rings |= {(code / part).resolve() for part in PARTS if (code / part).is_dir()}
    # Every file of the family is in a ring or in a part the family says is not one. This is the
    # assertion `least` could never make: a new folder of screens is a failure here even when the
    # count of what was read goes up.
    found += [f"{name}: {line}" for line in strays(code, family, rings)]
    # What the surfaces DEPEND ON, which the banned-word read below cannot see at all.
    found += [f"{name}: {line}" for line in dependencies(code, family)]
    files = surface_files(code, family)
    # A floor under the read, with a margin: a deleted surface is bookkeeping, a surfaces folder
    # that went missing is already a failure above.
    if len(files) < family["least"]:
        found.append(
            f"{name}: read {len(files)} surface files, under the floor of {family['least']}: "
            f"if files really went, lower `least` to keep its margin"
        )
    for path in files:
        text = path.read_text("utf-8")
        for port, words in family["banned"].items():
            for match in re.finditer(words, text):
                line = text[: match.start()].count("\n") + 1
                found.append(f"{path.relative_to(ROOT)}:{line}: reach this through {port}: {match.group()}")
    return found


def main() -> int:
    found = [line for name, family in FAMILIES.items() for line in check(name, family)]
    # What was read, out loud, so a run that read a corner of a family cannot read as a pass.
    for name, family in FAMILIES.items():
        code = ROOT / family["code"]
        folders, _ = surface_folders(code, family)
        builds, _ = surface_builds(code, family)
        print(
            f"rings: {name} read {len(surface_files(code, family))} surface files in "
            f"{len(folders)} folders, {len(tracked_code(code, family))} files of the family in git, "
            f"{len(builds)} surface build files for what they depend on"
        )
    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
