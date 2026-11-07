#!/usr/bin/env python3
"""Draws the Android launcher art from the brand, so a copy under another name gets its own tile."""
import json
import os
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BRAND = json.loads((ROOT / "brand.json").read_text("utf-8"))
ASSETS = ROOT / BRAND["assets"]
BACK = BRAND["colors"]["background"]
CHROME = os.environ.get("CHROME", "/Applications/Google Chrome Beta.app/Contents/MacOS/Google Chrome Beta")
RES = ROOT / "apps/android/app/src/main/res"


def draw(svg: str, wide: int, high: int, share: float, out: Path) -> None:
    # Headless Chrome will not shrink a window much, so small art is drawn large and scaled by zoom.
    zoom = max(1, 800 // wide)
    wide, high = wide * zoom, high * zoom
    page = (
        f'<!doctype html><body style="margin:0;background:{BACK};display:flex;align-items:center;'
        f'justify-content:center;height:{high}px"><img src="{(ASSETS / svg).as_uri()}" '
        f'style="width:{int(wide * share)}px;max-height:{int(high * 0.8)}px"></body>'
    )
    with tempfile.TemporaryDirectory() as folder:
        html = Path(folder) / "art.html"
        html.write_text(page, "utf-8")
        out.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars", f"--user-data-dir={folder}/profile",
             "--allow-file-access-from-files", f"--screenshot={out}", f"--window-size={wide},{high}", html.as_uri()],
            check=True, capture_output=True, timeout=60,
        )
        if zoom > 1:
            subprocess.run(["sips", "-z", str(high // zoom), str(wide // zoom), str(out)], check=True, capture_output=True)


if __name__ == "__main__":
    draw("lockup-on-dark.svg", 640, 360, 0.7, RES / "drawable-xhdpi/banner.png")
    draw("mark.svg", 192, 192, 0.66, RES / "mipmap-xxxhdpi/ic_launcher.png")
