"""Runs the group stage and writes `groups.npz` and `groups.db` into the work folder."""

from __future__ import annotations

import sqlite3
from pathlib import Path

import numpy as np

from ..context import Context
from ..embed.backends import make_embedder
from ..embed.quant import restore, unit
from ..embed.run import read_master
from ..errors import FeatureOff, StageFailed
from ..keys import unpack
from ..vocab import load
from .cluster import choose_method
from .params import Params
from .seasons import season_rows
from .stats import Fact, group_rows, make_fact
from .vibe import VibeLayer, build_vibe, format_bucket, mood_pointers, read_reviews
from .world import build_layers

MOOD_TASK = (
    "Given a mood or occasion, retrieve the films and series whose reviews and descriptions fit it"
)
MIN_MOOD_GROUP = 30
SCHEMA = """
CREATE TABLE groups (kind TEXT, level INTEGER, id INTEGER, parent INTEGER, size INTEGER,
  reviewed REAL, mean_conf REAL, stats TEXT, name TEXT, PRIMARY KEY (kind, level, id));
CREATE TABLE group_centroids (kind TEXT, level INTEGER, id INTEGER, n_reviewed INTEGER,
  dim INTEGER, data BLOB, PRIMARY KEY (kind, level, id));
CREATE TABLE mood_vectors (mood TEXT PRIMARY KEY, data BLOB);
CREATE TABLE mood_pointers (mood TEXT, title TEXT, late INTEGER, group_id INTEGER, score REAL,
  rank INTEGER);
CREATE TABLE season_members (event TEXT, sub INTEGER, media_type TEXT, tmdb_id INTEGER,
  rank INTEGER, kw INTEGER);
CREATE TABLE season_groups (event TEXT, sub INTEGER, name TEXT);
"""


def _facts(titles_db: Path, keys: np.ndarray) -> dict:
    db = sqlite3.connect(f"file:{titles_db}?mode=ro", uri=True)
    rows = {
        r[0]: r
        for r in db.execute(
            "SELECT id, title, release_year, media_type, original_language, genres, origin_country,"
            " popularity, length(overview), runtime, keywords, vote_average, poster_path,"
            " backdrop_path, vote_count FROM titles"
        )
    }
    db.close()
    n = len(keys)
    facts: list[Fact] = []
    text_length = np.zeros(n, np.int32)
    buckets = np.zeros(n, np.int8)
    popularity = np.zeros(n, np.float32)
    keyword_lists: dict[int, str] = {}
    quality: list[int] = []
    for i, key in enumerate(keys.tolist()):
        r = rows.get(key)
        if r is None:
            raise StageFailed("group: the vectors hold a title the titles file does not")
        facts.append(make_fact(r[1], r[2], r[3], r[4], r[5], r[6], r[7]))
        text_length[i] = r[8] or 0
        buckets[i] = format_bucket(r[3], r[9])
        popularity[i] = r[7] or 0
        if r[10] and r[10] not in ("", "[]"):
            keyword_lists[i] = r[10]
        if (r[11] or 0) >= 6.0 and (r[7] or 0) >= 2.0 and (r[8] or 0) >= 60 and r[12] and r[13]:
            quality.append(i)
    return {
        "facts": facts,
        "text_length": text_length,
        "buckets": buckets,
        "popularity": popularity,
        "keywords": keyword_lists,
        "quality": np.array(quality, dtype=np.int64),
    }


def _store_vibe(db: sqlite3.Connection, facts: list[Fact], vibe: VibeLayer, dim: int) -> None:
    for level, labels, core, fit, centres, seen, parents in (
        (1, vibe.fine, vibe.fine_core, vibe.fine_fit, vibe.fine_centres, vibe.fine_seen, vibe.mid),
        (2, vibe.mid, vibe.mid_core, vibe.mid_fit, vibe.mid_centres, vibe.mid_seen, None),
    ):
        shown = np.where(core, labels, -1)
        rows = group_rows(facts, shown, "vibe", level, parents, fit, vibe.seeded)
        db.executemany("INSERT INTO groups VALUES (?,?,?,?,?,?,?,?,?)", rows)
        db.executemany(
            "INSERT INTO group_centroids VALUES ('vibe',?,?,?,?,?)",
            [
                (level, gid, seen[gid], dim, centres[gid].astype("<f2").tobytes())
                for gid in sorted(centres)
            ],
        )


