#!/usr/bin/env python3
"""Stretch the chosen track to the longer film, and bring it back to its opening near the end.

The film's bar lines stay where fit_music put them: 70 frames at 24 fps, the first 12.75 s in. The
source clip carries on past the old end of the film. On a bar line near the end it crossfades into
the opening bars, so the last seconds are almost what the first seconds were, and the file loops
with the film.
"""
import json, subprocess, sys, tempfile
from pathlib import Path
import numpy as np, pyloudnorm as pyln, soundfile as sf

BAR = 70 / 24
FIRST_LINE = 12.75
LENGTH = 1216 / 24
root, ffmpeg, out = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
reprise_bars = int(sys.argv[4]) if len(sys.argv) > 4 else 6
facts = json.loads((root / "provenance.json").read_text())

with tempfile.TemporaryDirectory() as d:
    tmp = f"{d}/s.wav"
    subprocess.run([ffmpeg, "-y", "-loglevel", "error", "-i", str(root / facts["source_clip"]),
                    "-filter:a", f"atempo={facts['speed']:.6f}", "-ar", "48000", tmp], check=True)
    y, sr = sf.read(tmp, always_2d=True)

start = int(round(facts["cut_start_s"] * sr))  # measured on the stretched clip
body = y[start:]
join_s = LENGTH - reprise_bars * BAR
first_bar = FIRST_LINE % BAR
assert join_s < len(body) / sr, "the clip ends before the join"

fade = int(.75 * sr)
join = int(join_s * sr)
head = body[:join + fade // 2].copy()
tail = body[int(first_bar * sr) - fade // 2:].copy()[: int((LENGTH - join_s) * sr) + fade // 2]
t = np.linspace(0, np.pi / 2, fade)[:, None]
mix = head[-fade:] * np.cos(t) + tail[:fade] * np.sin(t)
song = np.vstack([head[:-fade], mix, tail[fade:]])[: int(LENGTH * sr)]
close = int(2.4 * sr)
song[-close:] *= np.cos(np.linspace(0, np.pi / 2, close))[:, None] ** 1.5

meter = pyln.Meter(sr)
song = pyln.normalize.loudness(song, meter.integrated_loudness(song), facts["lufs"])
song = np.clip(song, -.98, .98)
sf.write(out, song, sr, subtype="PCM_24")
print("wrote", out, round(len(song) / sr, 3), "s, join at", round(join_s, 2), "s")
