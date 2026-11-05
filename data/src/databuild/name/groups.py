"""Names for groups and seasonal sub groups.

With a text model each group gets a name written from its measured contents. Without one a name is
made from the same facts. Two groups of one level never share a name.
"""

from __future__ import annotations

import collections
import hashlib
import json
import re
import sqlite3
from pathlib import Path
from typing import Any

from ..context import Context
from ..keys import item_key
from ..vocab import load
from .llm import Chat

NAMED = (
    ("world", 1),
    ("world", 2),
    ("theme", 1),
    ("theme", 2),
    ("theme", 3),
    ("vibe", 1),
    ("vibe", 2),
)
MAX_WORDS = 6
MIN_SEASON_GROUP = 8
COUNTRY_SHARE = 0.35
DECADE_SHARE = 0.40
GENRE_SHARE = 0.4

SYSTEM = (
    "You write the row titles of a video service's home page. A row title tells a viewer in two to "
    "four words what the titles in the row have in common."
)
RULES = (
    "Write one row title for this group of titles. Rules: 2 to 4 words, Title Case, plain words a "
    "viewer would use. Name the kind of story, subject, setting, tone or tradition the titles "
    "share, using the genres, countries, decades and example titles as evidence. Be specific. Do "
    "not use a single title's name unless the group is clearly one franchise. Do not use the words "
    "Group, Collection or Titles. If the group is mostly one country or language, say so. If it is "
    "a mix of formats such as concerts, shorts or news, name that format. Output only the title."
)
SEASON_RULES = (
    "Write one row title for this sub group of a seasonal collection, to appear on the home page "
    "during that season. It must tell a viewer what kind of titles these are inside the season. "
    "Rules: 2 to 4 words, Title Case, plain words, honest to the genres and example titles, "
    "distinguishing this sub group from the others of the season. Do not use a person's name or "
    "the words Group, Collection, Titles. Output only the title."
)
COUNTRY_ADJECTIVE = {
    "US": "American",
    "GB": "British",
    "FR": "French",
    "DE": "German",
    "JP": "Japanese",
    "KR": "Korean",
    "CN": "Chinese",
    "HK": "Hong Kong",
    "TW": "Taiwanese",
    "IN": "Indian",
    "IT": "Italian",
    "ES": "Spanish",
    "MX": "Mexican",
    "BR": "Brazilian",
    "RU": "Russian",
    "SE": "Swedish",
    "TR": "Turkish",
    "AR": "Argentine",
    "CA": "Canadian",
    "AU": "Australian",
    "PL": "Polish",
    "NL": "Dutch",
    "DK": "Danish",
    "NO": "Norwegian",
    "FI": "Finnish",
    "PH": "Filipino",
    "TH": "Thai",
    "ID": "Indonesian",
    "EG": "Egyptian",
    "IR": "Iranian",
    "GR": "Greek",
    "CZ": "Czech",
    "HU": "Hungarian",
    "SU": "Soviet",
    "AT": "Austrian",
    "BE": "Belgian",
    "PT": "Portuguese",
    "IE": "Irish",
    "NZ": "New Zealand",
    "ZA": "South African",
    "NG": "Nigerian",
    "CL": "Chilean",
    "CO": "Colombian",
    "UA": "Ukrainian",
}


def prompt_for(stats: dict[str, Any], size: int) -> str:
    genres = "; ".join(f"{k} {s:.0%} ({lift:.1f}x)" for k, s, lift in stats["genres"])
    countries = ", ".join(f"{k} {s:.0%}" for k, s, _ in stats["countries"][:3])
    languages = ", ".join(f"{k} {c / size:.0%}" for k, c in stats["languages"])
    decades = ", ".join(f"{k}s {c / size:.0%}" for k, c in stats["decades"])
    return (
        f"Group of {size} titles ({stats['tv_share']:.0%} TV series), "
        f"median year {stats['median_year'] or 'unknown'}.\n"
        f"Genres (share, how much more common than in the whole catalog): {genres}.\n"
        f"Main countries: {countries or 'unknown'}. "
        f"Main original languages: {languages or 'unknown'}.\n"
        f"Decades: {decades}.\n"
        f"Most popular titles: {'; '.join(stats['examples'])}.\n"
        f"Random titles: {'; '.join(stats['random'])}."
    )


