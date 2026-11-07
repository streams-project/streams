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
if [ "${EFFECTIVE_PLATFORM_NAME:-}" = "-appletvos" ]; then
    TARGET="aarch64-apple-tvos"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    TARGET="aarch64-apple-tvos-sim"
else
    TARGET="x86_64-apple-tvos"
fi

DEST="$HERE/target/libbridge_c.a"
OUT_DIR="${CARGO_TARGET_DIR:-$HERE/target/cargo}"
ARCHIVE="$OUT_DIR/$TARGET/native/libbridge_c.a"
cd "$ROOT"
RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR="$OUT_DIR" CARGO_BUILD_JOBS=3 \
    nice -n 10 cargo build --locked --profile native -p bridge-c \
    --target "$TARGET" -Z build-std=std,panic_unwind
cp "$ARCHIVE" "$DEST"
