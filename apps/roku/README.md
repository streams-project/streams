# roku

The Roku client. **Roku forces no language on us.** It publishes two first-party routes, and this
folder takes the newer one for new code:

| Route | Language | Can publish to the Streaming Store today |
|---|---|---|
| SceneGraph + BrightScript, since 2014 | BrightScript and XML | **yes** |
| RSG 2.0 SDK, public beta since 2026-10-29 | TypeScript and TSX, running on Hermes | **no.** Sideload only. General availability targeted Fall 2027 |

Status, 2026-11-12: **18 source files and 2,088 lines, plus 6 Jest suites and 728 lines of tests
that need no device. The rules, the key maps, the geometry and the contract calls are written. The
host is not written, and no line of this folder has been compiled or run, because the toolchain is
not installed on this machine.** Measured with
`cat src/*.ts src/screens/* src/main.tsx roku-config.ts | wc -l`. What is left is listed in
section 5, each item with what unblocks it.

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

Nothing here is a Roku API call except in the four `.tsx` files and `src/screens/remote.ts`. Every
other file is plain TypeScript that could be read and checked on any machine.

| File | Lines | What it does |
|---|---|---|
| `src/numbers.ts` | 203 | Every number the surface lays out with, each beside the Kotlin line it was taken from. Corners and colours are read from `brand.json`, not transcribed |
| `src/keys.ts` | 78 | Roku's 15 documented key strings, turned into the six words a screen answers |
| `src/page.ts` | 80 | Which rows a page may draw, and whether the answer in hand is even its own |
| `src/focus.ts` | 178 | Where the remote is on a page of rows, and what each key does there |
| `src/title.ts` | 164 | A title page's menu lines, its three ratings, and its key map |
| `src/api.ts` | 111 | The routes this surface reads. Addresses come from the generated `ADDRESS`, not from strings here |
| `src/words.ts` | 57 | The string keys. No English in this folder |
| `src/art.ts` | 24 | Which picture goes in which place |
| `src/ports.ts` | 109 | The hexagon: what a screen may ask a Roku for |
| `src/contract.ts` | 42 | The wire types and `ADDRESS`, re-exported from the generated file. Nothing is redeclared |
| `src/screens/Browse.tsx` | 249 | A page of rows: the screen that owns the keys |
| `src/screens/Rows.tsx` | 209 | The open row and the dim row below it |
| `src/screens/TopBar.tsx` | 145 | Who is watching, search, the tabs, the mark |
| `src/screens/TitlePage.tsx` | 229 | One title's artwork, story, ratings and menu |
| `src/screens/remote.ts` | 54 | The one seam between Roku's keys and the six words |
| `src/main.tsx` | 56 | The one place that picks a screen |
| `roku-config.ts` | 41 | The generated manifest, with the app's name taken from `brand.json` |
| `src/tests/` | 728 | Six Jest suites: the rows a page draws, the two key maps, the key names, the words, the geometry |

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

### Testing without a Roku

This is the part that turns out to be better than expected:

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

1. **`src/host.ts`: the host.** Nothing fills a `Seen`, so no screen has data. It needs the SDK's
   http client, its persistent storage, and the decision in section 2 about where the rules run.
   *Unblocks with*: the owner's answer on A/B/C, and the SDK installed so its http and storage APIs
   can be read rather than guessed.
2. **The app's entry point.** `main` from `@roku-sdk/rsg-ts` is named on Roku's getting-started page
   but its signature was not published on any page read. *Unblocks with*: the SDK installed, or the
   [public repository](https://gitlab.com/rokudev/rsg-sdk) read.
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
8. **The screens that are not here**: the hero card, search, profiles, the door and sign-in, help,
   settings, episodes, more-like-this, and the player. The player additionally needs the Video
   node's fields and an answer on HLS and DRM. *Unblocks with*: the work, in that order; none of it
   needs a device either, except the player.
9. **Nothing has been compiled or run.** No `node` on this machine. *Unblocks with*: Node 24 and
   pnpm 11, about 300 MB of `node_modules`, and 4 GB of disk headroom.
10. **`POST /v1/core/step` does not answer.** `server/crates/server-api/tests/routes.rs:11`.
   *Unblocks with*: the server work, which is another agent's.

## 6. Words

301 English keys in 18 languages in `strings/`, delivered by the server at `/v1/strings/{lang}`
(`platform/contract/src/api.rs:183`). `src/words.ts` lists the keys this surface asks for and holds
no English. The brand's name comes from `brand.json` and reaches the manifest through
`roku-config.ts`.
11. **What a screen reader says as the remote moves.** Android gives the one focusable a live
    region, re-spoken each time it changes, because a surface that paints its own focus announces
    nothing otherwise (`Remote.kt:57-69`). The only accessibility field found on Roku's pages is
    `audioGuideText` on the Poster node; nothing equivalent to a live region was established, so
    `screenRemote` hands a screen its own sentence back and draws it nowhere.
    *Unblocks with*: Roku's accessibility documentation, which was not read.
