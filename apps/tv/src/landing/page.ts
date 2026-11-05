// Live page entry. The same draw path serves the TV and the preview renders.

import { audioSlot } from "./audio.ts";
import type { AudioSlot } from "./audio.ts";
import { buildAtlas, buildButtons, buildPeek, buildText, hexToRgb, loadImage, logoImage, MARK_CENTRE, MARK_HEIGHT, setOptions } from "./assets.ts";
import type { LandingData } from "./assets.ts";
import { buildGeometry } from "./layout.ts";
import { Renderer } from "./renderer.ts";
import type { Palette } from "./renderer.ts";
import { buildStripes } from "./stripes.ts";
import { LOGO_REST_V, makePlan, newScene, sceneAt } from "./timeline.ts";
import type { Plan, Scene } from "./timeline.ts";

interface Fonts { load(font: string): { then(ok: () => void, bad: () => void): void }; }

var state: {
  plan: Plan; scene: Scene; renderer: Renderer; restWidth: number; ready: boolean; audio: AudioSlot; startup: { [k: string]: number };
} | null = null;

function fetchJson(url: string, done: (data: LandingData) => void): void {
  var x = new XMLHttpRequest();
  x.open("GET", url);
  x.onload = function () { done(JSON.parse(x.responseText) as LandingData); };
  x.send();
}

function palette(data: LandingData): Palette {
  var c = data.config.colours;
  return {
    black: hexToRgb(c.nearBlack), ring: hexToRgb(c.ring), bandA: hexToRgb(c.bandLeft), bandB: hexToRgb(c.bandRight),
    edge: hexToRgb(c.glassEdge), fillTop: hexToRgb(c.fillTop), fillMid: hexToRgb(c.fillMid), fillLow: hexToRgb(c.fillLow),
    darkTop: hexToRgb(c.darkTop), darkLow: hexToRgb(c.darkLow)
  };
}

function loadFonts(data: LandingData, done: () => void): void {
  var fonts = (document as unknown as { fonts?: Fonts }).fonts;
  var f = data.config.font;
  if (!fonts) { setTimeout(done, 400); return; }
  var left = 2;
  var one = function (): void { if (--left === 0) done(); };
  fonts.load(f.headlineWeight + ' 40px "' + f.family + '"').then(one, one);
  fonts.load(f.buttonWeight + ' 26px "' + f.family + '"').then(one, one);
}

function loadTitles(data: LandingData, done: (images: HTMLImageElement[]) => void): void {
  var left = data.images.length;
  var images: HTMLImageElement[] = [];
  data.images.forEach(function (src, i) {
    loadImage(src, function (image) {
      if (image) images[i] = image;
      if (--left > 0) return;
      done(images.filter(function (m) { return !!m; }));
    });
  });
}

// Second stage of startup: runs behind the logo and the ring.
function prepareLanding(data: LandingData): void {
  var s = state!;
  var t0 = performance.now();
  loadFonts(data, function () {
    loadTitles(data, function (images) {
      var t1 = performance.now();
      var built = buildAtlas(images);
      var text = buildText(data.config, data.headlines);
      var ui = buildButtons(data.config);
      var input = s.renderer.input;
      input.atlas = built.canvas;
      input.text = text.canvas;
      input.lines = text.lines;
      input.ui = ui.canvas;
      input.uiRect = ui.rect;
      input.peek = buildPeek(images[data.config.seed % images.length]);
      input.geometry = buildGeometry(setOptions(text.lines, s.plan.setCount, s.plan.wallSet), built.atlas, data.config.seed);
      var t2 = performance.now();
      s.renderer.loadLanding();
      var t3 = performance.now();
      s.startup.loadMs = t1 - t0;
      s.startup.composeMs = t2 - t1;
      s.startup.uploadMs = t3 - t2;
      s.ready = true;
    });
  });
}

