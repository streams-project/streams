#!/usr/bin/env python3
"""The headlines of the showreel, each topic with its own layout and its own way of moving.

Frame numbers are the film's, and may be fractions. The wall and the opening logo read the film's
own frame. Everything after them reads that frame minus OPEN, the one bar added for the logo, so a
change of topic stays on the bar lines of the music, 70 frames apart from frame 896 on that clock.
Every card has its own clock, depth, focus and shadow.
"""
import math
from pathlib import Path
import cv2, numpy as np
from render import Art, backdrop, foot, smooth, wall_light, wall_tiles
from stage import (CX, CY, VH, VW, Card, bounce, clamp, cover, ease, fall, glow, lay, lay_card, lockup, project,
                   rise, rounded, soft_shadow, split_lockup, spring, word)

SHUTTER = .38  # film frames a fast card smears across
OPEN = 70  # frames added so the opening logo can stand
LOGO_IN, LOGO_LEAVE, LOGO_OUT = 636, 728, 738
LOGO_SCALE, LOGO_PUSH = 1.32, .03

def jitter(seed, a=1.0):
    """A fixed small number for this seed, so a card always keeps its own timing."""
    return a * (math.sin(seed * 12.9898) * 43758.5453 % 1 - .5) * 2

class Item:
    """One card and what its scene remembers about it."""
    def __init__(self, card, **kw):
        self.card = card
        self.__dict__.update(kw)

def sweep(out, items, state, n, fixed=False):
    """Draw cards far to near. state(item, n) gives x, y, z, degrees, alpha, light, sigma, squash x, squash y.
    fixed keeps the given order, for cards that overlap while each runs its own clock."""
    drawn = []
    for item in items:
        pose = state(item, n)
        if pose is not None:
            drawn.append((pose[2], item, pose))
    if not fixed:
        drawn.sort(key=lambda d: -d[0])
    for _, item, pose in drawn:
        samples = [pose]
        before = state(item, n - SHUTTER)
        if before is not None:
            a = project(pose[0], pose[1], pose[2])
            b = project(before[0], before[1], before[2])
            if math.hypot(a[0] - b[0], a[1] - b[1]) > 5:
                samples = [state(item, n - SHUTTER * k / 3) or pose for k in range(4)]
        poses = []
        for p in samples:
            x, y, s = project(p[0], p[1], p[2])
            poses.append((x, y, s * p[7], s * p[8], p[3]))
        lay_card(out, item.card, poses, alpha=pose[4], light=pose[5], sigma=pose[6], shadow=.5)

