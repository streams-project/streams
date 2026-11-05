// Builds the 1D stripe pattern used by the opening scene.
//
// The texture is 2048 by 8. Row 0 is every colour mixed, which is what the
// column and the burst show. Rows 1 and 2 hold the two families the colour
// split parts the picture into: row 1 violet, filling the left and reaching a
// little past the centre, row 2 blue, from further out to the right edge. The
// two are pushed outward by their own zooms, so the dark parting between them
// opens as the run goes on. Row 3 is a slow brightness sheen and row 4 thin
// threads that cut the mark apart while it folds.
//
// Both families leave a dark gap at the centre: half width 0.0037 of the
// pattern, which the violet family's zoom carries out to the measured 0.008 W
// six frames after the split and 0.014 W at ten.
//
// Stripes are packed tighter near the centre, so the push keeps showing stripes
// of about the same apparent width as it runs through them.

import { rng } from "./curves.ts";

export var STRIPE_WIDTH = 2048;
export var STRIPE_ROWS = 8;
export var ROW_MIXED = 0.5 / STRIPE_ROWS;
export var ROW_WARM = 1.5 / STRIPE_ROWS;
export var ROW_COOL = 2.5 / STRIPE_ROWS;
export var ROW_SHEEN = 3.5 / STRIPE_ROWS;
export var ROW_THREAD = 4.5 / STRIPE_ROWS;

// Group edges in pattern space, shares of width from the centre. The violet
// family's right edge and the blue family's left edge are the two the zooms
// were fitted to; the gap is the measured dark centre.
export var VIOLET_EDGE = 0.034;
export var BLUE_EDGE = 0.052;
export var GAP = 0.0037;

export interface StripeColours {
  violet: string[];
  blue: string[];
  white: string;
  pink: string;
  cyan: string;
}

function rgb(hex: string): number[] {
  var v = parseInt(hex.slice(1), 16);
  return [(v >> 16) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255];
}

// A highlight adds light. A base stripe paints over what is below, so the
// families keep their own colour and do not wash out to white.
function stroke(row: Float32Array, centre: number, width: number, colour: number[], gain: number, paint: boolean): void {
  var c = (centre + 0.5) * STRIPE_WIDTH;
  var w = Math.max(0.6, width * STRIPE_WIDTH);
  var a = Math.max(0, Math.floor(c - 3 * w));
  var b = Math.min(STRIPE_WIDTH - 1, Math.ceil(c + 3 * w));
  for (var i = a; i <= b; i++) {
    var d = (i - c) / w;
    // Flat topped, so a stripe reads as a band with a soft edge.
    var e = Math.exp(-d * d * d * d) * gain;
    for (var k = 0; k < 3; k++) {
      var at = i * 3 + k;
      row[at] = paint ? row[at] + (colour[k] - row[at]) * e : row[at] + colour[k] * e;
    }
  }
}

function pick<T>(list: T[], r: number): T {
  return list[Math.floor(r * list.length) % list.length];
}

function ramp(x: number): number {
  return x < 0 ? 0 : x > 1 ? 1 : x * x * (3 - 2 * x);
}

// Soft mask: 0 inside the dark gap, 1 outside it.
function gapMask(pos: number): number {
  return ramp((Math.abs(pos) - GAP) / (GAP * 0.55));
}

// Each family fades out over 0.0015 of the pattern at the parting, so the half
// bright point stays on the measured edge and the cut is not a hard line.
function edgeMask(pos: number, edge: number, inside: number): number {
  return ramp(0.5 + (inside * (edge - pos)) / 0.003);
}

