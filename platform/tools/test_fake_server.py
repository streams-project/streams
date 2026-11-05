#!/usr/bin/env python3
"""The fake server, built from the contract's files alone, walks the flows in the design."""

import json
import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import fake_server  # noqa: E402
from fake_server import FakeServer, Refused, check  # noqa: E402

CONTRACT = os.path.join(os.path.dirname(HERE), "contract")
HEADERS = {
    "Api-Version": "2026-11-05",
    "Client-App": "fake",
    "Client-Version": "1.0.0",
    "Client-Class": "web",
    "Client-Os": "fake",
    "Authorization": "Bearer fake",
}


def fixture_folder():
    return os.path.join(CONTRACT, "fixtures", "2026-11-05")


def fixtures(kind):
    folder = os.path.join(fixture_folder(), kind)
    for name in sorted(os.listdir(folder)):
        with open(os.path.join(folder, name)) as handle:
            yield name, json.load(handle)


def schema_of(type_name):
    with open(os.path.join(CONTRACT, "generated", "schemas", type_name + ".json")) as handle:
        return json.load(handle)


def fill(path):
    words = {"{kind}": "movie", "{id}": "1001", "{name}": "home", "{n}": "1"}
    for slot, word in words.items():
        path = path.replace(slot, word)
    return path.replace("{lang}", "en").replace("{cursor}", "c1").replace("{file}", "f1").replace("{job}", "j1")


class Validator(unittest.TestCase):
    def test_it_refuses_a_wrong_type_and_a_missing_field(self):
        schema = schema_of("Attempt")
        with self.assertRaises(Refused):
            check({"id": 1}, schema, schema)
        with self.assertRaises(Refused):
            check([], schema, schema)

    def test_every_valid_fixture_matches_its_schema_file(self):
        count = 0
        for name, value in fixtures("valid"):
            schema = schema_of(name.split(".")[0])
            check(value, schema, schema)
            count += 1
        self.assertGreater(count, 190)

    def test_every_refused_fixture_is_refused_by_the_schema_alone(self):
        count = 0
        for name, value in fixtures("refused"):
            schema = schema_of(name.split(".")[0])
            with self.assertRaises(Refused, msg=name):
                check(value, schema, schema)
            count += 1
        self.assertGreaterEqual(count, 20)

    def test_a_bare_id_is_refused_by_the_schema_alone_h57_h91(self):
        schema = schema_of("TitleKey")
        for value in ["550", 550]:
            with self.assertRaises(Refused):
                check(value, schema, schema)

    def test_extra_fields_pass_the_schema(self):
        count = 0
        for name, value in fixtures("tolerated"):
            if "extra_fields" in name:
                schema = schema_of(name.split(".")[0])
                check(value, schema, schema)
                count += 1
        self.assertGreaterEqual(count, 4)


class EveryOperation(unittest.TestCase):
    def setUp(self):
        self.server = FakeServer(CONTRACT)

    def test_every_operation_has_fixtures_and_answers(self):
        self.assertGreater(len(self.server.routes), 55)
        for method, template, _, operation in self.server.routes:
            body = None
            if "requestBody" in operation:
                content = operation["requestBody"]["content"]
                body = self.server.example(content)
            status, answer = self.server.answer(method, fill(template), HEADERS, body)
            self.assertEqual(status, 200, template)
            schema = operation["responses"]["200"]["content"]["application/json"]["schema"]
            check(answer, schema, self.server.document)

    def test_unknown_path_answers_a_coded_error(self):
        status, answer = self.server.answer("GET", "/v1/nothing", HEADERS)
        self.assertEqual(status, 404)
        self.assertEqual(answer["code"], "not_found")

    def test_a_missing_version_header_is_answered_as_data(self):
        status, answer = self.server.answer("GET", "/v1/hello", {})
        self.assertEqual(status, 400)
        self.assertEqual(answer["code"], "upgrade_required")
        self.assertIn("text", answer["notice"])

    def test_a_route_with_a_token_refuses_a_call_without_one(self):
        headers = dict(HEADERS)
        del headers["Authorization"]
        status, answer = self.server.answer("GET", "/v1/nav", headers)
        self.assertEqual(status, 401)
        self.assertEqual(answer["code"], "profile_required")

    def test_a_bare_id_in_a_body_is_refused_h57(self):
        status, answer = self.server.answer("POST", "/v1/play", HEADERS, {"title": "550"})
        self.assertEqual(status, 400)
        self.assertIn("refused", answer)

    def test_the_finder_routes_need_no_client_headers(self):
        status, _ = self.server.answer("GET", "/v1/health", {})
        self.assertEqual(status, 200)


