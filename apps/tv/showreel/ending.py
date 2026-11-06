#!/usr/bin/env python3
"""The end of the showreel, after the last headline. Frame numbers carry on from render.py, and a
frame number may be a fraction, because the film is drawn at 60 frames a second.

Every part starts on a bar line of the music, a bar being 70 frames from frame 896.
  screens  a television arrives, then a laptop, a tablet, a phone and a browser pass, each playing
           title art, with the platforms named under them
  return   the television comes back from the left and grows until it fills the picture, while the
           platform names tumble in over it with real physics
  drop     the floor goes, the names fall away and the art fades
  ask      the call to action, a word at a time
  mark     the play mark, then the name slides out from behind it
film.py renders this and render.py as one film. The name in the call to action comes from brand.json,
so a copy of this repository says its own.
"""
import math, random
import cv2, numpy as np, pymunk
from PIL import Image, ImageDraw, ImageFont
from render import VH, VW, backdrop, foot, smooth

BAR = 70
N0 = 1345
ARRIVE, RETURN, DROP, ASK, MARK, N_END = 1386, 1596, 1666, 1692, 1736, 1806
BACK = RETURN - 10  # the television starts back just before the bar line, and lands on the beat after it
STEP = 0.4  # film frames between drawn frames, 60 a second
WHITE, BLUE, SALMON = (248, 248, 248), (255, 120, 60), (170, 150, 250)  # BGR
GLASS, EDGE = (38, 30, 28), (120, 100, 92)
# Each screen, and the platforms on our chosen list it stands for.
SCREENS = [
    dict(kind="tv", w=960, h=540, names=["Apple TV", "Samsung", "LG", "Fire TV", "Roku", "Android TV"], stay=60, sheet=2),
    dict(kind="laptop", w=820, h=512, names=["Mac", "Windows", "Linux"], stay=40, sheet=3),
    dict(kind="tablet", w=680, h=510, names=["iPad", "Android tablets"], stay=28, sheet=5),
    dict(kind="phone", w=258, h=545, names=["iPhone", "Android"], stay=24, sheet=1),
    dict(kind="browser", w=900, h=506, names=["Any browser"], stay=18, sheet=0),
]
PITCH = 1240  # between screens on the belt
MOVE = 15

def ease(t):
    """Soft at both ends, with no sudden change of speed."""
    t = min(max(t, 0.0), 1.0)
    return t * t * t * (t * (t * 6 - 15) + 10)

def rise(t):
    """Quick then settling, for something arriving."""
    t = min(max(t, 0.0), 1.0)
    return 1 - (1 - t) ** 4

def rounded_mask(w, h, r):
    scale = 4
    im = Image.new("L", (w * scale, h * scale), 0)
    ImageDraw.Draw(im).rounded_rectangle((0, 0, w * scale - 1, h * scale - 1), r * scale, fill=255)
    return np.asarray(im.resize((w, h), Image.LANCZOS), np.float32) / 255

def word(font_path, text, size, colour):
    """A word as a tight premultiplied BGRA picture, with the headlines' weight and tracking.
    Two colours make a soft sideways gradient."""
    font = ImageFont.truetype(font_path, size)
    font.set_variation_by_axes([800])
    text = text.upper()
    track = -0.014 * size
    probe = ImageDraw.Draw(Image.new("L", (8, 8)))
    width = sum(probe.textlength(c, font=font) for c in text) + track * (len(text) - 1)
    box = probe.textbbox((0, 0), "H", font=font)
    pad = int(size * 0.1)
    w, h = int(width) + 2 * pad, box[3] - box[1] + 2 * pad
    im = Image.new("L", (w, h), 0)
    d, x = ImageDraw.Draw(im), pad
    for c in text:
        d.text((x, pad - box[1]), c, font=font, fill=255)
        x += d.textlength(c, font=font) + track
    a = np.asarray(im, np.float32) / 255
    if isinstance(colour[0], tuple):
        t = np.linspace(0, 1, w, dtype=np.float32)[None, :]
        rgb = [a * (colour[0][k] * (1 - t) + colour[1][k] * t) for k in range(3)]
    else:
        rgb = [a * colour[k] for k in range(3)]
    return np.dstack(rgb + [a * 255])

