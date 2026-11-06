// Drives page/host.js with the real core.wasm and a scripted network. It covers what a browser run
// cannot reach on demand: failed requests, a step asked for from inside a step, and a trap.
//
// Build the core first, as ./up does, and name it in CORE_WASM if it is not in ../../target.

import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { test } from "node:test";
import { createContext, runInContext } from "node:vm";

const ROOT = resolve(import.meta.dirname, "../../..");
const TARGET = process.env.CARGO_TARGET_DIR ?? join(ROOT, "target");
const WASM = process.env.CORE_WASM ?? join(TARGET, "wasm32v1-none/wasm/bridge_wasm.wasm");
const SOURCE = readFileSync(join(ROOT, "apps/tv/page/host.js"), "utf8");
// The fixture sets a version floor that this page is below. The real server sends none.
const HELLO = JSON.stringify({
  ...JSON.parse(readFileSync(join(ROOT, "platform/contract/fixtures/2026-11-05/valid/Hello.basic.json"), "utf8")),
  client_min: [],
});

type Answer = { status: number; text: string } | "offline" | "timeout";
type Sent = { method: string; url: string; headers: Record<string, string>; timeout: number };

/** One page: the host with a fake window, a scripted network and a counter around each step. */
function page(wasm: ArrayBuffer | null, answers: (url: string) => Answer | null) {
  const stored: Record<string, string> = {};
  const sent: Sent[] = [];
  const held: (() => void)[] = [];
  const views: any[] = [];
  const failures: string[] = [];
  const inputs: any[] = [];
  const state = { depth: 0, deepest: 0, steps: 0, trapNext: false };

  class FakeRequest {
    onload?: () => void;
    onerror?: () => void;
    ontimeout?: () => void;
    onabort?: () => void;
    responseType = "";
    timeout = 0;
    status = 0;
    response: ArrayBuffer | null = null;
    responseText = "";
    url = "";
    method = "";
    headers: Record<string, string> = {};
    open(method: string, url: string) { this.method = method; this.url = url; }
    setRequestHeader(name: string, value: string) { this.headers[name] = value; }
    send() {
      sent.push({ method: this.method, url: this.url, headers: this.headers, timeout: this.timeout });
      if (this.url === "core.wasm") {
        queueMicrotask(() => {
          this.status = 200;
          this.response = wasm;
          this.onload?.();
        });
        return;
      }
      const answer = answers(this.url);
      if (answer === null) return;
      held.push(() => {
        if (answer === "offline") this.onerror?.();
        else if (answer === "timeout") this.ontimeout?.();
        else {
          this.status = answer.status;
          this.responseText = answer.text;
          this.onload?.();
        }
      });
    }
  }

  // The real module, with a count of nested steps and a way to make the next one trap.
  const wrapped = {
    ...WebAssembly,
    instantiate: async (bytes: ArrayBuffer) => {
      const made = await WebAssembly.instantiate(bytes, {});
      const real = made.instance.exports as any;
      const exports = { ...real };
      exports.core_step = (...args: number[]) => {
        state.depth += 1;
        state.deepest = Math.max(state.deepest, state.depth);
        state.steps += 1;
        if (state.trapNext) {
          state.trapNext = false;
          state.depth -= 1;
          throw new WebAssembly.RuntimeError("unreachable");
        }
        const text = new TextDecoder().decode(
          new Uint8Array(real.memory.buffer, args[1], args[2]),
        );
        inputs.push(JSON.parse(text));
        const status = real.core_step(...args);
        state.depth -= 1;
        return status;
      };
      return { instance: { exports } };
    },
  };

  const window: any = {
    crypto: {
      getRandomValues: (bytes: Uint8Array) => crypto.getRandomValues(bytes),
    },
    localStorage: {
      getItem: (key: string) => (key in stored ? stored[key] : null),
      setItem: (key: string, value: string) => { stored[key] = value; },
      removeItem: (key: string) => { delete stored[key]; },
    },
    setTimeout: (work: () => void, after: number) => {
      const timer = setTimeout(work, after);
      timer.unref();
      return timer;
    },
    clearTimeout,
    addEventListener: () => {},
  };
  const context = createContext({
    window,
    document: { addEventListener: () => {}, hidden: false },
    console: { log: () => {} },
    performance: { now: () => performance.now() + 0.25 },
    XMLHttpRequest: FakeRequest,
    WebAssembly: wrapped,
    TextEncoder,
    TextDecoder,
    Uint8Array,
    DataView,
  });
  runInContext(SOURCE, context);
  const ui = {
    version: "0.1.0",
    os: "test",
    view: (view: any) => { views.push(view); },
    failed: (word: string) => { failures.push(word); },
  };
  return { host: window.tvHost, ui, sent, held, views, failures, inputs, stored, state };
}

function routes(views: any[]): string[] {
  return views.filter((view) => view.route).map((view) => view.route.target);
}

/** Let the module load and the first answers land. */
async function settled(open: ReturnType<typeof page>) {
  for (let round = 0; round < 50; round += 1) {
    await new Promise((done) => setImmediate(done));
    while (open.held.length > 0) open.held.shift()?.();
  }
}

function bytes(): ArrayBuffer {
  const file = readFileSync(WASM);
  return file.buffer.slice(file.byteOffset, file.byteOffset + file.length) as ArrayBuffer;
}

