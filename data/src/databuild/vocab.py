"""The project's own vocabulary, kept as data files beside the code."""

from __future__ import annotations

import json
from functools import lru_cache
from importlib import resources
from typing import Any


@lru_cache
def load(name: str) -> Any:
    text = resources.files("databuild").joinpath("vocabulary", f"{name}.json").read_text("utf-8")
    return json.loads(text)
