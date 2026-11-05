"""Turns the pull file into `titles.db`. One row per title, keyed by the packed key.

Rows older than the refresh limit are left out, so a stale copy of TMDB text never reaches a build.
"""

from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from ..context import Context
from ..errors import StageFailed
from ..keys import item_key
from ..pull.details import refresh_cutoff

MAX_CAST_STORED = 8

TITLES_SCHEMA = """
CREATE TABLE titles (
  id INTEGER PRIMARY KEY, tmdb_id INTEGER NOT NULL, media_type TEXT NOT NULL,
  title TEXT, release_year TEXT, release_date TEXT, director TEXT, cast_members TEXT, tagline TEXT,
  genres TEXT, overview TEXT, poster_path TEXT, backdrop_path TEXT, logo_path TEXT,
  vote_average REAL, vote_count INTEGER, popularity REAL, is_anime INTEGER,
  original_language TEXT, origin_country TEXT, certs TEXT, keywords TEXT, tv_type TEXT,
  runtime INTEGER, adult INTEGER, status TEXT, imdb_id TEXT, networks TEXT, network_logos TEXT,
  production_companies TEXT, company_logos TEXT, creators TEXT, crew TEXT, videos TEXT,
  translations TEXT, collection TEXT, seasons INTEGER, episodes INTEGER, source TEXT,
  fetched_at INTEGER);
CREATE UNIQUE INDEX idx_titles_key ON titles(media_type, tmdb_id);
CREATE INDEX idx_titles_popularity ON titles(popularity DESC);
CREATE TABLE seasons (
  media_type TEXT, tmdb_id INTEGER, season_number INTEGER, name TEXT, overview TEXT, air_date TEXT,
  poster_path TEXT, episode_count INTEGER, PRIMARY KEY (media_type, tmdb_id, season_number));
CREATE TABLE episodes (
  media_type TEXT, tmdb_id INTEGER, season_number INTEGER, episode_number INTEGER, name TEXT,
  overview TEXT, air_date TEXT, still_path TEXT, runtime INTEGER, vote_average REAL,
  PRIMARY KEY (media_type, tmdb_id, season_number, episode_number));
"""

TITLE_COLUMNS = (
    "id tmdb_id media_type title release_year release_date director cast_members tagline genres "
    "overview poster_path backdrop_path logo_path vote_average vote_count popularity is_anime "
    "original_language origin_country certs keywords tv_type runtime adult status imdb_id "
    "networks network_logos production_companies company_logos creators crew videos translations "
    "collection seasons episodes source fetched_at"
).split()


def json_list(text: str | None) -> list[Any]:
    try:
        value = json.loads(text) if text else []
    except ValueError:
        return []
    return value if isinstance(value, list) else []


def is_anime(genres: str, countries: list[str], language: str | None, keywords: list[str]) -> bool:
    """Animation made in Japan, in Japanese, or tagged anime or manga."""
    if "Animation" not in genres:
        return False
    tagged = any(k.lower() in ("anime", "manga") for k in keywords)
    return "JP" in countries or language == "ja" or tagged


