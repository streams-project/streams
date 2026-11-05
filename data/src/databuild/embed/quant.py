"""Master rows: int8 values plus a float16 scale per row (S134)."""

from __future__ import annotations

import numpy as np


def row_type(dimensions: int) -> np.dtype:
    return np.dtype([("q", "i1", (dimensions,)), ("s", "<f2")])


def unit(x: np.ndarray) -> np.ndarray:
    return x / (np.linalg.norm(x, axis=1, keepdims=True) + 1e-9)


def quantise(vectors: np.ndarray) -> np.ndarray:
    """Rows of int8 values and a scale, so that value = q x s."""
    scale = np.abs(vectors).max(axis=1) / 127.0 + 1e-12
    rows = np.empty(len(vectors), dtype=row_type(vectors.shape[1]))
    rows["q"] = np.round(vectors / scale[:, None]).astype(np.int8)
    rows["s"] = scale.astype("<f2")
    return rows


def restore(rows: np.ndarray) -> np.ndarray:
    """Value = q x s as float32, divided by its L2 norm plus 1e-9.

    The norm is summed in float64, because a float32 sum can change in its last bit with where the
    array sits in memory, and a build must give the same bytes each time.
    """
    x = rows["q"].astype(np.float32) * rows["s"].astype(np.float32)[:, None]
    norms = np.sqrt(np.einsum("ij,ij->i", x, x, dtype=np.float64))
    return x / (norms[:, None].astype(np.float32) + 1e-9)
