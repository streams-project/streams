// What a screen says when this app cannot reach its server, and the hole it is drawn into.
//
// The hard case is not the sentence. It is that this screen may have no words at all: every word
// this app draws arrives from `GET /v1/strings/{lang}`, so a device that cannot reach its server
// has no vocabulary to say so with. `error.no_server` and `say.try_again` exist in all eighteen
// languages (`src/tests/words.test.ts` holds that) and none of them is on the device.
//
// So the cases below are mostly the empty ones, and the rule they hold is `Lost.kt:114-115`: the
// address is the one thing on the screen that did not have to be fetched to be known, it is last,
// and it is never missing - which is also what keeps the spoken sentence from being empty, and an
// empty sentence leaves the only focusable node on the screen unnamed.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { lostSays, lostTrouble, tryAgainWord } from "../lost";
import { KEY } from "../words";

const ROOT = join(__dirname, "..", "..", "..", "..");

/** The real words of a language, from `strings/`, as the server would send them. */
function strings(language: string): Record<string, string> {
  return JSON.parse(readFileSync(join(ROOT, "strings", language + ".json"), "utf8")) as Record<
    string,
    string
  >;
}

const ENGLISH = strings("en");
const ARABIC = strings("ar");
const ADDRESS = "https://streams.example";

describe("the words this screen needs exist to be missing", () => {
  it("in English", () => {
    expect(ENGLISH[KEY.errorNoServer]).toBeDefined();
    expect(ENGLISH[KEY.sayTryAgain]).toBeDefined();
  });

  it("and in a language that runs the other way", () => {
    expect(ARABIC[KEY.errorNoServer]).toBeDefined();
    expect(ARABIC[KEY.sayTryAgain]).toBeDefined();
  });
});

describe("what a person is handed when the words did arrive", () => {
  it("is the trouble, the way out, and the address, in that order", () => {
    const said = lostSays(ENGLISH, ADDRESS);
    expect(said.split("\n")).toEqual([
      ENGLISH[KEY.errorNoServer],
      ENGLISH[KEY.sayTryAgain],
      ADDRESS,
    ]);
  });

  it("in whatever language they arrived in", () => {
    expect(lostSays(ARABIC, ADDRESS)).toContain(ARABIC[KEY.errorNoServer] ?? "");
  });
});

describe("what a person is handed when they did not", () => {
  it("is the address, which is the whole of what this app knows without asking", () => {
    expect(lostSays({}, ADDRESS)).toBe(ADDRESS);
  });

  it("and the sentence above it is drawn by nothing rather than as an empty line", () => {
    expect(lostTrouble({})).toBe("");
    expect(tryAgainWord({})).toBe("");
  });

  it("and it is still not the empty string, so the one control on the screen has a name", () => {
    expect(lostSays({}, ADDRESS).length).toBeGreaterThan(0);
  });

  it("which is the one case this whole screen exists for", () => {
    // A television whose server had gone drew 9 nodes, 0 focusable and 0 words, because every
    // word it draws comes from that server (`tv/app/Lost.kt:39-42`). This is that state.
    const said = lostSays({}, ADDRESS);
    expect(said.indexOf("\n")).toBe(-1);
    expect(said).toBe(ADDRESS);
  });
});

describe("a half translated server is not a blank screen", () => {
  it("so a language with one of the two words says that one, and the address", () => {
    const half: Record<string, string> = { [KEY.sayTryAgain]: "Eto'o" };
    expect(lostSays(half, ADDRESS).split("\n")).toEqual(["Eto'o", ADDRESS]);
  });
});
