#!/usr/bin/env python3
"""The Streams opening, in two separate parts with loading between them.

The play mark is the logo. The text is only text set beside it, the way YouTube works, so the text
always comes out of the mark.

Stage 1, the splash: the mark arrives and the text comes out of it. It stays as mark and text.
Loading: mark and text rest with the loading circle while the app does its work.
Stage 2, the secondary splash: starts when loading ends. The text goes back and the mark opens the landing.

Directions:
  A  the ribbon unfolds from behind the arrow, the text slides out. Streams leave the arrow where it sits.
  B  nothing slides. The mark pours into place where it stays, the letters ripple out of it, and in
     stage two the letters stretch into the streams.
  C  a line pulls back into the arrow, the text slides out. The film is seen through the growing arrow.
Moves use one smooth curve and take at least half a second, so nothing jumps.
"""
import argparse, io, math, re, subprocess
from pathlib import Path
import cv2, numpy as np, resvg_py
from PIL import Image

VW, VH, FPS = 1920, 1080, 60
ICON = "~/Desktop/streams-stream-play-icons/assets-logos/source-icon.svg"
BG = np.array([15, 7, 6], np.float32)
LOCK_W, RATIO, OVER, BIG = 760.0, 3.922, 2.2, 1.4
LOAD = 2.6
CARD = (55, 169, 1596, 1033)
FILM_SECONDS = 3.4

def smooth(x):
    x = np.clip(x, 0, 1)
    return x * x * x * (x * (6 * x - 15) + 10)

def rise(x, p=2.0):
    return float(np.clip(x, 0, 1) ** p)

def span(t, a, b):
    return float(np.clip((t - a) / (b - a), 0, 1))

def rgba(png):
    im = np.asarray(Image.open(io.BytesIO(bytes(png))).convert("RGBA"), np.float32)
    a = im[..., 3:4] / 255
    return np.dstack([im[..., 2:3] * a, im[..., 1:2] * a, im[..., 0:1] * a, im[..., 3:4]])

def place(out, lay, cx, cy, lcx, lcy, sx, sy=None, alpha=1.0, clip_left=None, mask=None):
    sy = sx if sy is None else sy
    M = np.array([[sx, 0, cx - sx * lcx], [0, sy, cy - sy * lcy]], np.float64)
    w = cv2.warpAffine(lay, M, (VW, VH), flags=cv2.INTER_LINEAR)
    if clip_left is not None:
        w[:, : max(0, int(clip_left))] = 0
    if mask is not None:
        w = w * mask
    a = w[..., 3:4] / 255 * alpha
    return out * (1 - a) + w[..., :3] * alpha

def wipe(edge, soft):
    """1 left of the edge, 0 right of it, with a soft boundary, one row wide."""
    x = np.arange(VW, dtype=np.float32)
    return (1 - smooth((x - (edge - soft)) / (2 * soft)))[None, :, None]

