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
  no strays   every code file the family has, in git OR on this disk and not ignored, is inside a
              ring, or inside a folder the family declares as `rest`, which is the part of the app
              that is not a ring. The disk half was added on 2026-11-13: reading the index alone
              meant a new folder of screens was invisible until somebody committed it, which is the
              one moment its author has stopped looking. `loose.py` holds the whole of that
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
sys.path.insert(0, str(Path(__file__).resolve().parent))

from loose import loose  # noqa: E402

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
    # measurements were rehoused in `reference/netflix-on-the-tv.md` on 2026-11-12, where the
    # families that cite them can read them. There is no family here because there is no code here.
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
        "comments": False,
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bfetch_with|\bRequestInit\b|\breqwest\b",
            "the secrets port": r"\blocal_storage\b|\bsession_storage\b",
            "the address port": r"\bhistory\(\)|\bpush_state",
            "the player port": r"\bwebPlayer\b|\bHtmlVideoElement\b",
            "a port, and adapters are not for surfaces": r"crate::adapters",
        },
    },
    # Written on 2026-11-12 and 2026-11-13 and held by nothing here until today: this family paints
    # nothing, it lays a screen out into a `Scene` of rectangles, words and brand roles, and a
    # painter of forty lines turns a scene into a window. That is exactly the shape this rule is
    # for -- the surfaces may name no system thing at all -- and the banned words are the four
    # adapter type names `lib.rs` exports, because in a one-crate family reaching past a port means
    # naming the adapter that fills it.
    "desktop": {
        "code": "apps/desktop/src",
        "ends": {".rs"},
        "surfaces": ("surfaces",),
        "least": 4,
        # `adapters` and `ports.rs` are rings and so are not named here. What is left of the family
        # is the host that steps the core and the crate root that wires the three together.
        "rest": ("host.rs", "lib.rs"),
        "skip": set(),
        "builds": (),
        "banned_deps": {},
        "allowed_deps": frozenset(),
        # Comments are read here, as they are for every family but `harmony`: the four names below
        # are this crate's own types and a surface has no reason to write one in prose either.
        "comments": False,
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\breqwest\b|\bTcpStream\b|\bstd::net\b|\bHttpNetwork\b",
            "the secrets port": r"\bFileSecrets\b",
            "the store port": r"\bFileStore\b",
            "the player port": r"\bNoPlayer\b|\blibmpv\b",
            "a port, and adapters are not for surfaces": r"crate::adapters|super::adapters",
        },
    },
    # HarmonyOS: ArkTS that no compiler on any machine here can read. `touch_check.py` holds the
    # same list against the same files with comments blanked and reports it file by file; this is
    # the family's entry in the hexagon rule, and it is where the list is now written once --
    # `touch_check.py` reads it from here rather than keeping a second copy that can drift.
    #
    # What a surface may not name is a CALL, not an import: `@kit.ImageKit` is imported by four
    # files in `surfaces/touch` for the type name `image.PixelMap`, which ArkTS requires because it
    # has no structural typing, and a type name has to be writable. `surfaces/touch/README.md`
    # states that and this list follows it.
    "harmony": {
        "code": "apps/harmony",
        # `.c` and `.h` are in here with no ring of this family holding one, and on purpose: the C
        # glue is under `core`, which is `rest`, and the day somebody drops a `.c` into a surface
        # folder it is a file this read sees rather than one it passes over on its suffix. That is
        # the android family's `.java` lesson taken before the fact rather than after it.
        "ends": {".ets", ".ts", ".c", ".h"},
        "surfaces": ("surfaces",),
        "least": 20,
        # The core in its NAPI shell with the C bridge under it, and the contract read into ArkTS.
        "rest": ("core", "contract"),
        "skip": set(),
        "builds": (),
        "banned_deps": {},
        "allowed_deps": frozenset(),
        # The one family whose comments are blanked before the read, and it is not a convenience:
        # this folder compiles nowhere, so what it knows about HarmonyOS is written in its comments,
        # including the names of the very calls banned below. `Pictures.ets:13` says in prose that
        # nothing here calls `image.createImageSource`, and `Pick.ets:63` quotes "/browse" to
        # explain why no screen may spell an address. Both are the rule being explained, and a rule
        # that cannot be explained in the file it holds is a rule people route around.
        # What it costs, named rather than left silent: the blanking is by regular expression and
        # knows nothing about quotes, so a `//` inside a string takes the rest of that line with
        # it. No file in this folder has one, and `touch_check.py` has read the same text the same
        # way since it was written.
        "comments": True,
        "banned": {
            "the address table": r'["\']/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)["\'/?]',
            "the network port": r"\bhttp\.createHttp\b|\bcreateHttp\(|@ohos\.net\.|@kit\.NetworkKit",
            "the secrets port": r"\basset\.(add|query|remove|update)\b|@kit\.AssetStoreKit",
            "the store port": r"\bfileIo\.|@kit\.CoreFileKit|@ohos\.file\.fs",
            "the player port": r"\bAVPlayer\b|createAVPlayer|createMediaSourceWithUrl",
            # A call and not the type: `createImageSource(` and `createPixelMap(` decode bytes,
            # which is the adapter's work, and `image.PixelMap` is a name a cast has to write.
            "the pictures port": r"createImageSource\s*\(|createPixelMap\s*\(",
        },
    },
    # Added 2026-11-14, the day this family stopped being screens with no program. Until then its
    # adapters were one file and the line above demanded a folder, so `roku` could not have an entry
    # here at all -- while the root README said of every row that "`./check` holds the family to its
    # ports with `check_rings.py`, whether or not it can build that family". Nothing stopped a Roku
    # screen calling `fetch` directly, in the one family whose TypeScript no compiler on this
    # machine has ever read.
    #
    # Its surfaces are `screens`, not `surfaces`: that is the word this platform's own documents use
    # and the folder a Roku developer looks in. The entry names it rather than the folder being
    # renamed to suit the check.
    "roku": {
        "code": "apps/roku/src",
        # `.tsx` as well as `.ts`, taken before the fact rather than after it. The android entry
        # above records what the other order cost: `.kt` alone left every `.java` file in no ring
        # and skipped by both reads, so the hexagon quietly stopped being enforced for it. Every
        # screen here is `.tsx` and one (`remote.ts`) is not, so both suffixes are the family.
        "ends": {".ts", ".tsx"},
        "surfaces": ("screens",),
        # 8 screen files today. The floor keeps a margin, like every other family's.
        "least": 6,
        # Everything of this family that is not a ring. `ports.ts` and `adapters.ts` are rings and
        # so are not named. `host.ts` is the one object that plugs adapters into ports; the rest are
        # the shared reading, geometry and words that screens are allowed to use.
        "rest": (
            "api.ts",
            "art.ts",
            "badges.ts",
            "contract.ts",
            "door.ts",
            "facts.ts",
            "focus.ts",
            "growth.ts",
            "hero.ts",
            "host.ts",
            "icons.ts",
            "keys.ts",
            "lost.ts",
            "main.tsx",
            "numbers.ts",
            "page.ts",
            "parts.ts",
            "route.ts",
            "tests",
            "tier.ts",
            "title.ts",
            "type.ts",
            "words.ts",
        ),
        "skip": set(),
        "builds": (),
        "banned_deps": {},
        "allowed_deps": frozenset(),
        # Read whole, comments included, like every family but `harmony`. The words below appear in
        # `adapters.ts` prose explaining why each one is that ring's business, and `adapters.ts` is
        # a ring, so the surfaces read never reaches it.
        "comments": False,
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            "the network port": r"\bfetch\(|\broUrlTransfer\b",
            "the store port": r"\broRegistry\b",
            "the clock port": r"\bsetTimeout\b|\bsetInterval\b",
            "the launch port": r"\broAppInfo\b|\bgetLaunchParam",
            "a port, and adapters are not for screens": r'from "\.\./adapters"|from "\./adapters"',
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
        "comments": False,
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
        "comments": False,
        "banned": {
            "the address table": r'"/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]',
            # Capability, not spelling. Until 2026-11-13 this banned the word URLSession alone,
            # and note 216's four `AsyncImage` sites reached the network straight past it for as
            # long as they existed. Proved on 2026-11-13 by putting one back: the check passed
            # green on it. The `Data(contentsOf:` lookahead lets a bundled read through --
            # `SurfacesTV/app/Opening.swift` loads its own geometry that way, which is a file and
            # not a network. 0 false positives measured across the whole apple tree.
            "the pictures port": r"\bAsyncImage\b",
            "the network port": r"\bURLSession\b|\bURLRequest\b|\bdataTask\b|\bData\(contentsOf:(?!\s*Bundle\.)",
            "the secrets port": r"\bSecItem\w*\b",
            # Capability, not spelling. `\bAVPlayer\b` needs a non-word character after the
            # name, so AVQueuePlayer, AVPlayerItem, AVPlayerLayer, AVPlayerLooper,
            # AVPlayerViewController, AVAudioPlayer, AVSampleBufferDisplayLayer and SwiftUI's
            # VideoPlayer all reached the device straight past it. Proved on 2026-11-13 by
            # planting a looping sound written the way this family's own adapter writes one
            # (AVQueuePlayer + AVPlayerLooper + AVPlayerItem): the old rule passed it green.
            "the player port": r"\bAV\w*Player\w*\b|\bAVAudioEngine\b"
            r"|\bAVSampleBufferDisplayLayer\b|\bimport\s+AV(Foundation|Kit)\b|\bVideoPlayer\b",
            # The android family has banned `SharedPreferences` since this check existed and the
            # apple family banned nothing, so a surface could put a session token or a page's
            # state in `UserDefaults` and pass. A missing rule rather than a spelling; it reports
            # 0 findings on today's tree, so it goes in clean.
            "the store port": r"\bUserDefaults\b|\bNSUbiquitousKeyValueStore\b|\bFileManager\b",
        },
    },
}
PARTS = ("adapters", "surfaces")