const server = (url: string): Answer =>
  url === "/v1/hello" ? { status: 200, text: HELLO } : { status: 404, text: "" };

const skip = existsSync(WASM) ? false : `no core at ${WASM}`;

test("the contract date here is the core's own", () => {
  const version = readFileSync(join(ROOT, "platform/contract/src/version.rs"), "utf8");
  const date = /"(\d{4}-\d{2}-\d{2})"/.exec(version)?.[1];
  assert.ok(SOURCE.includes(`var CONTRACT = "${date}"`), `host.js should say ${date}`);
});

test("the real core reaches start, as class tv, with whole numbers for time", { skip }, async () => {
  const open = page(bytes(), server);
  open.host.start(open.ui);
  await settled(open);
  assert.equal(open.failures.length, 0);
  assert.deepEqual(routes(open.views), ["wait", "start"]);
  const hello = open.sent.find((request) => request.url === "/v1/hello");
  assert.equal(hello?.headers["Client-Class"], "tv");
  assert.equal(hello?.headers["Client-App"], "tv");
  assert.equal(hello?.timeout, 60000);
  for (const input of open.inputs) {
    assert.ok(Number.isInteger(input.now_ms) && Number.isInteger(input.mono_ms));
  }
});

test("secrets carry the tv prefix, so the website's session is not read", { skip }, async () => {
  const open = page(bytes(), server);
  open.stored["secret:session"] = "the website's";
  open.host.start(open.ui);
  await settled(open);
  assert.deepEqual(routes(open.views), ["wait", "start"]);
  const kept = Object.keys(open.stored).filter((key) => key !== "secret:session");
  assert.ok(kept.length > 0, "the device key is kept");
  assert.ok(kept.every((key) => key.startsWith("tv:secret:")), kept.join());
});

test("a step asked for inside a step waits its turn", { skip }, async () => {
  const open = page(bytes(), server);
  open.ui.view = (view: any) => {
    open.views.push(view);
    open.host.go("/");
  };
  open.host.start(open.ui);
  await settled(open);
  assert.ok(open.state.steps > 2);
  assert.equal(open.state.deepest, 1);
});

test("a reply fires on an answer, a failure, a time out and a path the core drops", { skip }, async () => {
  const open = page(bytes(), (url) => {
    if (url === "/v1/gone") return "offline";
    if (url === "/v1/slow") return "timeout";
    if (url === "/v1/refused") return { status: 403, text: "no" };
    return server(url);
  });
  open.host.start(open.ui);
  await settled(open);
  const got: Record<string, [number, string]> = {};
  const ask = (path: string) =>
    open.host.ask("GET", path, null, (status: number, text: string) => { got[path] = [status, text]; });
  ask("/v1/gone");
  ask("/v1/slow");
  ask("/v1/refused");
  ask("/outside");
  await settled(open);
  assert.deepEqual(got["/v1/gone"], [0, ""]);
  assert.deepEqual(got["/v1/slow"], [0, ""]);
  assert.deepEqual(got["/v1/refused"], [403, "no"]);
  await new Promise((done) => setTimeout(done, 5));
  assert.deepEqual(got["/outside"], [0, ""]);
  const answers = open.inputs.flatMap((input) => input.events);
  assert.ok(answers.some((event) => event.type === "http_failed" && event.reason === "offline"));
  assert.ok(answers.some((event) => event.type === "http_failed" && event.reason === "timeout"));
});

test("a hello that fails is answered, and the retry waits for the core's wake", { skip }, async () => {
  let tries = 0;
  const open = page(bytes(), () => {
    tries += 1;
    return tries === 1 ? "offline" : { status: 200, text: HELLO };
  });
  open.host.start(open.ui);
  await settled(open);
  const events = open.inputs.flatMap((input) => input.events);
  assert.ok(events.some((event) => event.type === "http_failed"));
  assert.equal(tries, 1, "the second try waits for the wake the core asked for");
});

test("a page in place of the core file shows as no_file", async () => {
  const fallback = new TextEncoder().encode("<!doctype html><html></html>").buffer as ArrayBuffer;
  const open = page(fallback, server);
  open.host.start(open.ui);
  await settled(open);
  assert.deepEqual(open.failures, ["no_file"]);
  assert.equal(open.views.length, 0);
});

test("a trap starts a new core, which returns to the last address", { skip }, async () => {
  const open = page(bytes(), server);
  open.host.start(open.ui);
  await settled(open);
  const pending = new Promise<number>((done) => {
    open.host.ask("GET", "/v1/never", null, done);
  });
  open.state.trapNext = true;
  open.host.go("/sign-in");
  assert.equal(await pending, 0, "a waiting reply is told");
  await settled(open);
  assert.deepEqual(open.failures, []);
  assert.deepEqual(routes(open.views).slice(-2), ["wait", "sign_in"]);
});

test("a core that keeps trapping ends in a notice, with no loop", { skip }, async () => {
  const open = page(bytes(), server);
  open.host.start(open.ui);
  await settled(open);
  for (let round = 0; round < 4; round += 1) {
    open.state.trapNext = true;
    open.host.go("/");
    await settled(open);
  }
  assert.deepEqual(open.failures, ["stopped"]);
});
