"""Clustering on unit vectors. Two methods give the same shape of answer.

`leiden` is the method the current groups were made with. It needs the `full` extra, whose Leiden
and igraph packages are GPL, so it is never part of a default install. `kmeans` needs only numpy and
serves small sets and laptops.
"""

from __future__ import annotations

import importlib.util
from collections.abc import Callable

import numpy as np

from ..errors import StageFailed

EXACT_BELOW = 20000
BLOCK = 2048


def leiden_available() -> bool:
    return bool(importlib.util.find_spec("leidenalg") and importlib.util.find_spec("igraph"))


def usearch_available() -> bool:
    return bool(importlib.util.find_spec("usearch"))


def choose_method(setting: str, say: Callable[[str], None]) -> str:
    """Resolves the setting. Asking for Leiden without its packages is a failure, never a swap."""
    if setting == "auto":
        method = "leiden" if leiden_available() else "kmeans"
    elif setting == "leiden" and not leiden_available():
        raise StageFailed("group: Leiden needs the full extra: uv sync --extra full")
    elif setting in ("leiden", "kmeans"):
        method = setting
    else:
        raise StageFailed(f"group: unknown method {setting!r}")
    say(f"group: method {method}")
    return method


def consecutive(labels: np.ndarray) -> np.ndarray:
    return np.unique(labels, return_inverse=True)[1].astype(np.int32)


def spherical_kmeans(X: np.ndarray, k: int, seed: int, rounds: int = 15) -> np.ndarray:
    """K-means on the sphere, with a farthest style start, in row order, so it repeats exactly."""
    n = len(X)
    k = min(k, n)
    if k <= 1:
        return np.zeros(n, np.int32)
    rng = np.random.default_rng(seed)
    centres = np.empty((k, X.shape[1]), np.float32)
    centres[0] = X[rng.integers(n)]
    best = X @ centres[0]
    for i in range(1, k):
        gap = np.maximum(1.0 - best, 0.0)
        total = gap.sum()
        pick = rng.choice(n, p=gap / total) if total > 0 else rng.integers(n)
        centres[i] = X[pick]
        best = np.maximum(best, X @ centres[i])
    labels = np.zeros(n, np.int32)
    for _ in range(rounds):
        fit = np.empty(n, np.float32)
        for a in range(0, n, BLOCK):
            sims = X[a : a + BLOCK] @ centres.T
            labels[a : a + BLOCK] = sims.argmax(1)
            fit[a : a + BLOCK] = sims.max(1)
        order = np.argsort(fit, kind="stable")
        sizes = np.bincount(labels, minlength=k)
        moved = np.zeros((k, X.shape[1]), np.float64)
        np.add.at(moved, labels, X)
        for worst, j in enumerate(np.where(sizes == 0)[0]):
            moved[j] = X[order[worst]]
        norms = np.linalg.norm(moved, axis=1, keepdims=True)
        updated = (moved / np.maximum(norms, 1e-9)).astype(np.float32)
        if np.allclose(updated, centres, atol=1e-5):
            break
        centres = updated
    return consecutive(labels)


def knn_exact(E: np.ndarray, k: int, Q: np.ndarray | None = None) -> tuple[np.ndarray, np.ndarray]:
    """Nearest rows of E for each row of Q. With no Q, each row of E skips itself."""
    own = Q is None
    queries = E if own else Q
    n = len(queries)
    idx = np.empty((n, k), np.int32)
    sim = np.empty((n, k), np.float32)
    for a in range(0, n, BLOCK):
        sims = queries[a : a + BLOCK] @ E.T
        if own:
            sims[np.arange(sims.shape[0]), np.arange(a, a + sims.shape[0])] = -2.0
        ix = np.argsort(-sims, axis=1, kind="stable")[:, :k]
        idx[a : a + BLOCK] = ix
        sim[a : a + BLOCK] = np.take_along_axis(sims, ix, axis=1)
    return idx, sim


class Neighbours:
    """Nearest neighbour search: exact on small sets, usearch on large ones when installed."""

    def __init__(self, E: np.ndarray) -> None:
        self.E = E
        self.index = None
        if len(E) >= EXACT_BELOW and usearch_available():
            from usearch.index import Index

            self.index = Index(
                ndim=E.shape[1],
                metric="cos",
                dtype="f16",
                connectivity=16,
                expansion_add=128,
                expansion_search=128,
            )
            self.index.add(np.arange(len(E), dtype=np.uint64), E)

    def search(self, Q: np.ndarray, k: int) -> tuple[np.ndarray, np.ndarray]:
        k = min(k, len(self.E))
        if self.index is None:
            return knn_exact(self.E, k, Q)
        found = self.index.search(Q, k)
        return np.asarray(found.keys), 1.0 - np.asarray(found.distances)


