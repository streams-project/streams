// Every word this surface asks for must exist. Needs no SDK and no Roku, only the repository.
//
// `src/words.ts` holds keys and no English, so nothing in the shipped app can be checked against
// `strings/en.json` at build time without pulling English into the bundle. This test reads the file
// from disk instead, which keeps the check and leaves the bundle clean.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { KEY, fill, say, sayBoth } from "../words";

const ROOT = join(__dirname, "..", "..", "..", "..");

function strings(language: string): Record<string, string> {
  return JSON.parse(readFileSync(join(ROOT, "strings", `${language}.json`), "utf8"));
}

describe("the keys this surface asks for", () => {
  const english = strings("en");

  it("are all in strings/en.json", () => {
    const missing = Object.values(KEY).filter((key) => english[key] === undefined);
    expect(missing).toEqual([]);
  });

  it("are all in every other language, so no screen falls back to nothing", () => {
    const languages = ["ar", "de", "es", "fa", "fr", "he", "hi", "it", "ja", "ko", "nl", "pl", "pt", "ru", "sv", "tr", "zh"];
    const gaps: string[] = [];
    for (const language of languages) {
      const words = strings(language);
      for (const key of Object.values(KEY)) {
        if (words[key] === undefined) gaps.push(`${language} ${key}`);
      }
    }
    expect(gaps).toEqual([]);
  });

  it("include the three reaction words the ratings row cannot be heard without", () => {
    for (const key of ["player.down", "player.up", "player.love"]) {
      expect(english[key]).toBeDefined();
    }
  });

  it("name the two the title page fills a number into", () => {
    expect(english[KEY.titlePlayEpisode]).toContain("{n}");
    expect(english[KEY.titleContinueEpisode]).toContain("{n}");
  });
});

describe("drawing a word", () => {
  it("draws nothing rather than a key when a word is missing", () => {
    expect(say({}, "nav.search")).toBe("");
  });

  it("isolates a value put into a word, so its order survives either direction", () => {
    expect(fill("Episode {n}", "n", 4)).toBe("Episode ⁨4⁩");
  });

  it("leaves a word alone when it has no place for the value", () => {
    expect(fill("Play", "n", 4)).toBe("Play");
  });
});

describe("what a person hears", () => {
  it("joins two things with a line end, so no language's punctuation is decided in code", () => {
    expect(sayBoth("Title", "Row")).toBe("Title\nRow");
  });

  it("drops what is missing rather than leaving a gap", () => {
    expect(sayBoth("Title", null)).toBe("Title");
    expect(sayBoth(undefined, "Row")).toBe("Row");
    expect(sayBoth("", "")).toBe("");
  });
});
