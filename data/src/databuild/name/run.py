"""Runs the name stage."""

from __future__ import annotations

from ..context import Context
from ..embed.backends import make_embedder
from .events import write_events
from .groups import name_groups, name_seasons
from .llm import make_chat
from .profiles import write_profiles
from .rows import write_rows
from .taxonomy import write_taxonomy
from .topics import write_topics


def name(ctx: Context) -> None:
    chat = make_chat(ctx)
    if chat is None:
        ctx.say("name: no text model, name.endpoint is not set; names come from the data")
    name_groups(ctx, chat)
    name_seasons(ctx, chat)
    events = write_events(ctx)
    write_rows(ctx, events)
    write_taxonomy(ctx, make_embedder(ctx))
    write_profiles(ctx, chat)
    write_topics(ctx, chat)
