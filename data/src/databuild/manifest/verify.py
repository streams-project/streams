"""Checks a built set against its manifest. It has no switch that skips a check (H63, H64).

The server keeps its own copy of the rule for its own files; this one runs before a set leaves the
builder, so a bad set is never published.
"""

from __future__ import annotations

import json
import sqlite3
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np

from ..errors import StageFailed
from ..export.files import MANIFEST_FILE, SIGNATURE_FILE
from ..export.vectors import read_seeds
from ..keys import KEY_SCHEME
from .prove import prove_clean
from .schema import MANIFEST_SCHEMA, SIGNATURE_SCHEMA, check
from .sign import check_signature, read_public
from .write import SCHEMA_VERSION, sha256_file

F16_BYTES = 2
KEY_BYTES = 4


@dataclass(frozen=True)
class Verified:
    manifest: dict[str, Any]
    signed: bool
    manifest_sha256: str


def _check_signature(folder: Path, data: bytes, public_key_file: str, schema_dir: str) -> bool:
    path = folder / SIGNATURE_FILE
    if not public_key_file:
        return False
    if not path.exists():
        raise StageFailed("manifest: the set is not signed")
    try:
        signature = json.loads(path.read_text(encoding="utf-8"))
    except ValueError as e:
        raise StageFailed("manifest: the signature file is not JSON") from e
    check(signature, SIGNATURE_SCHEMA, schema_dir)
    check_signature(data, signature, read_public(public_key_file))
    return True


def _check_files(folder: Path, files: dict[str, Any]) -> None:
    for role, entry in files.items():
        path = folder / entry["path"]
        if not path.is_file():
            raise StageFailed(f"manifest: file of role {role} does not exist")
        if path.stat().st_size != entry["bytes"]:
            raise StageFailed(f"manifest: file of role {role} has the wrong size")
        if sha256_file(path) != entry["sha256"]:
            raise StageFailed(f"manifest: file of role {role} has the wrong hash")


def _check_shape(folder: Path, manifest: dict[str, Any]) -> None:
    files = manifest["files"]
    rows, dimensions = manifest["row_count"], manifest["dimensions"]
    if files["vectors"]["bytes"] != rows * dimensions * F16_BYTES:
        raise StageFailed(f"manifest: the vectors do not fit {rows} rows of {dimensions} values")
    if files["vector_keys"]["bytes"] != rows * KEY_BYTES:
        raise StageFailed(f"manifest: the keys do not fit {rows} rows")
    keys = np.fromfile(folder / files["vector_keys"]["path"], dtype="<u4")
    if len(np.unique(keys)) != len(keys):
        raise StageFailed("manifest: two rows hold one key")
    catalogue = folder / files["catalog"]["path"]
    db = sqlite3.connect(f"file:{catalogue}?mode=ro", uri=True)
    try:
        shown = np.array([k for (k,) in db.execute("SELECT id FROM catalog")], dtype="<u4")
        version = dict(db.execute("SELECT k, v FROM meta")).get("dataset_version")
    finally:
        db.close()
    if not np.isin(keys, shown).all():
        raise StageFailed("manifest: a vector key is not in the catalogue")
    if version != manifest["dataset_version"]:
        raise StageFailed("manifest: the catalogue belongs to another set")


def _check_vocabulary(folder: Path, manifest: dict[str, Any]) -> None:
    files = manifest["files"]
    for vectors, entries in (
        ("taxonomy_vectors", "taxonomy_entries"),
        ("kids_taxonomy_vectors", "kids_taxonomy_entries"),
    ):
        if vectors not in files or entries not in files:
            continue
        held = json.loads((folder / files[entries]["path"]).read_text(encoding="utf-8"))
        if held.get("model") != manifest["model"]:
            raise StageFailed(f"manifest: {files[entries]['path']} came from another model")
        if held.get("dimensions") != manifest["dimensions"]:
            raise StageFailed(f"manifest: {files[entries]['path']} has another size")
        raw = (folder / files[vectors]["path"]).read_bytes()
        count = struct.unpack("<I", raw[:4])[0] if len(raw) >= 4 else -1
        if count != len(held["entries"]) or len(raw) != 4 + count * 4 * manifest["dimensions"]:
            raise StageFailed(f"manifest: {files[vectors]['path']} does not fit its entries")
    if "seeds" in files:
        try:
            width, _ = read_seeds((folder / files["seeds"]["path"]).read_bytes())
        except ValueError as e:
            raise StageFailed(f"manifest: the seed file is cut short: {e}") from e
        if width != manifest["dimensions"]:
            raise StageFailed("manifest: the seed file has another size")


def verify_set(folder: Path, public_key_file: str = "", schema_dir: str = "") -> Verified:
    """Every check, or a refusal naming the first that failed."""
    path = folder / MANIFEST_FILE
    if not path.exists():
        raise StageFailed("manifest: no manifest in the folder")
    data = path.read_bytes()
    signed = _check_signature(folder, data, public_key_file, schema_dir)
    try:
        manifest = json.loads(data)
    except ValueError as e:
        raise StageFailed("manifest: the manifest is not JSON") from e
    check(manifest, MANIFEST_SCHEMA, schema_dir)
    if manifest["schema_version"] != SCHEMA_VERSION:
        raise StageFailed(f"manifest: schema {manifest['schema_version']} is not one this reads")
    if manifest["key_scheme"] != KEY_SCHEME:
        raise StageFailed("manifest: keys use a scheme this does not read")
    _check_files(folder, manifest["files"])
    _check_shape(folder, manifest)
    _check_vocabulary(folder, manifest)
    prove_clean(folder)
    return Verified(manifest, signed, sha256_file(path))
