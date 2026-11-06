"""Runs the export stage. A file from an earlier build never stays beside this one.

A build date, a set of inputs and a commit give the same bytes each time. The wall clock reaches
only the export record, which holds the time this set was built.
"""

from __future__ import annotations

import json
import os
import shutil
import sqlite3
from pathlib import Path
from typing import Any

import numpy as np

from ..context import Context
from ..embed.quant import restore
from ..embed.run import read_master
from ..errors import StageFailed
from ..keys import KEY_SCHEME
from ..vocab import load
from .catalog import Identity, write_catalog
from .files import BY_ROLE, EXPORT_RECORD, FILES, FileSpec
from .vectors import Member, make_seeds, seed_bytes, seed_groups, seed_names, vector_bytes

QUANTISATION = "f16"
PROFILES_FILE = "profiles.jsonl"
VOCABULARY_ROLES = ("taxonomy_entries", "kids_taxonomy_entries")
FROM_NAME_STAGE = (
    "row_requests",
    "events",
    "taxonomy_vectors",
    "taxonomy_entries",
    "kids_taxonomy_vectors",
    "kids_taxonomy_entries",
    "categories",
    "topics",
    "topics_meta",
)
OPERATOR_FILES = (
    ("awards", "awards_file"),
    ("skip_marks", "marks_file"),
    ("content_reasons", "reasons_file"),
)


def put(path: Path, data: bytes) -> None:
    """Writes a file whole."""
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_bytes(data)
    os.replace(tmp, path)


