#!/usr/bin/env python3
"""Writes each family's copy of the icon set from platform/icons/icons.json."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ICONS = json.loads((ROOT / "platform/icons/icons.json").read_text("utf-8"))


def television() -> None:
    bodies = {name: icon["body"] for name, icon in sorted(ICONS.items())}
    text = json.dumps(bodies, ensure_ascii=False, separators=(",", ":"), indent=0)
    (ROOT / "apps/tv/page/surfaces/parts/icon-data.js").write_text(
        "// Written by platform/tools/make_icons.py. Change platform/icons/icons.json instead.\n"
        f"window.tvIcons = {text};\n",
        "utf-8",
    )


if __name__ == "__main__":
    television()
