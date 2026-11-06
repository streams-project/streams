// Checks the timeline against the measured reference values.

import assert from "node:assert/strict";
import { test } from "node:test";
import { table } from "../src/landing/curves.ts";
import { BLUE_EDGE, GAP, VIOLET_EDGE } from "../src/landing/stripes.ts";
import {
  CARD, CYCLE_FRAMES, CYCLE_START, ENTRY_FRAMES, entryEase, EXIT_FRAMES, exitEase, FRAME_MS, F_BLACK, F_BURST,
  F_COLLAPSE, F_COLUMN, F_CUT, F_FULL, F_MARK_REST, F_PEEK_IN, F_SPLIT, F_STRETCH, F_TEXT_IN, F_TEXT_OUT_2,
  F_TEXT_OUT_3, F_TEXT_OUT_4, F_WALL_IN, F_WALL_OUT, F_FIRST_ENTRY, F_FIRST_REST, GAP_FRAMES, makePlan, markHeightOf,
  newScene, sceneAt, SLIDE_FRAMES, STRETCH_HEIGHT, cycleStartMs
} from "../src/landing/timeline.ts";

const plan = makePlan(10);
// Our own marks, at the measured rest heights.
const WORD_HALF = ((0.6 - 0.411) * 1080 * (3712 / 1024)) / 1920 / 2;
const MARK_HALF = (markHeightOf() * 1080) / 1920 / 2;

function at(ms: number) {
  return sceneAt(plan, ms, WORD_HALF, MARK_HALF, newScene());
}
function frame(f: number) {
  return at((f + 1e-6) * FRAME_MS);
}
function open(f: number) {
  return frame(F_STRETCH + f);
}
function land(f: number) {
  return at(plan.landingStartMs + (f + 1e-6) * FRAME_MS);
}

// Every phase the motion reading gives, as frames at 24 fps.
const PHASES: [string, number, number, number][] = [
  ["wordmark rest", 0, F_COLLAPSE, 108],
  ["wordmark collapses to the mark", F_COLLAPSE, F_MARK_REST, 28],
  ["mark rest", F_MARK_REST, F_STRETCH, 258],
  ["mark stretches and folds", F_STRETCH, F_COLUMN, 23],
  ["column widens", F_COLUMN, F_BURST, 6],
  ["burst to full width", F_BURST, F_FULL, 3],
  ["full width, one colour", F_FULL - 1, F_SPLIT, 3],
  ["stripes run outward", F_SPLIT, F_BLACK, 15],
  ["black with ring", F_BLACK, F_CUT, 10],
  ["card slide", 0, SLIDE_FRAMES, 8],
  ["empty card rest", SLIDE_FRAMES, F_WALL_IN, 8],
  ["wall fades in", F_WALL_IN, F_WALL_IN + 3, 3],
  ["wall rest", F_WALL_IN + 3, F_WALL_OUT, 44],
  ["wall out, wordmark in", F_WALL_OUT, F_WALL_OUT + 14, 14],
  ["wordmark leaves, first layer settles", F_FIRST_ENTRY, F_FIRST_REST, 42],
  ["first headline fades in", F_TEXT_IN, F_TEXT_IN + 8, 8],
  ["headline 1 to 2", F_TEXT_OUT_2, F_TEXT_OUT_2 + 15, 15],
  ["headline 2 to 3", F_TEXT_OUT_3, F_TEXT_OUT_3 + 15, 15],
  ["right peek fades in", F_PEEK_IN, F_PEEK_IN + 14, 14],
  ["headline 3 to 4", F_TEXT_OUT_4, F_TEXT_OUT_4 + 38, 38],
  ["layer cycle", CYCLE_START, CYCLE_START + CYCLE_FRAMES, 70]
];

test("every phase boundary is a whole frame and the right length", () => {
  for (const [name, from, to, frames] of PHASES) {
    assert.equal(Math.round(from), from, `${name} starts on a frame`);
    assert.equal(Math.round(to), to, `${name} ends on a frame`);
    assert.equal(to - from, frames, `${name} is ${frames} frames`);
  }
});

