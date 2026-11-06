"""Drawing for the showreel: one card at a time, each at its own place, depth, focus and light.

Positions are floats and go through warpAffine, so slow motion never steps from pixel to pixel.
"""
import math
import cv2, numpy as np

VW, VH = 1920, 1080
CX, CY = 960.0, 540.0

def clamp(t):
    return min(max(t, 0.0), 1.0)

def ease(t):
    """Soft at both ends, with no sudden change of speed."""
    t = clamp(t)
    return t * t * t * (t * (t * 6 - 15) + 10)

def rise(t):
    """Quick then settling."""
    return 1 - (1 - clamp(t)) ** 4

def fall(t):
    """Slow then quick, for something leaving."""
    return clamp(t) ** 3

def spring(t, swing=.16, turns=1.5):
    """Past the end and back, once or twice, and still."""
    t = clamp(t)
    return 1 - math.exp(-5.2 * t) * math.cos(turns * 2 * math.pi * t) * (1 - t) ** .3 if t < 1 else 1.0

def bounce(t):
    """A fall to the floor with two smaller bounces. 1 is on the floor."""
    t = clamp(t)
    if t < .55:
        return (t / .55) ** 2
    if t < .8:
        u = (t - .55) / .25
        return 1 - .17 * 4 * u * (1 - u)
    u = (t - .8) / .2
    return 1 - .045 * 4 * u * (1 - u)

def rounded(w, h, r):
    s = 4
    w4, h4, R = w * s, h * s, int(r * s)
    m = np.zeros((h4, w4), np.uint8)
    cv2.rectangle(m, (R, 0), (w4 - R, h4), 255, -1)
    cv2.rectangle(m, (0, R), (w4, h4 - R), 255, -1)
    for c in ((R, R), (w4 - R, R), (R, h4 - R), (w4 - R, h4 - R)):
        cv2.circle(m, c, R, 255, -1, cv2.LINE_AA)
    return cv2.resize(m, (w, h), interpolation=cv2.INTER_AREA).astype(np.float32) / 255

def cover(img, w, h):
    """Crop a picture to this shape and size."""
    ih, iw = img.shape[:2]
    want = w / h
    if iw / ih > want:
        cw = int(ih * want)
        x = (iw - cw) // 2
        img = img[:, x:x + cw]
    else:
        ch = int(iw / want)
        y = (ih - ch) // 3
        img = img[y:y + ch]
    return cv2.resize(img, (w, h), interpolation=cv2.INTER_AREA)

class Card:
    """A picture with rounded corners, a faint top edge and a soft shadow."""
    def __init__(self, img, w, h, radius=None):
        self.w, self.h = w, h
        r = radius if radius is not None else max(8, 14 / 432 * w)
        a = rounded(w, h, r)
        t = cover(img, w, h).astype(np.float32)
        edge = np.clip(a - np.roll(a, 2, axis=0), 0, 1)[..., None] * 26
        self.pic = np.dstack([(t + edge) * a[..., None], a * 255]).astype(np.float32)
        pad = 40
        s = np.zeros((h + 2 * pad, w + 2 * pad, 4), np.float32)
        s[pad:pad + h, pad:pad + w, 3] = a * 255
        self.shadow = cv2.GaussianBlur(s, (0, 0), 20)

def blurred(pic, sigma):
    """A soft copy, with room around it for the blur to spread. sigma is in the picture's pixels."""
    if sigma < .4:
        return pic, 1.0
    shrink = 1.0
    if sigma > 5:
        shrink = 4.0
        pic = cv2.resize(pic, None, fx=.25, fy=.25, interpolation=cv2.INTER_AREA)
        sigma /= 4
    m = int(3 * sigma) + 2
    pic = cv2.copyMakeBorder(pic, m, m, m, m, cv2.BORDER_CONSTANT, value=0)
    return cv2.GaussianBlur(pic, (0, 0), sigma), shrink

