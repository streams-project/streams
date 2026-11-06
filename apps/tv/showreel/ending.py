#!/usr/bin/env python3
"""The end of the showreel, after the last headline. A frame number may be a fraction.

Change 14: each screen has its own logic, its own content and its own way of naming its platforms.
  television  grows around the card that came from the last headline, names roll beneath it
  laptop      rises closed, the lid opens, the systems line up beside it
  tablet      arrives upright and turns on its side, a shelf of posters runs across it
  phones      one from each side, each scrolling its own feed
  browser     a window opens, a pointer presses play
Change 15: the television comes back and fills the picture, the names pour in, the floor tilts and
they run off it, then the ask, a word at a time, and the mark with the name sliding out of it.
Parts start on bar lines of the music. film.py renders this and scenes.py as one film.
"""
import math, random
import cv2, numpy as np, pymunk
from PIL import Image, ImageDraw
from render import backdrop, foot, smooth
from stage import (CX, CY, VH, VW, WHITE, Card, clamp, cover, ease, fall, glow, lay, lay_card, lockup, rise, rounded,
                   split_lockup, spring, word)

N0 = 1345
RETURN, DROP, ASK, MARK, N_END = 1596, 1666, 1694, 1736, 1806
BACK = RETURN - 10
STEP = 0.4  # film frames between drawn frames, 60 a second
TV_Y = 490.0
BLUE, SALMON = (255, 120, 60), (170, 150, 250)
SHELL, EDGE = (38, 30, 28), (120, 100, 92)
TV_NAMES = ["Apple TV", "Samsung", "LG", "Fire TV", "Roku", "Android TV"]
ALL_NAMES = TV_NAMES + ["Mac", "Windows", "Linux", "iPad", "Android tablets", "iPhone", "Android", "Any browser"]

def shell(w, h, pad, radius, top=0):
    """A plain dark body with a thin light rim, leaving a screen of w by h."""
    W, H = w + 2 * pad, h + 2 * pad + top
    outer = rounded(W, H, radius)
    inner = np.zeros_like(outer)
    inner[2:-2, 2:-2] = rounded(W - 4, H - 4, radius - 2)
    rim = np.clip(outer - inner, 0, 1)
    pic = np.zeros((H, W, 4), np.float32)
    for k in range(3):
        pic[..., k] = outer * SHELL[k] + rim * (EDGE[k] - SHELL[k])
    pic[..., 3] = outer * 255
    return pic

def inset(pic, screen, pad, top, radius):
    """A screen laid into a body."""
    h, w = screen.shape[:2]
    m = rounded(w, h, radius)[..., None]
    out = pic.copy()
    out[pad + top:pad + top + h, pad:pad + w, :3] = out[pad + top:pad + top + h, pad:pad + w, :3] * (1 - m) + screen * m
    return out

def stand(pic, W, body):
    """The foot of a television, under a body."""
    H = body + 40
    out = np.zeros((H, W, 4), np.float32)
    out[:body] = pic
    im = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((W * .44, body - 2, W * .56, body + 18), 4, fill=255)
    d.rounded_rectangle((W * .34, body + 18, W * .66, body + 30), 6, fill=255)
    a = np.asarray(im, np.float32) / 255
    for k in range(3):
        out[..., k] += a * EDGE[k] * .8
    out[..., 3] = np.maximum(out[..., 3], a * 255)
    return out

