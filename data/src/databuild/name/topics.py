"""Pages and topics the facts alone cannot fill. A text model judges each candidate title.

Candidates are the titles a rule finds, plus the nearest titles to their centre. Only accepted pairs
are kept, in `categories.json` and `topics.json`, as `{slug: {"movie:12": 1.0}}`.
"""

from __future__ import annotations

import json
import re
import sqlite3
from typing import Any

import numpy as np

from ..context import Context
from ..embed.quant import restore, unit
from ..embed.run import read_master
from ..keys import title_key, unpack
from ..vocab import load
from .llm import Chat

MIN_SEEDS_FOR_CENTRE = 15
MIN_VOTES = 20
MIN_RATING = 5.0
PER_TOPIC = 3000
SYSTEM = "You judge whether a film or series belongs to a named category. Answer yes or no."


def category_rule(spec: dict[str, Any], row: dict[str, Any]) -> bool:
    keywords, title, genres = row["keywords"], row["title"].lower(), row["genres"]
    has = any(p in k for k in keywords for p in spec["parts"])
    if spec.get("title_parts") and any(p in title for p in spec["title_parts"]):
        return True
    if spec.get("genres_or") and genres & set(spec["genres_or"]):
        return True
    needs = spec.get("needs_genres")
    if needs and not genres & set(needs):
        word = spec.get("or_keyword")
        return has and bool(word) and any(word in k for k in keywords)
    return has


def topic_rule(spec: dict[str, Any], row: dict[str, Any]) -> bool:
    countries = set(spec.get("countries", {}))
    if countries:
        return bool(countries & row["countries"])
    if spec["genres_any"] and not set(spec["genres_any"]) & row["genres"]:
        return False
    text = " ".join(sorted(row["keywords"])) + " " + row["title"].lower() + " "
    text += row["overview"].lower()
    return any(re.search(rf"\b{re.escape(t)}\b", text) for t in spec["terms"])


def load_pool(ctx: Context) -> dict[str, dict[str, Any]]:
    db = sqlite3.connect(f"file:{ctx.work / 'titles.db'}?mode=ro", uri=True)
    pool = {}
    for kind, tid, title, genres, kw, countries, overview, votes, rating in db.execute(
        "SELECT media_type, tmdb_id, title, genres, keywords, origin_country, overview,"
        " vote_count, vote_average FROM titles WHERE adult = 0 AND poster_path IS NOT NULL"
    ):
        try:
            words = {str(k).lower() for k in json.loads(kw)} if kw else set()
            held = set(json.loads(countries)) if countries else set()
        except ValueError:
            words, held = set(), set()
        pool[title_key(kind, tid)] = {
            "kind": kind,
            "title": title or "",
            "genres": {g.strip() for g in (genres or "").split(",") if g.strip()},
            "keywords": words,
            "countries": held,
            "overview": (overview or "")[:420],
            "votes": votes or 0,
            "rating": rating or 0,
        }
    db.close()
    return pool


def proposals(
    specs: dict[str, dict[str, Any]],
    rule: Any,
    pool: dict[str, dict[str, Any]],
    keys: list[str],
    vectors: np.ndarray,
) -> dict[str, set[str]]:
    position = {k: i for i, k in enumerate(keys)}
    out: dict[str, set[str]] = {}
    for slug, spec in specs.items():
        media = set(spec["media"])
        seeds = [k for k, r in pool.items() if r["kind"] in media and rule(spec, r)]
        held = [k for k in seeds if k in position]
        held.sort(key=lambda k: (-pool[k]["votes"], k))
        held = held[:400]
        found = set(seeds)
        if len(held) >= MIN_SEEDS_FOR_CENTRE:
            centre = unit(vectors[[position[k] for k in held]].mean(0, keepdims=True))[0]
            sims = vectors @ centre
            allowed = np.array([pool[k]["kind"] in media for k in keys])
            sims[~allowed] = -1.0
            found |= {keys[i] for i in np.argsort(-sims, kind="stable")[:PER_TOPIC]}
        out[slug] = found
    return out


def judge(
    chat: Chat, specs: dict[str, dict[str, Any]], found: dict[str, set[str]], pool: dict
) -> dict[str, dict[str, float]]:
    accepted: dict[str, dict[str, float]] = {}
    for slug, keys in found.items():
        definition = specs[slug]["definition"]
        for key in sorted(keys):
            row = pool[key]
            text = (
                f"Category: {definition}\nTitle: {row['title']}\n"
                f"Genres: {', '.join(sorted(row['genres']))}\n"
                f"Keywords: {', '.join(sorted(row['keywords'])[:14])}\n"
                f"Overview: {row['overview']}\nDoes this title belong? Answer yes or no."
            )
            if chat.ask(SYSTEM, text, max_tokens=2).strip().lower().startswith("yes"):
                accepted.setdefault(slug, {})[key] = 1.0
    return accepted


def write_topics(ctx: Context, chat: Chat | None) -> None:
    if chat is None:
        ctx.say("name: topics off, name.endpoint is not set; no judged pages or topic rows")
        return
    rows, packed, _ = read_master(ctx.work / "vectors")
    vectors = restore(rows)
    centred = unit(vectors - vectors.mean(0))
    pool = load_pool(ctx)
    keys = [title_key(*unpack(int(k))) for k in packed.tolist()]
    inside = [
        i
        for i, k in enumerate(keys)
        if k in pool and pool[k]["votes"] >= MIN_VOTES and pool[k]["rating"] >= MIN_RATING
    ]
    keys = [keys[i] for i in inside]
    centred = centred[inside]
    folder = ctx.work / "export"
    folder.mkdir(exist_ok=True)
    categories = load("categories")
    topics = load("topics")
    done_categories = judge(
        chat, categories, proposals(categories, category_rule, pool, keys, centred), pool
    )
    done_topics = judge(chat, topics, proposals(topics, topic_rule, pool, keys, centred), pool)
    (folder / "categories.json").write_text(json.dumps(done_categories))
    (folder / "topics.json").write_text(json.dumps(done_topics))
    meta = {
        slug: {"name": t["name"], "media": t["media"], "definition": t["definition"]}
        for slug, t in topics.items()
    }
    (folder / "topics_meta.json").write_text(json.dumps(meta, ensure_ascii=False))
    ctx.say(f"name: {len(done_categories)} judged pages, {len(done_topics)} topics")
