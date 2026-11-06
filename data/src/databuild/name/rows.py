"""Row definitions from the project's own vocabulary.

A row is a name and fact rules. Names come from fixed templates over `vocabulary/rows.json`, never
from examples. A row is kept when the catalogue holds enough quality titles for it and its best
titles sit close together in the vectors (S106 falls back to keywords through these rules).
"""

from __future__ import annotations

import collections
import json
import re
import sqlite3
from pathlib import Path
from typing import Any

import numpy as np

from ..clean.rule import rule_sql
from ..context import Context
from ..embed.quant import restore, unit
from ..embed.run import read_master
from ..errors import StageFailed
from ..vocab import load

ROWS_FILE = "rows.json"
ROWS_VERSION = 3
MIN_SUPPORT = 30
TOP = 30
FULL_SIZE = 20000


def stem(word: str) -> str:
    return word.lower().rstrip("s").replace("ie", "y")


def repeats(phrase: str, noun: str) -> bool:
    """True when the phrase already holds a word of the noun."""
    words = {stem(w) for w in phrase.split()}
    return any(stem(w) in words for w in noun.split())


def slug(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", text.lower()).strip("_")


def decade_label(decade: int) -> str:
    return f"{decade}s" if decade >= 2000 else f"{decade % 100}s"


def _year(value: Any) -> int | None:
    try:
        return int(str(value)[:4])
    except (ValueError, TypeError):
        return None


def _json_set(value: Any, lower: bool = False) -> set[str]:
    try:
        items = json.loads(value) if value else []
    except ValueError:
        return set()
    return {str(x).lower() if lower else str(x) for x in items}


def passes(t: dict[str, Any], r: dict[str, Any]) -> bool:
    """Whether a title carries every fact a rule asks for."""
    if r.get("genres_any") and not t["genres"] & set(r["genres_any"]):
        return False
    if r.get("genres_none") and t["genres"] & set(r["genres_none"]):
        return False
    if r.get("keywords_any") and not t["keywords"] & set(r["keywords_any"]):
        return False
    if "year_min" in r and (t["year"] is None or t["year"] < r["year_min"]):
        return False
    if "year_max" in r and (t["year"] is None or t["year"] > r["year_max"]):
        return False
    if r.get("languages_any") and t["language"] not in r["languages_any"]:
        return False
    if r.get("countries_any") and not t["countries"] & set(r["countries_any"]):
        return False
    return not ("anime" in r and t["anime"] != r["anime"])


def load_titles(titles_db: Path, where: str) -> dict[int, dict[str, Any]]:
    db = sqlite3.connect(f"file:{titles_db}?mode=ro", uri=True)
    held: dict[int, dict[str, Any]] = {}
    for row in db.execute(
        "SELECT id, media_type, genres, keywords, release_year, original_language, origin_country,"
        " vote_average, vote_count, popularity, poster_path, backdrop_path, length(overview),"
        f" is_anime FROM titles WHERE {where}"
    ):
        key, kind, genres, kw, year, language, country, rating, votes, pop = row[:10]
        poster, backdrop, text, anime = row[10:]
        held[key] = {
            "kind": kind,
            "genres": {g.strip() for g in (genres or "").split(",") if g.strip()},
            "keywords": _json_set(kw, lower=True),
            "year": _year(year),
            "language": language or "",
            "countries": _json_set(country),
            "anime": bool(anime),
            "quality": (rating or 0) >= 6.0
            and (pop or 0) >= 2.0
            and (text or 0) >= 60
            and bool(poster)
            and bool(backdrop)
            and ((votes or 0) == 0 or (votes or 0) >= 10),
        }
    db.close()
    return held


class Builder:
    def __init__(
        self,
        titles: dict[int, dict[str, Any]],
        keys: np.ndarray,
        vectors: np.ndarray,
        min_support: int,
        top: int,
    ) -> None:
        self.titles = titles
        self.keys = keys
        self.vectors = vectors
        self.min_support = min_support
        self.top = top
        self.row_of = {int(k): i for i, k in enumerate(keys.tolist())}
        self.quality = [k for k, t in titles.items() if t["quality"] and k in self.row_of]
        self.found: list[dict[str, Any]] = []

    def measure(self, rules: dict[str, Any]) -> dict[str, Any] | None:
        members = [k for k in self.quality if passes(self.titles[k], rules)]
        if len(members) < self.min_support:
            return None
        m = self.vectors[[self.row_of[k] for k in members]]
        anchor = unit(m.mean(axis=0, keepdims=True))[0]
        fit = m @ anchor
        best = np.argsort(-fit, kind="stable")[: self.top]
        sub = m[best]
        size = len(sub)
        sims = sub @ sub.T
        coherence = float((sims.sum() - size) / (size * (size - 1)))
        share = sum(1 for k in members if self.titles[k]["kind"] == "movie") / len(members)
        counts = collections.Counter(g for k in members for g in self.titles[k]["genres"])
        return {
            "support": len(members),
            "movie_share": round(share, 3),
            "coherence": round(coherence, 3),
            "genre_share": {g: round(c / len(members), 3) for g, c in counts.most_common(6)},
        }

    def add(
        self,
        row_id: str,
        name: str,
        kind: str,
        rules: dict[str, Any],
        season: dict[str, str] | None = None,
    ) -> None:
        facts = self.measure(rules)
        if facts is None:
            return
        row: dict[str, Any] = {"id": row_id, "name": name, "kind": kind, "rules": rules, **facts}
        if season:
            row["season"] = season
        self.found.append(row)


def genre_names(genre: str, tv_names: dict[str, str]) -> list[str]:
    """A row about a genre accepts the film name and the series name."""
    return sorted({genre, tv_names.get(genre, genre)})


def candidates(b: Builder, vocab: dict[str, Any], events: dict[str, Any]) -> None:
    genres, nouns = vocab["genres"], vocab["nouns"]
    tv_names = vocab["tv_genre_names"]
    noun_genres = set(vocab["theme_noun_genres"])
    plain_phrases = set(vocab["genre_word_phrases"])
    for phrase, theme in vocab["themes"].items():
        keywords = sorted(theme["keywords"])
        if theme["standalone"]:
            b.add(f"theme_{slug(phrase)}", phrase, "theme", {"keywords_any": keywords})
            continue
        if phrase.lower() in plain_phrases:
            continue
        for g in genres:
            if g not in nouns or g not in noun_genres or repeats(phrase, nouns[g]):
                continue
            rules = {"keywords_any": keywords, "genres_any": genre_names(g, tv_names)}
            b.add(f"theme_{slug(phrase)}_{slug(g)}", f"{phrase} {nouns[g]}", "theme", rules)
    for extra in vocab["extra"]:
        rules = {"keywords_any": sorted(extra["keywords"])}
        if extra["genre"]:
            rules["genres_any"] = genre_names(extra["genre"], tv_names)
        b.add(f"{extra['kind']}_{slug(extra['name'])}", extra["name"], extra["kind"], rules)
    for decade in vocab["decades"]:
        for g in genres:
            if g in nouns:
                rules = {
                    "year_min": decade,
                    "year_max": decade + 9,
                    "genres_any": genre_names(g, tv_names),
                }
                name = f"{decade_label(decade)} {nouns[g]}"
                b.add(f"era_{decade}_{slug(g)}", name, "era", rules)
    for origin in vocab["origins"]:
        adjective = origin["adjective"]
        for g in genres:
            if g not in nouns:
                continue
            rules = {"genres_any": genre_names(g, tv_names), **origin["rules"]}
            if origin["languages"]:
                rules["languages_any"] = origin["languages"]
            if origin["countries"]:
                rules["countries_any"] = origin["countries"]
            b.add(f"origin_{slug(adjective)}_{slug(g)}", f"{adjective} {nouns[g]}", "origin", rules)
    b.add("origin_anime", "Anime", "origin", {"anime": True})
    for event, info in events.items():
        words = sorted(info["keywords"])
        if not words:
            continue
        season = info["season"]
        label = info["label"]
        b.add(
            f"calendar_{event}", f"{label} Favourites", "calendar", {"keywords_any": words}, season
        )
        for g in genres:
            if g in nouns and g in info["genres"]:
                rules = {"keywords_any": words, "genres_any": genre_names(g, tv_names)}
                name = f"{label} {nouns[g]}"
                b.add(f"calendar_{event}_{slug(g)}", name, "calendar", rules, season)


def coherence_floor(b: Builder) -> float:
    """Twice what random sets reach, or the tenth percentile of the candidates."""
    held = [c["coherence"] for c in b.found]
    rng = np.random.default_rng(5)
    random_sets = []
    size = min(b.top, len(b.quality))
    if size >= 2:
        for _ in range(200):
            picked = rng.choice(b.quality, size, replace=False)
            sub = b.vectors[[b.row_of[int(k)] for k in picked]]
            s = sub @ sub.T
            random_sets.append((s.sum() - size) / (size * (size - 1)))
    high = float(np.percentile(random_sets, 99)) if random_sets else 0.0
    return round(max(high * 2, float(np.percentile(held, 10))), 3)


def build_rows(
    titles: dict[int, dict[str, Any]],
    keys: np.ndarray,
    vectors: np.ndarray,
    events: dict[str, Any],
    min_support: int = MIN_SUPPORT,
    top: int = TOP,
) -> dict[str, Any]:
    centred = vectors - vectors.mean(axis=0)
    builder = Builder(titles, keys, unit(centred), min_support, top)
    candidates(builder, load("rows"), events)
    if not builder.found:
        raise StageFailed("name: no row definition has enough titles")
    floor = coherence_floor(builder)
    kept: list[dict[str, Any]] = []
    names: set[str] = set()
    for row in sorted(builder.found, key=lambda r: (-r["support"], r["id"])):
        if row["coherence"] < floor or row["name"].lower() in names:
            continue
        names.add(row["name"].lower())
        share = row["movie_share"]
        row["page"] = "films" if share >= 0.7 else "series" if share <= 0.3 else "home"
        row["format"] = {"films": "movie", "series": "tv", "home": "both"}[row["page"]]
        kept.append(row)
    kept.sort(key=lambda r: (r["kind"], r["name"]))
    return {
        "version": ROWS_VERSION,
        "min_support": min_support,
        "min_coherence": floor,
        "requests": kept,
    }


def write_rows(ctx: Context, events: dict[str, Any]) -> None:
    rows, keys, _ = read_master(ctx.work / "vectors")
    vectors = restore(rows)
    where = rule_sql(ctx.as_of, ctx.votes)
    titles = load_titles(ctx.work / "titles.db", where)
    small = len(keys) < FULL_SIZE
    support = max(4, len(keys) // 100) if small else MIN_SUPPORT
    top = min(TOP, support)
    document = build_rows(titles, keys, vectors, events, support, top)
    target = ctx.work / "export" / ROWS_FILE
    target.parent.mkdir(exist_ok=True)
    target.write_text(json.dumps(document, indent=1, ensure_ascii=False) + "\n")
    by_kind = collections.Counter(r["kind"] for r in document["requests"])
    ctx.say(f"name: {len(document['requests'])} rows kept, {dict(sorted(by_kind.items()))}")
