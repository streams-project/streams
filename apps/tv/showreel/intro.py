#!/usr/bin/env python3
"""The Streams opening, direction A.

A line of light, the ribbon and arrow knock together, the word snaps out from behind the arrow,
the logo holds while the app loads, then the arrow fires and opens into horizontal streams.
Vertical stripes are the reference move; these run the way the arrow points.
"""
import argparse, io, math, re, subprocess
from pathlib import Path
import cv2, numpy as np, resvg_py
from PIL import Image

VW, VH, FPS = 1920, 1080, 60
ICON = "~/Desktop/streams-stream-play-icons/assets-logos/source-icon.svg"
BG = np.array([15, 7, 6], np.float32)          # BGR, near black with a little blue
LOCK_W = 760.0
RATIO = 3.922
OVER = 1.8                                     # layers are drawn larger than they appear

T_LINE = (0.12, 0.54)
T_FLY = (0.44, 0.70)
T_SNAP = (1.02, 1.17)
T_HOLD = 2.55
T_PULL = (2.55, 2.70)
T_COCK = (2.70, 2.82)
T_FIRE = (2.82, 3.08)
T_END = 3.72

def ease_out(x, p=4):
    x = np.clip(x, 0, 1)
    return 1 - (1 - x) ** p

def ease_in(x, p=3):
    return np.clip(x, 0, 1) ** p

def span(t, a, b):
    return float(np.clip((t - a) / (b - a), 0, 1))

def layers():
    s = Path(ICON).read_text()
    rib = re.search(r'<path d="[^"]+" fill="url\(#ig0\)"/>', s).group(0)
    arr = re.search(r'<path d="[^"]+" fill="url\(#ig1\)"/>', s).group(0)
    art = Image.open(io.BytesIO(bytes(resvg_py.svg_to_bytes(svg_string=s, width=1024, height=1024)))).convert("RGBA").getbbox()
    mh = LOCK_W / RATIO
    u = mh / (art[3] - art[1]) * OVER
    size = int(1024 * u)
    def draw(svg):
        png = bytes(resvg_py.svg_to_bytes(svg_string=svg, width=size, height=size))
        im = np.asarray(Image.open(io.BytesIO(png)).convert("RGBA"), np.float32)
        a = im[..., 3:4] / 255
        return np.dstack([im[..., 2:3] * a, im[..., 1:2] * a, im[..., 0:1] * a, im[..., 3:4]])
    return draw(s.replace(arr, "").split("<g opacity")[0] + "</g></svg>"), draw(s.replace(rib, "")), art, u, mh

def word_layer(path, height):
    png = bytes(resvg_py.svg_to_bytes(svg_path=path, height=int(height * 2)))
    im = np.asarray(Image.open(io.BytesIO(png)).convert("RGBA"), np.float32)
    a = im[..., 3:4] / 255
    return np.dstack([im[..., 2:3] * a, im[..., 1:2] * a, im[..., 0:1] * a, im[..., 3:4]])

def place(out, lay, cx, cy, lcx, lcy, sx, sy, alpha=1.0, clip_left=None):
    M = np.array([[sx, 0, cx - sx * lcx], [0, sy, cy - sy * lcy]], np.float64)
    w = cv2.warpAffine(lay, M, (VW, VH), flags=cv2.INTER_LINEAR)
    if clip_left is not None:
        w[:, : max(0, int(clip_left))] = 0
    a = w[..., 3:4] / 255 * alpha
    return out * (1 - a) + w[..., :3] * alpha

