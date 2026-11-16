# The window screens, by folder

Apple Vision Pro. A window in space, drawn from the same ports and the same answers as the
television and the phone, and designed as though this were the only platform there is.

**This is not a ported television.** It shares the ports, the contract, the brand and the words.
It does not share the layout, the chrome, the colours, the input model or the player, and each of
those is a decision argued in the file that makes it rather than a thing left out.

| Folder | What is in it |
|---|---|
| app | The root (`Window`), the gaze-and-pinch model and the minimum target (`Reach`), the glass and the z axis (`Surface`), the window rule and the brand's corners (`Sizes`), the shared re-export (`Shared`) |
| browse | A page of rows: the navigation ornament, the hero card, the rows (`Browse`, `Rows`) |
| title | One title: its artwork, facts, buttons, reactions and what else is like it (`TitlePage`) |
| watch | The player and its presses (`Watch`) |
| parts | Small pieces the screens share: the facts line, the badges on a tile |

Words and numbers: a language's words come from `strings/{lang}.json`, its row names from
`strings/rows/{lang}.json`; direction, digits and text size from that file's `language.*` keys. The
same as every other surface in this family.

## Why this is not the television surface

Not in its interaction, and **not in its content either**. What is shared is the ports, the
contract, the brand, the words and the catalogue the server sends — the things that make it the
same product. Everything about how that catalogue is laid out, lit, navigated, coloured and played
is decided here, for a window in a room, and the table below says which of the television's
measurements survive that (very few) and which describe a situation that does not occur (most).

Rows of tiles and a title page are not a television idea being borrowed; they are what a catalogue
of films looks like on any surface wide enough to show more than one at a time, and here they are
built from a window rule, a gaze-sized tile, a drawn title, an ornament and a z axis — none of
which the television has or could use.

The interaction then follows from one assumption rather than a list of differences.

The television decides where focus goes. `SurfacesTV/app/Remote.swift` turns an arrow key into a
`Word`, and every television screen keeps the answer as an integer of its own — a row, a column, a
line of a menu — paints an outline wherever that integer points, and keeps one box per screen whose
job is to tell a screen reader what the integer now means. The phone decides nothing, because
nothing is focused before a finger arrives.

Here the system decides and does not say. The viewer looks at something and the system lights it;
the app hears about a pinch and never about a direction, and never about where the eyes are. So
there is no `Word` in this module, no key map, and no integer anywhere meaning "where the viewer
is". `SurfacesShared`'s `says(_:on:)` and `saysItsWords(on:)` are never called here either: they
add the `selected` trait from the surface's own idea of focus, which is right on a television whose
outline is painted and would be an invented claim here. `Tests/VisionTests/Speech.swift` holds that.

What is left for the app is in `app/Reach.swift`: make every target big enough and far enough apart
to be looked at, give it a shape the system can light, and give it words.
`reachable(says:state:corner:)` does the four together so a control cannot end up with three.

## The television reference, line by line: what carries over and what is void

`reference/netflix-on-the-tv.md` measures a fixed 1920x1080 panel at a fixed viewing distance. Here
the viewer resizes the window and sits where they like, so most of that file is describing a
situation that does not arise. Rather than leave which is which to judgement at each call site:

| From the television reference | Here |
|---|---|
| 72 px page margin | **Void.** A length on a known raster. Replaced by a stepped margin in points (`Sizes.swift`). |
| 311 x 505 closed tile, 906 x 510 focused | **Void as lengths.** The tile is derived from how coarsely an eye can be aimed. |
| 328 px pitch (tile + gutter) | **The derivation carries, the number is void.** Pitch is still tile plus gutter; both are computed. |
| 2.91x focused-to-closed width | **Void, and replaced rather than dropped.** That expansion exists because a flat panel cannot bring a thing nearer. Here the z axis does it and costs no area (`Surface.swift`). |
| Focused row pinned to y 199–708 | **Void.** It only means something where the app moves focus. Nothing here knows what is focused, and a page that scrolled itself under a gaze it cannot see would move the thing being looked at. |
| 16:9 focused still / 0.616 portrait poster | **The 16:9 shape carries** — the web reference measures it independently. The portrait poster and the swap-on-focus do not: the swap is a focus effect, and there is no focus here. |
| Artwork swap, two assets per title per row | **Void.** It is the focus effect again. One asset, drawn at rest. |
| Top bar as a 96 px band | **Void.** Chrome goes outside the window in an ornament and costs the catalogue nothing (`Browse.swift`). |
| Dark panel colours, literal white ink | **Void, and actively wrong.** This draws on glass over a real room (`Surface.swift`). |
| Rank numeral in a 146 px widened gutter | **Void.** On a window that is an eighth of the row to show a number the tile already carries as a badge (`Badges.swift`). |
| Title's name revealed on focus | **Void, and inverted.** The name is drawn at rest, because anything shown on hover is shown only to whatever is being looked at (`Rows.swift`). |
| Facts order: genre, year, length, age | **Carries.** A content decision, not a measurement. |
| Row title to row title 797 px; vertical rhythm | **Void as numbers**, and the idea that rows have a rhythm carries. |

