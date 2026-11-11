"""Title details from TMDB into one resumable SQLite file, keyed by kind and id.

Each title holds the facts the old pull took, plus what clients still asked TMDB for: logos,
trailers, seasons, episode stills and runtimes, network and company logos, created by, crew roles
and translated text per interface language. Clients cannot lose TMDB until these exist.
"""

from __future__ import annotations

import json
import queue
import sqlite3
import threading
import time
from collections.abc import Callable, Iterable
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any

from ..context import Context
from ..errors import StageFailed
from ..net import NetError, Reply, Request, Transport
from ..settings import Secret, need, read_secret

ATTEMPTS = 6
APPEND = {
    "movie": "credits,keywords,release_dates,external_ids,images,videos,translations",
    "tv": "credits,keywords,content_ratings,external_ids,images,videos,translations",
}
MAX_CAST = 10
MAX_DIRECTORS = 3
MAX_COMPANIES = 5
MAX_VIDEOS = 12
CREW_JOBS = frozenset(
    {"Director", "Writer", "Screenplay", "Story", "Producer", "Executive Producer", "Creator"}
)

SCHEMA = """
CREATE TABLE IF NOT EXISTS titles (
  media_type TEXT, id INTEGER, title TEXT, original_title TEXT, original_language TEXT,
  release_date TEXT, runtime INTEGER, tagline TEXT, overview TEXT, genres TEXT,
  vote_average REAL, vote_count INTEGER, popularity REAL, poster_path TEXT, backdrop_path TEXT,
  logo_path TEXT, directors TEXT, creators TEXT, cast_members TEXT, crew TEXT, keywords TEXT,
  certs TEXT, origin_country TEXT, production_companies TEXT, company_logos TEXT, networks TEXT,
  network_logos TEXT, spoken_languages TEXT, collection TEXT, tv_type TEXT, seasons INTEGER,
  episodes INTEGER, status TEXT, adult INTEGER, imdb_id TEXT, videos TEXT, translations TEXT,
  source TEXT, fetched_at INTEGER, art_path TEXT, PRIMARY KEY (media_type, id));
CREATE TABLE IF NOT EXISTS seasons (
  id INTEGER, season_number INTEGER, name TEXT, overview TEXT, air_date TEXT, poster_path TEXT,
  episode_count INTEGER, PRIMARY KEY (id, season_number));
CREATE TABLE IF NOT EXISTS episodes (
  id INTEGER, season_number INTEGER, episode_number INTEGER, name TEXT, overview TEXT,
  air_date TEXT, still_path TEXT, runtime INTEGER, vote_average REAL,
  PRIMARY KEY (id, season_number, episode_number));
CREATE TABLE IF NOT EXISTS fetch_log (
  media_type TEXT, id INTEGER, status INTEGER, PRIMARY KEY (media_type, id));
"""

COLUMNS = (
    "media_type id title original_title original_language release_date runtime tagline overview "
    "genres vote_average vote_count popularity poster_path backdrop_path logo_path directors "
    "creators cast_members crew keywords certs origin_country production_companies company_logos "
    "networks network_logos spoken_languages collection tv_type seasons episodes status adult "
    "imdb_id videos translations source fetched_at art_path"
).split()
# Columns added after the first pull files were made, with their type. Opening a file adds them.
ADDED_COLUMNS = (("art_path", "TEXT"),)


def _j(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False)


def pick_logo(logos: list[dict[str, Any]]) -> str | None:
    """The English logo, else one without text, else any."""
    english = (x for x in logos if x.get("iso_639_1") == "en")
    no_text = (x for x in logos if x.get("iso_639_1") is None)
    pick = next(english, None) or next(no_text, None) or (logos[0] if logos else None)
    return pick.get("file_path") if pick else None


def pick_art(posters: list[dict[str, Any]]) -> str | None:
    """A poster with no words on it, for a tall screen: the best rated, then the most voted, then
    the largest. None when every poster carries text."""
    plain = [x for x in posters if x.get("iso_639_1") is None and x.get("file_path")]
    if not plain:
        return None
    best = max(
        plain,
        key=lambda x: (x.get("vote_average") or 0, x.get("vote_count") or 0, x.get("width") or 0),
    )
    return best["file_path"]


def _companies(items: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        {"name": c.get("name"), "logo_path": c.get("logo_path")}
        for c in items
        if c.get("name") and c.get("logo_path")
    ]


