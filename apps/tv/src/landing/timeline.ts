// The whole sequence as a pure function of time.
//
// Every number in this file comes from frame by frame readings of a 24 fps
// recording of a reference app on a real TV, unless marked "own" (our own
// design choice) or "assumed" (the reading could not settle it). Frame counts
// are kept as read, so a duration is good to plus or minus 42 ms.
//
// Frames are counted from the first frame of the wordmark, at 24 fps, and every
// phase boundary below is a whole frame. `tools/phases.ts` renders the sequence
// and reads each boundary back out of the pixels; `test/phases.test.ts` fails if
// any phase is more than one frame away from the reading it came from.
//
// sceneAt() fills a reused Scene object and allocates nothing, so a slow TV
// does not collect garbage in the middle of a move.

import { bezier, clamp01, lerp, smooth, table } from "./curves.ts";

export var FRAME_MS = 1000 / 24;

// Measured geometry, shares of picture width (u) and height (v).
export var CARD = { u0: 0.0287, u1: 0.8315, v0: 0.1565, v1: 0.9567, radiusPx: 24 };
export var PEEK = { u0: 0.852, v0: 0.155, v1: 0.957 };
export var PIVOT = { u: 0.43, v: 0.557 };
export var HEADLINE_U = 0.431;
export var TWO_LINE_V = [0.463, 0.582];
export var ONE_LINE_V = [0.558, 0.668, 0.357, 0.558, 0.668, 0.357, 0.271];
export var CAP_HEIGHT = { twoLine: 0.072, oneLine: 0.063 };

// Wordmark rest box: u 0.302 to 0.695, v 0.411 to 0.60. The width follows from
// our own wordmark's shape, so only the height and the centre are taken.
export var WORD_REST = { v0: 0.411, v1: 0.600, u: 0.4985 };
// Mark rest box: v 0.411 to 0.586, centre (0.4987, 0.4985).
export var MARK_REST = { v0: 0.411, v1: 0.586, u: 0.4987 };

// ---------------------------------------------------------------------------
// Opening. Frames from the first wordmark frame.
// ---------------------------------------------------------------------------
// 108 f, 4,500 ms  wordmark rest
export var F_COLLAPSE = 108;
// 28 f, 1,167 ms   wordmark collapses to the mark
export var F_MARK_REST = 136;
// 258 f, 10,750 ms mark rest
export var F_STRETCH = 394;
// 23 f, 958 ms     mark stretches and folds into a column
export var F_COLUMN = 417;
// 6 f, 250 ms      column widens, 0.105 to 0.305 W
export var F_BURST = 423;
// 3 f, 125 ms      burst to full width (reached at F_FULL)
export var F_FULL = 426;
// 3 f, 125 ms      full width, one colour, from the first frame at 0.99 W
export var F_SPLIT = 428;
// 1 f              colour split
// 15 f, 625 ms     stripes run outward from a dark centre gap
export var F_BLACK = 443;
// 6 f, 250 ms      fade to black: 437 is the first dimmed frame, 443 the first black one
var FADE_FROM = 436;
var FADE_SPAN = 7;
// 10 f, 417 ms     black with ring, then the hard cut
export var F_CUT = 453;

// Wordmark to mark, frames from F_COLLAPSE.
var COLLAPSE_FRAMES = 28;
// Slide of the first letter: Bezier (0.59, 0.21, 0.54, 0.92), rms 0.005.
var collapseEase = bezier(0.59, 0.21, 0.54, 0.92);
// Right edge of the word as letters go, scaled from the reference's 0.695 to
// 0.527 onto our own rest and mark edges.
var WORD_RIGHT_REF = [0, 0.695, 6, 0.697, 9, 0.659, 12, 0.624, 16, 0.594, 19, 0.569, 22, 0.546, 28, 0.527];
var WORD_BOTTOM = [0, 0.600, 28, 0.586];
// Own: our wordmark has no letter that is already the mark, so the stub hands
// over to the mark in the last third of the collapse.
var MARK_IN = [19, 0, 26, 1];
var WORD_OUT = [22, 1, 28, 0];

