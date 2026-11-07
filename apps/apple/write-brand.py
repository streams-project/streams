#!/usr/bin/env python3
"""Writes Brand.xcconfig from brand.json so the Xcode project carries no brand name."""
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
brand = json.loads((ROOT / "brand.json").read_text("utf-8"))
(HERE / "Brand.xcconfig").write_text(
    "// Generated from brand.json by write-brand.py. Do not commit.\n"
    f"PRODUCT_NAME = {brand['name']}\n"
    f"PRODUCT_BUNDLE_IDENTIFIER = {brand['command']}.app\n",
    "utf-8",
)
