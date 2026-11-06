"""Outside calls go through one small transport, so tests never touch a network.

An error text never holds an address or a query, because a query can hold a key.
"""

from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from typing import Any, Protocol


class NetError(Exception):
    """A call got no answer. The text names the kind of failure only."""


@dataclass(frozen=True)
class Request:
    base: str
    path: str = ""
    params: dict[str, Any] = field(default_factory=dict)
    headers: dict[str, str] = field(default_factory=dict)
    body: bytes | None = None
    timeout: float = 30.0

    def url(self) -> str:
        url = self.base.rstrip("/") + self.path
        return f"{url}?{urllib.parse.urlencode(self.params)}" if self.params else url


@dataclass(frozen=True)
class Reply:
    status: int
    body: bytes = b""
    headers: dict[str, str] = field(default_factory=dict)

    def json(self) -> Any:
        return json.loads(self.body)


class Transport(Protocol):
    def fetch(self, request: Request) -> Reply: ...


class UrlTransport:
    """The real transport. It is built only by the entry point."""

    def fetch(self, request: Request) -> Reply:
        method = "POST" if request.body is not None else "GET"
        req = urllib.request.Request(
            request.url(), data=request.body, headers=request.headers, method=method
        )
        try:
            with urllib.request.urlopen(req, timeout=request.timeout) as r:
                return Reply(r.status, r.read(), {k.lower(): v for k, v in r.headers.items()})
        except urllib.error.HTTPError as e:
            return Reply(e.code, e.read(), {k.lower(): v for k, v in e.headers.items()})
        except (urllib.error.URLError, TimeoutError, OSError) as e:
            raise NetError(type(e).__name__) from None


class NoTransport:
    """Refuses every call. Tests that must stay offline use it."""

    def fetch(self, request: Request) -> Reply:
        raise NetError("no network in this run")
