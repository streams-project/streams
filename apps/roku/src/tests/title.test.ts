// A title's page: which lines it has, and what each key does. Needs no SDK and no Roku.

import type { TitleDetail } from "../contract";
import { REACTION_COUNT, TitleFocus, linesOf, playWord } from "../title";

const WORDS: Record<string, string> = {
  "common.play": "P",
  "title.resume": "R",
  "title.play_episode": "E{n}",
  "title.continue_episode": "C{n}",
  "title.episodes_more": "EM",
  "title.more_like": "ML",
  "list.add": "A",
  "list.added": "D",
};

function film(extra: Partial<TitleDetail> = {}): TitleDetail {
  return { name: "f", title: "movie:1", ...extra };
}

function series(extra: Partial<TitleDetail> = {}): TitleDetail {
  return { name: "s", title: "tv:1", seasons: [{ number: 1, episode_count: 8 }], ...extra };
}

describe("the menu's lines", () => {
  it("gives a plain film two lines: play, and the list", () => {
    expect(linesOf(film())).toEqual(["play", "list"]);
  });

  it("adds episodes for anything with seasons, and the grid for anything with titles like it", () => {
    expect(linesOf(series())).toEqual(["play", "episodes", "list"]);
    expect(linesOf(film({ more_like_this: [{ name: "x", title: "movie:2" }] }))).toEqual([
      "play",
      "more",
      "list",
    ]);
  });

  it("does not add a line for an empty list of seasons or an empty grid", () => {
    expect(linesOf(film({ seasons: [], more_like_this: [] }))).toEqual(["play", "list"]);
  });
});

describe("what the first line says", () => {
  it("says play for a film, and resume for one already started", () => {
    expect(playWord(film(), WORDS)).toBe("P");
    expect(playWord(film({ resume: { position_s: 400 } }), WORDS)).toBe("R");
  });

  it("names the episode for a series, and carries on with one already started", () => {
    expect(playWord(series(), WORDS)).toBe("E⁨1⁩");
    const started = series({ resume: { position_s: 90, episode: { season: 2, episode: 4 } } });
    expect(playWord(started, WORDS)).toBe("C⁨4⁩");
  });

  it("names the episode rather than carrying on when the position is zero", () => {
    const fresh = series({ resume: { position_s: 0, episode: { season: 1, episode: 3 } } });
    expect(playWord(fresh, WORDS)).toBe("E⁨3⁩");
  });

  it("isolates the number, so a right to left line keeps it in its own order", () => {
    expect(playWord(series(), WORDS)).toContain("⁨");
    expect(playWord(series(), WORDS)).toContain("⁩");
  });
});

describe("the remote on a title's page", () => {
  it("opens on the menu's first line", () => {
    const focus = new TitleFocus(series());
    expect(focus.part).toBe("menu");
    expect(focus.here).toBe("play");
  });

  it("opens on the line the page was left on, when coming back", () => {
    expect(new TitleFocus(series(), 1).here).toBe("episodes");
    // A line that no longer exists cannot be opened on.
    expect(new TitleFocus(film(), 5).here).toBe("list");
  });

  it("stays at the last line on Down, as note 187 asks", () => {
    const focus = new TitleFocus(film());
    focus.press("down");
    expect(focus.here).toBe("list");
    focus.press("down");
    expect(focus.here).toBe("list");
  });

  it("goes up from the first line to the three ratings", () => {
    const focus = new TitleFocus(film());
    focus.press("up");
    expect(focus.part).toBe("reactions");
  });

  it("goes back on Left from the menu, as Back does", () => {
    expect(new TitleFocus(film()).press("left")).toEqual({ kind: "back" });
  });

  it("moves along the three ratings on Left and Right, and stops at each end", () => {
    const focus = new TitleFocus(film());
    focus.press("up");
    expect(focus.reaction).toBe(1);
    focus.press("left");
    expect(focus.reaction).toBe(0);
    focus.press("left");
    expect(focus.reaction).toBe(0);
    focus.press("right");
    focus.press("right");
    expect(focus.reaction).toBe(REACTION_COUNT - 1);
    focus.press("right");
    expect(focus.reaction).toBe(REACTION_COUNT - 1);
  });

  it("returns from the ratings to the menu on Back, and only then leaves", () => {
    const focus = new TitleFocus(film());
    focus.press("up");
    expect(focus.press("back")).toEqual({ kind: "nothing" });
    expect(focus.part).toBe("menu");
    expect(focus.press("back")).toEqual({ kind: "back" });
  });
});

describe("what a press does", () => {
  it("plays, opens episodes, opens the grid", () => {
    const focus = new TitleFocus(series({ more_like_this: [{ name: "x", title: "movie:2" }] }));
    expect(focus.press("ok")).toEqual({ kind: "play" });
    focus.press("down");
    expect(focus.press("ok")).toEqual({ kind: "episodes" });
    focus.press("down");
    expect(focus.press("ok")).toEqual({ kind: "more" });
  });

  it("saves to the list and takes it back, and the line's word follows", () => {
    const focus = new TitleFocus(film());
    focus.press("down");
    expect(focus.saved).toBe(false);
    expect(focus.press("ok")).toEqual({ kind: "list" });
    expect(focus.saved).toBe(true);
    focus.press("ok");
    expect(focus.saved).toBe(false);
  });

  it("starts from what the server said about the title", () => {
    expect(new TitleFocus(film({ in_list: true })).saved).toBe(true);
    expect(new TitleFocus(film({ rating: "love" })).rating).toBe("love");
  });

  it("sets a rating, and takes it back when the same one is pressed again", () => {
    const focus = new TitleFocus(film());
    focus.press("up");
    expect(focus.press("ok")).toEqual({ kind: "rate", rating: "up" });
    expect(focus.press("ok")).toEqual({ kind: "rate", rating: null });
    focus.press("left");
    expect(focus.press("ok")).toEqual({ kind: "rate", rating: "down" });
  });
});