def place(canvas, pic, poses, scale=1.0, alpha=1.0):
    """Lay a premultiplied picture at each (x, y, rot), averaged, which is motion blur when they differ."""
    if alpha <= 0:
        return
    h, w = pic.shape[:2]
    reach = int(math.hypot(w, h) * scale / 2) + 3
    xs, ys = [p[0] for p in poses], [p[1] for p in poses]
    x0, y0 = max(int(min(xs)) - reach, 0), max(int(min(ys)) - reach, 0)
    x1, y1 = min(int(max(xs)) + reach, VW), min(int(max(ys)) + reach, VH)
    if x1 <= x0 or y1 <= y0:
        return
    held = np.zeros((y1 - y0, x1 - x0, 4), np.float32)
    for x, y, rot in poses:
        c, s = scale * math.cos(rot), scale * math.sin(rot)
        M = np.array([[c, -s, x - c * w / 2 + s * h / 2 - x0], [s, c, y - s * w / 2 - c * h / 2 - y0]], np.float64)
        held += cv2.warpAffine(pic, M, (x1 - x0, y1 - y0), flags=cv2.INTER_LINEAR)
    held *= alpha / len(poses)
    k = held[..., 3:4] / 255
    canvas[y0:y1, x0:x1] = canvas[y0:y1, x0:x1] * (1 - k) + held[..., :3]

class Device:
    """A plain drawn screen of one kind, with moving art in it."""
    def __init__(self, spec, sheet):
        self.spec, self.sheet = spec, sheet
        self.w, self.h, kind = spec["w"], spec["h"], spec["kind"]
        self.pad = 16 if kind != "phone" else 14
        self.top = 40 if kind == "browser" else 0
        self.foot = {"tv": 44, "laptop": 30}.get(kind, 0)
        W, H = self.w + 2 * self.pad, self.h + 2 * self.pad + self.top + self.foot
        self.W, self.H = W, H
        body = H - self.foot
        radius = {"phone": 44, "tablet": 30, "tv": 18, "laptop": 18, "browser": 18}[kind]
        shell = np.zeros((H, W, 4), np.float32)
        outer = rounded_mask(W, body, radius)
        inner = rounded_mask(W - 4, body - 4, radius - 2)
        shell[:body, :, 3] = outer * 255
        rim = outer.copy()
        rim[2:-2, 2:-2] -= inner
        rim = np.clip(rim, 0, 1)
        for k in range(3):
            shell[:body, :, k] = outer * GLASS[k] + rim * (EDGE[k] - GLASS[k])
        if kind == "tv":
            im = Image.new("L", (W, H), 0)
            ImageDraw.Draw(im).rounded_rectangle((W * .42, body + 2, W * .58, body + 16), 4, fill=255)
            ImageDraw.Draw(im).rounded_rectangle((W * .32, body + 22, W * .68, body + 32), 5, fill=255)
            self.add_flat(shell, im)
        if kind == "laptop":
            im = Image.new("L", (W, H), 0)
            ImageDraw.Draw(im).rounded_rectangle((-30, body - 2, W + 30, body + 18), 9, fill=255)
            self.add_flat(shell, im)
        if kind == "browser":
            for i, colour in enumerate([(120, 110, 104)] * 3):
                x, y = self.pad + 18 + i * 26, self.pad + 12
                cv2.circle(shell, (x, y), 7, (*colour, 255), -1, cv2.LINE_AA)
        self.shell = shell
        self.mask = rounded_mask(self.w, self.h, max(radius - 8, 6))

    @staticmethod
    def add_flat(shell, im):
        a = np.asarray(im, np.float32) / 255
        a = np.where(shell[..., 3] > 0, 0, a)
        for k in range(3):
            shell[..., k] += a * EDGE[k] * .7
        shell[..., 3] += a * 255

    def art(self, n, w, h):
        """The moving picture on this screen, a slow pan across a sheet of title art."""
        sheet = self.sheet
        sh, sw = sheet.shape[:2]
        zoom = 1.25 * w / 1920
        tx = w / 2 - zoom * (sw / 2 + 1.6 * (n - 1500) - 120)
        ty = h / 2 - zoom * (sh / 2 + .22 * (n - 1500))
        M = np.array([[zoom, 0, tx], [0, zoom, ty]], np.float64)
        return cv2.warpAffine(sheet, M, (w, h), flags=cv2.INTER_LINEAR)

    def picture(self, n):
        pic = self.shell.copy()
        art = self.art(n, self.w, self.h)
        base = np.zeros((self.h, self.w, 3), np.float32) + (28, 22, 20)
        k = art[..., 3:4] / 255
        screen = base * (1 - k) + art[..., :3]
        m = self.mask[..., None]
        y, x = self.pad + self.top, self.pad
        pic[y:y + self.h, x:x + self.w, :3] = pic[y:y + self.h, x:x + self.w, :3] * (1 - m) + screen * m
        return pic

