#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

python3 "$HERE/write-brand.py"
mkdir -p "$HERE/Sources/CoreBridge/include" "$HERE/Sources/Contract" "$HERE/target"
cp "$ROOT/platform/bridges/c/core.h" "$HERE/Sources/CoreBridge/include/core.h"
cp "$ROOT/platform/contract/generated/types/Contract.swift" "$HERE/Sources/Contract/Contract.swift"

ARCH="${ARCHS:-$(uname -m)}"
ARM=0
case "$ARCH" in arm64|aarch64) ARM=1 ;; esac
# Each Apple platform keeps its own copy of the core, so a TV build and a phone build never
# overwrite each other. tvOS needs the standard library built from source; iOS ships it.
STD=""
case "${PLATFORM_NAME:-appletvsimulator}" in
    appletvos) TARGET="aarch64-apple-tvos"; KIND="tvos"; STD="-Z build-std=std,panic_unwind" ;;
    appletvsimulator)
        if [ $ARM = 1 ]; then TARGET="aarch64-apple-tvos-sim"; else TARGET="x86_64-apple-tvos"; fi
        KIND="tvos"; STD="-Z build-std=std,panic_unwind" ;;
    iphoneos) TARGET="aarch64-apple-ios"; KIND="ios" ;;
    iphonesimulator)
        if [ $ARM = 1 ]; then TARGET="aarch64-apple-ios-sim"; else TARGET="x86_64-apple-ios"; fi
        KIND="ios" ;;
    *) echo "unknown platform ${PLATFORM_NAME:-}" >&2; exit 1 ;;
esac

mkdir -p "$HERE/target/$KIND"
DEST="$HERE/target/$KIND/libbridge_c.a"
OUT_DIR="${CARGO_TARGET_DIR:-$HERE/target/cargo}"
ARCHIVE="$OUT_DIR/$TARGET/native/libbridge_c.a"
cd "$ROOT"
# shellcheck disable=SC2086
RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR="$OUT_DIR" CARGO_BUILD_JOBS=3 \
    nice -n 10 cargo build --locked --profile native -p bridge-c \
    --target "$TARGET" $STD
cp "$ARCHIVE" "$DEST"
