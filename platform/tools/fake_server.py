#!/usr/bin/env python3
"""A fake server built only from the contract's published files.

It reads contract/generated and contract/fixtures and nothing else: no Rust, no ledger, no notes.
It opens no socket; `answer` is a plain function, so a test can call it directly. Standard library
only. A person writing a fake server in any language needs exactly what this file uses.
"""

import json
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_CONTRACT = os.path.join(os.path.dirname(HERE), "contract")


class Refused(Exception):
    """A value does not match a schema."""


def check(value, schema, root, where="$"):
    """Validate value against the JSON Schema keywords the contract uses."""
    if "$ref" in schema:
        target = root
        for part in schema["$ref"].lstrip("#/").split("/"):
            target = target[part]
        check(value, target, root, where)
    kind = schema.get("type")
    if kind is not None and not type_matches(value, kind):
        raise Refused("%s: expected %s, got %r" % (where, kind, value))
    if "const" in schema and value != schema["const"]:
        raise Refused("%s: expected %r, got %r" % (where, schema["const"], value))
    if "enum" in schema and value not in schema["enum"]:
        raise Refused("%s: %r is not one of %r" % (where, value, schema["enum"]))
    if "pattern" in schema and isinstance(value, str):
        if re.search(schema["pattern"], value) is None:
            raise Refused("%s: %r does not match %s" % (where, value, schema["pattern"]))
    if isinstance(value, int) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            raise Refused("%s: %r is below the minimum" % (where, value))
        if "maximum" in schema and value > schema["maximum"]:
            raise Refused("%s: %r is above the maximum" % (where, value))
    if "oneOf" in schema:
        matches = 0
        reasons = []
        for option in schema["oneOf"]:
            try:
                check(value, option, root, where)
                matches += 1
            except Refused as reason:
                reasons.append(str(reason))
        if matches != 1:
            raise Refused("%s: %d options match. %s" % (where, matches, reasons[:2]))
    if isinstance(value, dict):
        for name in schema.get("required", []):
            if name not in value:
                raise Refused("%s: %s is required" % (where, name))
        properties = schema.get("properties", {})
        extra = schema.get("additionalProperties")
        for name, inner in value.items():
            if name in properties:
                check(inner, properties[name], root, "%s.%s" % (where, name))
            elif isinstance(extra, dict):
                check(inner, extra, root, "%s.%s" % (where, name))
    if isinstance(value, list) and "items" in schema:
        for index, inner in enumerate(value):
            check(inner, schema["items"], root, "%s[%d]" % (where, index))


def type_matches(value, kind):
    if kind == "object":
        return isinstance(value, dict)
    if kind == "array":
        return isinstance(value, list)
    if kind == "string":
        return isinstance(value, str)
    if kind == "boolean":
        return isinstance(value, bool)
    if kind == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    raise Refused("the contract does not use type %s" % kind)


class FakeServer:
    """Answers every operation in openapi.json with a fixture named by that operation."""

    def __init__(self, contract_dir=DEFAULT_CONTRACT):
        self.generated = os.path.join(contract_dir, "generated")
        with open(os.path.join(self.generated, "openapi.json")) as handle:
            self.document = json.load(handle)
        self.routes = []
        for path, item in sorted(self.document["paths"].items()):
            pattern = "^" + re.sub(r"\{[^}]+\}", "([^/]+)", path) + "$"
            for method, operation in sorted(item.items()):
                self.routes.append((method.upper(), path, re.compile(pattern), operation))
        self.routes.sort(key=lambda route: route[1].count("{"))

    def example(self, content, case=None):
        """The fixture an operation lists, the first by name unless a case is named."""
        examples = content["application/json"]["examples"]
        name = case if case is not None else sorted(examples)[0]
        with open(os.path.join(self.generated, examples[name]["externalValue"])) as handle:
            return json.load(handle)

    def error(self, operation, case):
        return self.example(operation["responses"]["default"]["content"], case)

    def find(self, method, path):
        for route_method, template, pattern, operation in self.routes:
            if route_method == method and pattern.match(path):
                return template, operation
        return None, None

    def required_headers(self, operation):
        names = []
        for parameter in operation["parameters"]:
            if "$ref" in parameter:
                name = parameter["$ref"].rsplit("/", 1)[1]
                names.append(self.document["components"]["parameters"][name]["name"])
        return names

    def answer(self, method, path, headers=None, body=None, case=None):
        """Return (status, body). Same rules a real door applies at the shape level."""
        headers = headers or {}
        template, operation = self.find(method, path)
        if operation is None:
            any_route = self.routes[0][3]
            return 404, self.error(any_route, "not_found")
        for name in self.required_headers(operation):
            if name not in headers:
                return 400, self.error(operation, "upgrade_required")
        if operation["security"] and "Authorization" not in headers:
            return 401, self.error(operation, "profile_required")
        if "requestBody" in operation:
            schema = operation["requestBody"]["content"]["application/json"]["schema"]
            try:
                check(body, schema, self.document)
            except Refused as reason:
                return 400, {"refused": str(reason), **self.error(operation, "not_found")}
        return 200, self.example(operation["responses"]["200"]["content"], case)


def main():
    server = FakeServer()
    print("%d operations from openapi.json" % len(server.routes))


if __name__ == "__main__":
    main()