def copy_in(source: Path, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    tmp = target.with_name(target.name + ".tmp")
    shutil.copyfile(source, tmp)
    os.replace(tmp, target)


def clear_earlier(ctx: Context) -> None:
    """Removes what an earlier export wrote. Daily files and sample files stay."""
    for spec in FILES:
        if spec.writer in ("export", "name"):
            (ctx.out / spec.path).unlink(missing_ok=True)


def kids_channel_names() -> list[str]:
    """The children's channel names, from this repository's own vocabulary, in written order.

    The server holds no channel name, so this vocabulary is the one home of the list and the
    exported file is a plain JSON array of names, which is what the server's loader reads.
    """
    held = load("kids_channels")["channels"]
    if not held or not all(isinstance(name, str) and name.strip() for name in held):
        raise StageFailed("export: the kids channel vocabulary must be a list of names")
    folded = [name.strip().lower() for name in held]
    if len(set(folded)) != len(folded):
        raise StageFailed("export: the kids channel vocabulary names a channel twice")
    return [name.strip() for name in held]


def read_removed(ctx: Context) -> list[str]:
    path = ctx.settings.export.removed_rows
    if not path:
        ctx.say("export: export.removed_rows is not set; no rows removed")
        return []
    try:
        value = json.loads(Path(path).read_text(encoding="utf-8"))
    except OSError as e:
        raise StageFailed(f"export: cannot read the removed rows file: {e.strerror}") from e
    except ValueError as e:
        raise StageFailed("export: the removed rows file is not JSON") from e
    if not isinstance(value, list) or not all(isinstance(x, str) for x in value):
        raise StageFailed("export: the removed rows file must be a list of names")
    return value


def read_groups(ctx: Context, keys: np.ndarray) -> Any:
    """Group arrays, which must come from these vectors."""
    path = ctx.work / "groups.npz"
    if not path.exists():
        raise StageFailed("export: no groups file; run the group stage")
    arrays = np.load(path)
    if not np.array_equal(arrays["keys"], keys):
        raise StageFailed("export: the groups were made from other vectors; run group again")
    return arrays


def check_vocabulary(ctx: Context, info: dict[str, Any]) -> None:
    """Vocabulary vectors must come from the model that made the title vectors (H63)."""
    for role in VOCABULARY_ROLES:
        path = ctx.work / "export" / BY_ROLE[role].path
        if not path.exists():
            raise StageFailed(f"export: no {path.name}; run the name stage")
        held = json.loads(path.read_text(encoding="utf-8"))
        if held.get("model") != info["model"] or held.get("dimensions") != info["dimensions"]:
            raise StageFailed(
                f"export: {path.name} came from another model or size; run the name stage again"
            )


def members_of(ctx: Context) -> tuple[dict[int, Member], dict[int, str]]:
    titles = sqlite3.connect(f"file:{ctx.work / 'titles.db'}?mode=ro", uri=True)
    members = {
        key: Member(pop or 0.0, rating or 0.0, votes or 0)
        for key, pop, rating, votes in titles.execute(
            "SELECT id, popularity, vote_average, vote_count FROM titles"
        )
    }
    titles.close()
    groups = sqlite3.connect(f"file:{ctx.work / 'groups.db'}?mode=ro", uri=True)
    names = {
        group: name or ""
        for group, name in groups.execute(
            "SELECT id, name FROM groups WHERE kind = 'world' AND level = 2"
        )
    }
    groups.close()
    return members, names


def copy_named(ctx: Context) -> None:
    """Files the name stage wrote. A required one that is missing stops the export."""
    for role in FROM_NAME_STAGE:
        spec = BY_ROLE[role]
        source = ctx.work / "export" / spec.path
        if source.exists():
            copy_in(source, ctx.out / spec.path)
        elif spec.required:
            raise StageFailed(f"export: no {spec.path}; run the name stage")
    profiles = ctx.work / PROFILES_FILE
    if profiles.exists():
        copy_in(profiles, ctx.out / BY_ROLE["title_profiles"].path)


def copy_operator_files(ctx: Context) -> None:
    """Private inputs pass through when the operator names them, and the build says when not."""
    export = ctx.settings.export
    for role, setting in OPERATOR_FILES:
        path = getattr(export, setting)
        spec: FileSpec = BY_ROLE[role]
        if not path:
            ctx.say(f"export: export.{setting} is not set; {spec.missing}")
            continue
        source = Path(path)
        if not source.is_file():
            raise StageFailed(f"export: export.{setting} names no file")
        copy_in(source, ctx.out / spec.path)


def write_open_set(ctx: Context, shown: np.ndarray) -> dict[str, Any] | None:
    """The second vector set, from open text only, when one was embedded."""
    folder = ctx.work / "vectors-open"
    if not ctx.votes or not (folder / "embed.json").exists():
        return None
    rows, keys, info = read_master(folder)
    keep = np.isin(keys, shown)
    put(ctx.out / BY_ROLE["open_vectors"].path, vector_bytes(restore(rows[keep])))
    put(ctx.out / BY_ROLE["open_vector_keys"].path, keys[keep].tobytes())
    return {
        "model": info["model"],
        "dimensions": info["dimensions"],
        "recipe": info["recipe"],
        "row_count": int(keep.sum()),
    }


def export(ctx: Context) -> None:
    rows, keys, info = read_master(ctx.work / "vectors")
    arrays = read_groups(ctx, keys)
    check_vocabulary(ctx, info)
    source = "full" if ctx.votes else "open"
    version = f"{source}-{ctx.as_of.isoformat()}-{ctx.commit[:12]}"
    removed = read_removed(ctx)
    clear_earlier(ctx)
    identity = Identity(version, ctx.as_of.isoformat(), ctx.commit, info["model"])
    shown, gone = write_catalog(ctx, identity, arrays, removed)

    keep = np.isin(keys, shown)
    count = int(keep.sum())
    if not count:
        raise StageFailed("export: no title of the catalogue has a vector")
    ctx.say(
        f"export: {len(shown)} catalogue titles, {count} with a vector, "
        f"{len(keys) - count} vectors left out, {gone} season sub groups removed"
    )
    unit_rows = restore(rows)[keep]
    kept_keys = keys[keep]
    put(ctx.out / "vectors.bin", vector_bytes(unit_rows))
    put(ctx.out / "vectors.keys", kept_keys.tobytes())

    members, names = members_of(ctx)
    w2 = arrays["w2"][keep]
    seeds = make_seeds(unit_rows, kept_keys, w2, members, ctx.votes)
    put(ctx.out / "seeds.bin", seed_bytes(seeds, info["dimensions"]))
    put(ctx.out / "seeds.names.json", seed_names(seeds, names))
    put(ctx.out / "seeds.groups.json", seed_groups(seeds))
    ctx.say(f"export: {len(seeds)} seeds, {sum(len(s.keys) for s in seeds)} members")

    copy_named(ctx)
    put(ctx.out / "search_aliases.json", json.dumps(load("search_aliases")).encode())
    channels = kids_channel_names()
    put(ctx.out / BY_ROLE["kids_channels"].path, (json.dumps(channels, indent=1) + "\n").encode())
    ctx.say(f"export: {len(channels)} children's channels")
    put(ctx.out / "removed_rows.json", (json.dumps(removed, indent=1) + "\n").encode())
    copy_operator_files(ctx)
    second = write_open_set(ctx, shown)

    record = {
        "dataset_version": version,
        "built_at_ms": int(ctx.now().timestamp() * 1000),
        "source": source,
        "as_of": ctx.as_of.isoformat(),
        "build_commit": ctx.commit,
        "model": info["model"],
        "dimensions": info["dimensions"],
        "quantisation": QUANTISATION,
        "recipe": info["recipe"],
        "row_count": count,
        "key_scheme": KEY_SCHEME,
    }
    if second:
        record["open_set"] = second
    put(ctx.out / EXPORT_RECORD, (json.dumps(record, indent=1, sort_keys=True) + "\n").encode())
