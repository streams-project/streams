"""Title facts from Wikidata, which is CC0. Its statements also hold the TMDB ids that key a title.

A short list comes first, ordered by how many Wikipedia languages cover a title. Facts then arrive
one property at a time in small batches, because one query joining them all multiplies its rows.
"""

from __future__ import annotations

import re
import time
import urllib.parse
from collections import defaultdict
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from ..errors import StageFailed
from ..net import NetError, Request, Transport

FILM_ID = "P4947"
SERIES_ID = "P4983"
TVMAZE_ID = "P8600"
IMDB_ID = "P345"
ATTEMPTS = 4
BATCH = 200
TIMEOUT = 90.0
MIN_RUNTIME = 20
MAX_RUNTIME = 600

_ITEM = re.compile(r"Q[0-9]+")
_DIGITS = re.compile(r"[1-9][0-9]*")

LIST_FILMS = """
SELECT ?item ?id ?links WHERE {{
  ?item wdt:{id_property} ?id ; wikibase:sitelinks ?links .
  FILTER(?links >= {min_links})
}} ORDER BY DESC(?links) ?item LIMIT {limit}
"""
LIST_SERIES = """
SELECT ?item ?id ?links ?maze ?imdb WHERE {{
  ?item wdt:{id_property} ?id ; wikibase:sitelinks ?links .
  OPTIONAL {{ ?item wdt:{maze_property} ?maze . }}
  OPTIONAL {{ ?item wdt:{imdb_property} ?imdb . }}
  FILTER(?links >= {min_links})
}} ORDER BY DESC(?links) ?item LIMIT {limit}
"""
NAMES = """
SELECT ?item ?value ?note WHERE {{
  VALUES ?item {{ {items} }}
  ?item rdfs:label ?value . FILTER(LANG(?value) = "en")
  OPTIONAL {{ ?item schema:description ?note . FILTER(LANG(?note) = "en") }}
}}
"""
LABELS = """
SELECT ?item ?value WHERE {{
  VALUES ?item {{ {items} }}
  ?item wdt:{prop} ?target . ?target rdfs:label ?value . FILTER(LANG(?value) = "en")
}}
"""
VALUES = """
SELECT ?item ?value WHERE {{
  VALUES ?item {{ {items} }}
  ?item wdt:{prop} ?value .
}}
"""
CODES = """
SELECT ?item ?value WHERE {{
  VALUES ?item {{ {items} }}
  ?item wdt:{prop} ?target . ?target wdt:{code} ?value .
}}
"""

LABEL_FACTS = {
    "genres": "P136",
    "directors": "P57",
    "creators": "P170",
    "cast": "P161",
    "subjects": "P921",
}
VALUE_FACTS = {"dates": "P577", "runtimes": "P2047"}
CODE_FACTS = {"countries": ("P495", "P297"), "languages": ("P364", "P218")}


@dataclass(frozen=True)
class Listed:
    item: str
    title_id: int
    links: int
    maze: int | None = None
    imdb: str = ""


@dataclass
class Facts:
    label: str = ""
    note: str = ""
    values: dict[str, list[str]] = field(default_factory=lambda: defaultdict(list))

    def all(self, name: str) -> list[str]:
        return sorted(set(self.values.get(name, [])))

    @property
    def date(self) -> str:
        dates = [d[:10] for d in self.all("dates") if re.match(r"[12][0-9]{3}-", d)]
        return dates[0] if dates else ""

    @property
    def runtime(self) -> int | None:
        for value in self.all("runtimes"):
            try:
                minutes = round(float(value))
            except ValueError:
                continue
            if MIN_RUNTIME <= minutes <= MAX_RUNTIME:
                return minutes
        return None


