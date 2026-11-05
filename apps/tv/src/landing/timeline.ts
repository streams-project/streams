// The whole sequence as a pure function of time.
//
// Every number in this file comes from frame by frame readings of a 24 fps
// recording of a reference app on a real TV, unless marked "own" (our own
// design choice) or "assumed" (the reading could not settle it). Frame counts
// are kept as read, so a duration is good to plus or minus 42 ms.
//
// sceneAt() fills a reused Scene object and allocates nothing, so a slow TV
// does not collect garbage in the middle of a move.

import { bezier, clamp01, smooth, table } from "./curves.ts";

export var FRAME_MS = 1000 / 24;

// Measured geometry, shares of picture width (u) and height (v).
export var CARD = { u0: 0.0287, u1: 0.8315, v0: 0.1565, v1: 0.9567, radiusPx: 24 };
export var PEEK = { u0: 0.852, v0: 0.155, v1: 0.957 };
export var PIVOT = { u: 0.43, v: 0.557 };
export var HEADLINE_U = 0.431;
export var TWO_LINE_V = [0.463, 0.582];
export var ONE_LINE_V = [0.558, 0.668, 0.357, 0.558, 0.668, 0.357, 0.271];
export var CAP_HEIGHT = { twoLine: 0.072, oneLine: 0.063 };
export var LOGO_REST_V = 0.175;

// Opening. Frames count from the start of the logo stretch.
export var OPENING_FRAMES = 60;
var HEIGHT = [0, 1, 1, 1.012, 3, 1.05, 6, 1.15, 9, 1.37, 12, 1.77, 15, 2.39, 18, 3.39, 20, 4.35, 22, 5.05, 23, 6.2];
var EXTRA_WIDTH = [0, 0, 3, 0.0021, 6, 0.0094, 9, 0.0177, 12, 0.0395, 15, 0.0739, 18, 0.1213];
var COLUMN = [23, 0.105, 25, 0.137, 27, 0.171, 28, 0.241, 29, 0.305, 30, 0.638, 31, 0.994, 32, 1];
// Own: before frame 23 the column grows out of the folding logo, from the
// width of its first bar, the way the reference's stem widens into it.
var COLUMN_GROW = [17, 0.02, 20, 0.045, 22, 0.08, 23, 0.105];
var FOLD_START = 18;
var FOLD_END = 23;
var SPLIT = 34;
var RUN_FRAMES = 15;
var FADE_START = 44;
var FADE_END = 50;

// Stripe push. Rate per second rises through the run; the fit reproduces the
// measured group edges within 0.03 W (see test/timeline.test.ts).
var RUN_RATE_START = 2.5;
var RUN_RATE_GAIN = 5.5;

// Landing. Frames count from the hard cut to the card.
var SLIDE_FRAMES = 9;
var SLIDE_START = 0.97; // assumed: card starts just off the right edge
var SLIDE_HALVING = 1.1;
var WALL_IN = [15, 0, 16, 0.49, 17, 0.9, 18, 1];
var WALL_OUT = [62, 0.06, 64, 0.19, 66, 0.29, 68, 0.39, 70, 0.52, 72, 0.67, 74, 0.84, 76, 0.97, 77, 1];
var MARK_COVER = [62, 0, 64, 0.3, 68, 1];
var MARK_SCALE = [76, 1, 80, 0.958, 84, 0.868, 92, 0.62];
var MARK_FADE = [84, 1, 92, 0];
var FIRST_ENTRY = 84;
var FIRST_REST = 126;
var FIRST_SCALE = [84, 2.8, 88, 2.36, 92, 1.85, 96, 1.48, 100, 1.31, 104, 1.22, 108, 1.16, 112, 1.09, 116, 1.04, 120, 1.015, 124, 1.004, 126, 1];
var FIRST_TEXT_IN = 104;
var TEXT_OUT_FRAMES = 7;
var TEXT_IN_FRAMES = 6;
var LINE_LAG = 3;
var SECOND_TEXT_OUT = 153;
var SECOND_CROSSFADE = 159;
var CROSSFADE_FRAMES = 3;
var THIRD_TEXT_OUT = 210;
var FOURTH_TEXT_OUT = 267;
var FOURTH_ENTRY = 273;
var FOURTH_ENTRY_FRAMES = 31;
var PEEK_IN = [260, 0, 274, 1];
var CYCLE_START = 322;

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

