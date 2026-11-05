// Builds the 1D stripe pattern used by the opening scene.
//
// The texture is 2048 by 4. Row 0 is every colour mixed, row 1 is the parted
// pattern (violet left, blue right, dark gap at the centre), row 2 is a slow
// brightness sheen, row 3 is thin threads that cut the logo apart.
// Stripes are packed tighter near the centre, so the zoom keeps showing stripes
// of the same apparent width as it pushes through them.

import { rng } from "./curves.ts";

export var STRIPE_WIDTH = 2048;
export var STRIPE_ROWS = 4;

// Group edges in pattern space, shares of width from the centre.
var VIOLET_EDGE = 0.034;
var BLUE_EDGE = 0.052;
var GAP = 0.008;

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

function centred(r: number, sign: number, power: number): number {
  return sign * Math.pow(r, power) * 0.5;
}

export function buildStripes(colours: StripeColours, seed: number): Uint8Array {
  var random = rng(seed);
  var violet = colours.violet.map(rgb);
  var blue = colours.blue.map(rgb);
  var white = rgb(colours.white);
  var pink = rgb(colours.pink);
  var cyan = rgb(colours.cyan);
  var mixed = new Float32Array(STRIPE_WIDTH * 3);
  var parted = new Float32Array(STRIPE_WIDTH * 3);
  var i: number;

  for (i = 0; i < 260; i++) {
    var family = random() < 0.56 ? violet : blue;
    stroke(mixed, random() - 0.5, 0.0016 + random() * 0.0085, pick(family, random()), 1, true);
  }
  for (i = 0; i < 46; i++) {
    var hi = random();
    var thin = hi < 0.3 ? white : hi < 0.65 ? pink : cyan;
    stroke(mixed, random() - 0.5, 0.0005 + random() * 0.0009, thin, 0.9, true);
  }

  // Violet family: spaced stripes over near black.
  for (i = 0; i < 120; i++) {
    var xv = centred(random(), random() < 0.9 ? -1 : 0.066, 1.7);
    var wv = 0.0008 + Math.abs(xv) * (0.004 + random() * 0.02);
    stroke(parted, xv, wv, pick(violet, random()), 1, true);
  }
  // Blue family: a dense block with darker and lighter bands.
  for (i = 0; i < 150; i++) {
    var xb = BLUE_EDGE + Math.pow(random(), 1.2) * (0.5 - BLUE_EDGE);
    stroke(parted, xb, 0.002 + random() * 0.016, pick(blue, random()), 1, true);
  }
  for (i = 0; i < 30; i++) {
    var xc = BLUE_EDGE + random() * (0.5 - BLUE_EDGE);
    stroke(parted, xc, 0.0005 + random() * 0.0009, random() < 0.75 ? cyan : white, 0.9, false);
  }
  // Thin white and pink lines hug the dark gap on both sides.
  for (i = 0; i < 30; i++) {
    var side = random() < 0.62 ? -1 : 1;
    var xh = side * (GAP + Math.pow(random(), 2) * (side < 0 ? 0.14 : VIOLET_EDGE - GAP));
    stroke(parted, xh, 0.0002 + random() * 0.0006, random() < 0.55 ? pink : white, 1, true);
  }
  for (i = 0; i < 16; i++) {
    stroke(parted, -0.5 * Math.pow(random(), 1.3), 0.0005 + random() * 0.0008, random() < 0.65 ? pink : white, 0.9, true);
  }

  var data = new Uint8Array(STRIPE_WIDTH * STRIPE_ROWS * 4);
  for (i = 0; i < STRIPE_WIDTH; i++) {
    var pos = i / STRIPE_WIDTH - 0.5;
    var gap = Math.min(1, Math.max(0, (Math.abs(pos) - GAP * 0.6) / (GAP * 0.8)));
    var parting = pos > VIOLET_EDGE && pos < BLUE_EDGE ? 0.05 : 1;
    for (var c = 0; c < 3; c++) {
      data[i * 4 + c] = Math.min(255, mixed[i * 3 + c] * 255);
      data[(STRIPE_WIDTH + i) * 4 + c] = Math.min(255, parted[i * 3 + c] * 255 * gap * parting);
    }
    data[i * 4 + 3] = 255;
    data[(STRIPE_WIDTH + i) * 4 + 3] = 255;
  }

  var sheen = 0.5;
  var thread = 0;
  for (i = 0; i < STRIPE_WIDTH; i++) {
    sheen += (random() - 0.5) * 0.09;
    sheen = sheen < 0 ? 0 : sheen > 1 ? 1 : sheen;
    if (i % 5 === 0) thread = random();
    data[(STRIPE_WIDTH * 2 + i) * 4] = sheen * 255;
    data[(STRIPE_WIDTH * 3 + i) * 4] = thread * 255;
    data[(STRIPE_WIDTH * 2 + i) * 4 + 3] = 255;
    data[(STRIPE_WIDTH * 3 + i) * 4 + 3] = 255;
  }
  return data;
}