class Stripes:
    """Bands that leave the arrow's nose in a fan, fill the picture, then run off to the right."""
    PALETTE = [(255, 107, 47), (242, 63, 123), (224, 56, 67), (189, 75, 29), (255, 160, 110), (190, 163, 247), (255, 255, 255)]
    WEIGHT = [.32, .3, .14, .12, .06, .04, .02]

    def __init__(self):
        r = np.random.default_rng(21)
        n = 46
        self.y = np.sort(r.uniform(-30, VH + 30, n))
        self.th = r.choice([3, 5, 9, 16, 28, 46, 70], n, p=[.2, .2, .2, .16, .12, .08, .04]).astype(float)
        self.col = r.choice(len(self.PALETTE), n, p=self.WEIGHT)
        self.lag = r.uniform(0, 0.10, n)
        self.tail = r.uniform(0, 0.16, n)

    def draw(self, out, t, nx, ny):
        c = np.zeros((VH, VW, 3), np.uint8)
        t0, t1 = T_FIRE[0], T_FIRE[1]
        for i in range(len(self.y)):
            p = span(t, t0 + self.lag[i], t1 + self.lag[i])
            if p <= 0:
                continue
            head = nx + (VW + 900 - nx) * ease_in(p, 2.2)
            fan = float(ease_out(span(t, t0 + 0.02, t1 + 0.16), 3))
            y0 = ny + (self.y[i] - ny) * fan
            th = max(1.0, self.th[i] * fan)
            leave = span(t, 3.12 + self.tail[i], 3.56 + self.tail[i])
            tail = nx - 30 * fan - (nx + 200) * fan ** 3 + (VW + 1000) * ease_in(leave, 2.4)
            if tail >= VW:
                continue
            taper = th * 9 + 60
            lift = (y0 - ny) * 0.10
            pts = np.array([[tail, y0 + th / 2], [tail + taper, y0], [head - taper, y0 + lift], [head, y0 + lift + th / 2], [head - taper, y0 + lift + th], [tail + taper, y0 + th]], np.int32)
            cv2.fillConvexPoly(c, pts, self.PALETTE[self.col[i]], cv2.LINE_AA)
        f = c.astype(np.float32)
        glow = cv2.GaussianBlur(cv2.resize(f, None, fx=.25, fy=.25, interpolation=cv2.INTER_AREA), (0, 0), 9)
        glow = cv2.resize(glow, (VW, VH), interpolation=cv2.INTER_LINEAR)
        mask = (f.max(2, keepdims=True) > 0).astype(np.float32)
        return out * (1 - mask) + f + glow * 0.55

