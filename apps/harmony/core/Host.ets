// Steps the core and performs what it asks, through the ports. It holds no rules and no screens.
//
// A line by line port of apps/android/core/src/main/kotlin/streams/core/Host.kt, which is the
// written and shipped version of this host. When that file changes, this one is diffed against it;
// nothing here is a second opinion about what a host does.
//
// Two things the Android host takes from its platform are handed in here instead, because the
// HarmonyOS call for each is unverified: monotonic time and a random seed. See [Machine] and
// OPEN QUESTION Q3.
//
// NOT COMPILED, NOT RUN, NOT TESTED. No HarmonyOS SDK or device was available.
//
// OPEN QUESTION Q2 is answered below, and the shape of this file did change because of it: every
// contract literal this host used to write, and every typed read of a body, now goes through
// `../contract/effects.ts` and `../contract/read.ts`, which are TypeScript and may say things ArkTS
// cannot.
import { ABI_VERSION } from '../contract/contract';
import {
  Action,
  CoreOut,
  DeviceClass,
  DriverEvent,
  Effect,
  Event,
  Form,
  Profile,
} from '../contract/contract';
import { abi, close, open, step } from './NativeCore';
import { Answer, Call, Network, Pictures, Player, Reel, Secrets, Seen, Shell, Store } from '../ports/Ports';
// The seam. Everything ArkTS refuses about the generated contract lives there, in TypeScript.
import {
  EffectHttp, EffectSecret, EffectStore,
  actionEvent, activityEvent, addressEvent, asHttp, asPlayer, asSecret, asStore, asView,
  coreConfigText, coreInText, driverEvent, httpDoneEvent, httpFailedEvent, networkEvent,
  pauseAction, readCoreOut, requestAction, secretDoneEvent, storeDoneEvent, unsupportedEvent,
  visibilityEvent,
} from '../contract/effects';
import {
  readNav, readNotificationCount, readScreen, readStrings, readTitle,
} from '../contract/read';

const SERVER = 'app:server';
const DEFAULT_LANGUAGE = 'en';
const WAKE_CAP_MS = 60000;
const TOUCH_EVERY_MS = 10000;
const ROUNDS = 64;

// OPEN QUESTION Q2 IS ANSWERED, AND THE ANSWER IS BETTER THAN THE QUESTION EXPECTED.
//
// The question was whether the generated contract types pass the ArkTS linter. Three of the four
// constructs it asked about are hard compile errors in ArkTS -- `(string & {})` is
// `arkts-no-intersection-types` (10605019), an anonymous object type in a union is
// `arkts-no-obj-literals-as-types` (10605040), `Extract<...>` is `arkts-no-utility-types` (10605138),
// and all three are severity `error`, which the cookbook defines as "the program will fail to
// compile". Counted in the generated file on 2026-11-12: forty intersections, eighty anonymous union
// members, three template literal types and one `as const`.
//
// And none of it has to change, because the rules are scoped by FILE EXTENSION. `arkts-no-ts-deps`
// (10605147) states the direction outright -- "An .ets file can import source code of an
// .ets/.ts/.js file, but a .ts/.js file cannot import source code of an .ets file" -- and the
// migration background page scopes the check itself: "the SDK adds the ArkTS syntax validation **for
// .ets files** to the compilation process".
//
// So the generated contract stays TypeScript, and everything it does that ArkTS refuses stays on
// that side of the seam. `../contract/effects.ts` is that seam: it narrows the unions with `Extract`
// where `Extract` is legal, and hands this file ordinary named types. What used to be here -- three
// `Extract` aliases and three casts between unrelated nominal types, which ArkTS would have refused
// for having no structural typing (`arkts-no-structural-typing`, 10605030) -- is gone, and no change
// to `platform/contract/gen` is needed for this family.
//
// The price is stated in the reference and is paid knowingly: "when data and objects of TS/JS are
// used as those of ArkTS, the static type check of ArkTS may be bypassed, causing unexpected
// behavior or extra overhead". Which is why the seam names every shape it exports rather than
// widening anything, and why this host reads only tagged members of tagged unions.

