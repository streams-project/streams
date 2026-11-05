// Lays tiles out for each tile set and writes them into one vertex buffer.
//
// A set is a flat grid turned away from the viewer: rows slope down to the
// right, columns lean with their tops to the right, and tiles shrink towards
// the left. All positions are px at 1920 by 1080.

import { rng } from "./curves.ts";
import { CARD, PIVOT } from "./timeline.ts";

export var VERTEX_FLOATS = 12;

export interface Rect {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

export interface SetOptions {
  tileW: number;
  tileH: number;
  gap: number;
  rowDeg: number;
  columnDeg: number;
  shrink: number; // one over the distance to the vanishing point, in px
  leftFade: number; // 0 flat, 1 the full measured fall towards the left
  clear: Rect[]; // headline areas, tiles touching these are left out
}

export interface Atlas {
  columns: number;
  rows: number;
  cellW: number;
  cellH: number;
  size: number;
  count: number;
}

export interface Geometry {
  vertices: Float32Array;
  indices: Uint16Array;
  sets: { first: number; count: number }[];
}

// Brightness against share of width, from wall readings normalised to the brightest.
function leftFall(u: number): number {
  if (u < 0.05) return 0.46;
  if (u < 0.2) return 0.46 + ((u - 0.05) / 0.15) * 0.1;
  if (u < 0.27) return 0.56 + ((u - 0.2) / 0.07) * 0.16;
  if (u < 0.5) return 0.72 + ((u - 0.27) / 0.23) * 0.02;
  if (u < 0.72) return 0.74 + ((u - 0.5) / 0.22) * 0.26;
  return 1;
}

function shuffled(count: number, random: () => number): number[] {
  var list: number[] = [];
  for (var i = 0; i < count; i++) list.push(i);
  for (var j = count - 1; j > 0; j--) {
    var k = Math.floor(random() * (j + 1));
    var t = list[j];
    list[j] = list[k];
    list[k] = t;
  }
  return list;
}

interface View {
  cx: number; cy: number; rx: number; ry: number; kx: number; ky: number; shrink: number;
}

// Grid point to homogeneous picture position: x and y over w give px at 1920.
function project(v: View, a: number, b: number): number[] {
  var w = 1 - a * v.shrink;
  return [v.cx * w + a * v.rx + b * v.kx, v.cy * w + a * v.ry + b * v.ky, w];
}

// Separating axis test: does a convex tile touch a headline's rectangle?
function touches(corners: number[][], r: Rect): boolean {
  var cx = (r.x0 + r.x1) / 2;
  var cy = (r.y0 + r.y1) / 2;
  var hw = (r.x1 - r.x0) / 2;
  var hh = (r.y1 - r.y0) / 2;
  var minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
  var x: number[] = [];
  var y: number[] = [];
  for (var i = 0; i < 4; i++) {
    x.push(corners[i][0] / corners[i][2] - cx);
    y.push(corners[i][1] / corners[i][2] - cy);
    minX = Math.min(minX, x[i]);
    maxX = Math.max(maxX, x[i]);
    minY = Math.min(minY, y[i]);
    maxY = Math.max(maxY, y[i]);
  }
  if (minX > hw || maxX < -hw || minY > hh || maxY < -hh) return false;
  for (var e = 0; e < 4; e++) {
    var f = (e + 1) % 4;
    var nx = y[f] - y[e];
    var ny = x[e] - x[f];
    var reach = Math.abs(nx) * hw + Math.abs(ny) * hh;
    var side = 0;
    var lo = Infinity;
    var hi = -Infinity;
    for (var k = 0; k < 4; k++) {
      side = x[k] * nx + y[k] * ny;
      lo = Math.min(lo, side);
      hi = Math.max(hi, side);
    }
    if (lo > reach || hi < -reach) return false;
  }
  return true;
}

export function buildGeometry(options: SetOptions[], atlas: Atlas, seed: number): Geometry {
  var verts: number[] = [];
  var idx: number[] = [];
  var sets: { first: number; count: number }[] = [];
  var px = 1920;
  var py = 1080;
  var cx = PIVOT.u * px;
  var cy = PIVOT.v * py;
  var margin = 70;
  var left = CARD.u0 * px - margin;
  var right = CARD.u1 * px + margin;
  var top = CARD.v0 * py - margin;
  var bottom = CARD.v1 * py + margin;

  for (var s = 0; s < options.length; s++) {
    var o = options[s];
    var random = rng(seed + s * 7919);
    var order = shuffled(atlas.count, random);
    var next = 0;
    var rowRad = (o.rowDeg * Math.PI) / 180;
    var colRad = (o.columnDeg * Math.PI) / 180;
    var rx = Math.cos(rowRad);
    var ry = Math.sin(rowRad);
    var kx = Math.sin(colRad);
    var ky = -Math.cos(colRad);
    var first = idx.length;
    var pitchY = o.tileH + o.gap;
    var rows = Math.ceil(1400 / pitchY);

    for (var row = -rows; row <= rows; row++) {
      var b0 = row * pitchY;
      var b1 = b0 + o.tileH;
      var a = -2200 + random() * o.tileW;
      while (a < 2200) {
        var pick = random();
        var w = o.tileW * (pick < 0.18 ? 1.45 : pick < 0.36 ? 0.72 : 1);
        var a0 = a;
        var a1 = a + w;
        a = a1 + o.gap;

        var view = { cx: cx, cy: cy, rx: rx, ry: ry, kx: kx, ky: ky, shrink: o.shrink };
        var corners = [project(view, a0, b0), project(view, a1, b0), project(view, a1, b1), project(view, a0, b1)];
        var minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
        var skip = false;
        for (var c = 0; c < 4; c++) {
          var p = corners[c];
          if (p[2] < 0.25) skip = true;
          var sx = p[0] / p[2];
          var sy = p[1] / p[2];
          minX = Math.min(minX, sx);
          maxX = Math.max(maxX, sx);
          minY = Math.min(minY, sy);
          maxY = Math.max(maxY, sy);
        }
        if (skip || maxX < left || minX > right || maxY < top || minY > bottom) continue;
        // A tile under a headline stays in the buffer with a negative
        // brightness, so the shader can fade it out as the text arrives.
        var underText = false;
        for (var r = 0; r < o.clear.length; r++) {
          if (touches(corners, o.clear[r])) underText = true;
        }

        var art = order[next++ % order.length];
        var col = art % atlas.columns;
        var line = Math.floor(art / atlas.columns) % atlas.rows;
        var inset = 3;
        var uw = (atlas.cellW - inset * 2) / atlas.size;
        var vh = (atlas.cellH - inset * 2) / atlas.size;
        var u0 = (col * atlas.cellW + inset) / atlas.size;
        var v0 = (line * atlas.cellH + inset) / atlas.size;
        // Cover crop the cell to the tile's own shape.
        var aspect = (a1 - a0) / o.tileH;
        var cellAspect = atlas.cellW / atlas.cellH;
        if (aspect > cellAspect) {
          var keep = cellAspect / aspect;
          v0 += (vh * (1 - keep)) / 2;
          vh *= keep;
        } else {
          var keepW = aspect / cellAspect;
          u0 += (uw * (1 - keepW)) / 2;
          uw *= keepW;
        }
        var midU = (minX + maxX) / 2 / px;
        var bright = (1 + (leftFall(midU) - 1) * o.leftFade) * (underText ? -1 : 1);
        // Late in an exit tiles drift apart slightly: mostly small, a few far.
        var drift = 0.45 + -Math.log(1 - random() * 0.985) * 0.6;
        var dir = random() * Math.PI * 2;
        var jx = Math.cos(dir) * drift * px;
        var jy = Math.sin(dir) * drift * px;
        var hx = (a1 - a0) / 2;
        var hy = o.tileH / 2;
        var base = verts.length / VERTEX_FLOATS;
        var us = [u0, u0 + uw, u0 + uw, u0];
        // The grid's b axis points up the picture, texture v points down.
        var vs = [v0 + vh, v0 + vh, v0, v0];
        var ls = [-hx, hx, hx, -hx];
        var ms = [hy, hy, -hy, -hy];
        for (var k = 0; k < 4; k++) {
          var pc = corners[k];
          verts.push(pc[0] / pc[2], pc[1] / pc[2], pc[2], us[k], vs[k], ls[k], ms[k], hx, hy, bright, jx, jy);
        }
        idx.push(base, base + 1, base + 2, base, base + 2, base + 3);
      }
    }
    sets.push({ first: first, count: idx.length - first });
  }
  return { vertices: new Float32Array(verts), indices: new Uint16Array(idx), sets: sets };
}
