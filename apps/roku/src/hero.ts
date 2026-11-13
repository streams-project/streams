// The card at the top of a page: where it sits, and what goes in it.
//
// A port of `apps/android/surfaces/tv/src/main/java/streams/tv/browse/HeroView.java:36-127`, the
// view that draws the hero today, and of `tv/src/main/kotlin/streams/tv/browse/HeroCard.kt:46-99`,
// the behaviour spec it cites line by line. The drawing is `src/screens/Hero.tsx`; this is the part
// that can be read and checked without a Roku, and `src/tests/hero.test.ts` is that check.
//
// **Why this exists at all.** `heroOn` (`src/page.ts:61`) has decided which title opens a page
// since the day that file was written, and `PageFocus` has had a `hero` place and two buttons on it
// since `src/focus.ts` was written. Nothing drew either. A page whose first answer carries a hero
// opens with the remote on it (`focus.ts:65`), so until today the first press on most pages moved a
// focus nobody could see.
//
// **Where the numbers belong.** `src/numbers.ts`, with every other number this surface lays out
// with. That file was another agent's for this pass, so the hero's own four sit here instead, each
// beside the line it came from, and no number is written twice: the width is the page less its two
// margins rather than the answer, as `HeroView.java:23-28` insists it stays written, and the
// logo box, the name's size and the story's three lines are imported from `src/numbers.ts` rather
// than restated.

import { logo } from "./art";
import type { Tile, TitleDetail } from "./contract";
import {
  PAGE_EDGE,
  SCREEN_WIDE,
  STORY_LINE,
  STORY_LINES,
  TITLE_LOGO,
  TITLE_NAME_SIZE,
} from "./numbers";
import { FACTS, PILL } from "./parts";

/**
 * The card itself.
 *
 * `wide` is written as the derivation and not as the answer, because `HeroView.java:23-28` says
 * why: "a rewrite that writes the answer loses the reason, and the next person to change the margin
 * finds the hero does not follow". The reference measured Netflix's own at 1790 x 772 at x 67
 * (`reference/netflix-on-the-tv.md:40`); this project's margin is 72, decided in note 188, so the
 * card is narrower by that decision and its foot lands on the same line. `src/tests/hero.test.ts`
 * holds both halves of that sentence.
 */
export const HERO = {
  x: PAGE_EDGE,
  /** `HeroView.java:38`, citing `HeroCard.kt:53`. */
  y: 143,
  wide: SCREEN_WIDE - PAGE_EDGE * 2,
  /** `HeroView.java:41`, citing `HeroCard.kt:54`. */
  high: 767,
  /** The words inside it sit on this column and this foot. (`HeroView.java:44-50`) */
  inset: 30,
  foot: 34,
  column: 760,
  /** The white edge the card is drawn with, at a quarter strength. (`HeroView.java:70`) */
  edge: 2,
  edgeShare: 28,
};

/**
 * The shade up the card: where it starts, as a share of the card's height, and how black it ends,
 * in hundredths. (`HeroView.java:85-86`)
 *
 * Two numbers that happen to read the same in the Kotlin - the stop is 0.55 and the strength is
 * `Ink.black(55)` - and are kept apart here, because moving one has nothing to do with the other.
 */
export const SHADE = { from: 55, strength: 55 };

/** Where that shade begins, down the card. A derivation, so it follows the card's height. */
export function shadeTop(): number {
  return Math.round(HERO.high * (SHADE.from / 100));
}

/** The gaps between the parts stacked inside it. (`HeroView.java:109,124,163`) */
const GAP = { aboveFacts: 16, aboveStory: 16, aboveButtons: 30, betweenButtons: 16 };

/** The name, when the title has no lettering of its own: two lines at this height. (`HeroView.java:105`) */
const NAME = { size: TITLE_NAME_SIZE, line: 80, lines: 2 };

/**
 * The title's own lettering for the hero, which may arrive on the tile or with its details.
 *
 * `HeroCard.kt:65-67` looks in both and takes the first: the tile that opened the page carries what
 * the page builder put on it, and the title's own page carries the rest. The logo is the name drawn
 * as a picture, so a card that has one does not draw the written name as well
 * (`HeroView.java:146-151`).
 *
 * The Kotlin's reading is the one taken here, and it is not the Java's: `HeroView.java:145` reads
 * the logo off the details alone and never off the tile. Note 255 recorded that divergence on
 * 2026-11-13 and says the Kotlin, the Apple surface and this one agree with each other. Whoever
 * settles it moves Android, not this file.
 */
export function heroLogo(lead: Tile, detail: TitleDetail | null): string | null {
  return logo(lead.images) ?? logo(detail?.images);
}

/** The height of the part at the top of the stack: the lettering's box, or two lines of the name. */
export function headHigh(hasLogo: boolean): number {
  // The lettering is drawn to fit its box and is usually shorter than it. Its full box is taken
  // here, which moves the stack up rather than letting it fall off the card's foot: the column is
  // measured and bottom-aligned on Android (`HeroView.java:178-186`), and nothing on this surface
  // can measure a picture before it is drawn.
  return hasLogo ? TITLE_LOGO.maxHigh : NAME.line * NAME.lines;
}

/**
 * Where each part of the card's stack sits, measured down from the card's own corner.
 *
 * The column is bottom-aligned (`HeroView.java:180-185`), so this is built from the foot upwards:
 * the buttons sit on the foot, the story above them, the facts above that when there are any, and
 * the name or the lettering on top. A card with no facts closes that gap rather than leaving a hole
 * where the line would have been - `HeroView.java:155-164` adds the view and its gap together, or
 * neither.
 */
export function heroStack(hasLogo: boolean, hasFacts: boolean): {
  head: number;
  facts: number;
  story: number;
  buttons: number;
} {
  const buttons = HERO.high - HERO.foot - PILL.high;
  const story = buttons - GAP.aboveButtons - STORY_LINE * STORY_LINES;
  const facts = story - GAP.aboveFacts - FACTS.size;
  const head = (hasFacts ? facts : story) - GAP.aboveStory - headHigh(hasLogo);
  return { head, facts, story, buttons };
}

/** The two buttons of the hero, the distance between them, and which is which. (`HeroView.java:114-123`) */
export const BUTTON_GAP = GAP.betweenButtons;

/** The name's own type, for the card that draws it as words. */
export const HERO_NAME = NAME;
