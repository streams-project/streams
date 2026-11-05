// Generator for the TV signed out opening scene and landing.
//
//   node tools/landing.ts cards                write the generated sample cards
//   node tools/landing.ts build                build the live page into dist/landing
//   node tools/landing.ts serve                serve the live page
//   node tools/landing.ts frames --out DIR     render still frames
//   node tools/landing.ts video --out FILE     render a preview video (WebM)
//   node tools/landing.ts measure              measure frame time
//
// Options: --images DIR, --headlines FILE, --config FILE, --dist DIR.
// frames, video and measure need CHROME_PATH set to a Chrome binary.

import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import type { Server } from "node:http";
import type { AddressInfo } from "node:net";
import { createRequire } from "node:module";
import { basename, dirname, extname, join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { bundle } from "./bundle.ts";
import { launchChrome } from "./chrome.ts";
import type { Chrome } from "./chrome.ts";
import { writeCards } from "./sample-cards.ts";
import { webm } from "./webm.ts";
import { cycleStartMs, FRAME_MS, makePlan } from "../src/landing/timeline.ts";

const ROOT = resolve(import.meta.dirname, "..");
const IMAGE_TYPES = new Set([".jpg", ".jpeg", ".png", ".webp", ".svg"]);
const MIME: Record<string, string> = {
  ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml",
  ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".webp": "image/webp", ".woff2": "font/woff2",
  ".txt": "text/plain", ".mp3": "audio/mpeg", ".m4a": "audio/mp4", ".ogg": "audio/ogg", ".opus": "audio/ogg", ".wav": "audio/wav"
};

const { values: args, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    images: { type: "string", default: join(ROOT, "landing/cards") },
    headlines: { type: "string", default: join(ROOT, "landing/headlines.txt") },
    config: { type: "string", default: join(ROOT, "landing/config.json") },
    dist: { type: "string", default: join(ROOT, "dist/landing") },
    out: { type: "string" },
    port: { type: "string", default: "8090" },
    fps: { type: "string" },
    from: { type: "string" },
    to: { type: "string" },
    times: { type: "string" },
    throttle: { type: "string", default: "1" },
    count: { type: "string", default: "48" },
    seconds: { type: "string", default: "45" }
  }
});

function readHeadlines(file: string): string[][] {
  return readFileSync(file, "utf8").split("\n")
    .map((l) => l.trim()).filter((l) => l && !l.startsWith("#"))
    .map((l) => l.split("|").map((p) => p.trim()));
}

function build(): void {
  const dist = resolve(args.dist);
  const config = JSON.parse(readFileSync(args.config, "utf8"));
  const headlines = readHeadlines(args.headlines);
  if (!headlines.length) throw new Error(`no headlines in ${args.headlines}`);
  const images = existsSync(args.images)
    ? readdirSync(args.images).filter((f) => IMAGE_TYPES.has(extname(f).toLowerCase())).sort()
    : [];
  if (!images.length) throw new Error(`no title images in ${args.images}`);

  rmSync(dist, { recursive: true, force: true });
  mkdirSync(join(dist, "titles"), { recursive: true });
  const names = images.map((f, i) => {
    const name = `titles/${String(i).padStart(3, "0")}${extname(f).toLowerCase()}`;
    copyFileSync(join(args.images, f), join(dist, name));
    return name;
  });

  const require = createRequire(join(ROOT, "package.json"));
  let faces = "";
  for (const face of config.font.files as { weight: number; file: string }[]) {
    const name = basename(face.file);
    copyFileSync(require.resolve(face.file), join(dist, name));
    faces += `@font-face{font-family:"${config.font.family}";font-weight:${face.weight};src:url(${name}) format("woff2")}\n`;
  }
  copyFileSync(require.resolve(join(dirname(config.font.files[0].file), "../LICENSE")), join(dist, "font-licence.txt"));
  // Audio slot: no sound ships, but a file named in the config is copied in.
  if (config.audio?.src) {
    const source = resolve(dirname(resolve(args.config)), config.audio.src);
    const name = `audio${extname(source).toLowerCase()}`;
    copyFileSync(source, join(dist, name));
    config.audio.src = name;
  }
  const html = readFileSync(join(ROOT, "src/landing/index.html"), "utf8").replace("/*FONTS*/", faces);
  writeFileSync(join(dist, "index.html"), html);
  writeFileSync(join(dist, "landing.js"), bundle(join(ROOT, "src/landing"), "page.ts"));
  writeFileSync(join(dist, "landing.json"), JSON.stringify({ config, headlines, images: names }));
  console.log(`built ${dist}: ${names.length} title images, ${headlines.length} headlines`);
}

