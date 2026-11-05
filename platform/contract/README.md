# contract

The one written promise between server, core, finder and device. Everything needed to build a
client or a fake server is in this folder.

## What is here

- `src/`: the wire types. `no_std`, and serde is the only dependency of a normal build. `schemars`
  sits behind the `schema` feature, which only the generator turns on.
- `src/api.rs`: every route as data. OpenAPI comes from this table.
- `gen/`: the generator, which is the crate `contract-gen`.
- `generated/openapi.json`: OpenAPI 3.1. It names no server address, because the operator holds it.
- `generated/schemas/`: one JSON Schema file per type, each standing alone.
- `generated/types/`: TypeScript, Kotlin and Swift types. Other languages read plain JSON.
- `fixtures/<date>/`: plain JSON, never deleted.
- `ledger.csv`: every behaviour and defect with its status.

## Generate and check

    cargo run --locked -p contract-gen -- write
    cargo run --locked -p contract-gen -- check

The same input gives the same bytes. CI writes the files again and fails on any difference.

`tools/check_types.py` compiles the generated types and runs the fixtures through them. It takes
compiler paths from the environment, and its header lists them.

## Fixtures

Fixture files are named `<Type>.<case>.json`.

- `valid/`: reads and writes back the same. OpenAPI lists these as `examples`.
- `refused/`: must fail to parse.
- `tolerated/`: must read. A name with four parts, `<Type>.<case>.<what>.json`, must equal
  `valid/<Type>.<case>.json` once read.

## Rules of the wire

- JSON in UTF-8. Integers stay under 2^53, ids are strings, and no float appears.
- Unknown fields are ignored at every level.
- An unknown enum word reads as `Unknown` and never fails. An unknown section kind draws as a
  plain row. An unknown event is answered in `unsupported`, by its id when it has one.
- An optional field is left out and never sent as null.
- A title key is `movie:<id>` or `tv:<id>`. A bare id does not parse.
- An http effect sent to the screen brings a 2xx body to the screen only. Any other answer comes
  to the core, because the core reads its error code.
- `next_wake_ms` is the one timer a host keeps. The timer effect stays in the shape, and the core
  takes `timer_fired` as a plain wake.
- `Action::Request` sends a prepared request with the session's token, under the id of its event.
  This is how anything around the core reaches the server without ever holding a token.
- Three versions, kept apart: the wire by date in `Api-Version`, the C boundary by number, crates
  by semver. Clients are compared against a range, never for equality.

## Build a fake server from these files

1. Read `generated/openapi.json` for paths, methods, required headers and which token each route
   needs.
2. Each operation lists its request and answer shapes as `$ref`, and its fixtures as `examples`
   with an `externalValue` relative to `generated/`.
3. Answer with those fixtures. Answer an error with an `ErrorBody` fixture from the `default`
   response.

`tools/fake_server.py` does this in a few lines, reads nothing outside these files and opens no
socket. `tools/test_fake_server.py` walks cold start, play, save position, device sign in and a
hosted step through it.
