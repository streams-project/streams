#!/usr/bin/env python3
"""Tests for tools/check_rings.py, against the blindness found on 2026-11-11.

The old check asserted a count and nothing else, so two things passed green: a `surfaces` pattern
that matched no folder at all, and a new folder of screens that no pattern named. Both are planted
here on the real tree, by handing `check` a family whose patterns are wrong, so no fixture tree is
needed and nothing is written anywhere.

Run from anywhere with: python3 tools/test_check_rings.py
"""

import re
import importlib.util
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location("check_rings", TOOLS / "check_rings.py")
rings = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rings)


def family(name, **changes):
    """The real family, with something changed, so a planted fault is one difference from truth."""
    one = dict(rings.FAMILIES[name])
    one.update(changes)
    return one


class Rings(unittest.TestCase):
    def test_every_family_is_in_its_rings_today(self):
        self.assertEqual(rings.main(), 0)

    def test_a_surfaces_pattern_that_names_no_folder_is_a_failure(self):
        folders, blind = rings.surface_folders(
            rings.ROOT / "apps/apple", family("apple", surfaces=("surfaces", "Sources/Screens*"))
        )
        self.assertTrue(blind)
        self.assertIn("Sources/Screens*", blind[0])
        found = rings.check("apple", family("apple", surfaces=("surfaces", "Sources/Screens*")))
        self.assertTrue(any("is no folder of this family" in line for line in found))

    def test_the_apple_blindness_of_2026_11_11_is_caught_now(self):
        # What shipped for weeks: only the folder at the family root was read, so the phone
        # surfaces were never scanned and the count still cleared its floor.
        found = rings.check("apple", family("apple", surfaces=("surfaces",)))
        strays = [line for line in found if "in no ring of this family" in line]
        self.assertTrue(strays)
        self.assertTrue(any("SurfacesPhone" in line for line in strays))

    def test_a_new_folder_of_screens_nobody_globbed_is_a_failure(self):
        # The same shape as the defect above, from the other end: a real folder of surface files
        # that the patterns do not name is a failure even though the count went up, not down.
        one = family("apple", surfaces=("Sources/SurfacesTV",), least=1)
        found = rings.check("apple", one)
        self.assertTrue(any("SurfacesShared" in line for line in found))
        self.assertTrue(any("in no ring of this family" in line for line in found))

    def test_the_surfaces_folder_at_the_root_must_be_in_the_read_set(self):
        found = rings.check("apple", family("apple", surfaces=("Sources/SurfacesPhone",)))
        self.assertTrue(
            any("surfaces folder at the family root is not in the read set" in line for line in found)
        )

    def test_every_floor_keeps_a_margin_under_the_real_count(self):
        # `least` equal to the count makes an ordinary deletion an architecture failure, which
        # invites lowering the number, which is how the first blindness was hidden.
        for name, one in rings.FAMILIES.items():
            with self.subTest(name=name):
                read = len(rings.surface_files(rings.ROOT / one["code"], one))
                self.assertGreater(read, one["least"], f"{name}: least has no margin")

    def test_the_floor_still_fires_on_a_read_of_almost_nothing(self):
        found = rings.check("web", family("web", least=1000))
        self.assertTrue(any("under the floor of 1000" in line for line in found))

    def test_the_family_files_are_read_from_git_and_not_from_this_disk(self):
        for name, one in rings.FAMILIES.items():
            with self.subTest(name=name):
                held = rings.tracked_code(rings.ROOT / one["code"], one)
                self.assertTrue(held)
                for path in held:
                    self.assertFalse(path.is_symlink())
                    self.assertIn(path.suffix, one["ends"])

    def test_every_family_with_code_in_apps_has_an_entry(self):
        """A family held by its own script alone is a family `./check` does not hold.

        On 2026-11-13 three families had code and no entry here: desktop, harmony and xbox. Two
        joined. `xbox` has no `ports` and no `adapters` folder -- its four ports are inside
        `host/core_host.h` and nothing fills them, because nothing draws yet -- and `check` refuses
        a family without both, so an entry for it today would be two failures rather than a rule.
        It is named here so that the day those two folders exist, this test is what says so.
        """
        families = {path.name for path in (rings.ROOT / "apps").iterdir() if path.is_dir()}
        held = set(rings.FAMILIES)
        # `roku` is a surface and no hexagon: `check_roku.py` type checks it and runs its suites,
        # and it has no `ports` or `adapters` folder to hold apart. `xbox` has neither either.
        elsewhere = {"roku", "xbox"}
        self.assertEqual(families - held - elsewhere, set(), "a family with no ring rule")
        for name in sorted(elsewhere & families):
            with self.subTest(name=name):
                code = rings.ROOT / "apps" / name
                self.assertFalse(
                    (code / "ports").is_dir() and (code / "adapters").is_dir(),
                    f"{name} now has both rings: give it an entry in FAMILIES",
                )

    def test_the_desktop_adapters_are_not_a_surfaces_to_name(self):
        """A one-crate family reaches past a port by naming the adapter that fills it.

        `lib.rs` exports `FileSecrets`, `FileStore`, `HttpNetwork` and `NoPlayer`, so those four
        names are what a surface would write if it went around the hexagon. Proved on the real tree
        on 2026-11-13 by planting each one in a surface file: every one was caught, by port name.
        """
        words = rings.FAMILIES["desktop"]["banned"]
        caught = lambda line: [
            port for port, pattern in words.items() if re.search(pattern, line)
        ]
        self.assertEqual(caught("fn planted() -> FileStore { todo!() }"), ["the store port"])
        self.assertEqual(caught("let keep = FileSecrets::new(at);"), ["the secrets port"])
        self.assertEqual(caught("let it = NoPlayer::default();"), ["the player port"])
        self.assertEqual(
            caught("use crate::adapters::HttpNetwork;"),
            ["the network port", "a port, and adapters are not for surfaces"],
        )
        self.assertEqual(caught("let scene = Scene::new(box);"), [])

    def test_the_harmony_ban_is_a_call_and_never_a_type_name(self):
        """ArkTS has no structural typing, so a cast has to write `image.PixelMap`.

        `surfaces/touch/README.md` settles this: four files import `@kit.ImageKit` for that type
        alone, and what reaches a device is the decode call. A ban on the import would have made
        the rule unaddable; a ban on the bare word would have caught the type.
        """
        words = rings.FAMILIES["harmony"]["banned"]
        caught = lambda line: [
            port for port, pattern in words.items() if re.search(pattern, line)
        ]
        self.assertEqual(
            caught("const made = image.createPixelMap({ desiredSize: box });"),
            ["the pictures port"],
        )
        self.assertEqual(caught("let shown: image.PixelMap = given as image.PixelMap;"), [])
        self.assertEqual(caught("import { image } from '@kit.ImageKit';"), [])
        self.assertEqual(
            caught("const reply = http.createHttp().request(where);"), ["the network port"]
        )
        self.assertEqual(caught("const go = '/browse?row=' + id;"), ["the address table"])
        self.assertEqual(caught('const go = "/browse";'), ["the address table"])

    def test_only_harmony_is_read_with_its_comments_blanked(self):
        """The weaker read is one family's, declared, and not a default that spread quietly.

        That folder compiles nowhere, so what it knows about HarmonyOS lives in its comments --
        including the names of the calls banned above, written there to explain the ban
        (`Pictures.ets:13`, `Pick.ets:63`). Every other family is read whole.
        """
        self.assertEqual(
            {name for name, one in rings.FAMILIES.items() if one["comments"]}, {"harmony"}
        )
        was = "a();\n// http.createHttp()\nb(); /* AVPlayer */ c();\n"
        blanked = rings.without_comments(was)
        self.assertNotIn("createHttp", blanked)
        self.assertNotIn("AVPlayer", blanked)
        self.assertEqual(blanked.count("\n"), 3)
        # Every line keeps its own length, so every line number in a finding is still the file's.
        self.assertEqual(len(blanked), len(was))
        for before, after in zip(was.splitlines(), blanked.splitlines()):
            self.assertEqual(len(before), len(after))
        # And a call on a code line is still a call.
        self.assertIn("http.createHttp", rings.without_comments("// why\nhttp.createHttp();\n"))

    def test_the_apple_picture_bypass_of_2026_11_12_is_caught_now(self):
        """`AsyncImage` fetches inside SwiftUI, so the `URLSession` ban never saw four draw sites.

        notes/216. The ban reads the capability now, and the one `Data(contentsOf:` the television
        surface really has - the app's own bundled opening geometry - is not one.
        """
        words = rings.FAMILIES["apple"]["banned"].values()
        caught = lambda line: any(re.search(pattern, line) for pattern in words)
        self.assertTrue(caught("AsyncImage(url: address) { phase in }"))
        self.assertTrue(caught("let task = session.dataTask(with: address) { data, _, _ in }"))
        self.assertTrue(caught("var request = URLRequest(url: url)"))
        self.assertTrue(caught("let bytes = try Data(contentsOf: address)"))
        self.assertFalse(
            caught('Data(contentsOf: Bundle.module.url(forResource: "opening", withExtension: "json")!)'),
            "the surface's own bundled asset is not a fetch",
        )


if __name__ == "__main__":
    unittest.main()
