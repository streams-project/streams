// The hexagon of the HarmonyOS family. The core sits in the middle, and everything it needs from a
// device goes through one of these. They name the same six ports as every other native family:
// apps/android/ports/src/main/kotlin/streams/ports/Ports.kt and apps/apple/Sources/Ports/Ports.swift.
// A surface draws and reaches the world only through a port.
//
// NOT COMPILED. No HarmonyOS SDK is on the machine this was written on. See ../OPEN-QUESTIONS.md.
//
// Written to ArkTS and not to TypeScript, which is a narrower language than the file this started
// as. What that costs is recorded in ../OPEN-QUESTIONS.md Q2, and the rules this file obeys are:
// no `any`, no intersection type, no anonymous object type inside a union, no utility type, no
// index signature (a `Record` instead), no destructuring, no regular expression literal.
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

/**
 * Two named interfaces, not two shapes written in place: ArkTS refuses an anonymous object type
 * inside a union (`arkts-no-obj-literals-as-types`, 10605040), so every member of every union in
 * this folder has a name of its own.
 */
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
  /**
   * The picture, as an object only the surface's own toolkit understands -- the same bargain
   * [Pictures] and [Reel] make, and for the same reason: this file may not name a type that belongs
   * to one device.
   *
   * On ArkUI it is a surface id: an `XComponent` of type `SURFACE` reports one through
   * `onSurfaceCreated(surfaceId: string)`, and `AVPlayer.surfaceId` takes it. That is why the
   * argument is the id and not a view: the surface the picture lands on is made by the surface
   * ring, and the adapter is handed the one string that names it. OPEN QUESTION Q5 asked what this
   * object was; the answer is a string, so the port can now carry it.
   *
   * Officially `XComponent` "does not support mirroring capabilities", which is the behaviour this
   * project already wanted: the interface mirrors for Arabic and the picture never does.
   */
  picture(surfaceId: string): void;

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
   * The same five arguments and the same handle back as the other two families, because this is how
   * a film reaches the port at all. The handle is typed `Object` and the first adapter names it.
   */
  picture(context: Object, url: string, loop: boolean, ready: () => void, ended: () => void): Object;
  /** Starts or stops one film. */
  play(picture: Object, on: boolean): void;
  /** Hands back the decoder, because the title player needs one. */
  release(picture: Object): void;
  /** Holds every film still while the app is off screen, and lets them go on after. */
  hold(still: boolean): void;
}

/** One picture wanted. `done` gets a [Painted] or an [Unpainted]. */
export interface Want {
  url: string;
  /**
   * The box it will fill, in pixels, or 0 for the picture's own size. The size is asked for here
   * and not left to the layout for two reasons: the read starts before the first measure instead
   * of a frame after it, and a picture read ahead of the viewer lands under the same key as the
   * one that is drawn, so reading ahead is worth something. The nearest sample size is taken, so
   * a w342 poster is not re-encoded to fit a box exactly.
   *
   * Absent is 0, which is what `Want` in the other native families defaults these to.
   */
  wide?: number;
  high?: number;
  /**
   * True when the caller will read the pixels back, which is what taking a tint off a billboard
   * or a profile gate does. The picture then has to be decoded into ordinary memory: the pixels
   * of one handed to the graphics card cannot be read, and asking for them throws.
   *
   * Absent is false.
   */
  pixels?: boolean;
}

/**
 * A picture, as an object only the surface's own toolkit understands -- the bargain [Player.picture]
 * already makes, and for the same reason: this file may not name a type that belongs to one device.
 * The family's adapter documents which object it hands over.
 *
 * On ArkUI that object is an `image.PixelMap`, which `Image(src: PixelMap | ResourceStr |
 * DrawableDescriptor)` takes as a source. It is typed `Object` here so that this file names no
 * toolkit, exactly as the Android port types it `Any`.
 */
export interface Painted {
  kind: 'painted';
  picture: Object;
}

/** Why there is no picture. The caller draws something else; it never waits for one that is not coming. */
export interface Unpainted {
  kind: 'unpainted';
  reason: string;
}

export type Picture = Painted | Unpainted;

/** The right to stop one read. Letting it go stops nothing. */
export interface Stop {
  stop(): void;
}

/**
 * Pictures from the network: artwork, logos and the brand's mark.
 *
 * This is the port the Android family added on 2026-11-12, and this family needs it for a reason of
 * its own that is sharper than Android's. `Image(src)` on ArkUI accepts a url string and then
 * performs the request itself, inside the UI framework: a surface that passed a url to `Image()`
 * would have the network reaching past its port with no line of code naming it, which is precisely
 * what `Ports.kt` records happening on Android -- 78 `okhttp3` and 89 `okio` classes in the APK
 * while a word ban on `okhttp3` passed every run, because no surface ever typed it.
 *
 * So no screen in `surfaces/touch` writes a url into a component. A screen asks this port and draws
 * what it hands back.
 */