class Intro:
    def __init__(self, word_svg):
        self.rib, self.arr, self.art, self.u, self.mh = layers()
        l, t, r, b = self.art
        self.lcx, self.lcy = (l + r) / 2 * self.u, (t + b) / 2 * self.u
        self.mw = self.mh * (r - l) / (b - t)
        cap = self.mh / 1.42
        self.word = word_layer(word_svg, cap * 1.08)
        self.wx = (VW - LOCK_W) / 2 + self.mw + cap * 0.20      # final left of the word
        self.ww = self.word.shape[1] / 2
        self.mark_x = (VW - LOCK_W) / 2 + self.mw / 2            # final centre of the mark
        self.stripes = Stripes()
        self.base = 1 / OVER

    def line(self, out, t):
        a, b = T_LINE
        if not a <= t <= b + 0.1:
            return out
        head = -60 + (VW / 2 + 60) * float(ease_in(span(t, a, b), 2.0))
        tail = -60 + (VW / 2 + 60) * float(ease_in(span(t, a + 0.13, b + 0.09), 2.6))
        c = np.zeros((VH, VW, 3), np.float32)
        x0, x1 = int(tail), int(head)
        if x1 <= x0:
            return out
        xs = np.arange(max(0, x0), min(VW, x1))
        if len(xs) == 0:
            return out
        p = (xs - tail) / max(1, head - tail)
        col = np.stack([255 * np.ones_like(p), 70 + 60 * p, 120 - 80 * p], 1).astype(np.float32) * (0.35 + 0.65 * p)[:, None]
        c[VH // 2 - 2: VH // 2 + 3, xs] = col[None]
        glow = cv2.GaussianBlur(c, (0, 0), 7) * 3.0
        return out + c + glow

    def mark(self, out, t, cx, cy, sx, sy, gap=0.0, rib_alpha=1.0, arr_alpha=1.0):
        s = self.base
        out = place(out, self.rib, cx - gap, cy, self.lcx, self.lcy, sx * s * .985 if False else sx * s, sy * s, rib_alpha)
        return place(out, self.arr, cx + gap, cy, self.lcx, self.lcy, sx * s, sy * s, arr_alpha)

    def frame(self, t):
        out = np.tile(BG, (VH, VW, 1)).copy()
        out = self.line(out, t)
        cy = VH / 2
        big = 1.5
        if t < T_FLY[0]:
            return out
        if t < T_SNAP[0]:
            # Ribbon from the left, arrow from the right, one knock, one small bounce.
            hit = T_FLY[1]
            if t < hit:
                p = float(ease_in(span(t, *T_FLY), 2.2))
                gapl = 640 * (1 - p); alpha = min(1, span(t, T_FLY[0], T_FLY[0] + 0.08))
                out = place(out, self.rib, VW / 2 - gapl, cy, self.lcx, self.lcy, big / OVER, big / OVER, alpha)
                return place(out, self.arr, VW / 2 + gapl, cy, self.lcx, self.lcy, big / OVER, big / OVER, alpha)
            d = t - hit
            bounce = 30 * math.exp(-d / 0.075) * math.sin(2 * math.pi * d / 0.21)
            pulse = 1 + 0.07 * math.exp(-d / 0.05)
            flash = math.exp(-d / 0.09)
            if flash > 0.02:
                r = np.hypot(*np.meshgrid(np.arange(VW) - VW / 2, np.arange(VH) - VH / 2))
                out += (np.exp(-(r / 300) ** 2) * flash * 0.5)[..., None] * np.array([255, 120, 90], np.float32)
            return self.mark(out, t, VW / 2, cy, big * pulse, big * pulse, gap=bounce)
        # Where the mark sits, from the moment it snaps to the moment it fires.
        snap = float(ease_out(span(t, *T_SNAP), 5))
        pull = float(ease_in(span(t, *T_PULL), 2.4))
        k = snap * (1 - pull)
        cx = VW / 2 + (self.mark_x - VW / 2) * k
        scale = big + (1 - big) * k
        wx = self.wx + (cx - self.mark_x) - (self.ww + 80) * (1 - k)
        drift = 0.0
        if T_SNAP[1] < t < T_HOLD:
            q = t - T_SNAP[1]
            drift = 4 * math.sin(2 * math.pi * q / 1.3) * min(1, q / 0.3)
        if t < T_FIRE[0] + 0.14:
            if 0.002 < k:
                out = place(out, self.word, wx, cy, 0, self.word.shape[0] / 2, .5, .5, 1.0, clip_left=cx)
        if t < T_PULL[1] + 0.001:
            return self.mark(out, t, cx, cy, scale, scale, gap=drift)
        if t < T_COCK[1]:
            q = float(ease_out(span(t, *T_COCK), 2))
            return self.mark(out, t, VW / 2 - 46 * q, cy, big * (1 - 0.08 * q), big * (1 + 0.05 * q))
        f = span(t, T_COCK[1], T_FIRE[1])
        nx = VW / 2 - 46 + 500 * float(ease_in(f, 2)) + 150
        if t < 3.2:
            out = self.stripes.draw(out, t, VW / 2 + 90, cy - 4)
            a = 1 - span(t, T_FIRE[0], T_FIRE[0] + 0.2)
            if a > 0:
                out = self.mark(out, t, VW / 2 - 46 + 1100 * float(ease_in(f, 2.2)), cy, big * (1 + 1.6 * float(ease_in(f, 2))), big * (1 - 0.45 * f), rib_alpha=a, arr_alpha=a)
            return out
        return self.stripes.draw(out, t, VW / 2 + 90, cy - 4)

    def sample(self, t):
        fast = T_LINE[0] < t < T_LINE[1] + .1 or T_FLY[0] < t < T_FLY[1] + .05 or T_SNAP[0] < t < T_SNAP[0] + .1 or T_PULL[0] < t < 3.6
        if not fast:
            return self.frame(t)
        n = 10 if T_FIRE[0] - 0.02 < t < 3.6 else 4
        return np.mean([self.frame(t + k / (FPS * n)) for k in range(n)], 0)

CARD = (55, 169, 1596, 1033)
SLIDE = [0, .647, 1, .323, 2, .106, 3, .077, 4, .039, 5, .017, 6, .005, 7, .001, 8, 0]
T_CARD = T_END
T_WALL = T_CARD + 16 / 24
T_TOTAL = T_WALL + 4.4

class Handover:
    """The card arrives and the showreel starts inside it. No buttons here, this only shows the join."""
    def __init__(self, frames):
        self.frames = frames
        x0, y0, x1, y1 = CARD
        self.w, self.h = x1 - x0, y1 - y0
        s = 4
        m = np.zeros((self.h * s, self.w * s), np.uint8)
        r = 24 * s
        cv2.rectangle(m, (r, 0), (self.w * s - r, self.h * s), 255, -1)
        cv2.rectangle(m, (0, r), (self.w * s, self.h * s - r), 255, -1)
        for c in ((r, r), (self.w * s - r, r), (r, self.h * s - r), (self.w * s - r, self.h * s - r)):
            cv2.circle(m, c, r, 255, -1)
        self.mask = cv2.resize(m, (self.w, self.h), interpolation=cv2.INTER_AREA).astype(np.float32)[..., None] / 255
        edge = cv2.morphologyEx((self.mask[..., 0] > .5).astype(np.uint8), cv2.MORPH_GRADIENT, np.ones((4, 4), np.uint8)).astype(np.float32)
        self.edge = cv2.GaussianBlur(edge, (0, 0), .8)[..., None]
        y, x = np.mgrid[0:VH, 0:VW].astype(np.float32)
        glow = np.clip(1 - (x / 1500 + y / 620), 0, 1) ** 1.5
        self.back = BG + glow[..., None] * np.array([235, 40, 95], np.float32)

    def frame(self, t):
        out = self.back.copy()
        f = (t - T_CARD) * 24
        shift = int(table_f(SLIDE, f) * VW)
        x0, y0 = CARD[0] + shift, CARD[1]
        im = np.full((self.h, self.w, 3), (22, 12, 10), np.float32)
        if t >= T_WALL:
            n = 590 + int((t - T_WALL) * 24)
            v = cv2.imread(f"{self.frames}/o{min(n, 1339):05d}.jpg")
            im = cv2.resize(v, (self.w, self.h), interpolation=cv2.INTER_AREA).astype(np.float32)
        im = im * (1 - self.edge) + np.array([248, 226, 196], np.float32) * self.edge
        x1 = min(VW, x0 + self.w)
        if x1 > x0:
            cut = x1 - x0
            a = self.mask[:, :cut]
            out[y0:y0 + self.h, x0:x1] = im[:, :cut] * a + out[y0:y0 + self.h, x0:x1] * (1 - a)
        return out

def table_f(pairs, x):
    return float(np.interp(x, pairs[0::2], pairs[1::2]))

def sound(music, path):
    import soundfile as sf
    from scipy.signal import butter, sosfilt
    sr = 48000
    n = int(T_TOTAL * sr)
    x = np.zeros((n, 2))
    time = np.arange(n) / sr
    rng = np.random.default_rng(4)
    def at(t0, sig, pan=0.0):
        i = int(t0 * sr)
        j = min(n, i + len(sig))
        x[i:j, 0] += sig[: j - i] * (1 - pan)
        x[i:j, 1] += sig[: j - i] * (1 + pan)
    # Line: air rising.
    L = int(.5 * sr)
    w = sosfilt(butter(2, [500, 5200], "bp", fs=sr, output="sos"), rng.standard_normal(L))
    at(.1, w * np.linspace(0, 1, L) ** 2 * .05)
    # Knock: a soft low hit with a small tick on the front.
    L = int(.7 * sr); tt = np.arange(L) / sr
    at(T_FLY[1], np.sin(2 * np.pi * (58 * tt + 26 * (1 - np.exp(-tt / .03)))) * np.exp(-tt / .13) * .62)
    at(T_FLY[1], sosfilt(butter(2, 1800, "hp", fs=sr, output="sos"), rng.standard_normal(L)) * np.exp(-tt / .006) * .1)
    # Snap: two bell notes in D, the key of the music.
    L = int(1.6 * sr); tt = np.arange(L) / sr
    for f0, a, d in ((587.33, .17, 0), (880.0, .13, .05)):
        at(T_SNAP[0] + d, (np.sin(2 * np.pi * f0 * tt) + .3 * np.sin(4 * np.pi * f0 * tt)) * np.exp(-tt / .42) * a * np.clip(tt / .004, 0, 1))
    # Fire: a pad and air swelling, peaking as the streams fill the picture.
    L = int(1.5 * sr); tt = np.arange(L) / sr
    env = np.clip(tt / .55, 0, 1) ** 2.2 * np.exp(-np.clip(tt - .55, 0, None) / .3)
    pad = sum(np.sin(2 * np.pi * f * tt * (1 + .03 * tt)) for f in (146.83, 220.0, 293.66, 440.0)) / 4
    air = sosfilt(butter(2, [700, 7000], "bp", fs=sr, output="sos"), rng.standard_normal(L))
    at(T_PULL[0], env * (pad * .3 + air * .1), .0)
    # The music starts with the film in the card.
    m, _ = sf.read(music, always_2d=True)
    i = int(T_WALL * sr)
    k = min(n - i, len(m))
    x[i:i + k] += m[:k]
    tail = int(.8 * sr)
    x[-tail:] *= np.linspace(1, 0, tail)[:, None]
    x *= .89 / max(np.abs(x).max(), 1e-6)
    sf.write(path, x, sr, subtype="PCM_24")

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--word", required=True)
    ap.add_argument("--stills", default="")
    ap.add_argument("--still-dir", default=".")
    ap.add_argument("--out", default="")
    ap.add_argument("--ffmpeg", default="")
    ap.add_argument("--frames", default="")
    ap.add_argument("--music", default="")
    a = ap.parse_args()
    film = Intro(a.word)
    hand = Handover(a.frames) if a.frames else None
    if a.stills:
        rows = []
        shots = []
        for t in [float(x) for x in a.stills.split(",")]:
            im = cv2.resize(np.clip(hand.frame(t) if hand and t >= T_CARD else film.sample(t), 0, 255).astype(np.uint8), (480, 270), interpolation=cv2.INTER_AREA)
            cv2.putText(im, f"{t:.2f}s", (6, 18), cv2.FONT_HERSHEY_SIMPLEX, .5, (0, 255, 255), 1)
            shots.append(im)
        for i in range(0, len(shots), 4):
            r = shots[i:i + 4] + [np.zeros((270, 480, 3), np.uint8)] * (4 - len(shots[i:i + 4]))
            rows.append(np.hstack(r))
        cv2.imwrite(f"{a.still_dir}/intro-strip.jpg", np.vstack(rows), [cv2.IMWRITE_JPEG_QUALITY, 90])
        return
    wav = a.out + ".wav"
    sound(a.music, wav)
    p = subprocess.Popen([a.ffmpeg, "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "bgr24", "-s", f"{VW}x{VH}", "-r", str(FPS), "-i", "-", "-i", wav,
                          "-c:v", "libx264", "-crf", "17", "-preset", "medium", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k", "-shortest", "-movflags", "+faststart", a.out], stdin=subprocess.PIPE)
    for n in range(int(T_TOTAL * FPS)):
        t = n / FPS
        p.stdin.write(np.clip(hand.frame(t) if t >= T_CARD else film.sample(t), 0, 255).astype(np.uint8).tobytes())
    p.stdin.close(); p.wait()
    Path(wav).unlink()
    print("wrote", a.out)

main()
