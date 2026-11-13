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