export interface Pictures {
  /**
   * Reads one picture. `done` runs once, with a [Painted] or an [Unpainted]. A read that was
   * stopped answers neither.
   */
  fetch(want: Want, done: (picture: Picture) => void): Stop;

  /**
   * Reads a picture into the cache before anything draws it, at the size the box will ask for, so
   * a row is already painted when the viewer reaches it.
   *
   * `done` runs whether the picture arrived or not: the one call site that passes it uses it to
   * stop waiting, and a failed read has to stop the wait as surely as a good one.
   */
  warm(want: Want, done?: () => void): void;
}

/** Nothing to stop, because nothing was started. */
class Stopped implements Stop {
  stop(): void {
  }
}

/**
 * What [Shell.pictures] answers while no picture adapter is plugged in.
 *
 * It refuses in the open and at once, because the alternative is worse: a port that quietly never
 * answers leaves a screen waiting on a read that cannot finish, and that reads to a person as a
 * slow network rather than as an app that was wired wrong.
 *
 * The reason is not said to a viewer, so it is not a word from `strings/`: it is handed to whoever
 * is wiring the app, and no screen draws it.
 */
export class NoPictures implements Pictures {
  fetch(want: Want, done: (picture: Picture) => void): Stop {
    const refusal: Unpainted = { kind: 'unpainted', reason: 'no picture adapter is plugged into this app' };
    done(refusal);
    return new Stopped();
  }

  warm(want: Want, done?: () => void): void {
    if (done !== undefined) {
      done();
    }
  }
}

/**
 * All a surface knows. It reads state and hands back what the viewer asked for. The same fields as
 * `Seen` in the other two native families, so one measurement of the reference holds for all three.
 *
 * A plain class, with no ArkUI decorator on it. `@ObservedV2` and `@Trace` would make a screen
 * redraw on a field's change without any of the work below, and they are refused here on purpose:
 * they belong to the UI framework, and a port that imports the UI framework is the okhttp3 mistake
 * in a new coat. How a screen hears about a change is [Seen.copy] and [Shell.watch], and the screen
 * side of it is one `@Local` field.
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

  /**
   * This, as a second object with the same fields.
   *
   * It is here because of how ArkUI hears about a change, and it is the one thing in this folder
   * that is not a port of `Ports.kt` line for line -- it is a port of what `Host.kt` already does.
   * `Host.kt` holds a `MutableStateFlow<Seen>` and every write is `_seen.value = _seen.value.copy(
   * ...)`, so a collector is handed a NEW object each time. A `@Local` field in ArkUI works the
   * same way round: it notices that the variable was assigned, not that a field inside the object
   * it already holds was written. So a host that mutates one `Seen` in place and calls its watchers
   * would leave every screen holding an object it has no reason to redraw.
   *
   * Field by field and not by any reflective copy: ArkTS bans `Object.assign` and the whole of
   * `Reflect` (`arkts-limited-stdlib`, 10605144). The arrays and the words are handed over as they
   * are, not cloned: the host replaces them rather than writing into them, which is what the Kotlin
   * `copy` does too.
   */
  copy(): Seen {
    const made = new Seen();
    made.route = this.route;
    made.device = this.device;
    made.profiles = this.profiles;
    made.play = this.play;
    made.screen = this.screen;
    made.nav = this.nav;
    made.active = this.active;
    made.words = this.words;
    made.language = this.language;
    made.rtl = this.rtl;
    made.detail = this.detail;
    made.notice = this.notice;
    made.notices = this.notices;
    made.brand = this.brand;
    made.server = this.server;
    made.serverPlain = this.serverPlain;
    made.positionMs = this.positionMs;
    made.lengthMs = this.lengthMs;
    made.audio = this.audio;
    made.text = this.text;
    made.unreadNotifications = this.unreadNotifications;
    return made;
  }
}

export interface Shell {
  readonly seen: Seen;
  readonly player: Player;
  readonly reel: Reel;

  /**
   * Where a surface's pictures come from.
   *
   * No default, for the reason `Ports.kt` gives: a defaulted port is a seam that rots, because a
   * surface compiles against it and gets refusals at runtime instead of pictures. The app wiring
   * builds one and hands it to the host, so a family that forgets fails to compile.
   */
  readonly pictures: Pictures;

  /**
   * Called on every change of `seen`, with a fresh object each time (see [Seen.copy]). A screen
   * holds what it is handed in one `@Local` field and assigns it here; that assignment is the whole
   * of how a screen redraws. OPEN QUESTION Q4 is closed by that and by nothing else in ArkUI.
   */
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
