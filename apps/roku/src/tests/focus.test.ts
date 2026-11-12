// The remote on a page of rows, pressed key by key. Needs no SDK and no Roku.
//
// Every case here is a rule from `notes/187-tv-navigation-per-page-2026-11-08.md`, which is the
// per-page key map, or a branch of `apps/android/surfaces/tv/.../browse/Browse.kt:107-151`.

import type { Section, Tile } from "../contract";
import { PageFocus, SEARCH_TAB, WHO_TAB } from "../focus";

function tiles(count: number, from = 0): Tile[] {
  return Array.from({ length: count }, (_unused, at) => ({
    name: `t${from + at}`,
    title: `movie:${from + at}` as const,
  }));
}

function rows(count: number, each = 4): Section[] {
  return Array.from({ length: count }, (_unused, at) => ({
    id: `r${at}`,
    kind: "row",
    title: `row ${at}`,
    tiles: tiles(each),
  }));
}

describe("where a page opens", () => {
  it("opens on the hero when it has one", () => {
    expect(new PageFocus(rows(3), 4, 0, true).where).toBe("hero");
  });

  it("opens on the first row when it has no hero", () => {
    expect(new PageFocus(rows(3), 4, 0, false).where).toBe("row");
  });

  it("opens on the bar when it has neither", () => {
    expect(new PageFocus([], 4, 0, false).where).toBe("top");
  });
});

describe("moving", () => {
  it("walks down the rows and stops at the last one", () => {
    const focus = new PageFocus(rows(2), 4, 0, false);
    expect(focus.row).toBe(0);
    expect(focus.press("down")).toBe(true);
    expect(focus.row).toBe(1);
    // Nothing below: the key is not used, so the page may do something else with it.
    expect(focus.press("down")).toBe(false);
    expect(focus.row).toBe(1);
  });

  it("goes up from the first row to the hero, then to the bar, then stops", () => {
    const focus = new PageFocus(rows(2), 4, 0, true);
    focus.press("down");
    focus.press("down");
    expect(focus.where).toBe("row");
    expect(focus.press("up")).toBe(true);
    expect(focus.where).toBe("hero");
    expect(focus.press("up")).toBe(true);
    expect(focus.where).toBe("top");
    expect(focus.press("up")).toBe(false);
  });

  it("runs the bar leftwards into search and then the profile, and back again", () => {
    const focus = new PageFocus([], 4, 0, false);
    expect(focus.tab).toBe(0);
    focus.press("left");
    expect(focus.tab).toBe(SEARCH_TAB);
    focus.press("left");
    expect(focus.tab).toBe(WHO_TAB);
    expect(focus.press("left")).toBe(false);
    focus.press("right");
    expect(focus.tab).toBe(SEARCH_TAB);
    focus.press("right");
    expect(focus.tab).toBe(0);
  });

  it("stops at the last tab", () => {
    const focus = new PageFocus([], 2, 0, false);
    expect(focus.press("right")).toBe(true);
    expect(focus.tab).toBe(1);
    expect(focus.press("right")).toBe(false);
  });

  it("moves between the hero's two buttons", () => {
    const focus = new PageFocus(rows(1), 4, 0, true);
    focus.press("down");
    expect(focus.where).toBe("hero");
    focus.press("right");
    expect(focus.button).toBe(1);
    focus.press("left");
    expect(focus.button).toBe(0);
  });
});