class Flows(unittest.TestCase):
    """The five flows of the design, driven by paths and fixtures only."""

    def setUp(self):
        self.server = FakeServer(CONTRACT)

    def ok(self, method, path, body=None, case=None):
        status, answer = self.server.answer(method, path, HEADERS, body, case)
        self.assertEqual(status, 200, path)
        return answer

    def test_cold_start(self):
        hello = self.ok("GET", "/v1/hello")
        self.assertLessEqual(hello["contract"]["min"], hello["contract"]["max"])
        self.assertIn("accounts_required", hello["features"])
        card = {"device_key": "0123456789abcdef", "card_hash": "h1"}
        self.assertFalse(self.ok("POST", "/v1/devices/me", card)["send_card"])
        nav = self.ok("GET", "/v1/nav")
        self.assertEqual(nav["items"][0]["screen"], "home")
        screen = self.ok("GET", "/v1/screens/home", case="home")
        self.assertTrue(screen["sections"])
        self.ok("GET", "/v1/changes")

    def test_press_play_then_a_source_dies(self):
        play = self.ok("POST", "/v1/play", {"title": "movie:1001"})
        attempts = play["attempts"]
        self.assertGreaterEqual(len(attempts), 2)
        self.assertNotEqual(attempts[0]["group"], attempts[1]["group"])
        self.assertGreater(play["start_at_ms"], 0)
        for attempt in attempts:
            self.assertIn("headers", attempt)

    def test_save_position_answers_per_op(self):
        batch = {"ops": [{"op_id": "1", "command": "position", "title": "tv:1001", "position_s": 61, "duration_s": 1200}]}
        answer = self.ok("POST", "/v1/commands", batch)
        self.assertEqual(
            sorted(entry["outcome"] for entry in answer["answers"]),
            ["applied", "rejected", "superseded"],
        )

    def test_tv_sign_in(self):
        code = self.ok("POST", "/v1/auth/device/code", {"device_name": "Fake"})
        self.assertEqual((code["expires_in"], code["interval"]), (900, 5))
        token = {"device_code": code["device_code"]}
        waiting = self.ok("POST", "/v1/auth/device/token", token, case="pending")
        self.assertEqual(waiting["poll"], "authorization_pending")
        done = self.ok("POST", "/v1/auth/device/token", token, case="signed_in")
        self.assertIn("access_token", done["signed_in"])

    def test_hosted_step(self):
        step = self.ok(
            "POST",
            "/v1/core/step",
            {
                "input": {
                    "contract": "2026-11-05",
                    "type": "core_in",
                    "now_ms": 1,
                    "mono_ms": 1,
                    "events": [{"type": "activity"}],
                }
            },
        )
        self.assertEqual(step["type"], "core_out")


class Isolation(unittest.TestCase):
    def test_the_fake_reads_only_contract_files_and_opens_no_socket(self):
        with open(fake_server.__file__) as handle:
            text = handle.read()
        for word in ["import socket", "http.server", "asyncio", "urllib", "cargo", "ledger"]:
            self.assertNotIn(word, text.split('"""', 2)[2])


if __name__ == "__main__":
    unittest.main()
