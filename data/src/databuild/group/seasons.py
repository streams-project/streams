"""Seasonal pools: one per calendar event, split into sub groups that get their own names."""

from __future__ import annotations

import collections
import json
from dataclasses import dataclass

import numpy as np

from ..embed.backends import Embedder
from ..embed.quant import unit
from ..vocab import load
from .cluster import merge_small, partition
from .params import Params

CENTRAL_WORDS = (
    "lgbt",
    "homosexual",
    "lesbian",
    "queer",
    "transgender",
    "bisexual",
    "drag queen",
    "drag race",
)
TASK = "Given a description of a season or occasion, retrieve the films and series that fit it"


@dataclass(frozen=True)
class Event:
    name: str
    keywords: frozenset[str]
    central: bool
    titles: tuple[str, ...]


def calendar_events() -> list[Event]:
    by_event: dict[str, list[dict]] = collections.defaultdict(list)
    for entry in load("calendar")["entries"]:
        if (entry.get("kind") or entry.get("type")) == "calendar" and entry.get("event"):
            by_event[entry["event"]].append(entry)
    return [
        Event(
            name,
            frozenset(w.lower() for e in entries for w in e.get("keywords", [])),
            any(e.get("lgbtq_central") for e in entries),
            tuple(e["title"] for e in entries),
        )
        for name, entries in sorted(by_event.items())
    ]


def _central(words: set[str]) -> bool:
    return any(
        k == "gay"
        or k.startswith("gay ")
        or k == "coming out"
        or any(p in k for p in CENTRAL_WORDS)
        for k in words
    )


def keyword_pool(event: Event, keywords: dict[int, set[str]], quality: np.ndarray) -> list[int]:
    """Quality rows that carry the event's keywords."""
    pool = []
    for row in quality.tolist():
        words = keywords.get(row)
        if words and (_central(words) if event.central else words & event.keywords):
            pool.append(row)
    return pool


def season_rows(
    E: np.ndarray,
    keys: np.ndarray,
    popularity: np.ndarray,
    keyword_lists: dict[int, str],
    quality: np.ndarray,
    embedder: Embedder,
    dimensions: int,
    p: Params,
    method: str,
    seed: int,
) -> list[tuple[str, int, int, int, int]]:
    """Rows of event, sub group, key, rank and keyword flag."""
    keywords: dict[int, set[str]] = {}
    for row, text in keyword_lists.items():
        try:
            keywords[row] = {str(x).lower() for x in json.loads(text)}
        except ValueError:
            continue
    queries = load("seasons")["queries"]
    out: list[tuple[str, int, int, int, int]] = []
    Eq = E[quality] if len(quality) else E[:0]
    for event in calendar_events():
        members = keyword_pool(event, keywords, quality)
        score = dict.fromkeys(members, 1.0)
        phrase = queries.get(event.name)
        if phrase and len(quality):
            texts = [phrase, *event.titles[:6]]
            found = embedder.encode_queries(texts, TASK, 160)[:, :dimensions]
            q = unit(found.mean(0, keepdims=True).astype(np.float32))[0]
            fits = Eq @ q
            position = {int(r): i for i, r in enumerate(quality.tolist())}
            floor = (
                float(np.percentile([fits[position[r]] for r in members], 10)) if members else -1.0
            )
            target = int(min(p.season_pool_ceiling, max(p.season_pool_floor, 2 * len(members))))
            have = set(members)
            for i in np.argsort(-fits, kind="stable"):
                if len(members) >= target or fits[i] < floor:
                    break
                row = int(quality[i])
                if row not in have:
                    members.append(row)
                    score[row] = float(fits[i])
                    have.add(row)
        if not members:
            continue
        rows = np.array(members, dtype=np.int64)
        if len(rows) < p.season_whole_below:
            labels = np.zeros(len(rows), np.int32)
        else:
            X = E[rows]
            target_groups = int(np.clip(len(rows) // p.season_size, 2, 40))
            labels = merge_small(X, partition(X, target_groups, method, seed), p.min_season)
        groups: dict[int, list[int]] = collections.defaultdict(list)
        for row, label in zip(rows.tolist(), labels.tolist(), strict=True):
            groups[label].append(row)
        for sub, group in sorted(groups.items()):
            group.sort(key=lambda r: (-score[r], -float(popularity[r]), r))
            for rank, row in enumerate(group):
                out.append((event.name, sub, int(keys[row]), rank, int(score[row] >= 0.999)))
    return out