function serve(root: string, port: number): Promise<Server> {
  const server = createServer((req, res) => {
    const path = decodeURIComponent((req.url ?? "/").split("?")[0]);
    const file = join(root, path === "/" ? "index.html" : path);
    if (!file.startsWith(root) || !existsSync(file)) {
      res.writeHead(404).end();
      return;
    }
    res.writeHead(200, { "content-type": MIME[extname(file)] ?? "application/octet-stream" });
    res.end(readFileSync(file));
  });
  return new Promise((ok) => server.listen(port, "127.0.0.1", () => ok(server)));
}

async function withPage<T>(job: (chrome: Chrome, config: { size: { width: number; height: number }; logoRestMs: number }, headlineCount: number) => Promise<T>): Promise<T> {
  const path = process.env.CHROME_PATH;
  if (!path || !existsSync(path)) throw new Error("set CHROME_PATH to a Chrome binary");
  build();
  const root = resolve(args.dist);
  const data = JSON.parse(readFileSync(join(root, "landing.json"), "utf8"));
  const server = await serve(root, 0);
  const port = (server.address() as AddressInfo).port;
  const chrome = await launchChrome(path);
  try {
    await chrome.open(`http://127.0.0.1:${port}/?preview`, data.config.size.width, data.config.size.height);
    await chrome.evaluate(`new Promise(function(ok,no){var n=0;(function w(){if(window.landingPreview&&landingPreview.isReady())ok(1);else if(++n>600)no(new Error("page not ready"));else setTimeout(w,50)})()})`);
    return await job(chrome, data.config, data.headlines.length);
  } finally {
    await chrome.close();
    server.close();
  }
}

// Stills that cover each stage of the sequence.
function keyFrames(logoRestMs: number, headlineCount: number): { name: string; ms: number }[] {
  const plan = makePlan(logoRestMs, headlineCount);
  const open = (f: number): number => plan.openingStartMs + f * FRAME_MS;
  const land = (f: number): number => plan.landingStartMs + f * FRAME_MS;
  const cycle = (c: number, f: number): number => cycleStartMs(plan, c) + f * FRAME_MS;
  return [
    { name: "01-logo-ring", ms: 900 },
    { name: "02-stretch", ms: open(14) },
    { name: "03-stretch-late", ms: open(19) },
    { name: "04-column", ms: open(24) },
    { name: "05-column-wide", ms: open(29) },
    { name: "06-burst", ms: open(31) },
    { name: "07-full-width", ms: open(33) },
    { name: "08-colour-split", ms: open(35) },
    { name: "09-stripes-run", ms: open(39) },
    { name: "10-stripes-run-late", ms: open(43) },
    { name: "11-stripes-fade", ms: open(47) },
    { name: "12-black-ring", ms: open(54) },
    { name: "13-card-slide", ms: land(1) },
    { name: "14-empty-card", ms: land(12) },
    { name: "15-wall", ms: land(40) },
    { name: "16-wall-to-logo", ms: land(68) },
    { name: "17-logo-on-card", ms: land(78) },
    { name: "18-first-layer-entry", ms: land(92) },
    { name: "19-first-headline", ms: land(135) },
    { name: "20-headline-change", ms: land(156) },
    { name: "21-second-headline", ms: land(190) },
    { name: "22-third-headline", ms: land(250) },
    { name: "23-peek", ms: land(290) },
    { name: "24-fourth-headline", ms: land(315) },
    { name: "25-exit-start", ms: cycle(0, 12) },
    { name: "26-exit-blur", ms: cycle(0, 19) },
    { name: "27-exit-late", ms: cycle(0, 22) },
    { name: "28-gap", ms: cycle(0, 25) },
    { name: "29-entry-first", ms: cycle(0, 29) },
    { name: "30-entry-blur", ms: cycle(0, 33) },
    { name: "31-entry-settle", ms: cycle(0, 41) },
    { name: "32-fifth-headline", ms: cycle(0, 62) },
    { name: "33-sixth-headline", ms: cycle(1, 62) },
    { name: "34-seventh-headline", ms: cycle(2, 62) },
    { name: "35-eighth-headline", ms: cycle(3, 62) },
    { name: "36-ninth-headline", ms: cycle(4, 62) },
    { name: "37-tenth-headline", ms: cycle(5, 62) }
  ];
}