class Pour:
    """Names and marks as bodies on a floor that tilts, stepped ahead so a frame only reads positions."""
    def __init__(self, items, seed, spawn, width, gravity, elastic, first):
        self.items, self.first = items, first
        space = pymunk.Space()
        space.gravity = (0, gravity)
        space.iterations = 40
        floor = pymunk.Body(body_type=pymunk.Body.KINEMATIC)
        floor.position = (-30, VH + 24)
        plate = pymunk.Segment(floor, (0, 0), (VW + 400, 0), 24)
        plate.elasticity, plate.friction = elastic, .12
        left = pymunk.Segment(space.static_body, (-24, -4000), (-24, VH + 24), 24)
        right = pymunk.Segment(space.static_body, (VW + 24, -4000), (VW + 24, VH + 24), 24)
        for w in (left, right):
            w.elasticity, w.friction = .4, .4
        space.add(floor, plate, left, right)
        r = random.Random(seed)
        bodies = []
        self.track = []
        n = float(first)
        tipped = False
        while n <= ASK + 34:
            due = int(len(items) * min(1.0, (n - first) / spawn) ** .8)
            while len(bodies) < due:
                h, w = items[len(bodies)].shape[:2]
                body = pymunk.Body(1, pymunk.moment_for_box(1, (w, h)))
                # They left to the left, so they come back from that side, over the top.
                body.position = (r.uniform(w / 2 + 10, VW * width), -h - r.uniform(0, 320))
                body.angle = r.uniform(-.6, .6)
                body.angular_velocity = r.uniform(-4.5, 4.5)
                body.velocity = (r.uniform(700, 1900), r.uniform(300, 800))
                shape = pymunk.Poly.create_box(body, (w * .97, h * .92))
                shape.elasticity, shape.friction = .4, .28
                space.add(body, shape)
                bodies.append(body)
            if n >= DROP and not tipped:
                tipped = True
                space.remove(right)
                floor.angular_velocity = 1.7
                space.gravity = (0, gravity * 2.2)
            if tipped and floor.angle > 1.05:
                floor.angular_velocity = 0
            for _ in range(4):
                space.step(STEP / 24 / 4)
            self.track.append([(b.position.x, b.position.y, b.angle) for b in bodies])
            n += STEP

    def draw(self, out, n, alpha=1.0, light=1.0, sigma=0.0, smear=True):
        i = int(round((n - self.first) / STEP))
        if i < 0:
            return
        i = min(i, len(self.track) - 1)
        now = self.track[i]
        before = self.track[max(i - 1, 0)]
        for k, pose in enumerate(now):
            x, y, a = pose
            d = math.degrees(a)
            poses = [(x, y, 1.0, 1.0, d)]
            if smear and k < len(before):
                bx, by, ba = before[k]
                if math.hypot(x - bx, y - by) > 7:
                    poses = [(bx + (x - bx) * t, by + (y - by) * t, 1.0, 1.0, math.degrees(ba + (a - ba) * t)) for t in (0, .33, .66, 1)]
            lay(out, self.items[k], poses, alpha=alpha, light=light, sigma=sigma)

