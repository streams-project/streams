"""Refuses what must never be in this repository or in a built data set.

A word is refused alone or as part of a compound name. The words themselves live in a private file
named by a setting, so the repository does not hold them (H122 to H126, H133).
"""

from __future__ import annotations

import hashlib
import json
import os
import re
from collections.abc import Iterable, Iterator
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

# A private file names what is refused: hashes of words and of row names, and a tag prefix. A
# copy of this project needs none, and then those checks have nothing to refuse.
REFUSED_FILE = "DATABUILD_REFUSED_FILE"


@lru_cache
def _private() -> dict:
    path = os.environ.get(REFUSED_FILE)
    return json.loads(Path(path).read_text("utf-8")) if path else {}


# Fields only a capture could fill. A built file holding one came from a refused input.
CAPTURE_FIELDS = frozenset({"depth", "imported"})

_WORD = re.compile(r"[a-z0-9]+(?:[_.-][a-z0-9]+)*")
_PART = re.compile(r"[_.-]")
_ADDRESS = re.compile(r"\bhttps?://([a-z0-9.-]+)", re.IGNORECASE)
_SECRETS = (
    re.compile(r"\beyJ[A-Za-z0-9_-]{20,}"),
    re.compile(r"\bgh[pousr]_[A-Za-z0-9]{20,}"),
    re.compile(r"\bhf_[A-Za-z0-9]{20,}"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{20,}"),
    re.compile(r"\b[0-9a-f]{32}\b"),
)

ALLOWED_ADDRESS_HOSTS = frozenset(
    {"127.0.0.1", "json-schema.org", "www.apache.org", "reuse.software"}
)
SKIP_DIRS = frozenset(
    {".git", ".venv", "__pycache__", ".pytest_cache", ".ruff_cache", "build", "LICENSES"}
)
SKIP_FILES = frozenset({"uv.lock", "LICENSE-APACHE", "LICENSE-MIT"})
BINARY_SUFFIXES = frozenset({".png", ".jpg", ".q8", ".bin", ".keys", ".npz", ".db", ".tar"})


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    what: str

    def __str__(self) -> str:
        return f"{self.path}:{self.line}: {self.what}"


def _hash(word: str) -> str:
    return hashlib.sha256(word.encode()).hexdigest()


def _refused(word: str) -> bool:
    words = _private().get("words", ())
    if _hash(word) in words:
        return True
    return any(_hash(part) in words for part in _PART.split(word))


def tag_prefix() -> str:
    """The prefix a capture puts on its tags, or nothing when none is set."""
    return _private().get("tag_prefix", "")


def _capture_names() -> frozenset[str]:
    """Hashes of row names read from third party captures.

    A copied name holds no refused word, so a word scan passes it. A plain word that the project's
    own vocabulary also uses is not in this list.
    """
    return frozenset(_private().get("names", ()))


def _strings(value: object) -> Iterator[str]:
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for key, item in value.items():
            yield str(key)
            yield from _strings(item)
    elif isinstance(value, list):
        for item in value:
            yield from _strings(item)


def scan_own_words(value: object, path: str = "") -> Iterator[Finding]:
    """Every string of a vocabulary file that equals a name from a capture."""
    names = _capture_names()
    for text in _strings(value):
        if _hash(text.lower().strip()) in names:
            yield Finding(path, 0, "name from a capture")


def scan_text(text: str, path: str = "") -> Iterator[Finding]:
    """Every refused word, tag, address or secret in a text."""
    for number, line in enumerate(text.splitlines(), 1):
        low = line.lower()
        for word in _WORD.findall(low):
            if _refused(word):
                yield Finding(path, number, "refused word")
                break
        prefix = tag_prefix()
        if prefix and re.search(rf"(?<![a-z0-9]){re.escape(prefix)}[a-z]", low):
            yield Finding(path, number, "capture tag prefix")
        for host in _ADDRESS.findall(line):
            name = host.lower()
            reserved = name.endswith((".test", ".invalid", ".example"))
            if not reserved and name not in ALLOWED_ADDRESS_HOSTS:
                yield Finding(path, number, "address in a file")
        if any(p.search(line) for p in _SECRETS):
            yield Finding(path, number, "text shaped like a key")


def scan_name(name: str) -> bool:
    """True when a file name is refused."""
    low = name.lower()
    return any(_refused(word) for word in _WORD.findall(low))


def scan_json_fields(value: object, path: str = "") -> Iterator[Finding]:
    """Fields that only a capture could fill, at any depth of a JSON value."""
    if isinstance(value, dict):
        for key, item in value.items():
            if key in CAPTURE_FIELDS or _refused(str(key).lower()):
                yield Finding(path, 0, "capture field")
            yield from scan_json_fields(item, path)
    elif isinstance(value, list):
        for item in value:
            yield from scan_json_fields(item, path)


def walk(root: Path) -> Iterator[Path]:
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        parts = path.relative_to(root).parts
        if set(parts[:-1]) & SKIP_DIRS or path.name in SKIP_FILES:
            continue
        yield path


def scan_paths(paths: Iterable[Path]) -> list[Finding]:
    found: list[Finding] = []
    for root in paths:
        files = walk(root) if root.is_dir() else iter([root])
        for path in files:
            base = root if root.is_dir() else root.parent
            shown = str(path.relative_to(base))
            if scan_name(path.name):
                found.append(Finding(shown, 0, "refused file name"))
            if path.suffix in BINARY_SUFFIXES:
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            found.extend(scan_text(text, shown))
            if path.suffix == ".json":
                try:
                    document = json.loads(text)
                except ValueError:
                    found.append(Finding(shown, 0, "not valid JSON"))
                    continue
                found.extend(scan_json_fields(document, shown))
                if path.parent.name == "vocabulary":
                    found.extend(scan_own_words(document, shown))
    return found
