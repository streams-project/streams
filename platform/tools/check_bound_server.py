#!/usr/bin/env python3
"""Enforce bound server configuration, secure defaults, and accounts_required handling.

Every decision that lives only in notes gets undone by the next edit. This holds the tree to four
invariants added for note 205 / note 208 and the accounts_required audit:

  no address screen   no `*ServerAddress*` file, `fun Server`, `struct ServerAddress`, `Face::Address`,
                      `Ask::Server`, or `"settings.server"` key in surfaces or strings
  no viewer connect   `Shell` ports expose `retry`, never `connect(server)`; hosts hold no
                      `app:server` secret or `SERVER_URL` fallback
  transport security  tracked Android network_security_config and Apple Info.plists never enable
                      app-wide cleartext or `NSAllowsLocalNetworking`
  accounts_required   every app door surface honours `accounts_required` / `accountsRequired` and
                      offers Home navigation for visitors when accounts are off
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

BUILD_DIRS = frozenset({"build", ".gradle", "target", "dist", "node_modules", "DerivedData", ".build"})
TEST_DIRS = frozenset({"test", "androidTest", "testFixtures", "Tests", "tests"})
SOURCE_SUFFIXES = frozenset({".kt", ".java", ".swift", ".rs", ".ets", ".ts", ".tsx", ".cpp", ".h"})

SURFACE_FORBIDDEN = (
    ("ServerAddress declaration", re.compile(r"\b(?:struct|class|fun|fn)\s+ServerAddress\b")),
    ("Phone Server composable", re.compile(r"\bfun\s+Server\s*\(")),
    ("Desktop Face::Address", re.compile(r"\bFace::Address\b|\bAsk::Server\b|\bdoor::address\b")),
    ("settings.server usage", re.compile(r'"settings\.server"')),
)

PORTS_FILES = (
    "apps/android/ports/src/main/kotlin/streams/ports/Ports.kt",
    "apps/apple/Sources/Ports/Ports.swift",
    "apps/harmony/ports/Ports.ets",
    "apps/roku/src/ports.ts",
)

HOST_FILES = (
    "apps/android/core/src/main/kotlin/streams/core/Host.kt",
    "apps/apple/Sources/CoreHost/Host.swift",
    "apps/harmony/core/Host.ets",
)

ANDROID_NSC = "apps/android/app/src/main/res/xml/network_security_config.xml"
APPLE_PLISTS = (
    "apps/apple/App/Info.plist",
    "apps/apple/PhoneApp/Info.plist",
)

DOOR_SURFACES = (
    ("apps/web/src/surfaces/door.rs", ("accounts_required", "address::HOME")),
    ("apps/desktop/src/surfaces/door.rs", ("accounts_required", "address::HOME")),
    ("apps/xbox/surfaces/remote/door.cpp", ("accounts_required", "Opening::Home")),
    ("apps/android/surfaces/phone/src/main/kotlin/streams/phone/door/Doors.kt", ("accountsRequired", "Address.HOME")),
    ("apps/android/surfaces/tv/src/main/kotlin/streams/tv/door/Door.kt", ("accountsRequired", "Address.HOME")),
    ("apps/apple/Sources/SurfacesPhone/door/Door.swift", ("accountsRequired", "Address.home")),
    ("apps/apple/Sources/SurfacesTV/door/Door.swift", ("accountsRequired", "Address.home")),
)

LAUNCHER_MAIN = "launcher/src/main.rs"


def _skip_parts(rel_parts: tuple[str, ...]) -> bool:
    return bool(BUILD_DIRS.intersection(rel_parts[:-1]) or TEST_DIRS.intersection(rel_parts[:-1]))


def check_strings(strings_dir: Path) -> list[str]:
    found: list[str] = []
    files = sorted(strings_dir.glob("*.json"))
    if not files:
        return [f"missing language files under {strings_dir}"]
    for path in files:
        data = json.loads(path.read_text(encoding="utf-8"))
        if "settings.server" in data:
            found.append(f"{path.name}: still holds settings.server")
    return found


def check_surface_source(rel: str, text: str) -> list[str]:
    found: list[str] = []
    for label, pattern in SURFACE_FORBIDDEN:
        for match in pattern.finditer(text):
            line = text[: match.start()].count("\n") + 1
            found.append(f"{rel}:{line}: forbidden {label}: {match.group()}")
    return found


def check_apps_tree(apps_dir: Path) -> list[str]:
    found: list[str] = []
    scanned = 0
    for path in sorted(apps_dir.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(apps_dir)
        if _skip_parts(rel.parts):
            continue
        if "ServerAddress" in path.name:
            found.append(f"apps/{rel}: ServerAddress file must not exist")
            continue
        if path.suffix not in SOURCE_SUFFIXES:
            continue
        scanned += 1
        found += check_surface_source(f"apps/{rel}", path.read_text(encoding="utf-8", errors="replace"))
    if scanned < 50:
        found.append(f"apps/: only scanned {scanned} source files, expected at least 50")
    return found


def check_ports_text(rel: str, text: str) -> list[str]:
    found: list[str] = []
    if re.search(r"\b(?:fun|func)\s+connect\b|\bconnect\s*\(\s*server\b", text):
        found.append(f"{rel}: Shell port must not declare connect(server)")
    if not re.search(r"\b(?:fun|func)?\s*retry\s*\(", text):
        found.append(f"{rel}: Shell port must declare retry()")
    return found


def check_host_text(rel: str, text: str) -> list[str]:
    found: list[str] = []
    for forbidden in ('"app:server"', '"SERVER_URL"', "This address must start with https://"):
        if forbidden in text:
            found.append(f"{rel}: forbidden viewer server-address artifact {forbidden}")
    return found


def check_android_nsc_text(rel: str, text: str) -> list[str]:
    found: list[str] = []
    if re.search(r'<base-config[^>]*cleartextTrafficPermitted\s*=\s*"true"', text):
        found.append(f"{rel}: base-config must not allow cleartext app-wide")
    if 'cleartextTrafficPermitted="false"' not in text:
        found.append(f"{rel}: base-config must default cleartextTrafficPermitted=\"false\"")
    return found


def check_apple_plist_text(rel: str, text: str) -> list[str]:
    found: list[str] = []
    if "NSAllowsLocalNetworking" in text:
        found.append(f"{rel}: tracked Info.plist must not enable NSAllowsLocalNetworking")
    if "<key>SERVER</key>" not in text:
        found.append(f"{rel}: tracked Info.plist must declare <key>SERVER</key>")
    return found


def check_door_text(rel: str, text: str, tokens: tuple[str, ...]) -> list[str]:
    return [
        f"{rel}: door does not reference {token!r} (accounts_required handling missing)"
        for token in tokens
        if token not in text
    ]


def check_launcher_defaults(rel: str, text: str) -> list[str]:
    found: list[str] = []
    if not re.search(
        r'"ACCOUNTS_REQUIRED"[\s\S]{0,200}String::from\("false"\)', text
    ) and not re.search(
        r'read_env\("ACCOUNTS_REQUIRED"\)[\s\S]{0,120}String::from\("false"\)', text
    ):
        found.append(f"{rel}: launcher must default ACCOUNTS_REQUIRED to false")
    if '"VISITOR_TIER"' not in text:
        found.append(f"{rel}: launcher must configure VISITOR_TIER for visitor browsing")
    return found


def run_all(root: Path = ROOT) -> list[str]:
    found: list[str] = []
    found += check_strings(root / "strings")
    found += check_apps_tree(root / "apps")
    for rel in PORTS_FILES:
        path = root / rel
        if not path.is_file():
            found.append(f"missing {rel}")
        else:
            found += check_ports_text(rel, path.read_text(encoding="utf-8"))
    for rel in HOST_FILES:
        path = root / rel
        if not path.is_file():
            found.append(f"missing {rel}")
        else:
            found += check_host_text(rel, path.read_text(encoding="utf-8"))
    nsc = root / ANDROID_NSC
    if not nsc.is_file():
        found.append(f"missing {ANDROID_NSC}")
    else:
        found += check_android_nsc_text(ANDROID_NSC, nsc.read_text(encoding="utf-8"))
    for rel in APPLE_PLISTS:
        path = root / rel
        if not path.is_file():
            found.append(f"missing {rel}")
        else:
            found += check_apple_plist_text(rel, path.read_text(encoding="utf-8"))
    for rel, tokens in DOOR_SURFACES:
        path = root / rel
        if not path.is_file():
            found.append(f"missing {rel}")
        else:
            found += check_door_text(rel, path.read_text(encoding="utf-8"), tokens)
    launcher_main = root / LAUNCHER_MAIN
    if not launcher_main.is_file():
        found.append(f"missing {LAUNCHER_MAIN}")
    else:
        found += check_launcher_defaults(LAUNCHER_MAIN, launcher_main.read_text(encoding="utf-8"))
    return found


def main() -> int:
    found = run_all(ROOT)
    print(f"bound-server: checked {len(DOOR_SURFACES)} door surfaces, {len(PORTS_FILES)} ports, {len(HOST_FILES)} hosts")
    for issue in found:
        print(issue)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
