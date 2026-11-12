#!/usr/bin/env python3
"""Writes each family's copy of the icon set from platform/icons/icons.json.

An icon marked `"mirror": true` points along the reading direction (back, leave) and turns in a
right to left language; every other icon, the media controls among them, never turns."""
import json
from pathlib import Path
from xml.etree import ElementTree

ROOT = Path(__file__).resolve().parents[2]
ICONS = json.loads((ROOT / "platform/icons/icons.json").read_text("utf-8"))
# The drawing grid of each source set. Every other set draws on 24.
GRIDS = {"carbon": 32, "ph": 256}


def grid(icon: dict) -> int:
    return GRIDS.get(icon["source"].split(":")[0], 24)


# `television()` wrote the icon set for the television DOM page at `apps/tv/page`. The owner stopped
# that page on 2026-11-07 (note 179) and it was deleted on 2026-11-12, so the emitter went with it:
# it was writing into a folder that no longer exists, which would have made a stray directory on the
# next run rather than failing. Roku will need its own emitter when it gets that far, and it cannot
# reuse this one - Roku draws bitmaps, so that emitter has to rasterise rather than emit a path.


def android_shape(tag: str, seen: dict) -> str:
    """One shape as a vector drawable path. Colour comes from the tint, so it is always white here."""
    if tag == "rect":
        x, y = float(seen.get("x", 0)), float(seen.get("y", 0))
        w, h = float(seen["width"]), float(seen["height"])
        data = f"M{x} {y}h{w}v{h}h{-w}z"
    else:
        data = seen["d"]
    parts = [f'android:pathData="{data}"']
    if seen.get("fill", "currentColor") != "none":
        parts.append('android:fillColor="#FFFFFFFF"')
    if seen.get("fill-rule") == "evenodd":
        parts.append('android:fillType="evenOdd"')
    if seen.get("stroke") not in (None, "none"):
        parts.append('android:strokeColor="#FFFFFFFF"')
        parts.append(f'android:strokeWidth="{seen.get("stroke-width", "1")}"')
        if "stroke-linecap" in seen:
            parts.append(f'android:strokeLineCap="{seen["stroke-linecap"]}"')
        if "stroke-linejoin" in seen:
            parts.append(f'android:strokeLineJoin="{seen["stroke-linejoin"]}"')
    return "    <path " + " ".join(parts) + " />\n"


def android() -> None:
    out = ROOT / "apps/android/surfaces/shared/src/main/res/drawable"
    for name, icon in sorted(ICONS.items()):
        shapes = []

        def walk(node, held):
            for child in node:
                seen = {**held, **child.attrib}
                tag = child.tag
                if tag == "g":
                    walk(child, seen)
                else:
                    shapes.append(android_shape(tag, seen))

        walk(ElementTree.fromstring(f"<svg>{icon['body']}</svg>"), {})
        (out / f"icon_{name.replace('-', '_')}.xml").write_text(
            "<!-- Written by platform/tools/make_icons.py. Change platform/icons/icons.json instead. -->\n"
            '<vector xmlns:android="http://schemas.android.com/apk/res/android" '
            + ('android:autoMirrored="true" ' if icon.get("mirror") else "")
            + 'android:width="24dp" android:height="24dp" '
            f'android:viewportWidth="{grid(icon)}" android:viewportHeight="{grid(icon)}">\n'
            + "".join(shapes)
            + "</vector>\n",
            "utf-8",
        )


def apple() -> None:
    catalog = ROOT / "apps/apple/Sources/SurfacesShared/Resources/Icons.xcassets"
    catalog.mkdir(parents=True, exist_ok=True)
    (catalog / "Contents.json").write_text(
        json.dumps({"info": {"author": "xcode", "version": 1}}, indent=2) + "\n",
        "utf-8",
    )
    for name, icon in sorted(ICONS.items()):
        stem = f"icon_{name.replace('-', '_')}"
        folder = catalog / f"{stem}.imageset"
        folder.mkdir(parents=True, exist_ok=True)
        size = grid(icon)
        (folder / f"{stem}.svg").write_text(
            f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}">{icon["body"]}</svg>\n',
            "utf-8",
        )
        (folder / "Contents.json").write_text(
            json.dumps(
                {
                    # A left to right picture that turns in a right to left interface.
                    "images": [
                        {"filename": f"{stem}.svg", "idiom": "universal"}
                        | ({"language-direction": "left-to-right"} if icon.get("mirror") else {})
                    ],
                    "info": {"author": "xcode", "version": 1},
                    "properties": {
                        "preserves-vector-representation": True,
                        "template-rendering-intent": "template",
                    },
                },
                indent=2,
            )
            + "\n",
            "utf-8",
        )


if __name__ == "__main__":
    android()
    apple()