def fix_name(name: str) -> str:
    """Repairs a place the model cut short."""
    return re.sub(r"\bHong\b(?! Kong)", "Hong Kong", name.strip())


def clean_name(text: str) -> str:
    line = text.strip().split("\n")[0].strip(" \"'*.")
    line = re.sub(r"^(title|row title|name)\s*:\s*", "", line, flags=re.IGNORECASE).strip(" \"'*.")
    return fix_name(" ".join(line.split()[:MAX_WORDS]))


def decade_label(decade: int) -> str:
    return f"{decade}s" if decade >= 2000 else f"{decade % 100}s"


def fallback_name(stats: dict[str, Any], size: int) -> str:
    """A name from the facts: decade and genre, two genres, one genre, or the lead title."""
    genres = [g[0] for g in stats["genres"]]
    decades = stats["decades"]
    if genres and decades and decades[0][1] / size >= DECADE_SHARE:
        return f"{decade_label(decades[0][0])} {genres[0]}"
    if len(genres) >= 2:
        return f"{genres[0]} & {genres[1]}"
    if genres:
        return f"{genres[0]} Picks"
    lead = stats["examples"][0].rsplit(" (", 1)[0] if stats["examples"] else "This Group"
    return f"Like {lead}"


def _qualifier(stats: dict[str, Any], size: int) -> dict[str, str]:
    found: dict[str, str] = {}
    countries = stats.get("countries") or []
    if countries and countries[0][1] >= COUNTRY_SHARE and countries[0][0] in COUNTRY_ADJECTIVE:
        found["country"] = COUNTRY_ADJECTIVE[countries[0][0]]
    decades = stats.get("decades") or []
    if decades and size and decades[0][1] / size >= DECADE_SHARE:
        found["decade"] = f"{decades[0][0]}s"
    return found


def qualify_shared(names: dict[str, str], facts: dict[str, tuple[dict, int]]) -> int:
    """Groups of one level that share a name take a country or decade only they hold."""
    shared: dict[tuple[str, str, str], list[str]] = collections.defaultdict(list)
    for key, name in names.items():
        kind, level, _ = key.split("|")
        if kind != "vibe":
            shared[(kind, level, name)].append(key)
    changed = 0
    for (_, _, name), keys in shared.items():
        if len(keys) < 2:
            continue
        info = {k: _qualifier(*facts[k]) for k in keys}
        used: set[tuple[str, str]] = set()
        for key in keys:
            pick = None
            for field in ("country", "decade"):
                value = info[key].get(field)
                others = [info[o].get(field) for o in keys if o != key]
                if value and value not in others:
                    pick = value
                    break
            if pick and pick.lower() not in name.lower() and (pick, name) not in used:
                names[key] = f"{pick} {name}"
                used.add((pick, name))
                changed += 1
    return changed


def settle_vibe(names: dict[str, str], facts: dict[str, tuple[dict, int]]) -> tuple[int, int]:
    """The largest group keeps a shared vibe name. The others take a qualifier or stay unnamed."""
    shared: dict[tuple[str, str], list[str]] = collections.defaultdict(list)
    for key, name in names.items():
        kind, level, _ = key.split("|")
        if kind == "vibe":
            shared[(level, name.lower())].append(key)
    taken = {name.lower() for name in names.values()}
    qualified = dropped = 0
    for keys in shared.values():
        keys.sort(key=lambda k: -facts[k][1])
        for key in keys[1:]:
            stats, size = facts[key]
            name = names[key]
            pick = None
            decades = stats.get("decades") or []
            genres = stats.get("genres") or []
            if decades and size and decades[0][1] / size >= DECADE_SHARE:
                pick = f"{decades[0][0]}s"
            elif (
                genres and genres[0][1] >= GENRE_SHARE and genres[0][0].lower() not in name.lower()
            ):
                pick = genres[0][0]
            full = f"{pick} {name}" if pick else ""
            if pick and pick.lower() not in name.lower() and full.lower() not in taken:
                names[key] = full
                taken.add(full.lower())
                qualified += 1
            else:
                del names[key]
                dropped += 1
    return qualified, dropped


def _load_saved(path: Path) -> dict[str, dict[str, str]]:
    return json.loads(path.read_text()) if path.exists() else {}