def lay(canvas, pic, poses, alpha=1.0, light=1.0, sigma=0.0):
    """Lay a premultiplied BGRA picture at each pose, averaged, which is motion blur when they differ.
    A pose is (x, y, sx, sy, degrees), with x and y the centre on screen."""
    if alpha <= .003:
        return
    biggest = max(max(abs(p[2]), abs(p[3])) for p in poses)
    if biggest <= .002:
        return
    src, shrink = blurred(pic, sigma / biggest)
    h, w = src.shape[:2]
    reach = math.hypot(w, h) * biggest * shrink / 2 + 3
    x0 = max(int(min(p[0] for p in poses) - reach), 0)
    y0 = max(int(min(p[1] for p in poses) - reach), 0)
    x1 = min(int(max(p[0] for p in poses) + reach), VW)
    y1 = min(int(max(p[1] for p in poses) + reach), VH)
    if x1 <= x0 or y1 <= y0:
        return
    held = np.zeros((y1 - y0, x1 - x0, 4), np.float32)
    for x, y, sx, sy, deg in poses:
        c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
        a, b = sx * shrink * c, -sy * shrink * s
        d, e = sx * shrink * s, sy * shrink * c
        M = np.array([[a, b, x - x0 - a * w / 2 - b * h / 2], [d, e, y - y0 - d * w / 2 - e * h / 2]], np.float64)
        flags = cv2.INTER_AREA if biggest * shrink < .6 else cv2.INTER_LINEAR
        held += cv2.warpAffine(src, M, (x1 - x0, y1 - y0), flags=cv2.INTER_LINEAR)
    held *= alpha / len(poses)
    k = held[..., 3:4] / 255
    canvas[y0:y1, x0:x1] = canvas[y0:y1, x0:x1] * (1 - k) + held[..., :3] * light

def lay_card(canvas, card, poses, alpha=1.0, light=1.0, sigma=0.0, shadow=.55):
    if shadow > 0:
        x, y, sx, sy, deg = poses[len(poses) // 2]
        lay(canvas, card.shadow, [(x, y + 16 * abs(sy), sx, sy, deg)], alpha=alpha * shadow)
    lay(canvas, card.pic, poses, alpha=alpha, light=light, sigma=sigma)

def project(x, y, z):
    """A point at depth z. 0 is the rest plane, negative is nearer the viewer."""
    s = 1 / max(1 + z, .05)
    return CX + (x - CX) * s, CY + (y - CY) * s, s

def glow(colour, x=.5, y=.5, rx=.7, ry=.7):
    """A soft pool of colour, in BGR, to add to the backdrop."""
    yy, xx = np.mgrid[0:VH // 4, 0:VW // 4].astype(np.float32)
    g = np.exp(-(((xx * 4 / VW - x) / rx) ** 2 + ((yy * 4 / VH - y) / ry) ** 2))
    small = np.dstack([g * colour[0], g * colour[1], g * colour[2]])
    return cv2.resize(small, (VW, VH), interpolation=cv2.INTER_LINEAR)

WHITE = (248, 248, 248)

def word(font_path, text, size, colour=WHITE, upper=True):
    """Text as a tight premultiplied BGRA picture, with the headlines' weight and tracking.
    Two colours make a soft sideways gradient."""
    from PIL import Image, ImageDraw, ImageFont
    font = ImageFont.truetype(font_path, size)
    font.set_variation_by_axes([800])
    text = text.upper() if upper else text
    track = -0.014 * size
    probe = ImageDraw.Draw(Image.new("L", (8, 8)))
    width = sum(probe.textlength(c, font=font) for c in text) + track * (len(text) - 1)
    box = probe.textbbox((0, 0), "H", font=font)
    pad = int(size * 0.14)
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

def split_lockup(logo):
    """The mark and the word of the lockup, from the gap between them."""
    filled = logo[..., 3].max(0) > 8
    third = logo.shape[1] // 8
    empty = np.where(~filled[third:logo.shape[1] // 2])[0]
    gap = empty[0] + third
    return logo[:, :gap], logo[:, gap:]

def lockup(out, mark, name, n, start, leave, end, scale=1.0, y=CY):
    """The mark arrives, then the name slides out from behind it. Never one turning into the other."""
    mw, ww = mark.shape[1] * scale, name.shape[1] * scale
    whole = mw + ww
    arrive = rise((n - start) / 16)
    slide = ease((n - start - 12) / 22)
    fade = 1 - ease((n - leave) / max(end - leave, 1))
    x = VW / 2 - (whole / 2 - mw / 2) * slide
    if slide > 0:
        layer = np.zeros((VH, VW, 3), np.float32)
        home = VW / 2 - whole / 2 + mw + ww / 2
        pos = home - (1 - slide) * (ww + mw * .3)
        lay(layer, name, [(pos, y, scale, scale, 0.0)])
        edge = int(x + mw * .28)
        ramp = np.clip((np.arange(VW, dtype=np.float32) - edge) / 26, 0, 1)[None, :, None]
        layer *= ramp
        k = (layer.max(2, keepdims=True) / 248).clip(0, 1) * fade
        out[:] = out * (1 - k) + layer * fade
    size = scale * (.8 + .2 * spring((n - start) / 22))
    lay(out, mark, [(x, y, size, size, 0.0)], alpha=min(1.0, arrive * 1.6) * fade)
