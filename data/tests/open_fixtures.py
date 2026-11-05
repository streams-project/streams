"""A made up Wikidata and TVmaze. Every title, name and number is generated here."""

from __future__ import annotations

import re
import sqlite3
import urllib.parse
from dataclasses import replace
from pathlib import Path

from databuild.net import NetError, Reply
from databuild.settings import Embed, Group, Open

from .conftest import FakeTransport

GENRES = ("drama film", "comedy film", "horror film", "science fiction film", "crime film")
SHOW_GENRES = ("Drama", "Comedy", "Horror", "Science-Fiction", "Crime")
SUBJECTS = ("ghost", "wedding", "robot", "heist", "family relationships")
FILM_BASE = 1000
SERIES_BASE = 5000
ENDPOINT = "https://wikidata.fake.test/sparql"
MAZE = "https://maze.fake.test"


def make_summary(i: int) -> str:
    words = SUBJECTS[i % 5]
    return f"<p>A <b>made up</b> story about {words} told in many parts, number {i} &amp; more.</p>"


def make_show(i: int) -> dict:
    return {
        "id": 700 + i,
        "name": f"Made Up Show {i}",
        "url": f"https://maze.fake.test/shows/{700 + i}",
        "language": "English",
        "premiered": f"{1990 + i % 30}-02-03",
        "runtime": 45,
        "type": "Scripted",
        "status": "Ended",
        "genres": [SHOW_GENRES[i % 5]],
        "summary": make_summary(i),
        "network": {"name": f"Channel {i % 4}", "country": {"code": "GB"}},
        "_embedded": {
            "episodes": [
                {
                    "season": s,
                    "number": n,
                    "name": f"Part {n}",
                    "airdate": f"{1990 + i % 30}-02-{n + 2:02d}",
                    "runtime": 45,
                    "summary": f"<p>Part {n} of the story.</p>",
                    "rating": {"average": None},
                }
                for s in (1, 2)
                for n in (1, 2, 3)
            ]
        },
    }


class FakeOpen:
    """Answers the queries the builder sends. `blank` titles hold no facts."""

    def __init__(self, films: int = 60, series: int = 40) -> None:
        self.films = films
        self.series = series
        self.queries: list[str] = []
        self.adult: set[str] = set()
        self.short: set[str] = set()
        self.maze_less: set[str] = set()

    def _film_item(self, i: int) -> str:
        return f"Q{FILM_BASE + i}"

    def _series_item(self, i: int) -> str:
        return f"Q{SERIES_BASE + i}"

    def _lists(self, query: str) -> list[dict]:
        limit = int(re.search(r"LIMIT (\d+)", query)[1])
        rows = []
        if "P4947" in query:
            for i in range(1, self.films + 1):
                rows.append(
                    {"item": f"https://wd.fake.test/entity/{self._film_item(i)}", "id": 100 + i}
                    | {"links": 200 - i}
                )
        else:
            for i in range(1, self.series + 1):
                item = self._series_item(i)
                row = {"item": f"https://wd.fake.test/entity/{item}", "id": 300 + i}
                row |= {"links": 150 - i, "imdb": f"tt{9000 + i}"}
                if item not in self.maze_less:
                    row["maze"] = 700 + i
                rows.append(row)
        return rows[:limit]

    def _number(self, item: str) -> tuple[str, int]:
        n = int(item[1:])
        return ("tv", n - SERIES_BASE) if n >= SERIES_BASE else ("movie", n - FILM_BASE)

    def _facts(self, query: str, item: str) -> list[dict]:
        _, i = self._number(item)
        if item in self.short:
            return []
        word = re.search(r"wdt:(P\d+)", query)
        prop = word[1] if word else ""
        address = f"https://wd.fake.test/entity/{item}"
        values: list[str] = []
        match prop:
            case "P136":
                values = [GENRES[i % 5]]
                if item in self.adult:
                    values.append("pornographic film")
            case "P57":
                values = [f"Director {i % 9}"]
            case "P170":
                values = [f"Creator {i % 9}"]
            case "P161":
                values = [f"Actor {i % 7}", f"Actor {i % 5 + 10}"]
            case "P921":
                values = [SUBJECTS[i % 5]]
            case "P577":
                values = [f"{1970 + i % 50}-03-04T00:00:00Z"]
            case "P2047":
                values = [str(80 + i)]
            case "P495":
                values = ["GB" if i % 3 else "US"]
            case "P364":
                values = ["en"]
        return [{"item": address, "value": v} for v in values]

    def answer(self, request) -> dict:
        query = urllib.parse.parse_qs(request.body.decode())["query"][0]
        self.queries.append(query)
        if "wikibase:sitelinks" in query:
            rows = self._lists(query)
        else:
            items = re.findall(r"wd:(Q\d+)", query)
            rows = []
            if "schema:description" in query:
                for item in items:
                    kind, i = self._number(item)
                    if item not in self.short:
                        name = "Made Up Film" if kind == "movie" else "Made Up Series"
                        rows.append(
                            {
                                "item": f"https://wd.fake.test/entity/{item}",
                                "value": f"{name} {i}",
                                "note": "a made up title",
                            }
                        )
            else:
                for item in items:
                    rows += self._facts(query, item)
        return {
            "results": {"bindings": [{k: {"value": str(v)} for k, v in r.items()} for r in rows]}
        }

    def transport(self) -> FakeTransport:
        routes: dict = {"": self.answer}
        for i in range(1, self.series + 1):
            routes[f"/shows/{700 + i}"] = make_show(i)
        routes["/lookup/shows"] = lambda r: {"id": 700 + int(r.params["imdb"][2:]) - 9000}
        return FakeTransport(routes)


def settle(ctx, fake: FakeOpen, titles: int = 100):
    """Settings for a small sample, with a fake on the wire."""
    ctx.settings = replace(
        ctx.settings,
        embed=Embed(dimensions=64),
        group=Group(method="kmeans"),
        open=Open(ENDPOINT, MAZE, 1000.0, "databuild tests", titles, 2),
    )
    ctx.transport = fake.transport()
    return ctx


def rows_of(path: Path, sql: str) -> list[tuple]:
    db = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
    try:
        return db.execute(sql).fetchall()
    finally:
        db.close()


__all__ = ["ENDPOINT", "MAZE", "FakeOpen", "NetError", "Reply", "make_show", "rows_of", "settle"]
