// Turns the folder of title images and the headlines into the textures the
// renderer needs. Runs in the page, so it works on any set of images.

import { CAP_HEIGHT, CARD, HEADLINE_U, ONE_LINE_V, PEEK, PIVOT, TWO_LINE_V } from "./timeline.ts";
import type { Atlas, Rect, SetOptions } from "./layout.ts";
import type { TextLine } from "./renderer.ts";

// One of the two files in brand/, read at build time: the drawing without its
// own <svg> wrapper, and the box it is drawn in.
export interface Drawing { viewBox: number[]; body: string; }

export interface Config {
  seed: number;
  size: { width: number; height: number };
  brand: { wordmark: string; mark: string; wordmarkSvg: Drawing; markSvg: Drawing };
  colours: {
    nearBlack: string; violet: string[]; blue: string[]; white: string; pink: string; cyan: string;
    ring: string; glassEdge: string; bandLeft: string; bandRight: string;
    fillTop: string; fillMid: string; fillLow: string; darkTop: string; darkLow: string;
    primaryButton: string; secondaryButton: string;
  };
  font: {
    family: string; headlineWeight: number; buttonWeight: number; tracking: number; maxWidth: number;
    uppercase: boolean; files: { weight: number; file: string }[];
  };
  buttons: { id: string; label: string }[];
  audio: { src: string | null; startAtMs: number; volume: number };
}

export interface LandingData {
  config: Config;
  headlines: string[][];
  images: string[];
}

export function hexToRgb(hex: string): number[] {
  var v = parseInt(hex.slice(1), 16);
  return [(v >> 16) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255];
}

function canvas(w: number, h: number): HTMLCanvasElement {
  var c = document.createElement("canvas");
  c.width = w;
  c.height = h;
  return c;
}

function context(c: HTMLCanvasElement): CanvasRenderingContext2D {
  return c.getContext("2d") as CanvasRenderingContext2D;
}

export function loadImage(src: string, done: (image: HTMLImageElement | null) => void): void {
  var image = new Image();
  image.onload = function () { done(image); };
  image.onerror = function () { done(null); };
  image.src = src;
}

function cover(ctx: CanvasRenderingContext2D, image: HTMLImageElement, x: number, y: number, w: number, h: number): void {
  var iw = image.naturalWidth || image.width;
  var ih = image.naturalHeight || image.height;
  var k = Math.max(w / iw, h / ih);
  var sw = w / k;
  var sh = h / k;
  ctx.drawImage(image, (iw - sw) / 2, (ih - sh) / 2, sw, sh, x, y, w, h);
}

// Each mark is drawn into one square texture with preserveAspectRatio none, so
// the renderer sets its shape from the measured box and not from the file. The
// aspect comes back with it, for the places that keep the mark's own shape.
export function drawingImage(drawing: Drawing, size: number, done: (image: HTMLCanvasElement, aspect: number) => void): void {
  var box = drawing.viewBox;
  var svg = '<svg xmlns="http://www.w3.org/2000/svg" width="' + size + '" height="' + size +
    '" preserveAspectRatio="none" viewBox="' + box.join(" ") + '">' + drawing.body + "</svg>";
  loadImage("data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg), function (image) {
    if (!image) throw new Error("a mark in brand/ did not load");
    var c = canvas(size, size);
    context(c).drawImage(image, 0, 0, size, size);
    done(c, box[2] / box[3]);
  });
}

export function brandImages(config: Config, done: (word: HTMLCanvasElement, wordAspect: number, mark: HTMLCanvasElement, markAspect: number) => void): void {
  drawingImage(config.brand.wordmarkSvg, 1024, function (word, wordAspect) {
    drawingImage(config.brand.markSvg, 512, function (mark, markAspect) {
      done(word, wordAspect, mark, markAspect);
    });
  });
}

export function buildAtlas(images: HTMLImageElement[]): { canvas: HTMLCanvasElement; atlas: Atlas } {
  var size = 2048;
  var atlas: Atlas = { columns: 6, rows: 10, cellW: 341, cellH: 192, size: size, count: Math.min(images.length, 60) };
  var c = canvas(size, size);
  var ctx = context(c);
  for (var i = 0; i < atlas.count; i++) {
    var x = (i % atlas.columns) * atlas.cellW;
    var y = Math.floor(i / atlas.columns) * atlas.cellH;
    cover(ctx, images[i], x, y, atlas.cellW, atlas.cellH);
  }
  return { canvas: c, atlas: atlas };
}

