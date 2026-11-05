// Checks the timeline against the measured reference values.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  CARD, CYCLE_FRAMES, ENTRY_FRAMES, entryEase, EXIT_FRAMES, exitEase, FRAME_MS, GAP_FRAMES, makePlan, newScene, sceneAt, cycleStartMs
} from "../src/landing/timeline.ts";

const plan = makePlan(1800, 10);
const REST_WIDTH = (0.175 * 1080) / 1920;

function at(ms: number) {
  return sceneAt(plan, ms, REST_WIDTH, newScene());
}
function open(f: number) {
  return at(plan.openingStartMs + (f + 1e-6) * FRAME_MS);
}
function land(f: number) {
  return at(plan.landingStartMs + (f + 1e-6) * FRAME_MS);
}

test("layer cycle is 70 frames, 2,917 ms, and its parts fill it", () => {
  assert.equal(CYCLE_FRAMES, 70);
  assert.ok(Math.abs(CYCLE_FRAMES * FRAME_MS - 2917) < 8);
  const rest = CYCLE_FRAMES - EXIT_FRAMES - GAP_FRAMES - ENTRY_FRAMES;
  assert.ok(rest >= 16 && rest <= 18, "rest about 700 ms");
  assert.ok(EXIT_FRAMES * FRAME_MS > 900 && EXIT_FRAMES * FRAME_MS < 1050, "exit about 958 ms");
  assert.ok(GAP_FRAMES * FRAME_MS >= 42 && GAP_FRAMES * FRAME_MS <= 250);
  assert.ok(ENTRY_FRAMES * FRAME_MS >= 1000 && ENTRY_FRAMES * FRAME_MS <= 1300);
});

test("exit and entry eases match the measured progress tables", () => {
  const exit = [0.011, 0.029, 0.057, 0.1, 0.169, 0.274, 0.422, 0.609, 0.816];
  const entry = [0.364, 0.631, 0.803, 0.888, 0.933, 0.959, 0.976, 0.987, 0.995];
  for (let i = 0; i < 9; i++) {
    assert.ok(Math.abs(exitEase((i + 1) / 10) - exit[i]) < 0.02, `exit at ${i}`);
    assert.ok(Math.abs(entryEase((i + 1) / 10) - entry[i]) < 0.03, `entry at ${i}`);
  }
});

test("layer scale rests at 1, ends the exit near 0.56, and enters from 2.8", () => {
  const t = cycleStartMs(plan, 0);
  assert.equal(at(t - FRAME_MS).layers[0].scale, 1);
  const late = at(t + (EXIT_FRAMES - 0.01) * FRAME_MS).layers[0].scale;
  assert.ok(Math.abs(late - 0.56) < 0.01);
  const start = at(t + (EXIT_FRAMES + GAP_FRAMES + 0.001) * FRAME_MS).layers[0].scale;
  assert.ok(Math.abs(start - 2.8) < 0.01);
  assert.equal(at(t + (EXIT_FRAMES + 2) * FRAME_MS).layerCount, 0, "gap shows only the backdrop");
});

test("corner speed peaks near the measured 0.47 W per second on exit", () => {
  const t = cycleStartMs(plan, 0);
  const corner = 0.46;
  let peak = 0;
  for (let f = 0; f < EXIT_FRAMES - 1; f += 0.25) {
    const a = at(t + f * FRAME_MS).layers[0].scale;
    const b = at(t + (f + 0.25) * FRAME_MS).layers[0].scale;
    peak = Math.max(peak, (Math.abs(b - a) * corner) / ((0.25 * FRAME_MS) / 1000));
  }
  assert.ok(peak > 0.4 && peak < 0.75, `peak ${peak}`);
});

test("line 2 follows line 1 by three frames, 125 ms", () => {
  const before = land(159 + 2.9);
  const after = land(159 + 3.1);
  assert.ok(!before.texts.slice(0, before.textCount).some((t) => t.headline === 1 && t.line === 1));
  assert.ok(after.texts.slice(0, after.textCount).some((t) => t.headline === 1 && t.line === 1));
});

test("headline goes out through pink: green falls before red", () => {
  const s = land(153 + 3);
  const t = s.texts.slice(0, s.textCount).find((x) => x.headline === 0)!;
  assert.ok(t.g < t.r && t.b < t.r && t.b > t.g);
});

test("opening column widths follow the reference table", () => {
  const widths: [number, number][] = [[23, 0.105], [25, 0.137], [27, 0.171], [29, 0.305], [30, 0.638], [31, 0.994]];
  for (const [f, w] of widths) assert.ok(Math.abs(open(f).column * 2 - w) < 0.001, `frame ${f}`);
});

test("stripe zoom reproduces the measured group edges within 0.03 W", () => {
  const blue = 0.052;
  const violet = 0.034;
  assert.equal(open(33.9).split, 0);
  assert.equal(open(34).split, 1);
  assert.ok(Math.abs(0.5 + blue * open(40).zoom - 0.604) < 0.03);
  assert.ok(Math.abs(0.5 + blue * open(44).zoom - 0.726) < 0.03);
  assert.ok(Math.abs(0.5 + violet * open(40).zoom - 0.568) < 0.03);
  assert.ok(Math.abs(0.5 + violet * open(44).zoom - 0.613) < 0.03);
});

test("the card lands on the measured rectangle within nine frames", () => {
  assert.ok(land(0).cardShift > 0.9);
  assert.ok(land(8).cardShift < 0.005);
  assert.equal(land(9).cardShift, 0);
  assert.equal(CARD.radiusPx, 24);
});

test("the sequence keeps running past the last headline", () => {
  const s = at(cycleStartMs(plan, 40) + 60 * FRAME_MS);
  assert.equal(s.stage, 1);
  assert.equal(s.layerCount, 1);
  assert.ok(s.texts[0].headline < 10);
});

test("works with fewer headlines than the first pass uses", () => {
  for (const n of [1, 2, 3, 4]) {
    const p = makePlan(0, n);
    for (let ms = 0; ms < 40000; ms += 97) {
      const s = sceneAt(p, ms, REST_WIDTH, newScene());
      for (let i = 0; i < s.layerCount; i++) assert.ok(s.layers[i].set < p.setCount, `n=${n} ms=${ms}`);
      for (let i = 0; i < s.textCount; i++) assert.ok(s.texts[i].headline < n, `n=${n} ms=${ms}`);
    }
  }
});

test("alpha stays between 0 and 1", () => {
  for (let ms = 0; ms < 60000; ms += 11) {
    const s = at(ms);
    for (let i = 0; i < s.layerCount; i++) assert.ok(s.layers[i].alpha >= 0 && s.layers[i].alpha <= 1, `ms=${ms}`);
  }
});
