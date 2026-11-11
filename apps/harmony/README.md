# harmony

ArkUI app for Huawei phones and tablets. **Touch** surface, **native** core.

Part of an open video platform.

Status: **a skeleton, not an app.** Nothing here draws a screen, and nothing here has been compiled
except the C glue's syntax and the core it links, which were both run on 2026-11-11 on a machine with
no HarmonyOS SDK, no DevEco Studio and no Huawei device. [What cannot be done
here](#what-cannot-be-done-without-the-sdk-and-a-device) says exactly what that leaves undone, and
`OPEN-QUESTIONS.md` holds the six questions a device and an SDK would answer. The root README's status
table carries a line for every platform; this folder is no longer only a README, and that table is not
owned here.

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
listed by `rustup target list`. All three produced a `libbridge_c.a`. No SDK is involved, because a
static library is assembled by rustc and never linked; the SDK's clang is wanted at the next step,
where CMake links `libstreamscore.so` against the archive, exactly as the Android NDK's clang is.

## Folder shape

| Folder | State |
|---|---|
| `core/cpp/` | written. `napi_bridge.c`, its `CMakeLists.txt` and the type declaration ArkTS reads. Syntax checked, never built against a real SDK |
| `core/` | written, never compiled. `NativeCore.ets` declares the four calls; `Host.ets` steps the core and performs its effects |
| `ports/` | written, never compiled. The five ports every family names: network, secrets, store, player, reel |
| `adapters/` | not written. `adapters/README.md` names the expected API per port and how sure each one is |
| `surfaces/touch/` | not written. `surfaces/touch/README.md` holds the measured shape a screen here has to meet |

`core/Host.ets` is a line by line port of `apps/android/core/src/main/kotlin/streams/core/Host.kt`,
which is the written and shipped version of this host. Nothing in it is a second opinion about what a
host does, and when that file changes this one is diffed against it. It carries the Android host's
plain-http gate word for word: plain http reaches only an address on the viewer's own network, because
a session's bearer and refresh tokens travel in the clear, and a refused address is said to the viewer
rather than swallowed.

## What cannot be done without the SDK and a device

So that nobody later mistakes this for an app:

1. **Nothing draws.** There is no screen, no navigation, no player view. `surfaces/touch` holds a
   README and no code.
2. **No adapter exists.** All five ports are declared and none is implemented, so the host cannot make
   a request, keep a token, keep a page, or play one frame.
3. **No ArkTS was compiled.** Not one line. This machine has no HarmonyOS SDK and no TypeScript
   compiler either — `node`, `npx` and `tsc` are all absent — so `Ports.ets`, `NativeCore.ets` and
   `Host.ets` have been read and never checked by a tool. `cmake` is absent too, so
   `core/cpp/CMakeLists.txt` has not even been parsed.
4. **The generated contract types may not be legal ArkTS.** They use 40 `(string & {})` intersections,
   80 anonymous union members and a template literal type. This is the one unknown that could change the
   shape of `Host.ets`, and the fix would be in `platform/contract/gen`, not here (`OPEN-QUESTIONS.md`
   Q2).
5. **No project file.** `build-profile.json5`, `oh-package.json5`, `module.json5` and `hvigorfile.ts` are
   versioned with the SDK, and writing them from memory would most likely produce a project that does
   not open. DevEco Studio generates them (Q6).
6. **The NAPI module registration is unverified.** Everything in `napi_bridge.c` is plain Node-API
   except its last block, which is OpenHarmony's own (Q1).
7. **No test, and no harness to write one in.** This family has none. The C glue was syntax checked
   with `clang -fsyntax-only -Wall -Wextra` against a hand written stub of the Node-API surface, which
   catches a typo and proves nothing about the real SDK. There is no equivalent of the Xbox family's
   `smoke_test.cpp` here, because the thing to test is a module loaded by a runtime that is not on this
   machine.

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
