// Which rows a page of the Roku surface draws, as plain functions of what it was given.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Page.kt`, line for line and
// on purpose: this is the one piece of a rows page that can be wrong without anything looking
// wrong. The screen the core holds is not always the screen the route asked for, and when it is
// not, a page draws another page's rows under its own name. A reviewer caught exactly that on the
// Games tab on 2026-11-11 (`Page.kt:11-15`).
//
// It lives apart from the drawing so it can be checked without a screen, a device or an SDK:
// `src/tests/page.test.ts` is that check, and it needs no Roku.

import type { Screen, Section, SectionKind, Tile } from "./contract";

/** The tab that has nothing to show yet. The server builds no rows for it. (`Page.kt:21`) */
const GAMES = "games";

/**
 * The section kinds this surface draws as a row of tiles. Anything else is not a row.
 * (`Page.kt:24`)
 *
 * The words are the contract's `SectionKind`, and the annotation is documentation rather than a
 * check: the generated type ends in `(string & {})` so that an unknown kind reads as a plain row
 * instead of failing (`platform/contract/README.md:41`), which also means TypeScript will accept
 * any string here. A misspelling is caught by `src/tests/page.test.ts`, not by the compiler.
 */
const ROW_KINDS: SectionKind[] = ["row", "ranked", "continue", "grid"];

/** A section with no tiles is not drawn: a row with no titles in it is an empty line under a name. */
function filled(sections: Section[]): Section[] {
  return sections.filter((section) => section.tiles !== undefined && section.tiles.length > 0);
}

/**
 * Whether the answer in hand belongs to the page the viewer is on.
 *
 * Home and the kids home name no screen on the route, so there the answer's own name stands for it.
 * Games is never the answer's name, so it never owns one. (`Page.kt:32-33`)
 */
export function answerIsThisPage(current: string, asked: string | null, answerName: string): boolean {
  return current !== GAMES && (asked === null || asked === answerName);
}

/**
 * The rows a page draws: the answer's own rows, and the rows read past the first twelve when the
 * answer is this page's. A page with nothing of its own draws nothing, whatever is held.
 * (`Page.kt:39-49`)
 */
export function rowsOn(
  current: string,
  asked: string | null,
  answerName: string,
  answer: Section[],
  extra: Section[],
): Section[] {
  const held = filled(current === GAMES ? [] : answer);
  const all = answerIsThisPage(current, asked, answerName) ? held.concat(filled(extra)) : held;
  return all.filter((section) => ROW_KINDS.indexOf(section.kind) >= 0);
}

/** The title the top of a page opens with, or nothing when the page has no hero of its own. (`Page.kt:52-55`) */
export function heroOn(current: string, answer: Section[]): Tile | null {
  const hero = filled(current === GAMES ? [] : answer).find((section) => section.kind === "hero");
  const tile = hero?.tiles?.[0];
  return tile ?? null;
}

/** The page's own name, as `Browse.kt:159` reads it: the route's screen, or home. */
export function screenOf(routeScreen: string | null | undefined): string {
  return routeScreen ?? "home";
}

/** The key a page's kept place is filed under, for the run of the app. (`Browse.kt:164`) */
export function pageKey(profile: string, current: string, genre: string | null | undefined): string {
  return `${profile} ${current} ${genre ?? ""}`;
}

/** The answer's own name, for `answerIsThisPage`. */
export function answerName(screen: Screen): string {
  return screen.name;
}
