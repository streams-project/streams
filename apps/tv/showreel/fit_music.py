#!/usr/bin/env python3
"""Check each generated clip, then cut it to the film so a bar line lands on every headline change.

A clip is dropped if it is silent, clipped, or off the tempo. The bar is 70 frames at 24 fps,
2.917 s, and the first layer change in the film falls 12.75 s in. The stretch runs through ffmpeg.
Arguments after the root and ffmpeg: length, reprise bars, take, output folder, fixed cut in seconds.
With a fixed cut the report is written next to the output, so earlier reports stay as they are.
"""
import glob, json, subprocess, sys, tempfile
from pathlib import Path
import numpy as np, pyloudnorm as pyln, soundfile as sf
from scipy.signal import stft

BAR = 70 / 24
FIRST_CHANGE = (896 - 590) / 24
root, ffmpeg = Path(sys.argv[1]), sys.argv[2]
LENGTH = float(sys.argv[3]) if len(sys.argv) > 3 else 750 / 24
# Bars of the opening played again at the end, so the track comes back to how it began. 0 is none.
REPRISE = int(sys.argv[4]) if len(sys.argv) > 4 else 0
TAKE = sys.argv[5] if len(sys.argv) > 5 else "*"
out = root / ("fitted" if len(sys.argv) < 7 else sys.argv[6])
out.mkdir(exist_ok=True)
# A cut in seconds of the stretched take. Given, it replaces the search for the evenest part.
START = float(sys.argv[7]) if len(sys.argv) > 7 else None

def onset(mono, sr):
    hop = 512
    f, t, Z = stft(mono, sr, nperseg=2048, noverlap=2048 - hop)
    mag = np.log1p(np.abs(Z) * 20)
    env = np.clip(np.diff(mag, axis=1), 0, None).mean(0)
    env = env - np.convolve(env, np.ones(43) / 43, "same")
    return np.clip(env, 0, None), sr / hop, float(np.exp(np.log(np.abs(Z) + 1e-9).mean(0)).mean() / (np.abs(Z).mean() + 1e-9))

def tempo_of(env, rate):
    ac = np.correlate(env, env, "full")[len(env) - 1:]
    lags = np.arange(int(rate * 60 / 124), int(rate * 60 / 60))
    # Weight towards 82, the tempo asked for, so a clip does not lock to a half or double beat.
    score = ac[lags] * np.exp(-0.5 * (np.log2((60 * rate / lags) / 82) / 0.4) ** 2)
    lag = lags[score.argmax()]
    # Refine on a few multiples of the beat.
    best = max(np.arange(lag - 1.5, lag + 1.5, 0.05), key=lambda l: sum(np.interp(k * l, np.arange(len(ac)), ac) for k in (1, 2, 4, 8)))
    return 60 * rate / best, best

def beats_of(env, lag):
    n = int(round(lag))
    phase = max(range(n), key=lambda p: env[p::n].sum() if False else np.interp(np.arange(p, len(env), lag), np.arange(len(env)), env).sum())
    return np.arange(phase, len(env), lag)

report = []
for path in sorted(glob.glob(str(root / "music" / TAKE / "*.flac"))):
    y, sr = sf.read(path, always_2d=True)
    mono = y.mean(1)
    rms = 20 * np.log10(np.sqrt((mono ** 2).mean()) + 1e-9)
    peak = float(np.abs(y).max())
    env, rate, flat = onset(mono, sr)
    tempo, lag = tempo_of(env, rate)
    off = abs(tempo - 82) / 82
    ok = bool(rms > -45 and peak <= 1.0 and off < 0.03 and np.isfinite(y).all())
    item = dict(prompt=Path(path).parent.name, file=Path(path).name[:8], seconds=round(len(y) / sr, 1), rms_db=round(float(rms), 1), peak=round(peak, 3), tempo=round(float(tempo), 2), flatness=round(flat, 4), ok=ok)
    report.append(item)
    if not ok:
        continue
    want = 240 / BAR
    speed = want / tempo
    with tempfile.TemporaryDirectory() as d:
        tmp = f"{d}/s.wav"
        subprocess.run([ffmpeg, "-y", "-loglevel", "error", "-i", path, "-filter:a", f"atempo={speed:.6f}", "-ar", str(sr), tmp], check=True)
        s, _ = sf.read(tmp, always_2d=True)
    beats = beats_of(env, lag) / rate / speed
    bar_in = FIRST_CHANGE % BAR
    cands = [b for b in beats if 3.5 < b - bar_in and b - bar_in + LENGTH + 1 < len(s) / sr]
    def uneven(b):
        seg = s[int((b - bar_in) * sr):int((b - bar_in + LENGTH) * sr)].mean(1)
        hop = sr // 4
        lv = np.sqrt(np.convolve(seg ** 2, np.ones(hop) / hop, "valid")[::hop] + 1e-12)
        return float(lv.std() / lv.mean())
    b = min(cands, key=uneven) if START is None else START + bar_in
    start = b - bar_in
    clip = s[int(start * sr):int(start * sr) + int(LENGTH * sr)].copy()
    if REPRISE:
        fade = int(.75 * sr)
        join = int((LENGTH - REPRISE * BAR) * sr)
        again = s[int((start + bar_in) * sr) - fade // 2:][:len(clip) - join + fade // 2]
        t = np.linspace(0, np.pi / 2, fade)[:, None]
        mix = s[int(start * sr) + join - fade // 2:][:fade] * np.cos(t) + again[:fade] * np.sin(t)
        clip = np.vstack([clip[:join - fade // 2], mix, again[fade:]])[:int(LENGTH * sr)]
    a, z = int(0.6 * sr), int(2.4 * sr)
    clip[:a] *= np.linspace(0, 1, a)[:, None]
    clip[-z:] *= (np.linspace(1, 0, z) ** 1.5)[:, None]
    meter = pyln.Meter(sr)
    clip = pyln.normalize.loudness(clip, meter.integrated_loudness(clip), -18.0)
    p = float(np.abs(clip).max())
    if p > 0.89:
        clip *= 0.89 / p
    name = f"{item['prompt']}-{item['file']}"
    sf.write(out / f"{name}.wav", clip, sr, subtype="PCM_24")
    item.update(name=name, start=round(float(start), 2), speed=round(float(speed), 4), lufs=round(meter.integrated_loudness(clip), 1), uneven=round(uneven(b), 3))
(root if START is None else out).joinpath("music-report.json").write_text(json.dumps(report, indent=1))
for r in report:
    print(r)
