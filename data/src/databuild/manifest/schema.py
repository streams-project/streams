"""The manifest's schema.

The shape belongs to the contract (`DatasetManifest` in `platform-contract`).
`contract.schema_dir` names that contract's `generated/schemas` folder. Until the contract
holds the type, this repository ships the same shape beside this file, and
`docs/contract-request.md` asks for it.
"""

from __future__ import annotations

import json
from importlib import resources
from pathlib import Path
from typing import Any

import jsonschema

from ..errors import StageFailed

MANIFEST_SCHEMA = "DatasetManifest.json"
SIGNATURE_SCHEMA = "DatasetSignature.json"


def load_schema(name: str, schema_dir: str = "") -> dict[str, Any]:
    """A schema from the contract's folder when one is named, else the shipped copy."""
    if schema_dir:
        path = Path(schema_dir) / name
        if not path.is_file():
            raise StageFailed(f"manifest: contract.schema_dir holds no {name}")
        return json.loads(path.read_text(encoding="utf-8"))
    text = resources.files("databuild.manifest").joinpath(name).read_text("utf-8")
    return json.loads(text)


def check(document: Any, name: str, schema_dir: str = "") -> None:
    """Refuses a document that does not fit its schema, naming the first wrong field."""
    schema = load_schema(name, schema_dir)
    validator = jsonschema.Draft202012Validator(schema)
    found = sorted(validator.iter_errors(document), key=lambda e: list(e.absolute_path))
    if found:
        where = "/".join(str(p) for p in found[0].absolute_path) or "the top level"
        raise StageFailed(f"manifest: does not fit {name} at {where}: {found[0].message[:120]}")