// Inter's capital height is 0.727 of its size.
var CAP_RATIO = 0.727;

// Set a line one letter at a time, so the tracking is the same on every
// browser. Returns the width it took. Pass draw false to measure only.
function setLine(ctx: CanvasRenderingContext2D, text: string, x: number, base: number, track: number, draw: boolean): number {
  var at = x;
  for (var i = 0; i < text.length; i++) {
    var ch = text.charAt(i);
    if (draw && ch !== " ") ctx.fillText(ch, at, base);
    at += ctx.measureText(ch).width + track;
  }
  return at - x - (text.length > 0 ? track : 0);
}

// Headline texture plus one quad per line, in px at 1920. The reference sets
// its headlines in heavy capitals, tight, centred on the card, over a soft
// shadow; cap height and the slots are measured, the face and tracking are ours.
export function buildText(config: Config, headlines: string[][]): { canvas: HTMLCanvasElement; lines: TextLine[][] } {
  var size = 2048;
  var total = 0;
  for (var h = 0; h < headlines.length; h++) total += headlines[h].length;
  var rowH = Math.min(150, Math.floor(size / total));
  var ink = rowH / 150;
  var c = canvas(size, size);
  var ctx = context(c);
  var lines: TextLine[][] = [];
  var row = 0;
  var maxWidth = config.font.maxWidth * 1920;
  ctx.textBaseline = "alphabetic";
  ctx.textAlign = "left";
  ctx.fillStyle = "#fff";
  ctx.shadowColor = "rgba(0,0,0,0.55)";
  ctx.shadowBlur = 22 * ink;
  ctx.shadowOffsetY = 3 * ink;
  var slot = 0;
  for (var i = 0; i < headlines.length; i++) {
    var two = headlines[i].length > 1;
    var cap = (two ? CAP_HEIGHT.twoLine : CAP_HEIGHT.oneLine) * 1080;
    var fontPx = cap / CAP_RATIO;
    var face = config.font.headlineWeight + " ";
    var tail = 'px "' + config.font.family + '", sans-serif';
    ctx.font = face + fontPx + tail;
    var widest = 0;
    for (var m = 0; m < headlines[i].length; m++) {
      var probe = config.font.uppercase ? headlines[i][m].toUpperCase() : headlines[i][m];
      widest = Math.max(widest, setLine(ctx, probe, 0, 0, fontPx * config.font.tracking, false));
    }
    if (widest > maxWidth) fontPx *= maxWidth / widest;
    var out: TextLine[] = [];
    for (var l = 0; l < headlines[i].length; l++) {
      var text = config.font.uppercase ? headlines[i][l].toUpperCase() : headlines[i][l];
      var drawPx = fontPx * ink;
      ctx.font = face + drawPx + tail;
      var pad = 24 * ink;
      var y = row * rowH;
      var base = y + rowH * 0.78;
      var w = setLine(ctx, text, pad, base, drawPx * config.font.tracking, true);
      var centreV = two ? TWO_LINE_V[l] : ONE_LINE_V[slot % ONE_LINE_V.length];
      var px0 = HEADLINE_U * 1920 - w / ink / 2 - 24;
      var px1 = HEADLINE_U * 1920 + w / ink / 2 + 24;
      // The cap sits with its middle on the slot, so the slot holds the eye.
      var top = centreV * 1080 - (rowH * 0.78 - (drawPx * CAP_RATIO) / 2) / ink;
      out.push({
        x0: px0, y0: top, x1: px1, y1: top + rowH / ink,
        u0: 0, v0: y / size, u1: (w + pad * 2) / size, v1: (y + rowH) / size
      });
      row++;
    }
    if (!two) slot++;
    lines.push(out);
  }
  return { canvas: c, lines: lines };
}