export interface Plan {
  logoRestMs: number;
  headlineCount: number;
  openingStartMs: number;
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
  // Opening
  logoAlpha: number;
  logoWidth: number; // share of width, 0 means the rest width
  logoHeight: number; // multiple of rest height
  filament: number;
  lines: number;
  column: number;
  columnAlpha: number;
  zoom: number;
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
  markAlpha: number;
  markScale: number;
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
    stage: 0, logoAlpha: 1, logoWidth: 0, logoHeight: 1, filament: 0, lines: 0, column: 0, columnAlpha: 0, zoom: 1,
    split: 0, fade: 1, ringAlpha: 0, ringU: 0.5, ringV: 0.5, ringRadiusPx: 0, ringStrokePx: 0,
    ringArcDeg: 0, ringAngleDeg: 0, cardShift: 0, emptyFill: 0, band: 0, peek: 0, markAlpha: 0,
    markScale: 1, layerCount: 0, layers: layers, textCount: 0, texts: texts
  };
}

// Tile sets: the first headline has its own set, the second and third share
// one, and each later headline has its own. The wall comes last.
export function setForHeadline(index: number): number {
  if (index === 0) return 0;
  if (index <= 2) return 1;
  return index - 1;
}

export function makePlan(logoRestMs: number, headlineCount: number): Plan {
  var wall = headlineCount > 3 ? headlineCount - 1 : 2;
  return {
    logoRestMs: logoRestMs,
    headlineCount: headlineCount,
    openingStartMs: logoRestMs,
    landingStartMs: logoRestMs + OPENING_FRAMES * FRAME_MS,
    wallSet: wall,
    setCount: wall + 1
  };
}

// Time at which a given layer cycle starts, used by previews and tests.
export function cycleStartMs(plan: Plan, cycle: number): number {
  return plan.landingStartMs + (CYCLE_START + cycle * CYCLE_FRAMES) * FRAME_MS;
}

function ringAt(s: Scene, alpha: number, u: number, v: number, radius: number, stroke: number, arc: number, degPerSecond: number, seconds: number): void {
  s.ringAlpha = alpha;
  s.ringU = u;
  s.ringV = v;
  s.ringRadiusPx = radius;
  s.ringStrokePx = stroke;
  s.ringArcDeg = arc;
  s.ringAngleDeg = (seconds * degPerSecond) % 360;
}

function logoRestAt(ms: number, s: Scene): void {
  s.stage = 0;
  s.logoAlpha = 1;
  s.logoWidth = 0;
  s.logoHeight = 1;
  s.filament = 0;
  s.lines = 0;
  s.column = 0;
  s.columnAlpha = 0;
  s.zoom = 1;
  s.split = 0;
  s.fade = 1;
  var arc = table([0, 190, 15, 280], ms / FRAME_MS);
  ringAt(s, 1, 0.5, 0.804, 33, 8, arc, 347, ms / 1000);
}

function runZoom(f: number): number {
  // Integral of the rising rate from the colour split to frame f.
  var t = (f - SPLIT) / 24;
  var total = RUN_FRAMES / 24;
  return Math.exp(RUN_RATE_START * t + (RUN_RATE_GAIN * t * t * t) / (3 * total * total));
}