// Mark stretch, frames from F_STRETCH. Height as a multiple of the rest height,
// kept so the test can check the box against the reading it came from.
export var STRETCH_HEIGHT = [0, 1, 3, 1.05, 6, 1.15, 9, 1.37, 12, 1.77, 15, 2.39, 18, 3.39, 20, 4.35, 22, 5.05, 23, 6.2];
// Top and bottom read straight off the reference, which rests in the same box.
var STRETCH_TOP = [0, 0.411, 3, 0.4065, 6, 0.3954, 9, 0.3741, 12, 0.3333, 15, 0.2694, 18, 0.1731, 20, 0.0926, 22, 0.0847, 23, 0];
var STRETCH_BOTTOM = [0, 0.586, 3, 0.5898, 6, 0.5963, 9, 0.6130, 12, 0.6426, 15, 0.6880, 18, 0.7657, 20, 0.8546, 22, 0.9685, 23, 1];
// Half width and centre of the reference glyph, frame by frame.
var STRETCH_HALF = [0, 0.0284, 3, 0.02945, 6, 0.0331, 9, 0.03725, 12, 0.04815, 15, 0.06535, 18, 0.08905, 20, 0.0839, 22, 0.0654, 23, 0.05075];
var STRETCH_MID = [0, 0.4987, 3, 0.49975, 6, 0.50080, 9, 0.50335, 12, 0.50755, 15, 0.51485, 18, 0.52605, 20, 0.50990, 22, 0.47840, 23, 0.46015];
// Our mark is wider than the reference glyph, so its box starts 1.733 times as
// wide and is drawn back to the reference's own width as it folds, which is
// where the column picks it up at 0.105 W.
var MARK_WIDE = 1.733;
var FOLD_FROM = 12;
var FILAMENT = [11, 0, 18, 1];
var LINES = [16, 0, 22, 1];

// Column width, frames from F_STRETCH. 0.105 W at the fold, 1.0 W at the burst.
var COLUMN = [23, 0.105, 25, 0.137, 27, 0.171, 28, 0.241, 29, 0.305, 30, 0.638, 31, 0.994, 32, 1];
// Own: before the fold ends the column grows out of the folding mark.
var COLUMN_GROW = [17, 0.02, 20, 0.045, 22, 0.08, 23, 0.105];

// Stripe push. Two families, each pushed out from the centre by its own zoom,
// fitted to the measured group edges: the violet family's right edge runs at
// 0.27 W per second and the blue family's left edge at 0.73 W per second, both
// read between 6 and 10 frames after the colour split. Z = exp(a t + b t cubed),
// t in seconds from the split, so a point's speed is its distance from the
// centre times the rate: the push gets faster the further out you look.
var WARM_A = 2.7106;
var WARM_B = 0.9922;
var COOL_A = 2.3487;
var COOL_B = 6.7830;

// ---------------------------------------------------------------------------
// Landing. Frames from the hard cut.
// ---------------------------------------------------------------------------
// 8 f, 333 ms      card slides in from the right
var SLIDE = [0, 0.647, 1, 0.323, 2, 0.106, 3, 0.077, 4, 0.039, 5, 0.017, 6, 0.005, 7, 0.001, 8, 0];
export var SLIDE_FRAMES = 8;
// 8 f, 333 ms      empty card rest, then 3 f, 125 ms, the wall fades in
var WALL_IN = [16, 0, 17, 0.45, 18, 0.85, 19, 1];
export var F_WALL_IN = 16;
// 44 f, 1,833 ms   wall rest, then 14 f, 583 ms, the wall goes and the wordmark comes
export var F_WALL_OUT = 63;
var WALL_OUT = [62, 0, 63, 0.06, 65, 0.19, 67, 0.29, 69, 0.39, 71, 0.52, 73, 0.67, 75, 0.84, 77, 1];
var MARK_COVER = [63, 0, 65, 0.3, 69, 1];
// 42 f, 1,750 ms   the wordmark leaves and the first tile layer settles
export var F_FIRST_ENTRY = 85;
export var F_FIRST_REST = 127;
var MARK_SCALE = [77, 1, 81, 0.9575, 85, 0.8679, 93, 0.58];
var MARK_FADE = [85, 1, 93, 0];
var FIRST_SCALE = [85, 2.80, 89, 2.36, 93, 1.85, 97, 1.48, 101, 1.31, 105, 1.22, 109, 1.16, 113, 1.09, 117, 1.04, 121, 1.015, 125, 1.004, 127, 1];
// 8 f, 333 ms      the first headline fades in
export var F_TEXT_IN = 105;
var TEXT_OUT_FRAMES = 7;
var TEXT_IN_FRAMES = 6;
var LINE_LAG = 3;
// 15 f, 625 ms each, 57 frames apart
export var F_TEXT_OUT_2 = 154;
export var F_TEXT_OUT_3 = 211;
export var F_TEXT_OUT_4 = 268;
var SECOND_CROSSFADE = 160;
var CROSSFADE_FRAMES = 3;
var FOURTH_ENTRY = 274;
var FOURTH_ENTRY_FRAMES = 32;
// 14 f, 583 ms     the right peek fades in, first pass only
export var F_PEEK_IN = 261;
var PEEK_IN = [261, 0, 275, 1];
// one change every 70 frames, 2,917 ms
export var CYCLE_START = 323;

