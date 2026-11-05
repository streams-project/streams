"""Embeds titles in blocks of 1,000. A stopped run resumes from its finished blocks.

A block's file name holds a hash of its keys and text, and the folder name holds the model, size and
recipe, so a rerun can never mix vectors from another model or older text.
"""

from __future__ import annotations

import hashlib
import json
import os
import sqlite3
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from ..context import Context
from ..errors import StageFailed
from ..keys import item_key, parse_title_key
from .backends import Embedder, make_embedder
from .quant import quantise, row_type, unit
from .text import FULL_QUERY, RECIPE_FULL, RECIPE_OPEN, full_text

BLOCK = 1000
QUANTISATION = "int8-with-f16-scale"


@dataclass(frozen=True)
class Item:
    key: int
    text: str


def full_items(titles_db: Path) -> list[Item]:
    """Every non adult title with an overview of 20 characters or more."""
    db = sqlite3.connect(f"file:{titles_db}?mode=ro", uri=True)
    items = [
        Item(key, full_text(kind, genres, tagline, overview, director, cast))
        for key, kind, genres, tagline, overview, director, cast in db.execute(FULL_QUERY)
    ]
    db.close()
    return items


def open_items(path: Path) -> list[Item]:
    """Open text lines, `{"key": "movie:12", "text": "..."}`. Only open sources wrote this file."""
    items = []
    for line in path.read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        kind, title_id = parse_title_key(row["key"])
        items.append(Item(item_key(kind, title_id), row["text"]))
    items.sort(key=lambda i: (i.key >= (1 << 31), i.key))
    return items


def _fingerprint(model: str, dimensions: int, recipe: str) -> str:
    return hashlib.sha256(f"{model}|{dimensions}|{recipe}".encode()).hexdigest()[:16]


def _digest(items: list[Item]) -> str:
    h = hashlib.sha256()
    for item in items:
        h.update(item.key.to_bytes(4, "little"))
        h.update(item.text.encode())
        h.update(b"\0")
    return h.hexdigest()[:16]


def embed_items(
    ctx: Context, embedder: Embedder, items: list[Item], recipe: str, target: Path
) -> None:
    """Writes `master.q8`, `master.keys` and `embed.json` into the target folder."""
    if not items:
        raise StageFailed("embed: no titles with text")
    dimensions = ctx.settings.embed.dimensions
    folder = target / "blocks" / _fingerprint(embedder.model, dimensions, recipe)
    folder.mkdir(parents=True, exist_ok=True)
    blocks: list[Path] = []
    made = 0
    for start in range(0, len(items), BLOCK):
        part = items[start : start + BLOCK]
        path = folder / f"block-{start // BLOCK:05d}-{_digest(part)}.q8"
        blocks.append(path)
        if path.exists():
            continue
        vectors = embedder.encode([i.text for i in part])
        if vectors.shape != (len(part), vectors.shape[1]) or vectors.shape[1] < dimensions:
            raise StageFailed(f"embed: the model gave shape {vectors.shape}, need {dimensions}")
        tmp = path.with_suffix(".tmp")
        tmp.write_bytes(quantise(unit(vectors[:, :dimensions])).tobytes())
        os.replace(tmp, path)
        made += len(part)
    master = target / "master.q8"
    with open(master.with_suffix(".tmp"), "wb") as out:
        for path in blocks:
            out.write(path.read_bytes())
    os.replace(master.with_suffix(".tmp"), master)
    keys = np.array([i.key for i in items], dtype="<u4")
    if len(np.unique(keys)) != len(keys):
        raise StageFailed("embed: two rows hold one key")
    (target / "master.keys").write_bytes(keys.tobytes())
    info = {
        "model": embedder.model,
        "dimensions": dimensions,
        "quantisation": QUANTISATION,
        "recipe": recipe,
        "rows": len(items),
    }
    (target / "embed.json").write_text(json.dumps(info, indent=1) + "\n")
    ctx.say(f"embed: {len(items)} rows, {made} new, model {embedder.model}, recipe {recipe}")


def read_master(folder: Path) -> tuple[np.ndarray, np.ndarray, dict]:
    """Master rows, keys and the embed record of a folder, checked against each other."""
    info = json.loads((folder / "embed.json").read_text())
    rows = np.fromfile(folder / "master.q8", dtype=row_type(info["dimensions"]))
    keys = np.fromfile(folder / "master.keys", dtype="<u4")
    if not len(rows) == len(keys) == info["rows"]:
        raise StageFailed("embed: master rows, keys and record disagree")
    return rows, keys, info


def embed(ctx: Context, open_text: Path | None = None, primary: bool = False) -> None:
    """Full text by default. A file of open text builds the second vector set, or with `primary`
    the only one, as the open sample does."""
    embedder = make_embedder(ctx)
    if open_text is None:
        items, recipe, target = (
            full_items(ctx.work / "titles.db"),
            RECIPE_FULL,
            ctx.work / "vectors",
        )
    else:
        folder = "vectors" if primary else "vectors-open"
        items, recipe, target = open_items(open_text), RECIPE_OPEN, ctx.work / folder
    target.mkdir(parents=True, exist_ok=True)
    embed_items(ctx, embedder, items, recipe, target)
