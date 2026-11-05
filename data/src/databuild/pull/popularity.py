"""Daily popularity. Written next to the catalogue, whole or not at all.

The old job failed for weeks behind green checks (H62). This one has no fallback path: a missing
catalogue or a missing export ends the job with a failure, and the old file stays.
"""

from __future__ import annotations

import json
import os
import sqlite3

from ..context import Context
from ..errors import StageFailed
from .exports import read_export

POPULARITY_FILE = "popularity.db"
NEW_TITLE_FLOOR = 2.0


def pull_popularity(ctx: Context) -> None:
    catalogue = ctx.out / "catalog.db"
    if not catalogue.exists():
        raise StageFailed(f"popularity: no catalogue at {catalogue}")
    con = sqlite3.connect(f"file:{catalogue}?mode=ro", uri=True)
    have = set(con.execute("SELECT media_type, tmdb_id FROM titles"))
    con.close()
    exports = [read_export(ctx, kind) for kind in ("movie", "tv")]
    rows: list[tuple[str, int, float]] = []
    new: list[list] = []
    for export in exports:
        for r in export.rows:
            key = (export.kind, r["id"])
            if key in have:
                rows.append((*key, float(r["popularity"])))
            elif r["popularity"] >= NEW_TITLE_FLOOR and not r.get("adult"):
                new.append([*key, r["popularity"]])
    if not rows:
        raise StageFailed("popularity: the exports match no catalogue title, old file kept")
    asof = min(e.day for e in exports).isoformat()
    target = ctx.out / POPULARITY_FILE
    tmp = target.with_suffix(".db.tmp")
    tmp.unlink(missing_ok=True)
    out = sqlite3.connect(tmp)
    out.executescript(
        "CREATE TABLE pop (media_type TEXT, id INTEGER, popularity REAL,"
        " PRIMARY KEY (media_type, id)) WITHOUT ROWID;"
        "CREATE TABLE meta (k TEXT PRIMARY KEY, v TEXT);"
    )
    out.executemany("INSERT INTO pop VALUES (?,?,?)", sorted(rows))
    out.execute("INSERT INTO meta VALUES ('asof', ?)", (asof,))
    out.execute("INSERT INTO meta VALUES ('written', ?)", (ctx.now().isoformat("T", "seconds"),))
    out.commit()
    out.close()
    os.replace(tmp, target)
    (ctx.work / f"new_titles_{asof}.json").write_text(json.dumps(new))
    ctx.say(f"popularity: {len(rows)} titles as of {asof}, {len(new)} above the floor not held")
