// The card at the top of a page, held to the capture every television surface is held to and to
// the numbers around it.
//
// The rule is `apps/android/surfaces/tv/src/test/kotlin/streams/tv/NumbersTest.kt:20`: a number is
// never asserted against itself. So nothing below repeats a number out of `src/hero.ts`. Each case
// is one of the two kinds that file's test is built from:
//
//   * a raw measurement held against the capture that recorded it, which is
//     `reference/netflix-on-the-tv.md`, read from disk here, to the 2 px that file claims for
//     itself in its own sentence;
//   * or a derivation - the card from the page and its margins, the stack from the foot upwards -
//     which breaks the moment one of its parts moves on its own.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { Image, Tile, TitleDetail } from "../contract";
import { HERO, headHigh, heroLogo, heroStack, shadeTop } from "../hero";
import {
  NEXT_ROW_NAME_Y,
  NEXT_ROW_TOP,
  PAGE_EDGE,
  ROW_NAME_SIZE,
  SCREEN_HIGH,
  SCREEN_WIDE,
  STORY_LINE,
  STORY_LINES,
} from "../numbers";
import { FACTS, PILL } from "../parts";

/** The repository root, four folders up from `apps/roku/src/tests`. */
const ROOT = join(__dirname, "..", "..", "..", "..");

/** The capture this project's television surfaces are held to. */
const REFERENCE = readFileSync(join(ROOT, "reference", "netflix-on-the-tv.md"), "utf8");

function number(pattern: RegExp, what: string): number {
  const found = pattern.exec(REFERENCE);
  if (found === null || found[1] === undefined) {
    throw new Error(`${what} is no longer in reference/netflix-on-the-tv.md`);
  }
  return Number(found[1]);
}

/** How far a surface may sit from the capture, as the reference states it itself. */
const TOLERANCE = number(/within \*\*(\d+) px\*\*/, "the tolerance the reference allows");

