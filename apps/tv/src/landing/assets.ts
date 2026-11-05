// Turns the folder of title images and the headlines into the textures the
// renderer needs. Runs in the page, so it works on any set of images.

import { CAP_HEIGHT, CARD, HEADLINE_U, ONE_LINE_V, PEEK, PIVOT, TWO_LINE_V } from "./timeline.ts";
import type { Atlas, Rect, SetOptions } from "./layout.ts";
import type { TextLine } from "./renderer.ts";

export interface LogoShape { d: string; fill: string; }

export interface Config {
  seed: number;
  size: { width: number; height: number };
  logoRestMs: number;
  logo: { viewBox: number[]; shapes: LogoShape[] };
  colours: {
    nearBlack: string; violet: string[]; blue: string[]; white: string; pink: string; cyan: string;
    ring: string; glassEdge: string; bandLeft: string; bandRight: string;
    fillTop: string; fillMid: string; fillLow: string; darkTop: string; darkLow: string;
    primaryButton: string; secondaryButton: string;
  };
  font: { family: string; headlineWeight: number; buttonWeight: number; files: { weight: number; file: string }[] };
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

export function logoImage(config: Config, done: (logo: HTMLCanvasElement, aspect: number) => void): void {
  var box = config.logo.viewBox;
  var body = "";
  for (var i = 0; i < config.logo.shapes.length; i++) {
    var s = config.logo.shapes[i];
    body += '<path d="' + s.d + '" fill="' + s.fill + '"/>';
  }
  var svg = '<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" preserveAspectRatio="none" viewBox="' + box.join(" ") + '">' + body + "</svg>";
  loadImage("data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg), function (image) {
    if (!image) throw new Error("the logo in the config did not load");
    var c = canvas(512, 512);
    context(c).drawImage(image, 0, 0, 512, 512);
    done(c, box[2] / box[3]);
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

// Headline texture plus one quad per line, in px at 1920.
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
  var maxWidth = 0.7 * 1920;
  ctx.textBaseline = "alphabetic";
  ctx.textAlign = "left";
  ctx.fillStyle = "#fff";
  ctx.shadowColor = "rgba(0,0,0,0.6)";
  ctx.shadowBlur = 16 * ink;
  ctx.shadowOffsetY = 2 * ink;
  var slot = 0;
  for (var i = 0; i < headlines.length; i++) {
    var two = headlines[i].length > 1;
    var cap = (two ? CAP_HEIGHT.twoLine : CAP_HEIGHT.oneLine) * 1080;
    // Inter's capital height is 0.727 of its size.
    var fontPx = cap / 0.727;
    ctx.font = config.font.headlineWeight + " " + fontPx + 'px "' + config.font.family + '", sans-serif';
    var widest = 0;
    for (var m = 0; m < headlines[i].length; m++) widest = Math.max(widest, ctx.measureText(headlines[i][m]).width);
    if (widest > maxWidth) fontPx *= maxWidth / widest;
    var out: TextLine[] = [];
    for (var l = 0; l < headlines[i].length; l++) {
      var text = headlines[i][l];
      ctx.font = config.font.headlineWeight + " " + fontPx * ink + 'px "' + config.font.family + '", sans-serif';
      var w = ctx.measureText(text).width;
      var pad = 24 * ink;
      var y = row * rowH;
      var base = y + rowH * 0.7;
      ctx.fillText(text, pad, base);
      var centreV = two ? TWO_LINE_V[l] : ONE_LINE_V[slot % ONE_LINE_V.length];
      var px0 = HEADLINE_U * 1920 - w / ink / 2 - 24;
      var px1 = HEADLINE_U * 1920 + w / ink / 2 + 24;
      var top = centreV * 1080 - (rowH * 0.7 - (fontPx * ink * 0.727) / 2) / ink;
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

// Tile sets in timeline order, then the wall. Wall angles are the measured
// medians. Sizes are ours: the first two sets are dense, later sets are larger,
// and every set keeps the tilt.
export function setOptions(lines: TextLine[][], setCount: number, wall: number): SetOptions[] {
  var out: SetOptions[] = [];
  for (var s = 0; s < setCount; s++) {
    if (s === wall) {
      out.push({ tileW: 250, tileH: 141, gap: 9, rowDeg: 6, columnDeg: 22, shrink: 1 / 2300, leftFade: 1, clear: [] });
      continue;
    }
    var dense = s < 2;
    var clear: Rect[] = [];
    for (var h = 0; h < lines.length; h++) {
      var set = h === 0 ? 0 : h <= 2 ? 1 : h - 1;
      if (set === s) clear.push(around(lines[h], 18, 4));
    }
    out.push({
      tileW: dense ? 196 : 290, tileH: dense ? 110 : 163, gap: dense ? 10 : 14,
      rowDeg: 6, columnDeg: 22, shrink: 1 / 3200, leftFade: 0.45, clear: clear
    });
  }
  return out;
}

export var MARK_CENTRE = [PIVOT.u * 1920, 0.544 * 1080];
export var MARK_HEIGHT = 0.205 * 1080;
