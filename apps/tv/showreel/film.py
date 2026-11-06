#!/usr/bin/env python3
"""The whole showreel, frame 590 to the last, rendered in one pass and encoded once.

The headlines come from render.py and the ending from ending.py. Both draw raw frames here, so the
film has one encode, and the same frames can take any of the music tracks.
"""
import argparse, json, multiprocessing as mp, subprocess
from pathlib import Path
import cv2, numpy as np
from ending import N_END, STEP, Ending, load
from render import F0, VH, VW, Film

HEADLINES_LAST = 1345  # the last headline's blur runs a few frames past where the old cut ended
FPS = 60
state = {}

def draw(n):
    return (state["film"].frame(n) if n < HEADLINES_LAST else state["ending"].frame(n)).tobytes()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--art", required=True)
    ap.add_argument("--font", required=True)
    ap.add_argument("--logo", required=True)
    ap.add_argument("--mark", required=True)
    ap.add_argument("--brand", required=True)
    ap.add_argument("--ffmpeg", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--stills", default="")
    a = ap.parse_args()
    cv2.setNumThreads(0)
    name = json.loads(Path(a.brand).read_text())["name"]
    state["film"] = Film(a.art, a.font, a.logo)
    state["ending"] = Ending(a.font, load(a.logo), load(a.mark), name, state["film"].genre)
    if a.stills:
        for n in (float(x) for x in a.stills.split(",")):
            frame = np.frombuffer(draw(n), np.uint8).reshape(VH, VW, 3)
            cv2.imwrite(f"{a.out}{n:07.1f}.jpg", frame, [cv2.IMWRITE_JPEG_QUALITY, 92])
        return
    p = subprocess.Popen([a.ffmpeg, "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "bgr24",
                          "-s", f"{VW}x{VH}", "-r", str(FPS), "-i", "-", "-c:v", "libx264", "-crf", "15",
                          "-preset", "slow", "-pix_fmt", "yuv420p", "-movflags", "+faststart", a.out],
                         stdin=subprocess.PIPE)
    frames = [F0 + i * STEP for i in range(int((N_END - F0) / STEP))]
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
