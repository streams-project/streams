// The three small parts more than one screen draws, as numbers and as plain functions.
//
// The facts line, the large button, and the marks on a tile. Each is drawn on at least two screens
// here - the facts line on the hero, under the open card and on a title's page - so each has one
// set of numbers, in one place, beside the line it was read from. `src/screens/Parts.tsx` draws
// them and holds no number of its own.
//
// **Where these belong.** `src/numbers.ts` is this surface's one home for a number, and these three
// blocks belong in it. That file was another agent's for this pass and is not edited here, so they
// sit in this one under the same rule: one home, each number beside the Kotlin or Java line it came
// from, nothing restated. Moving them is a cut and a paste, and nothing else in this folder has to
// change with them.
//
// A citation written `tv/...` is relative to `apps/android/surfaces/tv/src/main/`, where `kotlin/`
// holds the behaviour spec and `java/` holds the views that draw it today.

import { COLOUR } from "./numbers";

/**
 * The line of facts: the size of a word, the dot between two of them, and the room each side of it.
 * (`java/streams/tv/parts/Facts.java:32-38`, which cites `kotlin/streams/tv/parts/Facts.kt:58,60`)
 */
export const FACTS = { size: 24, dot: 6, side: 14 };

/**
 * A mark on a tile: the gap between two of them, the room inside one, the size of its word, and how
 * black the plate behind it is, in hundredths.
 * (`java/streams/tv/parts/Badges.java:36-47`, which cites `kotlin/streams/tv/parts/Badge.kt:40-53`)
 */
export const BADGE = { gap: 8, across: 12, down: 4, size: 22, plate: 72 };

/**
 * A large button: how tall it is, the room each side of its word, its word's size, and the icon
 * beside it with the gap after it.
 * (`kotlin/streams/tv/parts/Pill.kt:44-51`, and the same 84 at `java/streams/tv/browse/HeroView.java:118`)
 */
export const PILL = { high: 84, inset: 46, size: 32, icon: 34, gap: 12 };

/** A share, in hundredths, as the two hex figures a Roku ends a colour with. */
function share(hundredths: number): string {
  return Math.round((hundredths / 100) * 255)
    .toString(16)
    .padStart(2, "0");
}

/**
 * A plate behind a word that sits over artwork: black at a share of itself, as a Roku writes a
 * colour (`0xrrggbbaa`). The share is the one thing written, so `Ink.black(72)` and this read the
 * same. (`java/streams/tv/parts/Badges.java:46-47`)
 */
export function black(hundredths: number): string {
  return `0x000000${share(hundredths)}`;
}

/**
 * A white edge or fill at a share of itself. White on this surface is the brand's text colour,
 * which is where `src/numbers.ts:198` already takes a focus ring's white from, so nothing here
 * names a colour of its own. (`java/streams/tv/browse/HeroView.java:70`, `Ink.white(28)`)
 */
export function white(hundredths: number): string {
  return `${COLOUR.text.slice(0, -2)}${share(hundredths)}`;
}

/**
 * How tall a badge comes to: its word plus the room above and below it. A plate drawn shorter than
 * its word clips it, so this is a derivation and never a number of its own.
 */
export function badgeHigh(): number {
  return BADGE.size + BADGE.down * 2;
}