// Layer cycle: 70 frames. Exit plus gap plus entry plus rest, means of six reads.
export var CYCLE_FRAMES = 70;
export var EXIT_FRAMES = 23;
export var GAP_FRAMES = 4;
export var ENTRY_FRAMES = 26;
var START_SCALE = 2.8; // assumed: the two entries read from their first frame both gave about 2.8
var EXIT_DROP = 0.44;
export var exitEase = bezier(0.66, 0.04, 0.69, 0.48);
export var entryEase = bezier(0.25, 0.97, 0.33, 0.95);
var EXIT_GAIN = [0, 1, 18, 1, 19.5, 0.9, 20.5, 0.8, 21.5, 0.6, 22.5, 0.3, 23, 0];
var ENTRY_GAIN = [0, 0.35, 3, 1, 5, 1.3, 10, 1.15, 18, 1.03, 26, 1];
// Gaussian sigma in px at 1920, on the area near the zoom centre.
var EXIT_SIGMA = [0.56, 9, 0.7, 9, 0.77, 8.4, 0.82, 7.4, 0.85, 6.2, 0.88, 4.8, 0.92, 2.8, 0.93, 0.5, 1, 0];
// First value assumed: the entry reads 25 px or more at 1920 with low match scores.
var ENTRY_SIGMA = [0, 30, 5.7, 19, 8, 10, 11, 8, 14, 3.9, 17, 2.5, 20, 0];
var SCATTER = [0.74, 0.004, 0.88, 0.002, 0.93, 0];
var TINT_FRAMES = 6;

// Headline colour: green falls first, so white passes through pink.
var OUT_R = [0, 1, 3, 1, 5, 0.74, 7, 0.5];
var OUT_G = [0, 1, 2, 0.8, 3, 0.62, 4, 0.42, 5, 0.32, 7, 0.28];
var OUT_B = [0, 1, 2, 0.92, 3, 0.8, 4, 0.58, 5, 0.46, 7, 0.38];
var OUT_A = [0, 1, 4, 1, 7, 0];

// Ring A, the splash ring: centre (0.498, 0.804), radius 33 px at 1920, stroke
// 8 px. Clockwise, 347 degrees per second. Three spells, all tied to loading.
var RING_A = { u: 0.498, v: 0.804, radius: 33, stroke: 8, speed: 347 };
var SPELL_1 = [2, 71];
var SPELL_2 = [192, 202];
var SPELL_3 = [254, 402];
// Ring B, on black before the landing: centre (0.500, 0.497), radius 44 px,
// stroke 9 px, arc 288 degrees, clockwise at 446 degrees per second. It is
// first seen over the stripes and holds still for its last four frames.
var RING_B = { u: 0.5, v: 0.497, radius: 44, stroke: 9, speed: 446, arc: 288 };
var RING_B_IN = 433;
var RING_B_STOP = 450;

export interface Plan {
  headlineCount: number;
  holdMs: number; // the sequence waits here while the title art loads
  landingStartMs: number;
  wallSet: number;
  setCount: number;
}

export interface LayerDraw {
  set: number;
  scale: number;
  sigma: number;
  alpha: number;
  gain: number;
  scatter: number;
  underText: number; // 1 tiles under the headline show, 0 they are gone
}

export interface TextDraw {
  headline: number;
  line: number;
  alpha: number;
  r: number;
  g: number;
  b: number;
}