function live(): void {
  var s = state!;
  var start = -1;
  function tick(now: number): void {
    if (start < 0) start = now;
    var ms = now - start;
    // The logo and ring wait for the images, as a splash does.
    if (!s.ready && ms >= s.plan.logoRestMs) {
      ms = s.plan.logoRestMs - 1;
      start = now - ms;
    }
    s.renderer.draw(sceneAt(s.plan, ms, s.restWidth, s.scene));
    s.audio.sync(ms);
    window.requestAnimationFrame(tick);
  }
  window.requestAnimationFrame(tick);
}

// Hooks for the preview tool. Present only when the page is opened with ?preview.
function previewHooks(): void {
  var s = state!;
  var hooks = {
    isReady: function (): boolean { return s.ready; },
    plan: function (): Plan { return s.plan; },
    startup: function (): { [k: string]: number } { return s.startup; },
    seek: function (ms: number): void {
      s.renderer.last[0] = -1;
      s.renderer.draw(sceneAt(s.plan, ms, s.restWidth, s.scene));
    },
    gpu: function (): string {
      var gl = s.renderer.gl;
      var info = gl.getExtension("WEBGL_debug_renderer_info");
      return info ? String(gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) : "unknown";
    },
    // GPU throughput: draws the frame at ms count times, then waits for the
    // GPU once. The result is milliseconds per frame.
    bench: function (ms: number, count: number): number {
      var gl = s.renderer.gl;
      var pixel = new Uint8Array(4);
      var scene = sceneAt(s.plan, ms, s.restWidth, s.scene);
      s.renderer.last[0] = -1;
      s.renderer.draw(scene);
      gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
      var t0 = performance.now();
      for (var i = 0; i < count; i++) {
        s.renderer.last[0] = -1;
        s.renderer.draw(scene);
      }
      gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
      return (performance.now() - t0) / count;
    },
    // Runs in real time from fromMs for lengthMs, then calls done with one row
    // per animation frame: [interval ms, draw cost ms, drew 0 or 1, timeline ms].
    run: function (fromMs: number, lengthMs: number, done: (rows: number[][]) => void): void {
      var rows: number[][] = [];
      var start = -1;
      var last = 0;
      function tick(now: number): void {
        if (start < 0) { start = now; last = now; }
        var ms = fromMs + (now - start);
        var t0 = performance.now();
        var drew = s.renderer.draw(sceneAt(s.plan, ms, s.restWidth, s.scene));
        var cost = performance.now() - t0;
        rows.push([now - last, cost, drew ? 1 : 0, ms]);
        last = now;
        if (now - start < lengthMs) window.requestAnimationFrame(tick);
        else done(rows);
      }
      window.requestAnimationFrame(tick);
    }
  };
  (window as unknown as { landingPreview: typeof hooks }).landingPreview = hooks;
}

function boot(): void {
  var preview = /[?&]preview\b/.test(window.location.search);
  var canvasEl = document.getElementById("stage") as HTMLCanvasElement;
  fetchJson("landing.json", function (data) {
    var size = data.config.size;
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    logoImage(data.config, function (logo, aspect) {
      var plan = makePlan(data.config.logoRestMs, data.headlines.length);
      var renderer = new Renderer(canvasEl, {
        width: size.width, height: size.height, palette: palette(data),
        stripes: buildStripes(data.config.colours, data.config.seed),
        logo: logo, logoAspect: aspect, atlas: null, text: null, ui: null, uiRect: [0, 0, 0, 0], peek: null,
        geometry: null, lines: [], markCentre: MARK_CENTRE, markHeight: MARK_HEIGHT
      }, preview);
      state = {
        plan: plan, scene: newScene(), renderer: renderer, ready: false,
        restWidth: (LOGO_REST_V * 1080 * aspect) / 1920, audio: audioSlot(data.config.audio), startup: {}
      };
      if (preview) previewHooks();
      else live();
      prepareLanding(data);
    });
  });
}

boot();