# A `/* */` or `//` comment, blanked and not deleted so every line number stays what it was. Only a
# family that declares `"comments": True` is read this way; the rest are read whole, where a banned
# word in a comment is a finding like any other. `harmony` is the one family that needs it and the
# entry above says why.
A_BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)
A_LINE_COMMENT = re.compile(r"//[^\n]*")


def without_comments(text: str) -> str:
    """The same text, the same length, with every comment turned to spaces."""
    text = A_BLOCK_COMMENT.sub(lambda found: re.sub(r"[^\n]", " ", found.group(0)), text)
    return A_LINE_COMMENT.sub(lambda found: " " * len(found.group(0)), text)

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


def loose_code(code: Path, family: dict) -> list[Path]:
    """Every file of the family's language that is on this disk and NOT in git.

    The hole this closes, 2026-11-13: `strays` read the index alone, so a new file of screens that
    nobody had committed yet was in no ring and nothing said so. It went red on the commit that
    carried it, which is the one moment its author is no longer watching. `loose.py` holds why this
    answer was taken rather than refusing to run at all while anything is uncommitted; the escape is
    the same one `check_tracked.py` names, `.gitignore` or `.git/info/exclude`.

    A build folder is not here either way: git ignores those, so `loose` never reports one, and
    `BUILT` names them again in case a family's own ignore rules ever stop covering one.
    """
    files = []
    for name in loose(ROOT, str(code.relative_to(ROOT))):
        path = ROOT / name
        if path.is_symlink() or not path.is_file() or path.suffix not in family["ends"]:
            continue
        if path.name in family["skip"] or BUILT.intersection(Path(name).parts[:-1]):
            continue
        files.append(path)
    return files


