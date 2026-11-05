"""The text each vector is made from. A change here is a new recipe, so sets are not mixed."""

from __future__ import annotations

import json
from typing import Any

RECIPE_FULL = "genres-tagline-overview-people-v1"
RECIPE_OPEN = "open-text-v1"
MIN_OVERVIEW = 20
FULL_QUERY = (
    "SELECT id, media_type, genres, tagline, overview, director, cast_members FROM titles"
    " WHERE adult = 0 AND length(overview) >= 20 ORDER BY media_type, tmdb_id"
)


def clip(value: Any, size: int) -> str:
    return str(value or "").strip()[:size]


def cast_list(value: Any) -> list[str]:
    text = str(value or "").strip()
    if text.startswith("["):
        try:
            return [str(x) for x in json.loads(text)]
        except ValueError:
            return []
    return [c.strip() for c in text.split(",") if c.strip()]


def full_text(kind: str, genres: Any, tagline: Any, overview: Any, director: Any, cast: Any) -> str:
    """Genres, tagline, overview, people. The title is left out, or neighbours share title words."""
    parts = []
    if clip(genres, 100):
        parts.append("Genres: " + clip(genres, 100))
    if clip(tagline, 150):
        parts.append(clip(tagline, 150))
    parts.append(clip(overview, 800))
    if director:
        label = "Directed by: " if kind == "movie" else "Created by: "
        parts.append(label + clip(director, 80))
    names = cast_list(cast)
    if names:
        parts.append("Cast: " + ", ".join(names[:5]))
    return " | ".join(parts)