function rounded(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number): void {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

// Two pill buttons at the foot of the card, right aligned at u 0.80 (read by eye).
export function buildButtons(config: Config): { canvas: HTMLCanvasElement; rect: number[] } {
  var bw = 270;
  var bh = 60;
  var gap = 24;
  var count = config.buttons.length;
  var w = count * bw + (count - 1) * gap;
  var c = canvas(w, bh);
  var ctx = context(c);
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.font = config.font.buttonWeight + ' 26px "' + config.font.family + '", sans-serif';
  for (var i = 0; i < count; i++) {
    var x = i * (bw + gap);
    ctx.fillStyle = i === count - 1 ? config.colours.primaryButton : config.colours.secondaryButton;
    rounded(ctx, x, 0, bw, bh, bh / 2);
    ctx.fill();
    ctx.fillStyle = "#fff";
    ctx.fillText(config.buttons[i].label, x + bw / 2, bh / 2 + 1);
  }
  return { canvas: c, rect: [0.8 * 1920 - w, 0.885 * 1080 - bh / 2, w, bh] };
}

// The next card shows the left of its art, at the size the full card would use.
export function buildPeek(image: HTMLImageElement): HTMLCanvasElement {
  var ch = Math.round((PEEK.v1 - PEEK.v0) * 1080);
  var cw = Math.round((1 - PEEK.u0) * 1920);
  var full = Math.round((CARD.u1 - CARD.u0) * 1920);
  var c = canvas(cw, ch);
  cover(context(c), image, 0, 0, full, ch);
  return c;
}

function around(lines: TextLine[], mx: number, my: number): Rect {
  var r: Rect = { x0: 1e9, y0: 1e9, x1: -1e9, y1: -1e9 };
  for (var i = 0; i < lines.length; i++) {
    r.x0 = Math.min(r.x0, lines[i].x0 - mx);
    r.x1 = Math.max(r.x1, lines[i].x1 + mx);
    var mid = (lines[i].y0 + lines[i].y1) / 2;
    r.y0 = Math.min(r.y0, mid - 56 - my);
    r.y1 = Math.max(r.y1, mid + 56 + my);
  }
  return r;
}

// Tile sets in timeline order, then the wall.
//
// Measured grid, read off the rectified photo of one layer: tile 0.180 W by
// 0.178 H, column pitch 0.199 W, row pitch 0.212 H, rows sloping down to the
// right by 6 degrees, columns leaning 23 degrees with the top to the right.
// Row lines meet at about u -0.61, which is the shrink towards the left.
// "Other layers differ", so the first two sets are drawn at 0.68 of this; the
// tilt, the pitch to tile ratio and the wall are the measured ones.
var TILE_W = 0.180 * 1920;
var TILE_H = 0.178 * 1080;
var GAP_X = (0.199 - 0.180) * 1920;
var GAP_Y = (0.212 - 0.178) * 1080;
var DENSE = 0.68;

export function setOptions(lines: TextLine[][], setCount: number, wall: number): SetOptions[] {
  var out: SetOptions[] = [];
  for (var s = 0; s < setCount; s++) {
    if (s === wall) {
      out.push({
        tileW: TILE_W, tileH: TILE_H, gapX: GAP_X, gapY: GAP_Y,
        rowDeg: 6, columnDeg: 23, shrink: 1 / 2000, leftFade: 1, clear: []
      });
      continue;
    }
    var k = s < 2 ? DENSE : 1;
    var clear: Rect[] = [];
    for (var h = 0; h < lines.length; h++) {
      var set = h === 0 ? 0 : h <= 2 ? 1 : h - 1;
      if (set === s) clear.push(around(lines[h], 18, 4));
    }
    out.push({
      tileW: TILE_W * k, tileH: TILE_H * k, gapX: GAP_X * k, gapY: GAP_Y * k,
      rowDeg: 6, columnDeg: 23, shrink: 1 / 3200, leftFade: 0.45, clear: clear
    });
  }
  return out;
}

// The wordmark on the card at rest: u 0.218 to 0.642, v 0.441 to 0.646,
// centred on the card at (0.430, 0.544), 1.08 times the splash wordmark.
export var MARK_CENTRE = [PIVOT.u * 1920, 0.544 * 1080];
export var MARK_HEIGHT = 0.205 * 1080;
