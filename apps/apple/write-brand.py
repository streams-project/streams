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

catalog = HERE / "Sources/SurfacesTV/Resources/Brand.xcassets"
write(catalog / "Contents.json", json.dumps({"info": {"author": "xcode", "version": 1}}, indent=2) + "\n")
mark = catalog / "mark.imageset"
write(mark / "mark.svg", (ROOT / brand["assets"] / "mark.svg").read_text("utf-8"))
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

shape = brand["shape"]
write(
    HERE / "Sources/SurfacesTV/app/BrandShape.swift",
    "// Generated from brand.json by write-brand.py. Do not commit.\n"
    "import CoreGraphics\n\n"
    "/// The brand's corners from brand.json `shape`, in points on a phone. Each family scales them.\n"
    "public enum Corner {\n"
    f"    public static let card: CGFloat = {shape['card_radius']}\n"
    f"    public static let panel: CGFloat = {shape['panel_radius']}\n"
    f"    public static let field: CGFloat = {shape['field_radius']}\n"
    "}\n",
)
