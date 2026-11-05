"""Generated cards in place of posters: a plain gradient with the title. Nothing is copied.

The same title gives the same bytes, so a rebuild changes no hash.
"""

from __future__ import annotations

import hashlib
import io
import os
import tarfile
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

POSTER = (300, 450)
BACKDROP = (640, 360)
MARGIN = 20
COLOURS = 32


@dataclass(frozen=True)
class Card:
    kind: str
    title_id: int
    title: str
    line: str


def poster_path(kind: str, title_id: int) -> str:
    return f"/poster/{kind}-{title_id}.png"


def backdrop_path(kind: str, title_id: int) -> str:
    return f"/backdrop/{kind}-{title_id}.png"


def _colours(seed: str) -> tuple[tuple[int, int, int], tuple[int, int, int]]:
    d = hashlib.sha256(seed.encode()).digest()
    dark = (30 + d[0] % 60, 30 + d[1] % 60, 40 + d[2] % 70)
    bright = (90 + d[3] % 120, 80 + d[4] % 120, 90 + d[5] % 120)
    return dark, bright


def _gradient(size: tuple[int, int], top: tuple, bottom: tuple) -> Image.Image:
    width, height = size
    strip = Image.new("RGB", (1, height))
    for y in range(height):
        share = y / max(height - 1, 1)
        strip.putpixel(
            (0, y), tuple(round(a + (b - a) * share) for a, b in zip(top, bottom, strict=True))
        )
    return strip.resize((width, height))


def _wrap(draw: ImageDraw.ImageDraw, text: str, font: ImageFont.ImageFont, width: int) -> list[str]:
    lines: list[str] = []
    current = ""
    for word in text.split():
        trial = f"{current} {word}".strip()
        if draw.textlength(trial, font=font) <= width or not current:
            current = trial
        else:
            lines.append(current)
            current = word
    if current:
        lines.append(current)
    return lines


def render(card: Card, size: tuple[int, int]) -> bytes:
    """A PNG of one card."""
    dark, bright = _colours(f"{card.kind}:{card.title_id}")
    image = _gradient(size, bright, dark)
    draw = ImageDraw.Draw(image)
    width = size[0] - 2 * MARGIN
    big = ImageFont.load_default(size[1] // 13)
    small = ImageFont.load_default(size[1] // 24)
    lines = _wrap(draw, card.title, big, width)[:4]
    height = size[1] // 11
    y = size[1] - MARGIN - height * len(lines) - size[1] // 18
    for line in lines:
        draw.text((MARGIN, y), line, font=big, fill=(245, 245, 245))
        y += height
    draw.text((MARGIN, y + 4), card.line, font=small, fill=(215, 215, 215))
    out = io.BytesIO()
    image.quantize(COLOURS, dither=Image.Dither.NONE).save(out, "PNG", optimize=True)
    return out.getvalue()


def _member(tar: tarfile.TarFile, name: str, data: bytes) -> None:
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mode = 0o644
    tar.addfile(info, io.BytesIO(data))


def write_cards(path: Path, cards: Iterable[Card]) -> int:
    """One tar holding a poster and a backdrop per title, in a fixed order with no dates."""
    tmp = path.with_name(path.name + ".tmp")
    count = 0
    with tarfile.open(tmp, "w", format=tarfile.USTAR_FORMAT) as tar:
        for card in sorted(cards, key=lambda c: (c.kind, c.title_id)):
            poster = poster_path(card.kind, card.title_id).lstrip("/")
            backdrop = backdrop_path(card.kind, card.title_id).lstrip("/")
            _member(tar, poster, render(card, POSTER))
            _member(tar, backdrop, render(card, BACKDROP))
            count += 1
    os.replace(tmp, path)
    return count
