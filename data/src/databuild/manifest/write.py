"""Writes `manifest.json` and, when a signing key is set, `manifest.sig`.

The manifest names model, dimensions, quantisation, keys, row count, versions, the build commit and
a hash per file. It holds no index: the server builds that and stamps it with this file's hash.
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
from typing import Any

from ..context import Context
from ..errors import FeatureOff, StageFailed
from ..export.files import (
    BY_PATH,
    EXPORT_RECORD,
    FILES,
    MANIFEST_FILE,
    SIGNATURE_FILE,
)
from ..settings import read_secret
from .prove import prove_clean
from .schema import MANIFEST_SCHEMA, check
from .sign import read_private, sign, signature_bytes

SCHEMA_VERSION = 1
CHUNK = 1 << 20
RECORD_FIELDS = (
    "dataset_version",
    "build_commit",
    "model",
    "dimensions",
    "quantisation",
    "row_count",
    "key_scheme",
    "recipe",
    "source",
    "as_of",
)
BOOKKEEPING = frozenset({MANIFEST_FILE, SIGNATURE_FILE, EXPORT_RECORD})


def sha256_file(path: Path) -> str:
    """The lowercase hex hash of a file, read in steps, so a big file costs no memory."""
    digest = hashlib.sha256()
    with open(path, "rb") as source:
        while chunk := source.read(CHUNK):
            digest.update(chunk)
    return digest.hexdigest()


def manifest_bytes(document: dict[str, Any]) -> bytes:
    return (json.dumps(document, indent=1, sort_keys=True) + "\n").encode()


def unlisted_files(folder: Path) -> list[str]:
    """Files in the folder that the sidecar table does not name."""
    found = []
    for path in sorted(folder.rglob("*")):
        if not path.is_file():
            continue
        name = path.relative_to(folder).as_posix()
        if name not in BOOKKEEPING and name not in BY_PATH:
            found.append(name)
    return found


def read_record(folder: Path) -> dict[str, Any]:
    path = folder / EXPORT_RECORD
    if not path.exists():
        raise StageFailed("manifest: no export record; run the export stage")
    return json.loads(path.read_text(encoding="utf-8"))


def build_manifest(ctx: Context) -> dict[str, Any]:
    record = read_record(ctx.out)
    extra = unlisted_files(ctx.out)
    if extra:
        raise StageFailed(f"manifest: file not in the sidecar table: {extra[0]}")
    files: dict[str, dict[str, Any]] = {}
    for spec in FILES:
        path = ctx.out / spec.path
        if path.exists():
            files[spec.role] = {
                "path": spec.path,
                "sha256": sha256_file(path),
                "bytes": path.stat().st_size,
                "required": spec.required,
            }
        elif spec.required:
            raise StageFailed(f"manifest: required file {spec.path} is missing")
    document: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "built_at_ms": int(ctx.now().timestamp() * 1000),
        "files": files,
    }
    for field in RECORD_FIELDS:
        document[field] = record[field]
    if "open_set" in record:
        document["open_set"] = record["open_set"]
    return document


def _replace(path: Path, data: bytes) -> None:
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_bytes(data)
    os.replace(tmp, path)


def write_manifest(ctx: Context) -> None:
    prove_clean(ctx.out)
    document = build_manifest(ctx)
    check(document, MANIFEST_SCHEMA, ctx.settings.contract.schema_dir)
    data = manifest_bytes(document)
    signature = ctx.out / SIGNATURE_FILE
    try:
        secret = read_secret(ctx.settings.sign.key_file, "sign.key_file")
    except FeatureOff as off:
        signature.unlink(missing_ok=True)
        ctx.say(f"manifest: unsigned, {off}")
    else:
        _replace(signature, signature_bytes(sign(data, read_private(secret))))
    _replace(ctx.out / MANIFEST_FILE, data)
    ctx.say(
        f"manifest: {document['dataset_version']}, {document['row_count']} rows, "
        f"{len(document['files'])} files, model {document['model']}"
    )