class Head:
    """A headline, word by word, moving in a way that fits its topic."""
    def __init__(self, font, lines, size, ys, style, start, stop):
        self.style, self.start, self.stop = style, start, stop
        self.words = []
        for line, y in zip(lines, ys):
            pics = [word(font, w, size) for w in line.split()]
            gap = size * .02
            width = sum(p.shape[1] for p in pics) + gap * (len(pics) - 1)
            x = (VW - width) / 2
            for p in pics:
                self.words.append(dict(pic=p, soft=soft_shadow(p), x=x + p.shape[1] / 2, y=y, line=len(self.words)))
                x += p.shape[1] + gap
        for i, w in enumerate(self.words):
            w["i"] = i
        self.lines = len(lines)

    def pose(self, w, n):
        """x, y, scale x, scale y, degrees, alpha, sigma."""
        i, style = w["i"], self.style
        x, y, sx, sy, deg, alpha, sigma = w["x"], w["y"], 1.0, 1.0, 0.0, 1.0, 0.0
        go = (n - self.stop - i * 1.6) / 12
        if style == "rise":
            t = (n - self.start - i * 3.2) / 16
            y += 34 * (1 - rise(t)); alpha = ease(t * 1.5)
            y -= 18 * ease(go); alpha *= 1 - ease(go)
        elif style == "flip":
            t = (n - self.start - i * 2.6) / 13
            sy = abs(math.cos((1 - ease(t)) * math.pi / 2)); alpha = 1.0 if t > 0 else 0.0
            sy *= abs(math.cos(ease(go) * math.pi / 2)); alpha *= 1 if go < 1 else 0
        elif style == "sides":
            side = -1 if w["y"] < CY else 1
            t = (n - self.start - i * 2.4) / 18
            x += side * 420 * (1 - rise(t)); alpha = ease(t * 1.8); sigma = 8 * (1 - rise(t))
            x -= side * 300 * fall(go); alpha *= 1 - ease(go)
        elif style == "track":
            t = (n - self.start - i * 2) / 22
            s = 1.16 - .16 * rise(t); sx = sy = s; alpha = ease(t * 1.6); sigma = 12 * (1 - rise(t))
            x = CX + (x - CX) * s
            sx *= 1 - .04 * ease(go); sy = sx; alpha *= 1 - ease(go); sigma += 8 * ease(go)
        elif style == "slam":
            t = (n - self.start - i * 3) / 11
            s = 1.8 - .8 * rise(t); sx = sy = s; alpha = ease(t * 2.5)
            x -= 3200 * fall(go)
        elif style == "soft":
            t = (n - self.start - i * 4) / 30
            alpha = ease(t); sigma = 10 * (1 - ease(t)); y += 12 * (1 - ease(t))
            alpha *= 1 - ease(go * .6); sigma += 10 * ease(go * .6)
        elif style == "pop":
            t = (n - self.start - i * 3) / 15
            s = spring(t, turns=1.3); sx = sy = max(0.0, s); deg = (6 if i % 2 else -6) * (1 - ease(t * 1.3))
            alpha = 1.0 if t > 0 else 0.0
            sx *= 1 - ease(go); sy = sx
        elif style == "gate":
            t = (n - self.start - i * 6) / 12
            alpha = ease(t); sigma = 6 * (1 - ease(t))
            alpha *= 1 - ease(go)
        elif style == "hop":
            t = (n - self.start - i * 3.4) / 24
            y -= 380 * (1 - bounce(t)); alpha = ease(t * 3)
            y += 900 * fall(go); alpha *= 1 if go < 1 else 0
        elif style == "meet":
            side = -1 if i < len(self.words) / 2 else 1
            t = (n - self.start) / 24
            x += side * 520 * (1 - rise(t)); alpha = ease(t * 1.5); sigma = 10 * (1 - rise(t))
            alpha *= 1 - ease(go); sigma += 8 * ease(go)
        return x, y, sx, sy, deg, alpha, sigma

    def draw(self, out, n):
        if n < self.start - 1 or n > self.stop + 40:
            return
        for w in self.words:
            x, y, sx, sy, deg, alpha, sigma = self.pose(w, n)
            if alpha <= 0:
                continue
            lay(out, w["soft"], [(x, y + 6, sx, sy, deg)], alpha=alpha * .55)
            lay(out, w["pic"], [(x, y, sx, sy, deg)], alpha=alpha, sigma=sigma)

