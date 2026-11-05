"""The vibe layer, made from review vectors. It is an optional input.

Reviewed titles make the groups, in a space centred per format so features, shorts and series are
compared on feel. Other titles are placed by a vote of their nearest reviewed titles.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from ..embed.quant import restore, row_type, unit
from ..errors import StageFailed
from .cluster import Neighbours, merge_small, partition, place
from .params import Params

CORE_CONFIDENCE = 0.3
MIN_BUCKET = 30
SHORT_MINUTES = 40


@dataclass
class VibeLayer:
    fine: np.ndarray
    mid: np.ndarray
    fine_fit: np.ndarray
    mid_fit: np.ndarray
    seeded: np.ndarray
    fine_core: np.ndarray
    mid_core: np.ndarray
    fine_centres: dict[int, np.ndarray]
    mid_centres: dict[int, np.ndarray]
    fine_seen: dict[int, int]
    mid_seen: dict[int, int]


def format_bucket(kind: str, runtime: int | None) -> int:
    """0 feature, 1 series, 2 short."""
    if kind == "tv":
        return 1
    return 2 if runtime and runtime < SHORT_MINUTES else 0


def read_reviews(vectors: str, keys: str, dimensions: int) -> tuple[np.ndarray, np.ndarray]:
    rows = np.fromfile(vectors, dtype=row_type(dimensions))
    held = np.fromfile(keys, dtype="<u4")
    if len(rows) != len(held):
        raise StageFailed("group: review vectors and keys disagree")
    return restore(rows), held


def centred(R: np.ndarray, buckets: np.ndarray) -> np.ndarray:
    out = R.copy()
    for bucket in np.unique(buckets):
        members = buckets == bucket
        if members.sum() >= MIN_BUCKET:
            out[members] -= out[members].mean(0)
    return unit(out)


def _centres(R: np.ndarray, labels: np.ndarray) -> tuple[dict, dict]:
    centres, seen = {}, {}
    for gid in np.unique(labels):
        members = labels == gid
        centres[int(gid)] = unit(R[members].mean(0, keepdims=True))[0]
        seen[int(gid)] = int(members.sum())
    return centres, seen


def build_vibe(
    E: np.ndarray,
    keys: np.ndarray,
    buckets: np.ndarray,
    reviews: np.ndarray,
    review_keys: np.ndarray,
    p: Params,
    method: str,
    seed: int,
) -> VibeLayer:
    row_of = {int(k): i for i, k in enumerate(keys.tolist())}
    pairs = [(row_of[int(k)], i) for i, k in enumerate(review_keys.tolist()) if int(k) in row_of]
    if not pairs:
        raise StageFailed("group: no review vector matches a title")
    seeds = np.array([a for a, _ in pairs])
    R = centred(reviews[[b for _, b in pairs]], buckets[seeds])
    count = len(seeds)
    fine_s = merge_small(
        R, partition(R, max(2, count // p.vibe_fine_size), method, seed), p.min_vibe_fine
    )
    mid_s = merge_small(
        R,
        partition(R, max(p.vibe_mid_floor, count // p.vibe_mid_size), method, seed),
        p.min_vibe_mid,
    )
    n = len(E)
    fine, mid = np.full(n, -1, np.int32), np.full(n, -1, np.int32)
    fine_fit, mid_fit = np.ones(n, np.float32), np.ones(n, np.float32)
    seeded = np.zeros(n, bool)
    fine[seeds], mid[seeds], seeded[seeds] = fine_s, mid_s, True
    others = np.where(~seeded)[0]
    if len(others):
        picks, fits = place(Neighbours(E[seeds]), E[others], [fine_s, mid_s], p.k_vote, p.tau)
        fine[others], mid[others] = picks
        fine_fit[others], mid_fit[others] = fits
    fine_centres, fine_seen = _centres(R, fine_s)
    mid_centres, mid_seen = _centres(R, mid_s)
    return VibeLayer(
        fine,
        mid,
        fine_fit,
        mid_fit,
        seeded,
        seeded | (fine_fit >= CORE_CONFIDENCE),
        seeded | (mid_fit >= CORE_CONFIDENCE),
        fine_centres,
        mid_centres,
        fine_seen,
        mid_seen,
    )


def mood_pointers(
    moods: list[dict],
    phrases: np.ndarray,
    reviews: np.ndarray,
    centres: dict[int, np.ndarray],
    sizes: dict[int, int],
    min_size: int,
) -> list[tuple]:
    """Per mood the six fine groups closest to its phrase, in the centred review space."""
    kept = [g for g in sorted(centres) if sizes.get(g, 0) >= min_size]
    if not kept:
        return []
    C = np.stack([centres[g] for g in kept])
    Q = unit(phrases - reviews.mean(0))
    rows = []
    for mood, q in zip(moods, Q, strict=True):
        scores = C @ q
        for rank, i in enumerate(np.argsort(-scores, kind="stable")[:6]):
            rows.append((mood["id"], mood["title"], mood["late"], kept[i], float(scores[i]), rank))
    return rows
