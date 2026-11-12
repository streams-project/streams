# What could not be checked, and what would answer it

Written 2026-11-11, answered 2026-11-12 from the reference documentation, on a machine with **no
HarmonyOS SDK, no DevEco Studio, no Huawei device, no TypeScript compiler, no node and no cmake**
(all four are still absent; checked again on 2026-11-12).

**Four of the six are closed, and two need a device.** Q1, Q2, Q4 and most of Q3 are answered below
with the pages that answer them; Q5's one unknown and Q6's last step want a device and the SDK. The
code changed where the answers changed it — this is not a list of findings beside unchanged files.

Every source below is Huawei's own reference (`developer.huawei.com/consumer/en/doc/harmonyos-guides`
and `.../harmonyos-references`) or the OpenHarmony documentation it is generated from, read on
2026-11-12. Where a claim is not in one of those it is marked **assumed**.

---

## Q1. Is the Node-API module registered and imported the way `napi_bridge.c` assumes? — **ANSWERED: yes, and one file was missing**

Everything the registration block does is what the Node-API development guide does.

* `napi_module` with `nm_register_func` and `nm_modname`, handed to `napi_module_register` from a
  function marked `__attribute__((constructor))`: that is the guide's own shape, verbatim.
* `nm_modname` must be the shared library's name without `lib` or `.so`, and the guide warns that
  "The module names must be of the same case". `nm_modname = "streamscore"`,
  `add_library(streamscore SHARED ...)` and `import ... from 'libstreamscore.so'` agree.
* `libace_napi.z.so` is the right library to link, confirmed: "add the following dynamic link
  library to CMakeLists.txt: libace_napi.z.so". Logging would want `libhilog_ndk.z.so` as well, and
  this bridge writes no log line, so it is right not to be linked.
* `#include "napi/native_api.h"` is the documented header.

**What was missing, and is now written**: the types folder needs its own `oh-package.json5`. The
guide shows it at `entry/src/main/cpp/types/libentry/oh-package.json5` with `name` and `types`, and
says "Associate index.d.ts with .cpp in the oh-package.json5 file". It is now at
`core/cpp/types/libstreamscore/oh-package.json5`.

**What changed besides**: `armeabi-v7a` is not a thing. `externalNativeOptions.abiFilters` takes
exactly `arm64-v8a` and `x86_64` and defaults to `arm64-v8a` alone, so the 32-bit ARM branch of
`CMakeLists.txt` was unreachable and the `armv7-unknown-linux-ohos` core `build-core.sh` built on
2026-11-11 had nothing to link into. Both are gone. An emulator needs `x86_64` named in `abiFilters`
explicitly, because the default leaves it out.

**Still open, and it needs the SDK**: what the SDK's toolchain file calls the architecture it is
building for. `OHOS_ARCH` is this folder's guess at the variable name; the two values it is now
compared against are the reference's. Answered by: one `hvigorw assembleHap` with a
`message(STATUS ...)` in the CMake file.

---

## Q2. Do the generated contract types pass the ArkTS linter? — **ANSWERED: no, and they do not have to**

Three of the four constructs the question asked about are hard compile errors in ArkTS, each with a
rule name and an error code, each severity `error`, which the migration cookbook defines as "the
program will fail to compile":

| Construct | Count in the generated file | Rule | Code |
|---|---|---|---|
| `(string & {})` | **40** | `arkts-no-intersection-types` | 10605019 |
| anonymous object type in a union | **80** | `arkts-no-obj-literals-as-types` | 10605040 |
| `Extract<...>` | used by this folder, not by the generator | `arkts-no-utility-types` | 10605138 |
| template literal type | **3** (`TitleKey`, `WireDate`, `CalendarDay`) | none found | — |
| `as const` on `ADDRESS` | **1** | `arkts-no-as-const` | — |

Counted with `rg` in `platform/contract/generated/types/contract.ts` on 2026-11-12. Only the
template literal types are genuinely unknown: no rule forbids them and no page documents them, and
the nearest hard rules (`arkts-no-conditional-types`, `arkts-no-mapped-types`) suggest the
type-level machinery is thin.

