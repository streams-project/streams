// The marks on a tile, held against the tiles the contract's own fixtures carry and the words the
// server sends.
//
// Nothing below invents a badge. Every one is read out of
// `platform/contract/fixtures/2026-11-05/`, the corpus `platform/contract/tests/fixtures.rs` loads
// with `fs::read_dir` and round-trips through the Rust contract types, and whose first line says
// "Fixtures are plain JSON and every language reads the same files". The words come from
// `strings/`, the files the server hands out at `/v1/strings/{lang}`
// (`platform/contract/src/api.rs:183`).
//
// The expected words are derived from `parts/Badge.kt:23-30` and the language file, never from
// running this code and writing down what it said.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { badgeWord, badgeWords } from "../badges";
import type { Badge, Screen, Section, Tile } from "../contract";

/** The repository root, four folders up from `apps/roku/src/tests`. */
const ROOT = join(__dirname, "..", "..", "..", "..");

const CORPUS = join(ROOT, "platform", "contract", "fixtures", "2026-11-05");

function read(kind: string, name: string): unknown {
  return JSON.parse(readFileSync(join(CORPUS, kind, name), "utf8"));
}

function strings(language: string): Record<string, string> {
  return JSON.parse(readFileSync(join(ROOT, "strings", `${language}.json`), "utf8")) as Record<
    string,
    string
  >;
}

const en = strings("en");

/** Every tile of a built page, in the order the page carries them. */
function tilesOf(screen: Screen): Tile[] {
  const all: Tile[] = [];
  for (const section of screen.sections) for (const tile of section.tiles ?? []) all.push(tile);
  return all;
}

describe("the corpus holds the badges these cases read", () => {
  // A fixture that quietly lost its badges would make every case below pass by drawing nothing.
  it("Screen.home carries one new mark and one ranked mark with a rank on it", () => {
    const home = read("valid", "Screen.home.json") as Screen;
    const marks = tilesOf(home).flatMap((tile) => tile.badges ?? []);
    expect(marks.map((one) => one.kind)).toEqual(["new", "new", "ranked"]);
    expect(marks[2]?.rank).toBe(2);
  });

  it("Tile.series_in_progress carries a ranked mark", () => {
    const tile = read("valid", "Tile.series_in_progress.json") as Tile;
    expect((tile.badges ?? [])[0]?.kind).toBe("ranked");
  });
});

describe("badgeWord", () => {
  it("gives the new mark the word the language file holds for it", () => {
    const home = read("valid", "Screen.home.json") as Screen;
    const mark = (tilesOf(home)[0]?.badges ?? [])[0] as Badge;
    expect(badgeWord(mark, en)).toBe(en["badge.new"]);
  });

  it("fills the rank into the ranked word, isolated, so the number keeps its own order", () => {
    const tile = read("valid", "Tile.series_in_progress.json") as Tile;
    const mark = (tile.badges ?? [])[0] as Badge;
    const said = badgeWord(mark, en);
    expect(said).toContain("2");
    // U+2068 and U+2069: `fill` isolates a value so a right to left line does not reorder it
    // (`shared/Theme.kt:62-67`).
    expect(said).toContain("⁨");
    expect(said).toContain("⁩");
    // The rank is drawn inside the language's own word and never on its own.
    expect(said.replace("⁨", "").replace("⁩", "")).toBe(
      (en["badge.ranked"] ?? "").replace("{rank}", "2"),
    );
  });

  it("says nothing for a ranked mark with no rank behind it, rather than a word with a hole", () => {
    expect(badgeWord({ kind: "ranked" }, en)).toBe("");
  });

  it("says nothing for a kind this build has never heard of", () => {
    // The tolerated corpus carries a section whose kind is from the future; the same tolerance
    // applies to a badge, because `BadgeKind` ends in `(string & {})`
    // (`platform/contract/README.md:41`).
    expect(badgeWord({ kind: "kind_from_the_future" }, en)).toBe("");
  });

  it("says nothing at all when the language has not got the word yet", () => {
    expect(badgeWord({ kind: "new" }, {})).toBe("");
  });
});

describe("badgeWords", () => {
  it("keeps the marks a page's row carries, in the order the row carries them", () => {
    const home = read("valid", "Screen.home.json") as Screen;
    const row = home.sections.find((section: Section) => section.kind === "row");
    const tiles = row?.tiles ?? [];
    expect(badgeWords(tiles[0]?.badges, en)).toEqual([en["badge.new"]]);
    expect(badgeWords(tiles[1]?.badges, en)).toHaveLength(1);
  });

  it("gives a tile with no marks an empty list, so no plate is drawn on its artwork", () => {
    expect(badgeWords(undefined, en)).toEqual([]);
    expect(badgeWords([], en)).toEqual([]);
  });

  it("drops a mark that carries no word rather than drawing an empty plate", () => {
    expect(badgeWords([{ kind: "ranked" }, { kind: "new" }], en)).toEqual([en["badge.new"]]);
  });
});

describe("every language this project ships can say both marks", () => {
  // The two words are drawn over artwork on every page of rows, so a language missing one draws an
  // empty plate on a poster rather than nothing at all. `check_strings.py` holds every language to
  // `en.json`'s keys; this holds the two this surface draws, through the code that draws them.
  const languages = ["ar", "de", "en", "es", "fa", "fr", "he", "hi", "it", "ja", "ko", "nl", "pl", "pt", "ru", "sv", "tr", "zh"];

  it("has a word for a new title", () => {
    for (const language of languages) {
      expect(badgeWord({ kind: "new" }, strings(language))).not.toBe("");
    }
  });

  it("has a ranked word that puts the rank inside it", () => {
    for (const language of languages) {
      const said = badgeWord({ kind: "ranked", rank: 3 }, strings(language));
      expect(said).toContain("3");
      expect(said).not.toBe("");
    }
  });
});
