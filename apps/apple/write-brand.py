#!/usr/bin/env python3
"""Writes what the Apple apps take from brand.json: Brand.xcconfig, so the Xcode project carries
no brand name, the brand's mark as a vector image the screens draw by the name `mark`, and the
brand's corners as Swift constants. All are generated and ignored by git."""
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
brand = json.loads((ROOT / "brand.json").read_text("utf-8"))


def write(path: Path, text: str) -> None:
    """Writes only on a change, so Xcode does not rebuild what did not move."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_text("utf-8") != text:
        path.write_text(text, "utf-8")


write(
    HERE / "Brand.xcconfig",
    "// Generated from brand.json by write-brand.py. Do not commit.\n"
    f"PRODUCT_NAME = {brand['name']}\n"
    f"PRODUCT_BUNDLE_IDENTIFIER = {brand['command']}.app\n",
)

catalog = HERE / "Sources/SurfacesShared/Resources/Brand.xcassets"
write(catalog / "Contents.json", json.dumps({"info": {"author": "xcode", "version": 1}}, indent=2) + "\n")
mark = catalog / "mark.imageset"
mark_svg = (ROOT / brand["assets"] / "mark.svg").read_text("utf-8")
write(mark / "mark.svg", mark_svg)
write(
    mark / "Contents.json",
    json.dumps(
        {
            "images": [{"filename": "mark.svg", "idiom": "universal"}],
            "info": {"author": "xcode", "version": 1},
            "properties": {"preserves-vector-representation": True},
        },
        indent=2,
    )
    + "\n",
)

# Layered tvOS icon: Back, Middle, Front
bg_color = brand["colors"].get("background", "#0b0b10")
surface_color = brand["colors"].get("surface", "#16161d")
accent_color = brand["colors"].get("accent", "#2f6bff")

back_svg = f"""<svg width="400" height="240" viewBox="0 0 400 240" xmlns="http://www.w3.org/2000/svg">
<defs>
  <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
    <stop offset="0%" stop-color="{surface_color}"/>
    <stop offset="100%" stop-color="{bg_color}"/>
  </linearGradient>
</defs>
<rect width="400" height="240" fill="url(#bg)"/>
</svg>"""

middle_svg = f"""<svg width="400" height="240" viewBox="0 0 400 240" xmlns="http://www.w3.org/2000/svg">
<circle cx="200" cy="120" r="80" fill="{accent_color}" opacity="0.15" filter="blur(20px)"/>
</svg>"""

front_svg = mark_svg

icon_stack = catalog / "App Icon.imagestack"
write(
    icon_stack / "Contents.json",
    json.dumps(
        {
            "info": {"author": "xcode", "version": 1},
            "layers": [
                {"filename": "Front.imagestacklayer"},
                {"filename": "Middle.imagestacklayer"},
                {"filename": "Back.imagestacklayer"},
            ],
        },
        indent=2,
    )
    + "\n",
)

for layer_name, svg_content in [
    ("Back", back_svg),
    ("Middle", middle_svg),
    ("Front", front_svg),
]:
    layer_dir = icon_stack / f"{layer_name}.imagestacklayer" / "Content.imageset"
    write(layer_dir / "content.svg", svg_content)
    write(
        layer_dir / "Contents.json",
        json.dumps(
            {
                "images": [
                    {"filename": "content.svg", "idiom": "tv", "scale": "1x"},
                    {"filename": "content.svg", "idiom": "tv", "scale": "2x"},
                ],
                "info": {"author": "xcode", "version": 1},
                "properties": {"preserves-vector-representation": True},
            },
            indent=2,
        )
        + "\n",
    )
    write(
        icon_stack / f"{layer_name}.imagestacklayer" / "Contents.json",
        json.dumps({"info": {"author": "xcode", "version": 1}}, indent=2) + "\n",
    )

# Top Shelf wide banner
top_shelf = catalog / "Top Shelf Image.imageset"
top_shelf_svg = f"""<svg width="1920" height="720" viewBox="0 0 1920 720" xmlns="http://www.w3.org/2000/svg">
<defs>
  <linearGradient id="tsbg" x1="0%" y1="0%" x2="100%" y2="100%">
    <stop offset="0%" stop-color="{surface_color}"/>
    <stop offset="100%" stop-color="{bg_color}"/>
  </linearGradient>
</defs>
<rect width="1920" height="720" fill="url(#tsbg)"/>
<g transform="translate(860, 260) scale(2)">
{mark_svg.split("<svg", 1)[-1].split(">", 1)[-1].rsplit("</svg>", 1)[0] if "<svg" in mark_svg else mark_svg}
</g>
</svg>"""

write(top_shelf / "topshelf.svg", top_shelf_svg)
write(
    top_shelf / "Contents.json",
    json.dumps(
        {
            "images": [
                {"filename": "topshelf.svg", "idiom": "tv", "scale": "1x"},
                {"filename": "topshelf.svg", "idiom": "tv", "scale": "2x"},
            ],
            "info": {"author": "xcode", "version": 1},
            "properties": {"preserves-vector-representation": True},
        },
        indent=2,
    )
    + "\n",
)

shape = brand["shape"]
write(
    HERE / "Sources/SurfacesShared/BrandShape.swift",
    "// Generated from brand.json by write-brand.py. Do not commit.\n"
    "import CoreGraphics\n\n"
    "/// The brand's corners from brand.json `shape`, in points on a phone. Each family scales them.\n"
    "public enum Corner {\n"
    f"    public static let card: CGFloat = {shape['card_radius']}\n"
    f"    public static let panel: CGFloat = {shape['panel_radius']}\n"
    f"    public static let field: CGFloat = {shape['field_radius']}\n"
    "}\n",
)
