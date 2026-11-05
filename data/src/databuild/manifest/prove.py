"""Refuses a set that holds anything from a capture (H133).

Row definitions and vocabulary are the project's own words, so they are scanned word by word.
Title text is third party text and may name anything, so it is scanned for capture fields only.
"""

from __future__ import annotations

import json
import sqlite3
from pathlib import Path

from ..errors import StageFailed
from ..scan import scan_json_fields, scan_name, scan_text

OWN_WORDS = (
    "rows.json",
    "events.json",
    "taxonomy.json",
    "taxonomy_kids.json",
    "topics_meta.json",
    "search_aliases.json",
)
TAG_LIKE = '%"nf:%'


def _column_names(db: sqlite3.Connection, table: str) -> list[str]:
    return [row[1] for row in db.execute(f"PRAGMA table_info({table})")]


def prove_clean(folder: Path) -> None:
    for path in sorted(folder.rglob("*")):
        if path.is_file() and scan_name(path.name):
            raise StageFailed(f"manifest: refused file in the set: {path.name}")
    for path in sorted(folder.rglob("*.json")):
        try:
            document = json.loads(path.read_text(encoding="utf-8"))
        except ValueError as e:
            raise StageFailed(f"manifest: {path.name} is not JSON") from e
        found = next(scan_json_fields(document, path.name), None)
        if found:
            raise StageFailed(f"manifest: {path.name} holds a {found.what}")
    for name in OWN_WORDS:
        path = folder / name
        if path.exists():
            found = next(scan_text(path.read_text(encoding="utf-8"), name), None)
            if found and found.what == "refused word":
                raise StageFailed(f"manifest: {name} holds a refused word")
    catalogue = folder / "catalog.db"
    if catalogue.exists():
        db = sqlite3.connect(f"file:{catalogue}?mode=ro", uri=True)
        try:
            if any(scan_name(column) for column in _column_names(db, "titles")):
                raise StageFailed("manifest: the catalogue has a column from a refused input")
            tagged = db.execute(
                "SELECT count(*) FROM titles WHERE keywords LIKE ?", (TAG_LIKE,)
            ).fetchone()[0]
        finally:
            db.close()
        if tagged:
            raise StageFailed("manifest: the catalogue holds capture tags")
