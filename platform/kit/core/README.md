# kit/core

Proof that the core works, and that the offline layer changes nothing the core decides.

## What runs

    cargo test -p kit-core

- `tests/sessions.rs` runs every file in `sessions/` with the layer on and off, through the JSON
  boundary.
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

## Checked in every run

- No token in a view or the store, and none on a video request or an open route.
- Every request is a path on a route of the contract, with the client headers, the token its route
  needs, and the class the route table gives.
- The core never uses the store. No command gets past the layer.
- Close comes before open and the generation only goes up.
- No busy loop: after a step, the next wake is later than now.
- Running a session twice gives the same bytes.