describe("a row keeps its place", () => {
  it("comes back to the tile the viewer left it on", () => {
    const focus = new PageFocus(rows(2), 4, 0, false);
    focus.press("right");
    focus.press("right");
    expect(focus.column).toBe(2);
    focus.press("down");
    expect(focus.column).toBe(0);
    focus.press("up");
    expect(focus.column).toBe(2);
  });

  it("follows the title when a fresh answer reorders a row", () => {
    const first = rows(1);
    const focus = new PageFocus(first, 4, 0, false);
    focus.press("right");
    focus.press("right");
    expect(focus.column).toBe(2);
    // The same row, with the same id, rebuilt with two titles inserted before the one in hand.
    focus.rows = [{ id: "r0", kind: "row", title: "row 0", tiles: tiles(2, 90).concat(tiles(4)) }];
    expect(focus.column).toBe(4);
  });

  it("falls back to the kept place when the title is gone altogether", () => {
    const focus = new PageFocus(rows(1), 4, 0, false);
    focus.press("right");
    expect(focus.column).toBe(1);
    focus.rows = [{ id: "r0", kind: "row", title: "row 0", tiles: tiles(3, 70) }];
    expect(focus.column).toBe(1);
  });

  it("does not run off the end of a row that shrank", () => {
    const focus = new PageFocus(rows(1), 4, 0, false);
    focus.press("right");
    focus.press("right");
    focus.press("right");
    expect(focus.column).toBe(3);
    focus.rows = [{ id: "r0", kind: "row", title: "row 0", tiles: tiles(2, 70) }];
    expect(focus.column).toBeLessThanOrEqual(1);
  });
});

describe("Back walks out one level at a time", () => {
  it("goes row, hero, bar, and then refuses, so the page decides", () => {
    const focus = new PageFocus(rows(2), 4, 0, true);
    focus.press("down");
    focus.press("down");
    expect(focus.where).toBe("row");
    expect(focus.press("back")).toBe(true);
    expect(focus.where).toBe("hero");
    expect(focus.press("back")).toBe(true);
    expect(focus.where).toBe("top");
    expect(focus.press("back")).toBe(false);
  });

  it("brings a wandering focus home to the page's own tab before leaving", () => {
    // This page is the third tab, and the viewer has walked the bar without choosing.
    const focus = new PageFocus([], 5, 2, false);
    expect(focus.tab).toBe(2);
    focus.press("right");
    expect(focus.tab).toBe(3);
    expect(focus.press("back")).toBe(true);
    expect(focus.tab).toBe(2);
    expect(focus.press("back")).toBe(false);
  });
});

describe("OK", () => {
  it("is never used by the focus itself: the page decides what it means", () => {
    expect(new PageFocus(rows(1), 4, 0, false).press("ok")).toBe(false);
  });

  it("opens the title the remote is on", () => {
    const focus = new PageFocus(rows(2), 4, 0, false);
    focus.press("right");
    expect(focus.choose(null)).toEqual({ kind: "title", title: "movie:1" });
  });

  it("opens the profile menu and search from the bar", () => {
    const focus = new PageFocus([], 4, 0, false);
    focus.press("left");
    expect(focus.choose(null)).toEqual({ kind: "search" });
    focus.press("left");
    expect(focus.choose(null)).toEqual({ kind: "who" });
  });

  it("plays a film from the hero and opens a series, because a series has an episode to pick", () => {
    const film = new PageFocus(rows(1), 4, 0, true);
    expect(film.choose({ name: "f", title: "movie:7" })).toEqual({ kind: "watch", title: "movie:7" });
    expect(film.choose({ name: "s", title: "tv:7" })).toEqual({ kind: "title", title: "tv:7" });
    // The second button is More Info, which opens the title whatever its kind.
    film.press("right");
    expect(film.choose({ name: "f", title: "movie:7" })).toEqual({ kind: "title", title: "movie:7" });
  });
});

describe("reading the next rows", () => {
  it("asks at once on a page that arrived with no rows at all", () => {
    expect(new PageFocus([], 4, 0, false).withinReach(2)).toBe(true);
  });

  it("waits until the focus is within reach of the last row it holds", () => {
    const focus = new PageFocus(rows(12), 4, 0, false);
    expect(focus.withinReach(2)).toBe(false);
    for (let at = 0; at < 9; at += 1) focus.press("down");
    expect(focus.row).toBe(9);
    expect(focus.withinReach(2)).toBe(true);
  });

  it("does not ask while the remote is on the bar or the hero", () => {
    const focus = new PageFocus(rows(1), 4, 0, true);
    expect(focus.where).toBe("hero");
    expect(focus.withinReach(2)).toBe(false);
  });
});
