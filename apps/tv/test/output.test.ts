// Checks the built page and the repository content.

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { test } from "node:test";
import { bundle } from "../tools/bundle.ts";
import { cardSvg } from "../tools/sample-cards.ts";

const ROOT = resolve(import.meta.dirname, "..");

test("the page compiles to ES5 syntax", () => {
  const js = bundle(join(ROOT, "src/landing"), "page.ts");
  // Strip strings so shader text cannot trigger a false match.
  const code = js.replace(/"(?:[^"\\\n]|\\.)*"/g, '""').replace(/'(?:[^'\\\n]|\\.)*'/g, "''");
  for (const pattern of [/=>/, /\bclass\s/, /\bconst\s/, /\blet\s/, /`/]) {
    assert.ok(!pattern.test(code), `found ${pattern}`);
  }
  assert.doesNotThrow(() => new Function(js));
});

test("sample cards are plain generated shapes", () => {
  const svg = cardSvg(3, 1);
  assert.ok(svg.startsWith("<svg"));
  assert.ok(!/<image|href=/.test(svg), "no embedded art");
});

test("no price and no plan page", () => {
  const config = readFileSync(join(ROOT, "landing/config.json"), "utf8");
  const headlines = readFileSync(join(ROOT, "landing/headlines.txt"), "utf8");
  assert.ok(!/price|plan|month|£|\$/i.test(config + headlines));
});

test("the repository holds no binary images", () => {
  const files = execFileSync("git", ["-C", ROOT, "ls-files", "--cached", "--others", "--exclude-standard"], { encoding: "utf8" });
  assert.ok(!/\.(png|jpe?g|webp|gif|mp4|webm|woff2?|ttf|otf)$/im.test(files));
});
