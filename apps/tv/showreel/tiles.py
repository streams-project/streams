#!/usr/bin/env python3
"""Compose one landscape tile per title: textless backdrop, soft foot, title logo on top."""
import json, sys
from pathlib import Path
import cv2, numpy as np

TILE_W, TILE_H = 864, 486  # twice the measured 432 by 243

def compose(d: Path) -> np.ndarray:
    back = cv2.imread(str(d / "backdrop.jpg"))
    h, w = back.shape[:2]
    scale = max(TILE_W / w, TILE_H / h)
    back = cv2.resize(back, (round(w * scale), round(h * scale)), interpolation=cv2.INTER_AREA)
    y = (back.shape[0] - TILE_H) // 2
    x = (back.shape[1] - TILE_W) // 2
    img = back[y:y + TILE_H, x:x + TILE_W].astype(np.float32)
    foot = np.clip((np.linspace(0, 1, TILE_H) - 0.42) / 0.58, 0, 1)[:, None, None]
    img *= 1 - 0.62 * foot
    logo = cv2.imread(str(d / "logo.png"), cv2.IMREAD_UNCHANGED)
    if logo is None or logo.shape[2] != 4:
        return img.astype(np.uint8)
    pts = cv2.findNonZero((logo[:, :, 3] > 20).astype(np.uint8))
    x0, y0, lw, lh = cv2.boundingRect(pts)
    logo = logo[y0:y0 + lh, x0:x0 + lw]
    s = min(0.62 * TILE_W / lw, 0.36 * TILE_H / lh)
    logo = cv2.resize(logo, (round(lw * s), round(lh * s)), interpolation=cv2.INTER_AREA)
    a = logo[:, :, 3:4].astype(np.float32) / 255
    rgb = logo[:, :, :3].astype(np.float32)
    lum = (rgb.mean(2, keepdims=True) * a).sum() / max(a.sum(), 1)
    if lum < 95:  # a dark logo would vanish on a dimmed foot
        rgb = np.full_like(rgb, 245)
    ph, pw = logo.shape[:2]
    px = round(TILE_W * 0.055)
    py = TILE_H - ph - round(TILE_H * 0.075)
    roi = img[py:py + ph, px:px + pw]
    shade = cv2.GaussianBlur(a, (0, 0), 7)[..., None] if a.ndim == 2 else cv2.GaussianBlur(a, (0, 0), 7)[..., None]
    roi[:] = roi * (1 - 0.45 * shade) * (1 - a) + rgb * a
    return np.clip(img, 0, 255).astype(np.uint8)

def main():
    root = Path(sys.argv[1])
    picked = json.loads((root / "picked.json").read_text())
    out = root / "tiles"
    out.mkdir(exist_ok=True)
    for name, items in picked.items():
        for it in items:
            cv2.imwrite(str(out / f"{it['key']}.jpg"), compose(root / "art" / it["key"]), [cv2.IMWRITE_JPEG_QUALITY, 92])
        rows = []
        for i in range(0, len(items), 10):
            line = [cv2.resize(cv2.imread(str(out / f"{it['key']}.jpg")), (216, 121), interpolation=cv2.INTER_AREA) for it in items[i:i + 10]]
            line += [np.zeros((121, 216, 3), np.uint8)] * (10 - len(line))
            rows.append(np.hstack(line))
        cv2.imwrite(str(root / f"sheet-{name}.jpg"), np.vstack(rows), [cv2.IMWRITE_JPEG_QUALITY, 86])
        print(name, len(items))

main()