A number taken out of that file into this folder would be a false citation. None is.

## The one rule that is reused, and from where

`reference/netflix-on-the-web.md` measures the only container in either file that the viewer
resizes, and what it establishes is a **rule**: across nine widths the tiles fill the content box
exactly, with the page margin and the number across as the two step functions. That rule is reused
unchanged in `Sizes.swift`. Its measured thresholds, its 8 px gutter and its margins are not: they
were read off a browser page at arm's length with a mouse. What sets the steps here is
`Reach.tileFloor`, which comes from how coarsely an eye can be aimed.

The 16:9 tile shape is reused, because both reference files measure it independently.

**Everything else is this surface's own**, said so one by one in `Sizes.swift`, and none of it has
been seen on a device. `Reach.target` is Apple's published visionOS guidance — 60 pt, and 60 pt
between centres — and is marked in the file as borrowed rather than measured.

## What has never been run

No Vision Pro exists here, and no visionOS simulator either: on 2026-11-14 `xcrun simctl list
runtimes` answered iOS 26.5 and tvOS 26.5 and nothing else. The visionOS SDKs **are** installed, so
this module is compiled for visionOS, device and simulator, and the compile is the only thing about
visionOS that anything here checks.

`Tests/VisionTests` therefore runs on the iPhone simulator, and this module is written to compile
there for exactly that reason — a module behind `#if os(visionOS)` would be an empty module on the
one simulator that exists, and a test target that ran nothing would be a green run proving nothing.
What running there proves is the window rule's arithmetic, that every control carries words, and
how large each control is really laid out.

One thing it cannot read, which is not about visionOS and affects every family: **SwiftUI builds no
accessibility nodes at all for the contents of a `ScrollView` hosted in an offscreen window.**
Measured on the iOS simulator on 2026-11-14 — `heard(ScrollView { Text("inside") })` is the empty
list at every size from 390×800 to 1280×820 and at settles up to 2 s, while
`heard(VStack { Text("outside") })` returns the text at once. It is not the walker's doing: under
the hosting view the one element is the `HostingScrollView` itself, which answers nil to
`accessibilityElements` and 0 to `accessibilityElementCount()`, so there is nothing to find
whichever way a container is asked. So the **contents** of `Browse` and `TitlePage` cannot be read,
and every part they are built from is read on its own instead — the shape the television family's
speech tests already have.

Their **chrome** can be read, and that changed under the test rather than being designed for: moving
navigation into an ornament took it outside the scroller, so a page that used to reach a reader as
nothing at all now reaches it as its five navigation controls. Chrome that costs the catalogue
nothing turns out to cost the tests nothing either.
`testOnlyTheChromeOfAScrollingScreenCanBeRead` asserts both halves and fails the day the second
one changes, at which point the part-by-part reads can go back to being whole-screen reads.

## The decisions, and why

Each is argued at length in the file that makes it; this is the index.

