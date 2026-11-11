# What could not be checked, and what would answer it

Written 2026-11-11 on a machine with **no HarmonyOS SDK, no DevEco Studio, no Huawei device, no
TypeScript compiler and no cmake** (`node`, `npx`, `tsc` and `cmake` are all absent). The Rust below
was built and the C was syntax checked; not one line of ArkTS was compiled, and `CMakeLists.txt` has
not been parsed.

Each question is written so that it can be answered in one sitting by someone who has DevEco Studio
and a HarmonyOS device, without reading anything else first.

## Q1. Is the Node-API module registered and imported the way `napi_bridge.c` assumes?

`core/cpp/napi_bridge.c` is plain Node-API except its last block, which uses OpenHarmony's own module
registration: a `napi_module` with `nm_modname = "streamscore"`, handed to `napi_module_register` from
a function marked `__attribute__((constructor))`, imported from ArkTS as `'libstreamscore.so'` with its
types in `core/cpp/types/libstreamscore/index.d.ts`.

Unchecked with it: whether that types folder also needs its own `oh-package.json5` naming
`libstreamscore.so` and pointing at `index.d.ts`, and whether `libace_napi.z.so` is the right name to
link for Node-API (`core/cpp/CMakeLists.txt`).

Answered by: building the empty "Native C++" template project in DevEco Studio and reading the three
files it generates — the registration block, the `CMakeLists.txt` link line, and whatever it puts in
the types folder. If the template disagrees with what is here, the template wins.

## Q2. Do the generated contract types pass the ArkTS linter?

`platform/contract/generated/types/contract.ts` is written for TypeScript, and ArkTS is a narrower
language. Counted in that file on 2026-11-11 with `rg -c`:

- **40** uses of `(string & {})`, an intersection with an empty object type, in every open string enum
  (`Form`, `DeviceClass`, `HttpMethod`, `StoreOp`, and so on).
- **80** members of discriminated unions written as **anonymous** object shapes — `Action`, `Event`,
  `Effect` and `Screen` are all built this way, and ArkTS prefers named interfaces.
- `WireDate` is a **template literal type**: `` `${number}-${number}-${number}` ``.

If the linter refuses any of these, the fix is in `platform/contract/gen` (not in this folder): emit a
named interface per union member and a plain `string` for an open enum. `core/Host.ets` then loses its
three `Extract<...>` aliases, and nothing else here changes.

Answered by: copying that one file into a DevEco project and running the ArkTS linter over it. The
answer is a list of rule names, which says exactly what the generator has to change.

Who it affects: not only this family. `apps/tv` and `apps/web` read the same generated file, so a
change to the generator is a change for them too.

## Q3. Which thread does the host run on, and which call gives monotonic time?

A core handle may be used from one thread at a time (`platform/bridges/c/core.h`). The Android host
runs the core on a `HandlerThread` of its own. A HarmonyOS Worker has a **VM of its own**, so a module
imported inside a Worker — and every handle opened from it — lives there, and every port call has to
be posted across. The alternative is the UI thread, which is simpler and is what `core/Host.ets`
assumes today.

Also unchecked: the call that gives monotonic milliseconds, and the call that gives cryptographic
random bytes for the config's seed. Both are held behind the `Machine` interface in `core/Host.ets`, so
answering this changes the app wiring and not the host. The Android host uses
`SystemClock.elapsedRealtime()` and `SecureRandom`.

Answered by: stepping the core from the UI thread on a device and measuring the longest step against a
frame budget. 12 rows of tiles is the heaviest ordinary step. If a step costs more than about 8 ms, the
core moves to a Worker.

## Q4. What is the ArkUI form of an observable `Seen`?

`Shell.seen` is a `StateFlow` on Android and `@Published` on Apple. `ports/Ports.ets` declares a plain
`Seen` plus `watch(changed)`, which is the job both of those do. ArkUI's own forms are `@Observed` with
`@ObjectLink`, or `AppStorage`. The right choice depends on how a screen is written, and no screen is
written.

Answered by: the first screen. Until then `watch` is honest about what it is.

## Q5. How does a picture reach an ArkUI surface?

`Player` in the other two families has `picture(context) -> Any`, which hands a surface the video as an
object only that toolkit understands (a `SurfaceView` on Android, an `AVPlayerLayer` on Apple).
`Player` in `ports/Ports.ets` leaves it out, because the ArkUI equivalent is unknown here: an
`XComponent` with a surface id handed to `AVPlayer`, or the `Video` component, which may not take per
source headers.

`Reel.picture` is **not** left out: its url and its `ready`/`ended` callbacks are the only way a film
reaches that port, and without them the port has no input. What is unknown there is the type of the
handle it gives back, so it is declared as `Object` and the first adapter names it. Only the type is
open, not the shape of the call.

Answered by: playing one HLS stream on a device. The same test answers whether `createMediaSourceWithUrl`
really sets `Referer`, which is the whole reason this family is wanted (note 151, line 781: header
locked sources may be reachable only on Android, HarmonyOS and desktop).

## Q6. Where do the DevEco project files go?

This folder has the hexagon (`core`, `ports`, `adapters`, `surfaces`) and no project file:
`build-profile.json5`, `oh-package.json5`, `module.json5`, `AppScope/app.json5` and `hvigorfile.ts` are
all versioned with the SDK, and inventing them from memory would most likely produce a project that
does not open.

Answered by: generating an empty project with DevEco Studio and pointing its module at these folders —
`CMakeLists.txt` at `core/cpp`, the surface at `surfaces/touch`. Whether a HarmonyOS module can keep its
sources outside the standard `entry/src/main/ets` tree is itself part of the question; if it cannot, the
folders move and the ring check (handoff below) moves with them.

## Not a question, measured here

The core builds for HarmonyOS with no SDK at all. On 2026-11-11, with rustc 1.98.1:

```
rustup target add aarch64-unknown-linux-ohos armv7-unknown-linux-ohos x86_64-unknown-linux-ohos
cargo build --locked --profile native -p bridge-c --target <each>
```

All three produced `libbridge_c.a`: 34,991,550 bytes for aarch64, 30,993,132 for armv7, 33,316,522 for
x86_64 (unstripped archives, not what ships; the shared library CMake links out of one of these is far
smaller). `./build-core.sh` runs exactly that and was run end to end.

This **corrects** note 151 line 275, which says the OpenHarmony targets "need the SDK's clang wrapped by
hand". A static library is assembled by rustc and never linked, so no linker and no SDK are involved. The
SDK's clang is needed at the next step, when CMake links `libstreamscore.so` against the archive — which
is exactly where the Android family needs the NDK's clang too.
