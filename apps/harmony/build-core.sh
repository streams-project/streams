#!/bin/sh
# Builds the core for HarmonyOS and puts the generated contract types where ArkTS can import them.
# The same job as apps/apple/build-core.sh and the buildCore task in apps/android/core/build.gradle.kts.
#
# No HarmonyOS SDK is needed for this: a staticlib is assembled by rustc and never linked, so the
# SDK's clang is wanted only later, by CMake, when it links libstreamscore.so against this archive.
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="${CARGO_TARGET_DIR:-$ROOT/target}"

# Each HarmonyOS CPU and the Rust target that builds for it. Phones and tablets are 64 bit arm;
# armv7 is here for older HarmonyOS hardware and x86_64 for the emulator.
TARGETS="aarch64-unknown-linux-ohos armv7-unknown-linux-ohos x86_64-unknown-linux-ohos"
if [ "$#" -gt 0 ]; then
    TARGETS="$*"
fi

for target in $TARGETS; do
    ( cd "$ROOT" && CARGO_TARGET_DIR="$OUT" nice -n 15 \
        cargo build --locked --profile native -p bridge-c --target "$target" )
    ls -l "$OUT/$target/native/libbridge_c.a"
done

# ArkTS reads the contract from the same generated file the television page does. Copied, not
# edited: platform/contract is the only place it is written.
mkdir -p "$HERE/contract"
cp "$ROOT/platform/contract/generated/types/contract.ts" "$HERE/contract/contract.ts"
