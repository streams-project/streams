// The host, run against fake ports, with the contract's own fixtures as the answers.
//
// This is the suite the folder did not have, because until today the folder had no host. What it
// holds down is the four questions a host answers and the screens cannot:
//
//   1. where the server's address comes from, and what a run with none does;
//   2. what fills `Seen`, and - the half that cost the television surface a visible defect - what
//      a screen draws while nothing has filled it yet;
//   3. what happens when the server cannot be reached;
//   4. what this host refuses to do, named rather than silent.
//
// Every answer the fakes hand back is a file from `platform/contract/fixtures/2026-11-05/`, read
// as the same bytes `platform/contract/tests/fixtures.rs` round-trips through the Rust types. An
// invented answer would prove that this host parses what this suite writes.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { NoPlayer } from "../adapters";
import { ADDRESS } from "../contract";
import { SERVER_KEY, SERVER_PARAMETER } from "../door";
import { Host, LANGUAGE_PARAMETER } from "../host";
import { LOST_MS } from "../numbers";
import type { Answer, Call, Clock, Launch, Network, Seen, Stop, Store } from "../ports";

const ROOT = join(__dirname, "..", "..", "..", "..");
const CORPUS = join(ROOT, "platform", "contract", "fixtures", "2026-11-05");

function fixture(name: string): string {
  return readFileSync(join(CORPUS, "valid", name), "utf8");
}

const HOME = fixture("Screen.home.json");
const NAV = fixture("Nav.basic.json");
const FILM = fixture("TitleDetail.film.json");
const SERIES = fixture("TitleDetail.series.json");
const ARABIC = fixture("Strings.ar.json");
const ENGLISH = fixture("Strings.en.json");

/** One call the host made, and the way to answer it. */
interface Asked {
  call: Call;
  done: (answer: Answer) => void;
}

/** The calls already answered, so `answer` always takes the next one in flight. */
const answered = new Set<Asked>();

class FakeNetwork implements Network {
  readonly asked: Asked[] = [];

  send(call: Call, done: (answer: Answer) => void): void {
    this.asked.push({ call, done });
  }

  /** Answer the first unanswered call whose path ends this way. */
  answer(ending: string, status: number, text: string): boolean {
    for (const one of this.asked) {
      if (one.call.url.endsWith(ending) && !answered.has(one)) {
        answered.add(one);
        one.done({ kind: "reply", status, text });
        return true;
      }
    }
    return false;
  }

  /** The server said nothing at all. */
  nothing(ending: string): boolean {
    for (const one of this.asked) {
      if (one.call.url.endsWith(ending) && !answered.has(one)) {
        answered.add(one);
        one.done({ kind: "none", reason: "refused" });
        return true;
      }
    }
    return false;
  }

  paths(): string[] {
    return this.asked.map((one) => one.call.url);
  }
}

class FakeStore implements Store {
  readonly held: Record<string, string> = {};

  get(key: string): string | null {
    return this.held[key] ?? null;
  }

  put(key: string, value: string): void {
    this.held[key] = value;
  }

  delete(key: string): void {
    delete this.held[key];
  }
}

class FakeClock implements Clock {
  waiting: { ms: number; run: () => void; stopped: boolean }[] = [];

  after(ms: number, run: () => void): Stop {
    const one = { ms, run, stopped: false };
    this.waiting.push(one);
    return {
      stop(): void {
        one.stopped = true;
      },
    };
  }

  /** Run every wait that has not been stopped, as time passing would. */
  pass(): void {
    const due = this.waiting;
    this.waiting = [];
    for (const one of due) if (!one.stopped) one.run();
  }
}

/** A clock whose `stop` does nothing, which is a device with no way to cancel a timer. */
class UnstoppableClock implements Clock {
  waiting: { ms: number; run: () => void }[] = [];

  after(ms: number, run: () => void): Stop {
    this.waiting.push({ ms, run });
    return {
      stop(): void {
        // Nothing. That is the point.
      },
    };
  }

  pass(): void {
    const due = this.waiting;
    this.waiting = [];
    for (const one of due) one.run();
  }

  /** Only the oldest wait comes due, which is how a page's wait outlives that page. */
  passOne(): void {
    this.waiting.shift()?.run();
  }
}

class FakeLaunch implements Launch {
  constructor(private given: Record<string, string>) {}

  parameters(): Record<string, string> {
    return this.given;
  }

  set(given: Record<string, string>): void {
    this.given = given;
  }
}

