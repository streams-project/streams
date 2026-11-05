// Draws a Scene with WebGL 1. All state is set here, nothing else touches GL.

import { ROW_COOL, ROW_MIXED, ROW_SHEEN, ROW_THREAD, ROW_WARM, STRIPE_ROWS, STRIPE_WIDTH } from "./stripes.ts";
import { CARD, PEEK, PIVOT } from "./timeline.ts";
import type { LayerDraw, Scene } from "./timeline.ts";
import { VERTEX_FLOATS } from "./layout.ts";
import type { Geometry } from "./layout.ts";
import * as sh from "./shaders.ts";

export interface TextLine {
  // px at 1920
  x0: number; y0: number; x1: number; y1: number;
  // texture shares
  u0: number; v0: number; u1: number; v1: number;
}

export interface Palette {
  black: number[]; ring: number[]; bandA: number[]; bandB: number[]; edge: number[];
  fillTop: number[]; fillMid: number[]; fillLow: number[]; darkTop: number[]; darkLow: number[];
}

export interface RenderInput {
  width: number;
  height: number;
  palette: Palette;
  stripes: Uint8Array;
  word: TexImageSource;
  mark: TexImageSource;
  wordAspect: number;
  atlas: TexImageSource | null;
  text: TexImageSource | null;
  ui: TexImageSource | null;
  uiRect: number[]; // x, y, w, h px at 1920
  peek: TexImageSource | null;
  geometry: Geometry | null;
  lines: TextLine[][];
  markCentre: number[]; // px at 1920
  markHeight: number;
}

type Uniforms = { [name: string]: WebGLUniformLocation | null };
interface Program { id: WebGLProgram; u: Uniforms; a: { [name: string]: number }; }
interface Target { w: number; h: number; fb: WebGLFramebuffer[]; tex: WebGLTexture[]; }

var CARD_X = CARD.u0 * 1920;
var CARD_Y = CARD.v0 * 1080;
var CARD_W = (CARD.u1 - CARD.u0) * 1920;
var CARD_H = (CARD.v1 - CARD.v0) * 1080;

export class Renderer {
  gl: WebGLRenderingContext;
  input: RenderInput;
  full: WebGLBuffer;
  tileVbo: WebGLBuffer | null = null;
  tileIbo: WebGLBuffer | null = null;
  textVbo: WebGLBuffer | null = null;
  opening: Program;
  tile: Program;
  text: Program;
  blur: Program;
  card: Program;
  rect: Program;
  peek: Program;
  stripeTex: WebGLTexture;
  wordTex: WebGLTexture;
  markTex: WebGLTexture;
  atlasTex: WebGLTexture | null = null;
  textTex: WebGLTexture | null = null;
  uiTex: WebGLTexture | null = null;
  peekTex: WebGLTexture | null = null;
  targets: { [d: number]: Target } = {};
  last: Float32Array;
  key: Float32Array;
  drawn = 0;
  skipped = 0;
  weights = new Float32Array(7);

  constructor(canvas: HTMLCanvasElement, input: RenderInput, preserve: boolean) {
    var attrs = { alpha: false, antialias: false, depth: false, stencil: false, preserveDrawingBuffer: preserve };
    var gl = (canvas.getContext("webgl", attrs) || canvas.getContext("experimental-webgl", attrs)) as WebGLRenderingContext | null;
    if (!gl) throw new Error("WebGL is not available");
    this.gl = gl;
    this.input = input;
    gl.disable(gl.DEPTH_TEST);
    gl.disable(gl.CULL_FACE);
    gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
    gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);

