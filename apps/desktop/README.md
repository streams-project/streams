# desktop

Desktop shell for Mac, Windows and Linux, and a small C host for boxes.

Part of an open video platform. Status: a host and four ports, no screen. 1,158 lines of Rust in 8
files, and 539 lines of tests in 2 files, 29 tests in all, counted with `find src -name '*.rs' |
xargs wc -l` and `cargo test` in this folder. The core is linked as native Rust here, not Wasm and
not through the C bridge. `./check` does not reach this crate yet, because it is not a member of the root workspace:
build and test it from this folder with `cargo test`, which is how those figures were taken. The
root README's status table still groups this folder with the ones holding a README and nothing
else; that line is now wrong, and the row is shared, so it is not changed here.

This family has no surface of its own. It is a pointer surface, and the pointer screens are
`apps/web`'s; `DECISIONS.md` records why, what the shell around this host will be, and what is still
open. Three of the four ports are filled with what a desktop has, and the fourth takes commands
nothing performs:

| Port | What fills it here |
|---|---|
| network | reqwest over rustls, on the host's own runtime. The deadline covers reading the body |
| secrets | one file per secret under the folder the shell names, `0600` on unix. Not a keychain |
| store | one file per key, for the offline layer's waiting commands and kept pages |
| player | nothing. Commands are dropped, and the core fails over at the start deadline it armed |

The fifth port every other family names, `reel`, is not named in this crate. A reel draws into a
picture a surface's own toolkit owns, and there is no surface and no toolkit here, so the port
arrives with the shell that has one. `src/ports.rs` says that, and says where each of the remaining
shapes departs from the Android ones.

A shell opens it with the server address, a folder to write under and a byte cap, steps it, and
reads one `Turn`: the views, the page bodies, the replies to its own requests, the notices, the
targets the core opened and how long until it steps again. Nothing here holds a rule, a screen or a
setting.

Licensed under either of MIT or Apache 2.0, at your option.