class Art:
    def __init__(self, word_svg):
        s = Path(ICON).read_text()
        rib = re.search(r'<path d="[^"]+" fill="url\(#ig0\)"/>', s).group(0)
        arr = re.search(r'<path d="[^"]+" fill="url\(#ig1\)"/>', s).group(0)
        art = Image.open(io.BytesIO(bytes(resvg_py.svg_to_bytes(svg_string=s, width=1024, height=1024)))).convert("RGBA").getbbox()
        self.mh = LOCK_W / RATIO
        u = self.mh / (art[3] - art[1]) * OVER
        n = int(1024 * u)
        self.rib = rgba(resvg_py.svg_to_bytes(svg_string=s.replace(arr, "").split("<g opacity")[0] + "</g></svg>", width=n, height=n))
        self.arr = rgba(resvg_py.svg_to_bytes(svg_string=s.replace(rib, ""), width=n, height=n))
        l, t, r, b = art
        self.lcx, self.lcy = (l + r) / 2 * u, (t + b) / 2 * u
        self.mw = self.mh * (r - l) / (b - t)
        cap = self.mh / 1.42
        self.cap = cap
        self.word = rgba(resvg_py.svg_to_bytes(svg_path=word_svg, height=int(cap * 1.08 * 3)))  # three times size
        cols = self.word[..., 3].sum(0)
        i = int(np.argmax(cols > 0))
        while cols[i] > 0:
            i += 1
        self.split = i
        self.s_only = self.word.copy(); self.s_only[:, i:] = 0
        self.ww = self.word.shape[1] / 3
        self.wx = (VW - LOCK_W) / 2 + self.mw + cap * .20
        self.mark_x = (VW - LOCK_W) / 2 + self.mw / 2
        inside = (self.arr[..., 3] > 127).astype(np.uint8)
        self.sdf = (cv2.distanceTransform(inside, cv2.DIST_L2, 5) - cv2.distanceTransform(1 - inside, cv2.DIST_L2, 5)).astype(np.float32)
        al = np.maximum(self.arr[..., 3:4] / 255, 1e-3)
        c = np.clip(self.arr[..., :3] / al, 0, 255)
        c[self.arr[..., 3] < 200] = 0
        for _ in range(4):
            c = np.where(c.sum(2, keepdims=True) > 0, c, cv2.dilate(c, np.ones((9, 9), np.uint8)))
        self.arr_col = cv2.GaussianBlur(c, (0, 0), 3)

    def mark(self, out, cx, cy, s=1.0, gap=0.0, alpha=1.0, rib_alpha=None, sx=None):
        k = s / OVER; kx = (sx if sx else s) / OVER
        out = place(out, self.rib, cx - gap, cy, self.lcx, self.lcy, kx, k, alpha if rib_alpha is None else rib_alpha)
        return place(out, self.arr, cx + gap, cy, self.lcx, self.lcy, kx, k, alpha)

    def put_word(self, out, x, cy, scale=1.0, alpha=1.0, clip_left=None, mask=None, lay=None):
        lay = self.word if lay is None else lay
        return place(out, lay, x, cy, 0, lay.shape[0] / 2, scale / 3, None, alpha, clip_left, mask)

