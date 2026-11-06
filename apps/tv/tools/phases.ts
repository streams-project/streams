// Measures the signed out sequence out of its own rendered frames.
//
// Nothing here reads the timeline. The page is stepped one 24 fps frame at a
// time, each frame is read back off the canvas, and every frame is reduced to a
// few profiles and counts. The phase boundaries are then found in those numbers
// alone, and compared with the frame by frame reading of the reference that the
// motion spec records. `test/phases.test.ts` fails if any phase is more than one
// frame, 41.7 ms, away from its reading.
//
// Reductions, per frame, all on the canvas as drawn:
//   colMax      max luminance in each of the 1920 columns, over the rows that
//               hold no loading ring, so a spinning ring is not read as motion
//   rowMax      max luminance in each of the 1080 rows, same columns
//   hue         mean red minus green over 192 column bands, which parts the
//               violet family from the blue one
//   peak, mean  over the no ring rows
//   cardMax     max luminance per column inside the card rectangle
//   coreRow     mean luminance per row of the strip a headline is set in,
//               u 0.26 to 0.60, where the tiles under the live headline are
//               cleared away, so what moves there is the headline
//   cardMean    mean luminance inside the card
//   topMean     mean luminance of the card band above the wordmark, which only
//               the wall and the tile layers reach
//   peekMax     max luminance per column to the right of the card

import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";

export var FRAME_MS = 1000 / 24;

// The phases the motion spec reads, with the length it read, in milliseconds.
export interface Spec { key: string; what: string; frames: number; ms: number; }

export var SPEC: Spec[] = [
  { key: "wordmarkRest", what: "wordmark rest", frames: 108, ms: 4500 },
  { key: "wordmarkCollapse", what: "wordmark collapses to the mark", frames: 28, ms: 1167 },
  { key: "markRest", what: "mark rest", frames: 258, ms: 10750 },
  { key: "markStretch", what: "mark stretches and folds into a column", frames: 23, ms: 958 },
  { key: "columnWidens", what: "column widens, 0.105 to 0.305 W", frames: 6, ms: 250 },
  { key: "burst", what: "burst to full width", frames: 3, ms: 125 },
  { key: "fullWidth", what: "full width, one colour", frames: 3, ms: 125 },
  { key: "colourSplit", what: "colour split", frames: 1, ms: 42 },
  { key: "stripeRun", what: "stripes run outward from a dark centre gap", frames: 15, ms: 625 },
  { key: "fadeToBlack", what: "fade to black", frames: 6, ms: 250 },
  { key: "blackRing", what: "black with ring", frames: 10, ms: 417 },
  { key: "cardSlide", what: "card slides in from the right", frames: 8, ms: 333 },
  { key: "emptyCardRest", what: "empty card rest", frames: 8, ms: 333 },
  { key: "wallIn", what: "wall fades in", frames: 3, ms: 125 },
  { key: "wallRest", what: "wall rest", frames: 44, ms: 1830 },
  { key: "wallOut", what: "wall out while the wordmark fades in", frames: 14, ms: 583 },
  { key: "firstLayer", what: "wordmark leaves and the first tile layer settles", frames: 42, ms: 1750 },
  { key: "firstHeadline", what: "first headline fades in", frames: 8, ms: 333 },
  { key: "headline1to2", what: "headline 1 to 2", frames: 15, ms: 625 },
  { key: "headline2to3", what: "headline 2 to 3", frames: 15, ms: 625 },
  { key: "peekIn", what: "right peek fades in", frames: 14, ms: 583 },
  { key: "headline3to4", what: "headline 3 to 4, with the first zoom", frames: 38, ms: 1583 },
  { key: "layerCycle", what: "one change every 70 frames", frames: 70, ms: 2917 }
];

export interface Frame {
  colMax: Uint8Array;
  rowMax: Uint8Array;
  cardMax: Uint8Array;
  peekMax: Uint8Array;
  coreRow: Uint8Array;
  hue: Int16Array;
  peak: number;
  mean: number;
  cardMean: number;
  topMean: number;
}

export interface Phase {
  key: string;
  what: string;
  from: number; // frame
  to: number;
  frames: number;
  specFrames: number;
  ms: number;
  specMs: number;
  offFrames: number;
  offMs: number;
}

