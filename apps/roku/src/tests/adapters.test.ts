// The adapters, as far as a machine with no Roku can take them.
//
// Three of the five are written against things that are not a Roku API - `fetch`, a map, and
// string handling - so three of them can be run here. The other two refuse on purpose and the
// cases below say what a refusal looks like, because a refusal that is not visible is the same as
// a port that silently swallows what a screen asked for.
//
// `FetchNetwork` is handed its `fetch` rather than reaching for the global, so what is checked
// here is the part a reader cannot verify by looking: the deadline race, and that `done` runs
// exactly once however the call ends.

import { FetchNetwork, GivenLaunch, NoLaunch, NoPlayer, ServerPictures, StandingStore } from "../adapters";
import { serverFrom } from "../door";
import type { Answer, Call, Clock, Stop } from "../ports";

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

  pass(): void {
    const due = this.waiting;
    this.waiting = [];
    for (const one of due) if (!one.stopped) one.run();
  }
}

/** A `fetch` a test drives: it hands back a promise and the two ways to settle it. */
function waitingFetch(): {
  ask: typeof fetch;
  answer: (status: number, text: string) => void;
  fail: (reason: string) => void;
  seen: { url: string; options?: { method: string; headers: Record<string, string>; body?: string } }[];
} {
  const seen: {
    url: string;
    options?: { method: string; headers: Record<string, string>; body?: string };
  }[] = [];
  let settle: (answer: { status: number; text(): Promise<string> }) => void = () => {};
  let refuse: (reason: unknown) => void = () => {};
  const ask = ((url: string, options?: { method: string; headers: Record<string, string>; body?: string }) => {
    seen.push(options === undefined ? { url } : { url, options });
    return new Promise<{ status: number; text(): Promise<string> }>((keep, drop) => {
      settle = keep;
      refuse = drop;
    });
  }) as typeof fetch;
  return {
    ask,
    seen,
    answer: (status: number, text: string) =>
      settle({ status, text: () => Promise.resolve(text) }),
    fail: (reason: string) => refuse(new Error(reason)),
  };
}

const CALL: Call = {
  method: "GET",
  url: "https://streams.example/v1/nav",
  headers: {},
  body: null,
  deadlineMs: 10000,
};

/** A promise's `then` runs on the microtask queue, so a case that reads the answer waits for it. */
function settled(check: () => void): Promise<void> {
  return Promise.resolve()
    .then(() => undefined)
    .then(() => undefined)
    .then(check);
}

describe("the network, over fetch", () => {
  it("sends the method, the address and the headers it was given", () => {
    const clock = new FakeClock();
    const asked = waitingFetch();
    new FetchNetwork(clock, asked.ask).send(
      { ...CALL, method: "POST", headers: { "content-type": "application/json" }, body: "{}" },
      () => {},
    );
    expect(asked.seen).toHaveLength(1);
    expect(asked.seen[0]?.url).toBe("https://streams.example/v1/nav");
    expect(asked.seen[0]?.options?.method).toBe("POST");
    expect(asked.seen[0]?.options?.body).toBe("{}");
  });

  it("sends no body at all for a read, rather than an empty one", () => {
    const asked = waitingFetch();
    new FetchNetwork(new FakeClock(), asked.ask).send(CALL, () => {});
    expect(asked.seen[0]?.options?.body).toBeUndefined();
  });

  it("starts the deadline the call carried, and not one of its own", () => {
    const clock = new FakeClock();
    new FetchNetwork(clock, waitingFetch().ask).send({ ...CALL, deadlineMs: 1234 }, () => {});
    expect(clock.waiting.map((one) => one.ms)).toEqual([1234]);
  });

  it("answers once the server has answered, and stops the deadline", () => {
    const clock = new FakeClock();
    const asked = waitingFetch();
    const got: Answer[] = [];
    new FetchNetwork(clock, asked.ask).send(CALL, (answer) => got.push(answer));
    asked.answer(200, "{}");
    return settled(() => {
      expect(got).toEqual([{ kind: "reply", status: 200, text: "{}" }]);
      expect(clock.waiting[0]?.stopped).toBe(true);
    });
  });

  it("answers with no answer when the deadline comes first", () => {
    const clock = new FakeClock();
    const got: Answer[] = [];
    new FetchNetwork(clock, waitingFetch().ask).send(CALL, (answer) => got.push(answer));
    clock.pass();
    expect(got).toHaveLength(1);
    expect(got[0]?.kind).toBe("none");
  });

  it("answers once and not twice when the server answers after the deadline", () => {
    // The one a reader cannot see: a read abandoned here is still running underneath, because
    // there is no AbortController to stop it with.
    const clock = new FakeClock();
    const asked = waitingFetch();
    const got: Answer[] = [];
    new FetchNetwork(clock, asked.ask).send(CALL, (answer) => got.push(answer));
    clock.pass();
    asked.answer(200, "{}");
    return settled(() => {
      expect(got).toHaveLength(1);
      expect(got[0]?.kind).toBe("none");
    });
  });

  it("answers with no answer when the read itself failed", () => {
    const clock = new FakeClock();
    const asked = waitingFetch();
    const got: Answer[] = [];
    new FetchNetwork(clock, asked.ask).send(CALL, (answer) => got.push(answer));
    asked.fail("refused");
    return settled(() => {
      expect(got).toHaveLength(1);
      expect(got[0]?.kind).toBe("none");
    });
  });
});

describe("what this app keeps", () => {
  it("holds a value and hands it back", () => {
    const store = new StandingStore();
    expect(store.get("app:server")).toBeNull();
    store.put("app:server", "https://streams.example");
    expect(store.get("app:server")).toBe("https://streams.example");
    store.delete("app:server");
    expect(store.get("app:server")).toBeNull();
  });

  it("hands back nothing for a name nothing was ever written under", () => {
    // `constructor` and `toString` are on every plain object in this language, and a store that
    // answered with one of them would hand `src/door.ts` a function where an address should be.
    const store = new StandingStore();
    expect(store.get("constructor")).toBeNull();
    expect(store.get("toString")).toBeNull();
    expect(serverFrom({}, store.get("toString")).from).toBe("none");
  });
});

describe("where a screen's pictures come from", () => {
  const pictures = new ServerPictures(() => "https://streams.example");

  it("leaves a whole address alone, which is what the catalogue sends", () => {
    expect(pictures.of("https://images.example/p/1001.jpg")).toBe(
      "https://images.example/p/1001.jpg",
    );
  });

  it("resolves a path against the server, so no screen joins two halves of an address", () => {
    expect(pictures.of("/mark.png")).toBe("https://streams.example/mark.png");
    expect(pictures.of("mark.png")).toBe("https://streams.example/mark.png");
  });

  it("answers nothing for a title with no art of that shape", () => {
    expect(pictures.of(null)).toBeNull();
    expect(pictures.of(undefined)).toBeNull();
    expect(pictures.of("")).toBeNull();
  });
});

describe("the ports nobody has plugged anything into", () => {
  it("say what was asked of them rather than swallowing it", () => {
    const player = new NoPlayer();
    const heard: string[] = [];
    player.listen((event) => heard.push(event));
    player.command({ op: "play" });
    expect(player.asked).toEqual(["play"]);
    expect(heard).toHaveLength(1);
  });

  it("and a launch with no parameters falls through to the kept address", () => {
    expect(new NoLaunch().parameters()).toEqual({});
    expect(serverFrom(new NoLaunch().parameters(), "https://kept.example").from).toBe("kept");
    expect(new GivenLaunch({ server: "https://one.example" }).parameters()).toEqual({
      server: "https://one.example",
    });
  });
});
