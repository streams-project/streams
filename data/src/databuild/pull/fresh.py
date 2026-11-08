"""New, upcoming and trending titles into `fresh.json`, every six hours (S150, S220)."""

from __future__ import annotations

import json
import os
import time
from collections.abc import Callable
from datetime import timedelta
from typing import Any

from ..context import Context
from ..errors import StageFailed
from ..net import NetError, Request
from ..settings import need, read_secret

ATTEMPTS = 4
BACKOFF = 1.5
TIMEOUT = 30.0
PAST_DAYS = 90
SOON_DAYS = 180
# Series with an episode in this many days are looked up for a new season.
SEASON_DAYS = 60
FRESH_FILE = "fresh.json"


def lists(ctx: Context) -> list[tuple[str, str | None, str, dict[str, str], int]]:
    """Tag, kind, path, parameters and pages of each list."""
    today = ctx.as_of
    past = (today - timedelta(days=PAST_DAYS)).isoformat()
    soon = (today + timedelta(days=SOON_DAYS)).isoformat()
    next_day = (today + timedelta(days=1)).isoformat()
    by_popularity = {"sort_by": "popularity.desc"}
    new_movie = {
        **by_popularity,
        "primary_release_date.gte": past,
        "primary_release_date.lte": today.isoformat(),
    }
    new_tv = {
        **by_popularity,
        "first_air_date.gte": past,
        "first_air_date.lte": today.isoformat(),
    }
    soon_movie = {
        **by_popularity,
        "primary_release_date.gte": next_day,
        "primary_release_date.lte": soon,
    }
    soon_tv = {**by_popularity, "first_air_date.gte": next_day, "first_air_date.lte": soon}
    season_end = (today + timedelta(days=SEASON_DAYS)).isoformat()
    airing_tv = {**by_popularity, "air_date.gte": next_day, "air_date.lte": season_end}
    return [
        ("now_playing", "movie", "/movie/now_playing", {}, 5),
        ("upcoming", "movie", "/movie/upcoming", {}, 5),
        ("on_the_air", "tv", "/tv/on_the_air", {}, 5),
        ("airing_today", "tv", "/tv/airing_today", {}, 3),
        ("trending_day", None, "/trending/all/day", {}, 3),
        ("trending_week", None, "/trending/all/week", {}, 3),
        ("new_movie", "movie", "/discover/movie", new_movie, 5),
        ("new_tv", "tv", "/discover/tv", new_tv, 5),
        ("soon_movie", "movie", "/discover/movie", soon_movie, 5),
        ("soon_tv", "tv", "/discover/tv", soon_tv, 3),
        ("airing_tv", "tv", "/discover/tv", airing_tv, 5),
    ]


def next_season(doc: dict[str, Any] | None) -> dict[str, Any] | None:
    """The next season of a series that has already aired, when its first episode has a date."""
    upcoming = (doc or {}).get("next_episode_to_air") or {}
    number = upcoming.get("season_number") or 0
    if upcoming.get("episode_number") == 1 and number >= 2 and upcoming.get("air_date"):
        return {"number": number, "air_date": upcoming["air_date"]}
    return None


def build_fresh(ctx: Context, sleep: Callable[[float], None] = time.sleep) -> dict[str, Any]:
    tmdb = ctx.settings.tmdb
    base = need(tmdb.api_base, "tmdb.api_base")
    key = read_secret(tmdb.key_file, "tmdb.key_file")

    def ask(path: str, **params: Any) -> dict[str, Any] | None:
        """The answer, None for a title TMDB does not have, and a failure after every attempt."""
        query = {"api_key": key.reveal(), "language": "en-US", **params}
        for attempt in range(1, ATTEMPTS + 1):
            try:
                reply = ctx.transport.fetch(Request(base, path, query, timeout=TIMEOUT))
                if reply.status == 200:
                    return reply.json()
                if reply.status == 404:
                    return None
            except NetError:
                pass
            sleep(BACKOFF * attempt)
        raise StageFailed(f"fresh: {path} failed after {ATTEMPTS} attempts")

    def get(path: str, **params: Any) -> dict[str, Any]:
        answer = ask(path, **params)
        if answer is None:
            raise StageFailed(f"fresh: {path} answered not found")
        return answer

    genres: dict[int, str] = {}
    for kind in ("movie", "tv"):
        for g in get(f"/genre/{kind}/list")["genres"]:
            genres[g["id"]] = g["name"]

    items: dict[tuple[str, int], dict[str, Any]] = {}
    for tag, kind, path, params, pages in lists(ctx):
        position = 0
        for page in range(1, pages + 1):
            data = get(path, page=page, **params)
            for r in data.get("results", []):
                media = kind or r.get("media_type")
                if media not in ("movie", "tv"):
                    continue
                released = r.get("release_date") if media == "movie" else r.get("first_air_date")
                entry = items.setdefault(
                    (media, r["id"]),
                    {
                        "media_type": media,
                        "id": r["id"],
                        "title": r.get("title") or r.get("name") or "",
                        "original_language": r.get("original_language"),
                        "overview": r.get("overview") or "",
                        "poster_path": r.get("poster_path"),
                        "backdrop_path": r.get("backdrop_path"),
                        "genres": [genres[g] for g in r.get("genre_ids", []) if g in genres],
                        "vote_average": r.get("vote_average") or 0,
                        "vote_count": r.get("vote_count") or 0,
                        "popularity": r.get("popularity") or 0,
                        "release_date": released or "",
                        "origin_country": r.get("origin_country") or [],
                        "adult": bool(r.get("adult")),
                        "lists": [],
                        "ranks": {},
                    },
                )
                if tag not in entry["lists"]:
                    entry["lists"].append(tag)
                    entry["ranks"][tag] = position
                position += 1
            if page >= data.get("total_pages", 1):
                break
    today = ctx.as_of.isoformat()
    for entry in items.values():
        aired = entry["release_date"] and entry["release_date"] <= today
        if entry["media_type"] == "tv" and aired:
            season = next_season(ask(f"/tv/{entry['id']}"))
            if season and season["air_date"] > today:
                entry["next_season"] = season
    rows = sorted(items.values(), key=lambda e: (e["media_type"], e["id"]))
    return {"generated": ctx.now().isoformat(timespec="seconds"), "items": rows}


def pull_fresh(ctx: Context, sleep: Callable[[float], None] = time.sleep) -> None:
    """Writes the file whole. A pull with no items leaves the old file in place."""
    document = build_fresh(ctx, sleep)
    if not document["items"]:
        raise StageFailed("fresh: no items, old file kept")
    target = ctx.out / FRESH_FILE
    tmp = target.with_suffix(".json.tmp")
    tmp.write_text(json.dumps(document, ensure_ascii=True, separators=(",", ":")), encoding="utf-8")
    os.replace(tmp, target)
    dated = sum(1 for e in document["items"] if e["release_date"])
    seasons = sum(1 for e in document["items"] if e.get("next_season"))
    ctx.say(
        f"fresh: {len(document['items'])} items, {dated} with a release date, "
        f"{seasons} with a new season coming"
    )