export interface Scene {
  stage: number; // 0 opening, 1 landing
  // Opening wordmark: a box in u and v, clipped on the right as letters go.
  wordAlpha: number;
  wordU0: number;
  wordU1: number;
  wordClip: number;
  wordV0: number;
  wordV1: number;
  // Opening mark.
  markAlpha: number;
  markU0: number;
  markU1: number;
  markV0: number;
  markV1: number;
  filament: number;
  lines: number;
  column: number; // half width of the column, share of picture width
  columnAlpha: number;
  zoomWarm: number;
  zoomCool: number;
  split: number;
  fade: number;
  ringAlpha: number;
  ringU: number;
  ringV: number;
  ringRadiusPx: number;
  ringStrokePx: number;
  ringArcDeg: number;
  ringAngleDeg: number;
  // Landing
  cardShift: number;
  emptyFill: number;
  band: number;
  peek: number;
  cardWordAlpha: number;
  cardWordScale: number;
  layerCount: number;
  layers: LayerDraw[];
  textCount: number;
  texts: TextDraw[];
}

export function newScene(): Scene {
  var layers: LayerDraw[] = [];
  var texts: TextDraw[] = [];
  for (var i = 0; i < 2; i++) layers.push({ set: 0, scale: 1, sigma: 0, alpha: 0, gain: 1, scatter: 0, underText: 0 });
  for (var j = 0; j < 4; j++) texts.push({ headline: 0, line: 0, alpha: 0, r: 1, g: 1, b: 1 });
  return {
    stage: 0, wordAlpha: 0, wordU0: 0, wordU1: 0, wordClip: 1, wordV0: 0, wordV1: 0,
    markAlpha: 0, markU0: 0, markU1: 0, markV0: 0, markV1: 0,
    filament: 0, lines: 0, column: 0, columnAlpha: 0, zoomWarm: 1, zoomCool: 1,
    split: 0, fade: 1, ringAlpha: 0, ringU: 0.5, ringV: 0.5, ringRadiusPx: 0, ringStrokePx: 0,
    ringArcDeg: 0, ringAngleDeg: 0, cardShift: 0, emptyFill: 0, band: 0, peek: 0, cardWordAlpha: 0,
    cardWordScale: 1, layerCount: 0, layers: layers, textCount: 0, texts: texts
  };
}

// Tile sets: the first headline has its own set, the second and third share
// one, and each later headline has its own. The wall comes last.
export function setForHeadline(index: number): number {
  if (index === 0) return 0;
  if (index <= 2) return 1;
  return index - 1;
}

export function makePlan(headlineCount: number): Plan {
  var wall = headlineCount > 3 ? headlineCount - 1 : 2;
  return {
    headlineCount: headlineCount,
    holdMs: F_STRETCH * FRAME_MS,
    landingStartMs: F_CUT * FRAME_MS,
    wallSet: wall,
    setCount: wall + 1
  };
}

// Time at which a given layer cycle starts, used by previews and tests.
export function cycleStartMs(plan: Plan, cycle: number): number {
  return plan.landingStartMs + (CYCLE_START + cycle * CYCLE_FRAMES) * FRAME_MS;
}

function ringAt(s: Scene, r: { u: number; v: number; radius: number; stroke: number; speed: number }, arc: number, seconds: number): void {
  s.ringAlpha = 1;
  s.ringU = r.u;
  s.ringV = r.v;
  s.ringRadiusPx = r.radius;
  s.ringStrokePx = r.stroke;
  s.ringArcDeg = arc;
  s.ringAngleDeg = (seconds * r.speed) % 360;
}

// Ring A runs in three spells. Spell 3 outlasts the stretch onset by 8 frames.
function ringAAt(s: Scene, f: number): void {
  s.ringAlpha = 0;
  if (f >= SPELL_1[0] && f < SPELL_1[1]) ringAt(s, RING_A, 172, f / 24);
  else if (f >= SPELL_2[0] && f < SPELL_2[1]) ringAt(s, RING_A, table([192, 195, 202, 250], f), f / 24);
  else if (f >= SPELL_3[0] && f < SPELL_3[1]) {
    ringAt(s, RING_A, table([254, 190, 269, 280, 396, 280, 402, 200], f), f / 24);
  }
}

function logoBox(s: Scene, wordHalf: number): void {
  s.wordU0 = WORD_REST.u - wordHalf;
  s.wordU1 = WORD_REST.u + wordHalf;
  s.wordClip = s.wordU1;
  s.wordV0 = WORD_REST.v0;
  s.wordV1 = WORD_REST.v1;
  s.wordAlpha = 1;
  s.markAlpha = 0;
  s.markU0 = 0;
  s.markU1 = 0;
  s.markV0 = MARK_REST.v0;
  s.markV1 = MARK_REST.v1;
}

