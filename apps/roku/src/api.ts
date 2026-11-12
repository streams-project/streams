// Every route this surface reads, and how it names an address.
//
// THE ADDRESSES ARE NOT WRITTEN HERE. `ADDRESS` is generated from `platform/contract/src/address.rs`
// into `platform/contract/generated/types/contract.ts` and re-exported by `src/contract.ts`. A
// screen reads a name from it and never types an address. Note 187 recorded what happens otherwise:
// the television surface had hand typed `"/" + screen` and `"/genre/..."`, they had already stopped
// matching the table, and "those presses went nowhere"
// (`notes/187-tv-navigation-per-page-2026-11-08.md:24-26`).
//
// WHAT IS STILL HAND WRITTEN, AND WHY. Two things in the contract are data in Rust and reach no
// TypeScript file:
//   * the screen-word to address map (`address.rs:65-81`) and `of_screen` (`address.rs:83-104`);
//   * the route table (`api.rs:167-830`), which becomes `generated/openapi.json` but no TypeScript
//     value.
// Both are rebuilt below from generated pieces wherever possible: `SCREENS` maps a screen's word
// onto an `ADDRESS` entry rather than onto a typed out path, so no address string appears twice.
// The route paths have no generated source at all and are the one real transcription here. That
// gap belongs to the emitter, not to this folder: see the report.

import { ADDRESS, type ScreenName } from "./contract";

/**
 * Where a tab or a screen lives, by the screen's word on the wire. The words are `ScreenName`
 * values, so a word this build does not know fails to compile. Every address is an `ADDRESS`
 * entry. (`platform/contract/src/address.rs:65-81`)
 */
export const SCREENS: Partial<Record<ScreenName, string>> = {
  home: ADDRESS.HOME,
  kids: ADDRESS.KIDS,
  series: ADDRESS.SERIES,
  films: ADDRESS.FILMS,
  new: ADDRESS.NEW,
  games: ADDRESS.GAMES,
  "my-list": ADDRESS.MY_LIST,
  mine: ADDRESS.MINE,
  language: ADDRESS.LANGUAGE,
  search: ADDRESS.SEARCH,
  notifications: ADDRESS.NOTIFICATIONS,
  activity: ADDRESS.ACTIVITY,
  clips: ADDRESS.CLIPS,
  help: ADDRESS.HELP,
  settings: ADDRESS.SETTINGS,
};

/** The address of a screen. A genre page needs its genre. (`address.rs:83-104`) */
export function tabAddress(screen: ScreenName, genre?: string | null): string {
  if (screen === "genre") return ADDRESS.GENRE + (genre ?? "");
  return SCREENS[screen] ?? ADDRESS.HOME;
}

/**
 * A title key in an address: `movie:12` becomes `/title/movie-12`. Film and series ids overlap, so
 * the kind stays in the address. (`address.rs:31-32`)
 */
export function titleAddress(title: string): string {
  return ADDRESS.TITLE + title.replace(":", "-");
}

/** A title key in a path: `movie:12` becomes `movie/12`. (`address.rs:33-34`, `TitlePage.kt:58`) */
export function titlePath(title: string): string {
  return title.replace(":", "/");
}

/** Followed by `/season/episode` for a series, as an episode has no id of its own. (`address.rs:33-34`) */
export function watchAddress(title: string): string {
  return ADDRESS.WATCH + titlePath(title);
}

/**
 * The routes this surface reads, each a function so no screen builds a path by hand. Every line
 * names where in `platform/contract/src/api.rs` the route is written, because the generator emits
 * these paths only into `generated/openapi.json` and into no TypeScript value.
 */
export const ROUTE = {
  /** `api.rs:169`. Limits and features, read again at its `ttl_s`. */
  hello: () => "/v1/hello",

  /** `api.rs:183`. The 301 words of the interface in one language. */
  strings: (lang: string) => `/v1/strings/${lang}`,

  /** `api.rs:598`. The menu for this class and profile: what the top bar draws as tabs. */
  nav: () => "/v1/nav",

  /** `api.rs:606`. A built page of rows. */
  screen: (name: ScreenName) => `/v1/screens/${name}`,

  /**
   * `api.rs:619`. The rows past the first twelve. The server builds up to 50 and hands a screen 12
   * with a cursor to read the rest (`Browse.kt:60-63`).
   */
  more: (cursor: string) => `/v1/more/${cursor}`,

  /** `api.rs:626`. One title's own page. `kind` is `movie` or `tv`. */
  title: (kind: string, id: string) => `/v1/titles/${kind}/${id}`,

  /** `api.rs:633`. One season's episodes. */
  season: (kind: string, id: string, n: number) => `/v1/titles/${kind}/${id}/seasons/${n}`,

  /** `api.rs:700`. What the viewer changed. This surface sends changes as an action, not here. */
  commands: () => "/v1/commands",

  /**
   * `api.rs:791`. One step of the rules for a device that cannot run them itself. Sends
   * `HostedStep`, answers `CoreOut`.
   *
   * This is the route the Roku question turns on, and it is **not built yet**:
   * `server/crates/server-api/tests/routes.rs:11` lists it as still to build, waiting on
   * "server-hosted". Nothing in this folder calls it until it answers.
   */
  step: () => "/v1/core/step",
};
