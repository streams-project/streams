#!/usr/bin/env python3
"""Render the showreel that plays inside the landing card.

Every number comes from reading the recording in video coordinates, where the card's corners
are the corners of a 1920 by 1080 picture. Frame numbers are the recording's, so any frame here
can be laid beside the same frame there. The film starts at F0 and runs to F_END.
"""
import argparse, json, math, subprocess
from pathlib import Path
import cv2, numpy as np
from PIL import Image, ImageDraw, ImageFont

VW, VH = 1920, 1080
PIVOT = (960.0, 540.0)
FPS = 24
F0, F_END = 590, 1340
PAD_W, PAD_H = 3800, 2200  # room for a layer shrunk to 0.53
RADIUS = 14 / 432  # tile corner, share of tile width

# Flat grids. Columns and rows are left and top edges read from gradient projections on rest
# frames. Slots are the rows left empty for a headline.
COLS = [-1183, -700, -218, 263, 746, 1226, 1705, 2188, 2670]
TILE_W, TILE_H = 432, 243
ROW_SETS = {
    "mid": [-1003, -709, -415, -121, 173, 663, 956, 1250, 1544, 1838],   # slot y 416 to 663
    "low": [-855, -561, -267, 27, 321, 811, 1105, 1399, 1693],           # slot y 565 to 811
    "high": [-983, -689, -395, -101, 388, 682, 976, 1270, 1564],         # slot y 143 to 388
}
SMALL = dict(w=308, h=172, pitch_x=338, pitch_y=209.5, top=138.0, x_even=62.0, x_odd=178.0)
SIX = dict(w=488, h=272, xs=[175, 716, 1256], ys=[278, 602])

def smooth(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)

def table(pairs, x):
    return float(np.interp(x, pairs[0::2], pairs[1::2]))

def bezier(x1, y1, x2, y2):
    def f(x):
        lo, hi = 0.0, 1.0
        for _ in range(30):
            t = (lo + hi) / 2
            bx = 3 * (1 - t) ** 2 * t * x1 + 3 * (1 - t) * t ** 2 * x2 + t ** 3
            lo, hi = (t, hi) if bx < x else (lo, t)
        return 3 * (1 - t) ** 2 * t * y1 + 3 * (1 - t) * t ** 2 * y2 + t ** 3
    return f

# Scale tables read off the recording with scale_curve.py, four changes each. ENTRY is the excess
# over 1 by frames since arrival. EXIT is the scale by frames since the exit began.
ENTRY_EXCESS = [0, 1.8, 1, 1.35, 2, 1.0, 3, .68, 4, .42, 5, .25, 6, .15, 8, .09, 10, .062, 12, .04, 14, .025, 16, .016, 18, .01, 20, .005, 24, 0]
EXIT_SCALE = [0, 1, 2, .997, 4, .992, 6, .985, 8, .974, 10, .959, 12, .938, 14, .907, 16, .866, 18, .81, 20, .73, 22, .60, 23, .56, 28, .5]
ENTRY_GAIN = [0, .35, 3, 1, 5, 1.3, 10, 1.15, 18, 1.03, 26, 1]
EXIT_SIGMA = [.56, 9, .7, 9, .77, 8.4, .82, 7.4, .85, 6.2, .88, 4.8, .92, 2.8, .93, .5, 1, 0]
ENTRY_SIGMA = [0, 12, 2, 9, 4, 6, 5, 5, 6, 4, 8, 3, 10, 2, 12, 1.5, 14, 1, 17, .4, 20, 0]
REST_DRIFT = 0.0011  # the layer keeps shrinking through its rest, share per frame
OUT = dict(r=[0, 1, 3, 1, 5, .74, 7, .5], g=[0, 1, 2, .8, 3, .62, 4, .42, 5, .32, 7, .28],
           b=[0, 1, 2, .92, 3, .8, 4, .58, 5, .46, 7, .38], a=[0, 1, 4, 1, 7, 0])
TILE_DECAY = 5.5  # frames, first layer: each tile falls from 2.8 towards its place on this constant