def own_neighbours(E: np.ndarray, near: Neighbours, k: int) -> tuple[np.ndarray, np.ndarray]:
    """Neighbours of E among itself, each row's own entry left out."""
    k = min(k, len(E) - 1)
    if near.index is None:
        return knn_exact(E, k)
    idx, sim = near.search(E, k + 1)
    keep = np.arange(len(E))[:, None] != idx
    order = np.argsort(~keep, axis=1, kind="stable")[:, :k]
    return np.take_along_axis(idx, order, 1), np.take_along_axis(sim, order, 1)


def _graph(idx: np.ndarray, sim: np.ndarray):
    import igraph

    n, k = idx.shape
    u = np.repeat(np.arange(n, dtype=np.int64), k)
    v = idx.reshape(-1).astype(np.int64)
    w = sim.reshape(-1)
    a, b = np.minimum(u, v), np.maximum(u, v)
    _, first = np.unique(a * n + b, return_index=True)
    g = igraph.Graph(n=n, edges=np.stack([a[first], b[first]], 1).tolist())
    g.es["weight"] = np.maximum(w[first], 1e-4).tolist()
    return g


def _leiden(g, gamma: float, seed: int) -> np.ndarray:
    import leidenalg

    return np.array(
        leidenalg.find_partition(
            g,
            leidenalg.RBConfigurationVertexPartition,
            weights="weight",
            resolution_parameter=gamma,
            n_iterations=2,
            seed=seed,
        ).membership
    )


def leiden_target(
    idx: np.ndarray,
    sim: np.ndarray,
    target: int,
    low: float,
    high: float,
    rounds: int,
    seed: int,
) -> np.ndarray:
    """Bisects the resolution until the group count is within 12 percent of the target."""
    g = _graph(idx, sim)
    best: tuple[np.ndarray, int] | None = None
    for _ in range(rounds):
        gamma = (low * high) ** 0.5
        labels = _leiden(g, gamma, seed)
        count = int(labels.max()) + 1
        if best is None or abs(np.log(count / target)) < abs(np.log(best[1] / target)):
            best = (labels, count)
        if abs(np.log(count / target)) < 0.12:
            break
        if count > target:
            high = gamma
        else:
            low = gamma
    return best[0].astype(np.int32)


def partition(
    X: np.ndarray,
    target: int,
    method: str,
    seed: int,
    k: int = 15,
    low: float = 0.01,
    high: float = 30.0,
    rounds: int = 7,
    near: Neighbours | None = None,
) -> np.ndarray:
    """About `target` groups of the rows of X."""
    if len(X) <= 2 or target <= 1:
        return np.zeros(len(X), np.int32)
    if method == "kmeans":
        return spherical_kmeans(X, target, seed)
    near = near or Neighbours(X)
    idx, sim = own_neighbours(X, near, k)
    return leiden_target(idx, sim, target, low, high, rounds, seed)


def merge_small(X: np.ndarray, labels: np.ndarray, min_size: int) -> np.ndarray:
    """Groups under the minimum join the group with the closest centre."""
    labels = consecutive(labels)
    sizes = np.bincount(labels)
    big = np.where(sizes >= min_size)[0]
    if len(big) == 0 or len(big) == len(sizes):
        return labels
    centres = np.zeros((len(sizes), X.shape[1]), np.float64)
    np.add.at(centres, labels, X)
    centres /= np.maximum(np.linalg.norm(centres, axis=1, keepdims=True), 1e-9)
    remap = np.arange(len(sizes))
    for j in np.where(sizes < min_size)[0]:
        remap[j] = big[np.argmax(centres[big] @ centres[j])]
    return consecutive(remap[labels])


def place(
    near: Neighbours,
    Q: np.ndarray,
    labels: list[np.ndarray],
    k: int,
    tau: float,
) -> tuple[list[np.ndarray], list[np.ndarray]]:
    """Each query row takes the weighted vote of its nearest labelled rows, per label set."""
    picks = [np.empty(len(Q), np.int32) for _ in labels]
    fits = [np.empty(len(Q), np.float32) for _ in labels]
    for a in range(0, len(Q), 10000):
        idx, sim = near.search(Q[a : a + 10000], k)
        w = np.exp((sim - sim.max(1, keepdims=True)) / tau)
        w /= w.sum(1, keepdims=True)
        rows = np.repeat(np.arange(len(idx)), idx.shape[1])
        for pick, fit, label in zip(picks, fits, labels, strict=True):
            votes = np.zeros((len(idx), int(label.max()) + 1), np.float32)
            np.add.at(votes, (rows, label[idx].reshape(-1)), w.reshape(-1))
            pick[a : a + 10000] = votes.argmax(1)
            fit[a : a + 10000] = votes.max(1)
    return picks, fits