interface Made {
  host: Host;
  network: FakeNetwork;
  store: FakeStore;
  clock: FakeClock;
  launch: FakeLaunch;
  seen: () => Seen;
}

function made(
  given: Record<string, string> = {},
  kept: string | null = null,
  bound = "",
): Made {
  const network = new FakeNetwork();
  const store = new FakeStore();
  const clock = new FakeClock();
  const launch = new FakeLaunch(given);
  if (kept !== null) store.put(SERVER_KEY, kept);
  const host = new Host({ network, store, clock, player: new NoPlayer(), launch }, bound);
  return { host, network, store, clock, launch, seen: () => host.seen() };
}

const LAUNCHED = { [SERVER_PARAMETER]: "https://streams.example" };

/** A started host that has its server and has been handed its home page. */
function running(): Made {
  const all = made(LAUNCHED);
  all.host.start();
  all.network.answer("/v1/nav", 200, NAV);
  all.network.answer("/v1/screens/home", 200, HOME);
  return all;
}

describe("a run with no address", () => {
  it("has no server, and asks nothing of anybody", () => {
    const all = made();
    all.host.start();
    expect(all.seen().server).toBe("");
    expect(all.network.asked).toHaveLength(0);
  });

  it("draws nothing rather than a screen it has not got", () => {
    // `main.tsx` draws the background alone for this state. The head of `src/door.ts` is why a run
    // with no server draws nothing.
    const all = made();
    all.host.start();
    expect(all.seen().route).toBeNull();
    expect(all.seen().lost).toBe(false);
    expect(all.seen().refused).toBe("");
  });
});

describe("where the server's address comes from", () => {
  it("is the launch parameter, and it is kept for the next run", () => {
    const all = made(LAUNCHED);
    all.host.start();
    expect(all.seen().server).toBe("https://streams.example");
    expect(all.store.get(SERVER_KEY)).toBe("https://streams.example");
  });

  it("is the kept one when this launch carried none", () => {
    const all = made({}, "https://kept.example");
    all.host.start();
    expect(all.seen().server).toBe("https://kept.example");
  });

  it("falls back to the bound build address when neither launch nor kept carried one", () => {
    const all = made({}, null, "https://bound.example/");
    all.host.start();
    expect(all.seen().server).toBe("https://bound.example");
    expect(all.network.paths()).toContain("https://bound.example/v1/screens/home");
  });

  it("lets launch and kept addresses outrank the bound build address", () => {
    const keptWins = made({}, "https://kept.example", "https://bound.example");
    keptWins.host.start();
    expect(keptWins.seen().server).toBe("https://kept.example");

    const launchWins = made(LAUNCHED, "https://kept.example", "https://bound.example");
    launchWins.host.start();
    expect(launchWins.seen().server).toBe("https://streams.example");
  });

  it("is asked for the moment it is known, with no screen in between", () => {
    const all = made(LAUNCHED);
    all.host.start();
    const asked = all.network.paths();
    expect(asked).toContain("https://streams.example/v1/strings/en");
    expect(asked).toContain("https://streams.example/v1/nav");
    expect(asked).toContain("https://streams.example/v1/screens/home");
  });

  it("asks in the language the launch named", () => {
    const all = made({ ...LAUNCHED, [LANGUAGE_PARAMETER]: "ar" });
    all.host.start();
    expect(all.network.paths()).toContain("https://streams.example/v1/strings/ar");
  });

  it("starts at the home page and not at the door, because this family has no door", () => {
    const all = made(LAUNCHED);
    all.host.start();
    expect(all.seen().route).toEqual({ target: "home" });
  });
});