class Ending:
    def __init__(self, font, logo, mark, name, art, keep_image):
        self.back, self.foot = backdrop(), foot()
        self.light = glow((46, 22, 14), .5, .46, .34, .34)
        self.font, self.art = font, art
        self.logo_mark, self.logo_word = split_lockup(logo)
        self.head = word(font, "On every screen", 92)
        self.keep = cover(keep_image, 960, 540).astype(np.float32)
        self.keep_full = cover(keep_image, VW, VH).astype(np.float32)
        self.build_tv()
        self.build_laptop()
        self.build_tablet()
        self.build_phones()
        self.build_browser()
        self.build_pour(mark)
        self.ask_words = [word(font, "Try", 210), word(font, name, 210, ((255, 107, 47), (255, 170, 120))), word(font, "today.", 210)]

    # ---- television
    def build_tv(self):
        self.tv_names = [word(self.font, n, 78) for n in TV_NAMES]
        body = shell(960, 540, 16, 20)
        self.tv_body_h, self.tv_w = body.shape[0], body.shape[1]
        self.tv_shell = stand(body, body.shape[1], body.shape[0])
        self.tv_screen = Card(self.keep.astype(np.uint8), 960, 540, 16)
        self.tv_mask = rounded(960, 540, 16)
        self.tv_out = 1436

    def television(self, out, n):
        if n >= self.tv_out + 20:
            return
        away = ease((n - self.tv_out) / 18)
        x = CX - 1500 * away
        arrive = ease((n - N0) / 24)
        zoom = 1 + .05 * ease((n - N0) / 90)
        y = TV_Y
        shell_y = y + 20
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
        """The film on the television: a slow push in, and the bar that says it is playing."""
        z = 1 + .1 * ease((n - N0) / 110)
        M = np.array([[z, 0, 480 - z * 480], [0, z, 300 - z * 300]], np.float64)
        rgb = cv2.warpAffine(self.keep, M, (960, 540), flags=cv2.INTER_LINEAR)
        show = ease((n - N0 - 8) / 12)
        if show > 0:
            bar = rgb[510:516, 36:924]
            bar[:] = bar * (1 - .55 * show) + 170 * .55 * show
            done = int(888 * clamp((n - N0) / 120))
            rgb[510:516, 36:36 + done] = rgb[510:516, 36:36 + done] * (1 - show) + np.array(BLUE, np.float32) * show
        a = self.tv_mask
        return np.dstack([rgb * a[..., None], a * 255])

    # ---- laptop
    def build_laptop(self):
        w, h = 820, 512
        screen = np.zeros((h, w, 3), np.float32) + (30, 22, 20)
        for r in range(3):
            for c in range(3):
                t = cover(self.art.tile(self.art.next("small")), 254, 143).astype(np.float32)
                x, y = 14 + c * 266, 70 + r * 146
                screen[y:y + 143, x:x + 254] = t
        for i in range(4):
            cv2.rectangle(screen, (20 + i * 84, 28), (84 + i * 84, 38), (120, 110, 104), -1, cv2.LINE_AA)
        self.lap_pic = inset(shell(w, h, 14, 16), screen, 14, 0, 8)
        self.lap_names = [word(self.font, n, 104) for n in ("Mac", "Windows", "Linux")]
        self.lap_in, self.lap_out = 1442, 1487

    def laptop(self, out, n):
        a, b = self.lap_in, self.lap_out
        if n < a or n > b + 16:
            return
        up = rise((n - a) / 14)
        down = fall((n - b - 4) / 12)
        cx, hinge = 690.0, 770 + 560 * (1 - up) + 560 * down
        opened = ease((n - a - 8) / 20) * (1 - ease((n - b + 8) / 11))
        pic = self.lap_pic
        h0, w0 = pic.shape[:2]
        angle = (1 - opened) * math.radians(92)
        lift, inset_x = h0 * math.cos(angle), 34 * math.sin(angle)
        src = np.float32([[0, 0], [w0, 0], [w0, h0], [0, h0]])
        if lift > 2:
            dst = np.float32([[cx - w0 / 2 + inset_x, hinge - lift], [cx + w0 / 2 - inset_x, hinge - lift], [cx + w0 / 2, hinge], [cx - w0 / 2, hinge]])
            M = cv2.getPerspectiveTransform(src, dst)
            p = cv2.warpPerspective(pic, M, (VW, VH), flags=cv2.INTER_LINEAR)
            k = p[..., 3:4] / 255
            out[:] = out * (1 - k) + p[..., :3] * (1 - .35 * math.sin(angle))
        base_w = int(w0 * 1.12)
        y0 = int(hinge)
        if y0 < VH:
            cv2.rectangle(out, (int(cx - base_w / 2), y0), (int(cx + base_w / 2), min(y0 + 20, VH - 1)), tuple(float(v) * .9 for v in EDGE), -1, cv2.LINE_AA)
        for i, pic_name in enumerate(self.lap_names):
            t = (n - a - 20 - i * 8) / 18
            l = (n - b + 12 - (2 - i) * 2) / 11
            x = 1530 + 500 * (1 - rise(t)) + 700 * fall(l)
            lay(out, pic_name, [(x, 320 + i * 150, 1.0, 1.0, 0.0)], alpha=ease(t * 1.5) * (1 - ease(l)))

    # ---- tablet
    def build_tablet(self):
        h = 506
        strip = np.zeros((h, 1560, 3), np.float32) + (30, 22, 20)
        for i in range(6):
            t = cover(self.art.poster(self.art.next("drama")), 236, 354).astype(np.float32)
            strip[76:430, 24 + i * 252:260 + i * 252] = t
        self.tab_strip = strip
        self.tab_body = shell(680, h, 18, 34)
        self.tab_names = [word(self.font, n, 92) for n in ("iPad", "Android tablets")]
        self.tab_in, self.tab_out = 1496, 1532

    def tablet(self, out, n):
        a, b = self.tab_in, self.tab_out
        if n < a or n > b + 20:
            return
        pan = 90 + 4.4 * (n - a)
        M = np.array([[1, 0, -pan], [0, 1, 0]], np.float64)
        screen = cv2.warpAffine(self.tab_strip, M, (680, 506), flags=cv2.INTER_LINEAR)
        pic = inset(self.tab_body, screen, 18, 0, 18)
        arrive = rise((n - a) / 13)
        turn = spring((n - a - 9) / 24, turns=1.1)
        away = fall((n - b) / 16)
        x = 1230 + 1300 * (1 - arrive) + 1500 * away
        lay(out, pic, [(x, 480, .9, .9, -90 * (1 - turn) + 26 * away)])
        for i, p in enumerate(self.tab_names):
            t = (n - a - 12 - i * 7) / 16
            l = (n - b + 3 - i * 2) / 11
            px = 450 - 300 * (1 - rise(t))
            lay(out, p, [(px, 390 + i * 130, 1.0, 1.0, 0.0)], alpha=ease(t * 1.6) * (1 - ease(l)))

    # ---- phones
    def build_phones(self):
        self.feeds = []
        for pool in ("comedy", "kids"):
            feed = np.zeros((2400, 240, 3), np.float32) + (30, 22, 20)
            for i in range(6):
                t = cover(self.art.tile(self.art.next(pool)), 224, 330).astype(np.float32)
                feed[10 + i * 346:340 + i * 346, 8:232] = t
            self.feeds.append(feed)
        self.phone_body = shell(240, 520, 12, 40)
        self.phone_names = [word(self.font, n, 92) for n in ("iPhone", "Android")]
        self.ph_in, self.ph_out = 1532, 1558

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
            screen = cv2.warpAffine(self.feeds[i], M, (240, 520), flags=cv2.INTER_LINEAR)
            pic = inset(self.phone_body, screen, 12, 0, 30)
            y = 480 + 900 * (1 - rise(t)) + 900 * fall(l)
            deg = side * (3 + 14 * (1 - rise(t)))
            lay(out, pic, [(x0 + side * 40 * (1 - rise(t)), y, 1.0, 1.0, deg)])
            p = self.phone_names[i]
            lt = (n - a - 12 - i * 7) / 14
            lay(out, p, [(x0 + side * 410 + side * 200 * (1 - rise(lt)), 480, 1.0, 1.0, 0.0)], alpha=ease(lt * 1.6) * (1 - ease(l)))

    # ---- browser
    def build_browser(self):
        w, h = 900, 500
        page = cover(self.art.tile(self.art.next("wall")), w, h).astype(np.float32)
        page = page * np.linspace(1, .55, h, dtype=np.float32)[:, None, None]
        self.br_pic = shell(w, h, 12, 18, top=44)
        self.br_pic = inset(self.br_pic, page, 12, 44, 8)
        for i in range(3):
            cv2.circle(self.br_pic, (34 + i * 26, 34), 7, (120, 110, 104, 255), -1, cv2.LINE_AA)
        cv2.rectangle(self.br_pic, (150, 22), (760, 46), (60, 50, 46, 255), -1, cv2.LINE_AA)
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
        lay(out, self.br_pic, [(CX, cy, size * (1 - .15 * away), size * (1 - .15 * away), 0.0)], alpha=min(1.0, (n - a) / 4) * (1 - away))
        t = (n - a - 8) / 15
        x = 1600 + (CX - 1600) * ease(t)
        y = 880 + (cy + 30 - 880) * ease(t)
        click = n - a - 23
        ring = clamp(click / 7)
        if click > 0:
            r = int(16 + 70 * ring)
            layer = np.zeros((VH, VW, 3), np.float32)
            cv2.circle(layer, (int(CX), int(cy + 30)), r, WHITE, 4, cv2.LINE_AA)
            out[:] = out + layer * (1 - ring) * .8
            d = 1 - .12 * math.sin(clamp(click / 5) * math.pi)
        else:
            d = 1.0
        lay(out, self.arrow, [(x + 14, y + 22, d, d, 0.0)], alpha=(1 - away) * clamp((n - a - 4) / 4))
        nt = (n - a - 6) / 14
        lay(out, self.br_name, [(CX, 860 + 26 * (1 - rise(nt)), 1.0, 1.0, 0.0)], alpha=ease(nt * 1.5) * (1 - away))

    # ---- the return and the pour
    def build_pour(self, mark):
        r = random.Random(11)
        near = []
        for size in (172, 136, 108):
            for name in ALL_NAMES:
                colour = r.choices([WHITE, BLUE, SALMON], [.68, .22, .10])[0]
                near.append(word(self.font, name, size, colour))
        r.shuffle(near)
        marks = [cv2.resize(mark, (s, s), interpolation=cv2.INTER_AREA) for s in (200, 150, 250)]
        for i in range(9):
            near.insert(r.randrange(len(near)), marks[i % 3])
        far = [word(self.font, n, s, WHITE) for s in (70, 56) for n in ALL_NAMES]
        r.shuffle(far)
        self.far = Pour(far, 3, 50.0, .6, 1800, .3, RETURN + 8)
        self.near = Pour(near, 23, 46.0, .4, 2500, .5, RETURN + 14)

    def comes_back(self, out, n):
        """The television again, from the left, growing until its screen is the picture."""
        t = ease((n - BACK) / 48)
        s = 1 + (VW * 1.02 / 960 - 1) * t
        x = -600 + (CX + 600) * ease((n - BACK) / 38)
        y = TV_Y + (CY - TV_Y) * t
        fade = 1 - float(smooth(DROP, DROP + 20, n))
        dim = 1 - .55 * ease((n - BACK - 36) / 30)
        lay(out, self.tv_screen.pic, [(x, y, s, s, 0.0)], alpha=fade, light=dim)

    # ---- the ask and the mark
    def ask(self, out, n):
        a, b, c = self.ask_words
        h = a.shape[0]
        width = a.shape[1] + 12 + b.shape[1]
        left = (VW - width) / 2
        y1, y2 = CY - h * .55 - 40, CY + h * .55 - 40
        leave = ease((n - (MARK - 12)) / 12)
        t = rise((n - ASK) / 20)
        lay(out, a, [(left + a.shape[1] / 2 , y1 + 44 * (1 - t) - 12 * leave, 1.0, 1.0, 0.0)], alpha=min(1.0, t * 1.6) * (1 - leave))
        t = rise((n - ASK - 7) / 20)
        off = 260 * (1 - t)
        lay(out, b, [(left + a.shape[1] + 12 + b.shape[1] / 2 + off, y1 - 12 * leave, 1.0, 1.0, 0.0)], alpha=min(1.0, t * 1.6) * (1 - leave),
            sigma=10 * (1 - t))
        t = spring((n - ASK - 15) / 20, turns=1.1)
        lay(out, c, [(CX, y2 - 12 * leave, t, t, 0.0)], alpha=min(1.0, clamp((n - ASK - 15) / 6)) * (1 - leave))

    def frame(self, n):
        out = self.back * (1 - float(smooth(DROP, DROP + 20, n)))
        if n < RETURN:
            spot = ease((n - N0) / 30) * (1 - ease((n - (RETURN - 16)) / 16))
            out = out + self.light * spot
            self.television(out, n)
            self.laptop(out, n)
            self.tablet(out, n)
            self.phones(out, n)
            self.browser(out, n)
            arrive = rise((n - (N0 + 6)) / 24)
            leave = ease((n - (RETURN - 14)) / 14)
            lay(out, self.head, [(CX, 104 + 22 * (1 - arrive) - 16 * leave, 1.0, 1.0, 0.0)], alpha=arrive * (1 - leave))
        if BACK <= n < DROP + 22:
            self.comes_back(out, n)
        if n >= RETURN + 8 and n < ASK + 34:
            self.far.draw(out, n, alpha=.55, light=.8, sigma=2.5, smear=False)
            self.near.draw(out, n)
        if ASK <= n < MARK:
            self.ask(out, n)
        if n >= MARK:
            lockup(out, self.logo_mark, self.logo_word, n, MARK, N_END - 16, N_END - 2, scale=1.32, y=CY - 40)
        return np.clip(out * self.foot, 0, 255).astype(np.uint8)

def load(path):
    im = cv2.imread(str(path), cv2.IMREAD_UNCHANGED).astype(np.float32)
    im[..., :3] *= im[..., 3:4] / 255
    return im
