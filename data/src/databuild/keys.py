"""Title keys: a kind plus an id. Same rules as the contract, whose number ranges overlap."""

from __future__ import annotations

import re

TV_BIT = 1 << 31
MAX_TITLE_ID = TV_BIT - 1
KINDS = ("movie", "tv")
KEY_SCHEME = "kind_top_bit_u32_le"

_TITLE_KEY = re.compile(r"(movie|tv):([1-9][0-9]*)")


def item_key(kind: str, title_id: int) -> int:
    """Packed key: a film keeps its id, a series sets the top bit."""
    if kind not in KINDS:
        raise ValueError(f"unknown kind {kind!r}")
    if not 0 < title_id <= MAX_TITLE_ID:
        raise ValueError(f"title id out of range: {title_id}")
    return title_id | TV_BIT if kind == "tv" else title_id


def unpack(key: int) -> tuple[str, int]:
    """Kind and id of a packed key."""
    return ("tv", key & MAX_TITLE_ID) if key & TV_BIT else ("movie", key)


def title_key(kind: str, title_id: int) -> str:
    """The wire form, `movie:<id>` or `tv:<id>`."""
    item_key(kind, title_id)
    return f"{kind}:{title_id}"


def parse_title_key(text: str) -> tuple[str, int]:
    """Reads the wire form. A bare id means nothing, so it does not parse."""
    match = _TITLE_KEY.fullmatch(text) if isinstance(text, str) else None
    if not match:
        raise ValueError(f"not a title key: {text!r}")
    title_id = int(match[2])
    if title_id > MAX_TITLE_ID:
        raise ValueError(f"title id out of range: {text!r}")
    return match[1], title_id