def _translations(doc: dict[str, Any], languages: Iterable[str]) -> dict[str, dict[str, str]]:
    """Every region of each wanted language (`ar-SA` asks for Arabic from any region). The server
    picks the region its viewers read first and fills a gap from another."""
    wanted = {tag.split("-")[0].lower() for tag in languages}
    out: dict[str, dict[str, str]] = {}
    for t in (doc.get("translations") or {}).get("translations", []):
        tag = f"{t.get('iso_639_1')}-{t.get('iso_3166_1')}"
        if str(t.get("iso_639_1") or "").lower() not in wanted:
            continue
        data = t.get("data") or {}
        text = {
            "title": data.get("title") or data.get("name") or "",
            "overview": data.get("overview") or "",
            "tagline": data.get("tagline") or "",
        }
        if any(text.values()):
            out[tag] = text
    return out


def _videos(doc: dict[str, Any]) -> list[dict[str, Any]]:
    keep = []
    for v in (doc.get("videos") or {}).get("results", [])[:MAX_VIDEOS]:
        keep.append({k: v.get(k) for k in ("site", "key", "type", "name", "official", "size")})
    return keep


def parse_title(
    kind: str, doc: dict[str, Any], languages: Iterable[str], at: int
) -> dict[str, Any]:
    """One row of `titles` from a details answer."""
    credits = doc.get("credits") or {}
    crew = credits.get("crew") or []
    cast = [c["name"] for c in (credits.get("cast") or [])[:MAX_CAST]]
    if kind == "movie":
        directors = [c["name"] for c in crew if c.get("job") == "Director"][:MAX_DIRECTORS]
        keywords = [k["name"] for k in (doc.get("keywords") or {}).get("keywords", [])]
        certs: dict[str, str] = {}
        for region in (doc.get("release_dates") or {}).get("results", []):
            for release in region.get("release_dates", []):
                if release.get("certification"):
                    certs.setdefault(region["iso_3166_1"], release["certification"])
                    break
        row = {
            "title": doc.get("title"),
            "original_title": doc.get("original_title"),
            "release_date": doc.get("release_date"),
            "runtime": doc.get("runtime"),
            "directors": _j(directors),
            "creators": "[]",
            "networks": "[]",
            "network_logos": "[]",
            "origin_country": _j([c["iso_3166_1"] for c in doc.get("production_countries", [])]),
            "collection": (doc.get("belongs_to_collection") or {}).get("name"),
            "tv_type": None,
            "seasons": None,
            "episodes": None,
        }
    else:
        keywords = [k["name"] for k in (doc.get("keywords") or {}).get("results", [])]
        certs = {
            r["iso_3166_1"]: r["rating"]
            for r in (doc.get("content_ratings") or {}).get("results", [])
            if r.get("rating")
        }
        runs = sorted(doc.get("episode_run_time") or [])
        row = {
            "title": doc.get("name"),
            "original_title": doc.get("original_name"),
            "release_date": doc.get("first_air_date"),
            "runtime": runs[len(runs) // 2] if runs else None,
            "directors": "[]",
            "creators": _j([c["name"] for c in doc.get("created_by", [])]),
            "networks": _j([n["name"] for n in doc.get("networks", [])]),
            "network_logos": _j(_companies(doc.get("networks", []))),
            "origin_country": _j(doc.get("origin_country") or []),
            "collection": None,
            "tv_type": doc.get("type"),
            "seasons": doc.get("number_of_seasons"),
            "episodes": doc.get("number_of_episodes"),
        }
    companies = doc.get("production_companies", [])
    row.update(
        media_type=kind,
        id=doc["id"],
        original_language=doc.get("original_language"),
        tagline=doc.get("tagline"),
        overview=doc.get("overview"),
        genres=", ".join(g["name"] for g in doc.get("genres", [])),
        vote_average=doc.get("vote_average"),
        vote_count=doc.get("vote_count"),
        popularity=doc.get("popularity"),
        poster_path=doc.get("poster_path"),
        backdrop_path=doc.get("backdrop_path"),
        logo_path=pick_logo((doc.get("images") or {}).get("logos", [])),
        art_path=pick_art((doc.get("images") or {}).get("posters", [])),
        cast_members=_j(cast),
        crew=_j(
            [
                {"name": c.get("name"), "job": c.get("job")}
                for c in crew
                if c.get("job") in CREW_JOBS
            ]
        ),
        keywords=_j(keywords),
        certs=_j(certs),
        production_companies=_j([c["name"] for c in companies][:MAX_COMPANIES]),
        company_logos=_j(_companies(companies)[:MAX_COMPANIES]),
        spoken_languages=_j([x["iso_639_1"] for x in doc.get("spoken_languages", [])]),
        status=doc.get("status"),
        adult=int(bool(doc.get("adult"))),
        imdb_id=doc.get("imdb_id") or (doc.get("external_ids") or {}).get("imdb_id"),
        videos=_j(_videos(doc)),
        translations=_j(_translations(doc, languages)),
        source="tmdb",
        fetched_at=at,
    )
    return row


def parse_season(title_id: int, doc: dict[str, Any]) -> tuple[tuple, list[tuple]]:
    """A row of `seasons` and the rows of `episodes` from a season answer."""
    number = doc.get("season_number")
    episodes = doc.get("episodes") or []
    season = (
        title_id,
        number,
        doc.get("name"),
        doc.get("overview"),
        doc.get("air_date"),
        doc.get("poster_path"),
        len(episodes),
    )
    rows = [
        (
            title_id,
            number,
            e.get("episode_number"),
            e.get("name"),
            e.get("overview"),
            e.get("air_date"),
            e.get("still_path"),
            e.get("runtime"),
            e.get("vote_average"),
        )
        for e in episodes
    ]
    return season, rows


class Limiter:
    """Evenly spaced calls. Slows by a quarter on a 429, recovers by 1 percent per good answer."""

    FLOOR = 8.0

    def __init__(
        self,
        rps: float,
        clock: Callable[[], float] = time.monotonic,
        sleep: Callable[[float], None] = time.sleep,
    ) -> None:
        self.rate = rps
        self.target = rps
        self.clock = clock
        self.sleep = sleep
        self.lock = threading.Lock()
        self.next = clock()

    def wait(self) -> None:
        with self.lock:
            now = self.clock()
            at = max(self.next, now)
            self.next = at + 1.0 / self.rate
        if at > now:
            self.sleep(at - now)

    def slow(self) -> None:
        self.rate = max(min(self.FLOOR, self.target), self.rate * 0.75)

    def fast(self) -> None:
        self.rate = min(self.target, self.rate * 1.01)


@dataclass
class Counts:
    ok: int = 0
    gone: int = 0
    failed: int = 0
    limited: int = 0
    purged: int = 0
    extra: dict[str, int] = field(default_factory=dict)

    def text(self) -> str:
        return (
            f"{self.ok} ok, {self.gone} gone, {self.failed} failed, "
            f"{self.limited} rate limited, {self.purged} purged"
        )


class Client:
    """Calls with the old pull's retry rules. A wrong key stops the whole run."""

    def __init__(
        self,
        api_base: str,
        key: Secret,
        transport: Transport,
        limiter: Limiter,
        sleep: Callable[[float], None] = time.sleep,
    ) -> None:
        self.api_base = api_base
        self.key = key
        self.transport = transport
        self.limiter = limiter
        self.sleep = sleep
        self.counts = Counts()

    def get(self, path: str, **params: Any) -> Reply | None:
        """The answer, or None when every attempt failed. A 404 comes back as a reply."""
        query = {"api_key": self.key.reveal(), "language": "en-US", **params}
        for attempt in range(ATTEMPTS):
            self.limiter.wait()
            try:
                reply = self.transport.fetch(Request(self.api_base, path, query))
            except NetError:
                self.sleep(1 + attempt)
                continue
            if reply.status in (200, 404):
                self.limiter.fast()
                return reply
            if reply.status == 429:
                self.counts.limited += 1
                self.limiter.slow()
                self.sleep(float(reply.headers.get("retry-after", 2)))
                continue
            if reply.status in (401, 403):
                raise StageFailed(f"tmdb: the key was refused (status {reply.status})")
            self.sleep(1 + attempt)
        return None


def fetch_title(
    client: Client, kind: str, title_id: int, languages: tuple[str, ...], at: int
) -> tuple[int, dict[str, Any] | None, list[tuple], list[tuple]]:
    """Status, title row, season rows, episode rows. Status -1 means try again in a later run."""
    params = {"append_to_response": APPEND[kind], "include_image_language": "en,null"}
    reply = client.get(f"/{kind}/{title_id}", **params)
    if reply is None:
        return -1, None, [], []
    if reply.status == 404:
        return 404, None, [], []
    doc = reply.json()
    row = parse_title(kind, doc, languages, at)
    seasons: list[tuple] = []
    episodes: list[tuple] = []
    if kind == "tv":
        for s in doc.get("seasons") or []:
            number = s.get("season_number")
            answer = client.get(f"/tv/{title_id}/season/{number}")
            if answer is None:
                return -1, None, [], []
            if answer.status == 200:
                season, rows = parse_season(title_id, answer.json())
                seasons.append(season)
                episodes.extend(rows)
    return 200, row, seasons, episodes


def open_pull(path: Path) -> sqlite3.Connection:
    path.parent.mkdir(parents=True, exist_ok=True)
    db = sqlite3.connect(path, check_same_thread=False)
    db.executescript(SCHEMA)
    have = {row[1] for row in db.execute("PRAGMA table_info(titles)")}
    for name, kind in ADDED_COLUMNS:
        if name not in have:
            db.execute(f"ALTER TABLE titles ADD COLUMN {name} {kind}")
    db.execute("PRAGMA journal_mode=WAL")
    return db


def _store(
    db: sqlite3.Connection,
    kind: str,
    title_id: int,
    status: int,
    row: dict[str, Any] | None,
    seasons: list[tuple],
    episodes: list[tuple],
    counts: Counts,
) -> None:
    if status == 200 and row is not None:
        counts.ok += 1
        marks = ",".join("?" * len(COLUMNS))
        names = ",".join(COLUMNS)
        db.execute(
            f"INSERT OR REPLACE INTO titles ({names}) VALUES ({marks})",
            [row[c] for c in COLUMNS],
        )
        if kind == "tv":
            db.execute("DELETE FROM seasons WHERE id = ?", (title_id,))
            db.execute("DELETE FROM episodes WHERE id = ?", (title_id,))
            db.executemany("INSERT OR REPLACE INTO seasons VALUES (?,?,?,?,?,?,?)", seasons)
            db.executemany("INSERT OR REPLACE INTO episodes VALUES (?,?,?,?,?,?,?,?,?)", episodes)
    elif status == 404:
        counts.gone += 1
        had = db.execute(
            "DELETE FROM titles WHERE media_type = ? AND id = ?", (kind, title_id)
        ).rowcount
        counts.purged += had
        if kind == "tv":
            db.execute("DELETE FROM seasons WHERE id = ?", (title_id,))
            db.execute("DELETE FROM episodes WHERE id = ?", (title_id,))
    else:
        counts.failed += 1
    if status in (200, 404):
        db.execute("INSERT OR REPLACE INTO fetch_log VALUES (?,?,?)", (kind, title_id, status))


def pull_titles(
    ctx: Context,
    client: Client,
    work: Iterable[tuple[str, int]],
    db: sqlite3.Connection,
    workers: int,
) -> Counts:
    """Fetches each title on worker threads and writes on this one."""
    todo = list(work)
    tasks: queue.Queue[tuple[str, int] | None] = queue.Queue(maxsize=2000)
    results: queue.Queue[Any] = queue.Queue()
    at = int(ctx.now().timestamp())
    languages = ctx.settings.tmdb.languages

    def worker() -> None:
        while (task := tasks.get()) is not None:
            try:
                results.put((task, *fetch_title(client, *task, languages, at)))
            except StageFailed as e:
                results.put((task, e))
            except Exception as e:
                # An unexpected answer shape raises here. Every task must leave a result, or the
                # collector waits for one that never comes and the job hangs instead of ending.
                results.put((task, e))

    threads = [threading.Thread(target=worker, daemon=True) for _ in range(workers)]
    for t in threads:
        t.start()

    def feeder() -> None:
        for task in todo:
            tasks.put(task)
        for _ in threads:
            tasks.put(None)

    threading.Thread(target=feeder, daemon=True).start()
    counts = client.counts
    unreadable = 0
    for done in range(1, len(todo) + 1):
        task, *rest = results.get()
        if isinstance(rest[0], StageFailed):
            db.commit()
            raise rest[0]
        if isinstance(rest[0], Exception):
            # An answer this build cannot read counts as a failed fetch, the way an answer that
            # never came does: no fetch log row is written, so a later run asks again and one odd
            # shape cannot stop a whole set. The first is named and the count is said at the end.
            unreadable += 1
            if unreadable == 1:
                kind, title_id = task
                reason = f"{type(rest[0]).__name__}: {rest[0]}"
                ctx.say(f"pull: {kind} {title_id} answered what this build cannot read: {reason}")
            rest = [-1, None, [], []]
        _store(db, *task, *rest, counts)
        if done % 500 == 0:
            db.commit()
            ctx.say(f"pull: {done} of {len(todo)}: {counts.text()}")
    db.commit()
    if unreadable:
        ctx.say(f"pull: {unreadable} answers could not be read and count as failed fetches")
    return counts


def make_client(ctx: Context) -> Client:
    tmdb = ctx.settings.tmdb
    base = need(tmdb.api_base, "tmdb.api_base")
    key = read_secret(tmdb.key_file, "tmdb.key_file")
    return Client(base, key, ctx.transport, Limiter(tmdb.rps))


def pull_db(ctx: Context) -> Path:
    return ctx.work / "pull.db"


def stale_titles(db: sqlite3.Connection, cutoff: datetime) -> list[tuple[str, int]]:
    """Titles fetched before the cut off, oldest first."""
    rows = db.execute(
        "SELECT media_type, id FROM titles WHERE fetched_at < ?"
        " ORDER BY fetched_at, media_type, id",
        (int(cutoff.timestamp()),),
    )
    return [(kind, title_id) for kind, title_id in rows]