class Wall:
    """Change 1: the tilted wall settles in a diagonal wave, each card through true perspective."""
    def __init__(self, art):
        H, tiles = wall_tiles()
        light = wall_light()
        self.tiles = []
        for x0, y0, tw, th, portrait in tiles:
            key = art.next("wall")
            img = art.poster(key) if portrait else art.tile(key)
            ih, iw = img.shape[:2]
            A = cv2.getPerspectiveTransform(np.float32([[0, 0], [iw, 0], [iw, ih], [0, ih]]),
                                            np.float32([[x0, y0], [x0 + tw, y0], [x0 + tw, y0 + th], [x0, y0 + th]]))
            Hi = H @ A
            box = cv2.perspectiveTransform(np.float32([[[0, 0]], [[iw, 0]], [[iw, ih]], [[0, ih]]]), Hi).reshape(-1, 2)
            if box[:, 0].max() < -40 or box[:, 0].min() > VW + 40 or box[:, 1].max() < -40 or box[:, 1].min() > VH + 40:
                continue
            a = rounded(iw, ih, 14 / 432 * iw * (1.0 if not portrait else 1.25))
            cx, cy = box.mean(0)
            lit = float(light[int(clamp(cy / VH) * (VH - 1)), int(clamp(cx / VW) * (VW - 1)), 0])
            pic = np.dstack([img.astype(np.float32) * a[..., None], a * 255])
            self.tiles.append(dict(pic=pic, H=Hi, box=box, c=(float(cx), float(cy)), lit=lit,
                                   wave=(cx * .7 + cy * .6) / 1900, seed=len(self.tiles)))

    def draw(self, out, n):
        for t in self.tiles:
            wait = 22 * t["wave"] + jitter(t["seed"], 1.8)
            u = (n - 590 - wait) / 18
            # The lights fall in the wall's own wave, then it drains away behind the standing logo.
            l = (n - 622 - 14 * (1 - t["wave"]) - jitter(t["seed"] + 9, 1.5)) / 15
            go = (n - 696 - 22 * t["wave"] - jitter(t["seed"] + 9, 1.5)) / 18
            if u <= 0 or go >= 1:
                continue
            s = (1 + .2 * (1 - rise(u))) * (1 - .04 * ease(l)) * (1 + .0007 * (n - 610))
            alpha = ease(u * 1.7) * (1 - .74 * ease(l)) * (1 - ease(go))
            cx, cy = t["c"]
            M = np.array([[s, 0, cx - s * cx], [0, s, cy - s * cy], [0, 0, 1]], np.float64) @ t["H"]
            b = t["box"]
            x0, y0 = max(int(cx + (b[:, 0].min() - cx) * s) - 2, 0), max(int(cy + (b[:, 1].min() - cy) * s) - 2, 0)
            x1, y1 = min(int(cx + (b[:, 0].max() - cx) * s) + 3, VW), min(int(cy + (b[:, 1].max() - cy) * s) + 3, VH)
            if x1 <= x0 or y1 <= y0:
                continue
            T = np.array([[1, 0, -x0], [0, 1, -y0], [0, 0, 1]], np.float64)
            p = cv2.warpPerspective(t["pic"], T @ M, (x1 - x0, y1 - y0), flags=cv2.INTER_LINEAR)
            k = p[..., 3:4] / 255 * alpha
            out[y0:y1, x0:x1] = out[y0:y1, x0:x1] * (1 - k) + p[..., :3] * alpha * t["lit"]

class Grid:
    """Changes 3 and 4: the first cards land in a ripple, then flip to new art in a wave."""
    W, H = 308, 172

    def __init__(self, art):
        self.items = []
        pitch_x, pitch_y = 338, 209.5
        for r in range(-1, 7):
            y = 138 + (r - 1) * pitch_y + self.H / 2 + 95
            x0 = 62 if r % 2 else 178
            for k in range(-4, 7):
                x = x0 + k * pitch_x + self.W / 2
                near = math.hypot((x - 420) / 1000, (y - 640) / 560)
                self.items.append(Item(Card(art.tile(art.next("small")), self.W, self.H),
                                       other=Card(art.tile(art.next("small")), self.W, self.H), x=x, y=y, row=r,
                                       speed=1.42 * (1 + .22 * math.sin(r * 1.7)), depth=.04 * math.sin(r * 2.3 + 1),
                                       land=654 + 17 * clamp(near) + jitter(k * 31 + r, 2.2), seed=k * 7 + r * 13))
        self.state = self.pose

    def pose(self, c, n):
        t = (n - c.land) / 22
        if t <= 0:
            return None
        x = c.x + c.speed * (n - 713)
        leave = (n - 784 - abs(c.row - 2.5) * 3.2 - jitter(c.seed, 1.5)) / 24
        side = -1 if c.row % 2 else 1
        x += side * 2300 * fall(leave)
        if leave >= 1 or x < -300 or x > VW + 300:
            return None
        z = c.depth - .74 * (1 - rise(t))
        wave = 727 + 21 * clamp((x + 100) / 2000) + 2.2 * (c.row + 1) + jitter(c.seed + 3, 1.4)
        turn = ease((n - wave) / 13) * math.pi
        z -= .07 * math.sin(turn)
        squash = max(abs(math.cos(turn)), .012)
        light = (.92 + .08 * math.sin(c.seed)) * (1 - .4 * math.sin(turn))
        return x, c.y, z, 3 * (1 - rise(t)) * jitter(c.seed + 5), ease(t * 2.4), light, 14 * (1 - rise(t)) ** 1.4, squash, 1.0

    def draw(self, out, n):
        def faced(c, m):
            return self.pose(c, m)
        items = []
        for c in self.items:
            wave_x = c.x + c.speed * (n - 713)
            wave = 727 + 21 * clamp((wave_x + 100) / 2000) + 2.2 * (c.row + 1) + jitter(c.seed + 3, 1.4)
            flipped = ease((n - wave) / 13) > .5
            items.append(Item(c.other if flipped else c.card, **{k: v for k, v in c.__dict__.items() if k not in ("card",)}))
        sweep(out, items, self.pose, n)

