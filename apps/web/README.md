# web

The website, written in Rust and compiled to wasm with Leptos, for browsers on a phone, a tablet
and a desktop. The core runs in this process rather than over a bridge, so there is no host
language between a decision and the screen.

Part of an open video platform. Status: runs. It is a crate of the workspace, so `./check` builds,
lints and tests it on every push, and `cargo build -p web --target wasm32-unknown-unknown` is the
line that proves it still fits a browser. Its size is whatever these print, rather than a number in
this file that nothing measures (`platform/tools/check_claims.py` says why):

    git ls-files apps/web | wc -l
    git ls-files 'apps/web/*.rs' | xargs wc -l

The screens are in `src/surfaces`, each reaching the world only through `src/ports.rs`;
`platform/tools/check_rings.py` holds that true. Every word on every screen arrives from the server
at runtime (`/v1/strings/{lang}`) in the language the viewer reads, so there is no English in this
crate's source: `src/surfaces/words.rs` carries the four rules between a key and a line of text and
the test beside it reads this crate's own source back against `strings/en.json`.

Licensed under either of MIT or Apache 2.0, at your option.