describe("an address this app will not take", () => {
  it("is refused rather than used, and is said rather than swallowed", () => {
    const all = made({ [SERVER_PARAMETER]: "http://streams.example" });
    all.host.start();
    expect(all.seen().server).toBe("");
    expect(all.seen().refused).toBe("http://streams.example");
    expect(all.network.asked).toHaveLength(0);
  });

  it("but plain http to the viewer's own network is taken, and marked", () => {
    const all = made({ [SERVER_PARAMETER]: "http://192.168.1.4:8080" });
    all.host.start();
    expect(all.seen().server).toBe("http://192.168.1.4:8080");
    expect(all.seen().serverPlain).toBe(true);
  });

  it("does not take the old server's menu after being re-pointed at another one", () => {
    // The menu belongs to a server, not to a page, so it survives every move between pages - and
    // that is exactly why it has to be thrown away when the server changes underneath it. Tabs
    // from the old server lead nowhere on the new one.
    const all = made(LAUNCHED);
    all.host.start();
    all.launch.set({ [SERVER_PARAMETER]: "https://other.example" });
    all.host.start();
    expect(all.seen().nav).toHaveLength(0);
    all.network.answer("https://streams.example/v1/nav", 200, NAV);
    expect(all.seen().nav).toHaveLength(0);
    all.network.answer("https://other.example/v1/nav", 200, NAV);
    expect(all.seen().nav).toHaveLength(2);
  });

  it("and a good address afterwards clears the refusal", () => {
    const all = made({ [SERVER_PARAMETER]: "http://streams.example" });
    all.host.start();
    all.launch.set({ [SERVER_PARAMETER]: "https://streams.example" });
    all.host.retry();
    expect(all.seen().refused).toBe("");
    expect(all.seen().serverPlain).toBe(false);
  });
});

describe("what fills Seen", () => {
  it("is the answer, parsed into the field the page draws from", () => {
    const all = running();
    expect(all.seen().nav).toHaveLength(2);
    expect(all.seen().screen?.name).toBe("home");
    expect(all.seen().screen?.sections).toHaveLength(2);
  });

  it("and nothing while the answer is still coming", () => {
    const all = made(LAUNCHED);
    all.host.start();
    expect(all.seen().screen).toBeNull();
    expect(all.seen().reading).toBe(true);
  });

  it("stops reading once the page it is on has what it needs", () => {
    const all = running();
    expect(all.seen().reading).toBe(false);
  });

  it("asks for the title's own page when a title is opened", () => {
    const all = running();
    all.host.go(ADDRESS.TITLE + "movie-1001");
    expect(all.network.paths()).toContain("https://streams.example/v1/titles/movie/1001");
    all.network.answer("/v1/titles/movie/1001", 200, FILM);
    expect(all.seen().detail).toBeDefined();
    expect(all.seen().detail?.title).toBe("movie:1001");
  });
});

describe("a page draws only the data whose answer was for the state it is on", () => {
  it("so the last page's rows are gone the instant the viewer leaves it", () => {
    // The half of `tv/title/Episodes.kt:153-168` that a late-answer guard alone does not cover:
    // what a viewer actually saw was the previous page's data DURING the read, not after it.
    const all = running();
    expect(all.seen().screen).toBeDefined();
    all.host.go(ADDRESS.MY_LIST);
    expect(all.seen().screen).toBeNull();
    expect(all.seen().reading).toBe(true);
  });

  it("and the last title's details are gone the instant another title is opened", () => {
    const all = running();
    all.host.go(ADDRESS.TITLE + "movie-1001");
    all.network.answer("/v1/titles/movie/1001", 200, FILM);
    expect(all.seen().detail?.title).toBe("movie:1001");
    all.host.go(ADDRESS.TITLE + "tv-1001");
    expect(all.seen().detail).toBeNull();
  });

  it("drops an answer that arrives for a page the viewer has already left", () => {
    // This is `cardsOf(arrived, season)` at the host: the answer is for a state that has moved on,
    // so it is not drawn, however good it is.
    const all = running();
    all.host.go(ADDRESS.TITLE + "movie-1001");
    all.host.go(ADDRESS.TITLE + "tv-1001");
    all.network.answer("/v1/titles/movie/1001", 200, FILM);
    expect(all.seen().detail).toBeNull();
  });

  it("and takes the one that was for the page it is on, however late the other was", () => {
    const all = running();
    all.host.go(ADDRESS.TITLE + "movie-1001");
    all.host.go(ADDRESS.TITLE + "tv-1001");
    all.network.answer("/v1/titles/tv/1001", 200, SERIES);
    all.network.answer("/v1/titles/movie/1001", 200, FILM);
    expect(all.seen().detail?.title).toBe("tv:1001");
  });

  it("does not stop reading because a page it has left finally answered", () => {
    const all = running();
    all.host.go(ADDRESS.TITLE + "movie-1001");
    all.host.go(ADDRESS.TITLE + "tv-1001");
    all.network.answer("/v1/titles/movie/1001", 200, FILM);
    expect(all.seen().reading).toBe(true);
  });
});