def title_row(pulled: sqlite3.Row) -> dict[str, Any]:
    """One `titles` row from one pulled row."""
    kind = pulled["media_type"]
    genres = pulled["genres"] or ""
    countries = json_list(pulled["origin_country"])
    keywords = json_list(pulled["keywords"])
    people = json_list(pulled["directors"] if kind == "movie" else pulled["creators"])
    return {
        "id": item_key(kind, pulled["id"]),
        "tmdb_id": pulled["id"],
        "media_type": kind,
        "title": pulled["title"],
        "release_year": (pulled["release_date"] or "")[:4],
        "release_date": pulled["release_date"] or "",
        "director": ", ".join(people),
        "cast_members": ", ".join(json_list(pulled["cast_members"])[:MAX_CAST_STORED]),
        "tagline": (pulled["tagline"] or "").strip(),
        "genres": genres,
        "overview": (pulled["overview"] or "").strip(),
        "poster_path": pulled["poster_path"],
        "backdrop_path": pulled["backdrop_path"],
        "logo_path": pulled["logo_path"],
        "vote_average": pulled["vote_average"],
        "vote_count": pulled["vote_count"],
        "popularity": pulled["popularity"],
        "is_anime": int(is_anime(genres, countries, pulled["original_language"], keywords)),
        "original_language": pulled["original_language"],
        "origin_country": pulled["origin_country"],
        "certs": pulled["certs"],
        "keywords": pulled["keywords"],
        "tv_type": pulled["tv_type"],
        "runtime": pulled["runtime"],
        "adult": pulled["adult"],
        "status": pulled["status"],
        "imdb_id": pulled["imdb_id"],
        "networks": pulled["networks"],
        "network_logos": pulled["network_logos"],
        "production_companies": pulled["production_companies"],
        "company_logos": pulled["company_logos"],
        "creators": pulled["creators"],
        "crew": pulled["crew"],
        "videos": pulled["videos"],
        "translations": pulled["translations"],
        "collection": pulled["collection"],
        "seasons": pulled["seasons"],
        "episodes": pulled["episodes"],
        "source": "tmdb",
        "fetched_at": pulled["fetched_at"],
    }


def open_titles(path: Path) -> sqlite3.Connection:
    """A new, empty titles file."""
    path.unlink(missing_ok=True)
    db = sqlite3.connect(path)
    db.executescript(TITLES_SCHEMA)
    return db


def insert_titles(db: sqlite3.Connection, rows: list[dict[str, Any]]) -> None:
    names = ",".join(TITLE_COLUMNS)
    marks = ",".join("?" * len(TITLE_COLUMNS))
    db.executemany(
        f"INSERT INTO titles ({names}) VALUES ({marks})",
        [[r.get(c) for c in TITLE_COLUMNS] for r in rows],
    )


def clean(ctx: Context) -> None:
    source = ctx.work / "pull.db"
    if not source.exists():
        raise StageFailed("clean: no pull file; run the pull stage or the sample builder")
    pull = sqlite3.connect(f"file:{source}?mode=ro", uri=True)
    pull.row_factory = sqlite3.Row
    cutoff = int(refresh_cutoff(ctx).timestamp())
    out = open_titles(ctx.work / "titles.db")
    kept, stale = 0, 0
    batch: list[dict[str, Any]] = []
    held: set[int] = set()
    for row in pull.execute("SELECT * FROM titles ORDER BY media_type, id"):
        if row["fetched_at"] < cutoff:
            stale += 1
            continue
        batch.append(title_row(row))
        kept += 1
        if row["media_type"] == "tv":
            held.add(row["id"])
        if len(batch) >= 50000:
            insert_titles(out, batch)
            batch = []
    insert_titles(out, batch)
    for row in pull.execute("SELECT * FROM seasons"):
        if row["id"] in held:
            out.execute(
                "INSERT INTO seasons VALUES ('tv',?,?,?,?,?,?,?)",
                (
                    row["id"],
                    row["season_number"],
                    row["name"],
                    row["overview"],
                    row["air_date"],
                    row["poster_path"],
                    row["episode_count"],
                ),
            )
    for row in pull.execute("SELECT * FROM episodes"):
        if row["id"] in held:
            out.execute(
                "INSERT INTO episodes VALUES ('tv',?,?,?,?,?,?,?,?,?)",
                (
                    row["id"],
                    row["season_number"],
                    row["episode_number"],
                    row["name"],
                    row["overview"],
                    row["air_date"],
                    row["still_path"],
                    row["runtime"],
                    row["vote_average"],
                ),
            )
    out.commit()
    out.close()
    pull.close()
    ctx.say(f"clean: {kept} titles kept, {stale} older than the refresh limit left out")
    if not kept:
        raise StageFailed("clean: no titles left")