function markBox(s: Scene, half: number): void {
  s.markAlpha = 1;
  s.markU0 = MARK_REST.u - half;
  s.markU1 = MARK_REST.u + half;
  s.markV0 = MARK_REST.v0;
  s.markV1 = MARK_REST.v1;
  s.wordAlpha = 0;
}

function clearOpening(s: Scene): void {
  s.stage = 0;
  s.filament = 0;
  s.lines = 0;
  s.column = 0;
  s.columnAlpha = 0;
  s.zoomWarm = 1;
  s.zoomCool = 1;
  s.split = 0;
  s.fade = 1;
}

// Frames 0 to F_COLLAPSE: the wordmark rests.
function wordRestAt(s: Scene, f: number, wordHalf: number, markHalf: number): void {
  clearOpening(s);
  logoBox(s, wordHalf);
  s.markU0 = MARK_REST.u - markHalf;
  s.markU1 = MARK_REST.u + markHalf;
  ringAAt(s, f);
}

// F_COLLAPSE to F_MARK_REST: the word slides right and loses letters from the
// right, the mark takes over its box.
function collapseAt(s: Scene, f: number, wordHalf: number, markHalf: number): void {
  clearOpening(s);
  var e = f - F_COLLAPSE;
  var rest0 = WORD_REST.u - wordHalf;
  var rest1 = WORD_REST.u + wordHalf;
  var mark0 = MARK_REST.u - markHalf;
  var mark1 = MARK_REST.u + markHalf;
  var slide = collapseEase(e / COLLAPSE_FRAMES) * (mark0 - rest0);
  var refSpan = 0.695 - 0.527;
  var ourSpan = rest1 - mark1;
  var clip = rest1 - (0.695 - table(WORD_RIGHT_REF, e)) * (ourSpan / refSpan);
  s.wordU0 = rest0 + slide;
  s.wordU1 = rest1 + slide;
  s.wordClip = Math.min(rest1, Math.max(mark1, clip));
  s.wordV0 = WORD_REST.v0;
  s.wordV1 = table(WORD_BOTTOM, e);
  s.wordAlpha = table(WORD_OUT, e);
  s.markAlpha = table(MARK_IN, e);
  s.markU0 = mark0;
  s.markU1 = mark1;
  s.markV0 = MARK_REST.v0;
  s.markV1 = MARK_REST.v1;
  ringAAt(s, f);
}

// F_MARK_REST to F_STRETCH: the mark rests.
function markRestAt(s: Scene, f: number, markHalf: number): void {
  clearOpening(s);
  markBox(s, markHalf);
  ringAAt(s, f);
}

// Each family is pushed out from the centre by its own accelerating zoom.
function zoomOf(a: number, b: number, t: number): number {
  return Math.exp(a * t + b * t * t * t);
}

// F_STRETCH to F_CUT: the mark stretches, folds into a column, bursts to full
// width, splits into two colour families, runs outward, fades, then the ring.
function openingAt(s: Scene, f: number): void {
  s.stage = 0;
  var e = f - F_STRETCH;
  // Our mark is drawn back to the reference glyph's own width as it folds.
  var wide = e < FOLD_FROM ? MARK_WIDE : lerp(MARK_WIDE, 1, (e - FOLD_FROM) / (23 - FOLD_FROM));
  var half = table(STRETCH_HALF, e) * wide;
  var mid = table(STRETCH_MID, e);
  s.wordAlpha = 0;
  s.wordClip = 1;
  s.markAlpha = 1 - smooth(FOLD_FROM + 6, 23, e);
  s.markU0 = mid - half;
  s.markU1 = mid + half;
  s.markV0 = table(STRETCH_TOP, e);
  s.markV1 = table(STRETCH_BOTTOM, e);
  s.filament = smooth(FILAMENT[0], FILAMENT[2], e);
  s.lines = smooth(LINES[0], LINES[2], e);
  s.column = (e < 23 ? table(COLUMN_GROW, e) : table(COLUMN, e)) / 2;
  s.columnAlpha = smooth(16, 21, e);
  // Below full width the pattern is held squeezed to the column's own width.
  var t = (f - F_SPLIT) / 24;
  var squeeze = Math.max(0.35, table(COLUMN, e));
  s.zoomWarm = f < F_FULL ? squeeze : f < F_SPLIT ? 1 : zoomOf(WARM_A, WARM_B, t);
  s.zoomCool = f < F_FULL ? squeeze : f < F_SPLIT ? 1 : zoomOf(COOL_A, COOL_B, t);
  s.split = f >= F_SPLIT ? 1 : 0;
  s.fade = clamp01(1 - (f - FADE_FROM) / FADE_SPAN);
  if (f < SPELL_3[1]) ringAAt(s, f);
  else if (f >= RING_B_IN) ringAt(s, RING_B, RING_B.arc, Math.min(f, RING_B_STOP) / 24);
  else s.ringAlpha = 0;
}

