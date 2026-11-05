# templates/terminal

A host for the core in one C file. Copy it to start a host for a device with a C toolchain.

    make
    cargo run -p kit-core --bin tape -- the_slice_start_sign_in_heartbeat_pause_and_failover on out
    ../../target/terminal/terminal out/session.tape > out/out.txt
    cmp out/out.txt out/transcript.txt

`make` builds `bridges/c` with the `native` profile and links it. The JSON reader is the small part a
host needs to walk a message from the core, which holds no spaces.

To make it real, read your clocks in `advance_to`, send the request in `perform_http`, keep store and
secret on your device, and act on the player, view and open effects in `perform`.
