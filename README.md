# Open video platform

One server, one small core, and a real app for each device. Run it for yourself, or change one folder
and make it yours.

The server, core, data builder, finder and website run together. Each device is at a different point, so there is one label per platform under [Status](#status) with the evidence for it. The first run on a clean machine is still being tested, so tell us if `./up` trips.

## Run it

    ./up

That is the whole setup on macOS or Linux. It needs only Rust ([rustup.rs](https://rustup.rs)). It builds
everything, fetches a pinned database, and starts the server, the finder and the website at
`http://127.0.0.1:8080`. Add `--public` to serve other devices on your network, `--port N` for another port.
A TV signs in with a code that a phone opens, so `--public` also gives the server this machine's network
address and prints it. Behind a tunnel or a proxy, name the address with `--site-address https://...`.

Everything it writes stays in `~/.<command>`. After the first run the program is `bin/<command>`:

    bin/<command> doctor          check this install
    bin/<command> backup          save the database while the service is stopped
    bin/<command> build web       rebuild the website

Until you build a catalogue with `data`, the site shows a generated sample with invented titles.

## Make it yours

`brand.json` holds the name, the command, the domain, the colours, the fonts, the button roundness and the
spacing. Change that one file and run `./up` again. Put your own marks in a folder under `brand/` and point
`assets` at it.

Video comes from finders. `finder/open-films` plays films you list with their licence in
`~/.<command>/films.json`. A source pack is a separate program in `packs/<name>/`, see `finder/README.md`.

## What is here

| Folder | What it is |
|---|---|
| `platform` | The contract, the core, the offline layer, the bridges and the test kit |
| `server` | The one server every app talks to |
| `finder` | The service that finds video, and a source that plays open films from a list |
| `data` | Builds and proves the catalogue |
| `apps` | One app per device |
| `brand` | Marks. Add your own folder for your own product |
| `launcher` | The one command |
| `tests` | The parts running together for real |

Run every check with `./check`.

## Status

One label per platform and what the label rests on, so nobody has to guess from the folder. No
counts: a number in prose is a promise nothing enforces, and three of the four here were already
wrong a day after they were written. What each row rests on is a check anybody can run.

| Platform | Status | What that rests on |
|---|---|---|
| Server, core, contract, offline layer, bridges, launcher, finder | Runs | `./check` builds, lints and tests every crate on every push |
| Data builder (`data`) | Runs | its own checks run inside `./check`, and they cross check the contract and the server |
| Website (`apps/web`) | Runs | Rust behind the family's ports. It is a workspace crate, so `./check` builds, lints and tests it |
| Android (`apps/android`) | Runs on real devices, built by hand | Screens behind the family's ports: the television's `browse`, `parts`, `notifications` and `reminder` are Java and native Views, the rest is Compose in Kotlin. `./check` runs its unit tests through `platform/tools/check_android.py` where a JDK and an Android SDK are present, and prints what it did not run, by name, where they are not |
| Apple (`apps/apple`) | Compiles and tests, never run on a device | A phone surface and a television surface in Swift behind the family's ports. `./check` runs their tests through `platform/tools/check_apple.py` on the iOS and tvOS simulators where Xcode is present, and names what it did not run where it is not. A visionOS surface was started on 2026-11-14 |
| Desktop (`apps/desktop`) | Lays screens out, paints nothing | The screens before a film, in Rust, laid out into a `Scene` of rectangles, words and brand roles. No painter has been written and which one to use is undecided, so there is no window yet |
| Harmony (`apps/harmony`) | Read, compiled nowhere | A touch surface in ArkTS behind the family's ports, with a native bridge. No OpenHarmony SDK exists on any machine here, so not one line of it has been compiled, type-checked or run; it is held by being read against the Kotlin it was ported from |
| Xbox (`apps/xbox`) | Read, compiled nowhere | Key maps and drawing geometry for the television screens in toolkit-free C++, plus a native bridge. Everything that paints needs a Windows machine, and `check_rings.py` has no entry for it because nothing fills a port here yet |
| Roku (`apps/roku`) | Runs under no compiler | Screens, a host, ports and adapters in TypeScript. The whole path from a key press to an HTTP request and back onto a screen exists and its tests run under JavaScriptCore with a type eraser. No `tsc` exists on any machine here, so nothing in it has ever been type-checked |

What every row has in common, with one exception named: `./check` holds the family to its ports
with `platform/tools/check_rings.py`, whether or not it can build that family. The exception is
Xbox, which has no entry because nothing there fills a port yet, and the check refuses a family
whose rings do not exist rather than passing it vacuously. That sentence was false for Roku too
until 2026-11-14 -- its rings are one file each and the check demanded a folder, so the family it
claimed to hold was held by nothing.

The code is under MIT or Apache 2.0, at your option. The Streams name and marks are not, see
`TRADEMARKS.md`.
