# Decisions for the desktop family

What was decided while this foundation was laid, why, and what each one rejected. Dates are
absolute. Status words: **verified** (checked on real output), **assumed** (inferred), **open**
(needs a decision or work), **superseded**.

## 2026-11-11

### 1. The core is linked as native Rust, not Wasm and not through the C bridge

Decided by the owner on 2026-11-11: the core is native code on every platform that takes native
code, and WebAssembly is allowed only where the platform's one open runtime is a web engine, which
is Tizen, webOS, Vizio and the web app. A desktop takes native code.

Within that, the C bridge is not used either. `platform/bridges/c` exists so a host written in
another language can carry UTF-8 JSON across six functions. This host is Rust, so the bridge would
only turn typed messages into bytes and back for no reader, and it would hide a panic behind a
status where Rust already unwinds. The crate depends on `offline` and steps `offline::Machine`,
which is the same thing `bridge-c` holds.

Rejected: `bridge-c` with `--profile native`. It stays the right answer for the C box host, which is
a separate program and not this crate.

Supersedes, for where the core runs: `notes/151-architecture-2026-11-05.md` lines 435 to 437 put the
Mac, Windows and Linux core in the page's Wasm with the shell adding a player bridge only. The
owner's decision of 2026-11-11 moves the core out of the page on these three. The rest of those
lines, the shell in a system web view, still stands and is decision 3 below.

Status: verified. `cargo test` in this folder builds and runs 29 tests against the native core.

### 2. This family writes no surface. The pointer screens are `apps/web`'s

Surfaces split by input grammar, not by platform or screen size: pointer, touch, remote, decided by
the owner on 2026-11-11. A desktop and a browser on a desktop are the same grammar. `apps/web` has
already drawn that grammar's screens: 2,068 lines of Rust in 15 files, of which
`apps/web/src/surfaces/` is 1,201, a door, the pages of rows (home, films, series, a genre, my list
and search), a title page and a watch page. Counted at this commit with `git ls-files
'apps/web/src/**/*.rs' 'apps/web/src/*.rs' | xargs wc -l`; an earlier draft of this file gave 1,744,
which reproduces at no revision in this tree and is withdrawn.

It also already carries the measurements a pointer surface needs. `apps/web/src/surfaces/browse.rs`
appends rows at two viewports of remaining scroll and names the web reference as its source, and it
draws the nav item the viewer is on at weight 500 and the rest at 400. The rest of the web figures
sit next to those: a web home is 48 rows of 13 cards rendered, 20 maximum per row, 6 visible, tile
gap 8, row gap 32 (`notes/netflix-reference/web/`). A second pointer surface would re-derive every
one of them and drift, and it would put one rule in two places, which `CONTRIBUTING.md` rule 3
forbids.

So this crate adds no screen, no control, no animation and no setting. It performs effects and hands
a surface one `Turn`.

Status: verified by reading `apps/web/src/surfaces/`.

### 3. The shell is a system web view over this host, and the choice of binding is not made yet

The position: on Mac, Windows and Linux the window is the platform's own web view, drawing
`apps/web`, with this crate as the native host in the same process and the film played by a native
driver behind the view rather than by the view's video element. That is what decision 2 leaves, and
it is what `notes/151-architecture-2026-11-05.md` line 548 already planned.

What it costs, stated rather than left to be found later:

* Flathub's policy refuses thin web wrappers and bars AI assisted manifests
  (`notes/170-platform-sweep-2026-11-06.md` line 84), so the Linux route may have to be our own
  repository or a Snap. Status: verified from that note, which read the policy on 2026-11-06.
* `CONTRIBUTING.md` rule 4 says screens hold no rules. A web view that is handed its own bridge is
  the easiest place in this project to break that rule, so the shell gets the test note 151 calls
  the rule 4 test before it ships. Status: open.

No web view dependency is added in this crate today. `wry`, `tauri` and a hand written binding are
three different licence and review costs, and the honest small foundation is one that compiles and
is tested without any of them. Status: open, and it does not block this crate.