// Said to the viewer, not logged: the only words this host has of its own. The same sentence the
// Android host says, so the two families refuse an address in the same words.
const PLAIN_REFUSED =
  'This address must start with https://. Plain http reaches only a server on your own network.';

/**
 * The device facts the host needs that are not ports. The app wiring supplies all three.
 *
 * They stay behind this interface rather than being read here, so that this file names no HarmonyOS
 * module at all and reads the same as `Host.kt`. What each one is on HarmonyOS was open on
 * 2026-11-11 and is now read off the reference, so the wiring has no searching to do:
 *
 *   monoMs     `systemDateTime.getUptime(systemDateTime.TimeType.STARTUP, false)` from
 *              `@kit.BasicServicesKit`, API 10, no permission. STARTUP is "milliseconds elapsed
 *              since system startup, INCLUDING the deep sleep time", which is what Android's
 *              `SystemClock.elapsedRealtime()` counts. `TimeType.ACTIVE` excludes sleep and is
 *              Android's `uptimeMillis()`, so it is the wrong one of the two.
 *   seedHex    `cryptoFramework.createRandom().generateRandomSync(16)` from
 *              `@kit.CryptoArchitectureKit`, API 10, returning a `DataBlob` whose `data` is a
 *              `Uint8Array`; the wiring writes those 16 bytes as 32 hex digits.
 *   language   `i18n.System.getSystemLanguage()` from `@kit.LocalizationKit`, API 9, no
 *              permission. It answers with a script subtag ("zh-Hans"), not a bare language.
 */
export interface Machine {
  /** Monotonic milliseconds. Restarts at boot. */
  monoMs(): number;
  /** At least 32 hex digits from a cryptographic source. */
  seedHex(): string;
  /**
   * The language the device is set to, or null when the wiring cannot read one.
   *
   * The Android host speaks this before any profile when the server has it. It was handed in here
   * rather than read here for the same reason as the other two, and the host falls back to English
   * when it is null, which is what this file did unconditionally while the call was unknown.
   */
  language(): string | null;
}

/** Whether every character of a string is a Western digit, and how many there are. */
function digitsOnly(text: string, least: number, most: number): boolean {
  // Not `/^[0-9]{1,3}$/`: ArkTS refuses a regular expression LITERAL (`arkts-no-regexp-literals`,
  // which is recorded only in the Adaptation Cases page and not in the migration cookbook, so it
  // is easy to miss). `new RegExp('^[0-9]{1,3}$')` would be legal, and this is cheaper than either:
  // one pass, no engine, and it says what it tests.
  if (text.length < least || text.length > most) {
    return false;
  }
  for (let at = 0; at < text.length; at++) {
    const code = text.charCodeAt(at);
    if (code < 0x30 || code > 0x39) {
      return false;
    }
  }
  return true;
}

/** This address with every trailing slash taken off. */
function trimmedEnd(address: string): string {
  // Not `.replace(/\/+$/, '')`, for the same reason as above.
  let end = address.length;
  while (end > 0 && address.charAt(end - 1) === '/') {
    end--;
  }
  return address.substring(0, end);
}

/** The host of an address, with the port, the path and an IPv6 address's brackets taken off. */
function hostOf(address: string): string {
  const after = address.replace('http://', '').split('/')[0];
  if (after.startsWith('[')) {
    return after.substring(1).split(']')[0];
  }
  return after.split(':')[0];
}

/**
 * The viewer's own machine or their own network: a hop no stranger is on. Names are here because a
 * server is usually found by its name on the network, not by its number.
 */
function ownNetwork(host: string): boolean {
  if (host === 'localhost' || host.endsWith('.local') || host === '::1') {
    return true;
  }
  const parts = host.split('.');
  if (parts.length !== 4) {
    return false;
  }
  const numbers: number[] = [];
  for (const part of parts) {
    // Digits only, then parse. Kotlin's `toIntOrNull` refuses "1abc" where JS `Number.parseInt`
    // reads 1, and accepts a zero padded "010" where comparing the parsed value back to the text
    // would refuse it. This accepts and refuses exactly what the Android gate does.
    if (!digitsOnly(part, 1, 3)) {
      return false;
    }
    const value = Number.parseInt(part, 10);
    if (value > 255) {
      return false;
    }
    numbers.push(value);
  }
  if (numbers[0] === 127 || numbers[0] === 10) {
    return true;
  }
  if (numbers[0] === 192 && numbers[1] === 168) {
    return true;
  }
  if (numbers[0] === 169 && numbers[1] === 254) {
    return true;
  }
  return numbers[0] === 172 && numbers[1] >= 16 && numbers[1] <= 31;
}

