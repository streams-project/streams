#!/usr/bin/env python3
"""The end of the showreel, after the last headline. Frame numbers carry on from render.py.

Every part starts on a bar line of the music, a bar being 70 frames from frame 896.
  names   platform names ride a belt, right to centre to left, each quicker than the last
  pile    the names come back with real physics and fill the screen
  drop    the floor goes, and the pile falls away
  ask     the call to action on black
  mark    the lockup, then black
The name in the call to action comes from brand.json, so a copy of this repository says its own.
"""
import argparse, json, math, random, subprocess
from pathlib import Path
import cv2, numpy as np, pymunk
from PIL import Image, ImageDraw, ImageFont
from render import VH, VW, backdrop, foot, smooth, table

BAR = 70
N0 = 1340
NAMES_IN, NAMES_ARRIVE, PILE, DROP, ASK, MARK, N_END = 1351, 1386, 1526, 1596, 1618, 1666, 1736
SUB = 4  # samples per frame on anything moving
# On our chosen list of platforms, in the order they pass.
NAMES = ["Apple TV", "Samsung", "LG", "Fire TV", "Android TV", "Roku", "iPhone", "iPad", "Android",
         "Mac", "Windows", "Linux", "Web"]
WHITE, BLUE, SALMON = (248, 248, 248), (255, 120, 60), (170, 150, 250)  # BGR

def sprite(font_path, text, size, colour):
    """A word as a tight premultiplied BGRA picture, with the headlines' weight and tracking."""
    font = ImageFont.truetype(font_path, size)
    font.set_variation_by_axes([800])
    text = text.upper()
    track = -0.014 * size
    probe = ImageDraw.Draw(Image.new("L", (8, 8)))
    width = sum(probe.textlength(c, font=font) for c in text) + track * (len(text) - 1)
    top, bottom = probe.textbbox((0, 0), "H", font=font)[1], probe.textbbox((0, 0), "H", font=font)[3]
    pad = int(size * 0.1)
    w, h = int(width) + 2 * pad, bottom - top + 2 * pad
    im = Image.new("L", (w, h), 0)
    d, x = ImageDraw.Draw(im), pad
    for c in text:
        d.text((x, pad - top), c, font=font, fill=255)
        x += d.textlength(c, font=font) + track
    a = np.asarray(im, np.float32) / 255
    return np.dstack([a * colour[0], a * colour[1], a * colour[2], a * 255])

def place(canvas, pic, poses, scale=1.0):
    """Lay a premultiplied picture at each (x, y, rot), averaged, which is motion blur when they differ."""
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
    held /= len(poses)
    k = held[..., 3:4] / 255
    canvas[y0:y1, x0:x1] = canvas[y0:y1, x0:x1] * (1 - k) + held[..., :3]

def overshoot(t):
    """0 to 1 with one soft swing past the end, for a word arriving with weight."""
    t = min(max(t, 0.0), 1.0)
    return 1 - math.exp(-6.5 * t) * math.cos(8.5 * t)

def belt_stops():
    """Moves and holds of the belt the names ride. Each is shorter than the one before, and the
    last name has left by the start of the pile."""
    holds = [24 * .68 ** i for i in range(len(NAMES))]
    moves = [15 * .86 ** i for i in range(len(NAMES))]
    fit = (PILE - 3 - NAMES_ARRIVE) / (sum(holds) + sum(moves))
    return [h * fit for h in holds], [m * fit for m in moves]

def spawn_list(font):
    """What falls: every name in several sizes and the mark, shuffled the same way each run."""
    r = random.Random(11)
    items = []
    for round_no, size in enumerate([168, 132, 104, 84]):
        for name in NAMES:
            colour = r.choices([WHITE, BLUE, SALMON], [.66, .22, .12])[0]
            items.append(("word", sprite(font, name, size, colour)))
    r.shuffle(items)
    return items

