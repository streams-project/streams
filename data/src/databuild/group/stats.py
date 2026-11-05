"""What is measurably over represented in each group. Names are written from these facts."""

from __future__ import annotations

import collections
import json
import random
import statistics
from dataclasses import dataclass
from typing import Any

import numpy as np

EXAMPLES = 9
RANDOM_EXAMPLES = 6


@dataclass(frozen=True)
class Fact:
    title: str
    year: str
    kind: str
    language: str
    genres: tuple[str, ...]
    country: str
    popularity: float


def split_list(text: Any) -> list[str]:
    cleaned = str(text or "").replace("[", "").replace("]", "").replace('"', "")
    return [x.strip() for x in cleaned.split(",") if x.strip()]


def make_fact(title: Any, year: Any, kind: str, language: Any, genres: Any, country: Any, pop: Any):
    countries = split_list(country)
    return Fact(
        title or "",
        str(year or "")[:4],
        kind,
        language or "",
        tuple(split_list(genres)),
        countries[0] if countries else "",
        float(pop or 0),
    )


def _lift(counts: collections.Counter, whole: collections.Counter, size: int, total: int) -> list:
    items = [
        (k, c / size, (c / size) / (whole[k] / total))
        for k, c in counts.items()
        if c >= max(5, 0.03 * size)
    ]
    items.sort(key=lambda t: -t[2] * t[1] ** 0.5)
    return [(k, round(share, 2), round(lift, 1)) for k, share, lift in items[:6]]


def group_stats(members: list[Fact], whole: tuple, total: int, rng: random.Random) -> dict:
    """Shares, lifts, decades and examples of one group."""
    size = len(members)
    genres: collections.Counter = collections.Counter()
    countries: collections.Counter = collections.Counter()
    languages: collections.Counter = collections.Counter()
    decades: collections.Counter = collections.Counter()
    years = []
    for f in members:
        genres.update(set(f.genres))
        if f.country:
            countries[f.country] += 1
        if f.language:
            languages[f.language] += 1
        if f.year.isdigit():
            years.append(int(f.year))
            decades[int(f.year) // 10 * 10] += 1
    top = sorted(members, key=lambda f: (-f.popularity, f.title))[:EXAMPLES]
    pool = sorted(members, key=lambda f: f.title)
    chosen = rng.sample(pool, min(RANDOM_EXAMPLES, size))
    return {
        "genres": _lift(genres, whole[0], size, total),
        "countries": _lift(countries, whole[1], size, total)[:4],
        "languages": languages.most_common(3),
        "decades": decades.most_common(3),
        "tv_share": round(sum(f.kind == "tv" for f in members) / size, 2),
        "median_year": int(statistics.median(years)) if years else None,
        "examples": [f"{f.title} ({f.year})" for f in top],
        "random": [f"{f.title} ({f.year})" for f in chosen],
    }


def group_rows(
    facts: list[Fact],
    labels: np.ndarray,
    kind: str,
    level: int,
    parents: np.ndarray | None,
    confidence: np.ndarray,
    seeded: np.ndarray | None = None,
) -> list[tuple]:
    """Rows of the `groups` table for one level."""
    rng = random.Random(8)
    genres: collections.Counter = collections.Counter()
    countries: collections.Counter = collections.Counter()
    for f in facts:
        genres.update(set(f.genres))
        if f.country:
            countries[f.country] += 1
    whole = (genres, countries)
    rows = []
    members: dict[int, list[int]] = collections.defaultdict(list)
    for i, label in enumerate(labels):
        if label >= 0:
            members[int(label)].append(i)
    for gid, indices in sorted(members.items()):
        parent = None
        if parents is not None:
            parent = collections.Counter(int(parents[i]) for i in indices).most_common(1)[0][0]
        reviewed = float(np.mean(seeded[indices])) if seeded is not None else 0.0
        stats = group_stats([facts[i] for i in indices], whole, len(facts), rng)
        rows.append(
            (
                kind,
                level,
                gid,
                parent,
                len(indices),
                reviewed,
                float(np.mean(confidence[indices])),
                json.dumps(stats, ensure_ascii=False),
                None,
            )
        )
    return rows
