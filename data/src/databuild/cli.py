"""One entry point: `databuild <command>`. The clock and the real transport are made here only."""

from __future__ import annotations

import argparse
import subprocess
import sys
from datetime import UTC, date, datetime
from pathlib import Path

from .clean.catalog import clean
from .context import Context
from .embed.run import embed
from .errors import FeatureOff, StageFailed
from .group.run import group
from .name.run import name
from .net import UrlTransport
from .pull.details import pull_details, pull_refresh
from .pull.fresh import pull_fresh
from .pull.popularity import pull_popularity
from .scan import scan_paths
from .settings import Settings, load

DEFAULT_SETTINGS = "databuild.toml"
EXIT_FAILED = 1
EXIT_OFF = 3


def _git(*args: str) -> str:
    try:
        done = subprocess.run(["git", *args], capture_output=True, text=True, check=True)
    except (OSError, subprocess.CalledProcessError):
        return ""
    return done.stdout.strip()


def build_commit() -> str:
    """The commit this build ran from, marked when the tree held other changes."""
    head = _git("rev-parse", "HEAD")
    if not head:
        return "unknown"
    return f"{head}-dirty" if _git("status", "--porcelain") else head


def _context(args: argparse.Namespace, settings: Settings) -> Context:
    as_of = args.as_of or datetime.now(UTC).date()
    return Context(
        settings=settings,
        as_of=as_of,
        now=lambda: datetime.now(UTC),
        work=Path(args.work or settings.paths.work),
        out=Path(args.out or settings.paths.out),
        commit=build_commit(),
        transport=UrlTransport(),
    )


def _scan(args: argparse.Namespace) -> int:
    found = scan_paths([Path(p) for p in args.paths])
    for finding in found:
        print(finding)
    print(f"scan: {len(found)} findings")
    return EXIT_FAILED if found else 0


def _all(ctx: Context) -> None:
    clean(ctx)
    embed(ctx)
    group(ctx)
    name(ctx)


def parser() -> argparse.ArgumentParser:
    top = argparse.ArgumentParser(prog="databuild", description=__doc__)
    top.add_argument("--settings", help=f"settings file, default {DEFAULT_SETTINGS} when present")
    top.add_argument("--as-of", type=date.fromisoformat, help="the build date, YYYY-MM-DD")
    top.add_argument("--work", help="work folder, overrides the setting")
    top.add_argument("--out", help="output folder, overrides the setting")
    commands = top.add_subparsers(dest="command", required=True)
    for word, text in (
        ("pull", "pull details of new titles"),
        ("refresh", "pull again every title older than the refresh limit"),
    ):
        command = commands.add_parser(word, help=text)
        command.add_argument("--limit", type=int, default=0, help="most titles to fetch")
    commands.add_parser("popularity", help="write the daily popularity file")
    commands.add_parser("fresh", help="write the new and upcoming titles file")
    commands.add_parser("clean", help="turn the pull file into titles.db")
    embed_command = commands.add_parser("embed", help="write the vectors")
    embed_command.add_argument("--open-text", help="a file of open text, for the second set")
    commands.add_parser("group", help="write the groups")
    commands.add_parser("name", help="name groups, write row definitions and vocabulary")
    commands.add_parser("all", help="every stage after the pull, in order")
    scan = commands.add_parser("scan", help="refuse what must never be in a file")
    scan.add_argument("paths", nargs="+")
    return top


def run(args: argparse.Namespace, ctx: Context) -> None:
    match args.command:
        case "pull":
            pull_details(ctx, args.limit)
        case "refresh":
            pull_refresh(ctx, args.limit)
        case "popularity":
            pull_popularity(ctx)
        case "fresh":
            pull_fresh(ctx)
        case "clean":
            clean(ctx)
        case "embed":
            embed(ctx, Path(args.open_text) if args.open_text else None)
        case "group":
            group(ctx)
        case "name":
            name(ctx)
        case "all":
            _all(ctx)


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "scan":
            return _scan(args)
        path = args.settings or (DEFAULT_SETTINGS if Path(DEFAULT_SETTINGS).exists() else None)
        ctx = _context(args, load(path))
        run(args, ctx)
    except FeatureOff as off:
        print(f"off: {off}", file=sys.stderr)
        return EXIT_OFF
    except StageFailed as failed:
        print(f"failed: {failed}", file=sys.stderr)
        return EXIT_FAILED
    return 0


if __name__ == "__main__":
    sys.exit(main())
