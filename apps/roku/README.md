# roku

The Roku client. **Roku forces no language on us.** It publishes two first-party routes, and this
folder takes the newer one for new code:

| Route | Language | Can publish to the Streaming Store today |
|---|---|---|
| SceneGraph + BrightScript, since 2014 | BrightScript and XML | **yes** |
| RSG 2.0 SDK, public beta since 2026-10-29 | TypeScript and TSX, running on Hermes | **no.** Sideload only. General availability targeted Fall 2027 |

Status, 2026-11-14: **33 source files and 4,546 lines, plus 17 suites and 3,061 lines of tests that
need no device. 279 of 279 tests pass and 24 of 24 mutants were killed. The host is written.** The
rules, the key maps, the geometry, the contract calls, the ports, the adapters and the host are
all here; what is missing is the SDK call that hands the whole thing to a Roku, and that is now
one line rather than a folder (`src/main.tsx`, the foot). **The folder has still not been
type-checked, because there is no TypeScript compiler on this machine.** Measured with
`cat src/*.ts src/screens/* src/main.tsx roku-config.ts | wc -l`, `cat src/tests/*.ts | wc -l` and
`grep -c "^\s*it(" src/tests/*.ts` on 2026-11-14. What is left is listed in section 5, each item
with what unblocks it.

Yesterday's line here said "the host is not written, so no screen has data". That is what changed
today: section 8 is the host, and section 4 is what was actually run to believe any of it.

Licensed under either of MIT or Apache 2.0, at your option.

---

## 1. What Roku accepts today

Read from Roku's own pages on 2026-11-12. Every line says whether it was read there or inferred.

