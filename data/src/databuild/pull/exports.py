"""TMDB's daily id exports: every film and series id with its popularity. They need no key."""

from __future__ import annotations

import gzip
import json
from collections.abc import Iterator
from dataclasses import dataclass
from datetime import date, timedelta
from typing import Any

from ..context import Context
from ..errors import StageFailed
from ..net import NetError, Request
from ..settings import need

DAYS_BACK = 5
FILES = {"movie": "movie_ids", "tv": "tv_series_ids"}


@dataclass(frozen=True)
class Export:
    kind: str
    day: date
    rows: list[dict[str, Any]]


def read_export(ctx: Context, kind: str) -> Export:
    """The newest export of the last five days. None of them means the job fails, loudly."""
    base = need(ctx.settings.tmdb.exports_base, "tmdb.exports_base")
    name = FILES[kind]
    for back in range(DAYS_BACK):
        day = ctx.as_of - timedelta(days=back)
        path = f"/{name}_{day:%m_%d_%Y}.json.gz"
        try:
            reply = ctx.transport.fetch(Request(base, path, timeout=120.0))
        except NetError:
            continue
        if reply.status != 200:
            continue
        lines = gzip.decompress(reply.body).decode().splitlines()
        return Export(kind, day, [json.loads(line) for line in lines if line])
    raise StageFailed(f"exports: no {name} file in the last {DAYS_BACK} days")


def wanted(export: Export, min_popularity: float) -> Iterator[tuple[float, str, int]]:
    """Titles at or above the popularity floor, adult ones left out."""
    for row in export.rows:
        if row["popularity"] >= min_popularity and not row.get("adult"):
            yield row["popularity"], export.kind, row["id"]
