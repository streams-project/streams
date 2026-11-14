#!/bin/sh
# Runs everything in this folder that can run without a HarmonyOS SDK.
#
# `node`, `npx`, `tsc`, `cmake`, `ohpm`, `hvigorw` and `codelinter` are all absent from this
# machine, so not one line of the ArkTS can be compiled, type checked, linted or run, and nothing
# below pretends otherwise. What CAN be done is two different things, and this runs both:
#
#   1. THE C GLUE IS BUILT AND RUN. `core/cpp/napi_bridge.c` is the whole boundary between ArkTS
#      and the core, it is C, and C runs here. Twice: against a core that records every call and
#      holds the ledger that every block taken from `core_alloc` goes back to `core_free` with its
#      OWN size, and against the real `libbridge_c.a`, which proves bytes cross to the core that
#      ships. Both drive the bridge through the stub runtime in `core/cpp/test/`.
#
#   2. THE ARKTS IS READ. Every number, word, colour, corner, icon and import in `surfaces/touch`
#      is text, and so is the Android phone Kotlin that measured it, `brand.json` and
#      `strings/en.json`. `touch_check.py` holds one against the other, which is the same shape the
#      television's Kotlin is held in by `apps/android/.../streams/tv/NumbersTest.kt`. It proves
#      nothing about whether the ArkTS COMPILES; it proves that nothing in it was invented here.
#
# What none of it proves is that OpenHarmony's `napi/native_api.h` declares these calls the way the
# upstream Node-API headers do; that is stated in `core/cpp/test/napi/native_api.h` and is
# `OPEN-QUESTIONS.md` Q1, and the first build against a real SDK is what settles it.
#
# The root `./check` does not call the C suites below, because it carries no C compiler for this
# folder. It does now call the ArkTS reading: `touch_check.py` moved to `platform/tools/` on
# 2026-11-14 and is a step of `./check`, and `check_rings.py` gained a `harmony` family the same
# day. This script still runs the reading too, so that running this one file proves the whole
# folder.
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
echo "== the touch surface read against what it was written from"
python3 platform/tools/touch_check.py

echo
echo "all three held. Nothing here ran HarmonyOS, DevEco, a compiler for ArkTS or a Huawei device,"
echo "and no line of ArkTS in this folder has been compiled. See apps/harmony/README.md."