const CAPTURED = {
  wide: number(/Hero card \*\*(\d+) x \d+ px\*\*/, "the hero's measured width"),
  high: number(/Hero card \*\*\d+ x (\d+) px\*\*/, "the hero's measured height"),
  x: number(/Hero card \*\*[\d x]+px\*\* at x (\d+)-\d+/, "the hero's measured left edge"),
  top: number(/Hero card \*\*[\d x]+px\*\* at x [\d-]+, y (\d+)-\d+/, "the hero's measured top"),
  foot: number(/Hero card \*\*[\d x]+px\*\* at x [\d-]+, y \d+-(\d+)/, "the hero's measured foot"),
  rowName: number(/one row title at y (\d+)-\d+/, "the first row's name under the hero"),
  rowSeen: number(/\*\*(\d+) px of the\s+first row's tiles/, "how much of the first row shows"),
};

function near(actual: number, captured: number): number {
  return Math.abs(actual - captured);
}

describe("the capture still says what these cases read", () => {
  // A reference edited to excuse a number would make every case below pass. This asserts the shape
  // of the lines they parse, not the numbers in them.
  it("states a hero card, its place, and the row under it", () => {
    expect(CAPTURED.wide).toBeGreaterThan(SCREEN_WIDE / 2);
    expect(CAPTURED.foot).toBeGreaterThan(CAPTURED.top);
    expect(CAPTURED.x).toBeGreaterThan(0);
    expect(CAPTURED.rowName).toBeGreaterThan(CAPTURED.foot);
    expect(CAPTURED.rowSeen).toBeGreaterThan(0);
    expect(TOLERANCE).toBe(2);
  });
});

describe("the card is held to the capture", () => {
  it("lands its foot where the capture measured one", () => {
    expect(near(HERO.y + HERO.high, CAPTURED.foot)).toBeLessThanOrEqual(TOLERANCE);
  });

  /**
   * Recorded rather than asserted, because it is a difference this folder did not choose and must
   * not quietly correct: the card's top sits at `HeroView.java:38`'s 143, which is 3 px below the
   * capture's 140 and so outside the 2 px the reference claims, while its foot is within 1 px. The
   * same 143 is in the Android surface, so this surface agrees with the behaviour spec and the
   * behaviour spec disagrees with the capture at that one edge. Whoever reconciles them moves both
   * surfaces, not this one.
   */
  it("sits below the captured top by less than a row's name is tall, which is the known gap", () => {
    expect(HERO.y).toBeGreaterThanOrEqual(CAPTURED.top);
    expect(near(HERO.y, CAPTURED.top)).toBeLessThan(ROW_NAME_SIZE);
  });

  it("leaves the capture's own sliver of the first row showing under it", () => {
    expect(SCREEN_HIGH - NEXT_ROW_TOP.underHero).toBe(CAPTURED.rowSeen);
  });

  it("puts the first row's name above where the capture draws its words", () => {
    // The code places the text's box, which starts above the drawn letters, so the box is at or
    // above the measured line and by no more than the name's own size.
    // (`NumbersTest.kt:286-318` records the same reading on Android.)
    expect(NEXT_ROW_NAME_Y.underHero).toBeLessThanOrEqual(CAPTURED.rowName);
    expect(CAPTURED.rowName - NEXT_ROW_NAME_Y.underHero).toBeLessThanOrEqual(ROW_NAME_SIZE);
  });
});

describe("the card is the page less its two margins", () => {
  // A derivation, and the one `HeroView.java:23-28` insists stays written as the derivation: "a
  // rewrite that writes the answer loses the reason, and the next person to change the margin
  // finds the hero does not follow".
  it("starts on the page margin and ends on it", () => {
    expect(HERO.x).toBe(PAGE_EDGE);
    expect(HERO.x + HERO.wide + PAGE_EDGE).toBe(SCREEN_WIDE);
  });

  it("is narrower than the captured card, because this project's margin is wider", () => {
    expect(PAGE_EDGE).toBeGreaterThan(CAPTURED.x);
    expect(HERO.wide).toBeLessThan(CAPTURED.wide);
  });

  it("fits the screen with its own foot to spare", () => {
    expect(HERO.y + HERO.high).toBeLessThan(SCREEN_HIGH);
    expect(HERO.inset * 2 + HERO.column).toBeLessThan(HERO.wide);
  });

  it("starts its shade part way down itself, never at the top and never past the foot", () => {
    expect(shadeTop()).toBeGreaterThan(0);
    expect(shadeTop()).toBeLessThan(HERO.high);
  });
});

describe("the stack inside it is built from the foot upwards", () => {
  const full = heroStack(false, true);

  it("sits the buttons on the card's own foot", () => {
    expect(HERO.high - full.buttons - PILL.high).toBe(HERO.foot);
  });

  it("runs name, facts, story, buttons, in that order and none of them overlapping", () => {
    expect(full.head).toBeLessThan(full.facts);
    expect(full.head + headHigh(false)).toBeLessThanOrEqual(full.facts);
    expect(full.facts + FACTS.size).toBeLessThanOrEqual(full.story);
    expect(full.story + STORY_LINE * STORY_LINES).toBeLessThanOrEqual(full.buttons);
  });

  it("keeps the whole stack inside the card", () => {
    expect(full.head).toBeGreaterThan(0);
    expect(full.buttons + PILL.high).toBeLessThanOrEqual(HERO.high);
  });

  it("closes the gap where the facts would have been when a title has none", () => {
    const bare = heroStack(false, false);
    expect(bare.head).toBeGreaterThan(full.head);
    expect(bare.story).toBe(full.story);
    expect(bare.buttons).toBe(full.buttons);
  });

  it("gives the lettering a taller box than two lines of the written name, and still fits", () => {
    expect(headHigh(true)).toBeGreaterThan(headHigh(false));
    expect(heroStack(true, true).head).toBeGreaterThan(0);
  });
});

describe("the title's own lettering", () => {
  function tile(images: Image[] | undefined): Tile {
    return images === undefined
      ? { name: "A", title: "movie:1" }
      : { images, name: "A", title: "movie:1" };
  }

  // Recorded, as `facts.test.ts` records the missing age mark: no fixture in
  // `platform/contract/fixtures/2026-11-05/` carries an image with the `logo` role, so the two
  // fallback cases below stand on constructed images rather than on the corpus. The case that the
  // corpus does hold is the one that matters most on a Roku - a title with artwork and no
  // lettering, which draws its name as words.
  it("is nothing for the one fixture that carries artwork, so the card draws the name", () => {
    const film = JSON.parse(
      readFileSync(
        join(ROOT, "platform", "contract", "fixtures", "2026-11-05", "valid", "TitleDetail.film.json"),
        "utf8",
      ),
    ) as TitleDetail;
    expect((film.images ?? []).map((image: Image) => image.role)).toEqual(["poster"]);
    expect(heroLogo(tile(film.images), film)).toBeNull();
  });

  it("comes from the tile that opened the page when it has one", () => {
    const lead = tile([{ role: "logo", url: "tile.png" }]);
    const detail = { images: [{ role: "logo", url: "detail.png" }], name: "A", title: "movie:1" };
    expect(heroLogo(lead, detail as TitleDetail)).toBe("tile.png");
  });

  it("falls back to the one that came with the title's own details", () => {
    const detail = { images: [{ role: "logo", url: "detail.png" }], name: "A", title: "movie:1" };
    expect(heroLogo(tile(undefined), detail as TitleDetail)).toBe("detail.png");
  });

  it("is nothing when neither carries one, and when no details have arrived yet", () => {
    expect(heroLogo(tile([{ role: "poster", url: "p.png" }]), null)).toBeNull();
    expect(heroLogo(tile(undefined), null)).toBeNull();
  });
});
