"""Series text from TVmaze, which is CC BY-SA 4.0. It stays in its own folder, with credit."""

from __future__ import annotations

import html
import re
import time
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from ..errors import StageFailed
from ..net import NetError, Reply, Request, Transport
from ..pull.tmdb import Limiter

ATTEMPTS = 5
TIMEOUT = 30.0
_TAG = re.compile(r"<[^>]+>")
_SPACE = re.compile(r"\s+")
LANGUAGES = {
    "Arabic": "ar",
    "Chinese": "zh",
    "Danish": "da",
    "Dutch": "nl",
    "English": "en",
    "Finnish": "fi",
    "French": "fr",
    "German": "de",
    "Hindi": "hi",
    "Italian": "it",
    "Japanese": "ja",
    "Korean": "ko",
    "Norwegian": "no",
    "Polish": "pl",
    "Portuguese": "pt",
    "Russian": "ru",
    "Spanish": "es",
    "Swedish": "sv",
    "Thai": "th",
    "Turkish": "tr",
}


def plain(text: str | None) -> str:
    """Text with its markup gone."""
    return _SPACE.sub(" ", html.unescape(_TAG.sub(" ", text or ""))).strip()


@dataclass
class Show:
    maze_id: int
    name: str
    text: str
    url: str
    language: str
    premiered: str
    runtime: int | None
    tv_type: str | None
    status: str | None
    genres: list[str]
    networks: list[str]
    countries: list[str]
    seasons: list[tuple] = field(default_factory=list)
    episodes: list[tuple] = field(default_factory=list)


def parse_show(document: dict[str, Any], title_id: int) -> Show:
    """A show, with rows shaped like the pull file's seasons and episodes tables."""
    network = document.get("network") or document.get("webChannel") or {}
    country = (network.get("country") or {}).get("code")
    held: dict[int, dict[str, Any]] = {}
    episodes = []
    seen: set[tuple[int, int]] = set()
    for e in (document.get("_embedded") or {}).get("episodes") or []:
        season, number = e.get("season"), e.get("number")
        if season is None or number is None or (season, number) in seen:
            continue
        seen.add((season, number))
        aired = e.get("airdate") or ""
        episodes.append(
            (
                title_id,
                season,
                number,
                e.get("name") or "",
                plain(e.get("summary")),
                aired,
                "",
                e.get("runtime"),
                (e.get("rating") or {}).get("average"),
            )
        )
        place = held.setdefault(season, {"count": 0, "first": aired})
        place["count"] += 1
        place["first"] = min(place["first"] or aired, aired or place["first"])
    seasons = [
        (title_id, n, f"Season {n}", "", place["first"], "", place["count"])
        for n, place in sorted(held.items())
    ]
    return Show(
        maze_id=document["id"],
        name=document.get("name") or "",
        text=plain(document.get("summary")),
        url=document.get("url") or "",
        language=LANGUAGES.get(document.get("language") or "", ""),
        premiered=document.get("premiered") or "",
        runtime=document.get("runtime") or document.get("averageRuntime"),
        tv_type=document.get("type"),
        status=document.get("status"),
        genres=list(document.get("genres") or []),
        networks=[network["name"]] if network.get("name") else [],
        countries=[country] if country else [],
        seasons=seasons,
        episodes=episodes,
    )


class TvMaze:
    """Keeps under the limit of 20 calls in 10 seconds, and waits when told to."""

    def __init__(
        self,
        base: str,
        user_agent: str,
        transport: Transport,
        rps: float,
        sleep: Callable[[float], None] = time.sleep,
        clock: Callable[[], float] = time.monotonic,
    ) -> None:
        self.base = base
        self.headers = {"User-Agent": user_agent}
        self.transport = transport
        self.sleep = sleep
        self.limiter = Limiter(rps, clock, sleep)

    def _get(self, path: str, params: dict[str, str]) -> Reply | None:
        request = Request(self.base, path, params, self.headers, timeout=TIMEOUT)
        for attempt in range(1, ATTEMPTS + 1):
            self.limiter.wait()
            try:
                reply = self.transport.fetch(request)
            except NetError:
                self.sleep(attempt * 2.0)
                continue
            if reply.status in (200, 404):
                return reply
            if reply.status == 429:
                self.limiter.slow()
                self.sleep(float(reply.headers.get("retry-after", 10.0)))
                continue
            if reply.status >= 500:
                self.sleep(attempt * 2.0)
                continue
            raise StageFailed(f"tvmaze: the service answered {reply.status}")
        return None

    def show(self, maze_id: int) -> dict[str, Any] | None:
        reply = self._get(f"/shows/{maze_id}", {"embed": "episodes"})
        return reply.json() if reply and reply.status == 200 else None

    def lookup(self, imdb: str) -> int | None:
        """The show id of an IMDb id, or None when the service holds no such show."""
        reply = self._get("/lookup/shows", {"imdb": imdb})
        return reply.json()["id"] if reply and reply.status == 200 else None