class Rows:
    """Change 5: rows run in opposite directions at different depths, under still words."""
    def __init__(self, art):
        self.items = []
        for r, (y, z, speed) in enumerate([(158, 0.0, 6.2), (398, .42, 3.4), (682, .42, 3.4), (930, 0.0, 6.2)]):
            side = 1 if r % 2 else -1
            for k in range(-5, 8):
                self.items.append(Item(Card(art.tile(art.next("wide")), 480, 270), x=(CX + k * 520 + (r * 170)), y=y, z=z, side=side,
                                       speed=speed, row=r, seed=k * 11 + r * 5))

    def pose(self, c, n):
        t = (n - 790 - c.row * 2 - jitter(c.seed, 1.4)) / 28
        if t <= 0:
            return None
        x = c.x + c.side * c.speed * (n - 810) + c.side * -2200 * (1 - rise(t))
        leave = (n - 842 - c.row * 2.5 - jitter(c.seed + 2, 2)) / 20
        if leave >= 1:
            return None
        z = c.z + .7 * ease(leave)
        fog = 1 - c.z * .9
        return x, c.y, z, 0.0, ease(t * 2) * (1 - ease(leave)), fog, 2.5 * c.z + 8 * ease(leave), 1.0, 1.0

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n)

class Depth:
    """Change 6: cards come from in front of the viewer and settle back, each on its own clock."""
    def __init__(self, art, pool, start, stop):
        self.start, self.stop = start, stop
        self.items = []
        ys = [50, 300 - 8, 790, 1060]
        for r, y in enumerate(ys):
            for k, x in enumerate([-30, 480, 960, 1440, 1950]):
                d = math.hypot((x - CX) / 1100, (y - CY) / 560)
                i = len(self.items)
                self.items.append(Item(Card(art.tile(art.next(pool)), 464, 261), x=x + (24 if r % 2 else -24), y=y, near=clamp(d),
                                       seed=i * 3 + 1, deg=jitter(i, 4)))
        self.items.sort(key=lambda c: -c.near)

    def pose(self, c, n):
        wait = 22 * c.near + jitter(c.seed, 3)
        t = (n - self.start - wait) / 30
        l = (n - self.stop - 15 * (1 - c.near) - jitter(c.seed + 4, 2.5)) / 25
        if t <= 0 or l >= 1:
            return None
        z = -.74 * (1 - rise(t)) + .012 * math.sin(n / 26 + c.seed) + .66 * ease(l)
        return (c.x, c.y, z, c.deg * (1 - rise(t)) + .5 * ease(l) * c.deg, ease(t * 2.6) * (1 - ease(l * 1.1)),
                1 - .2 * (1 - c.near), 16 * (1 - rise(t)) ** 1.5 + 11 * ease(l), 1.0, 1.0)

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n, fixed=True)

class Action:
    """Change 7: one hero card and a stack. They slam in, the hit shakes the rest, and they whip away."""
    def __init__(self, art, hero, start, stop):
        self.start, self.stop = start, stop
        self.items = [Item(Card(hero, 1120, 630, 24), x=650, y=450, order=0, seed=1, big=True)]
        for i, y in enumerate([160, 448, 736]):
            self.items.append(Item(Card(art.tile(art.next("action")), 500, 281), x=1540, y=y, order=i + 1, seed=i + 2, big=False))

    def shake(self, n):
        hit = n - (self.start + 13)
        return 0.0 if hit < 0 else 8 * math.exp(-hit / 6) * math.sin(hit * 1.5)

    def pose(self, c, n):
        wait = 0 if c.big else 7 + c.order * 3.4
        t = (n - self.start - wait) / 13
        l = (n - self.stop - c.order * 2) / 17
        if t <= 0 or l >= 1:
            return None
        x = c.x + (2300 if c.big else 1300) * (1 - clamp(t)) ** 3 - 3000 * fall(l) + (n - self.stop) * (-.5 if c.big else .3)
        y = c.y + self.shake(n) * (.4 if c.big else 1) + (n - self.stop) * (0 if c.big else (.35 if c.order % 2 else -.35))
        return x, y, -.02 if c.big else 0.0, -2.5 * (1 - rise(t)), 1.0, 1.0, 0.0, 1.0, 1.0

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n)

