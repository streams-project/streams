// The remote's key names. Needs no SDK and no Roku.
//
// The 15 strings in `ROKU_KEYS` are Roku's own, from
// https://developer.roku.com/dev/docs/onkeyevent (page updated 2026-11-10, read 2026-11-12). The
// test below is the thing that will catch it if that list is ever edited from memory: a key this
// surface answers must be one Roku sends, and every key Roku sends must be either answered or
// deliberately left alone.

import { ROKU_KEYS, pressWord, turn, wordOf, type Word } from "../keys";

/** The six words a screen answers. (`Remote.kt:26`) */
const WORDS: Word[] = ["up", "down", "left", "right", "ok", "back"];

/** The keys this surface answers, and the word each becomes. */
const ANSWERED: Record<string, Word> = {
  up: "up",
  down: "down",
  left: "left",
  right: "right",
  OK: "ok",
  back: "back",
};

/** The keys Roku sends that no screen here answers yet. The player will take most of them. */
const LEFT_ALONE = [
  "replay",
  "play",
  "playonly",
  "rewind",
  "fastforward",
  "options",
  "pause",
  "channelup",
  "channeldown",
];

describe("the key list", () => {
  it("is exactly the keys Roku documents, each one answered or deliberately left alone", () => {
    expect([...ROKU_KEYS].sort()).toEqual([...Object.keys(ANSWERED), ...LEFT_ALONE].sort());
  });

  it("answers every key it says it answers, and nothing else", () => {
    for (const [key, word] of Object.entries(ANSWERED)) expect(wordOf(key)).toBe(word);
    for (const key of LEFT_ALONE) expect(wordOf(key)).toBeNull();
  });

  it("is case sensitive, as Roku's page says: OK is the only key with capitals", () => {
    expect(wordOf("OK")).toBe("ok");
    expect(wordOf("ok")).toBeNull();
    expect(wordOf("Back")).toBeNull();
  });

  it("gives nothing for a key that is not a Roku key at all", () => {
    expect(wordOf("")).toBeNull();
    expect(wordOf("select")).toBeNull();
    expect(wordOf("enter")).toBeNull();
  });

  it("turns every key Roku sends into a word or nothing, and never throws", () => {
    for (const key of ROKU_KEYS) {
      const word = wordOf(key);
      if (word !== null) expect(WORDS).toContain(word);
    }
  });
});

describe("a right to left page", () => {
  it("swaps left and right, and leaves the other four alone", () => {
    expect(turn("left")).toBe("right");
    expect(turn("right")).toBe("left");
    for (const word of ["up", "down", "ok", "back"] as Word[]) expect(turn(word)).toBe(word);
  });

  it("swaps once, at the one place a press is read", () => {
    expect(pressWord("left", false)).toBe("left");
    expect(pressWord("left", true)).toBe("right");
    expect(pressWord("up", true)).toBe("up");
    expect(pressWord("OK", true)).toBe("ok");
  });

  it("turning twice is the same as not turning, so no page can mirror itself twice", () => {
    for (const word of WORDS) expect(turn(turn(word))).toBe(word);
  });
});
