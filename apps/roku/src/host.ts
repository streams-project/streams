// The host: the one thing in this folder that is not a screen and not a rule.
//
// It holds the state every screen draws from, it performs what a screen asks for through the
// ports, and it decides nothing about what anything looks like. The twin to read beside it is
// `apps/android/core/src/main/kotlin/streams/core/Host.kt`, which is the same object for Android,
// and `apps/apple/Sources/CoreHost/Host.swift`, which is the same object for Apple.
//
// **Where this one differs from both, and it is the whole architectural question of this folder.**
// Those two step the shared Rust core: an event goes in, a `CoreOut` of effects comes out, and the
// host is a hundred lines of "perform this effect through that port". A Roku runs neither native
// code nor WebAssembly (README section 2), so there is no core here to step. The two ways out were
// `POST /v1/core/step`, which would step it on the server and which
// `server/crates/server-api/tests/routes.rs:13` still lists as unbuilt, and writing the rules
// again in this language. The owner chose the second on 2026-11-12 for the drawing half (README
// section 7). This file is the rest of that choice, and it is kept as small as it can be:
//
//   * what an address opens is `src/route.ts`, a named mirror held to the Rust;
//   * where the server comes from is `src/door.ts`;
//   * what this host does **not** do is as load-bearing as what it does, and is listed at the foot
//     of this file rather than silently missing.
//
// Nothing here is a Roku API. Every outside thing is a port, and `src/adapters.ts` is where a
// Roku is finally named.

import { ROUTE } from "./api";
import type {
  Action,
  Command,
  Nav,
  Screen,
  ScreenName,
  Strings,
  TitleDetail,
} from "./contract";
import { ADDRESS } from "./contract";
import { SERVER_KEY, encryptionOf, serverFrom, tidy } from "./door";
import { ASK_MS, LOST_MS } from "./numbers";
import type { Clock, Launch, Network, Player, Reply, Seen, Shell, Stop, Store } from "./ports";
import { NOTHING_SEEN } from "./ports";
import { targetOf } from "./route";

/** The language this app asks for when nothing else has said one. (`core/Host.kt:89`) */
export const DEFAULT_LANGUAGE = "en";

/** The launch parameter that names the language, for a device set up in one. This surface's own. */
export const LANGUAGE_PARAMETER = "lang";

/** What a Roku has to give this host. Each is a port, and each is named in `src/ports.ts`. */
export interface Fittings {
  network: Network;
  store: Store;
  clock: Clock;
  /**
   * No default, for the reason Android's `pictures` has none (`ports/Ports.kt:218-225`): a
   * defaulted port is a seam that rots, because a surface compiles against it and gets refusals at
   * run time instead of a build error where the mistake was made.
   */
  player: Player;
  launch: Launch;
}

/**
 * The state of one run, and the only thing the screens see.
 *
 * Everything is on one thread: a Roku's TypeScript cannot start one
 * (Dev Summit 2026 Q&A, Q16, read 2026-11-12 and recorded in the README), which removes the whole
 * class of problem `core/Host.kt` solves with a `HandlerThread` and makes this host simpler than
 * either of its twins rather than harder.
 */
export class Host implements Shell {
  private state: Seen = NOTHING_SEEN;
  private readonly watchers: ((seen: Seen) => void)[] = [];

  /**
   * Where the viewer is, and the trail Back walks out along. (`core/Host.kt:144-145`)
   *
   * It starts at the home page and not at `ADDRESS.START`, which is where both other families
   * start. `/` is the core's way of asking "who is this": `routes.rs:247` sends a member to the
   * home the server named for them and everybody else to the door. This host cannot ask, because
   * neither the door nor the profile gate is written here, so it starts where a member would have
   * been sent and a server that will not answer for this viewer says so in a status.
   */
  private here: string = ADDRESS.HOME;
  private readonly trail: string[] = [];