def strays(code: Path, family: dict, rings: set[Path]) -> list[str]:
    """Every file of the family that is in no ring and in no part the family declares.

    Read from the index AND from the disk: a file git does not hold yet is held to the hexagon the
    day it is written, not the day it is added.
    """
    found = []
    for path in tracked_code(code, family) + loose_code(code, family):
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
    # A ring is a folder in a large family and one file in a small one. `ports` has been read both
    # ways since this check was written -- the `glob("ports*")` above, with a comment below saying
    # why -- and `adapters` was read only as a folder. So a family whose adapters are one file,
    # which is the honest shape for a surface of a few thousand lines, could not have an entry here
    # at all and was therefore held by nothing.
    #
    # `roku` was in exactly that position on 2026-11-14, with `ports.ts` and `adapters.ts` as
    # files, while the root README said of every family that "`./check` holds the family to its
    # ports with `platform/tools/check_rings.py`, whether or not it can build that family". That
    # sentence was false for two of the eight rows, and the inconsistency between these two lines
    # is why. Globbed now, for the reason the ports line already gave.
    adapters = sorted(code.glob("adapters*"))
    if not adapters:
        found.append(f"{name}: no adapters")
    # `surfaces` is deliberately NOT globbed here. A family names its own surface folders in its
    # entry and they are not always called that -- `roku` draws in `screens` -- so the name to
    # check is the family's, not this literal. What enforces their existence is `surface_folders`
    # below, which reports a declared folder that is not there, and `least`.
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
    rings |= {part.resolve() for part in adapters}
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
        if family["comments"]:
            text = without_comments(text)
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
            f"{len(folders)} folders, {len(tracked_code(code, family))} files of the family in git "
            f"and {len(loose_code(code, family))} on this disk git does not hold, "
            f"{len(builds)} surface build files for what they depend on"
        )
    for line in found:
        print(line)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