/**
 * True when this address is reached in the clear, false when it is encrypted, and null when it may
 * not be used at all.
 */
function encryptionOf(server: string): boolean | null {
  const lower = server.toLowerCase();
  if (lower.startsWith('https://')) {
    return false;
  }
  if (!lower.startsWith('http://')) {
    return null;
  }
  return ownNetwork(hostOf(lower)) ? true : null;
}

export class Host implements Shell {
  readonly seen: Seen = new Seen();
  private readonly watchers: ((seen: Seen) => void)[] = [];
  private handle: number = 0;
  private base: string = '';
  private matched: string | null = null;
  private queue: Event[] = [];
  private asks: number = 0;
  private readonly waiting: Map<string, (status: number, text: string) => void> = new Map();
  private readonly trail: string[] = [];
  private here: string = '/';
  private wake: number = 0;
  private lastTouch: number = 0;
  private stepping: boolean = false;

  constructor(
    private readonly network: Network,
    private readonly secrets: Secrets,
    private readonly store: Store,
    readonly player: Player,
    readonly reel: Reel,
    // Carried, never called from here. The host performs no drawing and reads no picture; it holds
    // this only so that a screen reaches it through the one object it already has, which is what
    // `Host.kt` does with its own `pictures`. There is no default: a family that forgets to plug an
    // adapter in fails to compile, which is where that mistake belongs.
    readonly pictures: Pictures,
    private readonly machine: Machine,
    private readonly deviceClass: DeviceClass,
    private readonly form: Form,
    private readonly version: string,
  ) {
    this.network.watch((online: boolean) => this.step([networkEvent(online)]));
    this.player.listen((generation: number, event: DriverEvent) => {
      // The four driver sourced fields of [Seen], written where Host.kt's init block writes them: a
      // position event carries the scrubber, a tracks event carries the sound and subtitle pickers.
      // Nothing else in this host writes them, so without these two branches a player screen reads 0,
      // 0, [] and [] for the life of the app.
      if (event.kind === 'position') {
        this.seen.positionMs = event.time_ms;
        // Absent means not known, and Host.kt keeps the length it already had rather than clearing it.
        if (event.duration_ms !== undefined) {
          this.seen.lengthMs = event.duration_ms;
        }
        this.changed();
      } else if (event.kind === 'tracks') {
        this.seen.audio = event.audio;
        this.seen.text = event.text;
        this.changed();
      }
      this.step([driverEvent(generation, event)]);
    });
    const kept = this.secrets.get(SERVER);
    if (kept !== null) {
      this.connect(kept);
    }
  }

  watch(changed: (seen: Seen) => void): void {
    this.watchers.push(changed);
  }

  /**
   * Every watcher, each handed a fresh [Seen].
   *
   * The copy is the whole of how a screen redraws, and leaving it out was a bug this file carried
   * from the day it was written. `Host.kt` holds `MutableStateFlow<Seen>` and writes
   * `_seen.value = _seen.value.copy(...)`, so a collector is handed a new object per change; this
   * file wrote fields into one `Seen` and handed the same object out. Under Compose that would
   * still redraw, because a `StateFlow` emission is the signal and the object's identity is not;
   * under ArkUI a `@Local` field notices that the variable was ASSIGNED and nothing else, so every
   * screen would have held an object it had no reason to draw again and the app would have looked
   * frozen from the first step. Nothing about that is visible without a device, and nothing about
   * it needed one.
   */
  private changed(): void {
    if (this.watchers.length === 0) {
      return;
    }
    const sent = this.seen.copy();
    for (const watcher of this.watchers) {
      watcher(sent);
    }
  }

