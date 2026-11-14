// The adapters: the one file in this folder where a Roku is finally named.
//
// A port says what the rules need. An adapter says how a device does it, and it is the part that
// cannot be checked on a machine with no device and no SDK. So each one below says which of the
// three it is:
//
//   **written**   it uses something Roku has published and nothing else, so it can be read, run
//                 against a fake, and believed on sight;
//   **refusing**  what this port does until somebody writes it: it refuses in the open and at
//                 once. The alternative is worse - a port that quietly never answers leaves a
//                 screen waiting on a read that cannot finish, and that reads to a person as a
//                 slow network rather than as an app that was wired wrong
//                 (`apps/android/ports/src/main/kotlin/streams/ports/Ports.kt:158-164`);
//   **standing**  it works for this run of the app and not between runs, and says so.
//
// Nothing here is in the type check's blind spot: this file imports no `@roku-sdk` package, so
// `platform/tools/check_roku.py` type-checks it with everything else. The day the SDK is here, the
// two refusing adapters and the standing one are the three small files to write, and nothing else
// in this folder moves.

import type { DriverCommand } from "./contract";
import type { Answer, Call, Clock, Launch, Network, Pictures, Player, Stop, Store } from "./ports";

/**
 * **Written.** The network, over `fetch`.
 *
 * `fetch` is not an invention and not an SDK call: Roku's own RSG 2.0 Q&A says network calls use
 * ordinary `async`/`await` with `fetch`, that basic fetch is what the beta supports, and that
 * streaming is not there yet. **Assumed and not verified**: a search of Roku's own domains on
 * 2026-11-14 named the recap and the page was not opened. Basic fetch is all this
 * adapter uses, and the narrow shape it uses is declared in `types/ambient.d.ts` rather than
 * pulled in from a `@types` package this folder does not have.
 *
 * The deadline is honoured by racing the read, not by `AbortController`: that recap names
 * AbortController as something Roku **wants** natively, which is not the same as having it. A read
 * that is abandoned here is still running underneath; what is promised to the caller is that the
 * answer arrives once and on time, and that is what is kept.
 */
export class FetchNetwork implements Network {
  /**
   * `ask` defaults to the global and is handed in by a test.
   *
   * It is a parameter rather than a call to the global because this is the one place in the
   * folder that touches a global at all, and the behaviour worth checking here is not `fetch` -
   * it is the deadline race and the once-only answer around it, neither of which a reader can
   * verify by looking.
   */
  constructor(
    private readonly clock: Clock,
    private readonly ask: typeof fetch = fetch,
  ) {}

  send(call: Call, done: (answer: Answer) => void): void {
    let answered = false;
    const once = (answer: Answer): void => {
      if (answered) return;
      answered = true;
      timer.stop();
      done(answer);
    };
    const timer = this.clock.after(call.deadlineMs, () => {
      once({ kind: "none", reason: "the server did not answer in time" });
    });
    const asked: { method: string; headers: Record<string, string>; body?: string } = {
      method: call.method,
      headers: call.headers,
    };
    if (call.body !== null) asked.body = call.body;
    this.ask(call.url, asked).then(
      (answer) => {
        answer.text().then(
          (text) => once({ kind: "reply", status: answer.status, text }),
          () => once({ kind: "none", reason: "the answer could not be read" }),
        );
      },
      (reason: unknown) => once({ kind: "none", reason: String(reason) }),
    );
  }
}

/**
 * **Standing.** What this app keeps, for as long as this run of it lasts.
 *
 * A Roku's own persistent store is the registry, and the registry's TypeScript API was not
 * published on any page read: Roku's documents describe `roRegistry` in BrightScript, and the Q&A
 * says a flush in either language flushes both, which says the store is shared and does not say
 * what to call. So the address of the server survives a move between screens and does not survive
 * a power cut, and the television is handed its address again at the next launch
 * (`src/door.ts`). This is the one adapter whose absence a viewer would notice, and it is the
 * cheapest of the three to write once the API is known.
 */
export class StandingStore implements Store {
  private readonly held: Record<string, string> = {};

  get(key: string): string | null {
    // Read as a value and not as a presence: `held` is a plain object, so `held["toString"]` is a
    // function off the prototype and `?? null` would hand it back as an address.
    const held = this.held[key];
    return typeof held === "string" ? held : null;
  }

  put(key: string, value: string): void {
    this.held[key] = value;
  }

  delete(key: string): void {
    delete this.held[key];
  }
}

/**
 * **Written.** Where a screen's pictures come from.
 *
 * This port is one call on Roku and six on Android, and that is a real difference between the two
 * devices rather than a thing left out. A Roku draws a picture by handing a Poster node a `uri`
 * and nothing else: there is no fetch that gives the pixels back, so there is nothing for a
 * `warm`, a `Want` or a `Stop` to mean (`ports/Ports.kt:128-156`).
 *
 * What is left for it to do is the one thing the screens cannot: an address the server gave as a
 * path and not as a whole address is resolved here against the server, so no screen in this folder
 * ever joins two halves of a URL.
 */
export class ServerPictures implements Pictures {
  constructor(private readonly server: () => string) {}

  of(url: string | null | undefined): string | null {
    if (url === null || url === undefined || url === "") return null;
    if (/^[a-z]+:/.test(url)) return url;
    return this.server() + (url.startsWith("/") ? url : "/" + url);
  }
}

/**
 * **Written.** Time.
 *
 * `setTimeout` is a guess, and it is here rather than in `src/host.ts` so that it is one guess in
 * one place: no page of Roku's that was read says whether a TypeScript app gets timers, and the
 * Q&A that answers the network question does not answer this one. If a Roku has no `setTimeout`,
 * this class is the only thing in this folder that changes, and the behaviour it would take with
 * it is named in `src/numbers.ts` under `LOST_MS`.
 */
export class TimerClock implements Clock {
  after(ms: number, run: () => void): Stop {
    const token = setTimeout(run, ms);
    return {
      stop(): void {
        clearTimeout(token);
      },
    };
  }
}

/**
 * **Refusing.** The player.
 *
 * The player screen is not written, the Video node's field list was not read, and there is no
 * answer yet on HLS or DRM. A refusal that is collected and can be read back is the difference
 * between a port nobody has plugged in and a port that silently swallows what a screen asked for.
 */
export class NoPlayer implements Player {
  readonly asked: string[] = [];
  private told: ((event: string) => void) | null = null;

  command(given: DriverCommand): void {
    this.asked.push(given.op);
    this.told?.("no player is plugged into this app");
  }

  listen(report: (event: string) => void): void {
    this.told = report;
  }
}

/**
 * **Refusing.** The parameters this app was launched with.
 *
 * How a sideloaded RSG 2.0 app reads its launch parameters was not established on any page read,
 * so this hands back none and `src/door.ts` falls through to the kept address. It is the first
 * thing to write when the SDK is here, because with it a `rk dev` of this app reaches a server on
 * the first run and without it the app has no address at all on a fresh device.
 */
export class NoLaunch implements Launch {
  parameters(): Record<string, string> {
    return {};
  }
}

/** The parameters a run was given, for a caller that already has them. */
export class GivenLaunch implements Launch {
  constructor(private readonly given: Record<string, string>) {}

  parameters(): Record<string, string> {
    return this.given;
  }
}
