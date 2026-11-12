// The one seam between a Roku's keys and this surface's six words.
//
// Every screen in this folder owns one focusable and reads every key itself, because a television
// surface paints its own focus from plain state: that is the decision
// `apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Remote.kt:57-69` records, and the reason
// the key maps of note 187 could be designed per page at all.
//
// `useFocusable` and its `onKey` map are Roku's (verified 2026-11-12,
// https://developer.roku.com/dev/docs/rsg-sdk-usefocusable): `onKey` is "a declarative map from
// remote key names to handlers", a handler is called with a `press` argument, and the boundary
// derives its handled keys from the map's keys.
//
// OPEN, and the one thing in this folder that cannot be settled without the SDK installed: Roku's
// page shows a key written as `RemoteKey.Select` but does not list the enum's members, and the
// FocusBoundary page does not define `RemoteKey` either. The map below is keyed by the key strings
// Roku's own `onKeyEvent` page documents (`src/keys.ts`, verified), which is the only key naming
// Roku publishes. If `RemoteKey` turns out to carry different spellings, `ROKU_KEYS` in
// `src/keys.ts` is the single line to change and nothing else in this folder moves.

import { useFocusable } from "@roku-sdk/navigation";

import { ROKU_KEYS, pressWord, type Word } from "../keys";

/**
 * Gives a whole screen the remote.
 *
 * `read` is handed the word, already turned for a right to left page, and answers true when the
 * screen used it. A key the screen did not use is left unhandled, so it bubbles as Roku's own
 * handlers do (https://developer.roku.com/dev/docs/onkeyevent: returning false "lets the event keep
 * bubbling up the focus chain").
 *
 * `says` is what the remote is on now - a tab's name, a title's name, a button's word. On Android
 * it is a live region, because without it a person using a screen reader hears one thing when a
 * screen opens and nothing ever again (`Remote.kt:57-69`). What a Roku does with it is open: see
 * the README.
 */
export function screenRemote(options: {
  read: (word: Word) => boolean;
  rtl: () => boolean;
  says?: () => string;
}) {
  // OPEN, and the reason `says` is handed back rather than used: the only accessibility field found
  // on Roku's pages is `audioGuideText`, on the Poster node. Nothing equivalent to Android's live
  // region - a thing re-spoken each time it changes, which is what makes a self-drawn focus
  // audible - was established, so a screen is given its own sentence back and the README keeps this
  // open rather than this file pretending to solve it.
  const onKey: Record<string, (press: boolean) => boolean> = {};
  for (const key of ROKU_KEYS) {
    onKey[key] = (press: boolean): boolean => {
      // A key is answered on the way down only, as `Remote.kt:94` does: a screen that answered both
      // would move twice for one press.
      if (!press) return false;
      const word = pressWord(key, options.rtl());
      if (word === null) return false;
      return options.read(word);
    };
  }
  return { ...useFocusable({ onKey }), says: options.says ?? ((): string => "") };
}