def group(ctx: Context) -> None:
    rows, keys, info = read_master(ctx.work / "vectors")
    E = restore(rows)
    dimensions = info["dimensions"]
    gathered = _facts(ctx.work / "titles.db", keys)
    facts = gathered["facts"]
    p = Params.for_titles(len(keys))
    method = choose_method(ctx.settings.group.method, ctx.say)
    seed = ctx.settings.group.seed

    layers = build_layers(E, gathered["text_length"], p, method, seed, ctx.say)

    path = ctx.work / "groups.db"
    path.unlink(missing_ok=True)
    db = sqlite3.connect(path)
    db.executescript(SCHEMA)
    insert = "INSERT INTO groups VALUES (?,?,?,?,?,?,?,?,?)"
    specs = [
        ("world", 1, layers.w1, None),
        ("world", 2, layers.w2, layers.w1),
        ("atom", 1, layers.atom, layers.w2),
        ("theme", 1, layers.th1, None),
        ("theme", 2, layers.th2, layers.th1),
        ("theme", 3, layers.th3, layers.th2),
    ]
    for kind, level, labels, parents in specs:
        db.executemany(insert, group_rows(facts, labels, kind, level, parents, layers.wconf))

    arrays = {
        "keys": keys,
        "w1": layers.w1,
        "w2": layers.w2,
        "wconf": layers.wconf,
        "wsrc": layers.wsrc,
        "atom": layers.atom,
        "th1": layers.th1,
        "th2": layers.th2,
        "th3": layers.th3,
    }

    embedder = make_embedder(ctx)
    moods = load("moods")["moods"]
    phrases = embedder.encode_queries([m["phrase"] for m in moods], MOOD_TASK, 160)
    phrases = unit(phrases[:, :dimensions].astype(np.float32))
    db.executemany(
        "INSERT INTO mood_vectors VALUES (?,?)",
        [(m["id"], v.astype("<f4").tobytes()) for m, v in zip(moods, phrases, strict=True)],
    )

    reviews = ctx.settings.reviews
    try:
        if not reviews.vectors or not reviews.keys:
            raise FeatureOff("reviews.vectors and reviews.keys are not set")
        review_rows, review_keys = read_reviews(reviews.vectors, reviews.keys, dimensions)
    except FeatureOff as off:
        ctx.say(f"group: vibe layer off, {off}; no vibe groups and no mood pointers in this set")
    else:
        vibe = build_vibe(E, keys, gathered["buckets"], review_rows, review_keys, p, method, seed)
        _store_vibe(db, facts, vibe, dimensions)
        pointers = mood_pointers(
            moods,
            phrases,
            review_rows,
            vibe.fine_centres,
            vibe.fine_seen,
            min(MIN_MOOD_GROUP, p.min_sub),
        )
        db.executemany("INSERT INTO mood_pointers VALUES (?,?,?,?,?,?)", pointers)
        arrays.update(
            vf=vibe.fine,
            vm=vibe.mid,
            vconf=vibe.fine_fit,
            vmconf=vibe.mid_fit,
            vstate=(~vibe.seeded).astype(np.int8),
            vcore=vibe.fine_core.astype(np.int8),
            vmcore=vibe.mid_core.astype(np.int8),
        )
        ctx.say(f"group: vibe layer from {int(vibe.seeded.sum())} reviewed titles")

    seasons = season_rows(
        E,
        keys,
        gathered["popularity"],
        gathered["keywords"],
        gathered["quality"],
        embedder,
        dimensions,
        p,
        method,
        seed,
    )
    db.executemany(
        "INSERT INTO season_members VALUES (?,?,?,?,?,?)",
        [(event, sub, *unpack(key), rank, kw) for event, sub, key, rank, kw in seasons],
    )
    db.commit()
    db.close()
    np.savez(ctx.work / "groups.npz", method=np.array(method), **arrays)
    ctx.say(f"group: {len({(s[0], s[1]) for s in seasons})} season sub groups")
