"""Details pull and the six month refresh, one work list each."""

from __future__ import annotations

from datetime import UTC, datetime, time, timedelta

from ..context import Context
from .exports import read_export, wanted
from .tmdb import make_client, open_pull, pull_db, pull_titles, stale_titles


def pull_details(ctx: Context, limit: int = 0) -> None:
    """New titles above the popularity floor, most popular first. Finished ids are skipped."""
    client = make_client(ctx)
    db = open_pull(pull_db(ctx))
    done = set(db.execute("SELECT media_type, id FROM fetch_log"))
    work: list[tuple[float, str, int]] = []
    for kind in ("movie", "tv"):
        export = read_export(ctx, kind)
        work.extend(wanted(export, ctx.settings.tmdb.min_popularity))
    work.sort(key=lambda t: (-t[0], t[1], t[2]))
    todo = [(kind, title_id) for _, kind, title_id in work if (kind, title_id) not in done]
    if limit:
        todo = todo[:limit]
    ctx.say(f"pull: {len(work)} wanted, {len(done)} already done, {len(todo)} to do")
    counts = pull_titles(ctx, client, todo, db, ctx.settings.tmdb.workers)
    ctx.say(f"pull: {counts.text()}")


def refresh_cutoff(ctx: Context) -> datetime:
    day = ctx.as_of - timedelta(days=ctx.settings.tmdb.refresh_days)
    return datetime.combine(day, time.min, UTC)


def pull_refresh(ctx: Context, limit: int = 0) -> None:
    """Fetches again every title older than the limit. A title TMDB no longer has is purged."""
    client = make_client(ctx)
    db = open_pull(pull_db(ctx))
    todo = stale_titles(db, refresh_cutoff(ctx))
    if limit:
        todo = todo[:limit]
    ctx.say(f"refresh: {len(todo)} titles older than {ctx.settings.tmdb.refresh_days} days")
    counts = pull_titles(ctx, client, todo, db, ctx.settings.tmdb.workers)
    ctx.say(f"refresh: {counts.text()}")