// The reduction, as it runs inside the page. Picture shares are turned into
// pixels there, so this stays the only place that knows the canvas size.
export var REDUCER = `window.__phase=(function(){
var W=landingPreview.size()[0],H=landingPreview.size()[1];
// The two loading rings spin, so the box each sits in is left out of every
// reading: a spinning ring must not read as the picture moving.
var RINGS=[[0.470,0.531,0.448,0.546],[0.470,0.531,0.760,0.840]];
var skip0=new Int32Array(H),skip1=new Int32Array(H);
for(var y=0;y<H;y++){skip0[y]=-1;skip1[y]=-2;
  for(var r=0;r<RINGS.length;r++){var b=RINGS[r];
    if(y>=b[2]*H&&y<=b[3]*H){skip0[y]=Math.round(b[0]*W);skip1[y]=Math.round(b[1]*W);}}}
var CARD=[Math.round(0.0287*W),Math.round(0.8315*W),Math.round(0.1565*H),Math.round(0.9567*H)];
var PEEK=Math.round(0.852*W);
var TOP=[Math.round(0.17*H),Math.round(0.42*H)];
var CORE=[Math.round(0.26*W),Math.round(0.60*W)];
var BANDS=192;
function b64(a){var s="";for(var i=0;i<a.length;i+=8192)s+=String.fromCharCode.apply(null,a.subarray(i,i+8192));return btoa(s);}
function one(ms){
  var p=landingPreview.pixels(ms);
  var colMax=new Uint8Array(W),colLit=new Uint8Array(W),rowMax=new Uint8Array(H);
  var cardMax=new Uint8Array(W),peekMax=new Uint8Array(W),coreRow=new Uint8Array(H);
  var hue=new Float64Array(BANDS),hueN=new Float64Array(BANDS);
  var peak=0,sum=0,n=0,top=0,topN=0,card=0,cardN=0;
  for(var y=0;y<H;y++){
    var src=(H-1-y)*W*4;             // readPixels gives the bottom row first
    var inCard=y>=CARD[2]&&y<=CARD[3];
    var s0=skip0[y],s1=skip1[y];
    var inTop=inCard&&y>=TOP[0]&&y<=TOP[1];
    var core=0,coreN=0,rm=0;
    for(var x=0;x<W;x++){
      var i=src+x*4,R=p[i],G=p[i+1],B=p[i+2];
      var L=(R*77+G*151+B*28)>>8;
      if(x<s0||x>s1){
        if(L>colMax[x])colMax[x]=L;
        if(L>colLit[x])colLit[x]=L;
        if(L>rm)rm=L;
        if(L>peak)peak=L;
        sum+=L;n++;
        var band=(x*BANDS/W)|0;hue[band]+=R-G;hueN[band]++;
      }
      if(inCard){
        if(x>=CARD[0]&&x<=CARD[1]){
          if(L>cardMax[x])cardMax[x]=L;
          card+=L;cardN++;
          if(inTop){top+=L;topN++;}
          if(x>=CORE[0]&&x<=CORE[1]){core+=L;coreN++;}
        }
        if(x>=PEEK&&L>peekMax[x])peekMax[x]=L;
      }
    }
    rowMax[y]=rm;
    coreRow[y]=coreN?Math.round(core/coreN):0;
  }
  var hueOut=new Int16Array(BANDS);
  for(var k=0;k<BANDS;k++)hueOut[k]=hueN[k]?Math.round(hue[k]/hueN[k]):0;
  return {colMax:b64(colMax),rowMax:b64(rowMax),cardMax:b64(cardMax),peekMax:b64(peekMax),
    coreRow:b64(coreRow),hue:b64(new Uint8Array(hueOut.buffer)),peak:peak,mean:n?sum/n:0,
    cardMean:cardN?card/cardN:0,topMean:topN?top/topN:0};
}
return {one:one,many:function(from,count,step){var out=[];for(var i=0;i<count;i++)out.push(one((from+i)*step));return out;}};
})();1`;

interface Raw {
  colMax: string; rowMax: string; cardMax: string; peekMax: string; coreRow: string; hue: string;
  peak: number; mean: number; cardMean: number; topMean: number;
}

