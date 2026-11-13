# desktop

Desktop shell for Mac, Windows and Linux, and a small C host for boxes.

Part of an open video platform. Status: a host, four ports, and the four screens that come before a
film, laid out but not yet painted. 4,652 lines of Rust in 14 files, of which `src/surfaces` is
3,485 in 6 files, and 650 lines of tests in 2 files beside them; 76 tests in all, counted with
`find src -name '*.rs' | xargs wc -l` and `cargo test -p desktop` from the repository root. The core
is linked as native Rust here, not Wasm and not through the C bridge. `./check` reaches this crate:
it is a member of the root workspace, so `cargo clippy --workspace --all-targets --locked --
-D warnings` and `cargo test --workspace --locked` both cover it, which is how those figures were
taken. Two of the 76 are `#[cfg(unix)]`, a file mode and a secret's rights, so a machine that is not
unix runs 74. The root README's status table still groups this folder with the ones holding a README
and nothing else; that line is now wrong, and the row is shared, so it is not changed here.

Three of the four ports are filled with what a desktop has, and the fourth takes commands nothing
performs:

| Port | What fills it here |
|---|---|
| network | reqwest over rustls, on the host's own runtime. The deadline covers reading the body |
| secrets | one file per secret under the folder the shell names, `0600` on unix. Not a keychain |
| store | one file per key, for the offline layer's waiting commands and kept pages |
| player | nothing. Commands are dropped, and the core fails over at the start deadline it armed |

## The screens, and the window that is not here

The owner decided on 2026-11-13 that the desktop is its own app and not a web page in a frame, which
supersedes `DECISIONS.md` decision 2. `src/surfaces` holds the screens before playback:

| Screen | What it is |
|---|---|
| address | a box for a server and a way on. This family's own: a browser is already at a server and a television is handed one, and a desktop app is opened from a dock with nothing |
| door | the way in, and a look round where the server allows visitors |
| sign in | an email, a password that is never drawn, and what was typed |
| gate | who is watching, on the page's own grid, with a PIN where a profile has one |
| browse | the bar, the hero and the rows, paging sideways and scrolling down |

**No toolkit is linked, and no dependency was added for any of it.** A screen lays out into a
`Scene`: rectangles with corners, colours by their brand role, words, pictures and the caret. That
is the whole vocabulary a painter has to implement, and it is deliberately the part that needs no
window system - floats over measured numbers, built and checked on whatever machine builds the tree.
The model is `apps/xbox/surfaces/remote/paint.h`, which does the same for a surface whose toolkit is
not on this machine either. Which painter draws a scene is the owner's to decide and is the one
thing in the way of a window; the screens do not change when it is decided.

Three things hold the screens to the rest of the project rather than to anybody's taste:

* **Every measurement is `reference/netflix-on-the-web.md`'s.** `surfaces/sizes.rs` reads that file
  in its own tests and holds every function in it to all 28 widths the reference measured, from 160
  to 1920. The one rule is the reference's:
  `tile = (viewport - 2 x margin - (across - 1) x 8) / across`.
* **Every word is `strings/`.** The English file travels in the binary, because the address screen
  is drawn before any server exists and no server can have sent its words; `/v1/strings/{lang}`
  replaces all of them the moment one answers. No English is written into this crate.
* **Every colour, corner and face is a role resolved against `brand.json`.** No hex colour and no
  radius is written in a screen, and a test holds every role a part can name to a key that file
  really has.

The fifth port every other family names, `reel`, is still not named here. A reel draws into a
picture a surface's own toolkit owns, and there is a surface now but no toolkit, so the port arrives
with the painter. `src/ports.rs` says where each of the remaining shapes departs from the Android
ones.

A shell opens the host with the server address the first screen collected, a folder to write under
and a byte cap, steps it, and reads one `Turn`: the views, the page bodies, the replies to its own
requests, the notices, the targets the core opened and how long until it steps again. It hands the
`Turn` to the surface, draws the scene it gets back, and hands presses, typing and the wheel the
other way. Nothing here holds a rule or a setting, and a screen reaches no port.

Licensed under either of MIT or Apache 2.0, at your option.