class Pile:
    """All bodies, stepped ahead of time, so a frame only reads positions."""
    def __init__(self, font, mark):
        self.items = spawn_list(font)
        marks = [cv2.resize(mark, (s, s), interpolation=cv2.INTER_AREA) for s in (190, 150, 240)]
        r = random.Random(5)
        for i in range(9):
            self.items.insert(r.randrange(len(self.items)), ("mark", marks[i % 3]))
        space = pymunk.Space()
        space.gravity = (0, 3200)
        space.iterations = 30
        walls = [pymunk.Segment(space.static_body, a, b, 24) for a, b in
                 [((-24, VH + 24), (VW + 24, VH + 24)), ((-24, -4000), (-24, VH + 24)), ((VW + 24, -4000), (VW + 24, VH + 24))]]
        for w in walls:
            w.elasticity, w.friction = .35, .7
            space.add(w)
        floor = walls[0]
        bodies = []
        r = random.Random(23)
        steps_per_frame = 8
        self.track = {}
        count = len(self.items)
        for n in range(PILE, N_END + 1):
            due = int(count * min(1.0, (n - PILE) / 58.0) ** .8)
            while len(bodies) < due:
                _, pic = self.items[len(bodies)]
                h, w = pic.shape[:2]
                body = pymunk.Body(1, pymunk.moment_for_box(1, (w, h)))
                # They left to the left, so they come back from that side, over the top.
                body.position = (r.uniform(w / 2 + 10, VW * .34), -h - r.uniform(0, 300))
                body.angle = r.uniform(-.6, .6)
                body.angular_velocity = r.uniform(-6, 6)
                body.velocity = (r.uniform(900, 2300), r.uniform(500, 1100))
                shape = pymunk.Poly.create_box(body, (w * .97, h * .92))
                shape.elasticity, shape.friction = .42, .55
                space.add(body, shape)
                bodies.append(body)
            if n == DROP:
                for w in walls:
                    space.remove(w)
                space.gravity = (0, 5200)
                for b in bodies:
                    b.apply_impulse_at_local_point((r.uniform(-260, 260), r.uniform(-420, -60)))
                    b.angular_velocity += r.uniform(-3, 3)
            frame = []
            for _ in range(SUB):
                for _ in range(steps_per_frame // SUB):
                    space.step(1 / 24 / steps_per_frame)
                frame.append([(b.position.x, b.position.y, b.angle) for b in bodies])
            self.track[n] = frame

class Ending:
    def __init__(self, font, lockup, mark, name):
        self.back = backdrop()
        self.foot = foot()
        self.font = font
        self.words = [sprite(font, n, 190, WHITE) for n in NAMES]
        gap = 170
        self.seats, at = [], 0.0
        for i, pic in enumerate(self.words):
            if i:
                at += self.words[i - 1].shape[1] / 2 + gap + pic.shape[1] / 2
            self.seats.append(at)
        self.stops = belt_stops()
        self.pile = Pile(font, mark)
        self.try_line = sprite(font, "Try", 190, WHITE)
        self.name_line = sprite(font, name, 190, BLUE)
        self.today = sprite(font, "today.", 190, WHITE)
        self.lockup = lockup

    def belt(self, f):
        """How far the belt has run at frame f. Each move lands with a little swing."""
        if f < NAMES_ARRIVE:
            return (overshoot((f - NAMES_IN) / (NAMES_ARRIVE - NAMES_IN)) - 1) * (VW / 2 + self.words[0].shape[1] / 2 + 40)
        holds, moves = self.stops
        at = float(NAMES_ARRIVE)
        for i, (hold, move) in enumerate(zip(holds, moves)):
            if f < at + hold:
                return self.seats[i]
            at += hold
            if f < at + move:
                u = (f - at) / move
                if i + 1 < len(self.seats):
                    return self.seats[i] + (self.seats[i + 1] - self.seats[i]) * overshoot(u)
                return self.seats[i] + (VW + 300) * u * u
            at += move
        return self.seats[-1] + VW + 300

    def names(self, out, n):
        samples = [self.belt(n + s / SUB) for s in range(SUB)]
        for pic, seat in zip(self.words, self.seats):
            poses = [(VW / 2 + seat - run, VH / 2, 0.0) for run in samples]
            place(out, pic, poses)

    def falling(self, out, n):
        if n < PILE or n > ASK + 26:
            return
        moving = self.pile.track[n]
        for i, (kind, pic) in enumerate(self.pile.items):
            if i >= len(moving[-1]):
                break
            place(out, pic, [sample[i] for sample in moving if i < len(sample)])

    def ask(self, out, n):
        if n < ASK:
            return
        k = float(smooth(ASK, ASK + 5, n))
        left = 240
        a, b, c = self.try_line, self.name_line, self.today
        h = a.shape[0]
        lift = 22 * (1 - overshoot((n - ASK) / 12))
        y1, y2 = VH / 2 - h * .62 + lift, VH / 2 + h * .62 + lift
        place(out, a * k, [(left + a.shape[1] / 2, y1, 0.0)])
        place(out, b * k, [(left + a.shape[1] + 26 + b.shape[1] / 2, y1, 0.0)])
        place(out, c * k, [(left + c.shape[1] / 2, y2, 0.0)])

    def frame(self, n):
        out = np.zeros((VH, VW, 3), np.float32)
        glow = float(smooth(N0, N0 + 12, n)) * (1 - float(smooth(DROP, DROP + 14, n)))
        out += self.back * glow
        if NAMES_IN <= n < PILE:
            self.names(out, n)
        self.falling(out, n)
        if n < MARK:
            self.ask(out, n)
        else:
            a = float(smooth(MARK, MARK + 8, n)) * (1 - float(smooth(N_END - 14, N_END - 2, n)))
            place(out, self.lockup * a, [(VW / 2, VH / 2, 0.0)], scale=1 + .0009 * (n - MARK))
        return np.clip(out * self.foot, 0, 255).astype(np.uint8)

def load(path, width=None):
    im = cv2.imread(str(path), cv2.IMREAD_UNCHANGED).astype(np.float32)
    if width:
        h = int(im.shape[0] * width / im.shape[1])
        im = cv2.resize(im, (width, h), interpolation=cv2.INTER_AREA)
    im[..., :3] *= im[..., 3:4] / 255
    return im

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--font", required=True)
    ap.add_argument("--logo", required=True)
    ap.add_argument("--mark", required=True)
    ap.add_argument("--brand", required=True)
    ap.add_argument("--out", default="")
    ap.add_argument("--ffmpeg", default="")
    ap.add_argument("--stills", default="")
    ap.add_argument("--still-dir", default="")
    a = ap.parse_args()
    name = json.loads(Path(a.brand).read_text())["name"]
    film = Ending(a.font, load(a.logo), load(a.mark), name)
    wanted = [int(x) for x in a.stills.split(",") if x]
    if wanted:
        Path(a.still_dir).mkdir(parents=True, exist_ok=True)
        for n in wanted:
            cv2.imwrite(f"{a.still_dir}/e{n:05d}.jpg", film.frame(n), [cv2.IMWRITE_JPEG_QUALITY, 90])
        print("stills", len(wanted))
        return
    p = subprocess.Popen([a.ffmpeg, "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "bgr24", "-s", f"{VW}x{VH}",
                          "-r", "24", "-i", "-", "-c:v", "libx264", "-crf", "15", "-preset", "medium",
                          "-pix_fmt", "yuv420p", a.out], stdin=subprocess.PIPE)
    for n in range(N0, N_END):
        p.stdin.write(film.frame(n).tobytes())
    p.stdin.close()
    p.wait()
    print("wrote", a.out)

if __name__ == "__main__":
    main()