export function buildStripes(colours: StripeColours, seed: number): Uint8Array {
  var random = rng(seed);
  var violet = colours.violet.map(rgb);
  var blue = colours.blue.map(rgb);
  var white = rgb(colours.white);
  var pink = rgb(colours.pink);
  var cyan = rgb(colours.cyan);
  var mixed = new Float32Array(STRIPE_WIDTH * 3);
  var warm = new Float32Array(STRIPE_WIDTH * 3);
  var cool = new Float32Array(STRIPE_WIDTH * 3);
  var i: number;

  // Row 0: both families mixed across the whole width, for the column and the burst.
  for (i = 0; i < 300; i++) {
    var family = random() < 0.56 ? violet : blue;
    stroke(mixed, random() - 0.5, 0.0012 + random() * 0.0075, pick(family, random()), 1, true);
  }
  for (i = 0; i < 54; i++) {
    var hi = random();
    stroke(mixed, random() - 0.5, 0.0005 + random() * 0.0009, hi < 0.3 ? white : hi < 0.65 ? pink : cyan, 0.9, true);
  }

  // Row 1, violet: spaced stripes over near black, packed towards the centre,
  // filling the left and stopping just past it on the right.
  for (i = 0; i < 150; i++) {
    var side = random() < 0.9 ? -1 : 1;
    var xv = side * Math.pow(random(), 1.7) * (side < 0 ? 0.5 : VIOLET_EDGE);
    var wv = 0.0007 + Math.abs(xv) * (0.004 + random() * 0.02);
    stroke(warm, xv, wv, pick(violet, random()), 1, true);
  }
  // Thin white and pink lines hug the gap on both sides.
  for (i = 0; i < 34; i++) {
    var sh = random() < 0.62 ? -1 : 1;
    var xh = sh * (GAP + Math.pow(random(), 2) * (sh < 0 ? 0.14 : VIOLET_EDGE - GAP));
    stroke(warm, xh, 0.0002 + random() * 0.0006, random() < 0.55 ? pink : white, 1, true);
  }
  for (i = 0; i < 18; i++) {
    stroke(warm, -0.5 * Math.pow(random(), 1.3), 0.0005 + random() * 0.0008, random() < 0.65 ? pink : white, 0.9, true);
  }

  // Row 2, blue: a dense block from the outer edge of the parting to the right edge.
  for (i = 0; i < 170; i++) {
    var xb = BLUE_EDGE + Math.pow(random(), 1.2) * (0.5 - BLUE_EDGE);
    stroke(cool, xb, 0.0018 + random() * 0.015, pick(blue, random()), 1, true);
  }
  for (i = 0; i < 34; i++) {
    var xc = BLUE_EDGE + random() * (0.5 - BLUE_EDGE);
    stroke(cool, xc, 0.0005 + random() * 0.0009, random() < 0.75 ? cyan : white, 0.9, false);
  }

  var data = new Uint8Array(STRIPE_WIDTH * STRIPE_ROWS * 4);
  for (i = 0; i < STRIPE_WIDTH; i++) {
    var pos = i / STRIPE_WIDTH - 0.5;
    var mask = gapMask(pos);
    var inWarm = edgeMask(pos, VIOLET_EDGE, 1);
    var inCool = edgeMask(pos, BLUE_EDGE, -1);
    for (var c = 0; c < 3; c++) {
      data[i * 4 + c] = Math.min(255, mixed[i * 3 + c] * 255);
      data[(STRIPE_WIDTH + i) * 4 + c] = Math.min(255, warm[i * 3 + c] * 255 * mask * inWarm);
      data[(STRIPE_WIDTH * 2 + i) * 4 + c] = Math.min(255, cool[i * 3 + c] * 255 * inCool);
    }
    data[i * 4 + 3] = 255;
    data[(STRIPE_WIDTH + i) * 4 + 3] = 255;
    data[(STRIPE_WIDTH * 2 + i) * 4 + 3] = 255;
  }

  var sheen = 0.5;
  var thread = 0;
  for (i = 0; i < STRIPE_WIDTH; i++) {
    sheen += (random() - 0.5) * 0.09;
    sheen = sheen < 0 ? 0 : sheen > 1 ? 1 : sheen;
    if (i % 5 === 0) thread = random();
    data[(STRIPE_WIDTH * 3 + i) * 4] = sheen * 255;
    data[(STRIPE_WIDTH * 4 + i) * 4] = thread * 255;
    data[(STRIPE_WIDTH * 3 + i) * 4 + 3] = 255;
    data[(STRIPE_WIDTH * 4 + i) * 4 + 3] = 255;
  }
  return data;
}
