"""Vectors, keys and seeds, in the row order of the master file (S134, S154)."""

from __future__ import annotations

import io
import json
import math
import struct
from dataclasses import dataclass

import numpy as np

SEED_MIN_RATING = 6.0
SEED_MIN_VOTES = 50


@dataclass(frozen=True)
class Member:
    popularity: float
    rating: float
    votes: int


@dataclass(frozen=True)
class Seed:
    group: int
    mass: float
    centre: np.ndarray
    keys: np.ndarray


def vector_bytes(unit_rows: np.ndarray) -> bytes:
    """Unit vectors as little endian float16."""
    return unit_rows.astype("<f2").tobytes()


def make_seeds(
    rows: np.ndarray,
    keys: np.ndarray,
    w2: np.ndarray,
    members: dict[int, Member],
    votes: bool = True,
) -> list[Seed]:
    """One seed per world level 2 group with titles here, heaviest first.

    Mass is the sum of ln(1 + popularity) over members rated 6.0 or more with 50 votes or more.
    Open sources hold no votes, so an open set counts every member.
    """
    by_group: dict[int, list[int]] = {}
    for i, group in enumerate(w2.tolist()):
        if group >= 0:
            by_group.setdefault(group, []).append(i)
    seeds = []
    for group, held in sorted(by_group.items()):
        index = np.array(held)
        centre = rows[index].astype(np.float64).mean(0)
        centre = centre / (np.linalg.norm(centre) + 1e-9)
        mass = 0.0
        for key in keys[index].tolist():
            member = members.get(key)
            if member is None:
                continue
            good = member.rating >= SEED_MIN_RATING and member.votes >= SEED_MIN_VOTES
            if good or not votes:
                mass += math.log1p(max(member.popularity, 0.0))
        seeds.append(Seed(group, mass, centre.astype("<f4"), keys[index].astype("<u4")))
    seeds.sort(key=lambda s: (-s.mass, s.group))
    return seeds


def seed_bytes(seeds: list[Seed], dimensions: int) -> bytes:
    """A count and a width as u32 LE, then per seed a float32 centre, a member count, the keys."""
    out = io.BytesIO()
    out.write(struct.pack("<II", len(seeds), dimensions))
    for seed in seeds:
        out.write(seed.centre.tobytes())
        out.write(struct.pack("<I", len(seed.keys)))
        out.write(seed.keys.tobytes())
    return out.getvalue()


def read_seeds(data: bytes) -> tuple[int, list[tuple[np.ndarray, np.ndarray]]]:
    """The width and every centre with its keys. A short file raises ValueError."""
    if len(data) < 8:
        raise ValueError("seed file is shorter than its header")
    count, dimensions = struct.unpack_from("<II", data)
    place = 8
    found = []
    for _ in range(count):
        end = place + 4 * dimensions + 4
        if end > len(data):
            raise ValueError("seed file ends inside a seed")
        centre = np.frombuffer(data, "<f4", dimensions, place)
        size = struct.unpack_from("<I", data, end - 4)[0]
        if end + 4 * size > len(data):
            raise ValueError("seed file ends inside a member list")
        found.append((centre, np.frombuffer(data, "<u4", size, end)))
        place = end + 4 * size
    if place != len(data):
        raise ValueError("seed file has bytes after its last seed")
    return dimensions, found


def seed_names(seeds: list[Seed], names: dict[int, str]) -> bytes:
    """One name per seed in file order, empty when the group has none."""
    return json.dumps([names.get(s.group, "") for s in seeds], ensure_ascii=False).encode()


def seed_groups(seeds: list[Seed]) -> bytes:
    return json.dumps([s.group for s in seeds]).encode()
