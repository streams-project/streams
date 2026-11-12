// The remote, for a Roku. Every key a Roku sends lives here, so a new key or a corrected name
// touches one file.
//
// A television surface paints its own focus from plain state and takes keys at one place, as
// `apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Remote.kt:26-96` does. This file is the
// Roku half of that: it turns a Roku key name into one of the six words a screen answers, and
// nothing else in this folder knows a Roku key name.

/** What a remote can say. A screen answers true when it used the word. (Remote.kt:26) */
export type Word = "up" | "down" | "left" | "right" | "ok" | "back";

/**
 * The key strings a Roku sends, from Roku's own `onKeyEvent` page (verified 2026-11-12, page
 * updated 2026-11-10: https://developer.roku.com/dev/docs/onkeyevent). They are case sensitive,
 * and `OK` is the only one with capitals.
 *
 * The player's keys are listed because the player will need them; no screen in this folder answers
 * them yet.
 */
export const ROKU_KEYS = [
  "back",
  "up",
  "down",
  "left",
  "right",
  "OK",
  "replay",
  "play",
  "playonly",
  "rewind",
  "fastforward",
  "options",
  "pause",
  "channelup",
  "channeldown",
] as const;

export type RokuKey = (typeof ROKU_KEYS)[number];

/** A Roku key name as one of the six words, or nothing when no screen here answers that key. */
export function wordOf(key: string): Word | null {
  switch (key) {
    case "up":
      return "up";
    case "down":
      return "down";
    case "left":
      return "left";
    case "right":
      return "right";
    case "OK":
      return "ok";
    case "back":
      return "back";
    default:
      return null;
  }
}

/** Left and right swapped, for a right to left page. (Remote.kt:39-43) */
export function turn(word: Word): Word {
  if (word === "left") return "right";
  if (word === "right") return "left";
  return word;
}

/**
 * The one reading of a key press a screen ever needs: the word, already turned for the page's
 * direction. `rtl` comes from the view's `rtl` field, as `Television.kt:52` takes it.
 *
 * Pages say Left for towards the start of a line. In a right to left language the start is on the
 * right, so the arrows swap here, once, for every page. The player keeps its own left to right way.
 */
export function pressWord(key: string, rtl: boolean): Word | null {
  const word = wordOf(key);
  if (word === null) return null;
  return rtl ? turn(word) : word;
}