This leaves note 151 in a second place, which is written here so the owner was told about both
departures and not one. Its rows name the binding: `Tauri 2` on the Mac row (line 435), WebView2
`in the same shell` on Windows, WebKitGTK and Flatpak on Linux. This decision leaves all three
open. Those rows already read `Stays only if a rule 4 test passes`, so the departure is small, and
no dependency was added either way.

Rejected: a native Rust toolkit for the window. It would be a second pointer surface, which
decision 2 just refused, and no toolkit in Rust draws a Netflix grade artwork grid today without one.

### 4. Secrets sit in a file the owner alone can read, not in a keychain

macOS Keychain, the Windows Credential Manager and the Linux Secret Service are three different APIs
and would come in behind another dependency. The core asks one thing of this port: that a write
either lasts or says it was refused. A file under the folder the shell names does that.

On unix the file is created `0600` by `OpenOptions::mode`, not created and then chmodded: an earlier
draft of this crate wrote the bytes into a default-mode temporary and tightened it afterwards, which
left a session token readable by every account on the machine for the length of the write. A test on
the finished file cannot see that window, so there is now a test on the creation itself
(`src/adapters/mod.rs`) beside the one on the finished file (`tests/ports.rs`). Windows has no such
mode, so the file carries the rights of the folder it is in, which is weaker, and it is written here
so nobody assumes otherwise.

Status: open. A keychain is the better answer and the port does not change when it arrives: only
`FileSecrets` is replaced.

### 5. No film plays here, and nothing pretends one does

The driver for this family is libmpv, which is a system library and a surface that owns a window,
and this crate has neither. `NoPlayer` takes every command and does nothing with it.

The host answers the core nothing either, because there is nothing to answer. A player effect's id
comes from the core's own counter (`platform/core/src/lib.rs:496`) and is registered in no waiting
map, so `fn unsupported` (`lib.rs:824`) looks an id up in `secrets`, then in `net`, matches neither
and drops the event. An earlier draft of this file, and comments in `src/host.rs` and
`src/adapters/player.rs`, said the core hears a refused command and moves on because of it. That
causal chain does not exist and the claim is withdrawn. What the port returned was a `bool` no
other family's `Player` returns, and it has been removed with it.

What rescues the play is the core by itself, read out of `platform/core`: the play session holds its
start deadline
(`config::DEFAULT_START_DEADLINE_MS` is 15,000 ms until `/v1/hello` names one for the desktop
class), `player::tick` sees the deadline pass, fails over to the next attempt, and when the attempts
run out the play ends in `PlayState::Failed` with the position kept. That is the contract's own
degradation and nothing here pretends otherwise.

Status: verified by reading `platform/core/src/player.rs` (the deadline armed at line 410, fired at
line 548), `config.rs` and `lib.rs`. Not covered by a test in
this crate: proving it end to end needs a scripted play answer and scripted time, which is what
`platform/kit/core` is for. This crate's tests prove the wiring, not the core's rules.

### 6. No capability card is sent

`CapabilityCard` has to state the screen in pixels and what the player opens. This host has neither
a window nor a player, and a card full of zeros would make the server filter rows and attempts on
figures nobody measured. `card` is optional in the contract, so none is sent.

Measured at this commit: `rg -l CapabilityCard apps/` names this file and nothing else, so no
family builds a card today. The pointer input kind this surface would carry is therefore recorded
here and not in code that no reader has. An earlier draft grepped `InputKind` instead, which was
never evidence about cards and does not return nothing either.

Status: open. The card is the shell's to fill once a window can say how big it is.

### 7. A key becomes a file name as hex

Store keys hold a slash: the offline layer's queue writes `q/{seq:016x}`
(`KEY_PREFIX`, `platform/offline/src/queue.rs:25`, formatted at line 48) and its kept bodies `c/...`
(`BODY_PREFIX`, `platform/offline/src/cache.rs:16`). A secret is named by the core. Neither can be a file name as it stands, and escaping only
some characters makes two keys collide on a case insensitive file system. Every key is written as
hex, and a file this host did not write can never be read back as a key.

Status: verified. Both directions are tested, including a key with a slash, a space and Arabic.

### 8. Not a member of the root workspace yet