def ring(out, t, alpha):
    if alpha <= 0:
        return out
    r, sw, cx, cy = 33, 8, int(.498 * VW), int(.804 * VH)
    S = 4; n = 120
    c = np.zeros((n * S, n * S), np.uint8)
    ang = (t * 347) % 360
    cv2.ellipse(c, (n * S // 2, n * S // 2), (r * S, r * S), ang, 0, 280, 255, sw * S, cv2.LINE_AA)
    a = cv2.resize(c, (n, n), interpolation=cv2.INTER_AREA).astype(np.float32)[..., None] / 255 * alpha
    y0, x0 = cy - n // 2, cx - n // 2
    out[y0:y0 + n, x0:x0 + n] = out[y0:y0 + n, x0:x0 + n] * (1 - a) + np.array([255, 107, 47], np.float32) * a
    return out

class Streams:
    """Bands that open out of the arrow's nose, fill the picture and run off to the right."""
    PAL = [(255, 107, 47), (242, 63, 123), (224, 56, 67), (189, 75, 29), (255, 160, 110), (190, 163, 247), (255, 255, 255)]
    WEIGHT = [.32, .3, .14, .12, .06, .04, .02]

    def __init__(self, curved, nose):
        r = np.random.default_rng(21)
        n = 44
        self.nx, self.ny = nose[0], nose[1]
        self.y = np.sort(r.uniform(-40, VH + 40, n))
        self.th = r.choice([3, 5, 9, 16, 28, 46, 70], n, p=[.2, .2, .2, .16, .12, .08, .04]).astype(float)
        self.col = r.choice(len(self.PAL), n, p=self.WEIGHT)
        self.th[self.col >= 4] = np.minimum(self.th[self.col >= 4], 7)  # pale bands stay thin
        self.lag = r.uniform(0, .16, n)
        self.leave = r.uniform(0, .24, n)
        self.flow = r.uniform(0, VW + 900, (n, 3))

    def draw(self, out, u, length):
        """u is seconds since the streams began."""
        layer = np.zeros((VH, VW, 3), np.uint8)
        grow = .85
        for i in range(len(self.y)):
            p = span(u, self.lag[i], grow + self.lag[i])
            if p <= 0:
                continue
            head = self.nx + (VW + 700 - self.nx) * float(smooth(p)) ** 1.25
            fan = float(smooth(span(u, .02, grow + .1)))
            lv = span(u, length - .8 + self.leave[i], length + self.leave[i])
            tail = self.nx - (self.nx + 900) * fan + (VW + 2200) * float(smooth(lv)) ** 1.4
            if tail >= VW or head <= tail:
                continue
            y0 = self.ny + (self.y[i] - self.ny) * fan
            th = max(1.5, self.th[i] * fan)
            taper = th * 8 + 80
            xs = np.linspace(tail, head, 8)
            k = np.clip((xs - tail) / max(taper, 1), 0, 1) * np.clip((head - xs) / max(taper, 1), 0, 1) ** .6
            w = th * np.sqrt(k)
            c = y0 + (y0 - self.ny) * .1 * (xs / VW)
            pts = np.vstack([np.stack([xs, c - w / 2], 1), np.stack([xs[::-1], (c + w / 2)[::-1]], 1)])
            col = self.PAL[self.col[i]]
            cv2.fillPoly(layer, [np.int32(pts * 16)], col, cv2.LINE_AA, shift=4)
            hi = tuple(int(v + (255 - v) * .62) for v in col)
            for f0 in self.flow[i]:
                x_c = (f0 + u * 2100) % (VW + 900) - 450
                a0, a1 = max(tail + taper, x_c - 240), min(head - taper, x_c + 240)
                if a1 - a0 > 30:
                    yc = y0 + (y0 - self.ny) * .1 * (x_c / VW)
                    h = max(1.0, th * .34)
                    cv2.fillPoly(layer, [np.int32(np.array([[a0, yc], [(a0 + a1) / 2, yc - h], [a1, yc], [(a0 + a1) / 2, yc + h]]) * 16)], hi, cv2.LINE_AA, shift=4)
        f = layer.astype(np.float32)
        glow = cv2.resize(cv2.GaussianBlur(cv2.resize(f, None, fx=.25, fy=.25, interpolation=cv2.INTER_AREA), (0, 0), 10), (VW, VH), interpolation=cv2.INTER_LINEAR)
        m = cv2.GaussianBlur(np.clip(f.max(2, keepdims=True) / 90, 0, 1), (0, 0), .7)[..., None]
        return out * (1 - m) + f * m + glow * .5

class Poured:
    """Direction B. The logo stays where it is for the whole film and nothing slides."""
    def __init__(self, art):
        self.a = art
        cy = VH / 2
        blank = np.zeros((VH, VW, 3), np.float32)
        def still(lay):
            k = 1 / OVER
            M = np.array([[k, 0, art.mark_x - k * art.lcx], [0, k, cy - k * art.lcy]], np.float64)
            return cv2.warpAffine(lay, M, (VW, VH), flags=cv2.INTER_LINEAR)
        self.rib, self.arr = still(art.rib), still(art.arr)
        y, x = np.mgrid[0:VH, 0:VW].astype(np.float32)
        def tail_of(im):
            ys, xs = np.where(im[..., 3] > 30)
            i = np.argmax(ys - xs * .35)
            return xs[i], ys[i]
        self.d_rib = np.hypot(*(lambda p: (x - p[0], y - p[1]))(tail_of(self.rib)))
        self.d_arr = np.hypot(*(lambda p: (x - p[0], y - p[1]))(tail_of(self.arr)))
        self.r_rib = float(self.d_rib[self.rib[..., 3] > 10].max())
        self.r_arr = float(self.d_arr[self.arr[..., 3] > 10].max())
        # Letters: split the text at the gaps between them.
        cols = art.word[..., 3].sum(0) > 0
        edges = np.flatnonzero(np.diff(cols.astype(np.int8)))
        runs = [(edges[i] + 1, edges[i + 1] + 1) for i in range(0, len(edges) - 1, 2)]
        self.letters = []
        for x0, x1 in runs:
            lay = art.word.copy(); lay[:, :x0] = 0; lay[:, x1:] = 0
            self.letters.append(lay)
        # Streaks start at the right hand end of each row of the logo.
        word = cv2.warpAffine(art.word, np.array([[1 / 3, 0, art.wx], [0, 1 / 3, cy - art.word.shape[0] / 6]], np.float64), (VW, VH))[..., 3]
        r = np.random.default_rng(9)
        rows = np.flatnonzero((word > 120).any(1))
        self.row_group = np.full(VH, -1)
        y_ = rows.min()
        while y_ < rows.max():
            self.row_group[y_:y_ + 3] = r.integers(0, 3)
            y_ += int(r.integers(7, 12))
        pick = np.linspace(rows.min(), rows.max(), 46).astype(int)
        self.oy = pick.astype(np.float32)
        self.ox = np.array([np.flatnonzero(word[v] > 120).max() if (word[v] > 120).any() else art.wx + art.ww for v in pick], np.float32)
        self.ty = np.linspace(-30, VH + 30, len(pick)) + r.uniform(-9, 9, len(pick))
        r.shuffle(self.ty)
        self.th = r.choice([4, 7, 11, 18, 28, 42, 62], len(pick), p=[.2, .2, .2, .16, .12, .08, .04]).astype(float)
        self.col = r.choice(len(Streams.PAL), len(pick), p=[.34, .3, .14, .12, .06, .03, .01])
        self.flow = r.uniform(0, VW + 900, (len(pick), 3))
        self.lag = r.uniform(0, .34, len(pick))
        self.leave = r.uniform(0, .3, len(pick))

    def logo(self, out, t):
        """Stage 1. Each shape pours from its sharp tail to its nose, the arrow overtaking the ribbon."""
        a = self.a
        for im, d, r, t0, t1 in ((self.rib, self.d_rib, self.r_rib, .2, 1.05), (self.arr, self.d_arr, self.r_arr, .5, 1.4)):
            front = (r + 80) * float(smooth(span(t, t0, t1)))
            if front <= 0:
                continue
            m = smooth((front - d) / 70)[..., None]
            lit = 1 + .35 * (np.exp(-((d - front + 35) / 26) ** 2))[..., None] * (front < r + 40)
            al = im[..., 3:4] / 255 * m
            out = out * (1 - al) + im[..., :3] * m * lit
        return out

    def text(self, out, t, gone=0.0):
        """The letters ripple out of the mark, one after the next."""
        a = self.a
        for i, lay in enumerate(self.letters):
            p = float(smooth(span(t, 1.25 + .075 * i, 1.95 + .075 * i)))
            if p <= 0:
                continue
            out = place(out, lay, a.wx - 34 * (1 - p), VH / 2, 0, lay.shape[0] / 2, 1 / 3, None, p * (1 - gone))
        return out

    def stage_two(self, t, u, length):
        a = self.a
        out = np.tile(BG, (VH, VW, 1)).copy()
        out = self.logo(out, 9.0) if u < 1.5 else out
        if u < 1.5 and (al := 1 - float(smooth(span(u, .75, 1.4)))) < 1:
            out = BG + (out - BG) * al
        # Each row of each letter stretches to the right, and the letters let go from the left.
        go = span(u, 0, .85)
        if go < 1:
            w = cv2.warpAffine(a.word, np.array([[1 / 3, 0, a.wx], [0, 1 / 3, VH / 2 - a.word.shape[0] / 6]], np.float64), (VW, VH))[..., 3] / 255
            base = w * (1 - float(smooth(span(u, .1, .55))))
            streak = np.zeros_like(w)
            for g, mult in enumerate((.55, 1.0, 1.5)):
                k = 1 + int(760 * mult * float(smooth(go)) ** 1.4)
                row = (self.row_group == g).astype(np.float32)[:, None]
                part = w * row
                streak = np.maximum(streak, cv2.dilate(part, np.ones((1, k), np.uint8), anchor=(k - 1, 0)) if k > 1 else part)
            streak *= 1 - wipe(a.wx - 80 + (a.ww + 1000) * float(smooth(span(u, .2, .85))), 110)[0, :, 0]
            streak *= float(smooth(span(u, 0, .12)))
            x = np.arange(VW, dtype=np.float32)[None, :, None]
            tone = np.clip((x - a.wx - a.ww * .6) / 620, 0, 1)
            colour = np.array([255, 255, 255], np.float32) * (1 - tone) + np.array([255, 107, 47], np.float32) * tone
            al = np.maximum(base, streak)[..., None]
            out = out * (1 - al) + colour * al
        layer = np.zeros((VH, VW, 3), np.uint8)
        for i in range(len(self.oy)):
            p = span(u, .1 + self.lag[i], 1.05 + self.lag[i])
            if p <= 0:
                continue
            head = self.ox[i] + (VW + 700 - self.ox[i]) * float(smooth(p)) ** 1.3
            fan = float(smooth(span(u, .45 + self.lag[i] * .5, 1.45)))
            lv = span(u, length - .85 + self.leave[i], length + self.leave[i])
            tail = self.ox[i] - (self.ox[i] + 900) * fan + (VW + 2200) * float(smooth(lv)) ** 1.4
            if tail >= VW or head <= tail:
                continue
            y0 = self.oy[i] + (self.ty[i] - self.oy[i]) * fan + (self.ty[i] - VH / 2) * .03 * max(0, u - .8)
            th = max(1.5, 2.6 + (self.th[i] - 2.6) * fan)
            xs = np.linspace(tail, head, 8)
            tp = th * 8 + 80
            q = np.clip((xs - tail) / tp, 0, 1) * np.clip((head - xs) / tp, 0, 1) ** .6
            wd = th * np.sqrt(q)
            pts = np.vstack([np.stack([xs, y0 - wd / 2], 1), np.stack([xs[::-1], (y0 + wd / 2)[::-1]], 1)])
            col = Streams.PAL[self.col[i]]
            cv2.fillPoly(layer, [np.int32(pts * 16)], col, cv2.LINE_AA, shift=4)
            # Light keeps moving along each stream, so the full picture still flows.
            hi = tuple(int(c + (255 - c) * .62) for c in col)
            for f0 in self.flow[i]:
                c = (f0 + u * 2100) % (VW + 900) - 450
                x0, x1 = max(tail + tp, c - 240), min(head - tp, c + 240)
                if x1 - x0 > 30:
                    h = max(1.0, th * .34)
                    seg = np.array([[x0, y0], [(x0 + x1) / 2, y0 - h], [x1, y0], [(x0 + x1) / 2, y0 + h]])
                    cv2.fillPoly(layer, [np.int32(seg * 16)], hi, cv2.LINE_AA, shift=4)
        f = layer.astype(np.float32)
        glow = cv2.resize(cv2.GaussianBlur(cv2.resize(f, None, fx=.25, fy=.25, interpolation=cv2.INTER_AREA), (0, 0), 10), (VW, VH), interpolation=cv2.INTER_LINEAR)
        m = cv2.GaussianBlur(np.clip(f.max(2, keepdims=True) / 90, 0, 1), (0, 0), .7)[..., None]
        return out * (1 - m) + f * m + glow * .5

class Landing:
    def __init__(self, frames):
        self.frames = frames
        x0, y0, x1, y1 = CARD
        self.w, self.h = x1 - x0, y1 - y0
        s = 4; r = 24 * s
        m = np.zeros((self.h * s, self.w * s), np.uint8)
        cv2.rectangle(m, (r, 0), (self.w * s - r, self.h * s), 255, -1)
        cv2.rectangle(m, (0, r), (self.w * s, self.h * s - r), 255, -1)
        for c in ((r, r), (self.w * s - r, r), (r, self.h * s - r), (self.w * s - r, self.h * s - r)):
            cv2.circle(m, c, r, 255, -1)
        self.mask = cv2.resize(m, (self.w, self.h), interpolation=cv2.INTER_AREA).astype(np.float32)[..., None] / 255
        e = cv2.morphologyEx((self.mask[..., 0] > .5).astype(np.uint8), cv2.MORPH_GRADIENT, np.ones((4, 4), np.uint8)).astype(np.float32)
        self.edge = cv2.GaussianBlur(e, (0, 0), .8)[..., None]
        y, x = np.mgrid[0:VH, 0:VW].astype(np.float32)
        self.back = BG + (np.clip(1 - (x / 1500 + y / 620), 0, 1) ** 1.5)[..., None] * np.array([235, 40, 95], np.float32)

    def frame(self, shift, film_t=None):
        out = self.back.copy()
        x0, y0 = CARD[0] + int(shift), CARD[1]
        im = np.full((self.h, self.w, 3), (22, 12, 10), np.float32)
        if film_t is not None:
            v = cv2.imread(f"{self.frames}/o{min(590 + int(film_t * 24), 1339):05d}.jpg")
            im = cv2.resize(v, (self.w, self.h), interpolation=cv2.INTER_AREA).astype(np.float32)
        im = im * (1 - self.edge) + np.array([248, 226, 196], np.float32) * self.edge
        x1 = min(VW, x0 + self.w)
        if x1 > x0:
            cut = x1 - x0
            a = self.mask[:, :cut]
            out[y0:y0 + self.h, x0:x1] = im[:, :cut] * a + out[y0:y0 + self.h, x0:x1] * (1 - a)
        return out

class Film:
    def __init__(self, kind, art, landing):
        self.kind, self.a, self.land = kind, art, landing
        self.cy = VH / 2
        self.end1 = {"A": 2.3, "B": 2.6, "C": 3.0}[kind]
        self.load_end = self.end1 + LOAD
        self.part2 = {"A": 2.7, "B": 2.7, "C": 2.0}[kind]
        self.t_card = self.load_end + self.part2
        self.t_film = self.t_card + .75 if kind != "C" else self.load_end + .5
        self.total = self.t_film + FILM_SECONDS
        nose = (self.a.mark_x + self.a.mw * .46, self.cy + 3)
        if kind == "A":
            self.streams = Streams(False, nose)
        if kind == "B":
            self.pour = Poured(art)

    # Part 1
    def splash(self, t):
        out = np.tile(BG, (VH, VW, 1)).copy()
        a, cy = self.a, self.cy
        if self.kind == "A":
            al = float(smooth(span(t, .2, .85)))
            un = float(smooth(span(t, .6, 1.3)))
            if t < 1.3:
                out = place(out, a.rib, VW / 2 + 58 * (1 - un), cy, a.lcx, a.lcy, BIG * (.9 + .1 * un) / OVER, None, un)
                return place(out, a.arr, VW / 2, cy, a.lcx, a.lcy, BIG * (.86 + .14 * al) / OVER, None, al)
            return self.lockup(out, t, 1.3)
        if self.kind == "B":
            return self.pour.text(self.pour.logo(out, t), t)
        # C: a line grows, pulls back into a point, the arrow opens from it and the ribbon slides out.
        grow = float(smooth(span(t, .2, 1.0)))
        back = float(smooth(span(t, 1.0, 1.75)))
        ax = VW / 2 - 40
        reach = 560 * grow * (1 - back)
        if back < 1:
            w = 8
            y0 = int(cy) - w // 2
            x1 = int(ax + reach)
            if x1 > ax:
                c = np.zeros((VH, VW, 3), np.float32)
                p = np.linspace(0, 1, x1 - int(ax))[None, :, None]
                c[y0:y0 + w, int(ax):x1] = np.array([255, 120, 90], np.float32) * (1 - p) + np.array([255, 150, 150], np.float32) * p
                out = out + c + cv2.GaussianBlur(c, (0, 0), 12) * 2.4
        tri = float(smooth(span(t, .25, .75)))
        o = float(smooth(span(t, 1.45, 2.05)))
        if o <= 0:
            s = .12 + .06 * tri
            return place(out, a.arr, ax, cy, a.lcx, a.lcy, s / OVER * 1.0, None, tri)
        s = .18 + (BIG - .18) * o
        out = place(out, a.rib, VW / 2 - 60 * (1 - o), cy, a.lcx, a.lcy, s / OVER, None, float(smooth(span(t, 1.6, 2.05))))
        out = place(out, a.arr, ax + 40 * o, cy, a.lcx, a.lcy, s / OVER)
        if t < 2.05:
            return out
        return self.lockup(np.tile(BG, (VH, VW, 1)).copy(), t, 2.05)

    def at_lockup(self, out, q):
        """q 0 is the mark alone at the centre, q 1 is mark and text at rest."""
        a = self.a
        cx = VW / 2 + (a.mark_x - VW / 2) * q
        s = BIG + (1 - BIG) * q
        if q > .002:
            out = a.put_word(out, a.wx + (cx - a.mark_x) - (a.ww + 60) * (1 - q), self.cy, 1.0, clip_left=cx)
        return a.mark(out, cx, self.cy, s)

    def lockup(self, out, t, t_in):
        return self.at_lockup(out, float(smooth(span(t, t_in, t_in + .9))))

    def waiting(self, t):
        out = self.at_lockup(np.tile(BG, (VH, VW, 1)).copy(), 1.0)
        al = float(smooth(span(t, self.end1, self.end1 + .35))) * (1 - float(smooth(span(t, self.load_end - .35, self.load_end))))
        return ring(out, t, al)

    # Part 2
    def secondary(self, t):
        if self.kind == "B":
            return self.pour.stage_two(t, t - self.load_end, self.part2)
        u = t - self.load_end
        a, cy = self.a, self.cy
        out = np.tile(BG, (VH, VW, 1)).copy()
        word_al = 1 - float(smooth(span(u, .15, .65)))
        if word_al > 0:
            out = a.put_word(out, a.wx, cy, 1.0, alpha=word_al)
        if self.kind == "A":
            go = span(u, .3, .95)
            al = 1 - float(smooth(span(u, .6, 1.05)))
            if u >= .25:
                out = self.streams.draw(out, u - .25, self.part2 - .25)
            if al > 0:
                out = a.mark(out, a.mark_x - 10 * float(smooth(span(u, 0, .3))) + 120 * rise(go, 2.4), cy, 1.0, sx=1 + .9 * rise(go, 2.2), alpha=al)
            return out
        # C: the film is seen through the arrow as it grows.
        out = place(out, a.rib, a.mark_x, cy, a.lcx, a.lcy, 1 / OVER, None, 1 - float(smooth(span(u, .1, .5))))
        p = span(u, .3, self.part2)
        s_ = (1 + 90 * float(smooth(p)) ** 2.7) / OVER
        cx = a.mark_x + (VW / 2 - a.mark_x) * float(smooth(span(p, 0, .55)))
        M = np.array([[s_, 0, cx - s_ * a.lcx], [0, s_, cy - s_ * a.lcy]], np.float64)
        sd = cv2.warpAffine(a.sdf, M, (VW, VH), flags=cv2.INTER_LINEAR, borderValue=-9999)
        al = np.clip(sd * s_ + .5, 0, 1)[..., None]
        col = cv2.warpAffine(a.arr_col, M, (VW, VH), flags=cv2.INTER_LINEAR, borderMode=cv2.BORDER_REPLICATE)
        see = float(smooth(span(p, .02, .36)))
        inside = col * (1 - see) + self.land.frame(0, max(0.0, t - self.t_film)) * see if see > 0 else col
        return out * (1 - al) + inside * al

    def frame(self, t):
        if t < self.end1:
            return self.splash(t)
        if t < self.load_end:
            return self.waiting(t)
        if t < self.t_card:
            return self.secondary(t)
        if self.kind == "C":
            return self.land.frame(0, t - self.t_film)
        shift = (1 - float(smooth(span(t, self.t_card, self.t_card + .6)))) * VW
        return self.land.frame(shift, None if t < self.t_film else t - self.t_film)

    def sample(self, t):
        if self.end1 <= t < self.load_end or t >= self.t_film:
            return self.frame(t)
        n = 3 if self.kind == "B" and t >= self.load_end else 1 if self.kind == "B" else 4
        if n == 1:
            return self.frame(t)
        return np.mean([self.frame(t + k / (FPS * n)) for k in range(n)], 0)

def sound(film, music, path):
    import soundfile as sf
    from scipy.signal import butter, sosfilt
    sr = 48000
    n = int(film.total * sr)
    x = np.zeros((n, 2))
    rng = np.random.default_rng(4)
    def at(t0, sig):
        i = int(t0 * sr); j = min(n, i + len(sig))
        x[i:j] += sig[: j - i, None]
    # Splash: two soft notes in D as the logo settles.
    L = int(2.4 * sr); tt = np.arange(L) / sr
    hit = {"A": 1.3, "B": 1.3, "C": 2.05}[film.kind]
    for f0, a, d in ((293.66, .2, 0), (440.0, .15, .1), (587.33, .1, .22)):
        at(hit + d, np.sin(2 * np.pi * f0 * tt) * np.exp(-tt / .7) * a * np.clip(tt / .03, 0, 1))
    # Secondary splash: pad and air swelling into the card.
    L = int(3.0 * sr); tt = np.arange(L) / sr
    env = np.clip(tt / 1.0, 0, 1) ** 2 * np.exp(-np.clip(tt - 1.0, 0, None) / .5)
    pad = sum(np.sin(2 * np.pi * f * tt) for f in (146.83, 220.0, 293.66, 440.0)) / 4
    air = sosfilt(butter(2, [600, 6000], "bp", fs=sr, output="sos"), rng.standard_normal(L))
    at(film.load_end, env * (pad * .3 + air * .06))
    m, _ = sf.read(music, always_2d=True)
    i = int(film.t_film * sr); k = min(n - i, len(m))
    x[i:i + k] += m[:k]
    tail = int(.8 * sr)
    x[-tail:] *= np.linspace(1, 0, tail)[:, None]
    x *= .89 / max(np.abs(x).max(), 1e-6)
    sf.write(path, x, sr, subtype="PCM_24")

FILM = None

def one(n):
    return np.clip(FILM.sample(n / FPS), 0, 255).astype(np.uint8).tobytes()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("kind", choices=list("ABC"))
    ap.add_argument("--word", required=True)
    ap.add_argument("--frames", required=True)
    ap.add_argument("--stills", default="")
    ap.add_argument("--still-file", default="")
    ap.add_argument("--out", default="")
    ap.add_argument("--music", default="")
    ap.add_argument("--ffmpeg", default="")
    a = ap.parse_args()
    film = Film(a.kind, Art(a.word), Landing(a.frames))
    if a.stills:
        shots = []
        for tx in a.stills.split(","):
            t = float(tx)
            im = cv2.resize(np.clip(film.sample(t), 0, 255).astype(np.uint8), (384, 216), interpolation=cv2.INTER_AREA)
            cv2.putText(im, f"{a.kind} {t:.2f}s", (6, 18), cv2.FONT_HERSHEY_SIMPLEX, .5, (0, 255, 255), 1)
            shots.append(im)
        rows = [np.hstack((shots[i:i + 5] + [np.zeros((216, 384, 3), np.uint8)] * 5)[:5]) for i in range(0, len(shots), 5)]
        cv2.imwrite(a.still_file, np.vstack(rows), [cv2.IMWRITE_JPEG_QUALITY, 90])
        print("parts", film.end1, film.load_end, film.t_card, film.t_film, film.total)
        return
    wav = a.out + ".wav"
    sound(film, a.music, wav)
    p = subprocess.Popen([a.ffmpeg, "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "bgr24", "-s", f"{VW}x{VH}", "-r", str(FPS), "-i", "-", "-i", wav,
                          "-c:v", "libx264", "-crf", "17", "-preset", "medium", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k", "-shortest", "-movflags", "+faststart", a.out], stdin=subprocess.PIPE)
    import multiprocessing as mp
    global FILM
    FILM = film
    cv2.setNumThreads(1)
    with mp.get_context("fork").Pool(10) as pool:
        for i, data in enumerate(pool.imap(one, range(int(film.total * FPS)), chunksize=6)):
            p.stdin.write(data)
            if i % 120 == 0:
                print("frame", i, flush=True)
    p.stdin.close(); p.wait()
    Path(wav).unlink()
    print("wrote", a.out)

main()
