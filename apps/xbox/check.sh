#!/bin/sh
# Builds and runs everything in this folder that can run without an Xbox.
#
# Three tests: the core boundary (`host/`), which needs the C bridge built as a static library for
# whatever machine this is, and the input grammar and the drawing numbers (`surfaces/remote/`),
# which need nothing but a C++ compiler. All three are plain programs that print a line per check
# and return non-zero on a failure. The drawing numbers read `brand.json` from the repository root,
# which is why this script runs from there.
#
# The root `./check` is the whole of CI and it does not call this: it carries no C++ compiler and
# `platform/tools/check_rings.py` has no `xbox` family. So this runs when a person types it, and the
# first time the C ABI or the core's config shape moves, nothing will notice but this script.
#
# It proves nothing about an Xbox. What it proves is that the two pieces that do not need one are
# correct: bytes cross the boundary, and a controller speaks the six words of a remote.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

out=${TMPDIR:-/tmp}/streams-xbox-check
mkdir -p "$out"

case $(uname -s) in
Darwin) host="$(uname -m)-apple-darwin" ;;
Linux) host="$(uname -m)-unknown-linux-gnu" ;;
*) echo "no host target known for $(uname -s)" >&2; exit 1 ;;
esac

cxx=${CXX:-c++}
flags="-std=c++17 -Wall -Wextra"

echo "== the input grammar, which needs no core"
$cxx $flags apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/remote_test.cpp \
    -o "$out/remote"
"$out/remote"

echo
echo "== the drawing numbers, held against brand.json itself"
$cxx $flags apps/xbox/surfaces/remote/paint.cpp apps/xbox/surfaces/remote/paint_test.cpp \
    -o "$out/paint"
"$out/paint" brand.json

echo
echo "== the core boundary, against a real static library built for $host"
CARGO_INCREMENTAL=0 cargo build --locked --profile native -p bridge-c --target "$host"
$cxx $flags -I platform/bridges/c apps/xbox/host/core_host.cpp apps/xbox/host/smoke_test.cpp \
    "target/$host/native/libbridge_c.a" -o "$out/smoke"
"$out/smoke"

echo
echo "all three held. No test here touches an Xbox, WinRT, a screen or a controller."
