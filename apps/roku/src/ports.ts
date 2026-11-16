// The hexagon of the Roku family. The rules sit in the middle, and everything a surface needs from
// a Roku goes through one of these ports.
//
// The shape follows the deleted television page's `ports.js` (gone 2026-11-12; `apps/web/src/ports.rs` is the living twin), which is the same idea for the three web
// televisions, and `apps/android/ports/src/main/kotlin/streams/ports/Ports.kt:175-243`, which is
// the same idea for Android. A screen in `src/screens/` may use a port and never a Roku feature
// behind one, so the screens can be read and checked without a Roku.
//
// Nothing here is a Roku API. What a Roku really does for each port is an adapter, and
// `src/adapters.ts` is where a Roku is finally named. Three of the six adapters turned out to be
// writable on a machine with no device and no SDK, because what they use - `fetch`, a map, and
// string handling - is not a Roku API either; the other three say in that file which they are
// waiting on. The README's section 8 is the whole of it.

import type {
  Action,
  DriverCommand,
  NavItem,
  Profile,
  RouteTarget,
  Screen,
  TitleDetail,
  Track,
} from "./contract";

/**
 * What a screen sees. A port of `Ports.kt:175-211`, with the fields the screens in this folder
 * read. A field is added here when a screen needs it and not before.
 */
export interface Seen {
  route: RouteTarget | null;
  profiles: Profile[];
  screen: Screen | null;
  nav: NavItem[];
  /** Which profile is watching. */
  active: string | null;
  /** The words of the interface, with the brand filled in. Never English in code. */
  words: Record<string, string>;
  /** The language those words are in. */
  language: string;
  /** True when that language runs right to left: the interface mirrors, the player never does. */
  rtl: boolean;
  detail: TitleDetail | null;
  /** Where this app's server lives. Empty until the app is given an address. */
  server: string;
  /**
   * True when that server is reached over plain http. A plain address is only ever accepted on the
   * viewer's own network (`src/door.ts`).
   */
  serverPlain: boolean;
  /**
   * An address this app refused, and the empty string when none. It is the viewer's own data and
   * not an interface word, so it is the one thing a screen can show before any word has arrived.
   * (`apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Lost.kt:114-115`)
   *
   * Android and Apple say this refusal in a sentence their hosts hold in English
   * (`core/Host.kt:94-95`, `CoreHost/Host.swift:44`). This folder holds no English, and `strings/`
   * has no key for it, so the address is drawn and the sentence is not. See the README.
   */
  refused: string;
  /**
   * True while this app is waiting for the answer the page it is on needs. A page with no answer
   * yet draws its destination and an empty body, never the last page's data.
   * (`tv/browse/Waiting.kt`, and the rule in `tv/title/Episodes.kt:153-168`)
   */
  reading: boolean;
  /**
   * True once this app has a server and has given up waiting for it. The one state the television
   * surface could not reach until 2026-11-13, when a signed-in television whose server had gone
   * drew 9 nodes, 0 focusable and 0 words for as long as anyone looked at it (`Lost.kt:39-42`).
   */
  lost: boolean;
  audio: Track[];
  text: Track[];
}

/** A view with nothing in it, so a screen is never handed undefined. */
export const NOTHING_SEEN: Seen = {
  route: null,
  profiles: [],
  screen: null,
  nav: [],
  active: null,
  words: {},
  language: "",
  rtl: false,
  detail: null,
  server: "",
  serverPlain: false,
  refused: "",
  reading: false,
  lost: false,
  audio: [],
  text: [],
};

/** How an answer comes back: the http status, and the body as text. A status of 0 is no answer. */
export type Reply = (status: number, text: string) => void;

/**
 * One request. A port of `Ports.kt:19-25`, with one field dropped: the Android `Call` carries a
 * `deadlineMs` the core chose per effect, and nothing here chooses one per call, so the deadline is
 * this surface's own and lives in `src/numbers.ts` beside every other number it uses.
 */
export interface Call {
  method: string;
  url: string;
  headers: Record<string, string>;
  body: string | null;
  deadlineMs: number;
}

/** An answer, or the reason there was none. (`Ports.kt:27-29`) */
export type Answer =
  | { kind: "reply"; status: number; text: string }
  | { kind: "none"; reason: string };

