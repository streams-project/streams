#!/bin/sh
# Builds and runs everything in this folder that can run without an Xbox.
#
# Nine tests. The core boundary (`host/`) needs the C bridge built as a static library for whatever
# machine this is; the other eight (`surfaces/remote/`) need nothing but a C++ compiler. All nine are
# plain programs that print a line per check and return non-zero on a failure, and three of them are
# held against a file nobody here owns: the drawing numbers read `brand.json` and
# `reference/netflix-on-the-tv.md`, and the contract reading reads the contract's own fixtures,
# which is why this script runs from the repository root.
#
# The root `./check` is the whole of CI and it does not call this: it carries no C++ compiler and
# `platform/tools/check_rings.py` has no `xbox` family. So this runs when a person types it, and the
# first time the C ABI, the core's config shape, the brand or the contract moves, nothing will
# notice but this script.
#
# It proves nothing about an Xbox. What it proves is that the pieces that do not need one are
# correct: bytes cross the boundary, a controller speaks the six words of a remote, the drawn
# numbers are the brand's and the reference's, the remote reaches the stop it should on a page of
# rows, on a title's own three pages, on the profile gate and on the door, no move on the keyboard
# leaves the grid from any of its three hundred and sixty stops, and a C++ surface reads the
# contract with nothing between it and the generated header.
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

contract=platform/contract/generated/types
# The wire the generated header itself names, so the fixtures are never chosen by a date written
# twice.
wire=$(sed -n 's/^inline constexpr std::string_view wire_version = "\(.*\)";$/\1/p' \
    "$contract/Contract.hpp")
fixtures=platform/contract/fixtures/$wire/valid

cxx=${CXX:-c++}
flags="-std=c++17 -Wall -Wextra"

echo "== the input grammar, which needs no core"
$cxx $flags apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/remote_test.cpp \
    -o "$out/remote"
"$out/remote"

echo
echo "== the drawing numbers, held against brand.json and the television reference"
$cxx $flags apps/xbox/surfaces/remote/paint.cpp apps/xbox/surfaces/remote/paint_test.cpp \
    -o "$out/paint"
"$out/paint" brand.json reference/netflix-on-the-tv.md

echo
echo "== where the remote is on a page, and where a row opens when it comes back"
$cxx $flags apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/place.cpp \
    apps/xbox/surfaces/remote/place_test.cpp -o "$out/place"
"$out/place"

echo
echo "== where the remote is on a title's own pages, and what Play asks for"
$cxx $flags apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/place.cpp \
    apps/xbox/surfaces/remote/paint.cpp apps/xbox/surfaces/remote/title.cpp \
    apps/xbox/surfaces/remote/title_test.cpp -o "$out/title"
"$out/title"

echo
echo "== the profile gate, the pencil, and the panel in front of a locked profile"
$cxx $flags apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/gate.cpp \
    apps/xbox/surfaces/remote/gate_test.cpp -o "$out/gate"
"$out/gate"

echo
echo "== the keyboard: every key from every key, in all four directions"
$cxx $flags apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/keys.cpp \
    apps/xbox/surfaces/remote/keys_test.cpp -o "$out/keys"
"$out/keys"

echo
echo "== the door: the landing, the two ways in, and the keyboard under one of them"
$cxx $flags apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/keys.cpp \
    apps/xbox/surfaces/remote/door.cpp apps/xbox/surfaces/remote/door_test.cpp -o "$out/door"
"$out/door"

echo
echo "== reading the contract into a page, held against the contract's own $wire fixtures"
$cxx $flags -I "$contract" apps/xbox/surfaces/remote/remote.cpp \
    apps/xbox/surfaces/remote/place.cpp apps/xbox/surfaces/remote/reading.cpp \
    apps/xbox/surfaces/remote/reading_test.cpp -o "$out/reading"
"$out/reading" "$fixtures"

echo
echo "== the core boundary, against a real static library built for $host"
CARGO_INCREMENTAL=0 cargo build --locked --profile native -p bridge-c --target "$host"
$cxx $flags -I platform/bridges/c apps/xbox/host/core_host.cpp apps/xbox/host/smoke_test.cpp \
    "target/$host/native/libbridge_c.a" -o "$out/smoke"
"$out/smoke"

echo
echo "all nine held. No test here touches an Xbox, WinRT, a screen or a controller."
