# xbox

The Xbox app. A controller gives four directions and a select, so Xbox draws the same grammar as a
television: this is a **remote** surface. The core is native code here.

Every measured figure below comes from the Netflix reference folder in the project's notes, which
sits outside this repository; paths like `netflix-reference/tv/...` are inside that folder.

Status: no app, and the route to one is undecided. This folder holds a README, a `.gitignore` and
`host/`, which is the C++ side of the core boundary with a test that drives it. Nothing draws a
screen. [What is still open](#what-is-still-open) names the decision this folder waits on. The root
README's status table still groups this folder with the ones that hold a README and nothing else;
that line is now wrong and it is not owned here.

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

| Not verified, and what would settle it |
|---|
| **Whether a retail Xbox forbids dynamic code generation.** No Microsoft page read here says so for Xbox. The nearest documented fact is about .NET: runtime code generation fails under the .NET Native toolchain. Settled by the Xbox Requirements set that comes with Partner Center or ID@Xbox onboarding, or by a retail install of a package carrying a code generating runtime. It changes nothing here: the owner's rule already keeps WebAssembly off Xbox, so no engine of ours needs to generate code at run time. |
| **Whether a new app submission can select the Xbox device family without separate approval, and whether an individual account is accepted for a media app.** The FAQ says apps go through the Windows process "with additional reviews to meet today's Xbox One standards" and does not say what those reviews are. Settled by a Partner Center account and one submission attempt. |
| **The registration fee.** The pages read carry no figure, so none is quoted. Settled by the registration page at the time of registering. |
| **Whether WebView2 on Xbox is open to any UWP app.** The announcement says only "WebView2 is now available on Xbox too!" and names no programme or eligibility. Reported elsewhere as UWP only, with no GDK path. Moot for the core under the owner's rule, and it still decides whether a WebView2 shell is available at all. |
| **Whether `std` on a Windows target survives the app container.** The tier 3 `x86_64-uwp-windows-msvc` target exists because the app container forbids part of Win32, and tier 3 means nobody tests it. Settled by a Windows machine with Visual Studio and the Windows SDK, building `bridge-c` for both targets and running the certification kit over the package. |
| **Whether 1 GB holds the artwork.** Netflix pulled 2.17 MB over 15.2 s before its first screen settled and spent 288 kB of GPU texture per focus step on the profile gate, measured on a Google TV Streamer, not on an Xbox. Two assets per tile against a 1 GB ceiling is a number to measure, not to assume. Settled by a console. |

There is no console and no Microsoft developer account here, so nothing in the second table was
attempted.

## The core on Xbox

`host/` is the only code in this folder. It is the same boundary every native family crosses - six C
functions carrying UTF-8 JSON, described in `platform/bridges/README.md` - written once for C++:

| File | What it does |
|---|---|
| `host/core_host.h` | `streams::Core`: open, step, close, and the four statuses by name |
| `host/core_host.cpp` | the boundary. Out bytes are copied once and handed straight back to `core_free` |
| `host/smoke_test.cpp` | 11 checks that drive a real core through that class. Not run by CI |

It holds no rules, exactly as `apps/android/core/src/main/cpp/bridge.c` holds none for Java.

Ran on this machine, 2026-11-11, x86_64 macOS, rustc 1.98.1:

    cargo build --locked --profile native -p bridge-c --target x86_64-apple-darwin
    c++ -std=c++17 -Wall -Wextra -I platform/bridges/c apps/xbox/host/core_host.cpp \
        apps/xbox/host/smoke_test.cpp target/x86_64-apple-darwin/native/libbridge_c.a -o smoke
    ./smoke

11 of 11 held, no compiler warnings. That proves the C++ side of the boundary and the config shape a
console sends (`"class":"box"`, `"form":"tv"`), and it proves nothing about Xbox.

**CI never runs it.** The root `./check` is the whole of CI and it carries no C++ compiler and no C++
test runner, and `platform/tools/check_rings.py` has no `xbox` family, so this test runs only when
someone types the two commands above by hand. It will go stale silently the first time the C ABI or
the core's config shape moves. Registering a compile-and-run in `./check`, behind a "no C++
toolchain, not run" line like `check_types.py` already prints for a missing compiler, belongs to
whoever owns that file; it is not done here.

The equivalent of the build above on a console would be

    cargo build --locked --profile native -p bridge-c --target x86_64-pc-windows-msvc

which needs a Windows machine with the MSVC toolchain, or `x86_64-uwp-windows-msvc` with
`-Z build-std`, the way `apps/apple/build-core.sh` already does for tvOS. Neither was run.

## Folder shape

| Folder | State |
|---|---|
| `host/` | written. The core boundary and its test |
| `ports/` | not written. The five ports every family names: network, secrets, store, player, reel |
| `adapters/` | not written. What each port does on Xbox, through WinRT |
| `surfaces/remote/` | not written. The screens, in the remote grammar |

The empty three are listed and absent on purpose: a folder here would claim work that the decision
below has not released. `platform/tools/check_rings.py` has no `xbox` family, so nothing yet holds
this folder to its ports; adding one belongs to `platform/tools`.

## What is still open

1. **Which native route Xbox takes.** This is the decision the owner now owes, because Xbox was
   defined in terms of a surface that no longer exists. The three candidates this reading found:
   a UWP app in C++/WinRT, which is the only documented route to the Store for a thing that is not a
   game, on a doc set Microsoft has archived; Win32 with the Game Development Kit, which is reported
   to be for managed partners and to carry no WebView2, on a developer blog and a GitHub discussion
   rather than a Microsoft page, so treat that one as unverified; or leaving Xbox out until there is
   a console and a Partner Center account. Nothing else in the tree waits on this answer, and
   everything in this folder does.
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

Licensed under either of MIT or Apache 2.0, at your option.
