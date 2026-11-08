"""Writes `catalog.db`: the titles, the groups, the view the server reads and the text index."""

from __future__ import annotations

import json
import os
import shutil
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np

from ..clean.rule import make_catalog_view
from ..context import Context
from ..group.run import SCHEMA as GROUP_SCHEMA

# Marks the file as ours. Reads "PDAT" as bytes.
APPLICATION_ID = 0x50444154
SCHEMA_VERSION = 1
PLACEMENT = ("w1", "w2", "atom", "th1", "th2", "th3", "vf", "vm")
PLACEMENT_REAL = ("wconf", "vconf", "vmconf")
PLACEMENT_FLAGS = ("wsrc", "vstate", "vcore", "vmcore")
GROUP_TABLES = (
    "groups",
    "group_centroids",
    "mood_vectors",
    "mood_pointers",
    "season_members",
    "season_groups",
)
SEARCH = """
CREATE VIRTUAL TABLE catalog_fts USING fts5(
  id UNINDEXED, title, overview, genres, director, tokenize = 'porter ascii');
INSERT INTO catalog_fts (id, title, overview, genres, director)
  SELECT id, title, overview, genres, director FROM catalog;
"""


TEXTS = """
CREATE TABLE title_texts (
  lang TEXT NOT NULL, media_type TEXT NOT NULL, tmdb_id INTEGER NOT NULL,
  title TEXT NOT NULL, overview TEXT NOT NULL, tagline TEXT NOT NULL,
  PRIMARY KEY (lang, media_type, tmdb_id)) WITHOUT ROWID;
"""
# The region read first when TMDB has several for a language: the one most of its viewers read.
PREFERRED = (
    "ar-SA",
    "ar-AE",
    "de-DE",
    "he-IL",
    "es-ES",
    "es-MX",
    "fr-FR",
    "fr-CA",
    "hi-IN",
    "it-IT",
    "ja-JP",
    "ko-KR",
    "nl-NL",
    "pl-PL",
    "pt-BR",
    "pt-PT",
    "ru-RU",
    "sv-SE",
    "tr-TR",
    "zh-CN",
    "zh-TW",
)


def pick_texts(held: dict[str, dict[str, str]]) -> dict[str, tuple[str, str, str]]:
    """One title, story and tagline per language: the preferred region first, each field filled
    from the next region when the first lacks it."""

    def rank(tag: str) -> tuple[int, str]:
        return (PREFERRED.index(tag) if tag in PREFERRED else len(PREFERRED), tag)

    by_lang: dict[str, list[str]] = {}
    for tag in sorted(held, key=rank):
        by_lang.setdefault(tag.split("-")[0].lower(), []).append(tag)
    out = {}
    for lang, tags in by_lang.items():

        def first(field: str, tags: list[str] = tags) -> str:
            return next(
                (
                    str(held[t].get(field) or "").strip()
                    for t in tags
                    if str(held[t].get(field) or "").strip()
                ),
                "",
            )

        text = (first("title"), first("overview"), first("tagline"))
        if any(text) and lang != "en":
            out[lang] = text
    return out


def write_texts(db: sqlite3.Connection) -> int:
    """Titles in other languages, one row per title and language,
    read by the server per language.
    """
    db.executescript(TEXTS)
    rows = []
    query = (
        "SELECT media_type, tmdb_id, translations FROM titles WHERE translations NOT IN ('', '{}')"
    )
    for media_type, tmdb_id, raw in db.execute(query):
        try:
            held = json.loads(raw or "{}")
        except ValueError:
            continue
        for lang, (title, overview, tagline) in pick_texts(held).items():
            rows.append((lang, media_type, tmdb_id, title, overview, tagline))
    db.executemany("INSERT OR REPLACE INTO title_texts VALUES (?, ?, ?, ?, ?, ?)", rows)
    return len(rows)


@dataclass(frozen=True)
class Identity:
    """What the file says about itself."""

    dataset_version: str
    as_of: str
    commit: str
    model: str


def add_placement(db: sqlite3.Connection, arrays: Any) -> None:
    """Group ids as columns of `titles`. A title with no group holds NULL."""
    columns = [
        (name, kind)
        for names, kind in (
            (PLACEMENT, "INTEGER"),
            (PLACEMENT_REAL, "REAL"),
            (PLACEMENT_FLAGS, "INTEGER"),
        )
        for name in names
        if name in arrays
    ]
    for name, kind in columns:
        db.execute(f"ALTER TABLE titles ADD COLUMN {name} {kind}")
    values = [arrays[name].tolist() for name, _ in columns]
    keys = arrays["keys"].tolist()
    labels = len([1 for name, _ in columns if name in PLACEMENT])
    rows = []
    for i, key in enumerate(keys):
        row: list[Any] = [v[i] for v in values]
        for c in range(labels):
            if row[c] < 0:
                row[c] = None
        rows.append((*row, key))
    sets = ", ".join(f"{name} = ?" for name, _ in columns)
    db.executemany(f"UPDATE titles SET {sets} WHERE id = ?", rows)
    db.execute("CREATE INDEX idx_titles_w2 ON titles(w2)")


def copy_groups(db: sqlite3.Connection, groups_db: Path, removed: list[str]) -> int:
    """Group tables, less the season sub groups the operator removed by name."""
    db.executescript(GROUP_SCHEMA)
    db.execute("ATTACH DATABASE ? AS built", (str(groups_db),))
    for table in GROUP_TABLES:
        db.execute(f"INSERT INTO {table} SELECT * FROM built.{table}")
    db.commit()
    db.execute("DETACH DATABASE built")
    gone = 0
    if removed:
        marks = ",".join("?" * len(removed))
        low = [name.lower() for name in removed]
        found = db.execute(
            f"SELECT event, sub FROM season_groups WHERE lower(name) IN ({marks})", low
        ).fetchall()
        for event, sub in found:
            db.execute("DELETE FROM season_members WHERE event = ? AND sub = ?", (event, sub))
            db.execute("DELETE FROM season_groups WHERE event = ? AND sub = ?", (event, sub))
        gone = len(found)
    return gone


def write_catalog(
    ctx: Context, identity: Identity, arrays: Any, removed: list[str]
) -> tuple[np.ndarray, int]:
    """Writes the file whole and returns the keys the view shows, with removed sub groups."""
    target = ctx.out / "catalog.db"
    tmp = ctx.out / "catalog.db.tmp"
    shutil.copyfile(ctx.work / "titles.db", tmp)
    db = sqlite3.connect(tmp)
    add_placement(db, arrays)
    gone = copy_groups(db, ctx.work / "groups.db", removed)
    make_catalog_view(db, ctx.as_of, ctx.votes)
    db.executescript(SEARCH)
    write_texts(db)
    db.execute("CREATE TABLE meta (k TEXT PRIMARY KEY, v TEXT)")
    db.executemany(
        "INSERT INTO meta VALUES (?, ?)",
        [
            ("schema_version", str(SCHEMA_VERSION)),
            ("dataset_version", identity.dataset_version),
            ("as_of", identity.as_of),
            ("build_commit", identity.commit),
            ("model", identity.model),
        ],
    )
    db.execute(f"PRAGMA application_id = {APPLICATION_ID}")
    db.execute(f"PRAGMA user_version = {SCHEMA_VERSION}")
    shown = np.array([k for (k,) in db.execute("SELECT id FROM catalog")], dtype="<u4")
    db.commit()
    db.execute("VACUUM")
    db.close()
    os.replace(tmp, target)
    return shown, gone
