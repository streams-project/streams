#!/usr/bin/env python3
"""Mutation tests for tools/check_bound_server.py."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent

spec = importlib.util.spec_from_file_location("check_bound_server", TOOLS / "check_bound_server.py")
bound = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bound)


class BoundServerRules(unittest.TestCase):
    def test_catches_server_address_screens_and_settings_server(self):
        self.assertTrue(bound.check_surface_source("Door.kt", "fun Server(shell: Shell) {}"))
        self.assertTrue(bound.check_surface_source("Door.swift", "struct ServerAddress: View {}"))
        self.assertTrue(bound.check_surface_source("mod.rs", "Face::Address => door::address(self)"))
        self.assertTrue(bound.check_surface_source("Settings.kt", 'words.say("settings.server")'))
        self.assertEqual(bound.check_surface_source("Door.kt", "fun Door(shell: Shell) {}"), [])

    def test_catches_settings_server_in_strings_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "en.json").write_text(json.dumps({"door.sign_in": "Sign In"}), encoding="utf-8")
            self.assertEqual(bound.check_strings(root), [])
            (root / "en.json").write_text(
                json.dumps({"settings.server": "Server"}), encoding="utf-8"
            )
            self.assertTrue(bound.check_strings(root))

    def test_catches_shell_connect_or_missing_retry(self):
        self.assertTrue(bound.check_ports_text("Ports.kt", "interface Shell { fun connect(server: String) }"))
        self.assertTrue(bound.check_ports_text("Ports.kt", "interface Shell { fun go(address: String) }"))
        self.assertEqual(
            bound.check_ports_text("Ports.kt", "interface Shell { fun retry() }"), []
        )

    def test_catches_host_address_persistence_and_env_fallback(self):
        self.assertTrue(bound.check_host_text("Host.swift", 'let x = ProcessInfo.processInfo.environment["SERVER_URL"]'))
        self.assertTrue(bound.check_host_text("Host.kt", 'const val SERVER = "app:server"'))
        self.assertEqual(bound.check_host_text("Host.kt", "class Host(private val server: String) {}"), [])

    def test_catches_app_wide_cleartext_and_apple_local_networking(self):
        self.assertTrue(
            bound.check_android_nsc_text("nsc.xml", '<base-config cleartextTrafficPermitted="true" />')
        )
        self.assertEqual(
            bound.check_android_nsc_text("nsc.xml", '<base-config cleartextTrafficPermitted="false" />'),
            [],
        )
        self.assertTrue(
            bound.check_apple_plist_text(
                "Info.plist", "<dict><key>NSAllowsLocalNetworking</key><true/><key>SERVER</key></dict>"
            )
        )
        self.assertEqual(
            bound.check_apple_plist_text("Info.plist", "<dict><key>SERVER</key><string>$(SERVER)</string></dict>"),
            [],
        )

    def test_catches_door_ignoring_accounts_required(self):
        self.assertTrue(
            bound.check_door_text("Doors.kt", "fun Door() { shell.go(Address.SIGN_IN) }", ("accountsRequired", "Address.HOME"))
        )
        self.assertEqual(
            bound.check_door_text(
                "Doors.kt",
                "if (!seen.accountsRequired) { shell.go(Address.HOME) }",
                ("accountsRequired", "Address.HOME"),
            ),
            [],
        )

    def test_catches_server_address_file_on_disk(self):
        with tempfile.TemporaryDirectory() as tmp:
            apps = Path(tmp) / "apps"
            (apps / "android/surfaces").mkdir(parents=True)
            for i in range(50):
                (apps / f"android/surfaces/F{i}.kt").write_text("package x\n", encoding="utf-8")
            self.assertEqual(bound.check_apps_tree(apps), [])
            (apps / "android/surfaces/ServerAddress.kt").write_text("package x\n", encoding="utf-8")
            issues = bound.check_apps_tree(apps)
            self.assertTrue(any("ServerAddress.kt" in issue for issue in issues))


if __name__ == "__main__":
    unittest.main()
