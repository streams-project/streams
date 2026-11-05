# server

The one server of an open video platform: front door, accounts, library, rules, titles, pages,
search, playback answers, subtitles and marks.

It is one binary and a set of library crates, so a route test runs against the real router and a
crate edge stops what a module could not.

| Crate | Job |
|---|---|
| `server-accounts` | door, sessions, profiles, sign in, device sign in, mail |
| `server-catalog` | titles, vectors, index, the data manifest and its swap |
| `server-rules` | the one eligibility function, the proof types, safe answers |
| `server-library` | commands, changes, list, rating, position, taste |
| `server-pages` | pages, rows, hero, search, title pages, cursors |
| `server-play` | the play gate, one ranking, subtitles, skip marks |
| `server-api` | settings, the request guard, the router, jobs |
| `server` | the binary: `serve`, `migrate`, `index build`, `check pages` |

## Run

    SERVER_LISTEN=127.0.0.1:0 DATA_MANIFEST=/path/to/manifest.json server

Every setting is optional. A missing one turns its feature off, the server logs that at start, and
the feature's routes answer `unsupported` with a notice. `SERVER_LISTEN` alone stops a start, because
no address is built in. The full list is in `crates/server-api/src/config.rs`.

## Check

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

Tests name the ledger ids they cover. Tests marked `#[ignore]` need a real Postgres in
`TEST_DATABASE_URL`, with the chain in `migrations/` applied.

Licensed under either of MIT or Apache 2.0, at your option.
