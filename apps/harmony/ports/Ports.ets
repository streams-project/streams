// The hexagon of the HarmonyOS family. The core sits in the middle, and everything it needs from a
// device goes through one of these. They name the same five ports as every other family:
// apps/android/ports/src/main/kotlin/streams/ports/Ports.kt and apps/apple/Sources/Ports/Ports.swift.
// A surface draws and reaches the world only through a port.
//
// NOT COMPILED. No HarmonyOS SDK is on the machine this was written on. See ../OPEN-QUESTIONS.md.
import {
  Action,
  DeviceSignInView,
  DriverCommand,
  DriverEvent,
  NavItem,
  PlayView,
  Profile,
  RouteTarget,
  Screen,
  TitleDetail,
  Track,
} from '../contract/contract';

/** One request. `done` gets a [Reply] or a [NoReply]. */
export interface Call {
  method: string;
  url: string;
  headers: Record<string, string>;
  body?: string;
  deadlineMs: number;
}

/**
 * An answer, or the lack of one. The other two families write this as a sealed type; here it carries
 * its own `kind`, because ArkTS has no pattern matching and testing a field's presence to tell two
 * shapes apart is what the contract's own messages avoid by tagging every one of them.
 */
export interface Reply {
  kind: 'reply';
  status: number;
  text: string;
  etag?: string;
}

export interface NoReply {
  kind: 'no_reply';
  /** offline, timeout or cancelled. */
  reason: string;
}

export type Answer = Reply | NoReply;

export interface Network {
  send(call: Call, done: (answer: Answer) => void): void;

  /** Says when the device gains or loses its connection. */
  watch(changed: (online: boolean) => void): void;
}

/** What the core keeps between runs. */
export interface Secrets {
  get(name: string): string | null;

  /**
   * False when the store refused the write. The caller must say so rather than pretend: a session
   * token that was never written signs the viewer out at the next launch with no signal.
   */
  put(name: string, value: string): boolean;

  drop(name: string): void;
}

/** What the core's offline layer keeps between runs: waiting commands and kept pages. */
export interface Store {
  get(key: string): string | null;
  put(key: string, value: string): void;
  delete(key: string): void;
  keys(prefix: string): string[];
}

/** Plays films, reports facts, and decides nothing. */
export interface Player {
  // The other two families also have `picture(context)`, which hands the surface the picture as an
  // object only that toolkit understands. On ArkUI that object is not known yet: OPEN QUESTION Q5.
  command(given: DriverCommand): void;
  listen(report: (generation: number, event: DriverEvent) => void): void;
  /** Where the picture is now, read fresh for drawing subtitles. */
  now(): number;
  /** Raises subtitles by this share of the picture's height, to clear the controls. */
  lift(share: number): void;
  /** The look a profile keeps for subtitles: size 0, 1 or 2, and colours as #rrggbb or none. */
  wordsLook(size: number, color: string, background: string): void;
}

/** Brand films on the way in. The title player belongs to the core, so these stay apart from it. */
export interface Reel {
  /**
   * A picture of this address. It starts still until `play` says so. `ready` runs once, when it can
   * show and play; `ended` runs when it finishes or cannot play.
   *
   * The same five arguments and the same handle back as the other two families, because this is how a
   * film reaches the port at all. What the returned object is on ArkUI is OPEN QUESTION Q5: an
   * `XComponent`'s surface id, or a `Video` component's controller. So it is typed as `Object` here,
   * and the surface that writes the adapter names it.
   */
  picture(context: Object, url: string, loop: boolean, ready: () => void, ended: () => void): Object;
  /** Starts or stops one film. */
  play(picture: Object, on: boolean): void;
  /** Hands back the decoder, because the title player needs one. */
  release(picture: Object): void;
  /** Holds every film still while the app is off screen, and lets them go on after. */
  hold(still: boolean): void;
}

/**
 * All a surface knows. It reads state and hands back what the viewer asked for. The same fields as
 * `Seen` in the other two native families, so one measurement of the reference holds for all three.
 */
export class Seen {
  route: RouteTarget | null = null;
  device: DeviceSignInView | null = null;
  profiles: Profile[] = [];
  play: PlayView | null = null;
  screen: Screen | null = null;
  nav: NavItem[] = [];
  active: string | null = null;
  /** The words of the interface, with the brand filled in. */
  words: Record<string, string> = {};
  /** The language those words are in. */
  language: string = '';
  /** True when that language runs right to left: the interface mirrors, the player never does. */
  rtl: boolean = false;
  detail: TitleDetail | null = null;
  notice: string | null = null;
  /**
   * How many notices have been said in this run. The slot is app wide and a notice is one time, so
   * a screen that acts records this count first and reads the slot as its own answer only once the
   * count has moved.
   */
  notices: number = 0;
  brand: string = '';
  server: string = '';
  /**
   * True when that server is reached over plain http. The screen that shows the address marks it,
   * so nobody runs on an unencrypted connection without being told.
   */
  serverPlain: boolean = false;
  positionMs: number = 0;
  lengthMs: number = 0;
  /** The sound and subtitle tracks of what is playing. */
  audio: Track[] = [];
  text: Track[] = [];
  unreadNotifications: number = 0;
}

export interface Shell {
  readonly seen: Seen;
  readonly player: Player;
  readonly reel: Reel;
  /** Called on every change of `seen`. The ArkUI form this takes is OPEN QUESTION Q4. */
  watch(changed: (seen: Seen) => void): void;
  connect(server: string): void;
  go(address: string): void;
  /** Goes to this address in place of the one here, so Back passes over it. */
  swap(address: string): void;
  back(): void;
  act(action: Action): void;
  ask(path: string, reply: (status: number, text: string) => void): void;
  /** A change the viewer made. The core adds what the route needs. */
  send(method: string, path: string, body: string, reply: (status: number, text: string) => void): void;
  /** A saved profile, so every screen shows the change. */
  keep(profile: Profile): void;
  /** Replace the kept profile list after adding or deleting a profile. */
  keepAll(profiles: Profile[]): void;
  /** Read the words of this language, or the default when there is none. */
  speak(language: string | null): void;
  /** Refresh the unread notification count. */
  refreshNotifications(): void;
}
