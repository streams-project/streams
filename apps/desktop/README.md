# desktop

Desktop shell for Mac, Windows and Linux, and a small C host for boxes.

Part of an open video platform. Status: a host and its five ports, no screen. 1,123 lines of Rust in
9 files, and 539 lines of tests in 2 files, 28 tests in all. The core is linked as native Rust here,
not Wasm and not through the C bridge. `./check` does not reach this crate yet, because it is not a
member of the root workspace: build and test it from this folder with `cargo test`, which is how
those figures were taken. The root README's status table carries this line for every platform.

This family has no surface of its own. It is a pointer surface, and the pointer screens are
`apps/web`'s; `DECISIONS.md` records why, what the shell around this host will be, and what is still
open. Three ports are filled with what a desktop has, and two answer plainly that they cannot:

| Port | What fills it here |
|---|---|
| network | reqwest over rustls, on the host's own runtime. The deadline covers reading the body |
| secrets | one file per secret under the folder the shell names, `0600` on unix. Not a keychain |
| store | one file per key, for the offline layer's waiting commands and kept pages |
| player | nothing. Every command is refused, so the core fails over at its start deadline |
| reel | nothing. A brand film needs a picture, and a picture needs a surface |

A shell opens it with the server address, a folder to write under and a byte cap, steps it, and
reads one `Turn`: the views, the page bodies, the replies to its own requests, the notices, the
targets the core opened and when to step again. Nothing here holds a rule, a screen or a setting.

Licensed under either of MIT or Apache 2.0, at your option.
