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
// NOT COMPILED, NOT RUN, NOT TESTED. No HarmonyOS SDK or device was available. Reading JSON into the
// generated contract types is OPEN QUESTION Q2, and it may force a change in this file's shape.
import { ABI_VERSION, WIRE_VERSION } from '../contract/contract';
import {
  Action,
  CoreIn,
  CoreOut,
  DriverEvent,
  Effect,
  Event,
  DeviceClass,
  Form,
  NotificationCount,
  Profile,
  Nav,
  Screen,
  Strings,
  TitleDetail,
} from '../contract/contract';
import { abi, close, open, step } from './NativeCore';
import { Answer, Call, Network, Player, Reel, Secrets, Seen, Shell, Store } from '../ports/Ports';

const SERVER = 'app:server';
const DEFAULT_LANGUAGE = 'en';
const WAKE_CAP_MS = 60000;
const TOUCH_EVERY_MS = 10000;
const ROUNDS = 64;

// Each of these names one member of the contract's `Effect` union, which the generator writes as an
// anonymous shape. `Extract` is a TypeScript utility type; if the ArkTS linter refuses it the
// generator emits a named interface per member instead and these three lines go. OPEN QUESTION Q2.
type HttpEffect = Extract<Effect, { kind: 'http' }>;
type StoreEffect = Extract<Effect, { kind: 'store' }>;
type SecretEffect = Extract<Effect, { kind: 'secret' }>;

// Said to the viewer, not logged: the only words this host has of its own. The same sentence the
// Android host says, so the two families refuse an address in the same words.
const PLAIN_REFUSED =
  'This address must start with https://. Plain http reaches only a server on your own network.';

