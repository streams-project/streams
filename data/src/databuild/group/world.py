"""World groups, atoms and themes.

Titles with a long overview make the groups. The others are placed by a vote of their nearest
neighbours, with the vote's share kept as confidence.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

import numpy as np

from .cluster import Neighbours, merge_small, partition, place
from .params import Params


@dataclass
class Layers:
    w1: np.ndarray
    w2: np.ndarray
    wconf: np.ndarray
    wsrc: np.ndarray
    atom: np.ndarray
    th1: np.ndarray
    th2: np.ndarray
    th3: np.ndarray


def _level_two(Ec: np.ndarray, l1: np.ndarray, p: Params, method: str, seed: int) -> np.ndarray:
    l2 = np.zeros(len(Ec), np.int32)
    next_id = 0
    for c in range(int(l1.max()) + 1):
        members = np.where(l1 == c)[0]
        if len(members) < p.min_split:
            l2[members] = next_id
            next_id += 1
            continue
        sub = partition(
            Ec[members],
            max(2, round(len(members) / p.sub_size)),
            method,
            seed,
            k=min(p.k_graph, len(members) - 1),
        )
        sub = merge_small(Ec[members], sub, p.min_sub)
        l2[members] = next_id + sub
        next_id += int(sub.max()) + 1
    return l2


def build_layers(
    E: np.ndarray,
    text_length: np.ndarray,
    p: Params,
    method: str,
    seed: int,
    say: Callable[[str], None],
) -> Layers:
    core = text_length >= p.min_text
    if core.sum() < max(4, 2 * p.target_l1):
        core = np.ones(len(E), bool)
    ci, ti = np.where(core)[0], np.where(~core)[0]
    Ec = E[ci]
    say(f"group: {len(ci)} titles make the groups, {len(ti)} are placed by vote")
    near = Neighbours(Ec)
    l1 = partition(
        Ec, p.target_l1, method, seed, k=p.k_graph, low=0.8, high=3.0, rounds=6, near=near
    )
    l1 = merge_small(Ec, l1, p.min_l1)
    l2 = _level_two(Ec, l1, p, method, seed)
    say(f"group: world level 1 has {int(l1.max()) + 1}, level 2 has {int(l2.max()) + 1}")

    atoms = partition(
        Ec,
        max(2, round(len(Ec) / p.atom_size)),
        method,
        seed,
        k=p.k_graph,
        low=1.0,
        high=300.0,
        near=near,
    )
    atoms = merge_small(Ec, atoms, p.min_atom)
    centres = np.zeros((int(atoms.max()) + 1, Ec.shape[1]), np.float64)
    np.add.at(centres, atoms, Ec)
    centres = (centres / np.maximum(np.linalg.norm(centres, axis=1, keepdims=True), 1e-9)).astype(
        np.float32
    )
    themes = []
    for target in p.theme_targets:
        level = partition(centres, min(target, len(centres)), method, seed, k=p.k_graph)
        themes.append(level[atoms])
    say(f"group: {len(centres)} atoms, themes {[int(t.max()) + 1 for t in themes]}")

    labels = [l1, l2, atoms, *themes]
    out = [np.zeros(len(E), np.int32) for _ in labels]
    for target, source in zip(out, labels, strict=True):
        target[ci] = source
    wconf = np.ones(len(E), np.float32)
    wsrc = np.zeros(len(E), np.int8)
    if len(ti):
        picks, fits = place(near, E[ti], labels, p.k_vote, p.tau)
        for target, pick in zip(out, picks, strict=True):
            target[ti] = pick
        wconf[ti] = fits[1]
        wsrc[ti] = 1
    return Layers(out[0], out[1], wconf, wsrc, out[2], out[3], out[4], out[5])
