"""What every stage gets. The clock arrives here as input and is read nowhere else."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import date, datetime
from pathlib import Path

from .net import NoTransport, Transport
from .settings import Settings


@dataclass
class Context:
    settings: Settings
    as_of: date
    now: Callable[[], datetime]
    work: Path
    out: Path
    commit: str = "unknown"
    transport: Transport = field(default_factory=NoTransport)
    say: Callable[[str], None] = print

    def __post_init__(self) -> None:
        self.work.mkdir(parents=True, exist_ok=True)
        self.out.mkdir(parents=True, exist_ok=True)