const CAPTURE = `function(ms){landingPreview.seek(ms);var c=document.getElementById("stage");return new Promise(function(ok){c.toBlob(function(b){var r=new FileReader();r.onload=function(){ok(String(r.result).split(",")[1])};r.readAsDataURL(b)},"image/png")})}`;

async function frames(): Promise<void> {
  const out = resolve(args.out ?? join(ROOT, "dist/preview"));
  mkdirSync(out, { recursive: true });
  await withPage(async (chrome, config, count) => {
    let list = keyFrames(config.logoRestMs, count);
    if (args.times) {
      list = args.times.split(",").map((t) => ({ name: `t${t.padStart(6, "0")}`, ms: Number(t) }));
    } else if (args.fps) {
      const step = 1000 / Number(args.fps);
      const from = Number(args.from ?? 0);
      const to = Number(args.to ?? from + 5000);
      list = [];
      for (let ms = from, i = 0; ms <= to; ms = from + ++i * step) list.push({ name: `f${String(Math.round(ms)).padStart(6, "0")}`, ms });
    }
    for (const item of list) {
      const b64 = await chrome.evaluate<string>(`(${CAPTURE})(${item.ms})`);
      writeFileSync(join(out, `${item.name}.png`), Buffer.from(b64, "base64"));
    }
    console.log(`wrote ${list.length} frames to ${out}`);
  });
}

async function video(): Promise<void> {
  const out = resolve(args.out ?? join(ROOT, "dist/preview.webm"));
  const fps = Number(args.fps ?? 30);
  const seconds = Number(args.seconds);
  await withPage(async (chrome, config) => {
    const { width, height } = config.size;
    await chrome.evaluate(`window.__enc={chunks:[],error:null,e:new VideoEncoder({output:function(c){var b=new Uint8Array(c.byteLength);c.copyTo(b);__enc.chunks.push({ms:c.timestamp/1000,key:c.type==="key",b:b})},error:function(e){__enc.error=String(e)}})};__enc.e.configure({codec:"vp09.00.40.08",width:${width},height:${height},bitrate:16000000,framerate:${fps}});0`);
    const total = Math.round(seconds * fps);
    for (let i = 0; i < total; i += 30) {
      await chrome.evaluate(`(async function(){var c=document.getElementById("stage");for(var i=${i};i<${Math.min(i + 30, total)};i++){var ms=i*1000/${fps};landingPreview.seek(ms);var f=new VideoFrame(c,{timestamp:Math.round(ms*1000)});__enc.e.encode(f,{keyFrame:i%${fps * 2}===0});f.close();while(__enc.e.encodeQueueSize>4)await new Promise(function(r){setTimeout(r,2)})}})()`);
    }
    await chrome.evaluate(`__enc.e.flush()`);
    const error = await chrome.evaluate<string | null>(`__enc.error`);
    if (error) throw new Error(error);
    const n = await chrome.evaluate<number>(`__enc.chunks.length`);
    const chunks: { data: Buffer; ms: number; key: boolean }[] = [];
    for (let i = 0; i < n; i += 40) {
      const part = await chrome.evaluate<{ ms: number; key: boolean; d: string }[]>(`__enc.chunks.slice(${i},${i + 40}).map(function(c){var s="";for(var j=0;j<c.b.length;j+=8192)s+=String.fromCharCode.apply(null,c.b.subarray(j,j+8192));return{ms:c.ms,key:c.key,d:btoa(s)}})`);
      for (const p of part) chunks.push({ data: Buffer.from(p.d, "base64"), ms: p.ms, key: p.key });
    }
    writeFileSync(out, webm(chunks, width, height, "V_VP9"));
    console.log(`wrote ${out}: ${chunks.length} frames at ${fps} fps`);
  });
}

