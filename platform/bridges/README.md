# bridges

Six functions carry UTF-8 JSON between the core and a host. There are no rules here.

    core_abi()                                   the ABI number
    core_alloc(len)                              room for len bytes
    core_free(ptr, len)                          give bytes back
    core_open(cfg_ptr, cfg_len)                  an opaque handle, or null
    core_step(handle, in_ptr, in_len,
              out_ptr_out, out_len_out)          a status; the out bytes go to core_free
    core_close(handle)

Pointers and lengths are i32 on Wasm and native width elsewhere. Time sits in the JSON, so no i64
crosses a Wasm boundary. The ABI stays frozen; what changes lives inside the messages.

The text given to `core_open` is a core config. A `layer` key beside its keys, holding a
`LayerConfig`, turns the offline layer on. With no such key the core runs alone. The core never
reads that key, so it never learns the layer exists.

Statuses: 0 ok, 1 the message does not read, 2 the core panicked, 3 a pointer was null.

## c

`core.h` is written by hand. `tests/header.rs` reads it and both bridges and fails when a signature,
a parameter name, the ABI number or a status differs.

    cargo build --locked --profile native -p bridge-c

The output is `target/native/libbridge_c.a`. The `native` profile matters: release aborts on panic,
which is right for Wasm, and `native` unwinds, so a panic stops at the boundary as status 2. After
that the handle stays refused. Close it and open a new one from what was stored.

Leave the layer out on a device with no storage with `--no-default-features`.

## wasm

    cargo build --locked --release -p bridge-wasm --target wasm32v1-none

The output is `target/wasm32v1-none/release/bridge_wasm.wasm`, called core.wasm below. It imports
nothing and exports the six functions and `memory`. `src/heap.rs` is its allocator: powers of two,
one free list per size, memory only grows. A panic traps, so no guard can exist here. The host
loads the module again and starts from what it stored.

Measured 2026-11-05 with rustc 1.98.1 on x86_64 macOS:

| Build | Raw bytes | gzip -9 bytes |
|---|---|---|
| Core and offline layer | 664,446 | 183,199 |
| Core alone, `--no-default-features` | 518,739 | 139,484 |

The target is at most 400 KB raw for the core plus layer, allocator included, so this misses it by
66 percent. brotli was not on the machine, so no brotli figure exists. `kit/core/tests/boundary.rs`
holds a ceiling of 700,000 bytes so the size cannot grow unseen.

## ts

`core.ts` loads core.wasm with the plain WebAssembly API and has no dependency. Give it the bytes:

    const module = await CoreModule.load(bytes);
    const core = module.open(config);
    const out = core.step(input);
    core.close();

A refused call throws `CoreRefused` and the core carries on. A trap throws `CoreTrapped`, and that
module stays refused from then on: load it again.
