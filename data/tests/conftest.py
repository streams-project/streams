"""Shared fixtures. Nothing here reaches a network: the fake transport answers from a table."""

from __future__ import annotations

import gzip
import json
import os
from collections.abc import Callable
from dataclasses import dataclass, field, replace
from datetime import UTC, date, datetime
from pathlib import Path
from typing import Any

import pytest

os.environ["DATABUILD_REFUSED_FILE"] = str(Path(__file__).parent / "fixtures" / "refused.json")

from databuild.context import Context
from databuild.net import NetError, Reply, Request
from databuild.settings import Settings, Tmdb

AS_OF = date(2026, 11, 5)
NOW = datetime(2026, 11, 5, 12, 0, 0, tzinfo=UTC)
API = "https://api.fake.test/3"
EXPORTS = "https://exports.fake.test/p"
KEY = "not-a-real-key-value"


@dataclass
class FakeTransport:
    """`routes` maps a path to a reply, a list used in turn, or a function of the request."""

    routes: dict[str, Any] = field(default_factory=dict)
    calls: list[Request] = field(default_factory=list)

    def fetch(self, request: Request) -> Reply:
        self.calls.append(request)
        route = self.routes.get(request.path)
        if isinstance(route, list):
            route = route.pop(0) if len(route) > 1 else route[0]
        if callable(route):
            route = route(request)
        if route is None:
            return Reply(404)
        if isinstance(route, Exception):
            raise route
        if isinstance(route, Reply):
            return route
        return Reply(200, json.dumps(route).encode())


def gz_lines(rows: list[dict[str, Any]]) -> bytes:
    return gzip.compress("\n".join(json.dumps(r) for r in rows).encode())


def make_context(tmp: Path, transport: Any = None, **tmdb: Any) -> Context:
    settings = Settings(tmdb=replace(Tmdb(workers=2, rps=100000.0), **tmdb))
    sink: list[str] = []
    ctx = Context(
        settings=settings,
        as_of=AS_OF,
        now=lambda: NOW,
        work=tmp / "work",
        out=tmp / "out",
        commit="abc123",
        say=sink.append,
    )
    if transport is not None:
        ctx.transport = transport
    ctx.said = sink  # type: ignore[attr-defined]
    return ctx


@pytest.fixture
def context(tmp_path: Path) -> Callable[..., Context]:
    def make(transport: Any = None, **tmdb: Any) -> Context:
        return make_context(tmp_path, transport, **tmdb)

    return make


@pytest.fixture
def key_file(tmp_path: Path) -> str:
    path = tmp_path / "tmdb.key"
    path.write_text(KEY)
    return str(path)


__all__ = ["API", "AS_OF", "EXPORTS", "KEY", "NOW", "FakeTransport", "NetError", "gz_lines"]