export function decode(raw: Raw): Frame {
  const bytes = (s: string): Uint8Array => new Uint8Array(Buffer.from(s, "base64"));
  const hueBytes = bytes(raw.hue);
  return {
    colMax: bytes(raw.colMax), rowMax: bytes(raw.rowMax), cardMax: bytes(raw.cardMax),
    peekMax: bytes(raw.peekMax), coreRow: bytes(raw.coreRow),
    hue: new Int16Array(hueBytes.buffer, hueBytes.byteOffset, hueBytes.length / 2),
    peak: raw.peak, mean: raw.mean, cardMean: raw.cardMean, topMean: raw.topMean
  };
}

// ---------------------------------------------------------------------------
// Signals, all read off the frames above.
// ---------------------------------------------------------------------------

function diff(a: Uint8Array, b: Uint8Array): number {
  let total = 0;
  for (let i = 0; i < a.length; i++) total += Math.abs(a[i] - b[i]);
  return total;
}

// Share of the width that is lit, from the column profile.
function litWidth(f: Frame, floor: number): number {
  let x0 = -1;
  let x1 = -1;
  for (let x = 0; x < f.colMax.length; x++) {
    if (f.colMax[x] > floor) {
      if (x0 < 0) x0 = x;
      x1 = x;
    }
  }
  return x0 < 0 ? 0 : (x1 - x0 + 1) / f.colMax.length;
}

// Top and bottom of the lit part, as shares of height.
function litRows(f: Frame, floor: number): number[] {
  let y0 = -1;
  let y1 = -1;
  for (let y = 0; y < f.rowMax.length; y++) {
    if (f.rowMax[y] > floor) {
      if (y0 < 0) y0 = y;
      y1 = y;
    }
  }
  return y0 < 0 ? [1, 0] : [y0 / f.rowMax.length, y1 / f.rowMax.length];
}

// Left half minus right half of mean red-minus-green: near zero while both
// families are mixed, strongly positive once violet holds the left.
function hueSplit(f: Frame): number {
  let left = 0;
  let right = 0;
  const half = f.hue.length / 2;
  for (let i = 0; i < half; i++) left += f.hue[i];
  for (let i = half; i < f.hue.length; i++) right += f.hue[i];
  return (left - right) / half;
}

// Edges per column inside the card: a wall or a tile layer is full of them,
// an empty card or a wordmark on flat fill is not.
function cardTexture(f: Frame): number {
  let total = 0;
  for (let x = 1; x < f.cardMax.length; x++) total += Math.abs(f.cardMax[x] - f.cardMax[x - 1]);
  return total;
}

// Ink in the rows a headline is set on: the measured slots, with the tiles
// under the live headline cleared away, so what is left there is the headline.
var SLOTS = [[0.427, 0.499], [0.546, 0.618], [0.2395, 0.3025], [0.3255, 0.3885], [0.5265, 0.5895], [0.6365, 0.6995]];

function slotRows(h: number, which: number[]): number[] {
  const out: number[] = [];
  for (const i of which) {
    const [a, b] = SLOTS[i];
    for (let y = Math.round(a * h); y <= Math.round(b * h); y++) out.push(y);
  }
  return out;
}

// How much light is in a headline's own rows, in the strip it is set across.
function ink(f: Frame, rows: number[]): number {
  let total = 0;
  for (const y of rows) total += f.coreRow[y];
  return total;
}

function first(from: number, to: number, ok: (f: number) => boolean): number {
  if (from < 0) throw new Error("a phase boundary was not found before this one");
  for (let f = from; f < to; f++) if (ok(f)) return f;
  throw new Error(`no frame between ${from} and ${to} met the test`);
}

// Every signal the boundaries are read from, one row per frame, for checking.
export function signalRows(frames: Frame[]): string {
  const h = frames[0].coreRow.length;
  const two = slotRows(h, [0, 1]);
  const rows = ["frame,peak,mean,cardMean,topMean,litWidth96,litWidth24,litTop,litBottom,hueSplit,cardTexture,slotInk,dColMax,dCardMax,dPeekMax,dSlotInk"];
  for (let f = 0; f < frames.length; f++) {
    const a = frames[f];
    const b = frames[Math.min(f + 1, frames.length - 1)];
    const [top, bottom] = litRows(a, 24);
    rows.push([
      f, a.peak, a.mean.toFixed(2), a.cardMean.toFixed(3), a.topMean.toFixed(2), litWidth(a, LIT).toFixed(4), litWidth(a, 24).toFixed(4),
      top.toFixed(3), bottom.toFixed(3), hueSplit(a).toFixed(2), cardTexture(a), ink(a, two),
      diff(a.colMax, b.colMax), diff(a.cardMax, b.cardMax), diff(a.peekMax, b.peekMax),
      Math.abs(ink(a, two) - ink(b, two))
    ].join(","));
  }
  return rows.join("\n") + "\n";
}

