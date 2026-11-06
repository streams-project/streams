"""Row vocabulary as vectors: the calendar entries and the Kids entries, in query form."""

from __future__ import annotations

import json
import struct
from pathlib import Path
from typing import Any

import numpy as np

from ..context import Context
from ..embed.backends import Embedder
from ..embed.quant import unit
from ..errors import StageFailed
from ..vocab import load

TASK = "Given a description of a kind of film or series, retrieve the films and series that fit it"
MAX_LENGTH = 128


def calendar_entries() -> list[dict[str, Any]]:
    entries = []
    for entry in load("calendar")["entries"]:
        if (entry.get("kind") or entry.get("type")) == "calendar":
            entries.append({**entry, "kind": "calendar"})
    return entries


def kids_entries() -> list[dict[str, Any]]:
    entries = []
    for entry in load("kids")["entries"]:
        kept = {
            "title": entry["title"],
            "media_type": entry["media_type"],
            "type": entry["kind"],
            "kind": entry["kind"],
            "vibe_query": entry["vibe_query"],
        }
        if entry.get("season"):
            kept["seasonal"] = entry["season"]
        entries.append(kept)
    return entries


def write_set(
    embedder: Embedder, dimensions: int, entries: list[dict[str, Any]], stem: Path
) -> None:
    """A count as u32 LE, then float32 LE rows of unit length, beside a JSON file of entries."""
    texts = [f"{e['title']}: {e.get('vibe_query', '')}" for e in entries]
    found = embedder.encode_queries(texts, TASK, MAX_LENGTH)
    if found.shape[1] < dimensions:
        raise StageFailed(f"name: the model gave {found.shape[1]} values, need {dimensions}")
    vectors = unit(found[:, :dimensions].astype(np.float32))
    stem.with_suffix(".bin").write_bytes(
        struct.pack("<I", len(vectors)) + vectors.astype("<f4").tobytes()
    )
    document = {"model": embedder.model, "dimensions": dimensions, "entries": entries}
    stem.with_suffix(".json").write_text(json.dumps(document, ensure_ascii=False))


def write_taxonomy(ctx: Context, embedder: Embedder) -> None:
    folder = ctx.work / "export"
    folder.mkdir(exist_ok=True)
    dimensions = ctx.settings.embed.dimensions
    calendar = calendar_entries()
    kids = kids_entries()
    write_set(embedder, dimensions, calendar, folder / "taxonomy")
    write_set(embedder, dimensions, kids, folder / "taxonomy_kids")
    ctx.say(f"name: row vocabulary {len(calendar)} calendar, {len(kids)} kids entries")
