// Checks the sequence as it is actually drawn against the motion reading.
//
// `node tools/landing.ts phases` renders every frame at 24 fps, reads each
// phase boundary out of the pixels and writes test/phases.json. This test holds
// that file to the reading: no phase may be more than one frame, 41.7 ms, away,
// and the file must have been made from the source as it stands, so a change to
// the timeline that has not been measured again fails here.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { test } from "node:test";
import { FRAME_MS, SPEC, sourceHash } from "../tools/phases.ts";
import type { Phase } from "../tools/phases.ts";

const ROOT = resolve(import.meta.dirname, "..");
const FILE = join(ROOT, "test/phases.json");
const report = JSON.parse(readFileSync(FILE, "utf8")) as {
  sourceHash: string; fps: number; framesRendered: number; phases: Phase[];
};

test("the measurement was made from the source as it stands", () => {
  assert.equal(report.sourceHash, sourceHash(ROOT),
    "src/landing or landing/config.json changed: run `node tools/landing.ts phases` again");
  assert.equal(report.fps, 24);
});

test("every phase the motion reading gives was measured", () => {
  const measured = report.phases.map((p) => p.key);
  for (const spec of SPEC) assert.ok(measured.includes(spec.key), `${spec.key} was not measured`);
  assert.equal(measured.length, SPEC.length);
});

test("no phase is more than one frame from the reading", () => {
  const limit = FRAME_MS + 0.05;
  const bad: string[] = [];
  for (const p of report.phases) {
    if (Math.abs(p.offMs) > limit) bad.push(`${p.what}: ${p.ms} ms, reading ${p.specMs} ms, off ${p.offMs} ms`);
  }
  assert.deepEqual(bad, [], `phases outside one frame at 24 fps:\n${bad.join("\n")}`);
});

test("the phases run in order and none of them is empty", () => {
  for (const p of report.phases) {
    assert.ok(p.frames > 0, `${p.what} measured ${p.frames} frames`);
    assert.equal(p.to - p.from, p.frames);
    assert.ok(Math.abs(p.ms - p.frames * FRAME_MS) < 0.1, `${p.what} length does not match its frames`);
  }
  const opening = report.phases.slice(0, 11);
  for (let i = 1; i < opening.length; i++) {
    assert.ok(opening[i].from >= opening[i - 1].from, `${opening[i].what} starts before the phase before it`);
  }
});