  /**
   * Which state the app is in now, counted up on every move.
   *
   * This is the whole of the arrival rule and it is the reason this number exists rather than a
   * boolean. `tv/title/Episodes.kt:153-168` states it for one page: a season being read draws
   * nothing, and the cards in hand are drawn only when the answer that brought them was for the
   * season the page is on. Before that fix a television drew the previous season's names, stills
   * and descriptions under a column already marking the new season as chosen, for as long as the
   * read took. The same defect at this level is a page of rows drawing the last page's rows while
   * the new one is being read, so here the rule is at the host and not in one screen: every read
   * carries the number the app was on when it started, and an answer whose number has moved on is
   * **dropped**, not drawn.
   */
  private generation = 0;

  /**
   * Which server this is, counted up on every `open`.
   *
   * The same rule as `generation` at the other scope. The menu and the words belong to a server
   * and not to a page, so they are not thrown away when the viewer moves between pages, and they
   * ARE thrown away when the server changes under them. An app that took the old server's menu
   * after being re-pointed would draw tabs that lead nowhere.
   */
  private connection = 0;

  /** How many reads for the state the app is on now have not answered. */
  private reading = 0;

  /** The wait that ends in saying so, and the one in flight. */
  private waiting: Stop | null = null;

  constructor(
    private readonly fittings: Fittings,
    private readonly boundServer: string = "",
  ) {}

  // ---- what a screen sees ------------------------------------------------

  seen(): Seen {
    return this.state;
  }

  /** Told whenever `seen()` changes. The Roku half of Android's `StateFlow`. */
  watch(told: (seen: Seen) => void): void {
    this.watchers.push(told);
    told(this.state);
  }

  private change(made: Partial<Seen>): void {
    this.state = { ...this.state, ...made };
    for (const told of this.watchers) told(this.state);
  }

  // ---- starting ----------------------------------------------------------

  /**
   * One run of the app, from nothing to a drawn screen.
   *
   * A run with no address anywhere ends here with `server` empty, which is the state `main.tsx`
   * draws nothing for. See the head of `src/door.ts` for how launch, kept and bound addresses are
   * resolved.
   */
  start(): void {
    const given = serverFrom(
      this.fittings.launch.parameters(),
      this.fittings.store.get(SERVER_KEY),
      this.boundServer,
    );
    if (given.from === "none") return;
    this.open(given.server);
  }

  /** Ask the host to try the current server address again. */
  retry(): void {
    if (this.state.server !== "") {
      this.open(this.state.server);
    } else {
      this.start();
    }
  }

  /**
   * The one gate every server address passes, launched, kept or bound. A refused address is said,
   * not swallowed: `refused` carries it, and it is the viewer's own data, so a screen can show it
   * with no words at all.
   */
  private open(server: string): void {
    const address = tidy(server);
    const plain = encryptionOf(address);
    if (plain === null) {
      this.change({ refused: address, server: "", lost: false, reading: false });
      return;
    }
    this.connection += 1;
    this.change({ server: address, serverPlain: plain, refused: "", nav: [] });
    this.fittings.store.put(SERVER_KEY, address);
    this.speak(this.fittings.launch.parameters()[LANGUAGE_PARAMETER] ?? null);
    this.ofServer(ROUTE.nav(), (text) => {
      this.change({ nav: (JSON.parse(text) as Nav).items });
    });
    // `arrive` and not `go`: opening a server is not a move the viewer made, and a trail entry
    // here would make Back walk out to the page it is already on.
    this.arrive(this.here);
  }

  // ---- moving ------------------------------------------------------------

  go(address: string): void {
    this.trail.push(this.here);
    this.arrive(address);
  }

  swap(address: string): void {
    this.arrive(address);
  }

  back(): void {
    this.arrive(this.trail.pop() ?? ADDRESS.HOME);
  }