export function markHeightOf(): number {
  return MARK_REST.v1 - MARK_REST.v0;
}

function tint(t: TextDraw, frame: number, alpha: number): void {
  // frame 0 is the start of the move out; 7 is gone.
  t.r = table(OUT_R, frame);
  t.g = table(OUT_G, frame);
  t.b = table(OUT_B, frame);
  t.alpha = alpha * table(OUT_A, frame);
}

function addLine(s: Scene, headline: number, line: number): TextDraw {
  var t = s.texts[s.textCount++];
  t.headline = headline;
  t.line = line;
  t.alpha = 1;
  t.r = 1;
  t.g = 1;
  t.b = 1;
  return t;
}

// A flat headline move: out over seven frames, in over six, line 2 three frames late.
function flatText(s: Scene, headline: number, inFrame: number, outFrame: number, f: number): void {
  if (f < inFrame || f >= outFrame + TEXT_OUT_FRAMES) return;
  for (var line = 0; line < 2; line++) {
    var start = inFrame + line * LINE_LAG;
    if (f < start) continue;
    var t = addLine(s, headline, line);
    if (f < start + TEXT_IN_FRAMES) {
      tint(t, ((start + TEXT_IN_FRAMES - f) / TEXT_IN_FRAMES) * TEXT_OUT_FRAMES, 1);
    } else if (f >= outFrame) {
      tint(t, f - outFrame, 1);
    }
  }
}

function layerOf(s: Scene, set: number, scale: number, sigma: number, alpha: number, gain: number, scatter: number, underText: number): void {
  var l = s.layers[s.layerCount++];
  l.set = set;
  l.scale = scale;
  l.sigma = sigma;
  l.alpha = alpha;
  l.gain = gain;
  l.scatter = scatter;
  l.underText = underText;
}

function entryScale(e: number, frames: number): number {
  return 1 + (START_SCALE - 1) * (1 - entryEase(e / frames));
}

function exitScale(e: number): number {
  return 1 - EXIT_DROP * exitEase(e / EXIT_FRAMES);
}

function layerText(s: Scene, headline: number, alpha: number, pink: number): void {
  for (var line = 0; line < 2; line++) {
    var t = addLine(s, headline, line);
    t.alpha = alpha;
    tint(t, pink * 5, alpha);
  }
}

