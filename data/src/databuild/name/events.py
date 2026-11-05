"""Calendar events as pages: label, window, keywords, genres, languages (S107 reads these)."""

from __future__ import annotations

import json
from typing import Any

from ..context import Context
from ..vocab import load

EVENTS_FILE = "events.json"


def build_events() -> dict[str, dict[str, Any]]:
    vocab = load("events")
    labels = vocab["labels"]
    out: dict[str, dict[str, Any]] = {}
    for entry in load("calendar")["entries"]:
        event, window = entry.get("event"), entry.get("seasonal")
        if event not in labels or not isinstance(window, dict):
            continue
        held = out.setdefault(
            event,
            {
                "label": labels[event],
                "keywords": set(),
                "season": {"start": window["start"], "end": window["end"]},
            },
        )
        held["keywords"] |= {k.lower() for k in entry.get("keywords", [])}
    for event, words in vocab["extra_keywords"].items():
        if event in out:
            out[event]["keywords"] |= set(words)
    for event, words in vocab["drop_keywords"].items():
        if event in out:
            out[event]["keywords"] -= set(words)
    final = {}
    for event, held in sorted(out.items()):
        languages = vocab["languages"].get(event, [])
        if held["keywords"] or languages:
            final[event] = {
                "label": held["label"],
                "season": held["season"],
                "keywords": sorted(held["keywords"]),
                "genres": vocab["genres"].get(event, []),
                "languages": languages,
            }
    return final


def write_events(ctx: Context) -> dict[str, dict[str, Any]]:
    events = build_events()
    target = ctx.work / "export" / EVENTS_FILE
    target.parent.mkdir(exist_ok=True)
    document = {"version": 1, "events": events}
    target.write_text(json.dumps(document, indent=1, ensure_ascii=False) + "\n")
    return events
