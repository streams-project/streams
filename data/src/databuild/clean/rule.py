"""What the catalogue may show (S133). The date arrives as input, so a built view never moves.

The view adds `title_id`, the public id. The packed `id` stays the internal key.
"""

from __future__ import annotations

import sqlite3
from datetime import date

BASE = (
    "NOT (adult = 1 OR poster_path IS NULL OR poster_path = ''"
    " OR overview IS NULL OR length(overview) < 60)"
)


def rule_sql(as_of: date, votes: bool = True) -> str:
    """The WHERE clause. Open sources hold no votes, so the open sample keeps the base clause."""
    if not votes:
        return BASE
    since = as_of.year - 1
    return (
        f"{BASE} AND ("
        f"(media_type = 'movie' AND (CAST(release_year AS INT) >= {since}"
        " OR (vote_average >= 6.5 AND vote_count >= 10) OR vote_count >= 100))"
        " OR "
        f"(media_type = 'tv' AND (CAST(release_year AS INT) >= {since}"
        " OR (vote_average >= 5.0 AND vote_count >= 10)))"
        ")"
    )


def make_catalog_view(db: sqlite3.Connection, as_of: date, votes: bool = True) -> None:
    """(Re)creates the view `catalog`, the only titles the server may show."""
    db.execute("DROP VIEW IF EXISTS catalog")
    where = rule_sql(as_of, votes)
    db.execute(f"CREATE VIEW catalog AS SELECT *, tmdb_id AS title_id FROM titles WHERE {where}")
    db.commit()
