// Writes generated sample title cards as SVG. These are plain shapes made from
// a seed, so the repository never holds real title art.

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { rng } from "../src/landing/curves.ts";

const W = 640;
const H = 360;

const PALETTES = [
  ["#14213d", "#fca311", "#e5e5e5", "#b5451b"],
  ["#0b3954", "#087e8b", "#ff5a5f", "#bfd7ea"],
  ["#2b193d", "#c32f6b", "#f7b267", "#f4845f"],
  ["#10451d", "#2d6a4f", "#95d5b2", "#ffb703"],
  ["#03045e", "#0077b6", "#90e0ef", "#caf0f8"],
  ["#3c096c", "#7b2cbf", "#e0aaff", "#ffd60a"],
  ["#370617", "#9d0208", "#f48c06", "#ffba08"],
  ["#001219", "#005f73", "#94d2bd", "#ee9b00"],
  ["#22223b", "#4a4e69", "#c9ada7", "#f2e9e4"],
  ["#1a1423", "#b75d69", "#eacdc2", "#774c60"],
  ["#073b4c", "#118ab2", "#06d6a0", "#ffd166"],
  ["#231942", "#5e548e", "#be95c4", "#ef476f"]
];

type Random = () => number;

function hills(c: string[], r: Random): string {
  const sunX = 140 + r() * 360;
  const sunY = 70 + r() * 90;
  let out = `<circle cx="${sunX | 0}" cy="${sunY | 0}" r="${(38 + r() * 40) | 0}" fill="${c[3]}"/>`;
  for (let band = 0; band < 3; band++) {
    const base = 190 + band * 52;
    const a = 18 + r() * 26;
    const f = 0.006 + r() * 0.012;
    const ph = r() * 6;
    let d = `M0 ${H}`;
    for (let x = 0; x <= W; x += 32) d += `L${x} ${(base + Math.sin(x * f + ph) * a) | 0}`;
    out += `<path d="${d}L${W} ${H}Z" fill="${c[band === 2 ? 0 : band === 1 ? 1 : 2]}" opacity="${band === 0 ? 0.65 : 1}"/>`;
  }
  return out;
}

function peaks(c: string[], r: Random): string {
  let out = "";
  for (let i = 0; i < 7; i++) {
    const x = r() * W;
    const w = 150 + r() * 220;
    const h = 120 + r() * 170;
    out += `<path d="M${(x - w) | 0} ${H}L${x | 0} ${(H - h) | 0}L${(x + w) | 0} ${H}Z" fill="${c[i % 3]}" opacity="${0.72 + r() * 0.28}"/>`;
    out += `<path d="M${(x - 34) | 0} ${(H - h + 46) | 0}L${x | 0} ${(H - h) | 0}L${(x + 34) | 0} ${(H - h + 46) | 0}Z" fill="${c[3]}"/>`;
  }
  return out;
}

function orbits(c: string[], r: Random): string {
  const x = 170 + r() * 300;
  const y = 120 + r() * 120;
  let out = `<circle cx="${x | 0}" cy="${y | 0}" r="70" fill="${c[1]}"/>`;
  for (let i = 0; i < 4; i++) {
    out += `<ellipse cx="${x | 0}" cy="${y | 0}" rx="${120 + i * 48}" ry="${34 + i * 16}" fill="none" stroke="${c[2]}" stroke-width="${3 + i}" opacity="${0.9 - i * 0.16}" transform="rotate(${(-22 + r() * 44) | 0} ${x | 0} ${y | 0})"/>`;
  }
  for (let i = 0; i < 26; i++) out += `<circle cx="${(r() * W) | 0}" cy="${(r() * H) | 0}" r="${(1 + r() * 3) | 0}" fill="${c[3]}"/>`;
  return out;
}

function skyline(c: string[], r: Random): string {
  let out = `<circle cx="${(100 + r() * 440) | 0}" cy="${(60 + r() * 70) | 0}" r="${(30 + r() * 26) | 0}" fill="${c[3]}"/>`;
  for (let layer = 0; layer < 2; layer++) {
    let x = -10;
    while (x < W) {
      const w = 40 + r() * 70;
      const h = (layer === 0 ? 150 : 80) + r() * 130;
      out += `<rect x="${x | 0}" y="${(H - h) | 0}" width="${w | 0}" height="${h | 0}" fill="${c[layer === 0 ? 1 : 0]}"/>`;
      for (let wy = H - h + 12; wy < H - 10; wy += 22) {
        for (let wx = x + 8; wx < x + w - 10; wx += 16) {
          if (r() < 0.45) out += `<rect x="${wx | 0}" y="${wy | 0}" width="7" height="11" fill="${c[3]}" opacity="${layer === 0 ? 0.45 : 0.9}"/>`;
        }
      }
      x += w + 2;
    }
  }
  return out;
}