function openingAt(plan: Plan, f: number, logoRestWidth: number, s: Scene): void {
  s.stage = 0;
  s.logoHeight = table(HEIGHT, f);
  if (f <= FOLD_START) {
    s.logoWidth = logoRestWidth + table(EXTRA_WIDTH, f);
  } else {
    var fold = clamp01((f - FOLD_START) / (FOLD_END - FOLD_START));
    s.logoWidth = logoRestWidth + 0.1213 + (0.105 - logoRestWidth - 0.1213) * Math.pow(fold, 1.4);
  }
  s.logoAlpha = 1 - smooth(FOLD_START + 1, FOLD_END + 1, f);
  s.filament = smooth(11, 18, f);
  s.lines = smooth(16, 22, f);
  s.column = (f < FOLD_END ? table(COLUMN_GROW, f) : table(COLUMN, f)) / 2;
  s.columnAlpha = smooth(16, 21, f);
  // Below full width the stripe pattern is held at a third of its final size.
  s.zoom = f < 32 ? Math.max(0.35, table(COLUMN, f)) : f < SPLIT ? 1 : runZoom(f);
  s.split = f >= SPLIT ? 1 : 0;
  s.fade = 1 - clamp01((f - FADE_START) / (FADE_END - FADE_START));
  // Ring A outlasts the stretch onset by seven frames, shrinking over the last six.
  if (f < 7) {
    ringAt(s, 1, 0.5, 0.804, 33, 8, table([1, 260, 7, 200], f), 347, plan.logoRestMs / 1000 + f / 24);
  } else if (f >= 39) {
    // Ring B holds still for its last four frames.
    var spin = Math.min(f, 56);
    ringAt(s, 1, 0.5, 0.497, 44, 9, 288, 446, spin / 24);
  } else {
    s.ringAlpha = 0;
  }
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
  s.logoAlpha = 0;
  s.layerCount = 0;
  s.textCount = 0;
  s.cardShift = f < SLIDE_FRAMES ? SLIDE_START * Math.pow(2, -SLIDE_HALVING * f) : 0;
  s.emptyFill = 1 - table(WALL_IN, f);
  s.band = 1 - smooth(PEEK_IN[0], PEEK_IN[2], f);
  s.peek = table(PEEK_IN, f);
  s.markAlpha = table(MARK_COVER, f) * table(MARK_FADE, f);
  s.markScale = table(MARK_SCALE, f);

  if (f >= WALL_IN[0] && f < 77) {
    layerOf(s, plan.wallSet, 1, 0, table(WALL_IN, f) * (1 - table(WALL_OUT, f)), 1, 0, 1);
  }

  if (f >= FIRST_ENTRY && f < FOURTH_ENTRY) {
    var sigma = table(ENTRY_SIGMA, f - FIRST_ENTRY);
    var scale = table(FIRST_SCALE, f);
    var xfade = clamp01((f - SECOND_CROSSFADE) / CROSSFADE_FRAMES);
    var leave = clamp01((f - FOURTH_TEXT_OUT) / (FOURTH_ENTRY - FOURTH_TEXT_OUT));
    var firstGain = f < FIRST_REST ? table(ENTRY_GAIN, f - FIRST_ENTRY) : 1;
    // Tiles under the first headline leave as its text arrives.
    var underText = 1 - smooth(FIRST_TEXT_IN - 6, FIRST_TEXT_IN + 2, f);
    if (xfade < 1) layerOf(s, setForHeadline(0), scale, sigma, Math.min(1, firstGain), Math.max(1, firstGain), 0, underText);
    if (xfade > 0) layerOf(s, setForHeadline(1 % n), 1, 0, xfade * (1 - leave), 1, 0, 0);
    flatText(s, 0, FIRST_TEXT_IN, SECOND_TEXT_OUT, f);
    if (n > 1) flatText(s, 1, SECOND_CROSSFADE, THIRD_TEXT_OUT, f);
    if (n > 2) flatText(s, 2, THIRD_TEXT_OUT + 6, FOURTH_TEXT_OUT, f);
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

export function sceneAt(plan: Plan, ms: number, logoRestWidth: number, s: Scene): Scene {
  if (ms < plan.openingStartMs) {
    logoRestAt(ms, s);
  } else if (ms < plan.landingStartMs) {
    openingAt(plan, (ms - plan.openingStartMs) / FRAME_MS, logoRestWidth, s);
  } else {
    landingAt(plan, (ms - plan.landingStartMs) / FRAME_MS, s);
  }
  return s;
}