| Decision | Where | Short reason |
|---|---|---|
| Depth replaces the focused-tile expansion | `app/Surface.swift` | A television expands because it has no z axis, and it pays a row for it. Bringing a thing nearer costs no area. The app sets each layer's **resting** depth only; the lift under a gaze is the system's own hover effect, drawn out of process, and cannot and must not be reimplemented. |
| Chrome goes in an ornament, anchored to the bottom | `browse/Browse.swift` | An ornament is placed outside the window's bounds, so navigation costs the catalogue nothing. Bottom, not top, because a window sits at eye height for the length of a film while hands rest low, and a pinch is made where the hand is comfortable. |
| The system's glass and semantic ink, not the brand's dark plate | `app/Surface.swift` | There is nothing to paint on: the room shows through. An opaque `#0B0B10` slab throws away the material the system adjusts to the real light, and literal white ink assumes a dark plate. `Accent` and `Join` stay; the slab and the white go. |
| A window that grows, plus an **offered** dimmed immersive space — not a volume, not immersive by default | `watch/Watch.swift` | A film is a rectangle: a volume is for bounded 3D content walked around, and gains a film nothing while capping how large it can be. Taking somebody's room away unasked is the wrong default for half-watching something. |
| No scrub-by-drag; every control is a discrete press on a standard `Button` | `watch/Watch.swift`, `app/Reach.swift` | Dwell Control cannot hold-and-move at all; Pointer Control aims with a head or wrist, where a held drag is a tremor away from the wrong seek. A standard `Button` is driven by VoiceOver, Dwell, Pointer and a keyboard without this module knowing they exist. |
| Nothing is revealed on hover; a tile draws its title's name at rest | `browse/Rows.swift` | On a surface whose pointer is the gaze, anything shown on hover is shown only to the one thing being looked at. Useless to somebody scanning, and a paid dwell to read a label. |
| No key map, no `Place`, no `Spot`, no focus index anywhere | `app/Reach.swift` | The system moves focus and never says where it is. Both television surfaces are built on the opposite assumption; none of that scaffolding is carried over, and `says(_:on:)` is never called here because the app would be announcing a focus it invented. |
| Minimum 60 pt target and 60 pt between centres, as a correctness property | `app/Reach.swift`, `Tests/VisionTests/Reach.swift` | Borrowed from Apple's published guidance, not measured here. Read off the real laid-out frames, with the count asserted first so an empty screen cannot pass. |
| The window has a floor and no ceiling, and no position | `app/Window.swift` | The point of the hardware is that the screen can be as large as the room allows. Where the window goes is the viewer's, and the system already remembers it. |

What the tests cannot prove, and what a device has to settle:

- whether 60 pt is in fact large enough to look at, and whether 60 pt between centres is enough
  separation for a gaze — both are borrowed numbers;
- whether the system's highlight lands on the shape each control hands it;
- whether a bar drawn inline is right where an ornament exists. An ornament is this platform's own
  idiom for that bar; it is placed by the system outside the window's bounds, so nothing here can
  lay one out, and choosing it would mean shipping a bar never laid out anywhere;
- whether leaving the window as the system's glass, rather than filling it with the brand's dark
  plate, keeps text readable against a real room;
- whether the hero card at 0.34 of the window's width leaves enough of a row visible on opening —
  both reference files measure a card that leaves none, which is the one thing about them this
  surface sets out not to repeat;
- whether a row that scrolls, with nothing pinned, reads as well as the television's fixed band;
- **the immersive space for playback, which is specified and not built.** An `ImmersiveSpace`
  cannot be laid out, drawn, entered or left by anything on this machine, so building one would
  mean shipping a screen that has never been drawn anywhere with no way to find out. `watch/Watch.swift`
  leaves the seam and argues the shape: offered rather than default, progressive rather than full;
- whether the resting depths in `Surface.swift` are felt at all, or are too small to notice and too
  large to ignore — they are unmeasured points and a device is the only place to find out;
- whether an ornament anchored to the bottom is in fact the comfortable end of the reach, which is
  reasoning rather than measurement.

## Blocked on strings

Ten-second step-back and step-forward presses belong on the player and are **not** built: the
assets `icon_back_10` and `icon_forward_10` exist with no words to call them, and `strings/` has no
key for either. Writing English in Swift is what `SurfacesShared/Says.swift` exists to prevent, and
an unlabelled icon control is the exact defect `Tests/VisionTests/Speech.swift` fails on. Two keys
would unblock it — a word for stepping back ten seconds and one for stepping forward.

## The bridge

`Package.swift` wires a `target/visionos` slice of `bridge_c` the way it wires tvOS and iOS, and
`build-core.sh` knows how to build it. **It has not been built on this machine and cannot be.**
This machine is x86_64; the visionOS simulator SDK's x86_64 slice would need a
`x86_64-apple-visionos-sim` Rust target, and Rust 1.98.1 has only `aarch64-apple-visionos` and
`aarch64-apple-visionos-sim`. The device slice is a cross-compile and would build here exactly as
tvOS does, with `-Z build-std`; the simulator slice needs an Apple Silicon machine. No test target
links the bridge, so none of this blocks the tests.