function landingAt(plan: Plan, f: number, s: Scene): void {
  var n = plan.headlineCount;
  s.stage = 1;
  s.ringAlpha = 0;
  s.wordAlpha = 0;
  s.markAlpha = 0;
  s.layerCount = 0;
  s.textCount = 0;
  s.cardShift = table(SLIDE, f);
  s.emptyFill = 1 - table(WALL_IN, f);
  s.band = 1 - smooth(PEEK_IN[0], PEEK_IN[2], f);
  s.peek = table(PEEK_IN, f);
  s.cardWordAlpha = table(MARK_COVER, f) * table(MARK_FADE, f);
  s.cardWordScale = table(MARK_SCALE, f);

  if (f >= F_WALL_IN && f < 77) {
    layerOf(s, plan.wallSet, 1, 0, table(WALL_IN, f) * (1 - table(WALL_OUT, f)), 1, 0, 1);
  }

  if (f >= F_FIRST_ENTRY && f < FOURTH_ENTRY) {
    var sigma = table(ENTRY_SIGMA, f - F_FIRST_ENTRY);
    var scale = table(FIRST_SCALE, f);
    var xfade = clamp01((f - SECOND_CROSSFADE) / CROSSFADE_FRAMES);
    var leave = clamp01((f - F_TEXT_OUT_4) / (FOURTH_ENTRY - F_TEXT_OUT_4));
    var firstGain = f < F_FIRST_REST ? table(ENTRY_GAIN, f - F_FIRST_ENTRY) : 1;
    // Tiles under the first headline leave just before its text arrives.
    var underText = 1 - smooth(F_TEXT_IN - 11, F_TEXT_IN - 1, f);
    if (xfade < 1) layerOf(s, setForHeadline(0), scale, sigma, Math.min(1, firstGain), Math.max(1, firstGain), 0, underText);
    if (xfade > 0) layerOf(s, setForHeadline(1 % n), 1, 0, xfade * (1 - leave), 1, 0, 0);
    flatText(s, 0, F_TEXT_IN, F_TEXT_OUT_2, f);
    if (n > 1) flatText(s, 1, SECOND_CROSSFADE, F_TEXT_OUT_3, f);
    if (n > 2) flatText(s, 2, F_TEXT_OUT_3 + 6, F_TEXT_OUT_4, f);
  } else if (f >= FOURTH_ENTRY) {
    if (f < CYCLE_START) {
      var first = 3 % n;
      var e0 = f - FOURTH_ENTRY;
      if (e0 < FOURTH_ENTRY_FRAMES) {
        var gain0 = table(ENTRY_GAIN, (e0 * ENTRY_FRAMES) / FOURTH_ENTRY_FRAMES);
        var pink0 = clamp01(1 - e0 / TINT_FRAMES);
        layerOf(s, setForHeadline(first), entryScale(e0, FOURTH_ENTRY_FRAMES), table(ENTRY_SIGMA, e0), Math.min(1, gain0), Math.max(1, gain0), 0, 0);
        layerText(s, first, 1, pink0);
      } else {
        layerOf(s, setForHeadline(first), 1, 0, 1, 1, 0, 0);
        layerText(s, first, 1, 0);
      }
    } else {
      var since = f - CYCLE_START;
      var cycle = Math.floor(since / CYCLE_FRAMES);
      var c = since - cycle * CYCLE_FRAMES;
      var leaving = (3 + cycle) % n;
      var arriving = (3 + cycle + 1) % n;
      if (c < EXIT_FRAMES) {
        var exitS = exitScale(c);
        var exitGain = table(EXIT_GAIN, c);
        layerOf(s, setForHeadline(leaving), exitS, table(EXIT_SIGMA, exitS), exitGain, 1, table(SCATTER, exitS), 0);
        layerText(s, leaving, 1, clamp01((c - (EXIT_FRAMES - TINT_FRAMES)) / TINT_FRAMES));
      } else if (c >= EXIT_FRAMES + GAP_FRAMES) {
        var e = c - EXIT_FRAMES - GAP_FRAMES;
        if (e < ENTRY_FRAMES) {
          var entryGain = table(ENTRY_GAIN, e);
          layerOf(s, setForHeadline(arriving), entryScale(e, ENTRY_FRAMES), table(ENTRY_SIGMA, e), Math.min(1, entryGain), Math.max(1, entryGain), 0, 0);
          layerText(s, arriving, 1, clamp01(1 - e / TINT_FRAMES));
        } else {
          layerOf(s, setForHeadline(arriving), 1, 0, 1, 1, 0, 0);
          layerText(s, arriving, 1, 0);
        }
      }
    }
  }
}

// wordHalf and markHalf are half the rest width of each, which follows from the
// shape of the files in brand/ and so is not a measured value.
export function sceneAt(plan: Plan, ms: number, wordHalf: number, markHalf: number, s: Scene): Scene {
  var f = ms / FRAME_MS;
  // A time meant to be a whole frame can come back a hair under it, and a phase
  // that starts on that frame would then start one frame late.
  var whole = Math.round(f);
  if (Math.abs(f - whole) < 1e-4) f = whole;
  if (f < F_COLLAPSE) wordRestAt(s, f, wordHalf, markHalf);
  else if (f < F_MARK_REST) collapseAt(s, f, wordHalf, markHalf);
  else if (f < F_STRETCH) markRestAt(s, f, markHalf);
  else if (f < F_CUT) openingAt(s, f);
  else landingAt(plan, f - F_CUT, s);
  return s;
}