def name_groups(ctx: Context, chat: Chat | None) -> None:
    """Fills `groups.name`. A name stays while its group's measured facts stay the same."""
    path = ctx.work / "names.json"
    saved = _load_saved(path)
    db = sqlite3.connect(ctx.work / "groups.db")
    names: dict[str, str] = {}
    facts: dict[str, tuple[dict, int]] = {}
    written = 0
    for kind, level in NAMED:
        for gid, size, text in db.execute(
            "SELECT id, size, stats FROM groups WHERE kind = ? AND level = ? ORDER BY id",
            (kind, level),
        ):
            stats = json.loads(text)
            key = f"{kind}|{level}|{gid}"
            facts[key] = (stats, size)
            digest = hashlib.sha256(text.encode()).hexdigest()[:12]
            held = saved.get(key)
            if held and held["hash"] == digest and (held["model"] == bool(chat)):
                names[key] = held["name"]
                continue
            if chat:
                prompt = f"{prompt_for(stats, size)}\n\n{RULES}"
                name = clean_name(chat.ask(SYSTEM, prompt)) or fallback_name(stats, size)
            else:
                name = fallback_name(stats, size)
            names[key] = name
            saved[key] = {"hash": digest, "model": bool(chat), "name": name}
            written += 1
            if written % 100 == 0:
                path.write_text(json.dumps(saved, ensure_ascii=False))
    path.write_text(json.dumps(saved, ensure_ascii=False))
    changed = qualify_shared(names, facts)
    qualified, dropped = settle_vibe(names, facts)
    db.execute("UPDATE groups SET name = NULL")
    for key, name in names.items():
        kind, level, gid = key.split("|")
        db.execute(
            "UPDATE groups SET name = ? WHERE kind = ? AND level = ? AND id = ?",
            (name, kind, int(level), int(gid)),
        )
    db.commit()
    db.close()
    ctx.say(
        f"name: {len(names)} groups named, {written} written now, {changed + qualified} qualified, "
        f"{dropped} left unnamed"
    )


def event_label(event: str) -> str:
    labels = load("events")["labels"]
    return labels.get(event) or event.replace("_", " ").title()


def name_seasons(ctx: Context, chat: Chat | None) -> None:
    """Names for season sub groups. Inside one event a name is used once."""
    db = sqlite3.connect(ctx.work / "groups.db")
    titles = sqlite3.connect(f"file:{ctx.work / 'titles.db'}?mode=ro", uri=True)
    info = {
        r[0]: r[1:]
        for r in titles.execute("SELECT id, title, release_year, genres, popularity FROM titles")
    }
    titles.close()
    subs: dict[tuple[str, int], list[int]] = collections.defaultdict(list)
    for event, sub, kind, title_id in db.execute(
        "SELECT event, sub, media_type, tmdb_id FROM season_members ORDER BY event, sub, rank"
    ):
        subs[(event, sub)].append(item_key(kind, title_id))
    order = sorted(subs, key=lambda k: (-len(subs[k]), k))
    seen: set[tuple[str, str]] = set()
    kept = 0
    db.execute("DELETE FROM season_groups")
    for event, sub in order:
        members = subs[(event, sub)]
        if len(members) < MIN_SEASON_GROUP:
            continue
        genres: collections.Counter = collections.Counter()
        for key in members:
            genres.update(g.strip() for g in (info[key][2] or "").split(",") if g.strip())
        top = [g for g, _ in genres.most_common(4)]
        if chat:
            ranked = sorted(members, key=lambda k: -(info[k][3] or 0))[:10]
            lines = [f"{info[k][0]} ({info[k][1]})" for k in ranked]
            prompt = (
                f"Season or occasion: {event.replace('_', ' ')}.\n"
                f"Sub group of {len(members)} titles. Genres: {', '.join(top)}.\n"
                f"Most popular titles: {'; '.join(lines)}.\n\n{SEASON_RULES}"
            )
            name = clean_name(chat.ask(SYSTEM, prompt))
        else:
            name = f"{event_label(event)} {top[0]}" if top else event_label(event)
        if name and (event, name.lower()) not in seen:
            seen.add((event, name.lower()))
            db.execute("INSERT INTO season_groups VALUES (?,?,?)", (event, sub, name))
            kept += 1
    db.commit()
    db.close()
    ctx.say(f"name: {kept} season sub groups named of {len(order)}")