class Pile:
    """The names and marks as bodies, stepped ahead of time, so a frame only reads positions."""
    def __init__(self, font, mark):
        r = random.Random(11)
        self.items = []
        names = [n for s in SCREENS for n in s["names"]]
        for size in (172, 136, 108, 88):
            for name in names:
                colour = r.choices([WHITE, BLUE, SALMON], [.68, .22, .10])[0]
                self.items.append(word(font, name, size, colour))
        r.shuffle(self.items)
        marks = [cv2.resize(mark, (s, s), interpolation=cv2.INTER_AREA) for s in (200, 150, 250)]
        r = random.Random(5)
        for i in range(10):
            self.items.insert(r.randrange(len(self.items)), marks[i % 3])
        space = pymunk.Space()
        space.gravity = (0, 2600)
        space.iterations = 40
        walls = [pymunk.Segment(space.static_body, a, b, 24) for a, b in
                 [((-24, VH + 24), (VW + 24, VH + 24)), ((-24, -4000), (-24, VH + 24)), ((VW + 24, -4000), (VW + 24, VH + 24))]]
        for w in walls:
            w.elasticity, w.friction = .45, .6
            space.add(w)
        bodies = []
        r = random.Random(23)
        self.first = RETURN + 16
        self.track = []
        count = len(self.items)
        steps = 4
        n = float(self.first)
        dropped = False
        while n <= ASK + 30:
            due = int(count * min(1.0, (n - self.first) / 44.0) ** .75)
            while len(bodies) < due:
                pic = self.items[len(bodies)]
                h, w = pic.shape[:2]
                body = pymunk.Body(1, pymunk.moment_for_box(1, (w, h)))
                # They left to the left, so they come back from that side, over the top.
                body.position = (r.uniform(w / 2 + 10, VW * .38), -h - r.uniform(0, 300))
                body.angle = r.uniform(-.6, .6)
                body.angular_velocity = r.uniform(-5, 5)
                body.velocity = (r.uniform(800, 2100), r.uniform(400, 900))
                shape = pymunk.Poly.create_box(body, (w * .97, h * .92))
                shape.elasticity, shape.friction = .48, .5
                space.add(body, shape)
                bodies.append(body)
            if n >= DROP and not dropped:
                dropped = True
                for w in walls:
                    space.remove(w)
                space.gravity = (0, 6500)
                for b in bodies:
                    b.velocity = (b.velocity.x + r.uniform(-240, 240), b.velocity.y + r.uniform(500, 900))
                    b.angular_velocity += r.uniform(-2.5, 2.5)
            for _ in range(steps):
                space.step(STEP / 24 / steps)
            self.track.append([(b.position.x, b.position.y, b.angle) for b in bodies])
            n += STEP

    def poses(self, n):
        i = int(round((n - self.first) / STEP))
        return self.track[max(0, min(i, len(self.track) - 1))] if i >= 0 else []

