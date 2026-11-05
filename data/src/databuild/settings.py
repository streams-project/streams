"""Settings come from one TOML file. A missing setting turns its feature off and says so.

Nothing here holds an address, a key or an owner value. `config/settings.example.toml` lists every
setting with an empty value.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass, field, fields
from pathlib import Path
from typing import Any

from .errors import FeatureOff, StageFailed


@dataclass(frozen=True)
class Paths:
    work: str = "build/work"
    out: str = "build/out"


@dataclass(frozen=True)
class Tmdb:
    api_base: str = ""
    exports_base: str = ""
    key_file: str = ""
    languages: tuple[str, ...] = ("en-US",)
    min_popularity: float = 2.0
    rps: float = 30.0
    workers: int = 24
    refresh_days: int = 180


@dataclass(frozen=True)
class Embed:
    backend: str = "hashing"
    model: str = ""
    dimensions: int = 1024
    endpoint: str = ""
    key_file: str = ""
    device: str = ""
    batch_size: int = 128
    max_length: int = 384


@dataclass(frozen=True)
class Group:
    method: str = "auto"
    seed: int = 7


@dataclass(frozen=True)
class Name:
    endpoint: str = ""
    model: str = ""
    key_file: str = ""


@dataclass(frozen=True)
class Reviews:
    vectors: str = ""
    keys: str = ""


@dataclass(frozen=True)
class Open:
    wikidata_endpoint: str = ""
    tvmaze_api_base: str = ""
    tvmaze_rps: float = 1.9
    user_agent: str = ""
    titles: int = 1000
    min_links: int = 2


@dataclass(frozen=True)
class Sign:
    key_file: str = ""
    public_key_file: str = ""


@dataclass(frozen=True)
class Export:
    removed_rows: str = ""


@dataclass(frozen=True)
class Contract:
    schema_dir: str = ""


@dataclass(frozen=True)
class Age:
    set_days: int = 200
    popularity_hours: int = 36
    fresh_hours: int = 12


@dataclass(frozen=True)
class Settings:
    paths: Paths = field(default_factory=Paths)
    tmdb: Tmdb = field(default_factory=Tmdb)
    embed: Embed = field(default_factory=Embed)
    group: Group = field(default_factory=Group)
    name: Name = field(default_factory=Name)
    reviews: Reviews = field(default_factory=Reviews)
    open: Open = field(default_factory=Open)
    sign: Sign = field(default_factory=Sign)
    export: Export = field(default_factory=Export)
    contract: Contract = field(default_factory=Contract)
    age: Age = field(default_factory=Age)


SECTIONS: dict[str, type] = {
    "paths": Paths,
    "tmdb": Tmdb,
    "embed": Embed,
    "group": Group,
    "name": Name,
    "reviews": Reviews,
    "open": Open,
    "sign": Sign,
    "export": Export,
    "contract": Contract,
    "age": Age,
}


def _section(cls: type, raw: dict[str, Any], name: str) -> Any:
    known = {f.name: f for f in fields(cls)}
    unknown = sorted(set(raw) - set(known))
    if unknown:
        raise StageFailed(f"settings: unknown key in [{name}]: {', '.join(unknown)}")
    values = {k: tuple(v) if isinstance(v, list) else v for k, v in raw.items()}
    return cls(**values)


def load(path: str | Path | None) -> Settings:
    """Reads the file. No path means every default, so every outside feature is off."""
    if path is None:
        return Settings()
    try:
        with open(path, "rb") as f:
            raw = tomllib.load(f)
    except OSError as e:
        raise StageFailed(f"settings: cannot read {path}: {e.strerror}") from e
    except tomllib.TOMLDecodeError as e:
        raise StageFailed(f"settings: {path} is not valid TOML: {e}") from e
    unknown = sorted(set(raw) - set(SECTIONS))
    if unknown:
        raise StageFailed(f"settings: unknown section: {', '.join(unknown)}")
    return Settings(**{k: _section(SECTIONS[k], v, k) for k, v in raw.items()})


class Secret:
    """A value that never prints. `reveal` is the only way to read it."""

    __slots__ = ("_value",)

    def __init__(self, value: str) -> None:
        self._value = value

    def reveal(self) -> str:
        return self._value

    def __repr__(self) -> str:
        return "<hidden>"

    __str__ = __repr__


def read_secret(path: str, setting: str) -> Secret:
    """Reads a key from its file. Only a step that needs the key calls this."""
    if not path:
        raise FeatureOff(f"{setting} is not set")
    try:
        text = Path(path).expanduser().read_text(encoding="utf-8").strip()
    except OSError as e:
        raise StageFailed(f"{setting}: cannot read the key file: {e.strerror}") from e
    if not text:
        raise StageFailed(f"{setting}: the key file is empty")
    return Secret(text)


def need(value: str, setting: str) -> str:
    """Returns a setting, or turns the feature off when it is empty."""
    if not value:
        raise FeatureOff(f"{setting} is not set")
    return value