class Posters:
    """Change 8: tall posters in an arc. Focus pulls out from the centre, slowly."""
    def __init__(self, art, start, stop):
        self.start, self.stop = start, stop
        self.items = []
        for i in range(-3, 4):
            key = art.next("drama")
            self.items.append(Item(Card(art.poster(key), 372, 523, 18), i=i, x=CX + i * 318, y=610 + 26 * i * i, deg=i * 3.6,
                                   z=.05 * abs(i) - .02, seed=i + 9))

    def pose(self, c, n):
        wait = abs(c.i) * 8
        t = (n - self.start - wait) / 36
        l = (n - self.stop - (3 - abs(c.i)) * 3) / 26
        if t <= 0 or l >= 1:
            return None
        push = -.022 * ease((n - self.start) / 80)
        return (c.x, c.y + 170 * (1 - ease(t)), c.z + .22 * (1 - ease(t)) + push, c.deg, ease(t * 1.3) * (1 - ease(l)),
                .9 - .1 * abs(c.i), 15 * (1 - ease(t)) + 16 * ease(l) + abs(c.i) * .5, 1.0, 1.0)

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n)

class Scatter:
    """Change 9: loose tilted cards of different sizes pop in with a spring."""
    SPOTS = [(250, 190, 400, -6), (690, 140, 340, 5), (1140, 190, 420, -3), (1640, 170, 380, 7), (170, 500, 340, 4),
             (1750, 520, 360, -5), (330, 850, 420, 5), (800, 880, 360, -4), (1250, 860, 400, 3), (1700, 870, 340, -7)]

    def __init__(self, art, start, stop):
        self.start, self.stop = start, stop
        self.items = []
        order = [4, 0, 7, 2, 9, 5, 1, 8, 3, 6]
        for i, (x, y, w, deg) in enumerate(self.SPOTS):
            self.items.append(Item(Card(art.tile(art.next("comedy")), w, int(w * 9 / 16)), x=x, y=y, deg=deg, order=order[i], seed=i + 40))

    def pose(self, c, n):
        t = (n - self.start - c.order * 2.6) / 17
        l = (n - self.stop - (9 - c.order) * 1.5) / 12
        if t <= 0 or l >= 1:
            return None
        s = spring(t, turns=1.25) * (1 + .06 * math.sin(clamp(l) * math.pi) - clamp(l) ** 2)
        wob = c.deg + (1 - clamp(t)) ** 2 * 18 * (1 if c.order % 2 else -1) + 1.3 * math.sin(n / 11 + c.seed)
        return c.x, c.y + 5 * math.sin(n / 13 + c.seed * 2), 0.0, wob, 1.0, 1.0, 0.0, max(s, .002), max(s, .002)

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n)

class Slits:
    """Change 10: tall slits of dark film open along a line of light, and close like blinds."""
    def __init__(self, art, start, stop):
        self.start, self.stop = start, stop
        self.items = []
        for i in range(-2, 3):
            key = art.next("thriller")
            im = cv2.imread(str(art.root / "art" / key / "backdrop.jpg"))
            self.items.append(Item(Card(im, 340, 830, 16), i=i, x=CX + i * 362, y=470, seed=i + 60))

    def pose(self, c, n):
        order = c.i + 2
        t = (n - self.start - order * 7) / 20
        l = (n - self.stop - order * 3) / 14
        if t <= 0 or l >= 1:
            return None
        lit = .1 + .78 * ease((n - self.start - order * 9 - 8) / 18)
        return c.x, c.y + 8 * math.sin(n / 40 + c.i), 0.01 * order, 0.0, 1.0, lit, 0.0, max(1 - ease(l), .002), max(ease(t), .002)

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n)

