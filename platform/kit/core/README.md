# kit/core

Proof that the core works, and that the offline layer changes nothing the core decides.

## What runs

    cargo test -p kit-core

- `tests/sessions.rs` runs every file in `sessions/` with the layer on and off, through the six
  exported functions of `bridges/c`.
- `tests/boundary.rs` runs every session again through the C host in `templates/terminal`, and
  fails on one differing byte. It also holds the linked core.wasm to six functions of i32, no
  import, no feature past mutable globals and a size ceiling.
- `tests/held_bytes.rs` counts every byte after a closed core. `tests/heap_growth.rs` repeats the
  longest session on the Wasm allocator until its memory stops growing.
- `tests/pure.rs` is the gate of the pure core. It fails when `core` or `offline` gains std, async,
  a clock, IO, a thread, unsafe code, a global, an address or a dependency past serde and
  serde_json, when the core names the layer, or when compiled code uses a WebAssembly feature past
  mutable globals. It also builds a copy of the core with a planted `extern crate std` and checks
  that the wasm32v1-none build refuses it, while a clean copy passes.

- `tests/coverage.rs` holds every ledger row homed in the core or the layer to a test or session
  that names it, or to a line in `open.csv` that says what is missing and what unblocks it. A row
  marked `no` there may have no test claiming it, and a row marked `part` must have one.

## A session

Plain JSON with scripted time, tied to ledger ids by `covers`. `common.json` holds the config, the
fake server's usual answers and the prologues a session can start from.

- `device`: `secret`, `store` and `storage_bytes`. A missing store or secret store answers every
  effect `unsupported`.
- `server`: answers, tried before the common ones, by method and path. `times` limits a rule and
  `from_ms` delays it. A request with no rule fails the run, except `POST /v1/commands`, which
  applies every op.
- `moments`: scripted time, `events` from the host, `net` up or down, `restart` (false restarts
  the app, true also reboots, so monotonic time starts again), `forget_secrets`.
- `want`, `want_on`, `want_off`: what must hold after a moment, in both modes, with the layer on,
  or with it off. Lists hold exactly what happened since the last moment. `view` is compared by
  part, and `null` means absent.

Between moments the host steps the core at every `next_wake_ms`, as a real host does.

## The same bytes on every host

A run writes a transcript: one `open <config>` line for each open, and an `in <message>` and
`out <message>` line for each step. Every host must write the same bytes. Native is the kit's own
host. The other two are hosts in other languages that hold the clock and the one wake, perform
effects and answer them, so a wrong wake or a wrong store answer changes the bytes:

- C: `templates/terminal/terminal.c`. `cargo test -p kit-core --test boundary` runs it.
- A browser, from a local file with no server. The test is ignored by default because it needs two
  paths from the environment:

      KIT_BROWSER=<browser binary> TSC=<TypeScript compiler> \
        cargo test -p kit-core --test boundary -- --ignored --nocapture

  It compiles `bridges/ts/core.ts` and `browser/player.ts`, puts them with core.wasm and every tape
  into one page, and reads the result from the page with `--dump-dom`. It also runs the loader's own
  promises against the real module: a refused message, a trap, a load after a trap, and a repeated
  session that stops growing memory. Transcripts land in `target/kit-cross/`, one file per session,
  layer word and host.

## A tape

What another host needs to run a session, so the fake server stays in one language. `cargo run -p
kit-core --bin tape -- <session name> <on|off> <folder>` writes one with its transcript. Lines:

    contract <date>            the contract date of every in message
    wall <ms>                  wall time when the scripted clock reads 0
    device <store> <secret>    1 or 0: without one, the host answers those effects unsupported
    open <boot> <config>       close any open core and open a new one; boot 1 restarts monotonic time
    at <ms>                    run every wake up to this time, then set the clock here
    forget                     empty the secret store
    step <events>              step with these events until the core asks for nothing more
    http\t<id>\t<method>\t<path>\t<event>

Http lines are read in order, apart from the other lines. Each is the answer to one http effect,
with its id, method and path as JSON literals, and a host stops if they do not fit what the core
asked. Store, secret and timer effects are the host's own work, and the tape does not hold them.

## Measured

2026-11-06, with rustc 1.98.1 on x86_64 macOS, headless browser engine 158, 89 sessions with the
layer on and off:

- 178 of 178 runs byte identical on native, through the C host and in the browser.
- 4,876 steps through the browser run.
- Linear memory after all 178 runs through one module: 1,638,400 bytes, against a heap target of
  8 MB.

The compiler was TypeScript 7.0.2. TypeScript 5.9.3 refuses the run's empty `--types` value.

Not measured: step time. A browser run uses virtual time, and a Mac says nothing about the 2 ms
target on a 2021 TV. Not run: an old engine. The floor rests on the feature scan and on the loader
being built to ES2017. No old build is on this machine, and this step made no download.

## Checked in every run

- No token or device code in a view or the store, and no token on a video request or an open
  route.
- Every request is a path on a route of the contract, with the client headers, the token its route
  needs, and the class the route table gives.
- The core never uses the store. No command gets past the layer.
- Close comes before open and the generation only goes up.
- No busy loop: after a step, the next wake is later than now.
- Running a session twice gives the same bytes.
