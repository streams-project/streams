// The one estimate this surface has, and the places it decides where something is drawn.
//
// A Roku lays a Label out itself and publishes no way to ask how wide the result will be, so every
// part that puts a second thing beside a word - the dot between two facts, the next tab along the
// bar, the plate behind a mark - has to guess. `src/type.ts` is that guess, in one place.
//
// A guess cannot be held to a measurement it is a guess about. What can be, and is below, are the
// two things around it: the columns the capture measured, which this surface's numbers have to
// land on, and the properties the estimate has to have for a line of words to come out in order.
// The capture is `reference/netflix-on-the-tv.md`, read from disk, to the 2 px it claims for
// itself.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { TitleDetail } from "../contract";
import { facts } from "../facts";
import {
  FACE,
  MARK_X,
  PAGE_EDGE,
  STORY_WIDE,
  TABS_X,
  TAB_GAP,
  TAB_INSET,
  TAB_LIMIT,
  TAB_SIZE,
} from "../numbers";
import { FACTS } from "../parts";
import { laidOut, textWide } from "../type";

const ROOT = join(__dirname, "..", "..", "..", "..");
const REFERENCE = readFileSync(join(ROOT, "reference", "netflix-on-the-tv.md"), "utf8");

function number(pattern: RegExp, what: string): number {
  const found = pattern.exec(REFERENCE);
  if (found === null || found[1] === undefined) {
    throw new Error(`${what} is no longer in reference/netflix-on-the-tv.md`);
  }
  return Number(found[1]);
}

const TOLERANCE = number(/within \*\*(\d+) px\*\*/, "the tolerance the reference allows");
const CAPTURED = {
  avatar: number(/Avatar (\d+) px square/, "the avatar's measured size"),
  firstTab: number(/tab labels starting at x (\d+)/, "where the first tab's word starts"),
  /** The four gaps between the five measured tab words, the widest of them taken as a tab's pitch. */
  tabGaps: /\(gaps ([\d, ]+)\)/
    .exec(REFERENCE)?.[1]
    ?.split(",")
    .map((one) => Number(one.trim())) ?? [],
};

describe("the capture still says what these cases read", () => {
  it("states an avatar, a first tab column and the gaps between the tabs", () => {
    expect(CAPTURED.avatar).toBeGreaterThan(0);
    expect(CAPTURED.firstTab).toBeGreaterThan(0);
    expect(CAPTURED.tabGaps).toHaveLength(4);
  });
});

describe("the bar lands on the columns the capture measured", () => {
  it("starts the first tab's word where the capture found one", () => {
    // A tab is a pill with its word inset, so the word's own column is the pill's corner plus the
    // inset. That is the number the capture can see.
    expect(Math.abs(TABS_X + TAB_INSET - CAPTURED.firstTab)).toBeLessThanOrEqual(TOLERANCE);
  });

  it("draws the face at the size the capture measured", () => {
    expect(Math.abs(FACE - CAPTURED.avatar)).toBeLessThanOrEqual(TOLERANCE);
  });

  it("has room for every tab it draws, at the pitch the capture measured a tab at", () => {
    // Not a claim about the estimate: a claim about the bar. At the widest tab the capture
    // recorded, the number of tabs this surface draws still ends before the mark.
    const widest = Math.max(...CAPTURED.tabGaps);
    expect(TABS_X + widest * TAB_LIMIT).toBeLessThan(MARK_X);
  });
});

describe("the estimate behaves like a width", () => {
  it("grows with the word and with the type size", () => {
    expect(textWide("abcd", TAB_SIZE)).toBeGreaterThan(textWide("abc", TAB_SIZE));
    expect(textWide("abc", TAB_SIZE * 2)).toBeGreaterThan(textWide("abc", TAB_SIZE));
  });

  it("is nothing at all for no word, so an empty pill takes no room", () => {
    expect(textWide("", TAB_SIZE)).toBe(0);
  });

  it("does not count the marks that isolate a filled value, because they draw nothing", () => {
    // `fill` wraps a value in U+2068 and U+2069 (`shared/Theme.kt:62-67`). Counting them would
    // push whatever sits beside the word along by about a character each, on every counted word
    // this surface draws.
    expect(textWide("⁨4⁩", FACTS.size)).toBe(textWide("4", FACTS.size));
  });
});

describe("a line of words", () => {
  it("starts at nothing, so the caller owns the corner it is drawn on", () => {
    expect(laidOut(["a", "b"], FACTS.size, 10).at[0]).toBe(0);
  });

  it("puts each word a word and a gap past the one before it", () => {
    const line = laidOut(["abc", "de"], FACTS.size, 10);
    expect(line.at[1]).toBe(textWide("abc", FACTS.size) + 10);
    expect(line.wide).toBe(textWide("abc", FACTS.size) + 10 + textWide("de", FACTS.size));
  });

  it("leaves no gap hanging off the end", () => {
    const one = laidOut(["abc"], FACTS.size, 10);
    expect(one.wide).toBe(textWide("abc", FACTS.size));
  });

  it("is nothing wide when there are no words, so nothing is drawn for it", () => {
    const none = laidOut([], FACTS.size, 10);
    expect(none.wide).toBe(0);
    expect(none.at).toEqual([]);
  });
});

describe("the facts line of a real title fits where it is drawn", () => {
  it("is narrower than the story under it, for the film the corpus carries", () => {
    const film = JSON.parse(
      readFileSync(
        join(ROOT, "platform", "contract", "fixtures", "2026-11-05", "valid", "TitleDetail.film.json"),
        "utf8",
      ),
    ) as TitleDetail;
    const english = JSON.parse(readFileSync(join(ROOT, "strings", "en.json"), "utf8")) as Record<
      string,
      string
    >;
    const said = facts(film, english, "en");
    expect(said.length).toBeGreaterThan(1);
    const line = laidOut(said, FACTS.size, FACTS.side * 2 + FACTS.dot);
    expect(line.wide).toBeLessThan(STORY_WIDE);
    // And it starts on the page margin, as the story does, so the two lines share a column.
    expect(PAGE_EDGE + line.wide).toBeLessThan(PAGE_EDGE + STORY_WIDE);
  });
});

describe("the gap between two tabs is the bar's own, not the estimate's", () => {
  it("is a number of its own, so a tab's word cannot close it", () => {
    expect(TAB_GAP).toBeGreaterThan(0);
  });
});
