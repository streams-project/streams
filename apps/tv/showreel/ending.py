#!/usr/bin/env python3
"""The end of the showreel, after the last headline. A frame number may be a fraction.

Change 14: each screen has its own logic, its own content and its own way of naming its platforms.
  television  grows around the card that came from the last headline, names roll beneath it
  laptop      rises closed, the lid opens, the systems line up beside it
  tablet      arrives upright and turns on its side, a shelf of posters runs across it
  phones      one from each side, each scrolling its own feed
  browser     a window opens, a pointer presses play
Change 15: the television comes back and fills the picture, its progress bar drops to become the
floor, the names land on it row by row into one ordered wall and stand still while the bar fills.
Then the bar runs out from under them, left to right, and they peel off as chains of letters. The
ask follows, a word at a time, and the mark with the name sliding out from behind it.
Frames here run on the ending clock, the film's raw frame minus OPEN in scenes.py. Parts start on bar
lines of the music. film.py renders this and scenes.py as one film.
"""
import math
import cv2, numpy as np, pymunk
from PIL import Image, ImageDraw
from render import backdrop, foot, smooth
from stage import (CX, CY, VH, VW, WHITE, Card, clamp, cover, ease, fall, glow, lay, lay_card, lockup, rise, rounded,
                   soft_shadow, split_lockup, spring, word)

N0 = 1345
RETURN, REL, ASK, MARK, N_END = 1596, 1736, 1765, 1806, 1876
BACK = RETURN - 10
HOLD_LINE, ALL_STILL = 1666, 1672  # the last name lands, and every name is at rest
CLOSE_LEAVE, CLOSE_END = 1868, 1875
STEP = 0.4  # film frames between drawn frames, 60 a second
TV_Y = 490.0
BLUE, SALMON = (255, 120, 60), (170, 150, 250)
SHELL, EDGE = (38, 30, 28), (120, 100, 92)
DECK_H = 96  # how far the laptop's deck reaches toward us, in pixels
TV_NAMES = ["Apple TV", "Samsung", "LG", "Fire TV", "Roku", "Android TV"]

# The wall of names. Each row is justified, ink to ink, from x 100 to x 1820. Sizes and the numbers
# below come from the spec in notes/raw/169-names-wall. Stack checks its own solve against them.
LINES = [(("Apple TV", "Samsung", "LG"), 324.5, 140), (("Fire TV", "Roku", "Android TV"), 437.5, 124),
         (("Mac", "Windows", "Linux"), 566.5, 146), (("iPad", "Android tablets"), 701.5, 141),
         (("iPhone", "Android", "Any browser"), 820.5, 102)]
WALL_LEFT, WALL_RIGHT, WALL_BASE = 100, 1820, 858
ROW_GAPS = (16, 30, 30, 30)  # ink between rows, tighter inside the television family
CAPS = ((273, 376), (392, 483), (513, 620), (650, 753), (783, 858))
BANDS = ((265, 384), (384, 498), (498, 635), (635, 768), (768, 858))  # the hull of each row
# A name's rest: centre x, ink left, ink right.
REST = {"Apple TV": (435.0, 100, 770), "Samsung": (1211.0, 853, 1568), "LG": (1734.5, 1651, 1820),
        "Fire TV": (324.0, 100, 554), "Roku": (802.0, 632, 973), "Android TV": (1435.0, 1050, 1820),
        "Mac": (268.5, 100, 441), "Windows": (914.5, 535, 1293), "Linux": (1600.0, 1386, 1820),
        "iPad": (260.0, 100, 423), "Android tablets": (1166.5, 510, 1820),
        "iPhone": (285.5, 100, 474), "Android": (769.0, 536, 1000), "Any browser": (1440.5, 1063, 1820)}
ROW_HITS = (1666, 1657.25, 1648.5, 1639.75, 1631)  # the last name of a row lands on these half beats
HIT_STAGGER = 2
TILT = {"iPhone": 3, "Android": -2.5, "Any browser": 2, "iPad": -3, "Android tablets": 2, "Mac": 3, "Windows": -2.5,
        "Linux": 3, "Fire TV": -3, "Roku": 3, "Android TV": -2, "Apple TV": 2.5, "Samsung": -2.5, "LG": 4}
FALL = 6.34  # px per frame per frame
TILT_FLATTEN, REBOUND_PX, REBOUND_TAU, REBOUND_WAVE, REBOUND_END = 6, 11, 1.6, 4.5, 6
PRESS_PX, PRESS_TAU, PRESS_FRAMES = 3.5, 1.5, 3
SMEAR_MIN, SMEAR_MOST = 7, 8

# The film's bar, which is the floor and the clock of the wall.
BAR_LEFT, BAR_RIGHT, BAR_Y, BAR_H = 54, 1866, 864, 12
BAR_START = .114
BAR_LIFT, BAR_LIFT_FRAMES = 1600, 22
CARD_END = VW * 1.02 / 960  # the returning television's scale once it fills the picture
ART_PUSH, ART_PUSH_START = .0004, 1634
SHADOW_Y, SHADOW_ALPHA, SHADOW_FADE = 6, .55, 14
HEAD_Y = 104

# The release.
GRAVITY, ITERATIONS, SUBSTEPS, PRE_ROLL = 2400, 30, 4, 240
RETREAT_FRAMES, RETREAT_TO, FLOOR_FAR, ROUND = 34, 2400, 2800, 6
SPRING_HZ, SPRING_ZETA = 4, .7
RESIDUAL_BLEND = 8
TRACK_END = 1812

def metal(w, h, top=1.3, low=.6, seed=7):
    """Dark brushed metal lit from above, as a BGR picture."""
    ramp = np.linspace(top, low, h, dtype=np.float32)[:, None]
    grain = np.random.default_rng(seed).normal(0, 1, (h, w)).astype(np.float32)
    grain = cv2.GaussianBlur(grain, (0, 0), sigmaX=24, sigmaY=.7) * 14
    return np.dstack([np.clip(c * ramp + grain * .5, 0, 255) for c in SHELL])

def body(w, h, pad, radius, top=0):
    """A device body: brushed dark metal, and a thin rim that is bright on top and fades downward."""
    W, H = w + 2 * pad, h + 2 * pad + top
    outer = rounded(W, H, radius)
    inner = np.zeros_like(outer)
    inner[2:-2, 2:-2] = rounded(W - 4, H - 4, radius - 2)
    rim = np.clip(outer - inner, 0, 1) * np.linspace(1, .3, H, dtype=np.float32)[:, None]
    rgb = metal(W, H)
    pic = np.zeros((H, W, 4), np.float32)
    for k in range(3):
        pic[..., k] = outer * rgb[..., k] + rim * (EDGE[k] * 1.6 - rgb[..., k])
    pic[..., 3] = outer * 255
    return pic