    this.full = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.full);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0, 0, 1, 0, 0, 1, 1, 1]), gl.STATIC_DRAW);

    this.opening = this.program(sh.FULL_VERT, sh.OPENING_FRAG);
    this.tile = this.program(sh.TILE_VERT, sh.TILE_FRAG);
    this.text = this.program(sh.TEXT_VERT, sh.TEXT_FRAG);
    this.blur = this.program(sh.FULL_VERT, sh.BLUR_FRAG);
    this.card = this.program(sh.FULL_VERT, sh.CARD_FRAG);
    this.rect = this.program(sh.RECT_VERT, sh.RECT_FRAG);
    this.peek = this.program(sh.RECT_VERT, sh.PEEK_FRAG);

    this.stripeTex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, this.stripeTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, STRIPE_WIDTH, STRIPE_ROWS, 0, gl.RGBA, gl.UNSIGNED_BYTE, input.stripes);
    this.params(gl.LINEAR, gl.LINEAR);
    this.wordTex = this.image(input.word, true);
    this.markTex = this.image(input.mark, true);
    this.last = new Float32Array(96);
    this.key = new Float32Array(96);
  }

  // Called once the title images and text are ready.
  loadLanding(): void {
    var gl = this.gl;
    var input = this.input;
    this.atlasTex = this.image(input.atlas!, true);
    var aniso = gl.getExtension("EXT_texture_filter_anisotropic");
    if (aniso) gl.texParameterf(gl.TEXTURE_2D, aniso.TEXTURE_MAX_ANISOTROPY_EXT, 4);
    this.textTex = this.image(input.text!, true);
    this.uiTex = this.image(input.ui!, false);
    this.peekTex = this.image(input.peek!, false);

    var geo = input.geometry!;
    this.tileVbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.tileVbo);
    gl.bufferData(gl.ARRAY_BUFFER, geo.vertices, gl.STATIC_DRAW);
    this.tileIbo = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.tileIbo);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, geo.indices, gl.STATIC_DRAW);

    var quads: number[] = [];
    for (var h = 0; h < input.lines.length; h++) {
      for (var l = 0; l < input.lines[h].length; l++) {
        var t = input.lines[h][l];
        quads.push(t.x0, t.y0, t.u0, t.v0, t.x1, t.y0, t.u1, t.v0, t.x0, t.y1, t.u0, t.v1, t.x1, t.y1, t.u1, t.v1);
      }
    }
    this.textVbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.textVbo);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(quads), gl.STATIC_DRAW);
  }

  params(min: number, mag: number): void {
    var gl = this.gl;
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, min);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, mag);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  image(source: TexImageSource, mip: boolean): WebGLTexture {
    var gl = this.gl;
    var tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, source);
    if (mip) gl.generateMipmap(gl.TEXTURE_2D);
    this.params(mip ? gl.LINEAR_MIPMAP_LINEAR : gl.LINEAR, gl.LINEAR);
    return tex;
  }

  program(vs: string, fs: string): Program {
    var gl = this.gl;
    var id = gl.createProgram()!;
    var parts = [[gl.VERTEX_SHADER, vs], [gl.FRAGMENT_SHADER, fs]] as [number, string][];
    for (var i = 0; i < 2; i++) {
      var s = gl.createShader(parts[i][0])!;
      gl.shaderSource(s, parts[i][1]);
      gl.compileShader(s);
      if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s) || "shader failed");
      gl.attachShader(id, s);
    }
    gl.bindAttribLocation(id, 0, "aPos");
    gl.linkProgram(id);
    if (!gl.getProgramParameter(id, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(id) || "link failed");
    var u: Uniforms = {};
    var nu = gl.getProgramParameter(id, gl.ACTIVE_UNIFORMS) as number;
    for (var j = 0; j < nu; j++) {
      var name = gl.getActiveUniform(id, j)!.name;
      u[name] = gl.getUniformLocation(id, name);
    }
    var a: { [name: string]: number } = {};
    var na = gl.getProgramParameter(id, gl.ACTIVE_ATTRIBUTES) as number;
    for (var k = 0; k < na; k++) {
      var an = gl.getActiveAttrib(id, k)!.name;
      a[an] = gl.getAttribLocation(id, an);
    }
    return { id: id, u: u, a: a };
  }

  target(d: number): Target {
    var t = this.targets[d];
    if (t) return t;
    var gl = this.gl;
    var w = Math.max(1, Math.round(((CARD_W / 1920) * this.input.width) / d));
    var h = Math.max(1, Math.round(((CARD_H / 1080) * this.input.height) / d));
    t = { w: w, h: h, fb: [], tex: [] };
    for (var i = 0; i < 2; i++) {
      var tex = gl.createTexture()!;
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
      this.params(gl.LINEAR, gl.LINEAR);
      var fb = gl.createFramebuffer()!;
      gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
      t.tex.push(tex);
      t.fb.push(fb);
    }
    this.targets[d] = t;
    return t;
  }

  fullQuad(): void {
    var gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.full);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    this.only(1);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  }

  // Keeps only the first n attribute arrays on, so a later pass cannot read a stale one.
  only(n: number): void {
    var gl = this.gl;
    for (var i = n; i < 5; i++) gl.disableVertexAttribArray(i);
  }

  bind(unit: number, tex: WebGLTexture | null, loc: WebGLUniformLocation | null): void {
    var gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.uniform1i(loc, unit);
  }

  // True if this frame differs from the last one drawn.
  changed(s: Scene): boolean {
    var k = this.key;
    var n = 0;
    k[n++] = s.stage;
    if (s.stage === 0) {
      k[n++] = s.wordAlpha; k[n++] = s.wordU0; k[n++] = s.wordU1; k[n++] = s.wordClip; k[n++] = s.wordV1;
      k[n++] = s.markAlpha; k[n++] = s.markU0; k[n++] = s.markU1; k[n++] = s.markV0; k[n++] = s.markV1;
      k[n++] = s.filament; k[n++] = s.lines;
      k[n++] = s.column; k[n++] = s.columnAlpha; k[n++] = s.zoomWarm; k[n++] = s.zoomCool; k[n++] = s.split;
      k[n++] = s.fade; k[n++] = s.ringAlpha; k[n++] = s.ringAngleDeg;
      k[n++] = s.ringArcDeg; k[n++] = s.ringU; k[n++] = s.ringV;
    } else {
      k[n++] = s.cardShift; k[n++] = s.emptyFill; k[n++] = s.band; k[n++] = s.peek; k[n++] = s.cardWordAlpha; k[n++] = s.cardWordScale;
      k[n++] = s.layerCount; k[n++] = s.textCount;
      for (var i = 0; i < s.layerCount; i++) {
        var l = s.layers[i];
        k[n++] = l.set; k[n++] = l.scale; k[n++] = l.sigma; k[n++] = l.alpha; k[n++] = l.gain; k[n++] = l.scatter; k[n++] = l.underText;
      }
      for (var j = 0; j < s.textCount; j++) {
        var t = s.texts[j];
        k[n++] = t.headline; k[n++] = t.line; k[n++] = t.alpha; k[n++] = t.r; k[n++] = t.g; k[n++] = t.b;
      }
    }
    var same = true;
    for (var m = 0; m < n; m++) {
      if (k[m] !== this.last[m]) { same = false; this.last[m] = k[m]; }
    }
    k[n] = -1;
    if (this.last[n] !== -1) { same = false; this.last[n] = -1; }
    return !same;
  }

  draw(s: Scene): boolean {
    if (!this.changed(s)) {
      this.skipped++;
      return false;
    }
    this.drawn++;
    if (s.stage === 0) this.drawOpening(s);
    else this.drawLanding(s);
    return true;
  }

  drawOpening(s: Scene): void {
    var gl = this.gl;
    var p = this.opening;
    var c = this.input.palette;
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, this.input.width, this.input.height);
    gl.disable(gl.BLEND);
    gl.useProgram(p.id);
    this.bind(0, this.wordTex, p.u.uWord);
    this.bind(1, this.markTex, p.u.uMark);
    this.bind(2, this.stripeTex, p.u.uStripes);
    gl.uniform3fv(p.u.uBlack, c.black);
    gl.uniform3fv(p.u.uRingColour, c.ring);
    gl.uniform4f(p.u.uRows, ROW_MIXED, ROW_WARM, ROW_COOL, ROW_SHEEN);
    gl.uniform1f(p.u.uThreadRow, ROW_THREAD);
    gl.uniform4f(p.u.uWordBox, s.wordU0, s.wordU1, s.wordV0, s.wordV1);
    gl.uniform2f(p.u.uWordState, s.wordAlpha, s.wordClip);
    gl.uniform4f(p.u.uMarkBox, s.markU0, s.markU1, s.markV0, s.markV1);
    gl.uniform3f(p.u.uMarkState, s.markAlpha, s.filament, s.lines);
    gl.uniform4f(p.u.uStripeState, s.column, s.split, s.columnAlpha, s.fade);
    gl.uniform2f(p.u.uZoom, s.zoomWarm, s.zoomCool);
    gl.uniform4f(p.u.uRing, s.ringU * 1920, s.ringV * 1080, s.ringRadiusPx, s.ringStrokePx);
    gl.uniform3f(p.u.uRingState, s.ringAngleDeg / 360, s.ringArcDeg / 360, s.ringAlpha);
    this.fullQuad();
  }

  blurStep(sigma: number): void {
    // Gaussian weights for texels 0 to 6, folded into linear sampled pairs.
    var w = this.weights;
    var total = 0;
    for (var i = 0; i < 7; i++) {
      w[i] = Math.exp((-i * i) / (2 * sigma * sigma));
      total += i === 0 ? w[i] : 2 * w[i];
    }
    for (var j = 0; j < 7; j++) w[j] /= total;
  }

  blurPass(src: WebGLTexture, dst: WebGLFramebuffer, t: Target, dx: number, dy: number): void {
    var gl = this.gl;
    var p = this.blur;
    var w = this.weights;
    gl.bindFramebuffer(gl.FRAMEBUFFER, dst);
    gl.useProgram(p.id);
    this.bind(0, src, p.u.uTex);
    gl.uniform2f(p.u.uDir, dx / t.w, dy / t.h);
    var a = w[1] + w[2];
    var b = w[3] + w[4];
    var c = w[5] + w[6];
    gl.uniform3f(p.u.uOff, 1 + w[2] / a, 3 + w[4] / b, 5 + w[6] / c);
    gl.uniform4f(p.u.uWeight, w[0], a, b, c);
    this.fullQuad();
  }

  drawLayerTiles(l: LayerDraw, t: Target): void {
    var gl = this.gl;
    var p = this.tile;
    var set = this.input.geometry!.sets[l.set];
    gl.uniform3f(p.u.uPivot, PIVOT.u * 1920, PIVOT.v * 1080, l.scale);
    gl.uniform1f(p.u.uScatter, l.scatter);
    gl.uniform1f(p.u.uAlpha, l.alpha);
    gl.uniform1f(p.u.uUnderText, l.underText);
    gl.uniform1f(p.u.uAa, Math.min(1, (l.scale * t.w) / CARD_W));
    gl.drawElements(gl.TRIANGLES, set.count, gl.UNSIGNED_SHORT, set.first * 2);
  }

  drawLayer(s: Scene): WebGLTexture | null {
    var gl = this.gl;
    if (s.layerCount === 0) return this.target(8).tex[0];
    var sigma = 0;
    for (var i = 0; i < s.layerCount; i++) sigma = Math.max(sigma, s.layers[i].sigma);
    var scale = this.input.height / 1080;
    var onScreen = sigma * scale;
    var d = onScreen < 1 ? 1 : onScreen < 5 ? 2 : onScreen < 10 ? 4 : 8;
    var t = this.target(d);
    gl.bindFramebuffer(gl.FRAMEBUFFER, t.fb[0]);
    gl.viewport(0, 0, t.w, t.h);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.enable(gl.BLEND);

    var p = this.tile;
    gl.useProgram(p.id);
    this.bind(0, this.atlasTex, p.u.uAtlas);
    gl.uniform4f(p.u.uCard, CARD_X, CARD_Y, CARD_W, CARD_H);
    gl.uniform1f(p.u.uRadius, 7);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.tileVbo);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.tileIbo);
    var stride = VERTEX_FLOATS * 4;
    this.attrib(p, "aPos", 3, stride, 0);
    this.attrib(p, "aUv", 2, stride, 12);
    this.attrib(p, "aLocal", 2, stride, 20);
    this.attrib(p, "aHalf", 2, stride, 28);
    this.attrib(p, "aMisc", 3, stride, 36);
    for (var j = 0; j < s.layerCount; j++) this.drawLayerTiles(s.layers[j], t);

    if (s.textCount > 0) {
      var q = this.text;
      gl.useProgram(q.id);
      this.bind(0, this.textTex, q.u.uTex);
      gl.uniform4f(q.u.uCard, CARD_X, CARD_Y, CARD_W, CARD_H);
      gl.uniform3f(q.u.uPivot, PIVOT.u * 1920, PIVOT.v * 1080, s.layers[0].scale);
      gl.bindBuffer(gl.ARRAY_BUFFER, this.textVbo);
      this.only(2);
      this.attrib(q, "aPos", 2, 16, 0);
      this.attrib(q, "aUv", 2, 16, 8);
      for (var k = 0; k < s.textCount; k++) {
        var line = s.texts[k];
        var offset = this.lineOffset(line.headline, line.line);
        if (offset < 0) continue;
        gl.uniform4f(q.u.uTint, line.r, line.g, line.b, line.alpha);
        gl.drawArrays(gl.TRIANGLE_STRIP, offset, 4);
      }
    }
    gl.disable(gl.BLEND);

    if (onScreen >= 0.6) {
      this.blurStep(onScreen / d);
      this.blurPass(t.tex[0], t.fb[1], t, 1, 0);
      this.blurPass(t.tex[1], t.fb[0], t, 0, 1);
    }
    return t.tex[0];
  }

  lineOffset(headline: number, line: number): number {
    var lines = this.input.lines;
    if (line >= lines[headline].length) return -1;
    var first = 0;
    for (var i = 0; i < headline; i++) first += lines[i].length;
    return (first + line) * 4;
  }

  attrib(p: Program, name: string, size: number, stride: number, offset: number): void {
    var gl = this.gl;
    var loc = p.a[name];
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, size, gl.FLOAT, false, stride, offset);
  }

  drawLanding(s: Scene): void {
    var gl = this.gl;
    var layer = this.drawLayer(s);
    var c = this.input.palette;
    var w = this.input.width;
    var h = this.input.height;
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, w, h);
    gl.disable(gl.BLEND);
    var p = this.card;
    gl.useProgram(p.id);
    this.bind(0, layer, p.u.uLayer);
    gl.uniform3fv(p.u.uBlack, c.black);
    gl.uniform3fv(p.u.uBandA, c.bandA);
    gl.uniform3fv(p.u.uBandB, c.bandB);
    gl.uniform3fv(p.u.uEdge, c.edge);
    gl.uniform3fv(p.u.uFillTop, c.fillTop);
    gl.uniform3fv(p.u.uFillMid, c.fillMid);
    gl.uniform3fv(p.u.uFillLow, c.fillLow);
    gl.uniform3fv(p.u.uDarkTop, c.darkTop);
    gl.uniform3fv(p.u.uDarkLow, c.darkLow);
    var shift = s.cardShift * 1920;
    gl.uniform4f(p.u.uCard, CARD_X + shift, CARD_Y, CARD_W, CARD_H);
    var first = s.layerCount > 0 ? s.layers[0] : null;
    var gain = first ? first.gain : 1;
    gl.uniform4f(p.u.uState, s.band, s.emptyFill, s.layerCount > 0 ? 1 : 0, gain);
    gl.uniform1f(p.u.uPx, 1080 / h);
    gl.uniform1f(p.u.uRadius, CARD.radiusPx);
    this.fullQuad();

    gl.enable(gl.BLEND);
    var input = this.input;
    if (s.peek > 0 && this.peekTex) {
      var e = this.peek;
      var pw = (1 - PEEK.u0) * 1920;
      var ph = (PEEK.v1 - PEEK.v0) * 1080;
      gl.useProgram(e.id);
      this.bind(0, this.peekTex, e.u.uTex);
      gl.uniform4f(e.u.uRect, PEEK.u0 * 1920, PEEK.v0 * 1080, pw, ph);
      gl.uniform4f(e.u.uUv, 0, 0, 1, 1);
      gl.uniform1f(e.u.uAlpha, s.peek);
      gl.uniform2f(e.u.uSize, pw, ph);
      gl.uniform1f(e.u.uRadius, CARD.radiusPx);
      gl.uniform1f(e.u.uPx, 1080 / h);
      this.fullQuad();
    }
    var r = this.rect;
    gl.useProgram(r.id);
    if (s.cardWordAlpha > 0) {
      var mh = input.markHeight * s.cardWordScale;
      var mw = mh * input.wordAspect;
      this.bind(0, this.wordTex, r.u.uTex);
      gl.uniform4f(r.u.uRect, input.markCentre[0] - mw / 2 + shift, input.markCentre[1] - mh / 2, mw, mh);
      gl.uniform4f(r.u.uUv, 0, 0, 1, 1);
      gl.uniform1f(r.u.uAlpha, s.cardWordAlpha);
      this.fullQuad();
    }
    if (this.uiTex) {
      var u = input.uiRect;
      this.bind(0, this.uiTex, r.u.uTex);
      gl.uniform4f(r.u.uRect, u[0] + shift, u[1], u[2], u[3]);
      gl.uniform4f(r.u.uUv, 0, 0, 1, 1);
      gl.uniform1f(r.u.uAlpha, 1);
      this.fullQuad();
    }
    gl.disable(gl.BLEND);
  }
}
