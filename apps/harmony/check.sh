#!/bin/sh
# Builds and runs everything in this folder that can run without a HarmonyOS SDK.
#
# That is two tests, and both of them are the C glue. Nothing else here can run on this machine:
# `node`, `npx`, `tsc`, `cmake`, `ohpm`, `hvigorw` and `codelinter` are all absent, so not one line
# of the ArkTS -- the host, the ports, the contract seam and the twenty files of `surfaces/touch` --
# can be compiled, let alone run. The README says so and this script does not pretend otherwise.
#
#   1. The bridge against a recording core, which drives every refusal and holds the ledger: every
#      block taken from `core_alloc` goes back to `core_free` with its own size.
#   2. The bridge against the real `libbridge_c.a`, which proves bytes cross to the core that ships.
#
# Both drive `core/cpp/napi_bridge.c` itself through the stub runtime in `core/cpp/test/`. What
# neither proves is that OpenHarmony's `napi/native_api.h` declares these calls the way the upstream
# Node-API headers do; that is stated in `core/cpp/test/napi/native_api.h` and is `OPEN-QUESTIONS.md`
# Q1, and the first build against a real SDK is what settles it.
#
# The root `./check` does not call this, for the same reason it does not call `apps/xbox/check.sh`:
# it carries no C compiler for this folder and `platform/tools/check_rings.py` has no `harmony`
# family.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

out=${TMPDIR:-/tmp}/streams-harmony-check
mkdir -p "$out"

case $(uname -s) in
Darwin) host="$(uname -m)-apple-darwin" ;;
Linux) host="$(uname -m)-unknown-linux-gnu" ;;
*) echo "no host target known for $(uname -s)" >&2; exit 1 ;;
esac

cc=${CC:-cc}
flags="-std=c11 -Wall -Wextra"
glue="apps/harmony/core/cpp"
include="-I platform/bridges/c -I $glue/test"

echo "== the bridge against a core that records what it was asked"
$cc $flags $include "$glue/napi_bridge.c" "$glue/test/napi_stub.c" "$glue/test/fake_core.c" \
    "$glue/test/bridge_test.c" -o "$out/bridge"
"$out/bridge"

echo
echo "== the bridge against a real static library built for $host"
CARGO_INCREMENTAL=0 cargo build --locked --profile native -p bridge-c --target "$host"
$cc $flags $include "$glue/napi_bridge.c" "$glue/test/napi_stub.c" "$glue/test/bridge_smoke.c" \
    "target/$host/native/libbridge_c.a" -o "$out/smoke"
"$out/smoke"

echo
echo "both held. No test here touches HarmonyOS, DevEco, ArkTS or a Huawei device."