describe("when the server cannot be reached", () => {
  it("says so at once when the read fails, which is what a wrong address looks like", () => {
    const all = made(LAUNCHED);
    all.host.start();
    all.network.nothing("/v1/screens/home");
    expect(all.seen().lost).toBe(true);
  });

  it("says so when the server answers with an error", () => {
    const all = made(LAUNCHED);
    all.host.start();
    all.network.answer("/v1/screens/home", 500, "");
    expect(all.seen().lost).toBe(true);
  });

  it("says so when the server takes the connection and never answers", () => {
    const all = made(LAUNCHED);
    all.host.start();
    expect(all.seen().lost).toBe(false);
    all.clock.pass();
    expect(all.seen().lost).toBe(true);
  });

  it("waits the television's own wait before it says so", () => {
    // Not `toContain(LOST_MS)`: that is a number asserted against itself, which passes forever
    // (`apps/android/surfaces/tv/src/test/kotlin/streams/tv/NumbersTest.kt:20`, and the head of
    // `src/tests/numbers.test.ts`). It survived a mutation that changed the constant. So the wait
    // this host starts is held to the Kotlin the number was derived from, read as text: the
    // television waits twice its opening's own patience, and a second surface waiting a different
    // length for the same server would be a difference nobody chose.
    const opening = readFileSync(
      join(ROOT, "apps", "android", "surfaces", "tv", "src", "main", "kotlin", "streams", "tv", "app", "Opening.kt"),
      "utf8",
    );
    const found = /val WAIT_AT_MOST = ([0-9.]+)f/.exec(opening);
    expect(found).toBeDefined();
    const seconds = Number(found?.[1] ?? 0);
    expect(seconds).toBeGreaterThan(0);

    const all = made(LAUNCHED);
    all.host.start();
    const waits = all.clock.waiting.map((one) => one.ms);
    expect(waits).toEqual([2 * seconds * 1000]);
    expect(LOST_MS).toBe(2 * seconds * 1000);
  });

  it("does not say so when the answer arrived first", () => {
    const all = running();
    all.clock.pass();
    expect(all.seen().lost).toBe(false);
  });

  it("does not let a wait belonging to a page already left say it", () => {
    const all = made(LAUNCHED);
    all.host.start();
    all.host.go(ADDRESS.MY_LIST);
    all.network.answer("/v1/screens/my-list", 200, fixture("Screen.my_list.json"));
    all.clock.pass();
    expect(all.seen().lost).toBe(false);
  });

  it("and does not hand one page's patience to the page that followed it", () => {
    // The case the guard inside the wait is actually for, and it took a surviving mutation to
    // find it. A page the viewer left while it was still reading leaves a wait behind; the page
    // they are on now is reading too, and is seconds old. Without the guard the old wait comes
    // due, sees that something is being read, and says the app is lost - about a read that has
    // had no time at all.
    //
    // It needs a clock that ignores `stop`, because `arrive` stops the old wait and that is what
    // keeps the guard out of reach on an ordinary one. Whether a Roku can cancel a timer at all
    // was not established (`src/adapters.ts`, `TimerClock`), so this is not a contrived device.
    const network = new FakeNetwork();
    const clock = new UnstoppableClock();
    const host = new Host({
      network,
      store: new FakeStore(),
      clock,
      player: new NoPlayer(),
      launch: new FakeLaunch(LAUNCHED),
    });
    host.start();
    host.go(ADDRESS.MY_LIST);
    clock.passOne();
    expect(host.seen().reading).toBe(true);
    expect(host.seen().lost).toBe(false);
  });

  it("not even when the page it left has long since answered", () => {
    const network = new FakeNetwork();
    const clock = new UnstoppableClock();
    const host = new Host({
      network,
      store: new FakeStore(),
      clock,
      player: new NoPlayer(),
      launch: new FakeLaunch(LAUNCHED),
    });
    host.start();
    host.go(ADDRESS.MY_LIST);
    network.answer("/v1/screens/my-list", 200, fixture("Screen.my_list.json"));
    clock.pass();
    expect(host.seen().lost).toBe(false);
    expect(host.seen().screen?.name).toBe("my-list");
  });

  it("is not lost any more once it is asked again and answers", () => {
    const all = made(LAUNCHED);
    all.host.start();
    all.network.nothing("/v1/screens/home");
    expect(all.seen().lost).toBe(true);
    all.host.retry();
    all.network.answer("/v1/screens/home", 200, HOME);
    expect(all.seen().lost).toBe(false);
    expect(all.seen().screen?.name).toBe("home");
  });

  it("retries a bound server when lost and recovers once it answers", () => {
    const all = made({}, null, "https://bound.example");
    all.host.start();
    all.network.nothing("/v1/screens/home");
    expect(all.seen().lost).toBe(true);
    all.host.retry();
    all.network.answer("/v1/screens/home", 200, HOME);
    expect(all.seen().lost).toBe(false);
    expect(all.seen().screen?.name).toBe("home");
  });

  it("treats an answer it cannot read as an answer it has not got", () => {
    const all = made(LAUNCHED);
    all.host.start();
    all.network.answer("/v1/screens/home", 200, "{not json");
    expect(all.seen().lost).toBe(true);
    expect(all.seen().screen).toBeNull();
  });

  it("answers a screen's own read with a status of nothing when there is no server", () => {
    const all = made();
    all.host.start();
    let got = -1;
    all.host.ask("/v1/nav", (status) => {
      got = status;
    });
    expect(got).toBe(0);
    expect(all.network.asked).toHaveLength(0);
  });
});

