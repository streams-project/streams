#!/usr/bin/env python3
"""Writes opening.json, the shapes the TV apps draw their opening from.

The opening is drawn live by each app. Its shapes come from the brand's own art: the two parts of the
mark (the ribbon behind, the arrow in front, with the arrow's shine and glow) and the wordmark. SVG
arcs and quadratic curves become cubic curves here, so every family reads paths made of M, L, C and Z
only, and no app carries the brand in code.

Run from platform/: python3 tools/make_opening.py
"""
import json
import math
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = [
    ROOT / "apps/android/surfaces/shared/src/main/res/raw/opening.json",
    ROOT / "apps/apple/Sources/SurfacesTV/Resources/opening.json",
]
NUMBER = r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?"


def tokens(d: str) -> list:
    return re.findall(rf"[MmLlHhVvCcSsQqTtAaZz]|{NUMBER}", d)


def arc(x0, y0, rx, ry, phi, large, sweep, x, y) -> list:
    """An SVG arc as cubic curves, by the centre form of the SVG specification (F.6.5)."""
    if (x0, y0) == (x, y):
        return []
    if rx == 0 or ry == 0:
        return [("L", x, y)]
    rx, ry = abs(rx), abs(ry)
    c, s = math.cos(math.radians(phi)), math.sin(math.radians(phi))
    dx, dy = (x0 - x) / 2, (y0 - y) / 2
    x1, y1 = c * dx + s * dy, -s * dx + c * dy
    grow = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry)
    if grow > 1:
        rx, ry = rx * math.sqrt(grow), ry * math.sqrt(grow)
    top = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1
    root = math.sqrt(max(0.0, top / (rx * rx * y1 * y1 + ry * ry * x1 * x1)))
    if large == sweep:
        root = -root
    cx1, cy1 = root * rx * y1 / ry, -root * ry * x1 / rx
    cx, cy = c * cx1 - s * cy1 + (x0 + x) / 2, s * cx1 + c * cy1 + (y0 + y) / 2

    def angle(ux, uy, vx, vy):
        a = math.atan2(ux * vy - uy * vx, ux * vx + uy * vy)
        return a

    t1 = angle(1, 0, (x1 - cx1) / rx, (y1 - cy1) / ry)
    dt = angle((x1 - cx1) / rx, (y1 - cy1) / ry, (-x1 - cx1) / rx, (-y1 - cy1) / ry)
    if not sweep and dt > 0:
        dt -= 2 * math.pi
    elif sweep and dt < 0:
        dt += 2 * math.pi
    pieces = max(1, math.ceil(abs(dt) / (math.pi / 2) - 1e-9))
    step = dt / pieces
    k = 4 / 3 * math.tan(step / 4)

    def point(t, scale=1.0):
        px, py = rx * math.cos(t), ry * math.sin(t)
        return cx + c * px - s * py, cy + s * px + c * py

    def tangent(t):
        px, py = -rx * math.sin(t), ry * math.cos(t)
        return c * px - s * py, s * px + c * py

    out = []
    for i in range(pieces):
        a, b = t1 + i * step, t1 + (i + 1) * step
        (ax, ay), (bx, by) = point(a), point(b)
        (tax, tay), (tbx, tby) = tangent(a), tangent(b)
        end = (x, y) if i == pieces - 1 else (bx, by)
        out.append(("C", ax + k * tax, ay + k * tay, bx - k * tbx, by - k * tby, *end))
    return out


def plain(d: str) -> list:
    """A path as absolute M, L, C and Z steps."""
    out, words, i = [], tokens(d), 0
    x = y = sx = sy = 0.0
    last_c = last_q = None
    command = None
    while i < len(words):
        if re.fullmatch(r"[A-Za-z]", words[i]):
            command = words[i]
            i += 1
            if command in "Zz":
                out.append(("Z",))
                x, y = sx, sy
                last_c = last_q = None
                continue
        rel = command.islower()
        up = command.upper()
        take = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6, "S": 4, "Q": 4, "T": 2, "A": 7}[up]
        n = [float(v) for v in words[i : i + take]]
        i += take
        bx, by = (x, y) if rel else (0.0, 0.0)
        if up == "M":
            x, y = n[0] + bx, n[1] + by
            sx, sy = x, y
            out.append(("M", x, y))
            command = "l" if rel else "L"
            last_c = last_q = None
        elif up in "LHV":
            if up == "L":
                x, y = n[0] + bx, n[1] + by
            elif up == "H":
                x = n[0] + (x if rel else 0.0)
            else:
                y = n[0] + (y if rel else 0.0)
            out.append(("L", x, y))
            last_c = last_q = None
        elif up in "CS":
            if up == "C":
                c1 = (n[0] + bx, n[1] + by)
                c2, end = (n[2] + bx, n[3] + by), (n[4] + bx, n[5] + by)
            else:
                c1 = (2 * x - last_c[0], 2 * y - last_c[1]) if last_c else (x, y)
                c2, end = (n[0] + bx, n[1] + by), (n[2] + bx, n[3] + by)
            out.append(("C", *c1, *c2, *end))
            last_c, last_q = c2, None
            x, y = end
        elif up in "QT":
            if up == "Q":
                q, end = (n[0] + bx, n[1] + by), (n[2] + bx, n[3] + by)
            else:
                q = (2 * x - last_q[0], 2 * y - last_q[1]) if last_q else (x, y)
                end = (n[0] + bx, n[1] + by)
            c1 = (x + 2 / 3 * (q[0] - x), y + 2 / 3 * (q[1] - y))
            c2 = (end[0] + 2 / 3 * (q[0] - end[0]), end[1] + 2 / 3 * (q[1] - end[1]))
            out.append(("C", *c1, *c2, *end))
            last_q, last_c = q, None
            x, y = end
        else:
            end = (n[5] + bx, n[6] + by)
            out.extend(arc(x, y, n[0], n[1], n[2], int(n[3]), int(n[4]), *end))
            x, y = end
            last_c = last_q = None
    return out


