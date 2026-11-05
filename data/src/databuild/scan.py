"""Refuses what must never be in this repository or in a built data set.

The refused words are kept as hashes, so this file does not hold them. A word is refused alone or as
part of a compound name. The list covers third party capture names, the old project's names and
owner values (H122 to H126, H133).
"""

from __future__ import annotations

import hashlib
import json
import re
from collections.abc import Iterable, Iterator
from dataclasses import dataclass
from pathlib import Path

REFUSED_WORDS = frozenset(
    {
        "0e20679f13df2dc6dd29b1510e24f1924abf700f37d490b7207a7eed0edeef2c",
        "13ed070478ef62c3a7baa36c8d042a9d1cdc0fcbb2af93a795f2ad20ad6e9cb5",
        "1aa9b98763075c03379267f738d9fd4ebe2d78346d843992f04915abad0c0a7c",
        "246aea3c4d0fc25b4e23dc2a766578a2f0e5b6f73491caa2564f543db7df0682",
        "2afd53183efc5c9cb6cf283cfd91eb481e6a7e566112af7dbe90cc019a56ad62",
        "2f5a846ea9bf10e33f2edc8c19723d18443590f12106007f0643051121f17677",
        "34e79ad648889648dfafc878301c00e824d03b09d3689b8c81772a3360d34285",
        "49db07b32f5c290ed74ae92002a973e3ada5e0c0635a1bbf0b28a9080d3d2dd9",
        "4bfb4dcad3f443656e331d2503f22fc16ad4991f89c6064b2a8bccb655039ded",
        "556209d93237fc6f69cf5be6e7ea5bb87d9b48be5a13bd4de33ee63fa449e399",
        "62a991320f4f22741d9ad7d3590f82a9d2de5c08274419107b8f814724ccf66e",
        "65bcd7485f6c5682d5e526ee364f45531ece10f5d4aef5091620151e781b540e",
        "6a5473f459ecb4e34a75a1b726a45ea97edf0f449ec58d7c9da52c1254f42390",
        "80011e4e5e22faeacaaf9e0f64927880c73c3fd7e8b14b708b44052dfd2019a1",
        "845065dff35a314a21d484e2729c183ac5ec6d896a7875f61c46f3958b0ea8c4",
        "8466f0c76f7eace3e6c04bed025ed01c3941e6d37792e2f5b56bf517ed61ef8a",
        "847f1ace0a9087eb735877b03a712c76582ebee55de7f3d149eb163f845061d5",
        "84c22cc47f49c246ba161045aab5947df27e0bc410f9c897b898ca435f712b0e",
        "9099fdafbeb4b2ed7edf02627273078da891f40b83607f1ddb7be0981c83ed3d",
        "90c2760f8c235e655536925520bf87a93fb0ae4325d39e7a25430150d05f910c",
        "9711bcc773b67df9d02c1adc26240b1332048ab8ec17a23732ace9e0c030e0bd",
        "c3e9c1f54501cec82a66ecbbcc590c7c5ffb8139a0edb411cf149b8c46c57e15",
        "c5539358758140be6c2eca8570dc96eb1bb763e0f5baca3b9e31bb900ce8106a",
        "c573f42e8a1402d778c665ae8f549324490376e2e8bacf1e1866f4eaa5de6780",
        "c8cceb018e39f134b5417033ed88799cf3337aafafa54271bbe6463d0479ab24",
        "d7d614b33fb2c8c847176fdbafecee018d4061237d20f718d188ed222d9339e3",
        "dc58d38116d58137c3e7b5fcfe086ad23da2be28a588f333b0ecb0fb127ee9bd",
        "f64fbb06d3936d0344b52a3411c0e6b4bbd820691339a3966089df8fee17a75b",
        "f78c32f7e929f73506901f1052de6f4b1f0e1b6ee05a2f7389fdca7d8e19d494",
    }
)

# Fields only a capture could fill. A built file holding one came from a refused input.
CAPTURE_FIELDS = frozenset({"depth", "imported"})

_WORD = re.compile(r"[a-z0-9]+(?:[_.-][a-z0-9]+)*")
_PART = re.compile(r"[_.-]")
_TAG_PREFIX = re.compile(r"(?<![a-z0-9])nf:[a-z]")
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
BINARY_SUFFIXES = frozenset({".png", ".jpg", ".q8", ".bin", ".keys", ".npz", ".db"})


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
    if _hash(word) in REFUSED_WORDS:
        return True
    return any(_hash(part) in REFUSED_WORDS for part in _PART.split(word))


def scan_text(text: str, path: str = "") -> Iterator[Finding]:
    """Every refused word, tag, address or secret in a text."""
    for number, line in enumerate(text.splitlines(), 1):
        low = line.lower()
        for word in _WORD.findall(low):
            if _refused(word):
                yield Finding(path, number, "refused word")
                break
        if _TAG_PREFIX.search(low):
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
                    found.extend(scan_json_fields(json.loads(text), shown))
                except ValueError:
                    found.append(Finding(shown, 0, "not valid JSON"))
    return found