  /** A notice is said once. Every write of words moves the count a screen reads itself by. */
  private say(notice: string | null): void {
    this.seen.notice = notice;
    // Every write, including one that resolves to no words, the same as Host.kt's EffectView branch.
    // A screen waits on this count moving, so skipping the null case leaves it waiting forever.
    this.seen.notices += 1;
    this.changed();
  }

  /**
   * The one gate every server address passes, typed or kept. Plain http carries the session's
   * bearer and refresh tokens in the clear, so it may only reach the viewer's own network; anything
   * else must be https. A refused address is said, not swallowed.
   */
  connect(server: string): void {
    const typed = trimmedEnd(server.trim());
    const plain = encryptionOf(typed);
    if (plain === null) {
      this.say(PLAIN_REFUSED);
      return;
    }
    this.base = typed;
    this.secrets.put(SERVER, this.base);
    // This address was taken, so a refusal still sitting in the app wide slot is not news any more
    // and must not be read as some other screen's error.
    this.seen.server = this.base;
    this.seen.serverPlain = plain;
    this.seen.notice = null;
    this.changed();
    if (this.handle === 0) {
      this.start();
    }
  }

  private start(): void {
    if (abi() !== ABI_VERSION) {
      throw new Error('this core has another ABI');
    }
    const config: string = coreConfigText(
      'harmony',
      this.deviceClass,
      this.form,
      'harmony',
      this.machine.seedHex(),
      this.version,
    );
    this.handle = open(config);
    if (this.handle === 0) {
      throw new Error('the core did not open');
    }
    this.step([addressEvent('/')]);
  }

  stop(): void {
    if (this.handle !== 0) {
      close(this.handle);
      this.handle = 0;
    }
  }

  go(address: string): void {
    this.trail.push(this.here);
    this.here = address;
    this.step([addressEvent(address)]);
  }

  swap(address: string): void {
    this.here = address;
    this.step([addressEvent(address)]);
  }

  back(): void {
    this.here = this.trail.pop() ?? '/';
    this.step([addressEvent(this.here)]);
  }

  /** The app left the screen or came back. Nobody can see a film off screen, so it pauses. */
  shown(visible: boolean): void {
    this.reel.hold(!visible);
    const events: Event[] = [];
    if (!visible) {
      events.push(actionEvent(pauseAction()));
    }
    events.push(visibilityEvent(visible));
    this.step(events);
  }

  /** A touch. The core only needs to hear this now and then. */
  touched(): void {
    const now = this.machine.monoMs();
    if (now - this.lastTouch < TOUCH_EVERY_MS) {
      return;
    }
    this.lastTouch = now;
    this.step([activityEvent()]);
  }

  act(action: Action): void {
    this.step([actionEvent(action)]);
  }

  ask(path: string, reply: (status: number, text: string) => void): void {
    const id = this.awaited(reply);
    this.step([actionEvent(requestAction('GET', path), id)]);
  }

  send(method: string, path: string, body: string, reply: (status: number, text: string) => void): void {
    const id = this.awaited(reply);
    this.step([actionEvent(requestAction(method, path, body), id)]);
  }

  /** Names the answer a screen is waiting for. The http effect the core makes carries this id. */
  private awaited(reply: (status: number, text: string) => void): string {
    this.asks += 1;
    const id = `ask-${this.asks}`;
    this.waiting.set(id, reply);
    return id;
  }

  keep(profile: Profile): void {
    this.seen.profiles = this.seen.profiles.map((held: Profile) => (held.id === profile.id ? profile : held));
    this.changed();
  }

  keepAll(profiles: Profile[]): void {
    this.seen.profiles = profiles;
    this.changed();
  }

  speak(language: string | null): void {
    this.readWords(language ?? DEFAULT_LANGUAGE, this.seen.brand);
  }

  refreshNotifications(): void {
    if (this.seen.active === null) {
      return;
    }
    this.ask('/v1/notifications/count', (status: number, text: string) => {
      if (status >= 200 && status <= 299) {
        const count = readNotificationCount(text);
        if (count !== null) {
          this.seen.unreadNotifications = count.unread;
          this.changed();
        }
      }
    });
  }