def paste(rgb, pic, x, y, alpha=1.0):
    """A premultiplied picture onto a BGR picture at a whole pixel, clipped to it."""
    h, w = pic.shape[:2]
    x0, y0, x1, y1 = max(x, 0), max(y, 0), min(x + w, rgb.shape[1]), min(y + h, rgb.shape[0])
    if x1 <= x0 or y1 <= y0:
        return
    p = pic[y0 - y:y1 - y, x0 - x:x1 - x]
    rgb[y0:y1, x0:x1, :3] = rgb[y0:y1, x0:x1, :3] * (1 - p[..., 3:4] * alpha / 255) + p[..., :3] * alpha

def paste_pic(pic, top, x, y):
    """The same onto a premultiplied picture whose alpha stays as it was."""
    paste(pic, top, x, y)

def sized(pic, h):
    """A picture scaled to a height."""
    return cv2.resize(pic, (max(round(pic.shape[1] * h / pic.shape[0]), 1), h), interpolation=cv2.INTER_AREA)

def capsule(w, h, colour, radius=None):
    """A filled rounded shape, a full pill unless a radius is given."""
    m = rounded(w, h, h // 2 if radius is None else radius)
    return np.dstack([m * c for c in colour] + [m * 255]).astype(np.float32)

def shade(w, h, strength=.6):
    """A soft dark pool, for what a device casts."""
    yy, xx = np.mgrid[-1:1:h * 1j, -1:1:w * 1j]
    a = np.exp(-(xx ** 2) * 3 - (yy ** 2) * 5).astype(np.float32) * strength
    return np.dstack([np.zeros_like(a)] * 3 + [a * 255])

def mirror(pic, depth, fade=.16):
    """A faint reflection of a picture's lowest rows, which dies away downward."""
    return pic[-depth:][::-1] * np.linspace(fade, 0, depth, dtype=np.float32)[:, None, None]

class Sheen:
    """A soft band of light that crosses a glass screen on a slant."""
    def __init__(self, h, w):
        yy, xx = np.mgrid[0:h, 0:w]
        self.along, self.span = (xx + yy * .55).astype(np.float32), w + h * .55

    def add(self, rgb, t, strength=12.0):
        spot = (-.2 + 1.4 * (t % 1)) * self.span
        rgb += (np.exp(-((self.along - spot) / (.14 * self.span)) ** 2)[..., None] * strength).astype(np.float32)

def pedestal(pic, W, body_h):
    """The stand of a television under its body: a short neck and a slim oval foot."""
    H = body_h + 44
    out = np.zeros((H, W, 4), np.float32)
    out[:body_h] = pic
    im = Image.new("L", (W * 2, H * 2), 0)
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((W * .92, body_h * 2 - 4, W * 1.08, (body_h + 26) * 2), 8, fill=255)
    d.rounded_rectangle((W * .62, (body_h + 24) * 2, W * 1.38, (body_h + 40) * 2), 16, fill=255)
    a = np.asarray(im.resize((W, H), Image.LANCZOS), np.float32) / 255
    rgb = metal(W, H, top=1.6, low=1.6, seed=5)
    light = np.ones_like(a)
    light[body_h + 24:body_h + 28] = 2.0
    keep = out[..., 3:4] / 255
    out[..., :3] += (1 - keep) * rgb * (a * light)[..., None]
    out[..., 3] = np.maximum(out[..., 3], a * 255)
    return out

def deck(W):
    """The lower half of a laptop as seen from in front and above: key rows, a trackpad, the hinge and
    a front edge. Drawn flat, then laid back in perspective once, because only its place changes."""
    w, h = W, 300
    rgb = metal(w, h, top=.95, low=1.5, seed=3)
    keys = Image.new("L", (w * 2, h * 2), 0)
    d = ImageDraw.Draw(keys)
    left, right, y, kh, gap = w * .11, w * 1.89, h * .16, h * .16, h * .034
    for r in range(5):
        n = 14 - r % 2
        kw = (right - left - (n - 1) * 8) / n
        for c in range(n):
            x = left + c * (kw + 8)
            d.rounded_rectangle((x, y, x + kw, y + kh), 8, fill=255)
        y += kh + gap
    d.rounded_rectangle((w * .6, y, w * 1.4, y + kh), 8, fill=255)
    a = np.asarray(keys.resize((w, h), Image.LANCZOS), np.float32)[..., None] / 255
    rgb = rgb * (1 - a) + np.array((16, 12, 12), np.float32) * a
    pad = np.zeros((h, w), np.float32)
    pad[int(h * .72):int(h * .975), int(w * .33):int(w * .67)] = rounded(int(w * .67) - int(w * .33), int(h * .975) - int(h * .72), 14)
    rgb += pad[..., None] * 16
    rgb[:10] *= .35
    src = np.float32([[0, 0], [w, 0], [w, h], [0, h]])
    wide = int(w * 1.2)
    top = (wide - w) / 2
    dst = np.float32([[top, 0], [top + w, 0], [wide, DECK_H], [0, DECK_H]])
    pic = cv2.warpPerspective(np.dstack([rgb, np.full((h, w), 255, np.float32)]), cv2.getPerspectiveTransform(src, dst),
                              (wide, DECK_H + 12), flags=cv2.INTER_AREA)
    k = pic[..., 3:4] / 255
    pic[..., :3] *= k
    lip = np.zeros((12, wide, 4), np.float32)
    m = rounded(wide, 12, 6)
    lip[..., :3], lip[..., 3] = (m[..., None] * np.array(EDGE, np.float32) * np.linspace(1.1, .5, 12, dtype=np.float32)[:, None, None]), m * 255
    pic[DECK_H:] = lip
    return pic

def inset(pic, screen, pad, top, radius):
    """A screen laid into a body."""
    h, w = screen.shape[:2]
    m = rounded(w, h, radius)[..., None]
    out = pic.copy()
    out[pad + top:pad + top + h, pad:pad + w, :3] = out[pad + top:pad + top + h, pad:pad + w, :3] * (1 - m) + screen * m
    return out

def ink_box(pic):
    """Left, right, top and bottom of a word's ink inside its picture."""
    a = pic[..., 3] > 8
    xs, ys = np.where(a.any(0))[0], np.where(a.any(1))[0]
    return int(xs[0]), int(xs[-1]) + 1, int(ys[0]), int(ys[-1]) + 1

def snap(c, size):
    """A centre that starts the picture on a whole pixel, so no row looks softer than another."""
    return round(c - size / 2) + size / 2

def letters_of(pic):
    """Each letter of a word as its own sprite, cut from the same picture so it sits on the same pixels."""
    count, labels, boxes, _ = cv2.connectedComponentsWithStats((pic[..., 3] > 0).astype(np.uint8), connectivity=8)
    parts = sorted(tuple(int(v) for v in boxes[j][:4]) + (j,) for j in range(1, count) if boxes[j][4] >= 6)
    out = []
    for x, y, w, h, j in parts:
        sprite = np.zeros_like(pic)
        sprite[labels == j] = pic[labels == j]
        sprite = sprite[y:y + h, x:x + w].copy()
        out.append(dict(pic=sprite, soft=soft_shadow(sprite)[..., 3] / 255, w=w, h=h,
                        dx=x + w / 2 - pic.shape[1] / 2, dy=y + h / 2 - pic.shape[0] / 2))
    return out

def stamp(plane, soft, x, y, deg=0.0):
    """Add a soft shape to a plane of shadow, so at rest the letters add up to exactly their word."""
    h, w = soft.shape
    c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
    rx, ry = (w * abs(c) + h * abs(s)) / 2 + 3, (w * abs(s) + h * abs(c)) / 2 + 3
    x0, y0, x1, y1 = max(int(x - rx), 0), max(int(y - ry), 0), min(int(x + rx), VW), min(int(y + ry), VH)
    if x1 <= x0 or y1 <= y0:
        return
    M = np.array([[c, -s, x - x0 - c * w / 2 + s * h / 2], [s, c, y - y0 - s * w / 2 - c * h / 2]], np.float64)
    plane[y0:y1, x0:x1] += cv2.warpAffine(soft, M, (x1 - x0, y1 - y0), flags=cv2.INTER_LINEAR)

def pill(out, x0, x1, y, h, colour, alpha=1.0):
    """A rounded bar. Its edge is worked out per pixel, so it can move by less than a pixel."""
    r = h / 2
    left, right = max(int(x0) - 1, 0), min(int(x1) + 2, VW)
    top, bottom = max(int(y - r) - 1, 0), min(int(y + r) + 2, VH)
    if right <= left or bottom <= top or x1 - x0 < h:
        return
    xs = np.arange(left, right, dtype=np.float32) + .5
    ys = np.arange(top, bottom, dtype=np.float32) + .5
    dx = np.maximum(np.maximum(x0 + r - xs, xs - (x1 - r)), 0)[None, :]
    k = (np.clip(r + .5 - np.hypot(dx, (ys - y)[:, None]), 0, 1) * alpha)[..., None]
    out[top:bottom, left:right] = out[top:bottom, left:right] * (1 - k) + np.array(colour, np.float32) * k

def floor_left(n):
    """The left end of the bar and of the floor under it, once it starts to run out."""
    return BAR_LEFT + (RETREAT_TO - BAR_LEFT) * ease((n - REL) / RETREAT_FRAMES)

class Stack:
    """The 14 names. Scripted poses until the release, then the physics track, in one set of pictures."""
    def __init__(self, font):
        self.font = font
        self.solve()
        self.hits = sorted(t["hit"] for t in self.names)
        self.build_still()
        self.simulate()

    def solve(self):
        """Lay out the wall from the ink, and stop the build if a font or Pillow change moved it."""
        tops, bottom, caps = [], float(WALL_BASE), []
        for i in range(4, -1, -1):
            _, _, a, b = ink_box(word(self.font, "H", LINES[i][2]))
            tops.insert(0, (a, bottom - (b - a), bottom))
            if i:
                bottom = bottom - (b - a) - ROW_GAPS[i - 1]
        self.names, self.letters = [], []
        for r, (names, y, size) in enumerate(LINES):
            lift, top, base = tops[r]
            pics = [word(self.font, name, size) for name in names]
            inks = [ink_box(p) for p in pics]
            gap = (WALL_RIGHT - WALL_LEFT - sum(i[1] - i[0] for i in inks)) / (len(names) - 1)
            x = float(WALL_LEFT)
            band = BANDS[r]
            for k, (name, pic, ink) in enumerate(zip(names, pics, inks)):
                h, w = pic.shape[:2]
                cx, cy = snap(x - ink[0] + w / 2, w), snap(top - lift + h / 2, h)
                t = dict(name=name, pic=pic, soft=soft_shadow(pic)[..., 3] / 255, cx=cx, cy=cy, w=w, h=h, row=r, size=size,
                         ink=(cx - w / 2 + ink[0], cx - w / 2 + ink[1]), band=band, cap=(top, base), letters=letters_of(pic),
                         hit=ROW_HITS[r] - HIT_STAGGER * (len(names) - 1 - k), tilt=TILT[name], first=len(self.letters))
                self.names.append(t)
                self.letters += t["letters"]
                x += ink[1] - ink[0] + gap
            # The row must match the table to the pixel.
            if (top, base) != CAPS[r]:
                raise RuntimeError(f"row {r + 1} cap band {(top, base)} is not {CAPS[r]}")
        for t in self.names:
            want = (t["cx"], t["ink"][0], t["ink"][1])
            if want != REST[t["name"]] or t["cy"] != LINES[t["row"]][1]:
                raise RuntimeError(f"{t['name']} rests at {want}, {t['cy']}, not {REST[t['name']]}")
            if len(t["letters"]) != len(t["name"].replace(" ", "")):
                raise RuntimeError(f"{t['name']} cut into {len(t['letters'])} letters")

    def build_still(self):
        """One layer of the whole wall with its shadows, drawn the same way as every other frame."""
        plane = np.zeros((VH, VW), np.float32)
        ink = np.zeros((VH, VW, 3), np.float32)
        keep = np.ones((VH, VW, 3), np.float32)
        for t in self.names:
            stamp(plane, t["soft"], t["cx"], t["cy"] + SHADOW_Y)
        keep *= (1 - SHADOW_ALPHA * np.clip(plane, 0, 1))[..., None]
        for t in self.names:
            place = [(t["cx"], t["cy"], 1.0, 1.0, 0.0)]
            lay(ink, t["pic"], place)
            lay(keep, t["pic"], place, light=0.0)
        rows, cols = np.where(np.abs(keep - 1).max(2) > 1e-4)
        self.box = int(cols.min()), int(rows.min()), int(cols.max()) + 1, int(rows.max()) + 1
        x0, y0, x1, y1 = self.box
        self.still_keep, self.still_ink = keep[y0:y1, x0:x1].copy(), ink[y0:y1, x0:x1].copy()

    def simulate(self):
        """Rigid names settle, swap to chains of letters in the same space, and the floor runs out from under them."""
        space = pymunk.Space()
        space.gravity = (0, GRAVITY)
        space.iterations = ITERATIONS
        dt = STEP / 24 / SUBSTEPS
        floor = pymunk.Body(body_type=pymunk.Body.KINEMATIC)
        seg = pymunk.Segment(floor, (BAR_LEFT, BAR_Y), (FLOOR_FAR, BAR_Y), ROUND)
        seg.friction, seg.elasticity = .8, .02
        space.add(floor, seg)
        boxes = []
        for t in self.names:
            w, h = t["ink"][1] - t["ink"][0], t["band"][1] - t["band"][0]
            mass = w * h / 8000
            body = pymunk.Body(mass, pymunk.moment_for_box(mass, (w, h)))
            body.position = (t["ink"][0] + t["ink"][1]) / 2, (t["band"][0] + t["band"][1]) / 2
            hw, hh = w / 2 - ROUND, h / 2 - ROUND
            shape = pymunk.Poly(body, [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)], radius=ROUND)
            shape.friction, shape.elasticity = .8, .05
            space.add(body, shape)
            boxes.append((body, shape, tuple(body.position)))
        for _ in range(PRE_ROLL):
            space.step(dt)
        self.settle = max(max(abs(b.position.x - h[0]), abs(b.position.y - h[1])) for b, _, h in boxes)
        chains = []
        reach = 2 * math.pi * SPRING_HZ
        for i, (t, (box, shape, home)) in enumerate(zip(self.names, boxes)):
            space.remove(box, shape)
            ox, oy = box.position.x - home[0], box.position.y - home[1]
            h, centre = t["band"][1] - t["band"][0], (t["band"][0] + t["band"][1]) / 2
            bodies = []
            for l in t["letters"]:
                w = l["w"]
                mass = max(w, 8) * h / 8000
                body = pymunk.Body(mass, pymunk.moment_for_box(mass, (w, h)))
                body.position = t["cx"] + l["dx"] + ox, centre + oy
                hw, hh = max(w / 2 - ROUND, 1), h / 2 - ROUND
                part = pymunk.Poly(body, [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)], radius=ROUND)
                part.friction, part.elasticity = .08, .05
                part.filter = pymunk.ShapeFilter(group=i + 1)
                space.add(body, part)
                bodies.append(body)
                l["oy"] = t["cy"] + l["dy"] - centre
                l["rest"] = t["cx"] + l["dx"], centre
                l["reach"] = math.hypot(l["w"], l["h"]) / 2 + abs(l["oy"]) + 8
            for a, b, la, lb in zip(bodies, bodies[1:], t["letters"], t["letters"][1:]):
                # Neighbours join in the middle of the gap between them, across a space too.
                mid = t["cx"] + (la["dx"] + la["w"] / 2 + lb["dx"] - lb["w"] / 2) / 2
                pin = pymunk.PivotJoint(a, b, (mid + ox, centre + oy))
                moment = (a.moment + b.moment) / 2
                twist = pymunk.DampedRotarySpring(a, b, 0, moment * reach * reach, 2 * SPRING_ZETA * moment * reach)
                pin.collide_bodies = twist.collide_bodies = False
                space.add(pin, twist)
            chains.append(bodies)
        seg.friction = .08
        bodies = [b for chain in chains for b in chain]
        # The first step is the solver settling the swap, about .13 px. The track starts after it,
        # at REL + STEP, and the residual blend takes that up with the rest of the settling.
        track = []
        for step in range(round((TRACK_END - REL) / STEP)):
            for k in range(SUBSTEPS):
                seg.unsafe_set_endpoints((floor_left(REL + (step + (k + 1) / SUBSTEPS) * STEP), BAR_Y), (FLOOR_FAR, BAR_Y))
                space.reindex_shape(seg)
                space.step(dt)
            track.append([(b.position.x, b.position.y, b.angle) for b in bodies])
        self.track = np.array(track, np.float64)
        rest = np.array([(l["rest"][0], l["rest"][1], 0.0) for l in self.letters], np.float64)
        self.residual = rest - self.track[0]

    def press(self, n, after=0.0):
        """The quick dip a landing sends through everything beneath it."""
        total = 0.0
        for h in self.hits:
            v = n - h
            if h > after and 0 <= v < PRESS_FRAMES:
                total += PRESS_PX * math.exp(-v / PRESS_TAU) * math.sin(math.pi * v / PRESS_FRAMES)
        return total

    def scripted(self, t, n):
        """A name's y and tilt before the release: a straight fall, one small rebound, then rest."""
        before = t["hit"] - n
        if before > 0:
            return t["cy"] - .5 * FALL * before * before, t["tilt"] * min(1.0, before / TILT_FLATTEN)
        u = -before
        rebound = REBOUND_PX * math.exp(-u / REBOUND_TAU) * abs(math.sin(math.pi * u / REBOUND_WAVE)) * (1 - ease(u / REBOUND_END))
        return t["cy"] - rebound + self.press(n, t["hit"]), 0.0

    def at(self, n):
        """Every letter's body at n, read between two steps. The residual blend makes REL the still frame exactly."""
        spot = max(n - REL - STEP, 0.0) / STEP
        i = min(int(spot), len(self.track) - 2)
        pose = self.track[i] + (self.track[i + 1] - self.track[i]) * (spot - i)
        return pose + self.residual * (1 - ease((n - REL) / RESIDUAL_BLEND))

    def sprites(self, n):
        """Where each letter's sprite sits at n: x, y and degrees, offset from its body and rotated with it."""
        pose = self.at(n)
        oy = np.array([l["oy"] for l in self.letters])
        return np.stack([pose[:, 0] - oy * np.sin(pose[:, 2]), pose[:, 1] + oy * np.cos(pose[:, 2]), np.degrees(pose[:, 2])], 1)

    def draw_fall(self, out, n):
        plane = np.zeros((VH, VW), np.float32)
        shots = []
        for t in self.names:
            y, deg = self.scripted(t, n)
            if y + t["h"] / 2 + 60 < 0:
                continue
            y0, deg0 = self.scripted(t, n - STEP)
            move = abs(y - y0)
            poses = [(t["cx"], y, 1.0, 1.0, deg)]
            if move > SMEAR_MIN:
                poses = [(t["cx"], y0 + (y - y0) * u, 1.0, 1.0, deg0 + (deg - deg0) * u)
                         for u in np.linspace(0, 1, min(SMEAR_MOST, 2 + int(move / SMEAR_MIN)))]
            stamp(plane, t["soft"], t["cx"], y + SHADOW_Y, deg)
            shots.append((t["pic"], poses))
        out *= (1 - SHADOW_ALPHA * np.clip(plane, 0, 1))[..., None]
        for pic, poses in shots:
            lay(out, pic, poses)

    def draw_letters(self, out, n):
        now, before = self.sprites(n), self.sprites(n - STEP)
        fade = 1 - ease((n - REL) / SHADOW_FADE)
        if fade > 0:
            plane = np.zeros((VH, VW), np.float32)
            for l, p in zip(self.letters, now):
                if p[1] - l["reach"] - 60 < VH:
                    stamp(plane, l["soft"], p[0], p[1] + SHADOW_Y, p[2])
            out *= (1 - SHADOW_ALPHA * fade * np.clip(plane, 0, 1))[..., None]
        for l, p, b in zip(self.letters, now, before):
            if p[1] - l["reach"] > VH:
                continue
            poses = [(p[0], p[1], 1.0, 1.0, p[2])]
            move = math.hypot(p[0] - b[0], p[1] - b[1])
            if move > SMEAR_MIN:
                poses = [(b[0] + (p[0] - b[0]) * u, b[1] + (p[1] - b[1]) * u, 1.0, 1.0, b[2] + (p[2] - b[2]) * u)
                         for u in np.linspace(0, 1, min(SMEAR_MOST, 2 + int(move / SMEAR_MIN)))]
            lay(out, l["pic"], poses)

    def draw(self, out, n):
        if n > REL:
            self.draw_letters(out, n)
        elif n >= ALL_STILL:
            x0, y0, x1, y1 = self.box
            out[y0:y1, x0:x1] = out[y0:y1, x0:x1] * self.still_keep + self.still_ink
        else:
            self.draw_fall(out, n)

