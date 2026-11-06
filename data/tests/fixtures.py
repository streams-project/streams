"""Small generated data. No real title, no real text, no network."""

from __future__ import annotations

import json
import random
import sqlite3
from pathlib import Path

from databuild.clean.catalog import insert_titles, open_titles
from databuild.keys import item_key

GENRES = ["Drama", "Comedy", "Horror", "Science Fiction", "Romance", "Crime"]
TOPICS = {
    "Drama": "family grief memory house letters winter",
    "Comedy": "wedding neighbours chaos jokes picnic cake",
    "Horror": "ghost cellar scream midnight attic curse",
    "Science Fiction": "robot planet orbit signal engine colony",
    "Romance": "letters summer kiss train promise garden",
    "Crime": "detective heist alley witness safe gang",
}
KEYWORDS = {
    "Drama": ["family relationships"],
    "Comedy": ["wedding"],
    "Horror": ["ghost", "halloween"],
    "Science Fiction": ["robot", "space"],
    "Romance": ["first love"],
    "Crime": ["heist", "detective"],
}


def make_row(i: int, rng: random.Random, as_of_year: int = 2026) -> dict:
    kind = "tv" if i % 4 == 0 else "movie"
    genre = GENRES[i % len(GENRES)]
    words = TOPICS[genre].split()
    overview = (" ".join(rng.choices(words, k=16)) + f" story number {i}").capitalize()
    return {
        "id": item_key(kind, i),
        "tmdb_id": i,
        "media_type": kind,
        "title": f"Made Up {genre} {i}",
        "release_year": str(1970 + i % 55),
        "release_date": f"{1970 + i % 55}-01-01",
        "director": f"Person {i % 9}",
        "cast_members": f"Actor {i % 7}, Actor {i % 5}",
        "tagline": f"Line {genre}",
        "genres": f"{genre}, {GENRES[(i + 2) % len(GENRES)]}",
        "overview": overview,
        "poster_path": f"/p{i}.jpg",
        "backdrop_path": f"/b{i}.jpg",
        "vote_average": 6.0 + (i % 30) / 10,
        "vote_count": 50 + i,
        "popularity": 2.0 + (i % 50),
        "is_anime": 0,
        "original_language": "en",
        "origin_country": json.dumps(["US" if i % 3 else "GB"]),
        "certs": "{}",
        "keywords": json.dumps(KEYWORDS[genre]),
        "adult": 0,
        "source": "fixture",
        "fetched_at": 1_790_000_000,
    }


def make_titles(path: Path, count: int = 240, seed: int = 3) -> list[dict]:
    rng = random.Random(seed)
    rows = [make_row(i, rng) for i in range(1, count + 1)]
    db = open_titles(path)
    insert_titles(db, rows)
    db.commit()
    db.close()
    return rows


def rows_in(path: Path, sql: str) -> list[tuple]:
    db = sqlite3.connect(path)
    try:
        return db.execute(sql).fetchall()
    finally:
        db.close()