  /**
   * One step, then every step the effects ask for, up to [ROUNDS]. Everything here runs on one
   * thread, because a core handle is used from one thread at a time.
   */
  private step(events: Event[]): void {
    if (this.handle === 0) {
      return;
    }
    for (const event of events) {
      this.queue.push(event);
    }
    // One step at a time. The Android host gets this from posting every call to its own thread; here
    // a port can answer inside the step that asked, so a step already running takes the new events
    // on its next round instead of starting a second one.
    if (this.stepping) {
      return;
    }
    this.stepping = true;
    let rounds = 0;
    try {
      do {
        const batch = this.queue;
        this.queue = [];
        const input: string = coreInText(batch, this.machine.monoMs(), Date.now());
        const out = readCoreOut(step(this.handle, input));
        if (out === null) {
          // The core answered something that is not JSON. There is nothing to perform and nothing
          // to say, and going round again would ask the same question of the same handle.
          break;
        }
        this.apply(out);
        rounds += 1;
      } while (this.queue.length > 0 && rounds < ROUNDS);
    } finally {
      // A refused step throws. Without this the flag would stay up and the host would never step
      // again, which looks to a viewer like an app that stopped rather than one that said so.
      this.stepping = false;
    }
  }

  private apply(out: CoreOut): void {
    const view = out.view;
    if (view !== undefined) {
      const hello = view.hello;
      // Before any profile, the Android host speaks the device's language when the server has it.
      // The call that gives it is `i18n.System.getSystemLanguage()`, which the app wiring reads and
      // hands over as [Machine.language] -- this file names no HarmonyOS module. A null falls back
      // to English, and `readWords` falls back again if the server does not have the language, so a
      // screen always has words. (This was the readable half of OPEN QUESTION Q3.)
      if (hello !== undefined && Object.keys(this.seen.words).length === 0) {
        this.readWords(this.machine.language() ?? DEFAULT_LANGUAGE, hello.brand.name);
      }
      const wasActive = this.seen.active;
      this.seen.route = view.route ?? this.seen.route;
      this.seen.device = view.device_sign_in ?? this.seen.device;
      this.seen.profiles = view.profiles?.profiles ?? this.seen.profiles;
      this.seen.play = view.play ?? this.seen.play;
      this.seen.brand = view.hello?.brand.name ?? this.seen.brand;
      this.seen.active = view.session?.profile ?? this.seen.active;
      this.changed();
      this.matchLanguage();
      if (this.seen.active !== null && this.seen.active !== wasActive) {
        this.refreshNotifications();
      }
    }
    if (this.wake !== 0) {
      clearTimeout(this.wake);
      this.wake = 0;
    }
    const at = out.next_wake_ms;
    if (at !== undefined) {
      const after = Math.min(Math.max(at - this.machine.monoMs(), 0), WAKE_CAP_MS);
      this.wake = setTimeout(() => this.step([]), after);
    }
    for (const effect of out.effects) {
      this.perform(effect);
    }
  }

  /** A profile kept from an earlier run speaks its own language from the start. */
  private matchLanguage(): void {
    const active = this.seen.active;
    const profile = this.seen.profiles.find((held: Profile) => held.id === active);
    const wanted = profile?.language;
    if (wanted === undefined) {
      return;
    }
    if (wanted !== this.seen.language && wanted !== this.matched) {
      this.matched = wanted;
      this.readWords(wanted, this.seen.brand);
    }
  }

  /** The words of the interface, with the brand's name in them. */
  private readWords(language: string, brand: string): void {
    this.ask(`/v1/strings/${language}`, (status: number, text: string) => {
      const good = status >= 200 && status <= 299;
      // A language this server does not have falls back to English, so a screen always has words.
      if (!good && language !== DEFAULT_LANGUAGE) {
        this.readWords(DEFAULT_LANGUAGE, brand);
        return;
      }
      if (good) {
        const read = readStrings(text);
        if (read === null) {
          return;
        }
        const words: Record<string, string> = {};
        for (const key of Object.keys(read.texts)) {
          // One place each, so `replace` taking the first occurrence alone is the whole of it.
          words[key] = read.texts[key].replace('{brand}', brand);
        }
        this.seen.words = words;
        this.seen.language = read.lang;
        this.seen.rtl = read.dir === 'rtl';
        this.changed();
      }
    });
  }

