"""Ends a job with a failure when a set is too old, so an alarm can fire (H62, H99).

Green checks hid a frozen popularity file for weeks. This reads the dates inside the files.
"""

from __future__ import annotations

import json
import sqlite3
from datetime import UTC, datetime
from pathlib import Path

from ..context import Context
from ..errors import StageFailed
from ..export.files import BY_ROLE, MANIFEST_FILE

MS_PER_HOUR = 3_600_000


def _hours(then: datetime, now: datetime) -> float:
    return (now - then).total_seconds() / 3600


def _parse(text: str) -> datetime:
    moment = datetime.fromisoformat(text)
    return moment if moment.tzinfo else moment.replace(tzinfo=UTC)


def set_age_hours(folder: Path, now: datetime) -> float:
    path = folder / MANIFEST_FILE
    if not path.exists():
        raise StageFailed("age: no manifest in the folder")
    built = json.loads(path.read_text(encoding="utf-8"))["built_at_ms"]
    return (now.timestamp() * 1000 - built) / MS_PER_HOUR


def popularity_age_hours(folder: Path, now: datetime) -> float | None:
    path = folder / BY_ROLE["popularity"].path
    if not path.exists():
        return None
    db = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
    try:
        written = dict(db.execute("SELECT k, v FROM meta"))["written"]
    finally:
        db.close()
    return _hours(_parse(written), now)


def fresh_age_hours(folder: Path, now: datetime) -> float | None:
    path = folder / BY_ROLE["fresh"].path
    if not path.exists():
        return None
    return _hours(_parse(json.loads(path.read_text(encoding="utf-8"))["generated"]), now)


def check_age(ctx: Context) -> None:
    """Says each age and fails on the first past its limit. A set with no such file passes it."""
    limits = ctx.settings.age
    now = ctx.now()
    problems = []
    age = set_age_hours(ctx.out, now)
    ctx.say(f"age: set {age:.1f} hours")
    if age > limits.set_days * 24:
        problems.append(f"the set is {age / 24:.1f} days old, limit {limits.set_days}")
    popularity = popularity_age_hours(ctx.out, now)
    if popularity is not None:
        ctx.say(f"age: popularity {popularity:.1f} hours")
        if popularity > limits.popularity_hours:
            problems.append(
                f"popularity is {popularity:.1f} hours old, limit {limits.popularity_hours}"
            )
    fresh = fresh_age_hours(ctx.out, now)
    if fresh is not None:
        ctx.say(f"age: fresh titles {fresh:.1f} hours")
        if fresh > limits.fresh_hours:
            problems.append(f"fresh titles are {fresh:.1f} hours old, limit {limits.fresh_hours}")
    if problems:
        raise StageFailed("age: " + "; ".join(problems))
