# harmony

ArkUI app for Huawei phones and tablets. **Touch** surface, **native** core.

Part of an open video platform.

Status: **an app whose ArkTS has never been compiled.** The route switch, Home with its billboard
and rows, the title page, the profile gate, search and the notifications feed are written and laid
out against the measured reference, and not one line of them has been compiled: no HarmonyOS SDK, no
DevEco Studio and no Huawei device were on the machine they were written on, and `node`, `npx` and
`tsc` are not there either. [What cannot be done
here](#what-cannot-be-done-without-the-sdk-and-a-device) says exactly what that leaves.

Two parts of this folder are not in that state, and `./apps/harmony/check.sh` runs both.

The C glue -- `core/cpp/napi_bridge.c`, the whole of the boundary between ArkTS and the core -- is
built and run on whatever machine types that command, against a recording core and against the real
one.

And since 2026-11-13 the ArkTS, which cannot be compiled, is READ. `platform/tools/touch_check.py` holds every
number in `surfaces/touch` against the Android phone Kotlin that measured it, every colour and
corner against `brand.json`, every word key against `strings/en.json`, every icon against the
`R.drawable.icon_*` the Android phone draws, and every import against the file it names. It is the
same shape the television's Kotlin is held in by `apps/android/.../streams/tv/NumbersTest.kt` and it
proves nothing about whether any of this compiles -- only that nothing in it was invented here.
`surfaces/touch/README.md` has the table of what it reads against what.

Four of the six questions in `OPEN-QUESTIONS.md` are now answered, from Huawei's own reference rather
than from a device, and the code changed where the answers changed it. The two that are left want a
device and a sign-in, and both are named precisely. The root README's status table carries a line for
every platform, and that table is not owned here.

## What this folder is

Two decisions by the owner on 2026-11-11 fix its shape, and neither is a guess:

* **Surfaces split by input grammar** — pointer, touch, remote — not by platform or screen size. A
  Huawei phone and tablet are touch, so there is one surface here and it is `surfaces/touch`. A Huawei
  PC with a mouse is a **pointer** surface, which no family in this repository has built; it is listed
  under [what is still open](#what-is-still-open) rather than improvised as a stretched phone.
* **WebAssembly only where the one open runtime of a platform is a web engine** — Tizen, webOS, Vizio
  and the web app. HarmonyOS takes native code, so the core here is the same static library the Android
  and Apple families link, reached through Node-API instead of JNI. There is no `core.wasm` in this
  folder and there should never be one.

The README this file replaces said "ArkUI app for Huawei phones, tablets and PCs". The phones and
tablets part stands. The PCs part was a platform list, not a surface, and it is now one line of open
work instead of an implied third of an app.

## The core boundary

A host has to provide exactly what `apps/android/core` provides, which is the measure used here. Read
out of that folder and `platform/bridges/c` on 2026-11-11:

| What the host must do | Android | here |
|---|---|---|
| Link the core's static library into one shared library the runtime loads | CMake links `libbridge_c.a` into `libstreamscore.so` | `core/cpp/CMakeLists.txt`, the same, with OpenHarmony's CPU names |
| Carry four calls across the language boundary: `abi`, `open`, `step`, `close` | `bridge.c`, through JNI | `core/cpp/napi_bridge.c`, through Node-API |
| Copy the out bytes once and hand them straight back to `core_free` | `bridge.c` | the same |
| Turn a status that is not `CORE_OK` into a thrown error | `bridge.c` | the same |
| Refuse to open a core whose ABI is not the contract's | `Host.kt`, `check(NativeCore.abi() == ABI_VERSION)` | `core/Host.ets`, the same |
| Use one handle from one thread at a time | a `HandlerThread` of its own | one thread, which one is `OPEN-QUESTIONS.md` Q3 |
| Build the core for each device CPU | a gradle task per ABI | `build-core.sh` |

One difference, and it is the whole of what Node-API changes: JNI carries `ByteArray` because a Kotlin
string is UTF-16 and converting twice would be waste. Node-API hands out UTF-8, which is what the core
reads, so these four calls carry strings and nothing is converted at all. The glue needs no allocator
of its own either: the one buffer it wants comes from `core_alloc` and goes back to `core_free`, which
`platform/bridges/c/core.h` already promises.

### The core builds for HarmonyOS, with no SDK

Ran here on 2026-11-11, rustc 1.98.1:

    ./build-core.sh

which is `cargo build --locked --profile native -p bridge-c --target <t>` for each of
`aarch64-unknown-linux-ohos`, `armv7-unknown-linux-ohos` and `x86_64-unknown-linux-ohos`, all three
listed by `rustup target list`. All three produced a `libbridge_c.a`. Only `aarch64` traces to a note
(151, line 275, the CI target list); the other two are this folder's own guess at older hardware and
the emulator, and `build-core.sh` says so. No SDK is involved, because a
static library is assembled by rustc and never linked; the SDK's clang is wanted at the next step,
where CMake links `libstreamscore.so` against the archive, exactly as the Android NDK's clang is.

## Folder shape

| Folder | State |
|---|---|
| `core/cpp/` | written. `napi_bridge.c`, its `CMakeLists.txt` and the type declaration ArkTS reads. Built and run by `check.sh` against a stub runtime; never built against a real SDK |
| `core/cpp/test/` | the stub Node-API runtime, the recording core and the two tests that drive the bridge. None of it ships: it exists so the glue is run rather than read |
| `core/` | written, never compiled. `NativeCore.ets` declares the four calls; `Host.ets` steps the core and performs its effects |
| `ports/` | written, never compiled. The six ports every native family names: network, secrets, store, player, reel, pictures |
| `adapters/` | not written. `adapters/README.md` names the expected API per port and how sure each one is |
| `surfaces/touch/` | **written, never compiled.** `surfaces/touch/README.md` lists the files and holds the measured shape they meet |
| `contract/` | `effects.ts` and `read.ts`, written: the TypeScript seam that holds everything ArkTS refuses about the generated contract. `contract.ts` is copied by `build-core.sh` |

`core/Host.ets` is a line by line port of `apps/android/core/src/main/kotlin/streams/core/Host.kt`,
which is the written and shipped version of this host. Nothing in it is a second opinion about what a
host does, and when that file changes this one is diffed against it. It carries the Android host's
plain-http gate word for word: plain http reaches only an address on the viewer's own network, because
a session's bearer and refresh tokens travel in the clear, and a refused address is said to the viewer
rather than swallowed.

## What cannot be done without the SDK and a device

So that nobody later mistakes this for a finished app:

1. **No ArkTS has been compiled.** Not one line. `node`, `npx`, `tsc`, `cmake`, `ohpm`, `hvigorw`
   and `codelinter` are all absent from the machine this was written on, so every `.ets` and `.ts`
   file here has been read and never checked by a tool, and `core/cpp/CMakeLists.txt` has not been
   parsed. **This is now a download and not a device**: the Command Line Tools package carries
   `hvigorw` and `codelinter` and neither wants a phone (`OPEN-QUESTIONS.md`, the section after Q6).
   The C glue is the exception and is compiled and run; see item 6.
2. **No adapter exists.** All six ports are declared and none is implemented, so the host cannot make
   a request, keep a token, keep a page, show a picture or play one frame. The screens draw, and what
   they draw is whatever the ports answer — which today is nothing.
3. **No project file.** `build-profile.json5`, `oh-package.json5`, `module.json5`, `AppScope/app.json5`
   and `hvigorfile.ts` are versioned with the SDK. Where they go and what they must contain is now
   known (Q6); the files themselves still want the SDK that stamps today's version numbers into them.
4. **Four routes draw nothing.** The server address screen, `start`, `sign_in` and `watch` have no
   screen in this folder, and `surfaces/touch/Root.ets` names each one with the Kotlin that
   specifies it rather than sending the viewer somewhere that looks like it worked. It was five
   until 2026-11-13, when the notifications feed was written.
5. **The icons do not exist.** `surfaces/touch/Icons.ets` names all 23 as `$r('app.media.*')`
   references; what they resolve against is a module `resources/base/media` folder, which is part of
   Q6.
6. **Nothing RUNS the ArkTS, and nothing on this machine can.** `./apps/harmony/check.sh` holds this
   family three ways and none of them is a compiler.

   Two build and run `core/cpp/napi_bridge.c`: against a core that records every call, which drives
   each refusal and holds the ledger that every block taken from `core_alloc` goes back to
   `core_free` with its own size; and against the real `libbridge_c.a`, which opens, steps and
   closes a core with a phone's config. The runtime underneath both is `core/cpp/test/napi_stub.c`,
   a fake. What neither proves is that OpenHarmony's own `napi/native_api.h` declares these calls
   the way the upstream Node-API headers do, which is `OPEN-QUESTIONS.md` Q1 and is stated in
   `core/cpp/test/napi/native_api.h`.

   The third, `platform/tools/touch_check.py`, READS the ArkTS. It cannot tell whether a file compiles, whether a
   type is right, whether an `@Builder` may be called where it is called, whether `Repeat` takes the
   arguments given it, or whether a vp is a dp. What it does tell is whether anything in the folder
   was invented here: every number against the Kotlin that measured it, every colour and corner
   against `brand.json`, every word against `strings/en.json`, every icon against the Android
   phone's, every import against the file it names, and every length against being a named constant
   at all. `surfaces/touch/README.md` has the table.
7. **Six ArkUI calls are marked `ASSUMED` where they are used**, and no more than six: the alpha order
   of a hex colour string, `bindMenu`'s one-argument form, the six canvas text methods the Top 10
   numeral is drawn with, `effectKit`'s colour picker, whether `direction` is inherited down a
   subtree, and whether a vp equals a dp. Each one says, where it is written, what it costs if it is
   wrong.

## The contract seam

The one thing in this folder that no other family has, and it is the answer to Q2.

ArkTS is a narrower language than TypeScript, and the generated contract uses three constructs that
are hard compile errors in it — 40 `(string & {})` intersections, 80 anonymous union members and
`Extract<...>`. The rules are scoped by file extension (`arkts-no-ts-deps`, 10605147: "An .ets file
can import source code of an .ets/.ts/.js file, but a .ts/.js file cannot import source code of an
.ets file"), so the contract stays TypeScript and everything ArkTS refuses stays on that side:

* `contract/effects.ts` narrows the unions and builds every `Event`, `Action` and `Command`, so no
  `.ets` file in this family writes a contract literal.
* `contract/read.ts` reads a body into its type, because ArkTS bans `any` and `JSON.parse` is `any`.

No change to `platform/contract/gen` is needed, which matters because `apps/web` and `apps/tv` read
the same generated file.

## What is still open

1. **The pointer surface.** Huawei PCs run HarmonyOS, a mouse is pointer input, and no family has a
   pointer surface. Whether one is written here, shared across families, or dropped is the owner's.
2. **The ring check has no `harmony` family.** `platform/tools/check_rings.py` enforces per family that
   a surface never reaches a port directly, and it names four families. Nothing holds this folder to its
   ports yet. Adding it belongs to `platform/tools`, not here, and it needs the word list that names
   HarmonyOS's own APIs.
3. **The six questions in `OPEN-QUESTIONS.md`**, each with what would answer it.
4. **Whether HarmonyOS is in scope at all.** Project note 151 records HarmonyOS NEXT as China only
   today, AppGallery as the only route in, no Netflix app found to measure against, and China filing and
   licence duties unread. Note 135 records the same. None of that is settled here.

Licensed under either of MIT or Apache 2.0, at your option.