  /**
   * Land on an address: what it opens, with nothing of the last page still on the screen.
   *
   * The clearing is the half of the television's fix that is easy to leave out. Dropping a late
   * answer stops the app drawing the wrong page's data **after** the move; clearing here stops it
   * drawing the wrong page's data **during** the read, which is the part a viewer actually saw.
   */
  private arrive(address: string): void {
    this.here = address;
    this.generation += 1;
    this.reading = 0;
    this.waiting?.stop();
    this.waiting = null;
    this.change({ route: targetOf(address), screen: null, detail: null, reading: false, lost: false });
    this.read();
  }

  /**
   * A read whose answer belongs to the server rather than to a page: it outlives every move
   * between pages and is dropped only when the server changes. A failure here does not say the
   * app is lost - a menu that did not arrive is a bar with no tabs, not a page with no content.
   */
  private ofServer(path: string, took: (text: string) => void): void {
    const mine = this.connection;
    this.ask(path, (status, text) => {
      if (mine !== this.connection) return;
      if (status < 200 || status > 299) return;
      try {
        took(text);
      } catch {
        // Left alone: an answer this app cannot read is one it has not got.
      }
    });
  }

  /** What the page the app is now on needs from the server. */
  private read(): void {
    const route = this.state.route;
    if (route === null || this.state.server === "") return;
    if (route.target === "home") return this.page("home");
    if (route.target === "kids_home") return this.page("kids");
    if (route.target === "screen" && route.screen !== undefined) return this.page(route.screen);
    if (route.target === "title" && route.title !== undefined) {
      const [kind, id] = route.title.split(":");
      if (kind === undefined || id === undefined) return;
      return this.fill(ROUTE.title(kind, id), (text) => {
        this.change({ detail: JSON.parse(text) as TitleDetail });
      });
    }
  }

  private page(name: ScreenName): void {
    this.fill(ROUTE.screen(name), (text) => {
      this.change({ screen: JSON.parse(text) as Screen });
    });
  }

  // ---- reading -----------------------------------------------------------

  /**
   * A read whose answer fills `Seen`, and which is dropped when it answers for a page the app has
   * already left. `took` is called only for an answer that is still this page's.
   */
  private fill(path: string, took: (text: string) => void): void {
    const mine = this.generation;
    this.reading += 1;
    this.change({ reading: true });
    this.wait();
    this.ask(path, (status, text) => {
      if (mine !== this.generation) return;
      this.reading -= 1;
      if (status < 200 || status > 299) {
        this.change({ reading: this.reading > 0, lost: true });
        return;
      }
      try {
        took(text);
      } catch {
        // An answer this app cannot read is an answer it has not got. It is not drawn as half a
        // page: `JSON.parse` throwing here means the server sent something no version of this
        // contract describes, and the page stays empty rather than part filled.
        this.change({ reading: this.reading > 0, lost: true });
        return;
      }
      this.waiting?.stop();
      this.waiting = null;
      this.change({ reading: this.reading > 0, lost: false });
    });
  }

  /**
   * Start the wait that ends in saying so, once per page.
   *
   * This exists because a server that accepts a connection and then says nothing is the one
   * failure a read cannot report: `fill` already says lost the moment a read fails, which covers a
   * refused connection, a wrong address and an error from the server.
   */
  private wait(): void {
    if (this.waiting !== null) return;
    const mine = this.generation;
    this.waiting = this.fittings.clock.after(LOST_MS, () => {
      if (mine !== this.generation) return;
      this.waiting = null;
      if (this.reading > 0) this.change({ lost: true });
    });
  }

  ask(path: string, reply: Reply): void {
    this.call("GET", path, null, reply);
  }

  send(method: string, path: string, body: string, reply: Reply): void {
    this.call(method, path, body, reply);
  }

  private call(method: string, path: string, body: string | null, reply: Reply): void {
    if (this.state.server === "") {
      reply(0, "");
      return;
    }
    this.fittings.network.send(
      {
        method,
        url: this.state.server + path,
        headers: body === null ? {} : { "content-type": "application/json" },
        body,
        deadlineMs: ASK_MS,
      },
      (answer) => {
        if (answer.kind === "none") reply(0, "");
        else reply(answer.status, answer.text);
      },
    );
  }