| | Source | Status |
|---|---|---|
| "Developer applications are written using the BrightScript programming language." Apps run sandboxed in the BrightScript virtual machine and reach only the platform resources exposed as BrightScript components. | [Development environment overview](https://developer.roku.com/dev/docs/dev-environment) | verified |
| "The RSG 2.0 SDK lets you write SceneGraph components, or an entire app, in TypeScript instead of BrightScript and XML." It "does not replace BrightScript or require you to rewrite an existing app." | [RSG 2.0 SDK overview](https://developer.roku.com/dev/docs/rsg-sdk-standing-docs-getting-started-overview), page updated 2026-10-30 | verified |
| "RSG 2.0 is currently in a beta period. This means you can install RSG 2.0 SDK apps only by sideloading." "Publishing to the Roku Streaming Store is not supported yet." | same page | verified |
| "General availability is targeted for Fall 2027." | same page | verified |
| The SDK is Roku's own, open source, and its packages are public on npm: "The SDK packages are public npm packages; therefore, you do not need a registry override, a token, or an account." | [Getting started with the RSG 2.0 SDK](https://developer.roku.com/dev/docs/rsg-sdk-standing-docs-getting-started-first-app) | verified |
| It is **not** a transpiler to BrightScript. TypeScript is compiled to Hermes bytecode and run by Hermes on the device: "Sideloading Hermes bytecode with `rk dev --compile-ts` requires a device authorization token with the `rsg_typescript` claim." | same page | verified |
| SceneGraph nodes reached from TypeScript are JSI host objects, and the JS-to-render boundary is asynchronous: "Synchronous callFunc across the boundary is not possible." | [Dev Summit 2026 Q&A recap](https://forum.developer.roku.com/t/dev-summit-2026-q-a-recap/11740), 2026-10-29, answered by Roku engineers | verified (Roku's own forum, Roku staff, not a docs page) |
| TypeScript is not even forced inside the SDK: "Hermes runs plain JavaScript today, and nothing currently enforces TypeScript." Roku recommends TypeScript. | same recap | verified |
| Roku OS 16.0 or higher is required: "16.0 is required to sideload a TypeScript app." RSG 2.0 runs on every device, down to the GPU-less Liberty and Littlefield, which "cannot reach 60 fps". | [RSG 2.0 Live Q&A recap](https://forum.developer.roku.com/t/rsg-2-0-live-q-a-recap/11817), session 2026-11-11, posted 2026-11-12 | verified (Roku's own forum) |
| 4 MB maximum app package, unchanged for TypeScript apps: "The 4 MB maximum package size stays for now." | Dev Summit recap, Q53 | verified (forum) |
| Joining the developer program, building and publishing cost nothing. | Roku development overview, repeated by third parties | assumed: read in search results, not opened directly |
| TypeScript worker threads do not exist yet: "You cannot create new threads from TypeScript today." | Dev Summit recap, Q16, Chakri Kodali | verified (forum) |
| Generator functions are unsupported, because Hermes has no `function*`. | [RSG 2.0 SDK pitfalls](https://developer.roku.com/dev/docs/rsg-sdk-standing-docs-knowledge-base-rsg-ts-pitfalls) | verified |

### What this corrects

Note 170 (2026-11-06) said Roku needed "a whole new BrightScript client" and that the TypeScript
SDK "cannot publish until about September 2027".

* The publishing date was **right**: Roku's own page says Fall 2027, and the Dev Summit recap says
  "The publishing target is next September".
* "A whole new BrightScript client" is now **wrong as a statement of the only option**. Roku's own
  TypeScript SDK went to public beta on 2026-10-29, is open source, installs from public npm, and
  runs on every supported device.

### So why is this folder TypeScript?

Because a choice had to be made and this is it, **not because Roku demands it**. The reasoning, for
the owner to overturn if he wants:

* BrightScript is the only route that can publish today, and will be until about September 2027.
* Nothing in this folder can be published before then either: there is no Roku here to sideload to,
  the hero, search, profiles, player and sign-in screens are not written, and the host does not
  exist.
* So the first code written for Roku is going to be read, reviewed and rewritten before it ever
  faces the store. Written in TypeScript it also type-checks against
  `platform/contract/generated/types/contract.ts`, which is the generated contract every other
  client is held to; written in BrightScript it would check against nothing, because BrightScript
  has no types and the contract generates none for it.
* If the owner prefers BrightScript, the cost is bounded and known: the eight logic files in
  `src/` are plain functions and classes with no SDK import, and they are what would have to be
  rewritten. The four `.tsx` files are the only ones that touch the SDK at all.

**Decision, open.** The language is the owner's. Nothing here depends on it being settled today.

---

## 2. Can the shared Rust core run on a Roku?

**No, on the evidence available, and this is the one architectural question this folder raises.**
Note 215's claim stands, with one correction to how it is phrased.

| | Status |
|---|---|
| Roku's development environment page says apps are written in BrightScript, run inside the BrightScript virtual machine, and reach only "the platform resources exposed as BrightScript components". It describes no native extension, no shared library, no FFI. | verified, [dev-environment](https://developer.roku.com/dev/docs/dev-environment) |
| No page found on developer.roku.com offers a way to ship compiled native code in an app package. | verified as an **absence**: searched and read on 2026-11-12 and found nothing. Not the same as Roku stating a refusal, and no such sentence was found either |
| No WebAssembly runtime is documented on Roku, and Hermes has none. The RSG 2.0 pages name Hermes and TypeScript and never mention wasm. | assumed from the absence across the RSG 2.0 pages read |
| The only route ever demonstrated for C, C++ or Rust is **wasm2brs**, which converts WebAssembly into BrightScript source. It is a community project by MotleyCoder, posted in a thread that ran 2020-11-29 to 2021-05-26 and now sits in the forum's **Archived** category, with no Roku staff reply in it. Its own demo was the duktape interpreter compiled to BrightScript, and Rust was called an "experimental platform". | verified, [forum thread](https://forum.developer.roku.com/t/running-c-c-rust-on-roku-it-works/9656) |

### The options, and what this folder recommends

The decision is the owner's, because it changes whether the architecture's claim to be universal
holds. Three options:

**A. The core as WebAssembly.** What `apps/tv/page/host.js` did for Tizen, webOS and Vizio
until that folder was deleted on 2026-11-12 (note 224 keeps what it had worked out). On Roku it needs a wasm engine that is not documented to exist, or wasm2brs, a five-year-old
archived community transpiler, inside a 4 MB package cap, on devices Roku itself says have no GPU
and cannot reach 60 fps. **Rejected**: the dependency is dead and the budget is not there.

**B. A thin client, with the core stepped on the server.** The contract already has the route:
`POST /v1/core/step`, sending `HostedStep`, answering `CoreOut`, in the `HOSTED` auth group
(`platform/contract/src/api.rs:791-798`). It is **not built**:
`server/crates/server-api/tests/routes.rs:11` lists it as still to build, waiting on
"server-hosted". **Recommended.** The rules stay in one place, written once in Rust, and Roku
becomes a drawing surface with a network round trip per step. What it costs is latency per press,
which is measurable the moment the route answers, and a session held server-side.

**C. Roku gets no shared core, and its rules are written again in TypeScript.** Honest, shippable,
and it forks the behaviour spec: every rule the core holds would exist twice, and the second copy
would drift. **Rejected** unless B measures badly.

**Recommendation: B.** The reason is not Roku, it is the contract: the route was designed for
exactly this case and already has its types and its auth group. Building it serves every future
device that cannot run native code, and it is the only option that keeps one set of rules. The one
thing that could overturn it is latency: if a step across the network costs more than a few tens of
milliseconds per press, a row will not scroll like the Kotlin one does, and C becomes the honest
answer. That number cannot be measured until the route exists.

---

## 3. What is in this folder

Nothing here is a Roku API call except in the five `.tsx` files and `src/screens/remote.ts`. Every
other file is plain TypeScript that could be read and checked on any machine, and `src/adapters.ts`
is the one that would be a Roku API call and deliberately is not: see section 8.

| File | Lines | What it does |
|---|---|---|
| `src/numbers.ts` | 231 | Every number the surface lays out with, each beside the Kotlin line it was taken from. Corners and colours are read from `brand.json`, not transcribed |
| `src/keys.ts` | 78 | Roku's 15 documented key strings, turned into the six words a screen answers |
| `src/page.ts` | 80 | Which rows a page may draw, and whether the answer in hand is even its own |
| `src/tier.ts` | 133 | What a profile may be shown: its age tier, and the certificate a title carries. A mirror of the Rust in `server-rules`; see section 7 |
| `src/growth.ts` | 95 | How a page follows its cursor, and the one read at a time behind it |
| `src/facts.ts` | 81 | The line of facts about a title: genre, year, length, age |
| `src/focus.ts` | 178 | Where the remote is on a page of rows, and what each key does there |
| `src/title.ts` | 164 | A title page's menu lines, its three ratings, and its key map |
| `src/api.ts` | 111 | The routes this surface reads. Addresses come from the generated `ADDRESS`, not from strings here |
| `src/words.ts` | 154 | The string keys. No English in this folder |
| `src/art.ts` | 24 | Which picture goes in which place |
| `src/contract.ts` | 46 | The wire types and `ADDRESS`, re-exported from the generated file. Nothing is redeclared |
| `src/screens/Browse.tsx` | 249 | A page of rows: the screen that owns the keys |
| `src/screens/Rows.tsx` | 209 | The open row and the dim row below it |
| `src/screens/TopBar.tsx` | 145 | Who is watching, search, the tabs, the mark |
| `src/screens/TitlePage.tsx` | 229 | One title's artwork, story, ratings and menu |
| `src/screens/remote.ts` | 54 | The one seam between Roku's keys and the six words |
| `src/screens/Lost.tsx` | 101 | This app has a server and cannot reach it, said with the words it may not have |
| `src/main.tsx` | 82 | The one place that picks a screen |
| `roku-config.ts` | 41 | The generated manifest, with the app's name taken from `brand.json` |
| `src/host.ts` | 415 | **The host.** It fills `Seen`, performs what a screen asks through the ports, and holds no rules and no screens. Section 8 |
| `src/ports.ts` | 236 | The hexagon, now with the four device ports: network, store, player, pictures, plus a clock and this family's own launch |
| `src/adapters.ts` | 194 | The one file where a Roku is named. Three adapters written, two refusing in the open |
| `src/route.ts` | 222 | What an address opens. A mirror of `platform/core/src/routes.rs`, held to the fixtures and to that file read as text |
| `src/door.ts` | 114 | Where the server's address comes from, and which addresses this app will take |
| `src/lost.ts` | 53 | What a screen says when there is no server, including when it has no words to say it with |
| `src/tests/` | 3,061 | Seventeen suites, counted by `ls src/tests/*.test.ts`: the rows a page draws, the two key maps, the key names, the words, the geometry, the three mirrored rules of section 7, and the host, the router, the door, the lost screen and the adapters of section 8 |

### Nothing is transcribed that something else already states

| What | Where it comes from |
|---|---|
| Every address the app opens | `ADDRESS`, generated into `platform/contract/generated/types/contract.ts:6-29` from `address.rs`, re-exported by `src/contract.ts`. No address is a string in this folder |
| Every wire type | the same generated file. `src/contract.ts` re-exports and redeclares nothing |
| Corners and colours | `brand.json`, read by `src/numbers.ts`, as Android reads them through `BuildConfig` |
| The app's name | `brand.json`, read by `roku-config.ts` into the generated manifest |
| Every word | `strings/`, over the wire. `src/tests/words.test.ts` reads the files from disk and checks all 18 languages carry every key this surface asks for |
| Icon names | `platform/icons/icons.json`, the 50-icon set the whole project draws from. `src/icons.ts` names them and nothing draws an icon any other way |
| Layout numbers | transcribed from the Kotlin with a `file:line` on each, because they are `private const val` there and the contract generates no layout values for any language |
| Route paths | transcribed from `api.rs` with a line on each, because the generator emits them only into `openapi.json` and into no TypeScript value. **A finding for the emitter, not something to patch here** |

### The numbers come from the Kotlin

`apps/android/surfaces/tv/` is the behaviour spec, and the television surface measures in reference
pixels on a 1920 by 1080 grid
(`apps/android/surfaces/shared/src/main/kotlin/streams/shared/Measure.kt:9-12`). A Roku app with
`ui_resolutions=fhd` lays out on that same grid, so every number crosses with no scaling. The
load-bearing ones, each cited in `src/numbers.ts` at the line it was read from:

| Number | What | Where |
|---|---|---|
| 311 x 504 | a closed tile | `tv/browse/Rows.kt:61` |
| 904 x 508 | the open card | `tv/browse/Rows.kt:88`, measured from `row_screen.focused_tile` 906 x 510 and matched in `00-gaps.md` gap 9 |
| 328 | one tile to the next | `tv/browse/Rows.kt:44`; 311 and 17 added up, per `NumbersTest.kt:85`, not a measurement |
| 72 | the page margin | `tv/app/Round.kt:50`; a decision, not a measurement: the reference's own safe margin is 68 and 72 was chosen in note 188 |
| 68 / 64 / 48 | the bar's centre line, its round parts, the face | `tv/browse/TopBar.kt:37-39` |
| 810 / 320 ms | how far and how long a row rises | `tv/browse/Browse.kt:42-43` |
| 2 | rows held past the focused one | `tv/browse/Browse.kt:53`; this surface's own margin, not Netflix's |

### The key map is note 187's

`notes/187-tv-navigation-per-page-2026-11-08.md` is the per-page key map, and `src/focus.ts` and
`src/title.ts` carry its rules for the two pages written here:

| Page | Rule | Where it is |
|---|---|---|
| Browse | Back on the top bar, with focus moved off the active tab, returns focus to the active tab first | `src/focus.ts`, the `back` branch |
| Browse | Back from the top of any page but Home goes Home; on Home the app stays open (gap 7, the owner's) | `src/screens/Browse.tsx`, `read` |
| Browse | A row the viewer comes back to keeps the tile they left it on, by the row's id and the title on it | `src/focus.ts`, `columnOf` |
| Title | Left on the menu goes back, as Back does | `src/title.ts`, the `left` branch |
| Title | From the ratings, Back returns to the menu first, then leaves | `src/title.ts`, the `back` branch |
| Title | Down at the last line stays | `src/title.ts`, the `down` branch |
| every page | Left and Right swap once, for a right to left language | `src/keys.ts`, `turn` |

---

## 4. What a Roku package needs, and how it is built

Read on 2026-11-12 from Roku's own getting-started page. **None of this was run**: there is no
`node` and no `pnpm` on this machine (`node --version` and `pnpm --version` both fail), and the SDK
was deliberately not installed, because the machine has 5.5 GB free and has been stopped by a full
disk four times.

| | |
|---|---|
| Node | 24 or later |
| Package manager | pnpm 11 |
| CLI | `rk`, from `@roku-sdk/tooling` |
| Create a project | `npx --package @roku-sdk/tooling rk create my-first-app` — it "writes an app that already lists the SDK packages and installs them" |
| Packages named on Roku's pages | `@roku-sdk/tooling`, `@roku-sdk/rsg-ts`, `@roku-sdk/solid`, `@roku-sdk/components`, `@roku-sdk/navigation`, `@roku-sdk/testing-library`, `@roku-sdk/device-test-client` |
| Build and run | `rk dev` — "checks types, builds the app, installs it on the active device, and launches it". Options `--watch`, `--console`, `--debug`, `--device` |
| Manifest | `pkg/manifest`, or generated from `app.manifestEntries` in `roku-config.ts`. This folder generates it, so the app's name has one home: `brand.json` |
| Screenshot from a device | `rk ecp capture-screen --output ./shot.png` |
| An MCP server for agents | `rk mcp`, documented at [rk mcp](https://developer.roku.com/dev/docs/rsg-sdk-standing-docs-advanced-rk-mcp) |

**There is deliberately no `package.json` in this folder.** `rk create` writes one with the
versions the beta is on, and a hand-written one here would either pin versions nobody has checked or
fight the generator. The way in is `rk create`, then copy `src/` and `roku-config.ts` over what it
wrote.

### Testing without a Roku, and what actually ran

**The suites run here, today, with nothing installed.** They had never been run before 2026-11-13:
this machine has no `node`, no `pnpm` and no `tsc` (`node --version`, `pnpm --version` and
`tsc --version` all fail, re-measured 2026-11-13), so six suites and 728 lines of assertions had
been written blind and never executed. What it cost to find that out:

| | |
|---|---|
| What ran them | `/System/Library/Frameworks/JavaScriptCore.framework/Versions/A/Helpers/jsc`, the JavaScript shell macOS ships. It takes `-m`, `readFile`, `load` and `drainMicrotasks`; `print` is its only output |
| How the TypeScript reached it | type erasure by a small scanner, plus a Jest shim for `describe`, `it` and `expect`, both in a scratch folder and **neither in this repository**. They are a measurement instrument, like a ruler: nothing here depends on them |
| What it proves | that each test discriminates. It does **not** type-check anything: erasure is not checking, and the folder is still unchecked |
| Result, 2026-11-13 | 130 of 130 tests pass across nine suites. 17 of 17 mutants killed: every mutation was confirmed by the file's checksum moving before the run and returning after it |
| Result, 2026-11-14 | 279 of 279 tests pass across seventeen suites. 24 of 24 mutants killed, by the same method, each restored and checked byte for byte afterwards |

**What the second run found that a reading had not**, beyond the three defects below:

4. Four of the first twenty-three mutations **survived**, and each was a different kind of defect
   in the suite rather than in the code. One was a mutation that changed nothing (a `catch` given
   an unused binding). One was a rule the test exercised through only one of its four call sites,
   so a mutation on the other three was invisible. One was a guard that nothing could reach,
   because the line that makes it unreachable is the `stop()` in `arrive` — and the case that does
   reach it, a device whose timers cannot be cancelled, is one a Roku may well be, since nothing
   Roku publishes says it has timers at all. The fourth was `expect(waits).toContain(LOST_MS)`:
   **a number asserted against itself**, which is the one thing `src/tests/numbers.test.ts` opens
   by forbidding, written anyway in a suite that is not about numbers. It now reads
   `WAIT_AT_MOST` out of the Kotlin and asserts the derivation.
5. **One defect was found by reading and by neither the tests nor any compiler**, and it is a kind
   this language has and Rust does not. A table of words looked up as a plain object answers for
   every name `Object.prototype` has, so `/constructor`, `/toString` and `/valueOf` each found a
   *function* where `undefined` was expected, and an `=== undefined` test passed it on as the name
   of a screen. `strict` and `noUncheckedIndexedAccess` cannot see it: the type says
   `ScreenName | undefined` and is right about every value anybody wrote. It was in `src/route.ts`
   and in `src/adapters.ts`, both now read as a value (`typeof x === "string"`) rather than as a
   presence, and both are mutations 5 and 24 in the table.

**Three defects were found by running them, none of which a reading had found:**

1. `src/tests/focus.test.ts` had two cases that could never pass. Both assumed a page of rows opens
   on the top bar; it opens on the hero when it has one, which is what the Java spec does
   (`apps/android/surfaces/tv/src/main/java/streams/tv/browse/Place.java:85`). The tests were
   wrong and the code was right.
2. `src/tests/numbers.test.ts` imported `../../../brand.json`, which from `src/tests/` is
   `apps/brand.json` and does not exist. Three folders up, not four. That suite could not have been
   loaded by any runner.
3. The first draft of `src/facts.ts` wrapped the hours and minutes in a second pair of isolation
   marks, because it used this folder's `fill` where `Facts.kt:27-29` uses a plain replace. The
   fixture-held test caught it on its first run.

**The runner itself is an open boundary question for the lead, not something this folder settles.**
There is no `package.json` here and no TypeScript step in `check`, so `check` does not run any of
this. Three ways out, and the cost of each:

* **Jest**, which is what the suites are already written for and what Roku says it uses internally.
  Costs a `package.json` in this folder, `node_modules`, and a `check` step that needs node.
* **`node --test`**, which costs no framework but rewrites the assertions and still needs node.
* **Leave `check` out of it** and let the suites run only under `rk dev`. Costs: they go
  unrun on every push, which is how they came to hold three defects in the first place.

CI already installs node 22 and TypeScript 5 for `check_types.py` (`.github/workflows/check.yml`),
so the runner is nearer than this folder's history suggests. Picking one is the lead's.

The rest of this section is what Roku publishes, and is unchanged:

* **Jest runs on a computer**, with a mock render tree, and is what Roku uses internally: "Jest is
  what Roku uses internally to test the SDK", and "Jest does not run on the device"
  ([Live Q&A recap](https://forum.developer.roku.com/t/rsg-2-0-live-q-a-recap/11817), 2026-11-12).
  `@roku-sdk/testing-library` is named on Roku's own overview page.
* **The Roku Cloud Emulator** gives virtual players and TVs in a browser or over a REST API, with
  Dev Mode and Sideload buttons in the device viewer
  ([Roku Cloud Emulator User's Guide](https://developer.roku.com/dev/docs/rce), page updated
  2026-11-10). An individual developer account gets a **10-hour free trial** covering one device and
  one stream; after that it needs a paid AWS Marketplace subscription, **available only to users in
  the USA**. Access is rolling out in phases. Roku says virtual devices are "not intended for
  real-world platform, speed, and performance testing", and the unsupported list includes physical
  remotes, Wi-Fi behaviour, suspend and resume, and voice. DRM is Widevine L3 at 1080p or 720p only.
  Roku also said RCE support for the TypeScript SDK would arrive in the beta: "Yes, we will have RCE
  support in beta and you will be able to do exactly this" (Dev Summit recap, Q29).
* **Note the conflict**: the trial is 10 hours on Roku's own user guide and 5 hours in the blog post
  summary. The user guide is the page that was opened, so 10 is what is recorded here. `open`.

So "Roku needs a device" was too strong. A logic suite needs no device at all, and a running app
needs either a device, or a cloud emulator account, in the USA, with a card.

---

## 5. What is not written, and what unblocks each

In the order they block a running app.

1. ~~**`src/host.ts`: the host.**~~ **Written on 2026-11-14**, and section 8 is what it decided.
   It did not wait for the SDK, because it did not have to: the host talks to ports, and only the
   adapters behind them are a Roku. Two of the five adapters are still refusing in the open
   (`src/adapters.ts`), and those two are items 1a and 1b below rather than a host that is missing.
   1. **The registry adapter.** `StandingStore` keeps the server's address for the run of the app
      and not between runs, so a device that loses power is handed its address again at the next
      launch. Roku's documents describe the registry as BrightScript's `roRegistry` and its Q&A
      says a flush in either language flushes both; **the TypeScript spelling was published
      nowhere that was read**. *Unblocks with*: the SDK installed, or
      [the public repository](https://gitlab.com/rokudev/rsg-sdk) read. It is about ten lines.
   2. **The launch-parameter adapter.** `NoLaunch` hands back none, so a fresh device falls
      through to the kept address and finds nothing. How a sideloaded RSG 2.0 app reads the
      parameters it was launched with was not established. *Unblocks with*: the same. This is the
      **first** thing to write, because section 8's whole answer to "where does the address come
      from" rests on it.
2. **The app's entry point.** `main` from `@roku-sdk/rsg-ts` is named on Roku's getting-started page
   but its signature was not published on any page read. It is now the only thing between this
   folder and a running app: the foot of `src/main.tsx` is the five lines it would be, and every
   one of the names in them is written and type-checkable. *Unblocks with*: the SDK installed, or
   the [public repository](https://gitlab.com/rokudev/rsg-sdk) read.
3. **`RemoteKey`.** `useFocusable`'s `onKey` is "a declarative map from remote key names to
   handlers", and Roku's page writes one key as `RemoteKey.Select` without listing the enum. This
   folder keys the map with the key strings Roku's own [onKeyEvent](https://developer.roku.com/dev/docs/onkeyevent)
   page documents, which is the only key naming Roku publishes. *Unblocks with*: the SDK installed.
   If the spellings differ, `ROKU_KEYS` in `src/keys.ts` is the one line that changes.
4. **The brand's rounded corners.** Rectangle has no corner radius and Poster's page says rounded
   corners come from an `effect` node, with `shaderCompatible` set before `uri`. So the card's 21 px
   corner and the tab pills are square here. *Unblocks with*: the Effect node's field list, then one
   part that wraps it.
5. **The icons do not exist as files.** `platform/tools/make_icons.py` writes the 50 icons of
   `platform/icons/icons.json` out for the web televisions, Android and Apple, and **has no Roku
   emitter**. `src/icons.ts` names them at `pkg:/images/icons/<name>.png`; nothing produces those
   files. Roku draws a Poster from a bitmap, so the emitter has to rasterise rather than copy SVG.
   *Unblocks with*: a `roku()` function in that generator, which is not this folder's file.
6. **The mark.** Every other surface draws `{server}/mark.svg`, and the brand ships SVG. Whether a
   Roku Poster reads SVG was not established. This folder asks for PNG. *Unblocks with*: a decision
   on whether the server grows a PNG or the package carries one.
7. **Text that fits.** Neither `maxLines` nor `wrap` is in the Label page's field list, so the
   story and the card's name are drawn without a line limit. *Unblocks with*: the LabelBase field
   list.
8. **The screens that are not here**: the hero card, search, profiles, sign-in, help, settings,
   episodes, more-like-this, and the player. The player additionally needs the Video node's fields
   and an answer on HLS and DRM. *Unblocks with*: the work, in that order; none of it needs a
   device either, except the player. **The address screen is not on this list and is not an
   oversight**: section 8 says why this family has none, and what it would cost to add it the day
   the store opens.
9. **A string key that does not exist.** An address this app refuses — plain http to anything but
   the viewer's own network — is shown as the address and nothing else, because `strings/` has no
   key for the sentence. Android and Apple both hold that sentence as an **English literal in the
   host** (`apps/android/core/src/main/kotlin/streams/core/Host.kt:94-95`,
   `apps/apple/Sources/CoreHost/Host.swift:44`), which is the shape of the problem rather than a
   solution to it. *Unblocks with*: a key in `strings/`, which is not this folder's file — and
   which has the same chicken-and-egg as every other word on that screen, since the words come
   from the server that has not been named yet.
10. **Nothing has been type-checked.** The suites run (section 4), but erasing types is not
   checking them: no `tsc` has read a line of this folder, so every `strict` flag in
   `tsconfig.json` is still a promise. This matters more today than it did yesterday, because
   `src/host.ts` and `src/route.ts` are the first files here that would fail a type check in ways
   a reader cannot see. *Unblocks with*: Node 24 and a TypeScript compiler, about 300 MB of
   `node_modules`, and 4 GB of disk headroom. CI already installs both for `check_types.py`, and
   `platform/tools/check_roku.py` runs `tsc --noEmit` the moment one is on the PATH.
11. **`POST /v1/core/step` does not answer.** `server/crates/server-api/tests/routes.rs:13`.
   *Unblocks with*: the server work, which is another agent's. Until it does, `src/route.ts` and
   section 7's four rules are a second home for rules the Rust already holds, and section 8 says
   exactly which lines would be deleted the day that route answers.
12. **What a screen reader says as the remote moves.** Android gives the one focusable a live
    region, re-spoken each time it changes, because a surface that paints its own focus announces
    nothing otherwise (`Remote.kt:57-69`). The only accessibility field found on Roku's pages is
    `audioGuideText` on the Poster node; nothing equivalent to a live region was established, so
    `screenRemote` hands a screen its own sentence back and draws it nowhere.
    *Unblocks with*: Roku's accessibility documentation, which was not read.

## 6. Words

301 English keys in 18 languages in `strings/`, delivered by the server at `/v1/strings/{lang}`
(`platform/contract/src/api.rs:183`). `src/words.ts` lists the keys this surface asks for and holds
no English. The brand's name comes from `brand.json` and reaches the manifest through
`roku-config.ts`.

---

## 7. The mirror, and what keeps it from drifting

Section 2 ends in a recommendation the owner has not taken: step the core on the server. On
2026-11-12 he decided the other way for now — **mirror the rules in this language, and accept two
homes for them**. This section is what that decision looks like in files, and what was built to
keep the second home honest.

Two implementations of one rule drift. That is not a risk, it is the default, and a second
implementation checked only against its own expectations is worse than none, because it keeps
passing while the rule it mirrors moves underneath it. So every mirrored rule here is held to
something outside this folder.

### What is mirrored

| Rule | Mirrored from | Held against |
|---|---|---|
| A profile's age tier and the certificate a title may carry (`src/tier.ts`) | `server/crates/server-rules/src/tier.rs:7-120` and the certificate clause of `eligibility.rs:47-88` | `fixtures/2026-11-05/valid/ProfileLimits.basic.json`, `Profile.kids.json`, `Profile.main.json`, **and `tier.rs` itself, read from disk** |
| How a page follows its cursor and how far ahead it reads (`src/growth.ts`, `PageFocus.wantsMore`) | `tv/browse/Browse.kt:32-46` and `:139-168`, `browse/Place.java:342-360` | `fixtures/2026-11-05/valid/More.section.json` and `Screen.home.json` |
| The line of facts about a title (`src/facts.ts`, `counted` and `number` in `src/words.ts`) | `tv/parts/Facts.kt:21-48`, `shared/Theme.kt:88-134` | `fixtures/2026-11-05/valid/TitleDetail.{film,series,coming}.json` and the real `strings/*.json` |
| Which rows a page may draw (`src/page.ts`, already here) | `tv/browse/Page.kt` | `fixtures/2026-11-05/valid/Screen.home.json` and `tolerated/Section.unknown_kind.json` |

The fixtures are not this folder's. They are
`platform/contract/fixtures/2026-11-05/` — count them with
`find platform/contract/fixtures/2026-11-05 -type f | wc -l`, which answered 274 on 2026-11-14 and
271 when this line was written on 2026-11-12 — files that
`platform/contract/tests/fixtures.rs` reads with `fs::read_dir` and round-trips through the Rust
contract types, and whose first line reads: "Fixtures are plain JSON and every language reads the
same files." A TypeScript test reads them with `readFileSync` and gets the same bytes the Rust
does. **That is the whole answer to "what holds the mirror to the core": the corpus already
existed, in a form every language can read, and nothing had to be moved or reshaped to use it.**

### The drift guard

Fixtures hold the behaviour. They do not notice the day somebody adds a tier to the Rust enum or
changes a certificate rank, because a fixture carries data and not rules. So
`src/tests/tier.test.ts` also reads `server/crates/server-rules/src/tier.rs` **as text** and
compares its match arms with the tables here: the order of the tiers, every rating word, every
coarse level, every certificate rank, and the ceiling a Kids profile is clamped to. Change the Rust
and this folder goes red, naming that file.

It is a text read and it is brittle to a rewrite of that file, which is the price. The alternative
was no guard at all.

### What is deliberately not mirrored

* **`Tier::of_account`** (`tier.rs:78-90`), the widest tier across an account's profiles. It needs
  each profile's own age limit, and the wire gives a client a `Profile` with no limit on it at all.
  A mirror of it here would be a rule with no caller and no way to be wrong.
* **The rest of `eligible`** (`eligibility.rs:47-88`): the adult and suggestive flags, the blocked
  genres, the blocked keywords, and the scored profile model used when a title has no certificate.
  None of the five reaches a client over the wire. `certificateFits` answers `null` for that case
  rather than guessing, and the screen draws what the server sent.
* **Everything the core decides.** Sessions, refresh, the queue, play, the layer: `src/host.ts` is
  still unwritten and section 2's question is still open. This mirror is the drawing half of four
  rules, not a second core.

### The numbers

No number was copied out of the Rust into a literal here. The five certificate ranks are positions
in one ordered list of tiers, which is how `tier.rs:7` declares them ("A tier, narrowest first"),
so a rank and a tier cannot disagree. `REACH` stays where every other number this surface uses
stays, in `src/numbers.ts`, with the Kotlin line it came from.

---

## 8. The host

Written 2026-11-14. `src/host.ts` is the object that holds the state every screen draws from,
performs what a screen asks through the ports, and decides nothing about what anything looks like.
Its twins are `apps/android/core/src/main/kotlin/streams/core/Host.kt` and
`apps/apple/Sources/CoreHost/Host.swift`, and the difference between it and both of them is the
whole of section 2: those two step the shared Rust core and this one has no core to step.

Each decision below names its precedent, because none of them is new work where an answer already
existed somewhere in this repository.

### 8.1 Where the server's address comes from

**Decision: the parameters the channel was launched with, then the registry. There is no address
screen.**

| family | where the address comes from |
|---|---|
| web | nowhere: a browser is already at the server it loaded the page from |
| android | a kept value, else a screen of its own with a text field (`door/ServerAddress.kt`) |
| apple | a kept value, else the `SERVER_URL` environment variable, else that same screen (`CoreHost/Host.swift:127-131`) |
| **roku** | **the launch parameters, else a kept value** |

Apple's environment variable is the nearest precedent and is worth naming, because it is the same
idea: a device that is set up by somebody at a computer can be *handed* its address instead of
being asked for it. On a Roku that is not a convenience, it is the only shape that fits. An RSG 2.0
app cannot reach the Streaming Store before Fall 2027 (section 1), so the only way onto a device is
`rk dev`, run by a person who is already typing a command — and a television remote has no
keyboard.

What it buys, beyond not writing an on-screen keyboard against a text-input node nobody has read:
the address screen is the **one** screen in this folder that could not be drawn in the viewer's
language, because every word comes from `/v1/strings/{lang}` on the server that has not been named
yet. That is not a hypothetical. Android's own address screen holds four English literals for
exactly that reason (`door/ServerAddress.kt:57,70,74`) and Apple's holds the same, in a project
whose rule is that no surface contains an English literal. This folder holds none and now needs
none.

What it costs, stated rather than hidden: a viewer who is handed a device cannot move it to another
server without a computer. The day the store opens this family needs the screen the other two have,
and it needs string keys that can be drawn before any string has arrived.

The launch parameter **outranks** the kept address, deliberately: a kept value that won would make
a television impossible to re-point without clearing its registry, which is not something a person
with a remote can do. `src/tests/door.test.ts` holds that order.

**With no address anywhere**, `seen.server` is empty and `main.tsx` draws the background and
nothing else — which is what it did before the host existed, and now it is a decision with a
reason rather than a gap. The one exception is an address this app **refused**: that is shown, as
the address and nothing else, for the reason section 8.4 gives.

### 8.2 The ports

`src/ports.ts` is the family's seam and `src/adapters.ts` is the one file where a Roku is named.
The four every family has, plus two this one needs:

| port | shape | adapter | state |
|---|---|---|---|
| `Network` | one `send(call, done)`, `done` runs once whatever happened | `FetchNetwork` | **written** |
| `Store` | `get` / `put` / `delete` | `StandingStore` | **standing**: this run only. Section 5, item 1.i |
| `Player` | `command` / `listen` | `NoPlayer` | **refusing**, and collected so it can be read back |
| `Pictures` | one call: `of(url)` | `ServerPictures` | **written** |
| `Clock` | `after(ms, run)` | `TimerClock` | **written**, and the one guess in the folder |
| `Launch` | `parameters()` | `NoLaunch` | **refusing**. Section 5, item 1.ii |

Three of these are worth more than a row:

* **`Network` is written, and that was not expected.** Roku's own RSG 2.0 Q&A says network calls
  are ordinary `async`/`await` with `fetch`, that the beta supports basic fetch, and that streaming
  is not there yet. **Status: assumed, not verified.** That came from a search restricted to
  `developer.roku.com`, `forum.developer.roku.com` and `gitlab.com` on 2026-11-14, which named the
  RSG 2.0 Live Q&A recap and the Dev Summit recap; **neither page was opened**, unlike every line
  of section 1. The same search is where the registry's 32 KB below comes from, and it found no
  TypeScript registry API at all. `fetch` is a standard, not an SDK
  call, so `FetchNetwork` imports no `@roku-sdk` package and is type-checked and run with the rest.
  The narrow shape it uses is declared in `types/ambient.d.ts` and nothing wider: no `Request`, no
  `Headers`, no streaming body, and **no `AbortController`**, which that recap names as something
  Roku *wants* natively — which is not the same as having it. So a deadline is a race against the
  clock, the abandoned read is still running underneath, and `done` runs exactly once. That last
  part is a case in `src/tests/adapters.test.ts` because a reader cannot see it.
* **`Pictures` is one call here and six on Android** (`ports/Ports.kt:128-156`), and that is a real
  difference between the two devices rather than something left out. A Roku draws a picture by
  handing a Poster node a `uri`; there is no fetch that gives the pixels back, so there is nothing
  for a `warm`, a `Want` or a `Stop` to mean.
* **`Store` is 32 KB and that decides what it is for.** A Roku's registry is capped at 32 KB for
  the whole app (Roku's Dev Summit Q&A; same search, same status: **assumed, not verified**).
  Android's `Store` holds the core's
  offline layer — waiting commands and kept pages — and one kept page of twelve rows does not fit
  in 32 KB beside anything else. **So this family has no offline layer, and a command sent while
  the server is unreachable is gone.** That is written at `Host.act` and in the `Store` port rather
  than discovered later.

### 8.3 What fills `Seen`, and what a screen draws before anything has

**Decision: every read carries the number of the state it was started for, and an answer whose
number has moved on is dropped. A move clears the last page's data at once.**

The precedent is four days old and is `cardsOf(arrived, season)` in
`apps/android/surfaces/tv/src/main/kotlin/streams/tv/title/Episodes.kt:153-168`. The defect it was
written for reached every viewer: during a season change the page drew the *previous* season's
names, stills and descriptions under a column already marking the new season as chosen, because
the pair in hand was replaced only when the new answer landed.

Reading that before writing this changed the design in one way worth naming. The late-answer guard
is the half that is easy to see; **the half a viewer actually saw is the clearing**. So `arrive`
does both, in this order: count the state up, clear `screen` and `detail`, say `reading`, then ask.
Both halves have their own case in `src/tests/host.test.ts`, and both are in the mutation table.

The rule is at the host and not in one screen, because at this level the same defect is a page of
rows drawing the last page's rows while the new one is read. `src/screens/Browse.tsx` already did
its own half of it for a title's details (`if (got.title === tile.title)`); it no longer has to.

Two scopes, not one: the menu and the words belong to a **server** and not to a page, so they
survive every move between pages and are thrown away when the server changes. An app that kept the
old server's tabs after being re-pointed would draw tabs that lead nowhere.

### 8.4 When the server cannot be reached

**Decision: say so at once when a read fails, and after the television's own wait when a read
simply never answers. Draw the address whether or not any word has arrived.**

The television could not say it was offline until 2026-11-13: on a Google TV Streamer with the
server unreachable the app drew a flat rectangle with 9 nodes, 0 focusable and 0 words, held across
8 probes and a cold start (`tv/app/Lost.kt:39-42`). `error.no_server` and `say.try_again` now exist
in all eighteen languages, and `src/tests/words.test.ts` holds that this folder's keys do too.

**This family has the same hole**, and the README would be worth less if it pretended otherwise:
every word this app draws arrives from the server, so an app that cannot reach one has no
vocabulary to say so with. `src/lost.ts` carries the only answer there is, which is `Lost.kt`'s —
the address is the one thing on the screen that did not have to be fetched to be known, it is last,
and it is never missing, which is also what keeps the spoken sentence from being empty and the one
focusable node from being unnamed. Most of `src/tests/lost.test.ts` is that empty case.

**And one more hole of its own.** The television at least has an address to draw. A Roku that was
never handed one has no server *and* no words, and there is nothing on the screen at all. That is
the cost of 8.1 and it is listed there.

Two things here are better than the television's, and both came from writing the host rather than
porting the screen:

* **A failed read says so immediately.** The television only has a timer, so a refused connection
  and a wrong address both cost sixteen seconds of blank screen. Here a read that fails — no
  answer, an error status, or a body this app cannot parse — sets `lost` on the spot. The timer is
  only for the server that accepts a connection and then says nothing.
* **A wait belongs to the page that started it.** A page left while it was still reading leaves its
  wait behind; without a guard that wait comes due, sees that something is being read, and says the
  app is lost about a page that is seconds old. Finding that case took a surviving mutation, and
  the test for it runs on a clock that ignores `stop` — which is not a contrived device, since
  nothing Roku publishes says a TypeScript app can cancel a timer, or has one.

### 8.5 What the host does not do

Named here so nothing reads it as a core: sessions and sign-in, the offline queue, playing,
`GET /v1/hello` (so `{brand}` draws empty and no moved-address table is mirrored), and stepping
anything. `ROUTE.step` is in `src/api.ts` and called from nowhere.

`src/route.ts` is the largest mirror in the folder and the one most worth deleting. It mirrors two
of the four steps of `platform/core/src/routes.rs` — `place` and `member_target` — and names the
other two as not mirrored with what each would need. The day `POST /v1/core/step` answers, that
file and the four rules of section 7 are what comes out.
