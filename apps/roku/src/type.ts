// How wide a drawn word is, when nothing on this machine can measure one.
//
// A Roku lays a Label out itself, and the SDK publishes no way to ask how wide the result is
// before it is drawn (the Label node's field list was read on 2026-11-12,
// https://developer.roku.com/dev/docs/label, and carries `text`, `color` and `lineSpacing`; nothing
// on it answers a measurement). Every part of this surface that puts a second thing beside a word -
// the dot between two facts, the next tab along the bar - needs that answer before it can place it.
//
// So this file holds the one estimate, in one place, with what it is worth written next to it.
// `src/screens/TopBar.tsx` carried the same arithmetic privately until today; two estimates would
// have been two different wrong answers.
//
// **This is an estimate and it is named as one.** It is not a measurement and it is not in
// `src/numbers.ts`, where every number comes from a Kotlin line, from `reference/netflix-on-the-tv.md`
// or from `brand.json`. What is held against something real is in `src/tests/type.test.ts`: the
// column the bar's first tab word lands on, which the reference measured at x 642.

/** The share of its type size a character takes, across a word, for the faces this surface draws. */
const PER_CHARACTER = 0.56;

/**
 * The isolation marks `fill` puts around a filled value (U+2068, U+2069). They draw nothing, so a
 * word that carries them is no wider for them, and counting them would push whatever sits beside
 * the word along by about a character each.
 * (`apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt:62-67`)
 */
const INVISIBLE = /[⁦-⁩‎‏]/g;

/**
 * About how wide this word is, drawn at this type size.
 *
 * Over by a little rather than under, deliberately: every caller uses it to decide where the next
 * thing starts, and a word that is given too little room collides with what follows, while one
 * given too much leaves a gap.
 */
export function textWide(word: string, size: number): number {
  return Math.round(word.replace(INVISIBLE, "").length * size * PER_CHARACTER);
}

/**
 * Where each of these words starts, when they are drawn in a line with `gap` between them, and how
 * wide the line comes to. The first starts at nothing, so a caller adds its own corner.
 *
 * It is one function rather than arithmetic at each call site because a line of words is drawn
 * three times on this surface - the facts, the tabs, the badges - and all three have to agree about
 * what "beside" means.
 */
export function laidOut(words: string[], size: number, gap: number): { at: number[]; wide: number } {
  const at: number[] = [];
  let x = 0;
  for (const word of words) {
    at.push(x);
    x += textWide(word, size) + gap;
  }
  return { at, wide: words.length === 0 ? 0 : x - gap };
}