**And none of it has to change, because the rules are scoped by file extension.** `arkts-no-ts-deps`
(10605147) states the direction outright:

> "An .ets file can import source code of an .ets/.ts/.js file, but a .ts/.js file cannot import
> source code of an .ets file."

and the migration background page scopes the check itself: "the SDK adds the ArkTS syntax validation
**for .ets files** to the compilation process". The official interop example is an `.ets` file
importing both a type and a runtime value from a `.ts` file, with the `.ts` file's own diagnostic
attributed to "TS strict mode".

So this folder gained a seam instead of the generator gaining a rewrite:

* `contract/effects.ts` — the unions narrowed with `Extract`, and a builder per member of `Event`,
  `Action` and `Command`, so no `.ets` file ever writes a contract literal.
* `contract/read.ts` — one typed read per body, because ArkTS bans `any`
  (`arkts-no-any-unknown`, 10605008) and `JSON.parse` is typed `any`.

`core/Host.ets` is 290 lines shorter in intent for it: three `Extract` aliases and every `as Event`
literal are gone, and the file names no type ArkTS cannot.

**The price, stated in the reference and paid knowingly**: "when data and objects of TS/JS are used
as those of ArkTS, the static type check of ArkTS may be bypassed, causing unexpected behavior or
extra overhead". Which is why the seam names every shape it exports rather than widening anything.
Two more constraints that bind it: a `.ts` file can never import an `.ets` file, so the seam is a
leaf; and the ArkCompiler runtime prohibits circular dependency outright ("Application loading
fails").

**Who it affects**: not this family. `apps/web` and `apps/tv` read the same generated file and are
unaffected, which they would not have been by a generator change.

**Still open**: whether the compiler applies any `arkts-no-*` rule to a `.ts` file. The scoping
sentence says no and there is no sentence that enumerates it rule by rule. Answered by: one local
build with a deliberate `(string & {})` in a `.ts` file imported from `.ets`.

Nine other ArkTS rules bear on anything written in `.ets` here, and all nine are obeyed — the
audit for them is in the note. The two that are easy to miss: **a regular expression LITERAL is
refused** (`arkts-no-regexp-literals`, recorded only on the Adaptation Cases page, not in the
cookbook), which `core/Host.ets` had two of; and **destructuring is refused in all three forms**.

---

## Q3. Which thread does the host run on, and which call gives monotonic time? — **the calls are ANSWERED; the thread needs a device**

**Monotonic time, answered.** `systemDateTime.getUptime(timeType, isNanoseconds)` from
`@kit.BasicServicesKit`, API 10, no permission. `TimeType.STARTUP` is "Number of milliseconds
elapsed since system startup, **including** the deep sleep time" and `ACTIVE` excludes it — so
STARTUP is Android's `SystemClock.elapsedRealtime()` and ACTIVE is `uptimeMillis()`. STARTUP is the
one this host wants. (`getRealTime` is deprecated since API 12 in favour of it.)

**Cryptographic random, answered.** `cryptoFramework.createRandom().generateRandomSync(len)` from
`@kit.CryptoArchitectureKit`, API 10, returning a `DataBlob` whose `data` is a `Uint8Array`.

**The device's language, answered.** `i18n.System.getSystemLanguage()` from `@kit.LocalizationKit`,
API 9, no permission; it answers with a script subtag ("zh-Hans"), not a bare language. `Machine`
gained a third call for it, and `Host.ets` no longer asks for English unconditionally before a
profile.

All three are behind `Machine` in `core/Host.ets` and are the app wiring's to supply, so the host
still names no HarmonyOS module.

**The thread is still open, and it is the one that wants a device.** The question stands as written:
a core handle may be used from one thread at a time, a HarmonyOS Worker has a VM of its own, and the
UI thread is what `core/Host.ets` assumes. Answered by: stepping the core from the UI thread on a
device and measuring the longest step against a frame budget. 12 rows of tiles is the heaviest
ordinary step. If a step costs more than about 8 ms, the core moves to a Worker — and
`surfaces/touch/Paging.ets` carries the comment that says what else has to be looked at if it does.

One related fact worth having before that measurement: the Node-API FAQ warns that `nm_modname`
"does not match the module name in multi-thread scenarios (such as worker threads and taskpool)",
so moving the core to a Worker is not only a matter of posting calls across.

---

## Q4. What is the ArkUI form of an observable `Seen`? — **ANSWERED, and it found a bug**

`Seen` stays a plain class, `Shell.watch` stays, and the screen side is one `@Local` field in
`surfaces/touch/Root.ets`. What makes the assignment a redraw is that **the host hands watchers a
fresh `Seen` each time** (`Seen.copy`).

That is not a style choice; the alternative was broken. `@Local` notices that its variable was
**assigned**, not that a field inside the object it already holds was written. `core/Host.ets` wrote
fields into one `Seen` and handed the same object out, so every screen would have held an object it
had no reason to draw again and the app would have looked frozen from the first step. `Host.kt` does
not have that bug: it holds `MutableStateFlow<Seen>` and every write is `_seen.value =
_seen.value.copy(...)`, a new object per change. The copy is now there, field by field, because
ArkTS bans `Object.assign` and the whole of `Reflect` (`arkts-limited-stdlib`, 10605144).

**Rejected: `@ObservedV2` with `@Trace` on `Seen`** (API 12). It would give field-level observation
with none of the work above — and it would put a UI framework decorator in the ports ring, which is
the mistake `apps/android/ports/.../Ports.kt:128-140` records being caught with okhttp3.
**Rejected: a version counter in `@Local`** — it redraws the whole tree on any change and loses
field diffing.

`@Observed`/`@ObjectLink` and `AppStorage`, the two forms the question named, are both V1; this
family is V2 throughout, and `@Reusable` is explicitly "not compatible with @ComponentV2" (the V2
form is `@ReusableV2`, API 18).

---

## Q5. How does a picture reach an ArkUI surface? — **ANSWERED for the port's shape; the header test needs a device**

**`Player.picture` is a surface id, and the port now carries it.** An `XComponent` of type `SURFACE`
reports one through `onSurfaceCreated(surfaceId: string)` — which the reference says fires "only
when the libraryname parameter is not set", the right case here since the player is ArkTS and not
native — and `AVPlayer.surfaceId` takes it. So the port's call is `picture(surfaceId: string)`, not
`picture(context) -> Any`: the surface ring makes the surface and hands the adapter the one string
that names it. The current `XComponentOptions` is `{ type, controller }`; the `{ id, type,
libraryname }` form is deprecated since API 12.

A bonus the reference states outright: "The **XComponent** component does not support mirroring
capabilities", which is the behaviour this project already wanted — the interface mirrors for Arabic
and the picture never does.

**Per-source headers are confirmed, and `Referer` is named.**
`media.createMediaSourceWithUrl(url, headers?: Record<string, string>)` is API 12, paired with
`setMediaSource(src, strategy?)`, callable only in the `idle` state. The streaming guide's own
example is `media.createMediaSourceWithUrl(url, {"User-Agent":"…","Cookie":"…","Referer":"…"})`.
There is no way to attach headers to the plain `url` property and no way to change them mid-stream
without `reset()` back to idle. HLS is confirmed supported for both on-demand and live.

The `Video` component has no header field of any kind, and the reference redirects: "For complex
development scenarios, you are advised to use the AVPlayer playback control API and the XComponent
component." So the guess in the original question — `Video` "may not take per source headers" — is
right, and `Video` is out.

**Pictures got a port of their own**, which this question did not ask for and the family needed:
`Image(src)` accepts a url string and then performs the request **inside the UI framework**, so a
screen that wrote a url into a component would have the network reaching past its port with no line
of code naming it. `Pictures` (matching the one Android and Apple added on 2026-11-12) hands back an
`image.PixelMap`, which `Image(src: PixelMap | ResourceStr | DrawableDescriptor)` takes.

**Still open, and it needs a device**: whether `createMediaSourceWithUrl` really sets `Referer` on
the wire. The document says it does; note 151 line 781 is the reason the family is wanted at all, so
this is the one test that justifies it. Answered by: playing one header-locked HLS stream on a
device and reading the request the server saw.

---

## Q6. Where do the DevEco project files go? — **the layout question is ANSWERED; generating them needs the SDK**

**A module can live here.** The project-level `build-profile.json5` has `modules[].srcPath`, "Path
from the project root to the module root", and the reference adds "**The module can sit outside the
project**". So `apps/harmony` can be the module root, pointed at from a project file elsewhere, and
the folders do not have to move into somebody else's tree.

**What is inside a module is fixed.** There is no `sourceRoots` field and no official statement
about relocating `src/main/ets` — searched the app-level schema and the build-customizing guide and
found nothing either way. So the half of the question that asked "whether a HarmonyOS module can
keep its sources outside the standard `entry/src/main/ets` tree" is answered **no, assume not**: the
module root may be anywhere, and inside it the standard tree holds. What that means for this folder
is a thin module wrapper whose `src/main/ets` re-exports `surfaces/touch`, `core` and `ports`, or a
move of the three ring folders under it. That is a decision for whoever runs the SDK, and both
options keep the hexagon.

The required file set and its schema pages are now known: project `build-profile.json5` (`app` and
`modules`, with a product that must be named `default`), module `build-profile.json5` (where
`externalNativeOptions.path` points at `core/cpp/CMakeLists.txt`), `oh-package.json5` at both levels,
`module.json5` (with `srcEntry` naming the ability's entry file), `AppScope/app.json5`, and
`hvigorfile.ts` at both levels.

**Still open, and it needs the SDK**: the files themselves, with the version numbers of the day.
Answered by: generating an empty project with DevEco Studio or the command line tools and pointing
its module at these folders.

---

## Not a question, and it changes what "blocked" means

**ArkTS can be compiled and linted on this machine, with no DevEco Studio and no device.** The
Command Line Tools package is a separate download whose contents the reference lists verbatim:
"**codelinter** (code check tool), **hstack** (stack parsing tool), **hvigorw** (command line build
tool), **ohpm** (third-party dependency management tool), and a series of tools contained in the
SDK". `hvigorw assembleHap` builds a HAP; `codelinter <projectRoot> -f json -o out.json -e error`
lints the TypeScript and ArkTS in a project. Neither mentions a device.

Two honest limits. `codelinter` is a **lint** tool and not the ArkTS type checker: the `arkts-no-*`
rules above are compiler errors with error codes, and they surface during a build, not from
codelinter's rule sets. And `codelinter`'s `dir` argument is documented as a project root, so a
folder of loose `.ets` files is outside its documented usage. Both point the same way: what would
check this folder is a project file (Q6) and `hvigorw assembleHap`.

**DevEco Studio runs on an Intel Mac.** The stated requirement is "macOS (x86) 11/12/13/14/15/26 or
macOS (Arm) 12/13/14/15/26", with separate installers. The emulator's host requirements are **not**
stated either way on any page reached; what is stated is that its images are "currently available
only in the Chinese mainland".

So the precise unblock is one download: `developer.huawei.com/consumer/en/download/command-line-tools-for-hmos`.
Whether it needs a Huawei ID, or a Chinese one, is not said on the page and could not be checked
without signing in — **that is the open item**, and it is a sign-in, not a device.

## Not a question, measured here

The core builds for HarmonyOS with no SDK at all. On 2026-11-11, with rustc 1.98.1:

```
rustup target add aarch64-unknown-linux-ohos x86_64-unknown-linux-ohos
cargo build --locked --profile native -p bridge-c --target <each>
```

Both produced `libbridge_c.a`: 34,991,550 bytes for aarch64 and 33,316,522 for x86_64 (unstripped
archives, not what ships). `./build-core.sh` runs exactly that and was run end to end. The third
target it built that day, `armv7-unknown-linux-ohos` at 30,993,132 bytes, has been dropped: see Q1.

This **corrects** note 151 line 275, which says the OpenHarmony targets "need the SDK's clang
wrapped by hand". A static library is assembled by rustc and never linked, so no linker and no SDK
are involved. The SDK's clang is needed at the next step, when CMake links `libstreamscore.so`
against the archive — which is exactly where the Android family needs the NDK's clang too.