`./check` runs `cargo fmt`, `cargo clippy` and `cargo test` over the workspace, and this crate is
not in it, so a green `./check` says nothing about this folder. It is its own workspace root instead,
so `cargo` in this folder works and the root workspace is untouched. That is a holding position, not
a decision: the root `Cargo.toml` belongs to whoever owns it.

Joining is three changes, and the third is the one that bites. Add `"apps/desktop"` to `members`
in the root `Cargo.toml`; in this folder's `Cargo.toml` delete the `[workspace]` table and replace
the two `[lints]` tables with `[lints] workspace = true`; and regenerate and commit the root
`Cargo.lock`, because `./check` runs `cargo clippy --workspace --all-targets --locked` and
`cargo test --workspace --locked`, and a new member adds a package entry the old lock does not
have. Without that third change both commands fail, and they fail on CI rather than on the machine
making the edit.

Status: superseded by decision 9 on 2026-11-12, which did it. The paragraphs above describe the
tree before that date: this folder is no longer a workspace of its own and `./check` no longer
skips it.

## 2026-11-12

### 9. The crate is a member of the root workspace, and `./check` covers it

Decision 8 is done, by the three changes it named and nothing else. `"apps/desktop"` is in
`members` in the root `Cargo.toml`; this folder's manifest inherits `version`, `edition`,
`license`, `publish` and `[lints]` from the workspace, and takes `contract`, `rand`, `reqwest` and
`tokio` from `[workspace.dependencies]`; this folder's own `Cargo.lock` is deleted, with the
`.gitignore` line that hid it, and the root `Cargo.lock` gained one package entry, `desktop`, with
no dependency version moved.

Two of decision 8's premises were wrong by the time they were acted on, and both are recorded
rather than quietly fixed. The root workspace declares `edition = "2024"` and this crate already
said `edition = "2024"`, so joining is not an edition change and no lint arrived with a new
edition. And the two `[lints]` tables this crate declared were already the workspace's two, byte
for byte, so `[lints] workspace = true` changed no lint level at all. What had in fact never run
over these files was `-D warnings` together with `--all-targets`, which is what `./check` runs, and
it found nothing: four clippy units, the library, the library's own tests, `tests/host.rs` and
`tests/ports.rs`, with no warning and so no `#[expect]` anywhere in the crate.

Rejected: keeping this crate's narrower `reqwest` and `tokio` feature lists. The workspace's
`reqwest` carries `json`, which this host does not use, and its `tokio` is `full` against the four
features this host named. Both are supersets of what is needed, both are already built that way for
the server crates in the same invocation, so neither costs a byte here, and the convention in
`launcher`, `finder/service`, `platform/base/net-guard` and `tests/wired` is to inherit.
`serde_json` is still written out, because the workspace declaration is `default-features = false`
with `alloc` alone for the no_std core and this host is std, which is what every std member in this
repository does.

Status: verified. 29 tests run under `cargo test --workspace --locked`: 11 in the library, 7 in
`tests/host.rs`, 11 in `tests/ports.rs`. Two of the 29 are `#[cfg(unix)]`, so a machine that is not
unix runs 27, and nobody should read that as a regression.

## Open items

| What | What unblocks it |
|---|---|
| The root README still calls this folder empty | the owner of that shared status row replaces it |
| No window, so no screen and no capability card | decision 3's web view binding, then the card |
| No film plays | libmpv behind the window, once there is a window |
| Secrets are a file, not a keychain | a keychain dependency the project accepts |
| The connection is never watched | an OS call per platform, so the core hears the network return |
| The player port has no way back | `EventBody::Driver` has no home in this port, so a driver could not report what it did. Decided when a driver lands: either `listen`, as Android has, or a shell that hands driver events to `Host::step` beside the port |
| `check_rings.py` has no `desktop` family | a surface to scan. With none, an entry fails red on the spot: `check()` requires both an `adapters/` and a `surfaces/` folder under the family's code path (`platform/tools/check_rings.py:101-110`) and this crate has no `surfaces/`. An earlier draft of this table said it would scan zero and pass green, which is not what that code does |
