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

# Each HarmonyOS CPU and the Rust target that builds for it.
#
# Two, and there is no third. The `externalNativeOptions.abiFilters` field of a module's
# `build-profile.json5` takes exactly `arm64-v8a` and `x86_64` (read off the reference on
# 2026-11-12, "Configuring for C++ Builds"), and defaults to `arm64-v8a` alone. So a 32-bit ARM core
# has nothing to link into, and the `armv7-unknown-linux-ohos` this script used to build on
# 2026-11-11 -- "here on the guess that older HarmonyOS hardware wants them" -- was 31 MB of archive
# for an architecture the toolchain will not package. It is gone. aarch64 is the one target note 151
# (line 275) names for CI; x86_64 is the emulator, which has to be named in `abiFilters` explicitly
# because the default leaves it out.
TARGETS="aarch64-unknown-linux-ohos x86_64-unknown-linux-ohos"
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