/**
 * The network. `done` runs once per call, whatever happened: a port that answers neither way
 * leaves a screen waiting on a read that cannot finish, which reads to a person as a slow network
 * rather than as an app that was wired wrong (`Ports.kt:158-164`).
 *
 * There is no `watch` as Android has (`Ports.kt:35`): no way to be told that a Roku gained or lost
 * its connection was found on Roku's pages, so this port does not pretend to offer one, and
 * `src/host.ts` learns that the server is gone by asking for something and getting nothing.
 */
export interface Network {
  send(call: Call, done: (answer: Answer) => void): void;
}

/**
 * What this app keeps between runs: the server's address, and nothing large.
 *
 * A Roku's own persistent store is the registry, and it is **32 KB for the whole app**
 * (Roku's Dev Summit 2026 Q&A recap, read 2026-11-14). That number decides what this port is for.
 * Android's `Store` (`Ports.kt:52-57`) holds the core's offline layer - waiting commands and kept
 * pages - and a kept page of 12 rows does not fit in 32 KB beside anything else. So this family's
 * store holds settings-sized values, the offline layer is not kept here, and the README says so
 * rather than this port quietly losing writes.
 */
export interface Store {
  get(key: string): string | null;
  put(key: string, value: string): void;
  delete(key: string): void;
}

/**
 * Plays films, reports facts, and decides nothing. A port of `Ports.kt:59-74`, cut to the two
 * calls a surface with no player screen can honestly declare.
 *
 * The player screen is not written and the Video node's field list was not read, so this is the
 * port and not the player: `src/adapters.ts` plugs a refusing one in, which is what a surface
 * reaching for a player it has not got should meet.
 */
export interface Player {
  command(given: DriverCommand): void;
  listen(report: (event: string) => void): void;
}

/** The right to stop one thing. Letting it go stops nothing. (`Ports.kt:123-126`) */
export interface Stop {
  stop(): void;
}

/**
 * Time, as a port, so every wait in `src/host.ts` can be run forward in a test without one.
 *
 * It is a port and not a call to `setTimeout` because no page of Roku's documents that was read
 * says a Roku runs timers for a TypeScript app. The adapter is where that guess lives, in one
 * place, and the host never makes it.
 */
export interface Clock {
  after(ms: number, run: () => void): Stop;
}

/**
 * How this app was started: the parameters a sideloaded channel is launched with.
 *
 * This is the Roku family's own port and no other family has one. The reason is in the README:
 * until the store opens, an RSG 2.0 app can only be installed by someone at a computer running
 * `rk`, so the address of the server can be handed to the app at the moment it is installed, and
 * a television that was handed one needs no screen to type it on.
 */
export interface Launch {
  parameters(): Record<string, string>;
}

/**
 * The one way out of a screen. A port of the `Shell` of `Ports.kt:213-243`, cut to what the screens
 * in this folder call.
 */
export interface Shell {
  /** What the screens draw from now. */
  seen(): Seen;

  /** Ask the host to try the current server address again. */
  retry(): void;

  /** Open an address from `platform/contract/src/address.rs`. A screen never types an address. */
  go(address: string): void;

  /** Go to this address in place of the one here, so Back passes over it. */
  swap(address: string): void;

  back(): void;

  /** A command or an effect for the rules to carry out. */
  act(action: Action): void;

  /** Read a route of the api. The path comes from `src/api.ts` and never from a screen. */
  ask(path: string, reply: Reply): void;

  /** A change the viewer made. */
  send(method: string, path: string, body: string, reply: Reply): void;

  /** Read the words of this language, or the default when there is none. */
  speak(language: string | null): void;
}

/**
 * Turning a remote press into a word is `src/keys.ts`; this is the port that delivers the press.
 * `listen` is given a reader and calls it for each press; the reader answers true when it used the
 * press, as Roku's own `onKeyEvent` does (verified 2026-11-12,
 * https://developer.roku.com/dev/docs/onkeyevent: the handler "must return `true` if the XML
 * component handled the event").
 */
export interface Keys {
  listen(read: (key: string, down: boolean) => boolean): void;
}

/** Where a screen's pictures come from. A Roku draws a picture by its address and nothing else. */
export interface Pictures {
  /** The address to draw, or nothing when the title has no art of that shape. */
  of(url: string | null | undefined): string | null;
}
