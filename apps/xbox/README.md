# xbox

The Xbox app. A controller gives four directions and a select, so Xbox draws the same grammar as a
television: this is a **remote** surface. The core is native code here.

Every measured figure below comes from the Netflix reference folder in the project's notes, which
sits outside this repository; paths like `netflix-reference/tv/...` are inside that folder.

Status: no app, and one native route left standing. This folder holds a README, a `.gitignore`,
`host/` (the C++ side of the core boundary, with a test that drives it) and `surfaces/remote/` (the
input grammar, with a test). Nothing draws a screen, and the reason is written down: there is no C++
contract, so no screen can read what the core hands back. Both tests run on a developer's own
machine with no Xbox: `./apps/xbox/check.sh`.

The core cross-compiles for Windows from macOS, which was the one architectural question this
folder carried. See [The core on Xbox](#the-core-on-xbox).

The root README's status table still groups this folder with the ones that hold a README and nothing
else; that line is now wrong and it is not owned here.

Part of an open video platform.

## What this file said before, and why it was wrong

> A small shell that runs the television app on Xbox through WebView2.

Wrong twice, as of 2026-11-11.

* The television app it meant is `apps/tv`, a page served from the server. Work on it stopped on
  2026-11-07 when the owner decided every platform gets a binary app (project note 179). Xbox was
  defined in terms of a surface that is no longer the product.
* WebAssembly is allowed only where the one open runtime of a platform is a web engine - Tizen,
  webOS, Vizio and the web app (owner, 2026-11-11). Xbox takes native code, so the core here is the
  same static library the Android and Apple families link, not `core.wasm`.

Project note 167 (2026-11-06) also carries "Xbox needs one small shell" and `apps/xbox` against a
WebView2 line. Both readings are superseded by the two decisions above.

## The surface is remote

A remote surface speaks six words. They already exist, as `Word` in
`apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Remote.kt:22`, and Xbox adds no seventh:

| Word | Gamepad | Xbox remote control |
|---|---|---|
| UP, DOWN, LEFT, RIGHT | D-pad, and the left stick | D-pad |
| OK | A | Select |
| BACK | B | Back |

Verified on Microsoft's own mapping table: arrow keys map to the D-pad and the left stick, Spacebar
and Enter map to A/Select, and Escape maps to B/Back. X and Y, both sticks, both triggers and both
bumpers are listed for the gamepad and **not** for the remote control, so nothing a viewer needs can
hang on them. Source: [Gamepad and remote control
interactions](https://learn.microsoft.com/en-us/windows/uwp/ui-input/gamepad-and-remote-interactions),
read 2026-11-11.

That table is the whole mapping and it is now code: `surfaces/remote/remote.h`, with 39 checks in
`surfaces/remote/remote_test.cpp` that run on any machine. Three things make a controller different
from a television remote, and all three live in that one file rather than in any screen:

* **A controller is polled, a remote is delivered.** A television remote arrives as key events and
  the system repeats a held key. A gamepad is read as a state each frame with nothing behind it, so
  a held direction has to be counted in the surface (`Repeat`). Its two numbers, 500 ms before the
  first repeat and 120 ms between, are **not measured** - see open item 6.
* **A controller has a second way to say the four directions**: the left stick, which is analogue
  and never reports zero at rest. A dead zone decides when a push counts, and it has to be left
  before it counts again or a resting hand walks the screen. The dead zone is
  `XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE`, 7849 of a 32767 axis, read as a circle rather than per axis
  as Microsoft recommends. Source: [Getting started with
  XInput](https://learn.microsoft.com/en-us/windows/win32/xinput/getting-started-with-xinput)
  (`ms.date` 2025-07-14, read 2026-11-12). The 5000 it must fall back under is our choice.
* **A controller has eight buttons the Xbox remote control has not**: X, Y, both sticks' own
  buttons, both bumpers, View and Menu. None of them says a word, because a screen that needed one
  would not work for a viewer holding the remote. The code answers "no word" for each by name, and
  the test checks all eight.

One trap, recorded because the names collide: the pad's own button at bit `0x0020` is called `BACK`
in `xinput.h` and `View` on the controller, and it is **not** our BACK word. BACK is B, bit
`0x2000`. The bitmasks are verified from [the XINPUT_GAMEPAD
reference](https://learn.microsoft.com/en-us/windows/win32/api/xinput/ns-xinput-xinput_gamepad)
(`ms.date` 2018-12-05, read 2026-11-12), which is the one button layout Microsoft writes down with
numbers; GameInput's and WinRT's own flag values are not written down here, because neither is
reachable from this machine and neither will be guessed.

Two platform facts decide how that surface is built, rather than how it looks:

* **Mouse mode is on by default** for XAML and hosted web apps on Xbox. Microsoft: "For both XAML
  and hosted web apps running on Xbox One, mouse mode is turned on by default for the entire app. It
  is highly recommended that you turn this off and optimize your app for XY navigation", by setting
  `Application.RequiresPointerMode = WhenRequested`. A host that forgets this ships a pointer surface
  on a console, which is the opposite of the decision above.
* **Back is the app's own job.** When the app handles neither `KeyDown` nor `KeyUp` for B,
  `SystemNavigationManager.BackRequested` fires, and Microsoft's own words are "which should result
  in back navigation within the app. However, you have to implement this yourself". So the measured
  Netflix behaviour is reachable on Xbox: four consecutive `BACK` presses on a television never left
  Netflix (verified, n=4, `netflix-reference/tv/00-netflix-on-the-tv-measured.md` section 7), and the
  way out is a menu line. Gap 7 in `netflix-reference/00-gaps.md` records that our own television
  surface leaves the app on Back from Home and that whether it should is **open**.

### The numbers this surface holds

Not restated here, because they are measured elsewhere and one of our surfaces is already scored
against them. A remote surface for Xbox is held to the same figures as `apps/android/surfaces/tv`,
which gap 9 of `netflix-reference/00-gaps.md` scores against Netflix line by line. That scoring is
not a match: its own heading reads "TV geometry: five small misses, all under 1.2% except the bar",
and the misses are gutter 17 against our 19, the top bar 53 px against 64, a 2 line synopsis against
3, and the first tab label at x 642 against 609. The four figures that govern the grammar rather
than the decoration:

* The focused row is pinned to a fixed band, y 199-708 of 1080, in 24 of 24 sampled frames. The
  content scrolls under it and focus never walks down the screen.
* A closed tile is a 311 x 505 portrait poster; the same tile focused is a 906 x 510 landscape still,
  2.91x the width at the same height. Two assets per title per row, not one image scaled.
* Tile pitch 328, gutter 17, so 5.85 closed tiles cross the screen. A Top 10 row keeps the 311 tile
  and widens the gutter to 146.
* While a tab loads the body is pure black for 1.1 to 4.7 s, with the top bar standing and the
  destination tab already lit. No skeleton, no held content.

Source for all four: `netflix-reference/tv/00-netflix-on-the-tv-measured.md`, sections 6b, 5 and 10,
measured 2026-11-11.

One figure that is **not** measured on a television, and an earlier draft of this file put it in the
list above as though it were. Netflix's **web** card carries no text at all - `<img alt=""
height="192" width="342" class="standard-card tracked-card">` - with titles and badges composited
into the artwork server side. That is gap 4 of `netflix-reference/00-gaps.md` (line 84), whose own
evidence is the web section, "What surprised me" item 2 (`netflix-reference/web/README.md:234`). The
TV note records the opposite for the one Home row it measured: "Each tile is a word or a show logo
on a dark rounded rectangle - no artwork at all" (section 6). So whether a remote surface's cards
carry text is open, and nothing here settles it from a TV measurement.

Xbox would add one piece of arithmetic to those four, **if** it goes the XAML route. The console
renders at 1920 x 1080 whatever the television is, and scales a XAML app 200% by default, so a XAML
layout is designed at 960 x 540 effective pixels and every measured figure above halves. Source:
[Designing for Xbox and
TV](https://learn.microsoft.com/en-us/windows/apps/design/devices/designing-for-tv), read
2026-11-11. The halving is a property of XAML's scale, not of the console: on Win32 with the Game
Development Kit there is no XAML scale and the figures stand as measured. Open item 1 below is the
one that decides which, so until it is answered neither form of the arithmetic is settled.

## What an Xbox application is, for a developer in this position

Read on Microsoft Learn on 2026-11-11. Each line says where it came from, because four of these
pages now sit under `previous-versions` with `is_archived` set and `NOINDEX`: the UWP on Xbox doc set
is archived, while the FAQ (updated 2026-01-14) and the gamepad page (updated 2026-03-04) are live.

| Verified | Source |
|---|---|
| x64 only: "All apps and games must target the x64 architecture in order to be developed or submitted to the store for Xbox", and "Xbox no longer supports x86 app development or x86 app submissions to the store" | [system resources](https://learn.microsoft.com/windows/uwp/xbox-apps/system-resource-allocation) (archived), [FAQ](https://learn.microsoft.com/windows/uwp/xbox-apps/frequently-asked-questions) |
| An app gets **1 GB** of memory in the foreground, 128 MB in the background, a share of 2 to 4 CPU cores and a share of 45% of the GPU. A game gets 5 GB, 4 exclusive cores and the whole GPU. On Series X\|S an app gets DirectX 11 and 12 at feature level 11.0 | system resources (archived) |
| Apps are still publishable: "You can publish apps through Partner Center if you have a developer account. UWP apps created and tested on a retail Xbox One console will go through the same ingestion, review, and publication process that Windows conducts today, with additional reviews to meet today's Xbox One standards" | FAQ |
| UWP **games** are not: "UWP based games are no longer accepted in the Xbox Store. Please use the ID@XBOX program" | FAQ |
| Developer Mode runs on any retail console, through the Xbox Dev Mode app in the Store, with an activation code entered in Partner Center at `partner.microsoft.com/xboxconfig/devices`. It needs a registered Partner Center app developer account, and "You have a limited number of activations associated with your account" | [Developer Mode activation](https://learn.microsoft.com/windows/uwp/xbox-apps/devkit-activation) (archived) |
| In Developer Mode "you cannot play retail games or run retail apps". The console switches both ways | Developer Mode activation (archived) |
| A Dev Mode pass does not prove a retail pass: "the package contents are encrypted on an Xbox in retail mode, but not in Dev Mode. As a result, the app may appear to work as expected during development and testing, but then fail once it has been published and then installed to a retail Xbox" | [known issues](https://learn.microsoft.com/windows/uwp/xbox-apps/known-issues) (archived) |
| UWP apps on Xbox may not bind ports 57344 to 65535; traffic can be dropped silently. Use port 0, or 1025 to 49151 | known issues (archived) |
| Rust has six `*-uwp-windows-*` targets and all six are **tier 3**: not built or tested by the project, "may or may not work", no official builds. `x86_64-pc-windows-msvc` is tier 1 | [platform support](https://doc.rust-lang.org/nightly/rustc/platform-support.html) |
| Our C bridge needs `std`: `platform/bridges/c/src/lib.rs:3` uses `std::panic::catch_unwind` to turn a panic into status 2. The core and the offline layer are both `no_std` | read in this tree |

Read again on 2026-11-12, because the two routes below were the decision this folder waited on and
three of the lines above were softer than the evidence allows:

| Verified, 2026-11-12 | Source |
|---|---|
| **A developer account costs nothing.** "With the new onboarding experience, there are **no registration fees** for either account type, so you can create your developer account and start publishing at no cost." Individual onboarding needs a personal Microsoft account and identity verification with a government-issued ID and a selfie, and the only entry point for the free flow is `storedeveloper.microsoft.com` | [Open a developer account](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account), `ms.date` and `updated_at` both 2026-10-29, live page |
| **UWP is supported and frozen.** "Although still supported, UWP is not under active development." Microsoft recommends the Windows App SDK and WinUI for new Windows apps. The same page still names Xbox among the device families one submission reaches | [What's a UWP app?](https://learn.microsoft.com/en-us/windows/uwp/get-started/universal-application-platform-guide), `ms.date` 2024-08-21, `updated_at` 2026-04-09, live page, `ROBOTS: INDEX, FOLLOW` |
| **The GDK route to a console is closed to anyone outside a managed programme.** The public GDK on GitHub carries the *common* tools only; console work needs the Xbox Extensions (GDKX), which are "only available under confidential license within an NDA Xbox program", ID@Xbox being the example given, and the Xbox graphics driver and the DirectX 12 shader compiler for Xbox are among what the public package lacks | [Public GDK](https://github.com/microsoft/GDK) README, read 2026-11-12 |
| The same, from Microsoft's own docs: secure Xbox downloads "require that you are part of an organization that has been onboarded as an Xbox development partner, and that you or your employer has a valid Microsoft Game Development Kit (GDK) Agreement", access is managed through a Partner Center account with a Microsoft Entra tenant, and "Personal Microsoft accounts (MSAs) are no longer supported for access at this time" | [Access GDK development resources](https://learn.microsoft.com/en-us/gaming/gdk/docs/gdk-dev/development-downloads/access-resources), `updated_at` 2026-05-12, live, monikers `gdk-2510`/`gdk-2604`/`gdk-2610` |
| The GDK doc set is current, not archived: its input FAQ carries `ms.date` 2026-10-29 and `updated_at` 2026-11-04, and names the October 2026 GDK (2610) runtime. So the GDK is alive and the barrier is the licence, not the age of the route | [GameInput FAQ](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-faq) |
| **The Rust core cross-compiles for Windows from this Mac.** `x86_64-pc-windows-msvc` is a tier 1 target, `rustup target add` installs it with no Visual Studio, and a `staticlib` needs no linker because `rustc` writes the archive itself | run in this tree, see below |

| Not verified, and what would settle it |
|---|
| **Whether a retail Xbox forbids dynamic code generation.** No Microsoft page read here says so for Xbox. The nearest documented fact is about .NET: runtime code generation fails under the .NET Native toolchain. Settled by the Xbox Requirements set that comes with Partner Center or ID@Xbox onboarding, or by a retail install of a package carrying a code generating runtime. It changes nothing here: the owner's rule already keeps WebAssembly off Xbox, so no engine of ours needs to generate code at run time. |
| **Whether a new app submission can select the Xbox device family without separate approval, and whether an individual account is accepted for a media app.** The FAQ says apps go through the Windows process "with additional reviews to meet today's Xbox One standards" and does not say what those reviews are. Settled by a Partner Center account and one submission attempt. |
| ~~**The registration fee.**~~ Settled on 2026-11-12: there is none. See the row above. |
| **Whether WebView2 on Xbox is open to any UWP app.** The announcement says only "WebView2 is now available on Xbox too!" and names no programme or eligibility. Reported elsewhere as UWP only, with no GDK path. Moot for the core under the owner's rule, and it still decides whether a WebView2 shell is available at all. |
| **Whether `std` on a Windows target survives the app container.** The tier 3 `x86_64-uwp-windows-msvc` target exists because the app container forbids part of Win32, and tier 3 means nobody tests it. Settled by a Windows machine with Visual Studio and the Windows SDK, building `bridge-c` for both targets and running the certification kit over the package. |
| **Whether 1 GB holds the artwork.** Netflix pulled 2.17 MB over 15.2 s before its first screen settled and spent 288 kB of GPU texture per focus step on the profile gate, measured on a Google TV Streamer, not on an Xbox. Two assets per tile against a 1 GB ceiling is a number to measure, not to assume. Settled by a console. |

There is no console here and no Microsoft developer account, so nothing in the second table was
attempted on hardware. What changed on 2026-11-12 is the cost of the account: it is nothing, so the
route to a retail console in Developer Mode is gated on identity verification and a console, not on
a fee or a programme. Developer Mode's own page is archived (`ms.date` 2023-09-23, `updated_at`
2024-10-30, canonical under `previous-versions`), and it is still the only documented way to run
your own code on a console you bought in a shop.

## The core on Xbox

`host/` is the only code in this folder. It is the same boundary every native family crosses - six C
functions carrying UTF-8 JSON, described in `platform/bridges/README.md` - written once for C++:

| File | What it does |
|---|---|
| `host/core_host.h` | `streams::Core`: open, step, close, and the four statuses by name |
| `host/core_host.cpp` | the boundary. Out bytes are copied once and handed straight back to `core_free` |
| `host/smoke_test.cpp` | 11 checks that drive a real core through that class. Not run by CI |
| `surfaces/remote/remote.h` | the six words, the button mapping, the stick's dead zone, the repeat clock |
| `surfaces/remote/remote.cpp` | that grammar. No WinRT, no core, no screen: a mask and two axes in, words out |
| `surfaces/remote/remote_test.cpp` | 39 checks over all of it. Not run by CI |
| `check.sh` | builds and runs both tests on whatever machine this is |

It holds no rules, exactly as `apps/android/core/src/main/cpp/bridge.c` holds none for Java.

Ran on this machine, 2026-11-12, x86_64 macOS 25.6.0, rustc 1.98.1:

    ./apps/xbox/check.sh

39 of 39 held for the input grammar and 11 of 11 for the boundary, no compiler warnings under
`-Wall -Wextra`. That proves the C++ side of the boundary, the config shape a console sends
(`"class":"box"`, `"form":"tv"`) and the whole input grammar. It proves nothing about Xbox.

**CI never runs it.** The root `./check` is the whole of CI and it carries no C++ compiler and no C++
test runner, and `platform/tools/check_rings.py` has no `xbox` family, so these tests run only when
someone types `./apps/xbox/check.sh`. They will go stale silently the first time the C ABI or the
core's config shape moves. `check.sh` exists so that it is one command rather than four, which is as
far as this folder can take it: registering it in `./check`, behind a "no C++ toolchain, not run"
line like `check_types.py` already prints for a missing compiler, belongs to whoever owns that file.

### The Windows library, built on a Mac

This file said on 2026-11-11 that the Windows build "needs a Windows machine with the MSVC
toolchain". That was wrong, and it was the sentence that made this folder look blocked. A
`staticlib` is a bag of object files with an index: `rustc` writes the archive itself and never
calls a linker, so no `link.exe`, no Visual Studio and no Windows SDK are needed to produce one.

Run on this machine, 2026-11-12, x86_64 macOS 25.6.0, rustc 1.98.1:

    rustup target add x86_64-pc-windows-msvc
    cargo build --locked --profile native -p bridge-c --target x86_64-pc-windows-msvc

`target/x86_64-pc-windows-msvc/native/bridge_c.lib`, 24,544,044 bytes, 313 archive members of which
306 carry the COFF machine word `0x8664`, which is `IMAGE_FILE_MACHINE_AMD64`. All six C functions
are present and global: `core_abi`, `core_alloc`, `core_free`, `core_open`, `core_step`,
`core_close`. The archive names `ucrt`, `api-ms-win-core-synch-l1-2-0`, `bcryptprimitives` and
`kernel32`, which is the Windows that `std` wants under it.

So the core is ready for Xbox before Xbox is ready for us. Three things that library does **not**
prove, and must not be read as proving:

* **It has never been linked.** Linking an executable is where `link.exe`, the Windows SDK and the
  Xbox libraries come in. A library that assembles is not a library that links.
* **Those four imports may not all be allowed in an app container.** A UWP app runs in one and only
  a subset of Win32 is reachable; the archived [API Sets for UWP
  apps](https://learn.microsoft.com/en-us/previous-versions/mt186421%28v=vs.85%29) list is what
  settles which, and the tier 3 `x86_64-uwp-windows-msvc` target exists precisely because of this.
  Open item 7.
* **x64 is right, and that is checked**: Xbox is x64 only (see the table above), and this is the x64
  target.

`x86_64-pc-windows-gnu` was not attempted. It would need mingw-w64, which is not installed and
cannot be installed here, and the Xbox toolchain is MSVC, so the gnu target would prove the wrong
thing.

## Video on the screen, and a controller read

The two things an app must do that a core cannot. Both are WinRT, and both are therefore the first
work in this folder that cannot even be *compiled* here.

| What | How, on the UWP route | State |
|---|---|---|
| A controller read | `Windows.Gaming.Input.Gamepad`, or the key events Microsoft's own mapping table describes, where the D-pad and left stick arrive as arrow keys, A as Spacebar or Enter and B as Escape | verified that the mapping exists (the gamepad page, read 2026-11-11); the flag values are not written down here |
| Video on the screen | `Windows.Media.Playback.MediaPlayer`, rendered by `MediaPlayerElement` in XAML, or in **frame server mode** - `IsVideoFrameServerEnabled`, then `CopyFrameToVideoSurface` into an `IDirect3DSurface` - which is the route for an app that presents its own frames with DirectX rather than hosting XAML | verified |
| The GPU underneath | An app, not a game, gets DirectX 11 and 12 at feature level 11.0 on Series X\|S, 1 GB of memory, 2 to 4 shared cores and 45% of the GPU | verified, from the archived system resources page |

Source for the player: [Play audio and video with
MediaPlayer](https://learn.microsoft.com/en-us/windows/apps/develop/media-playback/play-audio-and-video-with-mediaplayer),
`ms.date` 2026-08-23, `updated_at` 2026-08-27, read 2026-11-12. Noted because it matters: the UWP
address for that article now answers with the WinUI one, so Microsoft has folded the UWP media
documentation into the Windows App SDK set. `Windows.Media.Playback.MediaPlayer` is still a WinRT
class and still reachable from C++/WinRT; the article's examples are C#.

On the GDK route the input API is GameInput, which is "available on XBOX ... Windows 10 (version
19H1/May 2019 update and later)" and ships in the `Microsoft.GameInput` NuGet package, with the
runtime inside the Game OS image on console (GameInput FAQ, `ms.date` 2026-10-29). It is noted for
completeness only: that route is behind the licence wall in open item 1.

### Where hardware becomes necessary

Seven steps from here to an Xbox app, and the ladder is worth reading because the first four need no
Xbox at all:

| Step | Needs | State |
|---|---|---|
| 1. The core as a Windows x64 static library | nothing but this Mac | **done**, 2026-11-12 |
| 2. The remote's input grammar | nothing but a C++ compiler | **done**, 39 checks |
| 3. The contract as C++ types | a `cpp.rs` in `platform/contract/gen` | open item 8, no hardware |
| 4. The screens, in the remote grammar, against those types | step 3 | blocked by step 3, no hardware |
| 5. The five ports as WinRT adapters: network, secrets, store, player, reel | **a Windows machine** with Visual Studio, the Windows SDK and the C++/WinRT projection. Not a console - the headers do not exist on macOS and cannot be installed here | hardware wall 1 |
| 6. Run it, see it, measure it | **a retail Xbox** in Developer Mode, plus a free developer account with identity verification. One activation of a limited number | hardware wall 2 |
| 7. Publish it | a Store submission, plus the unnamed "additional reviews to meet today's Xbox One standards" | open, unquantified |

So the honest line is between 4 and 5, and it is a Windows machine rather than a console. Steps 3 and
4 are the largest piece of remaining work in this folder and neither of them needs anything bought.

## Folder shape

| Folder | State |
|---|---|
| `host/` | written. The core boundary and its test |
| `surfaces/remote/` | the input grammar, written and tested. **No screens**, and none can be written yet: see below |
| `ports/` | not written. The five ports every family names: network, secrets, store, player, reel |
| `adapters/` | not written. What each port does on Xbox, through WinRT |

The empty two are listed and absent on purpose: a folder here would claim work nothing has released.
`platform/tools/check_rings.py` has no `xbox` family, so nothing yet holds this folder to its ports;
adding one belongs to `platform/tools`.

### Why there are no screens, and it is not the console's fault

A screen reads the view the core hands back. On Android that view arrives as generated Kotlin types
(`contract.Target`, `seen.words`, `seen.route`); on Apple, as generated Swift. The generator is
`platform/contract/gen`, and it writes **five** languages: `java.rs`, `kotlin.rs`, `swift.rs`,
`typescript.rs` and `openapi.rs`. There is no `cpp.rs`.

So the next thing standing between this folder and a screen is not a console, not an account and not
a licence. It is a C++ emitter in the contract generator, which is a file in
`platform/contract/gen/src/` and belongs to the contract's owner, not here. Hand-writing the
contract types in C++ instead would put a second copy of the contract in the tree that nothing
checks, which is how the two versions drift. Open item 8.

The input grammar does not wait on that, which is why it exists: a key press never crosses the C
boundary. `EventBody` in `platform/contract/src/core_msg.rs:279` has sixteen kinds and not one of
them is a key - the surface keeps its own focus and sends `Action` and `Address` events, exactly as
`Television.kt:92` and `Browse.kt:109-148` do. A remote surface's key map is therefore entirely the
surface's own, and entirely testable without a core.

## What is still open

1. ~~**Which native route Xbox takes.**~~ **Answered on 2026-11-12, and the answer is forced.** The
   unverified half of this item is now verified from Microsoft's own pages, and it removes a route
   rather than adding one:

   * **Win32 with the Game Development Kit is not available to us.** Not a blog claim any more: the
     public GDK carries the common parts only, and console work needs the Xbox Extensions, "only
     available under confidential license within an NDA Xbox program". Microsoft's own access page
     requires an organisation onboarded as an Xbox development partner with a GDK Agreement, and
     states that personal Microsoft accounts are no longer supported for that access. One person
     with a console cannot take this route at any price. The GDK itself is in good health - the
     October 2026 release is current - which makes this a licence wall, not a dead end.
   * **UWP with C++/WinRT is the one route left.** It is supported and frozen: "Although still
     supported, UWP is not under active development." Its Xbox doc set is archived. It still
     publishes apps to Xbox, and apps, unlike games, are still accepted. Building on a frozen
     platform is a real cost and it is the only door that opens.
   * **Leaving Xbox out** is no longer the third candidate it was, because a developer account now
     costs nothing and Developer Mode runs on a retail console.

   Decision: **UWP, C++/WinRT, x64, Developer Mode on a retail console.** Rejected: Win32 + GDK, for
   the licence wall above; WebView2, by the owner's rule of 2026-11-11 and because its availability
   on Xbox is still unverified; waiting, because the two pieces that need no console are now built.
   This decision is the lead's to confirm - it is recorded here as the only route the evidence
   leaves standing, not as permission to spend money.
2. **Where the remote surface lives.** Surfaces split by input grammar (owner, 2026-11-11), and
   there are already three remote surfaces: `apps/tv/page/surfaces` (stopped), `apps/android/surfaces/tv`
   and `apps/apple/Sources/SurfacesTV`. Xbox would be the fourth copy of one grammar. Unblocked by
   the owner saying whether a remote surface is one code base or one per family.
3. **Which `DeviceClass` an Xbox reports**, `box` or `tv`. It changes the per class floors the core
   picks (`platform/core/src/config.rs:81`). `host/smoke_test.cpp` uses `box` so the test has a
   value; it is not a decision.
4. **The safe margin.** Netflix keeps 68 px of 1920, which is 3.5%. Microsoft says keep essential
   UI 48 effective pixels from the left and right and 27 from the top and bottom. Those two are in
   different units, and converting between them needs item 1 answered: at XAML's 200% scale
   Microsoft's 48 and 27 are 96 px and 54 px at 1920, and Netflix's 68 px is 34 effective pixels, so
   following the measurement would put our row titles inside the area Microsoft calls unsafe. On
   Win32 with the GDK there is no such scale and the comparison has to be made again. Unblocked by
   item 1, then by the owner choosing, or by one look at a real television.
5. **Gamepad against remote in the contract.** `InputKind` names both
   (`platform/contract/src/client.rs:42`) and the owner's rule sends both to the remote surface.
   Whether the contract should also carry which surface was chosen is for the contract's owner.
6. **How long a held direction waits, and how fast it then repeats.** `Repeat::Hold` is 500 ms and
   `Repeat::Gap` is 120 ms in `surfaces/remote/remote.h`, and both are choices, not measurements.
   Nothing in this tree measures them and Microsoft documents no figure, because on a television
   remote the system owns the repeat and on a gamepad nobody does. Unblocked by one measurement
   against the reference on a television, or by a console. Until then the numbers are wrong by an
   unknown amount and a viewer crossing a long row will feel it.
7. **Whether `std`'s four Windows imports are reachable inside an app container.** The library names
   `ucrt`, `api-ms-win-core-synch-l1-2-0`, `bcryptprimitives` and `kernel32`. Unblocked by the
   archived API Sets list for UWP apps read against those four, which can be done from here, and
   settled for certain by one link on a Windows machine. This is the cheapest open item left and the
   next one worth doing.
8. **A C++ emitter in `platform/contract/gen`.** The one thing between this folder and a screen, and
   it needs no Xbox and no Windows. Unblocked by the contract's owner adding `cpp.rs` beside
   `kotlin.rs` and `swift.rs`. Do not hand-write the types here instead.

Licensed under either of MIT or Apache 2.0, at your option.