def split_lockup(logo):
    """The mark and the word of the lockup, from the gap between them."""
    filled = logo[..., 3].max(0) > 8
    empty = np.where(~filled[logo.shape[1] // 8:logo.shape[1] // 2])[0]
    gap = empty[0] + logo.shape[1] // 8
    return logo[:, :gap], logo[:, gap:]

class Ending:
    def __init__(self, font, logo, mark, name, sheets):
        self.back = backdrop()
        self.foot = foot()
        self.devices = [Device(s, sheets[s["sheet"]][0]) for s in SCREENS]
        yy, xx = np.mgrid[0:VH, 0:VW].astype(np.float32)
        glow = np.exp(-(((xx - VW / 2) / 620) ** 2 + ((yy - 500) / 340) ** 2))
        self.light = np.dstack([glow * 46, glow * 22, glow * 14])
        self.head = word(font, "On every screen", 92, WHITE)
        self.labels = [[word(font, n, 78, WHITE) for n in s["names"]] for s in SCREENS]
        self.pile = Pile(font, mark)
        self.ask_words = [word(font, "Try", 210, WHITE), word(font, name, 210, ((255, 107, 47), (255, 170, 120))),
                          word(font, "today.", 210, WHITE)]
        self.logo_mark, self.logo_word = split_lockup(logo)
        fit = (RETURN - 8 - ARRIVE) / (sum(s["stay"] for s in SCREENS) + MOVE * len(SCREENS))
        self.stays = [s["stay"] * fit for s in SCREENS]
        self.move = MOVE * fit

    @staticmethod
    def lit(n):
        return ease((n - N0) / 30) * (1 - ease((n - (RETURN - 16)) / 16))

    def belt(self, n):
        """Where the belt stands, in screens, and how long this screen has been resting."""
        if n < ARRIVE:
            return (ease((n - N0) / (ARRIVE - N0)) - 1) * 1.45, 0.0
        at = float(ARRIVE)
        for i, stay in enumerate(self.stays):
            if n < at + stay:
                return float(i), (n - at) / stay
            at += stay
            if n < at + self.move:
                return i + ease((n - at) / self.move), 1.0
            at += self.move
        return float(len(self.stays)), 1.0

    def screens(self, out, n):
        run, rest = self.belt(n)
        for i, device in enumerate(self.devices):
            away = i - run
            if abs(away) > 1.5:
                continue
            x = VW / 2 + away * PITCH
            near = 1 - min(abs(away), 1)
            out += self.light * ease(near) * self.lit(n)
            scale = .74 + .26 * ease(near)
            y = 500 + 26 * (1 - ease(near))
            place(out, device.picture(n), [(x, y, 0.0)], scale=scale, alpha=.45 + .55 * ease(near))
            self.label(out, i, x, near, rest if abs(away) < .5 else (1.0 if away < 0 else 0.0))

    def label(self, out, i, x, near, rest):
        names = self.labels[i]
        spot = rest * len(names)
        now = min(int(spot), len(names) - 1)
        turn = spot - now
        change = ease((turn - .78) / .22) if now < len(names) - 1 else 0.0
        y = 856
        k = ease(near)
        place(out, names[now], [(x, y - 44 * change, 0.0)], alpha=k * (1 - change))
        if change > 0:
            place(out, names[now + 1], [(x, y + 44 * (1 - change), 0.0)], alpha=k * change)

    def headline(self, out, n):
        arrive = rise((n - (N0 + 6)) / 26)
        leave = ease((n - (RETURN - 14)) / 14)
        place(out, self.head, [(VW / 2, 104 + 22 * (1 - arrive) - 16 * leave, 0.0)], alpha=arrive * (1 - leave))

    def comes_back(self, out, n):
        """The television again, from the left, growing until its screen is the picture."""
        tv = self.devices[0]
        t = ease((n - BACK) / 48)
        scale = 1 + (VW * 1.02 / tv.w - 1) * t
        w, h = int(tv.w * scale), int(tv.h * scale)
        x = -600 + (VW / 2 + 600) * ease((n - BACK) / 38)
        y = 500 + (VH / 2 - 500) * t
        art = tv.art(n, w, h)
        fade = 1 - float(smooth(DROP, DROP + 18, n))
        dim = 1 - .5 * ease((n - BACK - 36) / 30)
        ax, ay = x - w / 2, y - h / 2
        M = np.array([[1, 0, ax], [0, 1, ay]], np.float64)
        moved = cv2.warpAffine(art, M, (VW, VH), flags=cv2.INTER_LINEAR)
        k = moved[..., 3:4] / 255 * fade
        out[:] = out * (1 - k) + moved[..., :3] * dim * fade

    def falling(self, out, n):
        poses = self.pile.poses(n)
        for i, pose in enumerate(poses):
            place(out, self.pile.items[i], [pose])

    def ask(self, out, n):
        a, b, c = self.ask_words
        h = a.shape[0]
        width = a.shape[1] + 30 + b.shape[1]
        left = (VW - width) / 2
        y1, y2 = VH / 2 - h * .6 - 40, VH / 2 + h * .6 - 40
        leave = ease((n - (MARK - 12)) / 12)
        spots = [(left + a.shape[1] / 2, y1), (left + a.shape[1] + 30 + b.shape[1] / 2, y1), (VW / 2, y2)]
        for i, (pic, (x, y)) in enumerate(zip(self.ask_words, spots)):
            t = rise((n - ASK - i * 7) / 22)
            place(out, pic, [(x, y + 46 * (1 - t) - 14 * leave, 0.0)], alpha=min(1.0, t * 1.6) * (1 - leave))

    def mark(self, out, n):
        m, w = self.logo_mark, self.logo_word
        scale = 1.32
        mw, ww = m.shape[1] * scale, w.shape[1] * scale
        whole = mw + ww
        arrive = rise((n - MARK) / 18)
        slide = ease((n - MARK - 16) / 26)
        fade = 1 - ease((n - (N_END - 16)) / 14)
        centre_x = VW / 2 - (whole / 2 - mw / 2) * slide
        y = VH / 2 - 40
        size = scale * (.82 + .18 * arrive)
        if slide > 0:
            layer = np.zeros((VH, VW, 3), np.float32)
            home = VW / 2 - whole / 2 + mw + ww / 2
            x = home - (1 - slide) * (ww + mw * .3)
            place(layer, w, [(x, y, 0.0)], scale=scale)
            edge = int(centre_x + mw * .28)
            ramp = np.clip((np.arange(VW, dtype=np.float32) - edge) / 26, 0, 1)[None, :, None]
            layer *= ramp
            shown = layer.sum(2, keepdims=True) > 0
            out[:] = np.where(shown, out * (1 - fade * ramp * (layer.max(2, keepdims=True) / 248).clip(0, 1)) + layer * fade, out)
        place(out, m, [(centre_x, y, 0.0)], scale=size, alpha=min(1.0, arrive * 1.5) * fade)

    def frame(self, n):
        out = np.zeros((VH, VW, 3), np.float32)
        out += self.back * (1 - float(smooth(DROP, DROP + 18, n)))
        if n < RETURN:
            self.screens(out, n)
            self.headline(out, n)
        if BACK <= n < DROP + 20:
            self.comes_back(out, n)
        if n >= self.pile.first:
            self.falling(out, n)
        if ASK <= n < MARK:
            self.ask(out, n)
        if n >= MARK:
            self.mark(out, n)
        return np.clip(out * self.foot, 0, 255).astype(np.uint8)

def load(path, width=None):
    im = cv2.imread(str(path), cv2.IMREAD_UNCHANGED).astype(np.float32)
    if width:
        h = int(im.shape[0] * width / im.shape[1])
        im = cv2.resize(im, (width, h), interpolation=cv2.INTER_AREA)
    im[..., :3] *= im[..., 3:4] / 255
    return im