class Ending:
    def __init__(self, font, logo, name, art, keep_image):
        self.back, self.foot = backdrop(), foot()
        self.light = glow((46, 22, 14), .5, .46, .34, .34)
        self.font, self.art, self.name = font, art, name
        self.logo_mark, self.logo_word = split_lockup(logo)
        self.head = word(font, "On every screen", 92)
        self.head_soft = soft_shadow(self.head)
        self.head_x, self.head_rest = snap(CX, self.head.shape[1]), snap(HEAD_Y, self.head.shape[0])
        self.keep = cover(keep_image, 960, 540).astype(np.float32)
        self.keep_full = cover(keep_image, VW, VH).astype(np.float32)
        self.build_tv()
        self.build_laptop()
        self.build_tablet()
        self.build_phones()
        self.build_browser()
        self.tv_soft = cv2.GaussianBlur(cv2.copyMakeBorder(self.tv_screen.pic, 48, 48, 48, 48, cv2.BORDER_CONSTANT, value=0), (0, 0), 7)
        self.stack = Stack(font)
        self.ask_words = [word(font, "Try", 210), word(font, name, 210, ((255, 107, 47), (255, 170, 120))), word(font, "today.", 210)]

    # ---- television
    def build_tv(self):
        self.tv_names = [word(self.font, n, 78) for n in TV_NAMES]
        pad = 9
        pic = body(960, 540, pad, 18)
        self.tv_body_h, self.tv_w = pic.shape[0], pic.shape[1]
        self.tv_shell = pedestal(pic, pic.shape[1], pic.shape[0])
        self.tv_screen = Card(self.keep.astype(np.uint8), 960, 540, 16)
        self.tv_mask = rounded(960, 540, 16)
        self.tv_out = 1436
        self.tv_floor = shade(760, 56, .75)
        self.tv_mirror = mirror(self.tv_shell, 44)
        self.tv_sheen = Sheen(540, 960)
        self.tv_time = word(self.font, "1:04:12", 20, upper=False)
        self.tv_pause = np.zeros((22, 20, 4), np.float32)
        for x in (3, 12):
            self.tv_pause[2:20, x:x + 5] = 255

    def television(self, out, n):
        if n >= self.tv_out + 20:
            return
        away = ease((n - self.tv_out) / 18)
        x = CX - 1500 * away
        arrive = ease((n - N0) / 24)
        y = TV_Y
        h = self.tv_shell.shape[0]
        shell_y = y - 279 + h / 2
        ground = y - 279 + h
        lay(out, self.tv_floor, [(x, ground + 2, 1.0, 1.0, 0.0)], alpha=arrive)
        lay(out, self.tv_mirror, [(x, ground + 22, 1.0, 1.0, 0.0)], alpha=arrive)
        lay(out, self.tv_shell, [(x, shell_y, 1.0, 1.0, 0.0)], alpha=arrive)
        lay(out, self.playing(n), [(x, y, 1.0, 1.0, 0.0)])
        spot = clamp((n - N0 - 8) / (self.tv_out - N0 - 8)) * len(self.tv_names)
        now = min(int(spot), len(self.tv_names) - 1)
        turn = spot - now
        change = ease((turn - .7) / .3) if now < len(self.tv_names) - 1 else 0.0
        keep = (1 - away) * ease((n - N0 - 6) / 10)
        ly = 860
        lay(out, self.tv_names[now], [(x, ly - 50 * change, 1.0, 1 - .4 * change, 0.0)], alpha=keep * (1 - change))
        if change > 0:
            lay(out, self.tv_names[now + 1], [(x, ly + 50 * (1 - change), 1.0, .6 + .4 * change, 0.0)], alpha=keep * change)

    def playing(self, n):
        """The film on the television: a slow push in, the bar, and light moving across the glass."""
        z = 1 + .1 * ease((n - N0) / 110)
        M = np.array([[z, 0, 480 - z * 480], [0, z, 300 - z * 300]], np.float64)
        rgb = cv2.warpAffine(self.keep, M, (960, 540), flags=cv2.INTER_LINEAR)
        show = ease((n - N0 - 8) / 12)
        if show > 0:
            rgb[470:540] = rgb[470:540] * (1 - .55 * show * np.linspace(0, 1, 70, dtype=np.float32)[:, None, None])
            bar = rgb[510:516, 76:850]
            bar[:] = bar * (1 - .55 * show) + 170 * .55 * show
            done = int(774 * BAR_START * clamp((n - N0) / 91))
            rgb[510:516, 76:76 + done] = rgb[510:516, 76:76 + done] * (1 - show) + np.array(BLUE, np.float32) * show
            paste(rgb, self.tv_pause, 42, 502, show)
            paste(rgb, self.tv_time, 862, 498, show)
        self.tv_sheen.add(rgb, (n - N0) / 120)
        a = self.tv_mask
        return np.dstack([rgb * a[..., None], a * 255])

    # ---- laptop
    def build_laptop(self):
        w, h = 820, 512
        page = np.zeros((h, w, 3), np.float32) + (24, 18, 17)
        hero = cover(self.art.tile(self.art.next("drama")), w, 276).astype(np.float32)
        hero *= np.linspace(.3, 1, w, dtype=np.float32)[None, :, None]
        hero *= np.linspace(1, .22, 276, dtype=np.float32)[:, None, None]
        page[:276] = hero
        paste(page, sized(self.logo_mark, 30), 22, 14)
        paste(page, sized(self.logo_word, 24), 62, 17)
        for i, label in enumerate(("Home", "Films", "Series", "My List")):
            paste(page, word(self.font, label, 17, (225, 225, 225), upper=False), 214 + i * 84, 20)
        paste(page, word(self.font, "Tonight", 46, upper=False), 26, 148)
        button = capsule(104, 36, (250, 250, 250))
        paste(button, word(self.font, "Play", 18, (24, 24, 24), upper=False), 30, 7)
        paste(page, button, 28, 218)
        paste(page, word(self.font, "Top picks", 19, (225, 225, 225), upper=False), 24, 280)
        self.lap_page = page
        strip = np.zeros((240, 2600, 3), np.float32) + (24, 18, 17)
        for i in range(12):
            for r in range(2):
                t = cover(self.art.tile(self.art.next("small")), 196, 110).astype(np.float32) * rounded(196, 110, 8)[..., None]
                strip[4 + r * 122:114 + r * 122, 24 + i * 210:220 + i * 210] += t * .96
        self.lap_strip = strip
        self.lap_lid = body(w, h, 12, 16)
        cv2.circle(self.lap_lid, (self.lap_lid.shape[1] // 2, 6), 2, (70, 60, 56, 255), -1, cv2.LINE_AA)
        H0, W0 = self.lap_lid.shape[:2]
        back = metal(W0, H0, top=.8, low=1.25, seed=11)
        paste(back, sized(self.logo_mark, 84), W0 // 2 - 42, H0 // 2 - 42, .35)
        m = rounded(W0, H0, 16)
        self.lap_back = np.dstack([back * m[..., None], m * 255])
        self.lap_sheen = Sheen(h, w)
        self.lap_deck = deck(W0)
        self.lap_shadow = shade(int(W0 * 1.5), 70, .8)
        self.lap_names = [word(self.font, n, 104) for n in ("Mac", "Windows", "Linux")]
        self.lap_in, self.lap_out = 1442, 1487

    def laptop_lid(self, n):
        pan = 40 + 2.2 * (n - self.lap_in)
        M = np.array([[1, 0, -pan], [0, 1, 0]], np.float64)
        screen = self.lap_page.copy()
        screen[306:] = cv2.warpAffine(self.lap_strip, M, (820, 206), flags=cv2.INTER_LINEAR)
        self.lap_sheen.add(screen, (n - self.lap_in) / 70, 10.0)
        return inset(self.lap_lid, screen, 12, 0, 8)

    def laptop(self, out, n):
        a, b = self.lap_in, self.lap_out
        if n < a or n > b + 16:
            return
        up = rise((n - a) / 14)
        down = fall((n - b - 4) / 12)
        cx, hinge = 690.0, 716 + 600 * (1 - up) + 600 * down
        opened = ease((n - a - 8) / 20) * (1 - ease((n - b + 8) / 11))
        dh = self.lap_deck.shape[0]
        lay(out, self.lap_shadow, [(cx, hinge + dh + 2, 1.0, 1.0, 0.0)])
        lay(out, self.lap_deck, [(cx, hinge + dh / 2, 1.0, 1.0, 0.0)])
        h0, w0 = self.lap_lid.shape[:2]
        angle = (1 - opened) * math.radians(100)
        lift = h0 * math.cos(angle)
        if abs(lift) > 2:
            forward = lift < 0
            pic = self.lap_back if forward else self.laptop_lid(n)
            top = w0 / 2 * (1 + .2 * -lift / DECK_H) if forward else w0 / 2 - 16 * lift / h0
            src = np.float32([[0, 0], [w0, 0], [w0, h0], [0, h0]])
            dst = np.float32([[cx - top, hinge - lift], [cx + top, hinge - lift], [cx + w0 / 2, hinge], [cx - w0 / 2, hinge]])
            p = cv2.warpPerspective(pic, cv2.getPerspectiveTransform(src, dst), (VW, VH), flags=cv2.INTER_LINEAR)
            k = p[..., 3:4] / 255
            out[:] = out * (1 - k) + p[..., :3] * (.85 if forward else 1 - .35 * math.sin(angle))
        for i, pic_name in enumerate(self.lap_names):
            t = (n - a - 20 - i * 8) / 18
            l = (n - b + 12 - (2 - i) * 2) / 11
            x = 1530 + 500 * (1 - rise(t)) + 700 * fall(l)
            lay(out, pic_name, [(x, 320 + i * 150, 1.0, 1.0, 0.0)], alpha=ease(t * 1.5) * (1 - ease(l)))

    # ---- tablet
    def build_tablet(self):
        h = 506
        strip = np.zeros((h, 1800, 3), np.float32) + (24, 18, 17)
        for i in range(7):
            t = cover(self.art.poster(self.art.next("drama")), 236, 354).astype(np.float32) * rounded(236, 354, 12)[..., None]
            strip[86:440, 24 + i * 252:260 + i * 252] += t
        self.tab_strip = strip
        self.tab_body = body(680, h, 12, 30)
        cv2.circle(self.tab_body, (self.tab_body.shape[1] // 2, 6), 3, (70, 60, 56, 255), -1, cv2.LINE_AA)
        rail = np.zeros((h, 66, 4), np.float32)
        rail[..., :3], rail[..., 3] = (20, 15, 14), 240
        for i in range(5):
            paste(rail, capsule(26, 26, BLUE if i == 1 else (96, 88, 84), 9), 20, 96 + i * 62)
        self.tab_rail = rail
        self.tab_head = word(self.font, "Drama", 30, upper=False)
        self.tab_sheen = Sheen(h, 680)
        self.tab_shadow = shade(900, 640, .6)
        self.tab_names = [word(self.font, n, 92) for n in ("iPad", "Android tablets")]
        self.tab_in, self.tab_out = 1496, 1532

    def tablet(self, out, n):
        a, b = self.tab_in, self.tab_out
        if n < a or n > b + 20:
            return
        pan = 60 + 3.6 * (n - a)
        M = np.array([[1, 0, -pan], [0, 1, 0]], np.float64)
        screen = cv2.warpAffine(self.tab_strip, M, (680, 506), flags=cv2.INTER_LINEAR)
        paste(screen, self.tab_rail, 0, 0)
        paste(screen, self.tab_head, 84, 34)
        self.tab_sheen.add(screen, (n - a) / 60, 10.0)
        pic = inset(self.tab_body, screen, 12, 0, 20)
        arrive = rise((n - a) / 13)
        turn = spring((n - a - 9) / 24, turns=1.1)
        away = fall((n - b) / 16)
        x = 1230 + 1300 * (1 - arrive) + 1500 * away
        lay(out, self.tab_shadow, [(x, 500, 1.0, 1.0, 0.0)], alpha=turn)
        lay(out, pic, [(x, 480, .95, .95, -90 * (1 - turn) + 26 * away)])
        for i, p in enumerate(self.tab_names):
            t = (n - a - 12 - i * 7) / 16
            l = (n - b + 3 - i * 2) / 11
            px = 450 - 300 * (1 - rise(t))
            lay(out, p, [(px, 390 + i * 130, 1.0, 1.0, 0.0)], alpha=ease(t * 1.6) * (1 - ease(l)))

    # ---- phones
    def build_phones(self):
        self.feeds = []
        for pool in ("comedy", "kids"):
            feed = np.zeros((2400, 244, 3), np.float32) + (24, 18, 17)
            for i in range(6):
                t = cover(self.art.tile(self.art.next(pool)), 224, 330).astype(np.float32) * rounded(224, 330, 14)[..., None]
                feed[10 + i * 346:340 + i * 346, 10:234] += t
            self.feeds.append(feed)
        pic = body(244, 524, 7, 38)
        H, W = pic.shape[:2]
        wide = np.zeros((H, W + 6, 4), np.float32)
        for y0, y1, x0 in ((110, 150, 0), (166, 206, 0), (136, 196, W)):
            m = rounded(6, y1 - y0, 2)
            wide[y0:y1, x0:x0 + 6, :3] = np.array(EDGE, np.float32) * m[..., None] * .8
            wide[y0:y1, x0:x0 + 6, 3] = m * 255
        self.phone_side, self.phone_body = wide, pic
        self.phone_over = []
        for active in (0, 2):
            over = np.zeros((524, 244, 4), np.float32)
            over[454:] = (16, 12, 11, 240)
            over[454:, ..., :3] *= 240 / 255
            for i in range(4):
                paste(over, capsule(26, 26, BLUE if i == active else (96, 88, 84), 9), 26 + i * 52, 472)
            paste(over, capsule(84, 24, (6, 6, 6)), 80, 12)
            cv2.circle(over, (110, 24), 5, (46, 34, 30, 255), -1, cv2.LINE_AA)
            self.phone_over.append(over)
        self.phone_names = [word(self.font, n, 92) for n in ("iPhone", "Android")]
        self.ph_in, self.ph_out = 1532, 1550

    def phones(self, out, n):
        a, b = self.ph_in, self.ph_out
        if n < a or n > b + 16:
            return
        for i, (x0, side) in enumerate([(780, -1), (1140, 1)]):
            t = (n - a - i * 7) / 18
            l = (n - b - i * 2) / 13
            if t <= 0:
                continue
            scroll = (2.6 + i * .9) * (n - a) + 200 * i
            M = np.array([[1, 0, 0], [0, 1, -scroll]], np.float64)
            screen = cv2.warpAffine(self.feeds[i], M, (244, 524), flags=cv2.INTER_LINEAR)
            paste(screen, self.phone_over[i], 0, 0)
            face = inset(self.phone_body, screen, 7, 0, 32)
            pic = self.phone_side.copy()
            pic[:, 3:-3] = face + pic[:, 3:-3] * (1 - face[..., 3:4] / 255)
            y = 480 + 900 * (1 - rise(t)) + 900 * fall(l)
            deg = side * (3 + 14 * (1 - rise(t)))
            lay(out, pic, [(x0 + side * 40 * (1 - rise(t)), y, 1.0, 1.0, deg)])
            p = self.phone_names[i]
            lt = (n - a - 12 - i * 7) / 14
            lay(out, p, [(x0 + side * 410 + side * 200 * (1 - rise(lt)), 480, 1.0, 1.0, 0.0)], alpha=ease(lt * 1.6) * (1 - ease(l)))

    # ---- browser
    def build_browser(self):
        w, h, top = 900, 500, 72
        page = np.zeros((h, w, 3), np.float32) + (24, 18, 17)
        hero = cover(self.art.tile(self.art.next("comedy")), w, 300).astype(np.float32)
        hero *= np.linspace(.25, 1.1, w, dtype=np.float32)[None, :, None]
        hero *= np.linspace(1, .16, 300, dtype=np.float32)[:, None, None]
        page[:300] = hero
        paste(page, word(self.font, "New this week", 44, upper=False), 34, 182)
        self.br_cards, self.br_at = [], []
        for i in range(4):
            t = cover(self.art.tile(self.art.next("small")), 200, 150).astype(np.float32)
            m = rounded(200, 150, 12)
            pic = np.dstack([t * m[..., None], m * 255])
            x = 34 + i * 216
            self.br_cards.append(pic)
            self.br_at.append((x + 100, 320 + 75))
            paste(page, pic, x, 320)
        pic = body(w, h, 10, 18, top=top)
        pic = inset(pic, page, 10, top, 8)
        for i in range(3):
            cv2.circle(pic, (32 + i * 22, 28), 6, (120, 110, 104, 255), -1, cv2.LINE_AA)
        paste_pic(pic, capsule(210, 34, (64, 54, 50), 12), 110, 12)
        paste_pic(pic, sized(self.logo_mark, 18), 122, 20)
        paste_pic(pic, word(self.font, self.name, 15, upper=False), 150, 18)
        paste_pic(pic, capsule(170, 34, (40, 32, 30), 12), 328, 12)
        paste_pic(pic, capsule(w - 150, 26, (20, 15, 14)), 110, 50)
        lock = np.zeros((16, 14, 4), np.float32)
        cv2.rectangle(lock, (1, 7), (12, 15), (200, 200, 200, 255), -1, cv2.LINE_AA)
        cv2.ellipse(lock, (6, 7), (4, 5), 0, 180, 360, (200, 200, 200, 255), 2, cv2.LINE_AA)
        paste_pic(pic, lock, 126, 55)
        paste_pic(pic, word(self.font, "home.server/browse", 14, (190, 190, 190), upper=False), 146, 53)
        self.br_pic = pic
        self.br_top = top + 10
        self.br_shadow = shade(int(pic.shape[1] * 1.5), int(pic.shape[0] * 1.5), .75)
        self.br_card_shadow = shade(300, 220, .8)
        arrow = np.zeros((56, 40, 4), np.float32)
        pts = np.array([[4, 4], [4, 44], [14, 34], [22, 52], [30, 48], [22, 31], [36, 31]], np.int32)
        cv2.fillPoly(arrow, [pts], (248, 248, 248, 255), cv2.LINE_AA)
        self.arrow = arrow
        self.br_name = word(self.font, "Any browser", 78)
        self.br_in, self.br_out = 1562, 1592

    def browser(self, out, n):
        a, b = self.br_in, self.br_out
        if n < a or n > b + 12:
            return
        size = .3 + .7 * spring((n - a) / 18, turns=1.1)
        away = ease((n - b) / 12)
        cy = 470.0
        s = size * (1 - .15 * away)
        fade = min(1.0, (n - a) / 4) * (1 - away)
        lay(out, self.br_shadow, [(CX, cy + 26, s, s, 0.0)], alpha=fade)
        lay(out, self.br_pic, [(CX, cy, s, s, 0.0)], alpha=fade)
        H, W = self.br_pic.shape[:2]
        cardx, cardy = self.br_at[2]
        px, py = CX + (cardx + 10 - W / 2) * s, cy + (cardy + self.br_top - H / 2) * s
        t = (n - a - 8) / 15
        x = 1600 + (px - 1600) * ease(t)
        y = 880 + (py - 880) * ease(t)
        lift = ease((n - a - 17) / 7)
        if lift > 0:
            big = s * (1 + .1 * lift)
            lay(out, self.br_card_shadow, [(px, py + 14, big, big, 0.0)], alpha=fade * lift)
            lay(out, self.br_cards[2], [(px, py - 8 * lift, big, big, 0.0)], alpha=fade)
        click = n - a - 23
        ring = clamp(click / 7)
        if click > 0:
            r = int(16 + 70 * ring)
            layer = np.zeros((VH, VW, 3), np.float32)
            cv2.circle(layer, (int(px), int(py)), r, WHITE, 4, cv2.LINE_AA)
            out[:] = out + layer * (1 - ring) * .8
            d = 1 - .12 * math.sin(clamp(click / 5) * math.pi)
        else:
            d = 1.0
        lay(out, self.arrow, [(x + 14, y + 22, d, d, 0.0)], alpha=(1 - away) * clamp((n - a - 4) / 4))
        nt = (n - a - 6) / 14
        lay(out, self.br_name, [(CX, 860 + 26 * (1 - rise(nt)), 1.0, 1.0, 0.0)], alpha=ease(nt * 1.5) * (1 - away))

    # ---- the return, the bar and the wall
    def card_place(self, n):
        """The returning television, from the left, growing until its screen is the picture."""
        t = ease((n - BACK) / 48)
        return -600 + (CX + 600) * ease((n - BACK) / 38), TV_Y + (CY - TV_Y) * t, 1 + (CARD_END - 1) * t, t

    def comes_back(self, out, n):
        """The art dims and softens behind the wall, and only the art keeps pushing in."""
        x, y, s, _ = self.card_place(n)
        s *= 1 + ART_PUSH * max(n - ART_PUSH_START, 0)
        fade = 1 - float(smooth(REL, REL + 20, n))
        dim = 1 - .7 * ease((n - 1604) / 44)
        soft = ease((n - 1612) / 36)
        if soft < 1:
            lay(out, self.tv_screen.pic, [(x, y, s, s, 0.0)], alpha=fade, light=dim)
        if soft > 0:
            lay(out, self.tv_soft, [(x, y, s, s, 0.0)], alpha=fade * soft, light=dim)

    def bar(self, out, n):
        """The film's bar: it arrives on the television, drops to the floor, fills, then runs out to the right."""
        if n > REL:
            pill(out, floor_left(n), BAR_RIGHT, BAR_Y, BAR_H, BLUE)
            return
        x, y, s, t = self.card_place(n)
        half = 444 * s + ((BAR_RIGHT - BAR_LEFT) / 2 - 444 * CARD_END) * t
        x0, x1 = x - half, x + half
        by = y + 243 * s
        by += (BAR_Y - by) * ease((n - BAR_LIFT) / BAR_LIFT_FRAMES) + self.stack.press(n)
        h = max(6, round(6 * s))
        done = BAR_START + (1 - BAR_START) * clamp((n - HOLD_LINE) / (REL - HOLD_LINE))
        pill(out, x0, x1, by, h, (170, 170, 170), .55)
        pill(out, x0, x0 + (x1 - x0) * done, by, h, BLUE)

    def head_back(self, out, n):
        """The head stands up on the last landing, in full white, and rises away at the release."""
        t = rise((n - HOLD_LINE) / 17.5)
        leave = ease((n - REL) / 12)
        y = self.head_rest + 22 * (1 - t) - 16 * leave
        alpha = t * (1 - leave)
        lay(out, self.head_soft, [(self.head_x, y + SHADOW_Y, 1.0, 1.0, 0.0)], alpha=alpha * SHADOW_ALPHA)
        lay(out, self.head, [(self.head_x, y, 1.0, 1.0, 0.0)], alpha=alpha)

    # ---- the ask and the mark
    def ask(self, out, n):
        a, b, c = self.ask_words
        h = a.shape[0]
        width = a.shape[1] + 12 + b.shape[1]
        left = (VW - width) / 2
        y1, y2 = CY - h * .55 - 40, CY + h * .55 - 40
        leave = ease((n - (MARK - 6)) / 6)
        t = rise((n - ASK) / 12)
        lay(out, a, [(left + a.shape[1] / 2, y1 + 44 * (1 - t) - 12 * leave, 1.0, 1.0, 0.0)], alpha=min(1.0, t * 1.6) * (1 - leave))
        t = rise((n - ASK - 3.5) / 12)
        off = 260 * (1 - t)
        lay(out, b, [(left + a.shape[1] + 12 + b.shape[1] / 2 + off, y1 - 12 * leave, 1.0, 1.0, 0.0)], alpha=min(1.0, t * 1.6) * (1 - leave),
            sigma=10 * (1 - t))
        t = spring((n - ASK - 7) / 12, turns=1.1)
        lay(out, c, [(CX, y2 - 12 * leave, t, t, 0.0)], alpha=clamp((n - ASK - 7) / 6) * (1 - leave))

    def frame(self, n):
        out = self.back * (1 - float(smooth(REL, REL + 20, n)))
        if n < RETURN:
            spot = ease((n - N0) / 30) * (1 - ease((n - (RETURN - 16)) / 16))
            out = out + self.light * spot
            self.television(out, n)
            self.laptop(out, n)
            self.tablet(out, n)
            self.phones(out, n)
            arrive = rise((n - (N0 + 6)) / 24)
            leave = ease((n - (RETURN - 14)) / 14)
            lay(out, self.head, [(CX, HEAD_Y + 22 * (1 - arrive) - 16 * leave, 1.0, 1.0, 0.0)], alpha=arrive * (1 - leave))
        # The browser leaves after the bar line, under the television coming back.
        self.browser(out, n)
        if BACK <= n < REL + 22:
            self.comes_back(out, n)
        if BACK <= n < REL + RETREAT_FRAMES:
            self.bar(out, n)
        if RETURN <= n < MARK:
            self.stack.draw(out, n)
        if HOLD_LINE <= n < REL + 12:
            self.head_back(out, n)
        if ASK <= n < MARK:
            self.ask(out, n)
        if n >= MARK:
            lockup(out, self.logo_mark, self.logo_word, n, MARK, CLOSE_LEAVE, CLOSE_END, scale=1.32, y=CY - 40)
        return np.clip(out * self.foot, 0, 255).astype(np.uint8)

def load(path):
    im = cv2.imread(str(path), cv2.IMREAD_UNCHANGED).astype(np.float32)
    im[..., :3] *= im[..., 3:4] / 255
    return im