/** The two device facts the host needs that are not ports. The app wiring supplies both. */
export interface Machine {
  /** Monotonic milliseconds. Restarts at boot. */
  monoMs(): number;
  /** At least 32 hex digits from a cryptographic source. */
  seedHex(): string;
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
    if (!/^[0-9]{1,3}$/.test(part)) {
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
    private readonly machine: Machine,
    private readonly deviceClass: DeviceClass,
    private readonly form: Form,
    private readonly version: string,
  ) {
    this.network.watch((online: boolean) => this.step([{ type: 'network', online } as Event]));
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
      this.step([{ type: 'driver', event, generation } as Event]);
    });
    const kept = this.secrets.get(SERVER);
    if (kept !== null) {
      this.connect(kept);
    }
  }

  watch(changed: (seen: Seen) => void): void {
    this.watchers.push(changed);
  }

  private changed(): void {
    for (const watcher of this.watchers) {
      watcher(this.seen);
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
    const typed = server.trim().replace(/\/+$/, '');
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
    const config: string = JSON.stringify({
      app: 'harmony',
      class: this.deviceClass,
      contract: WIRE_VERSION,
      form: this.form,
      mode: 'local',
      os: 'harmony',
      seed: this.machine.seedHex(),
      type: 'core_config',
      version: this.version,
    });
    this.handle = open(config);
    if (this.handle === 0) {
      throw new Error('the core did not open');
    }
    this.step([{ type: 'address', address: '/' } as Event]);
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
    this.step([{ type: 'address', address } as Event]);
  }

  swap(address: string): void {
    this.here = address;
    this.step([{ type: 'address', address } as Event]);
  }

  back(): void {
    this.here = this.trail.pop() ?? '/';
    this.step([{ type: 'address', address: this.here } as Event]);
  }

  /** The app left the screen or came back. Nobody can see a film off screen, so it pauses. */
  shown(visible: boolean): void {
    this.reel.hold(!visible);
    const events: Event[] = [];
    if (!visible) {
      events.push({ type: 'action', action: { action: 'pause' } } as Event);
    }
    events.push({ type: 'visibility', visible } as Event);
    this.step(events);
  }

  /** A touch. The core only needs to hear this now and then. */
  touched(): void {
    const now = this.machine.monoMs();
    if (now - this.lastTouch < TOUCH_EVERY_MS) {
      return;
    }
    this.lastTouch = now;
    this.step([{ type: 'activity' } as Event]);
  }

  act(action: Action): void {
    this.step([{ type: 'action', action } as Event]);
  }

  ask(path: string, reply: (status: number, text: string) => void): void {
    const id = this.awaited(reply);
    this.step([{ type: 'action', id, action: { action: 'request', method: 'GET', path } } as Event]);
  }

  send(method: string, path: string, body: string, reply: (status: number, text: string) => void): void {
    const id = this.awaited(reply);
    this.step([{ type: 'action', id, action: { action: 'request', method, path, body } } as Event]);
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
        const count = JSON.parse(text) as NotificationCount;
        this.seen.unreadNotifications = count.unread;
        this.changed();
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
        const input: string = JSON.stringify({
          contract: WIRE_VERSION,
          events: batch,
          mono_ms: this.machine.monoMs(),
          now_ms: Date.now(),
          type: 'core_in',
        } as CoreIn);
        this.apply(JSON.parse(step(this.handle, input)) as CoreOut);
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
      // The HarmonyOS call that gives the device's language is unverified here (OPEN QUESTION Q3),
      // so this asks for English and the first profile's own language replaces it.
      if (hello !== undefined && Object.keys(this.seen.words).length === 0) {
        this.readWords(DEFAULT_LANGUAGE, hello.brand.name);
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
        const read = JSON.parse(text) as Strings;
        const words: Record<string, string> = {};
        for (const key of Object.keys(read.texts)) {
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
      case 'http':
        this.http(effect);
        break;
      case 'secret':
        this.queue.push(this.secret(effect));
        break;
      case 'store':
        this.queue.push(this.kept(effect));
        break;
      case 'player':
        this.player.command(effect.command);
        break;
      case 'view':
        if (effect.notice !== undefined) {
          const label = effect.notice.label;
          this.say(effect.notice.text.length > 0 ? effect.notice.text : (this.seen.words[label] ?? null));
        }
        break;
      case 'open':
      case 'log':
        break;
      default:
        this.queue.push({ type: 'unsupported', id: effect.id } as Event);
    }
  }

  private kept(effect: StoreEffect): Event {
    const key = effect.key ?? '';
    switch (effect.op) {
      case 'get':
        return { type: 'store_done', id: effect.id, value: this.store.get(key) ?? undefined } as Event;
      case 'put':
        this.store.put(key, effect.value ?? '');
        return { type: 'store_done', id: effect.id } as Event;
      case 'delete':
        this.store.delete(key);
        return { type: 'store_done', id: effect.id } as Event;
      case 'list':
        return { type: 'store_done', id: effect.id, keys: this.store.keys(key) } as Event;
      default:
        return { type: 'unsupported', id: effect.id } as Event;
    }
  }

  private secret(effect: SecretEffect): Event {
    const name = `core:${effect.name}`;
    if (effect.op === 'get') {
      return { type: 'secret_done', id: effect.id, value: this.secrets.get(name) ?? undefined } as Event;
    }
    if (effect.op === 'put') {
      // A refused write is not a write. Saying it was done would sign the viewer out at the next
      // launch with no signal, where the core keeps the tokens in memory instead.
      if (this.secrets.put(name, effect.value ?? '')) {
        return { type: 'secret_done', id: effect.id } as Event;
      }
      return { type: 'unsupported', id: effect.id } as Event;
    }
    this.secrets.drop(name);
    return { type: 'secret_done', id: effect.id } as Event;
  }

  private http(effect: HttpEffect): void {
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
        this.step([{ type: 'http_failed', id: effect.id, reason: answer.reason } as Event]);
        return;
      }
      const good = answer.status >= 200 && answer.status <= 299;
      const toScreen = effect.to === 'screen' && good;
      if (toScreen) {
        this.read(effect.path, answer.text);
      }
      reply?.(answer.status, answer.text);
      this.step([
        {
          type: 'http_done',
          id: effect.id,
          body: !toScreen && answer.text.length > 0 ? answer.text : undefined,
          headers: answer.etag !== undefined ? { etag: answer.etag } : undefined,
          status: answer.status,
        } as Event,
      ]);
    });
  }

  /** A body the core sent to the screen. The host keeps it; it reads nothing out of it. */
  private read(path: string, text: string): void {
    if (path.startsWith('/v1/nav')) {
      this.seen.nav = (JSON.parse(text) as Nav).items;
    } else if (path.startsWith('/v1/screens/')) {
      this.seen.screen = JSON.parse(text) as Screen;
    } else if (path.startsWith('/v1/notifications/count')) {
      this.seen.unreadNotifications = (JSON.parse(text) as NotificationCount).unread;
    } else if (path.includes('/titles/') && !path.includes('/seasons/')) {
      this.seen.detail = JSON.parse(text) as TitleDetail;
    }
    this.changed();
  }
}