// ---------------------------------------------------------------------------
// Phase boundaries.
// ---------------------------------------------------------------------------

const STILL = 24; // a profile this close to the last one is the same picture
// A stripe reaches this, the soft glow beside the column does not.
const LIT = 96;

export function findPhases(frames: Frame[]): Phase[] {
  const n = frames.length;
  const height = frames[0].coreRow.length;
  const twoLine = slotRows(height, [0, 1]);
  const changed = (f: number): boolean => f + 1 < n && diff(frames[f].colMax, frames[f + 1].colMax) > STILL;
  const cardChanged = (f: number): boolean => f + 1 < n &&
    (diff(frames[f].cardMax, frames[f + 1].cardMax) > STILL || Math.abs(frames[f].cardMean - frames[f + 1].cardMean) > 0.01);
  const peekChanged = (f: number): boolean => f + 1 < n && diff(frames[f].peekMax, frames[f + 1].peekMax) > STILL;
  const inkChanged = (f: number): boolean => f + 1 < n && Math.abs(ink(frames[f], twoLine) - ink(frames[f + 1], twoLine)) > 2;
  const out: Phase[] = [];
  const add = (key: string, from: number, to: number): void => {
    const spec = SPEC.find((s) => s.key === key)!;
    const ms = (to - from) * FRAME_MS;
    out.push({
      key, what: spec.what, from, to, frames: to - from, specFrames: spec.frames,
      ms: Math.round(ms * 10) / 10, specMs: Math.round(spec.frames * FRAME_MS * 10) / 10,
      offFrames: to - from - spec.frames, offMs: Math.round((ms - spec.frames * FRAME_MS) * 10) / 10
    });
  };

  // Opening. The wordmark, the mark and the stretch all live in the picture, so
  // the first boundaries are simply where the picture starts and stops moving.
  const collapse = first(0, n, changed);
  const markRest = first(collapse + 1, n, (f) => !changed(f));
  const stretch = first(markRest, n, changed);
  add("wordmarkRest", 0, collapse);
  add("wordmarkCollapse", collapse, markRest);
  add("markRest", markRest, stretch);

  // The stretch ends the frame the picture first runs from top to bottom.
  const column = first(stretch, n, (f) => {
    const [top, bottom] = litRows(frames[f], 24);
    return top <= 0.02 && bottom >= 0.98;
  });
  add("markStretch", stretch, column);

  // Then the lit width tells the column apart from the burst and full width.
  const wide = first(column, n, (f) => litWidth(frames[f], LIT) >= 0.30);
  const full = first(wide, n, (f) => litWidth(frames[f], 24) >= 0.999);
  const nearFull = first(wide, n, (f) => litWidth(frames[f], 24) >= 0.99);
  add("columnWidens", column, wide);
  add("burst", wide, full);

  // The colour split is the one frame the two families part on.
  let split = full;
  let jump = 0;
  for (let f = full; f < full + 12; f++) {
    const step = hueSplit(frames[f + 1]) - hueSplit(frames[f]);
    if (step > jump) { jump = step; split = f + 1; }
  }
  add("fullWidth", nearFull, split);
  add("colourSplit", split, split + 1);

  // The run ends on the first black frame; the fade is the dimming before it.
  const plateau = frames[split].peak;
  const black = first(split, n, (f) => frames[f].peak < 20);
  const fade = first(split, black, (f) => frames[f].peak < plateau * 0.97);
  add("stripeRun", split, black);
  add("fadeToBlack", fade, black);

  // The hard cut: the card and its band fill most of the picture in one frame.
  const cut = first(black, n, (f) => frames[f].mean > frames[black].mean + 6);
  add("blackRing", black, cut);

  // Landing. Frames from the cut.
  const L = (f: number): number => cut + f;
  const slide = first(0, n - cut, (f) => !cardChanged(L(f)));
  const wallIn = first(slide, n - cut, (f) => cardChanged(L(f)));
  const wallRest = first(wallIn + 1, n - cut, (f) => !cardChanged(L(f)));
  const wallOut = first(wallRest, n - cut, (f) => cardChanged(L(f)));
  add("cardSlide", 0, slide);
  add("emptyCardRest", slide, wallIn);
  add("wallIn", wallIn, wallRest);
  add("wallRest", wallRest, wallOut);

  // Above the wordmark only the wall and the tile layers reach, so that band
  // dims all the way through the wall going and holds once it has gone.
  const topAt = (f: number): number => frames[L(f)].topMean;
  const gone = first(wallOut + 1, n - cut, (f) => topAt(f) - topAt(f + 1) < 0.08);
  add("wallOut", wallOut, gone);

  // The first tile layer: the picture starts getting brighter again the frame
  // it arrives, and the card stops moving for good the frame it has settled.
  const tiles = first(gone, n - cut, (f) => frames[L(f) + 1].mean > frames[L(f)].mean + 1);
  const settled = first(tiles, n - cut, (f) => !cardChanged(L(f)) && !cardChanged(L(f) + 1) && !cardChanged(L(f) + 2));
  add("firstLayer", tiles, settled);

  // The first headline comes up in its own rows. Its rows are quiet for a few
  // frames first, so the arrival is the last quiet frame up to the plateau.
  // Its rows hold still for a few frames first, and the climb is far faster
  // than the drift that the layer's last bit of scaling leaves behind.
  const climb = (f: number): number => Math.abs(ink(frames[L(f)], twoLine) - ink(frames[L(f) + 1], twoLine));
  let inkTo = settled;
  while (inkTo > tiles && climb(inkTo - 1) <= 400) inkTo--;
  let inkFrom = inkTo;
  while (inkFrom > tiles && climb(inkFrom - 1) > 400) inkFrom--;
  add("firstHeadline", inkFrom, inkTo);

  // Two headline swaps on still tiles: the slots change and settle again.
  const swap1 = first(settled, n - cut, (f) => inkChanged(L(f)));
  const swap1End = first(swap1 + 1, n - cut, (f) => !inkChanged(L(f)));
  const swap2 = first(swap1End, n - cut, (f) => inkChanged(L(f)));
  const swap2End = first(swap2 + 1, n - cut, (f) => !inkChanged(L(f)));
  add("headline1to2", swap1, swap1End);
  add("headline2to3", swap2, swap2End);

  // The peek lives to the right of the card and only ever arrives once.
  const peek = first(swap2End, n - cut, (f) => peekChanged(L(f)));
  const peekEnd = first(peek + 1, n - cut, (f) => !peekChanged(L(f)));
  add("peekIn", peek, peekEnd);

  // The third swap takes the whole card with it: text out, then a layer zoom.
  const zoom = first(swap2End, n - cut, (f) => cardChanged(L(f)));
  const zoomEnd = first(zoom + 1, n - cut, (f) => !cardChanged(L(f)) && !cardChanged(L(f) + 1));
  add("headline3to4", zoom, zoomEnd);

  // From there one change every cycle. An exit starts so gently that its first
  // frames cannot be told apart, so the cycle is read off the gap instead: the
  // short dark hold between one layer leaving and the next arriving.
  const holds: { at: number; mean: number }[] = [];
  for (let f = zoomEnd; f < n - cut - 2; f++) {
    if (!cardChanged(L(f)) && cardChanged(L(f) - 1)) {
      let end = f;
      while (end < n - cut - 2 && !cardChanged(L(end))) end++;
      holds.push({ at: f, mean: frames[L(f)].cardMean });
      f = end;
    }
  }
  const mid = holds.reduce((a, h) => a + h.mean, 0) / Math.max(1, holds.length);
  const gaps = holds.filter((h) => h.mean < mid);
  if (gaps.length >= 2) add("layerCycle", gaps[0].at, gaps[1].at);
  return out;
}

export function sourceHash(root: string): string {
  const dir = join(root, "src/landing");
  const files = ["assets.ts", "audio.ts", "curves.ts", "layout.ts", "page.ts", "renderer.ts", "shaders.ts", "stripes.ts", "timeline.ts"];
  const h = createHash("sha256");
  for (const f of files) h.update(readFileSync(join(dir, f)));
  h.update(readFileSync(join(root, "landing/config.json")));
  return h.digest("hex").slice(0, 16);
}
