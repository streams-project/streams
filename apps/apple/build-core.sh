#!/bin/bash
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

python3 "$HERE/write-brand.py"
mkdir -p "$HERE/Sources/CoreBridge/include" "$HERE/Sources/Contract" "$HERE/target"
cp "$ROOT/platform/bridges/c/core.h" "$HERE/Sources/CoreBridge/include/core.h"
cp "$ROOT/platform/contract/generated/types/Contract.swift" "$HERE/Sources/Contract/Contract.swift"

RAW_ARCHS="${ARCHS:-$(uname -m)}"
if [ "${CURRENT_ARCH:-}" != "undefined_arch" ] && [ -n "${CURRENT_ARCH:-}" ]; then
    RAW_ARCHS="$CURRENT_ARCH"
fi

PLAT="${PLATFORM_NAME:-appletvsimulator}"
case "$PLAT" in
    appletvos) KIND="tvos" ;;
    appletvsimulator) KIND="tvos" ;;
    iphoneos) KIND="ios" ;;
    iphonesimulator) KIND="ios" ;;
    *) echo "unknown platform ${PLAT:-}" >&2; exit 1 ;;
esac

# Each Apple platform keeps its own copy of the core, so a TV build and a phone build never
# overwrite each other. tvOS needs the standard library built from source; iOS ships it.
mkdir -p "$HERE/target/$KIND"
DEST="$HERE/target/$KIND/libbridge_c.a"
OUT_DIR="${CARGO_TARGET_DIR:-$HERE/target/cargo}"

ARCHIVES=()
TARGETS=()

for a in $RAW_ARCHS; do
    T=""
    S=""
    case "$a" in
        arm64|aarch64)
            case "$PLAT" in
                iphoneos) T="aarch64-apple-ios"; S="" ;;
                iphonesimulator) T="aarch64-apple-ios-sim"; S="" ;;
                appletvos) T="aarch64-apple-tvos"; S="-Z build-std=std,panic_unwind" ;;
                appletvsimulator) T="aarch64-apple-tvos-sim"; S="-Z build-std=std,panic_unwind" ;;
            esac
            ;;
        x86_64)
            case "$PLAT" in
                iphoneos|iphonesimulator) T="x86_64-apple-ios"; S="" ;;
                appletvos|appletvsimulator) T="x86_64-apple-tvos"; S="-Z build-std=std,panic_unwind" ;;
            esac
            ;;
        *)
            continue
            ;;
    esac

    already=0
    for prev in "${TARGETS[@]-}"; do
        if [ "$prev" = "$T" ]; then already=1; break; fi
    done
    if [ "$already" -eq 1 ]; then continue; fi

    TARGETS+=("$T")
    ARCHIVE="$OUT_DIR/$T/native/libbridge_c.a"
    ARCHIVES+=("$ARCHIVE")

    cd "$ROOT"
    # shellcheck disable=SC2086
    RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR="$OUT_DIR" CARGO_BUILD_JOBS=3 \
        nice -n 10 cargo build --locked --profile native -p bridge-c \
        --target "$T" $S
done

if [ ${#ARCHIVES[@]} -eq 1 ]; then
    cp "${ARCHIVES[0]}" "$DEST"
elif [ ${#ARCHIVES[@]} -gt 1 ]; then
    lipo -create -output "$DEST" "${ARCHIVES[@]}"
fi