  // ---- words -------------------------------------------------------------

  /**
   * The words of the interface. A language this server has not got falls back to English, so a
   * screen always has words, and the brand's name is filled in where a word asks for it.
   * (`core/Host.kt:311-330`)
   */
  speak(language: string | null): void {
    this.readWords(language ?? DEFAULT_LANGUAGE);
  }

  private readWords(language: string): void {
    const mine = this.connection;
    this.ask(ROUTE.strings(language), (status, text) => {
      if (mine !== this.connection) return;
      if ((status < 200 || status > 299) && language !== DEFAULT_LANGUAGE) {
        this.readWords(DEFAULT_LANGUAGE);
        return;
      }
      if (status < 200 || status > 299) return;
      let read: Strings;
      try {
        read = JSON.parse(text) as Strings;
      } catch {
        return;
      }
      const words: Record<string, string> = {};
      for (const key of Object.keys(read.texts)) {
        words[key] = (read.texts[key] ?? "").replace("{brand}", this.brand);
      }
      this.change({ words, language: read.lang, rtl: read.dir === "rtl" });
    });
  }

  /**
   * The brand's name, for the `{brand}` a word may carry.
   *
   * Empty, and knowingly. Android reads it from `View.hello.brand.name` as the core hands the view
   * over (`core/Host.kt:343-348`); nothing in this folder reads `Hello` yet, so a word with
   * `{brand}` in it draws with the mark missing rather than with the wrong name in it. The README
   * lists `GET /v1/hello` under what is not written.
   */
  private readonly brand = "";

  // ---- what the viewer changed -------------------------------------------

  /**
   * A change the viewer made.
   *
   * On Android this goes to the core, which queues it, retries it and holds the session token
   * (`TitlePage.kt:135,140`). There is no core here, so the one kind of action the screens in this
   * folder send - a `command` - is posted to `POST /v1/commands` as a batch of one, and every
   * other kind is refused in the open rather than dropped on the floor.
   *
   * What is lost by not having the core here, said plainly: the queue. A command sent while the
   * server is unreachable is gone. The offline layer that would hold it needs somewhere to keep
   * it, and a Roku's registry is 32 KB for the whole app (`src/ports.ts`, `Store`).
   */
  act(action: Action): void {
    if (action.action !== "command") {
      this.refuse(action.action);
      return;
    }
    const ops: Command[] = [action.command];
    this.send("POST", ROUTE.commands(), JSON.stringify({ ops }), () => {});
  }

  /**
   * An action this host cannot carry out. It is collected rather than performed, so the day a
   * screen asks for one the answer is a name and not silence.
   */
  readonly refusedActions: string[] = [];

  private refuse(name: string): void {
    if (this.refusedActions.indexOf(name) < 0) this.refusedActions.push(name);
  }
}

// WHAT THIS HOST DOES NOT DO, so nothing reads it as a core.
//
//   * **Sessions.** Signing in, the refresh of a token, the profile gate and the PIN. Every one of
//     them is in the Rust core and none of the screens that drive them is written here. A viewer
//     reaches this app already signed in or not at all.
//   * **The offline layer.** The queue of waiting commands, and kept pages. See `act` above and
//     the `Store` port for the 32 KB that decides it.
//   * **Playing.** `Player` is a port with a refusing adapter behind it. The Video node's field
//     list was not read and there is no answer yet on HLS or DRM.
//   * **`Hello`.** Limits, features, the brand's name and the moved-address table. Nothing reads
//     it, so `{brand}` draws empty and `src/route.ts` mirrors no moved table.
//   * **Stepping anything.** `POST /v1/core/step` is in `src/api.ts` as `ROUTE.step` and is called
//     from nowhere, because it does not answer yet.
