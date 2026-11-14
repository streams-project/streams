// What the Roku sources and their tests use from outside themselves, declared here because this
// folder has no `node_modules` and must not grow one.
//
// `tools/check_roku.py` type-checks this folder with `tsconfig.check.json`, which carries this
// file. Everything declared below is either a Node built-in the tests call or a name the test
// runner's harness puts on `globalThis` before a suite loads. Installing `@types/node` and a test
// framework's types would mean a `package.json` and a lockfile in a repository whose whole
// discipline is not having one, so the four Node names the suites actually use are written out
// here instead, narrowed to the one call shape each site uses.
//
// This file declares no module of its own on purpose: a top level `import` or `export` would turn
// it into a module and the global names below would stop being global.
//
// The matcher list is not free. `check_roku.py` reads the matchers out of its own harness and out
// of every suite, and fails when a suite uses one the harness has not got - so a matcher added
// here without being added there is a red run, not a silent `undefined is not a function`.

/** The repository root, reached from a compiled suite by `join(__dirname, "..", ...)`. */
declare const __dirname: string;

/**
 * `fetch`, narrowed to the one call shape `src/adapters.ts` uses.
 *
 * Not an SDK API and not an invention: Roku's RSG 2.0 Q&A says network calls are ordinary
 * `async`/`await` with `fetch`, that the beta supports basic fetch, and that streaming is not
 * there yet. **Assumed, not verified**: that is a search of Roku's own domains on 2026-11-14 and
 * the recap page itself was not opened. If the spelling differs, this declaration and
 * `FetchNetwork` are the two places that change. What is declared here is basic fetch and
 * nothing beyond it - no `Request`, no `Headers`, no `AbortSignal`, no streaming body - because a
 * declaration wider than what was read is a type check that reads green against an API nobody has
 * seen. `AbortController` is deliberately absent: that recap names it as something Roku wants
 * natively, which is not the same as having it, and `FetchNetwork` races a clock instead.
 */
declare function fetch(
  url: string,
  options?: { method: string; headers: Record<string, string>; body?: string },
): Promise<{ status: number; text(): Promise<string> }>;

/**
 * Timers, narrowed the same way and for the same reason. Whether a Roku runs timers for a
 * TypeScript app was not established on any page read; `src/adapters.ts` says so at the one place
 * that calls them.
 */
declare function setTimeout(run: () => void, ms: number): number;
declare function clearTimeout(token: number): void;

declare module "node:fs" {
  /** Only the one call shape the suites use: a whole file, as text. */
  export function readFileSync(path: string, encoding: "utf8"): string;
}

declare module "node:path" {
  export function join(...parts: string[]): string;
}

/** A group of tests. `node:test` exports one of these; the harness makes it global. */
declare function describe(name: string, body: () => void): void;

/** One test. */
declare function it(name: string, body: () => void): void;

/**
 * The assertions the suites use, and nothing else.
 *
 * `toBe` and `toEqual` take `unknown` rather than the asserted value's own type, the way Jest's own
 * declarations do: narrowing them turns a legitimate comparison against a wider or narrower literal
 * into a type error that says nothing about the code under test.
 */
interface Matchers {
  /** `Object.is`. */
  toBe(expected: unknown): void;
  /** Deep equality, with own properties whose value is `undefined` ignored on both sides. */
  toEqual(expected: unknown): void;
  toBeNull(): void;
  toBeDefined(): void;
  toBeUndefined(): void;
  toBeGreaterThan(expected: number): void;
  toBeGreaterThanOrEqual(expected: number): void;
  toBeLessThan(expected: number): void;
  toBeLessThanOrEqual(expected: number): void;
  /** `includes`, on a string or on an array. */
  toContain(expected: unknown): void;
  toMatch(expected: RegExp | string): void;
  toHaveLength(expected: number): void;
}

declare function expect(actual: unknown): Matchers & { not: Matchers };
