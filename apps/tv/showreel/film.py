#!/usr/bin/env python3
"""The whole showreel, frame 590 to the last, rendered in one pass and encoded once, at 60 frames a
second. scenes.py draws the headlines and ending.py the end. The same frames can take any music."""
import argparse, json, multiprocessing as mp, subprocess
from pathlib import Path
import cv2, numpy as np
from ending import N0, N_END, STEP, Ending, load
from render import F0
from scenes import OPEN, Scenes
from stage import VH, VW

FPS = 60
BEAT = 70 / 4  # film frames in a beat of the music
# The pace, in raw film frames. Each part from a frame to the next gains this many beats, so topic
# changes stay on beats. Parts meant to feel slow get one, and fast ones like action and comedy get none.
PACE = [(590, 660, 1), (660, 730, 0), (730, 803, .75), (803, 860, .5), (860, 966, .75), (966, 1036, 0),
        (1036, 1106, 1), (1106, 1176, 1), (1176, 1246, 1), (1246, 1316, 0), (1316, 1386, 1), (1386, 1456, 1),
        (1456, 1526, 2), (1526, 1596, 2), (1596, 1666, 2), (1666, 1736, 0), (1736, 1806, 0), (1806, 1876, 1),
        (1876, 1946, 1)]
state = {}

def timeline():
    """Every drawn frame's place in the film. Speed changes are eased, so nothing lurches."""
    last = N_END + OPEN
    grid = np.arange(F0, last + 1, .1)
    speed = np.ones_like(grid)
    for a, b, beats in PACE:
        speed[(grid >= a) & (grid < b)] = (b - a) / (b - a + beats * BEAT)
    width = 8.0
    reach = np.arange(-24, 24.1, .1)
    kernel = np.exp(-.5 * (reach / width) ** 2)
    speed = np.convolve(np.pad(speed, len(reach) // 2, mode="edge"), kernel / kernel.sum(), "valid")[:len(grid)]
    frames, n = [], float(F0)
    while n < last:
        frames.append(n)
        n += STEP * float(np.interp(n, grid, speed))
    return frames

def draw(n):
    return (state["scenes"].frame(n) if n < N0 + OPEN else state["ending"].frame(n - OPEN)).tobytes()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--art", required=True)
    ap.add_argument("--font", required=True)
    ap.add_argument("--logo", required=True)
    ap.add_argument("--mark", default="", help="no longer drawn, accepted so older commands still run")
    ap.add_argument("--brand", required=True)
    ap.add_argument("--ffmpeg", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--workers", type=int, default=9)
    ap.add_argument("--stills", default="")
    a = ap.parse_args()
    cv2.setNumThreads(0)
    name = json.loads(Path(a.brand).read_text())["name"]
    logo = load(a.logo)
    scenes = Scenes(a.art, a.font, logo)
    state["scenes"] = scenes
    state["ending"] = Ending(a.font, logo, name, scenes.art, scenes.keep_image)
    if a.stills:
        for n in (float(x) for x in a.stills.split(",")):
            frame = np.frombuffer(draw(n), np.uint8).reshape(VH, VW, 3)
            cv2.imwrite(f"{a.out}{n:07.1f}.jpg", frame, [cv2.IMWRITE_JPEG_QUALITY, 92])
        return
    p = subprocess.Popen([a.ffmpeg, "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "bgr24",
                          "-s", f"{VW}x{VH}", "-r", str(FPS), "-i", "-", "-c:v", "libx264", "-crf", "15",
                          "-preset", "slow", "-pix_fmt", "yuv420p", "-movflags", "+faststart", a.out],
                         stdin=subprocess.PIPE)
    frames = timeline()
    print("drawing", len(frames), "frames,", round(len(frames) / FPS, 2), "seconds", flush=True)
    with mp.get_context("fork").Pool(a.workers) as pool:
        for i, raw in enumerate(pool.imap(draw, frames, chunksize=6)):
            p.stdin.write(raw)
            if i % 200 == 0:
                print("drawn", i, "of", len(frames), flush=True)
    p.stdin.close()
    p.wait()
    print("wrote", a.out, len(frames), "frames")

if __name__ == "__main__":
    main()