  private perform(effect: Effect): void {
    switch (effect.kind) {
      // Narrowed on the TypeScript side of the seam, so nothing here casts.
      case 'http':
        this.http(asHttp(effect));
        break;
      case 'secret':
        this.queue.push(this.secret(asSecret(effect)));
        break;
      case 'store':
        this.queue.push(this.kept(asStore(effect)));
        break;
      case 'player':
        this.player.command(asPlayer(effect).command);
        break;
      case 'view': {
        const notice = asView(effect).notice;
        if (notice !== undefined) {
          this.say(notice.text.length > 0 ? notice.text : (this.seen.words[notice.label] ?? null));
        }
        break;
      }
      case 'open':
      case 'log':
        break;
      default:
        this.queue.push(unsupportedEvent(effect.id));
    }
  }

  private kept(effect: EffectStore): Event {
    const key = effect.key ?? '';
    switch (effect.op) {
      case 'get':
        return storeDoneEvent(effect.id, this.store.get(key) ?? undefined);
      case 'put':
        this.store.put(key, effect.value ?? '');
        return storeDoneEvent(effect.id);
      case 'delete':
        this.store.delete(key);
        return storeDoneEvent(effect.id);
      case 'list':
        return storeDoneEvent(effect.id, undefined, this.store.keys(key));
      default:
        return unsupportedEvent(effect.id);
    }
  }

  private secret(effect: EffectSecret): Event {
    const name = `core:${effect.name}`;
    if (effect.op === 'get') {
      return secretDoneEvent(effect.id, this.secrets.get(name) ?? undefined);
    }
    if (effect.op === 'put') {
      // A refused write is not a write. Saying it was done would sign the viewer out at the next
      // launch with no signal, where the core keeps the tokens in memory instead.
      if (this.secrets.put(name, effect.value ?? '')) {
        return secretDoneEvent(effect.id);
      }
      return unsupportedEvent(effect.id);
    }
    this.secrets.drop(name);
    return secretDoneEvent(effect.id);
  }

  private http(effect: EffectHttp): void {
    const call: Call = {
      method: effect.method,
      url: this.base + effect.path,
      headers: effect.headers ?? {},
      body: effect.body,
      deadlineMs: effect.deadline_ms,
    };
    this.network.send(call, (answer: Answer) => {
      const reply = this.waiting.get(effect.id);
      this.waiting.delete(effect.id);
      if (answer.kind === 'no_reply') {
        reply?.(0, '');
        this.step([httpFailedEvent(effect.id, answer.reason)]);
        return;
      }
      const good = answer.status >= 200 && answer.status <= 299;
      const toScreen = effect.to === 'screen' && good;
      if (toScreen) {
        this.read(effect.path, answer.text);
      }
      reply?.(answer.status, answer.text);
      this.step([
        httpDoneEvent(
          effect.id,
          answer.status,
          !toScreen && answer.text.length > 0 ? answer.text : undefined,
          answer.etag !== undefined ? { etag: answer.etag } : undefined,
        ),
      ]);
    });
  }

  /** A body the core sent to the screen. The host keeps it; it reads nothing out of it. */
  private read(path: string, text: string): void {
    if (path.startsWith('/v1/nav')) {
      const read = readNav(text);
      if (read !== null) {
        this.seen.nav = read.items;
      }
    } else if (path.startsWith('/v1/screens/')) {
      const read = readScreen(text);
      if (read !== null) {
        this.seen.screen = read;
      }
    } else if (path.startsWith('/v1/notifications/count')) {
      const read = readNotificationCount(text);
      if (read !== null) {
        this.seen.unreadNotifications = read.unread;
      }
    } else if (path.includes('/titles/') && !path.includes('/seasons/')) {
      const read = readTitle(text);
      if (read !== null) {
        this.seen.detail = read;
      }
    }
    this.changed();
  }
}