class Wikidata:
    """Calls one query endpoint. A wrong answer never brings an address into its error text."""

    def __init__(
        self,
        endpoint: str,
        user_agent: str,
        transport: Transport,
        sleep: Callable[[float], None] = time.sleep,
    ) -> None:
        self.endpoint = endpoint
        self.user_agent = user_agent
        self.transport = transport
        self.sleep = sleep

    def select(self, query: str) -> list[dict[str, str]]:
        """Rows of a query as plain text values."""
        body = urllib.parse.urlencode({"query": query, "format": "json"}).encode()
        headers = {
            "Accept": "application/sparql-results+json",
            "Content-Type": "application/x-www-form-urlencoded",
            "User-Agent": self.user_agent,
        }
        request = Request(self.endpoint, headers=headers, body=body, timeout=TIMEOUT)
        for attempt in range(1, ATTEMPTS + 1):
            try:
                reply = self.transport.fetch(request)
            except NetError:
                self.sleep(attempt * 2.0)
                continue
            if reply.status == 200:
                rows = reply.json()["results"]["bindings"]
                return [{k: v["value"] for k, v in row.items()} for row in rows]
            if reply.status == 429:
                self.sleep(float(reply.headers.get("retry-after", attempt * 5.0)))
                continue
            if reply.status >= 500:
                self.sleep(attempt * 2.0)
                continue
            raise StageFailed(f"wikidata: the endpoint answered {reply.status}")
        raise StageFailed(f"wikidata: no answer after {ATTEMPTS} attempts")


def item_of(address: str) -> str:
    """The Q number at the end of an entity address."""
    return address.rsplit("/", 1)[-1]


def _number(text: str | None) -> int | None:
    return int(text) if text and _DIGITS.fullmatch(text) else None


def parse_list(rows: list[dict[str, str]]) -> list[Listed]:
    """Good rows in order, each key once. A key shared by two entries keeps the better known."""
    found: list[Listed] = []
    seen: set[int] = set()
    for row in rows:
        item = item_of(row.get("item", ""))
        title_id = _number(row.get("id"))
        if not _ITEM.fullmatch(item) or title_id is None or title_id in seen:
            continue
        seen.add(title_id)
        found.append(
            Listed(
                item,
                title_id,
                int(float(row.get("links", "0"))),
                _number(row.get("maze")),
                row.get("imdb", ""),
            )
        )
    return found


def list_films(client: Wikidata, limit: int, min_links: int) -> list[Listed]:
    query = LIST_FILMS.format(id_property=FILM_ID, min_links=min_links, limit=limit)
    return parse_list(client.select(query))


def list_series(client: Wikidata, limit: int, min_links: int) -> list[Listed]:
    query = LIST_SERIES.format(
        id_property=SERIES_ID,
        maze_property=TVMAZE_ID,
        imdb_property=IMDB_ID,
        min_links=min_links,
        limit=limit,
    )
    return parse_list(client.select(query))


def read_facts(client: Wikidata, items: list[str]) -> dict[str, Facts]:
    """Facts of every item, a batch at a time."""
    facts: dict[str, Facts] = {item: Facts() for item in items}
    for start in range(0, len(items), BATCH):
        part = " ".join(f"wd:{item}" for item in items[start : start + BATCH])
        for row in client.select(NAMES.format(items=part)):
            held = facts.get(item_of(row["item"]))
            if held:
                held.label = row.get("value", "")
                held.note = row.get("note", "")
        queries: list[tuple[str, str]] = []
        for name, prop in LABEL_FACTS.items():
            queries.append((name, LABELS.format(items=part, prop=prop)))
        for name, prop in VALUE_FACTS.items():
            queries.append((name, VALUES.format(items=part, prop=prop)))
        for name, (prop, code) in CODE_FACTS.items():
            queries.append((name, CODES.format(items=part, prop=prop, code=code)))
        for name, query in queries:
            for row in client.select(query):
                held = facts.get(item_of(row["item"]))
                if held:
                    held.values[name].append(row["value"])
    return facts


def describe_film(facts: Facts, genres: list[str]) -> str:
    """A sentence made from facts alone, so the text stays CC0. It leaves out the title."""
    year = facts.date[:4]
    kind = " and ".join(g.lower() for g in genres[:2])
    opening = " ".join(p for p in ("A", year, kind, "film") if p)
    directors = facts.all("directors")[:2]
    cast = facts.all("cast")[:3]
    text = opening
    if directors:
        text += f" directed by {' and '.join(directors)}"
    if cast:
        text += f", with {', '.join(cast)}"
    text += "."
    subjects = facts.all("subjects")[:3]
    if subjects:
        text += f" It is about {', '.join(subjects)}."
    return text


def answer_row(values: dict[str, Any]) -> dict[str, Any]:
    """One row of a query answer, for tests and fakes."""
    return {k: {"value": str(v)} for k, v in values.items()}