def text(steps: list) -> str:
    return " ".join(step[0] + " ".join(f"{v:.2f}".rstrip("0").rstrip(".") for v in step[1:]) for step in steps)


def box(steps: list) -> list:
    """The bounds of a path, sampling its curves."""
    xs, ys = [], []
    x = y = 0.0
    for step in steps:
        if step[0] in "ML":
            x, y = step[1], step[2]
            xs.append(x), ys.append(y)
        elif step[0] == "C":
            p = [(x, y), step[1:3], step[3:5], step[5:7]]
            for i in range(21):
                t = i / 20
                m = 1 - t
                xs.append(m**3 * p[0][0] + 3 * m * m * t * p[1][0] + 3 * m * t * t * p[2][0] + t**3 * p[3][0])
                ys.append(m**3 * p[0][1] + 3 * m * m * t * p[1][1] + 3 * m * t * t * p[2][1] + t**3 * p[3][1])
            x, y = step[5], step[6]
    return [round(min(xs), 2), round(min(ys), 2), round(max(xs), 2), round(max(ys), 2)]


def gradient(svg: str, name: str) -> dict:
    head = re.search(rf'<linearGradient id="{name}"([^>]*)>(.*?)</linearGradient>', svg, re.S)
    at = dict(re.findall(r'(\w+)="([^"]+)"', head.group(1)))
    stops = []
    for stop in re.findall(r"<stop([^>]*)/>", head.group(2)):
        s = dict(re.findall(r'([\w-]+)="([^"]+)"', stop))
        alpha = round(float(s.get("stop-opacity", "1")) * 255)
        stops.append([float(s["offset"]), s["stop-color"].upper() + f"{alpha:02X}"])
    return {"from": [float(at["x1"]), float(at["y1"])], "to": [float(at["x2"]), float(at["y2"])], "stops": stops}


def main() -> None:
    brand = json.loads((ROOT / "brand.json").read_text("utf-8"))
    folder = ROOT / brand["assets"]
    mark = (folder / "mark.svg").read_text("utf-8")
    ribbon = plain(re.search(r'<path d="([^"]+)" fill="url\(#ig0\)"/>', mark).group(1))
    arrow = plain(re.search(r'<path d="([^"]+)" fill="url\(#ig1\)"/>', mark).group(1))
    shine = re.search(r'<g opacity="([\d.]+)">\s*<g clip-path="url\(#ic2\)">\s*<path d="[^"]+" fill="url\(#(\w+)\)"/>', mark)
    glow = re.search(
        r'<g opacity="([\d.]+)">\s*<g clip-path="url\(#ic3\)">\s*<g filter="url\(#(\w+)\)">\s*<path d="([^"]+)" fill="(#\w+)"/>',
        mark,
    )
    blur = float(re.search(rf'<filter id="{glow.group(2)}"[^>]*>\s*<feGaussianBlur stdDeviation="([\d.]+)"', mark).group(1))
    rb, ab = box(ribbon), box(arrow)
    word_svg = (folder / "wordmark.svg").read_text("utf-8")
    view = [float(v) for v in re.search(r'viewBox="([^"]+)"', word_svg).group(1).split()]
    word = []
    for d in re.findall(r'<path[^>]* d="([^"]+)"', word_svg):
        word.extend(plain(d))
    out = {
        "note": "Generated by platform/tools/make_opening.py from the brand's mark.svg and wordmark.svg. Do not edit.",
        "mark": {
            "box": [min(rb[0], ab[0]), min(rb[1], ab[1]), max(rb[2], ab[2]), max(rb[3], ab[3])],
            "ribbon": {"path": text(ribbon), "fill": gradient(mark, "ig0")},
            "arrow": {"path": text(arrow), "fill": gradient(mark, "ig1"), "box": ab},
            "shine": {"fill": gradient(mark, shine.group(2)), "opacity": float(shine.group(1))},
            "glow": {"path": text(plain(glow.group(3))), "color": glow.group(4).upper(), "opacity": float(glow.group(1)), "blur": blur},
        },
        "word": {"path": text(word), "view": view},
    }
    for path in OUT:
        # Each family that is checked out here gets its copy.
        if not path.parents[1].exists():
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(out, indent=1) + "\n", "utf-8")
        print("wrote", path.relative_to(ROOT))


if __name__ == "__main__":
    main()