class Tumble:
    """Change 11: big friendly cards fall in, bounce, settle, and later hop off."""
    def __init__(self, art, start, stop):
        self.start, self.stop = start, stop
        self.items = []
        for r, y in enumerate([440, 758]):
            for k, x in enumerate([410, 960, 1510]):
                i = r * 3 + k
                self.items.append(Item(Card(art.tile(art.next("kids")), 520, 293, 34), x=x, y=y, i=i, seed=i + 70, deg=jitter(i + 5, 15)))

    def pose(self, c, n):
        t = (n - self.start - c.i * 4.2) / 30
        l = (n - self.stop - (5 - c.i) * 2.2) / 22
        if t <= 0 or l >= 1:
            return None
        y = c.y - 1050 * (1 - bounce(t)) - 90 * math.sin(clamp(l * 2.4) * math.pi) + 1500 * fall(l) + 4 * math.sin(n / 15 + c.i * 1.7)
        return c.x, y, 0.0, c.deg * (1 - ease(t * 1.2)) + c.deg * .7 * ease(l), 1.0, 1.0, 0.0, 1.0, 1.0

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n)

class World:
    """Change 12: six cards swirl in on arcs. Five swirl out, and one becomes the television's screen."""
    SCREEN = (CX, 490.0, 960, 540)

    def __init__(self, art, keep_image, start, stop, end):
        self.start, self.stop, self.end = start, stop, end
        self.items = []
        spots = [(x, y) for y in (452, 766) for x in (419, 960, 1501)]
        for i, (x, y) in enumerate(spots):
            keep = i == 4
            card = Card(keep_image, 960, 540, 16) if keep else Card(art.tile(art.next("world")), 488, 272)
            self.items.append(Item(card, x=x, y=y, i=i, keep=keep, seed=i + 80, r=math.hypot(x - CX, y - CY), a=math.atan2(y - CY, x - CX)))

    def pose(self, c, n):
        t = (n - self.start - c.i * 3.4) / 28
        l = (n - self.stop - c.i * 1.6) / 24
        if t <= 0:
            return None
        e = rise(t)
        size = (488 / c.card.w)
        if c.keep:
            m = ease((n - self.stop) / (self.end - self.stop))
            r = c.r + 1250 * (1 - e)
            a = c.a + 1.5 * (1 - e)
            x0, y0 = CX + r * math.cos(a), CY + r * math.sin(a)
            x, y = x0 + (self.SCREEN[0] - x0) * m, y0 + (self.SCREEN[1] - y0) * m
            s = size + (1 - size) * m
            return x, y, -.0001, -34 * (1 - e), ease(t * 2.4), 1.0, 10 * (1 - e), s, s
        if l >= 1:
            return None
        spin = 1.5 * (1 - e) + 1.9 * ease(l)
        r = c.r + 1250 * (1 - e) + 1500 * fall(l)
        return (CX + r * math.cos(c.a + spin), CY + r * math.sin(c.a + spin) + 5 * math.sin(n / 17 + c.i * 2), 0.0, -34 * (1 - e) + 30 * ease(l), ease(t * 2.4),
                1.0, 10 * (1 - e), size, size)

    def draw(self, out, n):
        sweep(out, self.items, self.pose, n)

def hero_image(root, key):
    """A title's wide art with its logo on it, large enough for a big card."""
    im = cv2.imread(str(root / "art" / key / "backdrop.jpg"))
    im = cover(im, 1280, 720).astype(np.float32)
    path = root / "art" / key / "logo.png"
    if path.exists():
        logo = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
        if logo is not None and logo.ndim == 3 and logo.shape[2] == 4:
            w = 420
            h = int(logo.shape[0] * w / logo.shape[1])
            if h < 240:
                logo = cv2.resize(logo, (w, h), interpolation=cv2.INTER_AREA).astype(np.float32)
                x, y = 70, 720 - h - 66
                a = logo[..., 3:4] / 255
                im[y:y + h, x:x + w] = im[y:y + h, x:x + w] * (1 - a) + logo[..., :3] * a
    shade = np.linspace(.92, 1, 1280, dtype=np.float32)[None, :, None]
    return np.clip(im * shade, 0, 255).astype(np.uint8)