test("the opening runs 4.50, 1.167 and 10.75 seconds before the stretch", () => {
  assert.ok(Math.abs(F_COLLAPSE * FRAME_MS - 4500) < 1);
  assert.ok(Math.abs((F_MARK_REST - F_COLLAPSE) * FRAME_MS - 1167) < 1);
  assert.ok(Math.abs((F_STRETCH - F_MARK_REST) * FRAME_MS - 10750) < 1);
  assert.ok(Math.abs(plan.landingStartMs - F_CUT * FRAME_MS) < 0.001);
});

test("the wordmark rests still, then slides right and loses letters", () => {
  const a = frame(0);
  const b = frame(F_COLLAPSE - 1);
  assert.equal(a.wordU0, b.wordU0);
  assert.equal(a.wordClip, b.wordClip);
  assert.ok(Math.abs(a.wordV0 - 0.411) < 1e-9 && Math.abs(a.wordV1 - 0.6) < 1e-9);
  const mid = frame(F_COLLAPSE + 14);
  assert.ok(mid.wordU0 > a.wordU0, "slides right");
  assert.ok(mid.wordClip < a.wordClip, "loses letters from the right");
  assert.ok(mid.wordV1 < a.wordV1, "the word drops in height");
  const end = frame(F_MARK_REST);
  assert.equal(end.wordAlpha, 0);
  assert.equal(end.markAlpha, 1);
  assert.ok(Math.abs(end.markV0 - 0.411) < 1e-9 && Math.abs(end.markV1 - 0.586) < 1e-9);
});

test("the mark rests still for its 258 frames", () => {
  const a = frame(F_MARK_REST);
  const b = frame(F_STRETCH - 1);
  assert.equal(a.markU0, b.markU0);
  assert.equal(a.markV1, b.markV1);
  assert.equal(a.column, 0);
});

test("the stretch follows the measured height and reaches full height", () => {
  for (const f of [3, 6, 9, 12, 15, 18, 20, 22]) {
    const s = open(f);
    const got = (s.markV1 - s.markV0) / markHeightOf();
    assert.ok(Math.abs(got - table(STRETCH_HEIGHT, f)) < 0.02, `frame ${f}: ${got}`);
  }
  const end = open(23);
  assert.ok(end.markV0 <= 0.001 && end.markV1 >= 0.999, "full height at the fold");
});

test("opening column widths follow the reference table", () => {
  const widths: [number, number][] = [[23, 0.105], [25, 0.137], [27, 0.171], [28, 0.241], [29, 0.305], [30, 0.638], [31, 0.994], [32, 1]];
  for (const [f, w] of widths) assert.ok(Math.abs(open(f).column * 2 - w) < 0.001, `frame ${f}`);
});

test("the colour split lands on one frame, then two families run outward", () => {
  assert.equal(frame(F_SPLIT - 0.1).split, 0);
  assert.equal(frame(F_SPLIT).split, 1);
  assert.ok(Math.abs(frame(F_SPLIT).zoomWarm - 1) < 1e-5);
  assert.ok(Math.abs(frame(F_SPLIT).zoomCool - 1) < 1e-5);
  for (let f = F_SPLIT; f < F_BLACK - 1; f++) {
    assert.ok(frame(f + 1).zoomWarm > frame(f).zoomWarm, `warm still rising at ${f}`);
    assert.ok(frame(f + 1).zoomCool > frame(f).zoomCool, `cool still rising at ${f}`);
  }
});

test("the group edges and the dark gap match the readings", () => {
  // Six and ten frames after the split: violet right edge 0.568 then 0.613,
  // blue left edge 0.604 then 0.726, gap half width 0.008 then 0.014.
  const a = frame(F_SPLIT + 6);
  const b = frame(F_SPLIT + 10);
  assert.ok(Math.abs(0.5 + VIOLET_EDGE * a.zoomWarm - 0.568) < 0.005, "violet at +6");
  assert.ok(Math.abs(0.5 + VIOLET_EDGE * b.zoomWarm - 0.613) < 0.005, "violet at +10");
  assert.ok(Math.abs(0.5 + BLUE_EDGE * a.zoomCool - 0.604) < 0.005, "blue at +6");
  assert.ok(Math.abs(0.5 + BLUE_EDGE * b.zoomCool - 0.726) < 0.005, "blue at +10");
  assert.ok(Math.abs(GAP * a.zoomWarm - 0.008) < 0.001, "gap at +6");
  assert.ok(Math.abs(GAP * b.zoomWarm - 0.014) < 0.002, "gap at +10");
});