describe("the words of the interface", () => {
  it("arrive by key, with the language and its direction", () => {
    const all = made({ ...LAUNCHED, [LANGUAGE_PARAMETER]: "ar" });
    all.host.start();
    all.network.answer("/v1/strings/ar", 200, ARABIC);
    expect(all.seen().language).toBe("ar");
    expect(all.seen().rtl).toBe(true);
    expect(all.seen().words["common.play"]).toBeDefined();
  });

  it("fall back to English for a language this server has not got", () => {
    const all = made({ ...LAUNCHED, [LANGUAGE_PARAMETER]: "cy" });
    all.host.start();
    all.network.answer("/v1/strings/cy", 404, "");
    expect(all.network.paths()).toContain("https://streams.example/v1/strings/en");
    all.network.answer("/v1/strings/en", 200, ENGLISH);
    expect(all.seen().language).toBe("en");
    expect(all.seen().rtl).toBe(false);
  });

  it("do not fall back for ever when English is the one that failed", () => {
    const all = made(LAUNCHED);
    all.host.start();
    all.network.answer("/v1/strings/en", 500, "");
    const asked = all.network.paths().filter((one) => one.indexOf("/v1/strings/") >= 0);
    expect(asked).toHaveLength(1);
  });

  it("are not drawn half way when the answer is not readable", () => {
    const all = made(LAUNCHED);
    all.host.start();
    all.network.answer("/v1/strings/en", 200, "{not json");
    expect(all.seen().words).toEqual({});
  });
});

describe("where the viewer has been", () => {
  it("is walked out along by Back", () => {
    const all = running();
    all.host.go(ADDRESS.TITLE + "movie-1001");
    all.host.back();
    expect(all.seen().route).toEqual({ target: "home" });
  });

  it("passes over a page that was swapped in, which is what swap is for", () => {
    const all = running();
    all.host.go(ADDRESS.MY_LIST);
    all.host.swap(ADDRESS.NEW);
    all.host.back();
    expect(all.seen().route).toEqual({ target: "home" });
  });

  it("ends at the home page rather than nowhere", () => {
    const all = running();
    all.host.back();
    expect(all.seen().route).toEqual({ target: "home" });
  });
});

describe("a change the viewer made", () => {
  it("is posted as a batch of one, because there is no core here to queue it", () => {
    const all = running();
    all.host.act({ action: "command", command: { command: "list", in_list: true, title: "movie:1001" } });
    const sent = all.network.asked[all.network.asked.length - 1];
    expect(sent?.call.method).toBe("POST");
    expect(sent?.call.url).toBe("https://streams.example/v1/commands");
    expect(JSON.parse(sent?.call.body ?? "null")).toEqual({
      ops: [{ command: "list", in_list: true, title: "movie:1001" }],
    });
  });

  it("names an action this host cannot carry out rather than dropping it", () => {
    const all = running();
    const before = all.network.asked.length;
    all.host.act({ action: "sign_out", everywhere: false });
    expect(all.host.refusedActions).toContain("sign_out");
    expect(all.network.asked).toHaveLength(before);
  });
});

describe("the screens are told when the state moves", () => {
  it("at once, and with the state as it is now", () => {
    const all = made(LAUNCHED);
    const saw: string[] = [];
    all.host.watch((seen) => saw.push(seen.server));
    expect(saw).toHaveLength(1);
    all.host.start();
    expect(saw[saw.length - 1]).toBe("https://streams.example");
  });
});
