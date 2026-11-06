"""Short written profiles of quality titles, read by page and age rules. Needs a text model."""

from __future__ import annotations

import json
import sqlite3
from typing import Any

from ..context import Context
from ..keys import title_key
from .llm import Chat

PROFILES_FILE = "profiles.jsonl"
SAVE_EVERY = 50
AUDIENCES = ("kids", "family", "teen", "adult")
SYSTEM = (
    "You write structured profiles of films and TV series for a recommendation engine. Use only "
    "the facts given and general knowledge you are certain about. When unsure write unknown. Never "
    "mention the title or any character name. No spoilers. Plain words."
)
ASK = (
    "Write the profile as JSON with: summary (at most 40 words, what the story is about), themes "
    "(3 to 6 short phrases such as found family, grief, class conflict), tone (2 to 4 adjectives), "
    "audience (kids, family, teen or adult), who (the main characters in at most 10 words, no "
    "names), setting (place and period in at most 8 words), lgbtq (central if LGBTQ+ characters or "
    "themes are central to the story, present if a notable part, none, unknown), content (the "
    "content flags that apply: violence, sex, nudity, horror, drugs, strong language, war, abuse, "
    "suicide)."
)
QUERY = (
    "SELECT media_type, tmdb_id, title, release_year, genres, origin_country, original_language,"
    " certs, director, cast_members, keywords, tagline, overview FROM titles"
    " WHERE adult = 0 AND vote_average >= 6.0 AND popularity >= 2.0 AND length(overview) >= 60"
    " AND poster_path IS NOT NULL AND poster_path <> '' ORDER BY popularity DESC, id"
)


def parse_profile(text: str) -> dict[str, Any]:
    """The JSON inside an answer, or an empty profile when none can be read."""
    start, end = text.find("{"), text.rfind("}")
    try:
        value = json.loads(text[start : end + 1])
    except ValueError:
        return {}
    return value if isinstance(value, dict) else {}


def user_text(row: tuple) -> str:
    (
        kind,
        _,
        title,
        year,
        genres,
        country,
        language,
        certs,
        director,
        cast,
        kw,
        tagline,
        overview,
    ) = row
    held = json.loads(certs) if certs else {}
    cert = ", ".join(f"{k} {v}" for k, v in held.items() if k in ("GB", "US"))
    label = "TV series" if kind == "tv" else "film"
    return (
        f"Title: {title} ({year}), {label}\nGenres: {genres}\nCountry: {country}  "
        f"Language: {language}  Certificate: {cert or 'none'}\n"
        f"Director or creator: {director or 'unknown'}\nCast: {cast}\nKeywords: {kw}\n"
        f"Tagline: {tagline}\nOverview: {overview}\n\n{ASK}"
    )


def write_profiles(ctx: Context, chat: Chat | None) -> None:
    if chat is None:
        ctx.say("name: profiles off, name.endpoint is not set; no written profiles in this set")
        return
    path = ctx.work / PROFILES_FILE
    done: dict[str, str] = {}
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            done[json.loads(line)["k"]] = line
    db = sqlite3.connect(f"file:{ctx.work / 'titles.db'}?mode=ro", uri=True)
    rows = db.execute(QUERY).fetchall()
    db.close()
    unreadable = 0
    with open(path, "a", encoding="utf-8") as out:
        for count, row in enumerate(rows, 1):
            key = title_key(row[0], row[1])
            if key in done:
                continue
            profile = parse_profile(chat.ask(SYSTEM, user_text(row), max_tokens=420))
            unreadable += not profile
            out.write(json.dumps({"k": key, "p": profile}, ensure_ascii=False) + "\n")
            if count % SAVE_EVERY == 0:
                out.flush()
    ctx.say(f"name: {len(rows)} profiles, {unreadable} unreadable answers this run")
