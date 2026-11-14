// What a screen with no answer says, and the hole underneath it.
//
// A MIRROR of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Lost.kt:80-86`, which is
// four days old and was written for a measured defect: on 2026-11-13 a signed-in television whose
// server had gone drew 9 nodes, 0 focusable and 0 words, held across 8 probes and across a
// force-stop and cold start (`Lost.kt:39-42`). There was no number to be too long or too short,
// because nothing was ever drawn.
//
// **This family has the same hole, and one more of its own.** Every word this app draws arrives
// from `GET /v1/strings/{lang}`, so an app that cannot reach its server has no vocabulary at all:
// `error.no_server` and `say.try_again` exist in all eighteen languages and none of them is on
// the device. The hole Roku adds is at the other end of the run: this family has no address
// screen (`src/door.ts`), so a device that was never handed an address has no server to name
// either, and `src/host.ts` leaves `refused` holding the address it would not take.
//
// So the rule this file carries is the one `Lost.kt:114-115` found: the address is the one thing
// on the screen that did not have to be fetched to be known, and with no strings it is the whole
// of what a person is handed. It is therefore last and never missing, which is also what keeps
// the sentence from being empty - an empty sentence leaves the only focusable node on the screen
// unnamed.

import { KEY, say, sayBoth } from "./words";

/**
 * What the remote is on, said when the screen opens and again whenever it changes.
 *
 * Never the empty string as long as there is an address, which is the point of the order.
 * (`Lost.kt:83-86`)
 */
export function lostSays(words: Record<string, string>, server: string): string {
  return sayBoth(sayBoth(say(words, KEY.errorNoServer), say(words, KEY.sayTryAgain)), server);
}

/**
 * The word on the one control this screen has, or nothing.
 *
 * Nothing is drawn rather than an empty shape when the word has not arrived, which is the rule
 * `parts/Pill.kt` states from the other end - a pill with no word draws nothing rather than an
 * empty white shape - but the control itself stays, because this screen exists for the case where
 * the words did not arrive. (`Lost.kt:123-126`)
 */
export function tryAgainWord(words: Record<string, string>): string {
  return say(words, KEY.sayTryAgain);
}

/**
 * The sentence above the address, or nothing. Separate from [lostSays] because one is read aloud
 * and the other is drawn: a screen reader is handed the whole thing, and the page draws only the
 * parts it has. (`Lost.kt:105-113`)
 */
export function lostTrouble(words: Record<string, string>): string {
  return say(words, KEY.errorNoServer);
}