# Layer changes. entry and exit are inclusive recording frames, end is the scale an exit reaches,
# mid is the centre of the rest the grid was read on.
GENRE = [
    dict(pool="wide2", rows="mid", slot=540, entry=(853, 878), exit=(896, 917), mid=886.5),
    dict(pool="action", rows="low", slot=688, entry=(924, 947), exit=(966, 988), mid=956.5),
    dict(pool="drama", rows="high", slot=270, entry=(995, 1018), exit=(1035, 1059), mid=1025.5),
    dict(pool="comedy", rows="mid", slot=540, entry=(1062, 1088), exit=(1105, 1128), mid=1096),
    dict(pool="thriller", rows="low", slot=688, entry=(1134, 1158), exit=(1176, 1198), mid=1166.5),
    dict(pool="kids", rows="high", slot=270, entry=(1205, 1228), exit=(1246, 1268), mid=1236.5),
    dict(pool="world", rows=None, slot=150, entry=(1271, 1300), exit=(1316, 1338), mid=1301),
]

class Art:
    """Tiles by pool. A title appears once across the film until a pool runs out."""
    SKIP = {"After", "Midsommar", "Hereditary", "Alien: Covenant", "The Conjuring", "Nightcrawler"}

    def __init__(self, root):
        self.root = Path(root)
        picked = json.loads((self.root / "picked.json").read_text())
        wide = [i for i in picked["wide"] if i["title"] not in self.SKIP]
        rng = np.random.default_rng(7)
        rng.shuffle(wide)
        self.pools = {k: list(v) for k, v in picked.items() if k != "wide"}
        for k in self.pools:
            rng.shuffle(self.pools[k])
        third = len(wide) // 3
        everything = [i for k, v in picked.items() if k != "kids" for i in v if i["title"] not in self.SKIP]
        rng.shuffle(everything)
        self.pools.update(wall=everything, small=wide[:2 * third] + picked["world"][:8] + picked["drama"][:8], wide=wide[2 * third:] + picked["thriller"][:10], wide2=wide[third:2 * third] + picked["comedy"][:8])
        self.at = {}
        self.cache = {}

    def next(self, pool):
        items = self.pools[pool]
        i = self.at.get(pool, 0)
        self.at[pool] = i + 1
        return items[i % len(items)]["key"]

    def tile(self, key):
        if key not in self.cache:
            self.cache[key] = cv2.imread(str(self.root / "tiles" / f"{key}.jpg"))
        return self.cache[key]

    def poster(self, key):
        im = cv2.imread(str(self.root / "art" / key / "poster.jpg"))
        h, w = im.shape[:2]
        want = int(h * 6.4 / 9)
        x = max(0, (w - want) // 2)
        return im[:, x:x + want]

def rounded(w, h, r):
    s = 4
    m = np.zeros((h * s, w * s), np.uint8)
    R = int(r * s)
    cv2.rectangle(m, (R, 0), (w * s - R, h * s), 255, -1)
    cv2.rectangle(m, (0, R), (w * s, h * s - R), 255, -1)
    for c in ((R, R), (w * s - R, R), (R, h * s - R), (w * s - R, h * s - R)):
        cv2.circle(m, c, R, 255, -1, cv2.LINE_AA)
    return cv2.resize(m, (w, h), interpolation=cv2.INTER_AREA).astype(np.float32) / 255

def paste(canvas, img, x, y, w, h):
    """Draw one tile onto a BGRA premultiplied canvas. x and y are video coordinates."""
    ox, oy = (PAD_W - VW) // 2, (PAD_H - VH) // 2
    x, y = int(round(x)) + ox, int(round(y)) + oy
    if x >= PAD_W or y >= PAD_H or x + w <= 0 or y + h <= 0:
        return
    t = cv2.resize(img, (w, h), interpolation=cv2.INTER_AREA).astype(np.float32)
    a = rounded(w, h, RADIUS * w)
    x0, y0, x1, y1 = max(x, 0), max(y, 0), min(x + w, PAD_W), min(y + h, PAD_H)
    t, a = t[y0 - y:y1 - y, x0 - x:x1 - x], a[y0 - y:y1 - y, x0 - x:x1 - x]
    canvas[y0:y1, x0:x1, :3] = t * a[..., None] + canvas[y0:y1, x0:x1, :3] * (1 - a[..., None])
    canvas[y0:y1, x0:x1, 3] = a * 255 + canvas[y0:y1, x0:x1, 3] * (1 - a)

def piece(img, w, h):
    """One tile at its rest size, premultiplied, with rounded corners."""
    t = cv2.resize(img, (w, h), interpolation=cv2.INTER_AREA).astype(np.float32)
    a = rounded(w, h, RADIUS * w)
    return np.dstack([t * a[..., None], a * 255])

def flat_layer(art, g):
    c = np.zeros((PAD_H, PAD_W, 4), np.float32)
    rng = np.random.default_rng(len(g["pool"]) * 13 + g["slot"])
    tiles = []
    if g["rows"] is None:
        spots = [(x, y, SIX["w"], SIX["h"]) for y in SIX["ys"] for x in SIX["xs"]]
    else:
        spots = [(x, y, TILE_W, TILE_H) for y in ROW_SETS[g["rows"]] for x in COLS]
    for x, y, w, h in spots:
        img = art.tile(art.next(g["pool"]))
        paste(c, img, x, y, w, h)
        if -1500 < x < 3400 and -1100 < y < 2100:
            tiles.append(dict(x=x, y=y, w=w, h=h, img=piece(img, w, h), fade=float(rng.uniform(-5, 5)), wander=float(rng.normal(0, 1))))
    return c, tiles

def small_layer(art):
    c = np.zeros((PAD_H, PAD_W, 4), np.float32)
    s = SMALL
    rng = np.random.default_rng(11)
    tiles = []
    for r in range(-8, 12):
        y = s["top"] + (r - 1) * s["pitch_y"]
        x0 = s["x_even"] if (r % 2) else s["x_odd"]
        for k in range(-5, 8):
            x = x0 + k * s["pitch_x"]
            img = art.tile(art.next("small"))
            paste(c, img, x, y, s["w"], s["h"])
            if -400 < y < 1300 and -500 < x < 2500:
                # The tile that lands first in the recording sits low on the left.
                near = float(np.hypot((x - 300) / 900, (y - 620) / 500))
                tiles.append(dict(x=x, y=y, w=s["w"], h=s["h"], img=piece(img, s["w"], s["h"]), start=654.0 + 12.0 * min(1.0, 0.12 * near + 0.92 * float(rng.uniform(0, 1)) ** 0.35)))
    return c, tiles

def tilted(art, plane_to_video, tiles, pool, size=(2600, 1700)):
    """Draw plane tiles through a homography at twice size, then bring it down."""
    w, h = size
    ox, oy = (w - VW) / 2, (h - VH) / 2
    S = np.array([[2, 0, ox * 2], [0, 2, oy * 2], [0, 0, 1]], np.float64) @ plane_to_video
    c = np.zeros((h * 2, w * 2, 4), np.float32)
    for x0, y0, tw, th, portrait in tiles:
        key = art.next(pool)
        img = art.poster(key) if portrait else art.tile(key)
        ih, iw = img.shape[:2]
        H = S @ cv2.getPerspectiveTransform(np.float32([[0, 0], [iw, 0], [iw, ih], [0, ih]]),
                                            np.float32([[x0, y0], [x0 + tw, y0], [x0 + tw, y0 + th], [x0, y0 + th]]))
        corners = cv2.perspectiveTransform(np.float32([[[0, 0]], [[iw, 0]], [[iw, ih]], [[0, ih]]]), H).reshape(-1, 2)
        if corners[:, 0].max() < 0 or corners[:, 0].min() > w * 2 or corners[:, 1].max() < 0 or corners[:, 1].min() > h * 2:
            continue
        a = rounded(iw, ih, RADIUS * iw * (1.0 if not portrait else 16 / 6.4 * 0.5))
        rgb = cv2.warpPerspective(img.astype(np.float32) * a[..., None], H, (w * 2, h * 2), flags=cv2.INTER_LINEAR)
        al = cv2.warpPerspective(a, H, (w * 2, h * 2), flags=cv2.INTER_LINEAR)
        c[..., :3] = rgb + c[..., :3] * (1 - al[..., None])
        c[..., 3] = al * 255 + c[..., 3] * (1 - al)
    out = cv2.resize(c, (w, h), interpolation=cv2.INTER_AREA)
    canvas = np.zeros((PAD_H, PAD_W, 4), np.float32)
    x, y = (PAD_W - w) // 2, (PAD_H - h) // 2
    canvas[y:y + h, x:x + w] = out
    return canvas

def wall_tiles():
    H = cv2.getPerspectiveTransform(np.float32([[0, 0], [16, 0], [16, 9], [0, 9]]),
                                    np.float32([[1312, 222], [1780, 262], [1692, 532], [1222, 458]])).astype(np.float64)
    L, P, GAP, PITCH = 16.0, 6.4, 0.5, 9.7
    anchors = {-1: (0.5, "PLPLLPL", "LPPLPLLPLLPLPL"), 0: (0.0, "LPLPL", "PPLLPLPLLPLPLL"), 1: (-5.2, "LPLPLP", "PLPLLPLPLLPLPL"),
               2: (-27.4, "LPPLPLPL", "LPLLPLPLLPLP"), 3: (-40.0, "LPPLLPLP", "PLLPLPLPLL"), 4: (-36.0, "LPLPLLPL", "LPLLPLPL"), 5: (-30.0, "LLPLPL", "PLPLLP"), -2: (-40.0, "LPLLPPLL", "LPLPLLPL"), -3: (-30.0, "LLPLPLL", "PLPLLPL")}
    tiles = []
    for r, (x, right, left) in anchors.items():
        b = r * PITCH
        cur = x
        for ch in right:
            w = L if ch == "L" else P
            tiles.append((cur, b, w, 9.0, ch == "P"))
            cur += w + GAP
        cur = x
        for ch in left:
            w = L if ch == "L" else P
            cur -= w + GAP
            tiles.append((cur, b, w, 9.0, ch == "P"))
    return H, tiles

def second_tiles():
    P = 17.85
    tops = lambda x: 48 + 0.136 * (x - 195)
    bots = lambda x: 405 + 0.061 * (x - 195)
    src, dst = [], []
    for i, x in enumerate([195, 605, 957, 1265, 1525, 1765]):
        src += [[i * P, 0], [i * P, 9]]
        dst += [[x, tops(x)], [x, bots(x)]]
    H, _ = cv2.findHomography(np.float32(src), np.float32(dst), 0)
    tiles = [(c * P + (P / 2 if r % 2 else 0) - (P if r % 2 else 0), r * 11.0, 16.0, 9.0, False) for r in range(-5, 7) for c in range(-3, 12)]
    return H, tiles

def backdrop():
    """Dark sheet with a blue band at the top and a violet and pink glow low down, made of blurred blobs."""
    b = np.zeros((VH, VW, 3), np.float32)
    b[:] = (30, 13, 12)
    rng = np.random.default_rng(3)
    for x in np.arange(-40, 2000, 150):
        cv2.circle(b, (int(x + rng.integers(-40, 40)), int(36 + rng.integers(-24, 24))), int(rng.integers(36, 62)),
                   tuple(float(v) for v in rng.choice([(190, 70, 24), (214, 86, 30), (150, 44, 40)])), -1, cv2.LINE_AA)
    for x in np.arange(-30, 2000, 118):
        cv2.circle(b, (int(x + rng.integers(-34, 34)), int(690 + rng.integers(-58, 58))), int(rng.integers(38, 72)),
                   tuple(float(v) for v in rng.choice([(170, 60, 120), (200, 56, 96), (205, 52, 150), (190, 60, 70)])), -1, cv2.LINE_AA)
    b = cv2.GaussianBlur(b, (0, 0), 46)
    y = np.arange(VH, dtype=np.float32)
    b *= (0.6 - 0.36 * smooth(760, 1000, y))[:, None, None]
    return b

def foot():
    """The fixed dimming along the bottom of the picture, under the buttons."""
    y = np.arange(VH, dtype=np.float32)
    return np.interp(y, [0, 860, 960, 1080], [1, 1, .32, .22]).astype(np.float32)[:, None, None]

def wall_light():
    x, y = np.meshgrid(np.arange(VW, dtype=np.float32), np.arange(VH, dtype=np.float32))
    return (0.34 + 0.66 * smooth(120, 860, x + 0.34 * (y - 540)))[..., None]

def grade(stops, b, g, r):
    """How a layer sits under its headline: colour kept high up, darker and cooler low down."""
    y = np.arange(VH, dtype=np.float32) / VH
    return np.dstack([np.tile(np.interp(y, stops, c)[:, None], (1, VW)) for c in (b, g, r)]).astype(np.float32)

def text_image(font_path, line, size, centre_y):
    """White text on black, as an alpha picture one line tall, positioned on a full frame."""
    font = ImageFont.truetype(font_path, size)
    font.set_variation_by_axes([800])
    im = Image.new("L", (VW, VH), 0)
    d = ImageDraw.Draw(im)
    track = -0.014 * size
    text = line.upper()
    width = sum(d.textlength(ch, font=font) for ch in text) + track * (len(text) - 1)
    box = d.textbbox((0, 0), "H", font=font)
    x = (VW - width) / 2
    y = centre_y - (box[1] + box[3]) / 2
    for ch in text:
        d.text((x, y), ch, font=font, fill=255)
        x += d.textlength(ch, font=font) + track
    return np.asarray(im, np.float32) / 255

def warp(src, scale, tx=0.0, ty=0.0, rot=0.0, centre=None):
    cx, cy = centre if centre else (src.shape[1] / 2, src.shape[0] / 2)
    t = math.radians(rot)
    a, b = scale * math.cos(t), scale * math.sin(t)
    M = np.array([[a, -b, PIVOT[0] + tx - a * cx + b * cy], [b, a, PIVOT[1] + ty - b * cx - a * cy]], np.float64)
    return cv2.warpAffine(src, M, (VW, VH), flags=cv2.INTER_LINEAR, borderMode=cv2.BORDER_CONSTANT, borderValue=0)

def blur(x, sigma):
    if sigma < 0.3:
        return x
    if sigma > 6:
        f = 0.25
        s = cv2.resize(x, None, fx=f, fy=f, interpolation=cv2.INTER_AREA)
        s = cv2.GaussianBlur(s, (0, 0), sigma * f)
        return cv2.resize(s, (x.shape[1], x.shape[0]), interpolation=cv2.INTER_LINEAR)
    return cv2.GaussianBlur(x, (0, 0), sigma)

def tint(n_from_start, inbound):
    """Headline colour and alpha. Going out, white passes through pink. Coming in is the reverse."""
    t = 7 - n_from_start * 7 / 6 if inbound else n_from_start
    return [table(OUT[k], t) for k in "rgba"]

class Film:
    def __init__(self, art_root, font, logo):
        self.art = Art(art_root)
        lines = [l.strip() for l in Path(__file__).with_name("headlines.txt").read_text().splitlines() if l.strip() and not l.startswith("#")]
        self.heads = [[p.strip() for p in l.split("|")] for l in lines]
        self.font = font
        print("building layers", flush=True)
        H, tiles = wall_tiles()
        self.wall = tilted(self.art, H, tiles, "wall")
        self.first, self.first_tiles = small_layer(self.art)
        H, tiles = second_tiles()
        self.second = tilted(self.art, H, tiles, "wide", size=(3000, 1900))
        self.genre = [flat_layer(self.art, g) for g in GENRE]
        self.back = backdrop()
        self.foot = foot()
        self.light = wall_light()
        self.grade1 = grade([0, .35, .6, .8, 1], [1.02, .98, .88, .72, .46], [.82, .76, .62, .42, .2], [.78, .78, .7, .54, .34])
        self.grade2 = grade([0, .42, .55, .8, 1], [.96, .9, .86, .72, .4], [.9, .84, .58, .4, .2], [.9, .86, .6, .44, .28])
        im = cv2.imread(str(logo), cv2.IMREAD_UNCHANGED).astype(np.float32)
        im[..., :3] *= im[..., 3:4] / 255
        c = np.zeros((VH, VW, 4), np.float32)
        h, w = im.shape[:2]
        x, y = (VW - w) // 2, 540 - h // 2 - 18
        c[y:y + h, x:x + w] = im
        self.logo = c
        # Headline pictures: two line slots at y 414 and 574, cap 97; one line slots at the layer's slot, cap 85.
        self.text = []
        for i, parts in enumerate(self.heads):
            if i < 3:
                self.text.append([text_image(font, parts[0], 133, 410), text_image(font, parts[1], 133, 571)])
            else:
                self.text.append([text_image(font, parts[0], 117, GENRE[i - 3]["slot"])])
        print("layers ready", flush=True)

    def spread(self, tiles, place):
        """Draw tiles one at a time. place gives each tile's scale, alpha, sigma, gain and shift."""
        c = np.zeros((VH, VW, 4), np.float32)
        items = [(t, place(t)) for t in tiles]
        for t, (s, a, sig, gain, tx) in sorted(items, key=lambda i: i[1][0]):
            if a <= 0:
                continue
            w, h = max(2, int(round(t["w"] * s))), max(2, int(round(t["h"] * s)))
            x = PIVOT[0] + s * (t["x"] + tx - PIVOT[0])
            y = PIVOT[1] + s * (t["y"] - PIVOT[1])
            if x > VW or y > VH or x + w < 0 or y + h < 0:
                continue
            p = cv2.resize(t["img"], (w, h), interpolation=cv2.INTER_AREA if s < 1 else cv2.INTER_LINEAR)
            if sig > 0.3:
                m = int(3 * sig) + 1
                p = cv2.GaussianBlur(cv2.copyMakeBorder(p, m, m, m, m, cv2.BORDER_CONSTANT, value=0), (0, 0), sig)
                x -= m
                y -= m
                h, w = p.shape[:2]
            x, y = int(round(x)), int(round(y))
            x0, y0, x1, y1 = max(x, 0), max(y, 0), min(x + w, VW), min(y + h, VH)
            if x1 <= x0 or y1 <= y0:
                continue
            q = p[y0 - y:y1 - y, x0 - x:x1 - x] * a
            k = q[..., 3:4] / 255
            c[y0:y1, x0:x1, :3] = q[..., :3] * gain + c[y0:y1, x0:x1, :3] * (1 - k)
            c[y0:y1, x0:x1, 3] = q[..., 3] + c[y0:y1, x0:x1, 3] * (1 - k[..., 0])
        return c

    def over(self, out, layer, alpha=1.0, gain=1.0, sigma=0.0, **kw):
        w = warp(layer, **kw)
        w = blur(w, sigma)
        a = w[..., 3:4] / 255 * alpha
        return out * (1 - a) + w[..., :3] * alpha * gain

    def words(self, out, pic, scale, sigma, rgba):
        a = blur(warp(pic, scale), sigma)[..., None] * rgba[3]
        colour = np.array([rgba[2], rgba[1], rgba[0]], np.float32) * 248
        return out * (1 - a) + colour * a

    def frame(self, n):
        out = np.zeros((VH, VW, 3), np.float32)
        # Backdrop arrives with the first tile layer.
        back = float(smooth(656, 672, n))
        if back > 0:
            out += self.back * back
        # Wall.
        if 589 <= n <= 650:
            a = table([589, 0, 590, .49, 591, .9, 592, 1, 636, .94, 638, .81, 640, .71, 642, .61, 644, .48, 646, .33, 648, .16, 650, .03, 651, 0], n)
            lit = warp(self.wall, 1.0)
            lit[..., :3] *= self.light
            k = lit[..., 3:4] / 255 * a
            out = out * (1 - k) + lit[..., :3] * a
        # Logo on the card.
        if 636 <= n <= 666:
            s = table([636, 1, 650, 1, 654, .9575, 658, .8679, 666, .58], n)
            a = table([636, 0, 638, .3, 642, 1, 658, 1, 666, 0], n)
            out = self.over(out, self.logo, alpha=a, scale=s)
        # First layer: each tile flies in on its own clock, then the sheet pans right under still text.
        if 654 <= n <= 736 or 846 <= n <= 857:
            a = table([733, 1, 734, .5, 735, 0], n) if n < 800 else table([846, 0, 849, .9, 853, .9, 857, 0], n)
            pan = 1.42 * (n - 713.5)
            if n < 702:
                def place(t):
                    f = n - t["start"]
                    if f < 0:
                        return 1.0, 0.0, 0.0, 1.0, 0.0
                    s = 1 + 1.8 * math.exp(-f / TILE_DECAY)
                    return s, min(1.0, (f + 1) / 2), min(7.0, 6 * (s - 1)) if s > 1.05 else 0.0, 1 + 0.22 * math.exp(-f / 8), pan
                w = self.spread(self.first_tiles, place)
            else:
                w = blur(warp(self.first, 1.0, tx=pan), 4.5 if n > 800 else 0)
            dim = table([672, 0, 682, .4, 694, 1], n)
            al = w[..., 3:4] / 255
            rgb = w[..., :3] * (1 - dim + dim * self.grade1) * (1.45 if n > 800 else 1.0)
            out = out * (1 - al * a) + rgb * a
        # Second layer: a slow move through a tilted sheet, under still text.
        if 733 <= n <= 856:
            d = n - 770
            sig = table([845, 0, 853, 10], n)
            a = table([733, 0, 734, .5, 735, 1, 845, 1, 850, 0], n)
            w = warp(self.second, 1 - 0.00143 * d, tx=1.31 * d, ty=0.14 * d, rot=-0.03 * d)
            w = blur(w, sig)
            al = w[..., 3:4] / 255
            out = out * (1 - al * a) + w[..., :3] * self.grade2 * a
        # Genre layers.
        for i, g in enumerate(GENRE):
            e0 = g["entry"][0]
            x0, x1 = g["exit"]
            if n < e0 or n > x1 + 6:
                continue
            drift = 1 - REST_DRIFT * (min(n, x0) - g["mid"])
            s_entry = 1 + table(ENTRY_EXCESS, n - e0)
            s_exit = table(EXIT_SCALE, n - x0) if n >= x0 else 1.0
            s = s_entry * s_exit * drift
            sig = table(ENTRY_SIGMA, n - e0)
            gain = table(ENTRY_GAIN, n - e0)
            sheet, tiles = self.genre[i]
            if n >= x0:
                sig = max(sig, min(5.5, table(EXIT_SIGMA, s_exit) * 0.8))
            if n >= x0 + 10:
                ramp = float(np.clip((.93 - s_exit) / .33, 0, 1))
                def place(t):
                    return s * (1 + 0.05 * t["wander"] * ramp), 1 - float(smooth(x1 + t["fade"] - 2, x1 + t["fade"] + 1, n)), 0.0, 1.0, 0.0
                w = blur(self.spread(tiles, place), sig)
                k = w[..., 3:4] / 255
                out = out * (1 - k) + w[..., :3]
            else:
                out = self.over(out, sheet, alpha=min(1, gain), gain=max(1, gain), sigma=sig, scale=s)
            # Its headline lives in the layer.
            pic = self.text[i + 3][0]
            if e0 <= n <= x1:
                rgba = tint(min(6, n - e0 + 1), True) if n < e0 + 6 else (tint(min(7, n - x1 + 6), False) if n >= x1 - 6 else [1, 1, 1, 1])
                out = self.words(out, pic, s, sig, rgba)
        # Headlines 1 to 3, over the first two layers.
        spans = [(0, 678, 727), (1, 733, 784), (2, 790, 841)]
        for h, t_in, t_out in spans:
            for k, pic in enumerate(self.text[h]):
                lag = 3 * k
                if n < t_in + lag or n > t_out + 7:
                    continue
                rgba = tint(n - t_in - lag, True) if n < t_in + lag + 6 else (tint(n - t_out, False) if n >= t_out else [1, 1, 1, 1])
                s = 1 + 1.8 * math.exp(-(n - 655) / 9.5) if h == 0 else 1.0
                out = self.words(out, pic, s, 0, rgba)
        return np.clip(out * self.foot, 0, 255).astype(np.uint8)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--art", required=True)
    ap.add_argument("--font", required=True)
    ap.add_argument("--logo", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--stills", default="")
    ap.add_argument("--still-dir", default="")
    ap.add_argument("--ffmpeg", default="")
    ap.add_argument("--dump", default="")
    a = ap.parse_args()
    film = Film(a.art, a.font, a.logo)
    wanted = {int(x) for x in a.stills.split(",") if x}
    if wanted:
        Path(a.still_dir).mkdir(parents=True, exist_ok=True)
        for n in sorted(wanted):
            cv2.imwrite(f"{a.still_dir}/o{n:05d}.jpg", film.frame(n), [cv2.IMWRITE_JPEG_QUALITY, 93])
        print("stills", len(wanted))
        return
    p = subprocess.Popen([a.ffmpeg, "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "bgr24", "-s", f"{VW}x{VH}", "-r", str(FPS), "-i", "-",
                          "-c:v", "libx264", "-crf", "15", "-preset", "medium", "-pix_fmt", "yuv420p", "-movflags", "+faststart", a.out], stdin=subprocess.PIPE)
    if a.dump:
        Path(a.dump).mkdir(parents=True, exist_ok=True)
    for n in range(F0, F_END):
        f = film.frame(n)
        p.stdin.write(f.tobytes())
        if a.dump:
            cv2.imwrite(f"{a.dump}/o{n:05d}.jpg", f, [cv2.IMWRITE_JPEG_QUALITY, 92])
        if n % 100 == 0:
            print("frame", n, flush=True)
    p.stdin.close()
    p.wait()
    print("wrote", a.out)

if __name__ == "__main__":
    main()
