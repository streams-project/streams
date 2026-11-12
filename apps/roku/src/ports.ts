// The hexagon of the Roku family. The rules sit in the middle, and everything a surface needs from
// a Roku goes through one of these ports.
//
// The shape follows `apps/tv/page/ports.js:6-20`, which is the same idea for the three web
// televisions, and `apps/android/ports/src/main/kotlin/streams/ports/Ports.kt:175-243`, which is
// the same idea for Android. A screen in `src/screens/` may use a port and never a Roku feature
// behind one, so the screens can be read and checked without a Roku.
//
// Nothing here is a Roku API. What a Roku really does for each port is an adapter, and the
// adapters are the part that cannot be written or checked on this machine: see the README.

import type {
  Action,
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
  /** Where this app's server lives. Empty until the viewer gives an address. */
  server: string;
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
  audio: [],
  text: [],
};

/** How an answer comes back: the http status, and the body as text. A status of 0 is no answer. */
export type Reply = (status: number, text: string) => void;

/**
 * The one way out of a screen. A port of the `Shell` of `Ports.kt:213-243`, cut to what the screens
 * in this folder call.
 */
export interface Shell {
  /** What the screens draw from now. */
  seen(): Seen;

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