function rays(c: string[], r: Random): string {
  const x = 160 + r() * 320;
  const y = 180 + r() * 180;
  let out = "";
  const count = 14 + ((r() * 10) | 0);
  for (let i = 0; i < count; i++) {
    const a0 = (i / count) * Math.PI * 2;
    const a1 = ((i + 0.5) / count) * Math.PI * 2;
    out += `<path d="M${x | 0} ${y | 0}L${(x + Math.cos(a0) * 900) | 0} ${(y + Math.sin(a0) * 900) | 0}L${(x + Math.cos(a1) * 900) | 0} ${(y + Math.sin(a1) * 900) | 0}Z" fill="${c[1]}"/>`;
  }
  out += `<circle cx="${x | 0}" cy="${y | 0}" r="${(62 + r() * 50) | 0}" fill="${c[3]}"/><circle cx="${x | 0}" cy="${y | 0}" r="26" fill="${c[2]}"/>`;
  return out;
}

function waves(c: string[], r: Random): string {
  let out = "";
  for (let band = 0; band < 6; band++) {
    const base = 80 + band * 52;
    const a = 12 + r() * 24;
    const f = 0.01 + r() * 0.02;
    const ph = r() * 6;
    let d = `M0 ${H}`;
    for (let x = 0; x <= W; x += 24) d += `L${x} ${(base + Math.sin(x * f + ph) * a) | 0}`;
    out += `<path d="${d}L${W} ${H}Z" fill="${c[1 + (band % 3)]}" opacity="${0.5 + band * 0.1}"/>`;
  }
  return out;
}

function blocks(c: string[], r: Random): string {
  let out = "";
  for (let i = 0; i < 9; i++) {
    const x = ((i % 3) * W) / 3;
    const y = (Math.floor(i / 3) * H) / 3;
    const kind = r();
    const fill = c[1 + ((r() * 3) | 0)];
    if (kind < 0.34) out += `<circle cx="${(x + W / 6) | 0}" cy="${(y + H / 6) | 0}" r="52" fill="${fill}"/>`;
    else if (kind < 0.67) out += `<rect x="${(x + 18) | 0}" y="${(y + 14) | 0}" width="${(W / 3 - 36) | 0}" height="${(H / 3 - 28) | 0}" fill="${fill}"/>`;
    else out += `<path d="M${x | 0} ${(y + H / 3) | 0}L${(x + W / 3) | 0} ${(y + H / 3) | 0}L${(x + W / 6) | 0} ${y | 0}Z" fill="${fill}"/>`;
  }
  return out;
}

const SCENES = [hills, peaks, orbits, skyline, rays, waves, blocks];

export function cardSvg(index: number, seed: number): string {
  const r = rng(seed + index * 104729);
  const colours = PALETTES[index % PALETTES.length];
  const scene = SCENES[(index * 5 + Math.floor(index / 7)) % SCENES.length];
  const number = String(index + 1).padStart(2, "0");
  const sky = `<linearGradient id="s" x1="0" y1="0" x2="${r() < 0.5 ? 0 : 1}" y2="1"><stop offset="0" stop-color="${colours[0]}"/><stop offset="1" stop-color="${colours[1]}"/></linearGradient>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}"><defs>${sky}</defs><rect width="${W}" height="${H}" fill="url(#s)"/>${scene(colours, r)}<g font-family="sans-serif" font-weight="800"><rect x="22" y="${H - 66}" width="118" height="44" rx="8" fill="#000" opacity="0.5"/><text x="81" y="${H - 34}" font-size="28" fill="#fff" text-anchor="middle">No. ${number}</text></g></svg>`;
}

export function writeCards(folder: string, count: number, seed: number): void {
  rmSync(folder, { recursive: true, force: true });
  mkdirSync(folder, { recursive: true });
  for (let i = 0; i < count; i++) {
    writeFileSync(join(folder, `card-${String(i + 1).padStart(2, "0")}.svg`), cardSvg(i, seed));
  }
}