function percentile(sorted: number[], p: number): number {
  if (!sorted.length) return 0;
  return sorted[Math.min(sorted.length - 1, Math.floor(p * sorted.length))];
}

function stats(values: number[]): { n: number; median: number; p95: number; p99: number; max: number } {
  const s = [...values].sort((a, b) => a - b);
  const round = (x: number): number => Math.round(x * 100) / 100;
  return { n: s.length, median: round(percentile(s, 0.5)), p95: round(percentile(s, 0.95)), p99: round(percentile(s, 0.99)), max: round(s[s.length - 1] ?? 0) };
}

async function measure(): Promise<void> {
  const seconds = Number(args.seconds);
  const throttle = Number(args.throttle);
  await withPage(async (chrome, config, count) => {
    if (throttle > 1) await chrome.send("Emulation.setCPUThrottlingRate", { rate: throttle });
    const gpu = await chrome.evaluate<string>(`landingPreview.gpu()`);
    const startup = await chrome.evaluate<Record<string, number>>(`landingPreview.startup()`);
    const plan = makePlan(config.logoRestMs, count);
    const run = (): Promise<number[][]> =>
      chrome.evaluate<number[][]>(`new Promise(function(ok){landingPreview.run(0,${seconds * 1000},ok)})`);
    await run(); // warm up shaders and textures, not counted
    const rows = await run();
    const stage = (r: number[]): string => (r[3] < plan.openingStartMs ? "logo" : r[3] < plan.landingStartMs ? "opening" : "landing");
    const pick = (s: string, col: number, drawnOnly: boolean): number[] =>
      rows.filter((r) => stage(r) === s && (!drawnOnly || r[2] === 1)).map((r) => r[col]);
    const intervals = rows.slice(1).map((r) => r[0]);
    const gpuPerFrame: Record<string, number> = {};
    for (const item of keyFrames(config.logoRestMs, count)) {
      const ms = await chrome.evaluate<number>(`landingPreview.bench(${item.ms},120)`);
      gpuPerFrame[item.name] = Math.round(ms * 1000) / 1000;
    }
    const gpuValues = Object.values(gpuPerFrame);
    const report = {
      gpu, cpuThrottle: throttle, canvas: `${config.size.width}x${config.size.height}`, secondsMeasured: seconds,
      startupMs: Object.fromEntries(Object.entries(startup).map(([k, v]) => [k, Math.round(v)])),
      frameIntervalMs: { all: stats(intervals), opening: stats(pick("opening", 0, false)), landing: stats(pick("landing", 0, false)) },
      intervalsOver17_5ms: intervals.filter((x) => x > 17.5).length,
      intervalsOver34ms: intervals.filter((x) => x > 34).length,
      jsDrawMs: { opening: stats(pick("opening", 1, true)), landing: stats(pick("landing", 1, true)) },
      framesDrawn: rows.filter((r) => r[2] === 1).length,
      framesSkippedUnchanged: rows.filter((r) => r[2] === 0).length,
      gpuMsPerFrame: { worst: Math.max(...gpuValues), median: stats(gpuValues).median, byKeyFrame: gpuPerFrame }
    };
    const text = JSON.stringify(report, null, 2);
    console.log(text);
    if (args.out) writeFileSync(args.out, text);
  });
}

async function main(): Promise<void> {
  const command = positionals[0];
  if (command === "cards") {
    const config = JSON.parse(readFileSync(args.config, "utf8"));
    writeCards(join(ROOT, "landing/cards"), Number(args.count), config.seed);
    console.log(`wrote ${args.count} sample cards`);
  } else if (command === "build") {
    build();
  } else if (command === "serve") {
    build();
    await serve(resolve(args.dist), Number(args.port));
    console.log(`live page at http://127.0.0.1:${args.port}/`);
  } else if (command === "frames") {
    await frames();
  } else if (command === "video") {
    await video();
  } else if (command === "measure") {
    await measure();
  } else {
    console.log("commands: cards, build, serve, frames, video, measure");
    process.exitCode = 1;
  }
}

await main();