test("the group edge speeds are the measured 0.27 and 0.73 of width per second", () => {
  const dt = 4 / 24;
  const a = frame(F_SPLIT + 6);
  const b = frame(F_SPLIT + 10);
  const warm = (VIOLET_EDGE * (b.zoomWarm - a.zoomWarm)) / dt;
  const cool = (BLUE_EDGE * (b.zoomCool - a.zoomCool)) / dt;
  assert.ok(Math.abs(warm - 0.27) < 0.03, `violet ${warm}`);
  assert.ok(Math.abs(cool - 0.73) < 0.03, `blue ${cool}`);
  assert.ok(cool > warm, "the push is faster further from the centre");
});

test("the picture fades to black over six frames and the ring holds on black", () => {
  assert.ok(frame(F_BLACK - 7).fade > 0.9999, "still full before the fade");
  for (let f = F_BLACK - 6; f < F_BLACK; f++) {
    assert.ok(frame(f).fade > 0 && frame(f).fade < 1, `frame ${f} is part way`);
  }
  assert.equal(frame(F_BLACK).fade, 0);
  assert.equal(frame(F_CUT - 1).ringAlpha, 1);
  assert.equal(frame(F_CUT - 1).fade, 0);
});

test("ring A runs in three spells and outlasts the stretch onset", () => {
  assert.equal(frame(10).ringAlpha, 1);
  assert.equal(frame(100).ringAlpha, 0);
  assert.equal(frame(195).ringAlpha, 1);
  assert.equal(frame(220).ringAlpha, 0);
  assert.equal(frame(300).ringAlpha, 1);
  assert.equal(frame(F_STRETCH + 7).ringAlpha, 1, "seven frames past the onset");
  assert.equal(frame(F_STRETCH + 9).ringAlpha, 0);
});

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
  const before = land(160 + 2.9);
  const after = land(160 + 3.1);
  assert.ok(!before.texts.slice(0, before.textCount).some((t) => t.headline === 1 && t.line === 1));
  assert.ok(after.texts.slice(0, after.textCount).some((t) => t.headline === 1 && t.line === 1));
});

test("headline goes out through pink: green falls before red", () => {
  const s = land(F_TEXT_OUT_2 + 3);
  const t = s.texts.slice(0, s.textCount).find((x) => x.headline === 0)!;
  assert.ok(t.g < t.r && t.b < t.r && t.b > t.g);
});

test("the card lands on the measured rectangle within eight frames", () => {
  assert.ok(land(0).cardShift > 0.6);
  assert.ok(land(7).cardShift < 0.005);
  assert.equal(land(SLIDE_FRAMES).cardShift, 0);
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
    const p = makePlan(n);
    for (let ms = 0; ms < 60000; ms += 97) {
      const s = sceneAt(p, ms, WORD_HALF, MARK_HALF, newScene());
      for (let i = 0; i < s.layerCount; i++) assert.ok(s.layers[i].set < p.setCount, `n=${n} ms=${ms}`);
      for (let i = 0; i < s.textCount; i++) assert.ok(s.texts[i].headline < n, `n=${n} ms=${ms}`);
    }
  }
});

test("alpha stays between 0 and 1", () => {
  for (let ms = 0; ms < 80000; ms += 11) {
    const s = at(ms);
    for (const a of [s.wordAlpha, s.markAlpha, s.fade, s.peek, s.cardWordAlpha, s.ringAlpha]) {
      assert.ok(a >= 0 && a <= 1, `ms=${ms}`);
    }
    for (let i = 0; i < s.layerCount; i++) assert.ok(s.layers[i].alpha >= 0 && s.layers[i].alpha <= 1, `ms=${ms}`);
  }
});