class Scenes:
    def __init__(self, root, font, logo):
        root = Path(root)
        self.art = art = Art(root)
        lines = [l.strip() for l in (Path(__file__).with_name("headlines.txt")).read_text().splitlines() if l.strip() and not l.startswith("#")]
        heads = [[p.strip() for p in l.split("|")] for l in lines]
        self.back, self.foot = backdrop(), foot()
        print("building scenes", flush=True)
        self.wall = Wall(art)
        self.mark, self.name = split_lockup(logo)
        self.grid = Grid(art)
        self.rows = Rows(art)
        hero_action = hero_image(root, art.next("action"))
        self.keep_key = art.next("world")
        self.keep_image = hero_image(root, self.keep_key)
        self.layers = [
            (838, 926, Depth(art, "wide2" if "wide2" in art.pools else "wide", 838, 896)),
            (904, 990, Action(art, hero_action, 904, 966)),
            (966, 1064, Posters(art, 966, 1036)),
            (1042, 1134, Scatter(art, 1042, 1106)),
            (1112, 1200, Slits(art, 1112, 1176)),
            (1180, 1272, Tumble(art, 1180, 1246)),
            (1250, 1345, World(art, self.keep_image, 1250, 1316, 1345)),
        ]
        two = (410, 571)
        self.heads = [
            Head(font, heads[0], 133, two, "rise", 678, 727),
            Head(font, heads[1], 133, two, "flip", 736, 784),
            Head(font, heads[2], 133, two, "sides", 792, 841),
            Head(font, heads[3], 117, (540,), "track", 858, 896),
            Head(font, heads[4], 117, (870,), "slam", 920, 966),
            Head(font, heads[5], 117, (150,), "soft", 984, 1036),
            Head(font, heads[6], 117, (540,), "pop", 1060, 1106),
            Head(font, heads[7], 117, (540,), "gate", 1134, 1176),
            Head(font, heads[8], 117, (170,), "hop", 1196, 1246),
            Head(font, heads[9], 117, (170,), "meet", 1268, 1316),
        ]
        self.lights = [
            (654, 736, glow((46, 26, 12), .5, .45)), (733, 856, glow((52, 20, 26), .5, .5)),
            (838, 926, glow((58, 18, 10), .5, .5)), (904, 990, glow((12, 34, 74), .3, .4)),
            (966, 1064, glow((24, 30, 56), .5, .7)), (1042, 1130, glow((16, 42, 58), .5, .5)),
            (1112, 1200, glow((16, 8, 50), .5, .5)), (1180, 1272, glow((60, 44, 18), .5, .4)),
            (1250, 1345, glow((64, 24, 34), .5, .5)),
        ]
        print("scenes ready", flush=True)

    def frame(self, n):
        m = n - OPEN
        out = self.back * float(smooth(656, 672, m))
        for a, b, light in self.lights:
            k = ease((m - a) / 24) * (1 - ease((m - b + 22) / 22))
            if k > 0:
                out = out + light * k
        if n <= LOGO_OUT:
            self.wall.draw(out, n)
        if 654 <= m <= 812:
            self.grid.draw(out, m)
        if 788 <= m <= 864:
            self.rows.draw(out, m)
        for a, b, layer in self.layers:
            if a <= m <= b:
                layer.draw(out, m)
        if LOGO_IN <= n <= LOGO_OUT:
            # A slow push that lands on the closing logo's size as the fade starts.
            scale = LOGO_SCALE * (1 - LOGO_PUSH * (1 - clamp((n - LOGO_IN) / (LOGO_LEAVE - LOGO_IN))))
            lockup(out, self.mark, self.name, n, LOGO_IN, LOGO_LEAVE, LOGO_OUT, scale=scale, y=CY - 40)
        for head in self.heads:
            head.draw(out, m)
        return np.clip(out * self.foot, 0, 255).astype(np.uint8)
