# games

A sandboxed WebAssembly game host and two sample games. Part of an open video platform.

Status: the host runs version 1 of the game ABI, and `platform/games/host/src/lib.rs` refuses a
module that exports anything else or imports anything at all. `platform/games/samples` holds two
games (`paddle`, `plasma`) and `platform/games/bench` times the host. Run the host's tests with
`cargo test --locked -p game-host` from the repository root.

## Decisions

### 2026-11-19: game audio is deferred to ABI version 2

Decided by the owner. No audio export exists in version 1, so a game has no way to make a sound.
Adding one changes what a module may export, which is a version change: it goes in version 2 and
not into version 1.
