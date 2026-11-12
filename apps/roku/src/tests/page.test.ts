// Which rows a page may draw. No SDK, no device: Jest runs on a computer, and Roku says it is what
// it uses internally to test the SDK (https://forum.developer.roku.com/t/rsg-2-0-live-q-a-recap/11817,
// 2026-11-12).
//
// The case that matters is the one a reviewer found on Android on 2026-11-11: the Games tab drew
// whatever the tab before it was showing, because the core never asks for Games and never clears
// what it holds (`apps/android/surfaces/tv/.../browse/Page.kt:11-15`).

import type { Section, Tile } from "../contract";
import { answerIsThisPage, heroOn, pageKey, rowsOn, screenOf } from "../page";

function tile(id: number): Tile {
  return { name: `t${id}`, title: `movie:${id}` };
}

function section(id: string, kind: string, count = 1): Section {
  return {
    id,
    kind,
    title: id,
    tiles: Array.from({ length: count }, (_unused, at) => tile(at)),
  };
}

describe("rowsOn", () => {
  it("draws the answer's own rows when the answer is this page's", () => {
    const answer = [section("a", "row"), section("b", "ranked")];
    expect(rowsOn("home", null, "home", answer, []).map((one) => one.id)).toEqual(["a", "b"]);
  });

  it("draws nothing on games, whatever is held", () => {
    const answer = [section("a", "row"), section("b", "row")];
    expect(rowsOn("games", "games", "home", answer, [])).toEqual([]);
  });

  it("drops a section with no tiles: a row with no titles is an empty line under a name", () => {
    const answer = [section("a", "row", 0), section("b", "row", 2)];
    expect(rowsOn("home", null, "home", answer, []).map((one) => one.id)).toEqual(["b"]);
  });

  it("drops a kind that is not a row", () => {
    const answer = [section("hero", "hero"), section("note", "notice"), section("a", "continue")];
    expect(rowsOn("home", null, "home", answer, []).map((one) => one.id)).toEqual(["a"]);
  });

  it("keeps the rows read past the first twelve only when the answer is this page's", () => {
    const answer = [section("a", "row")];
    const extra = [section("x", "row")];
    expect(rowsOn("films", "films", "films", answer, extra).map((one) => one.id)).toEqual(["a", "x"]);
    // The answer in hand belongs to another page: its own rows draw, the extra ones do not.
    expect(rowsOn("films", "films", "series", answer, extra).map((one) => one.id)).toEqual(["a"]);
  });

  it("draws every kind the surface calls a row, and no other", () => {
    const kinds = ["row", "ranked", "continue", "grid", "hero", "notice", "something_new"];
    const answer = kinds.map((kind) => section(kind, kind));
    expect(rowsOn("home", null, "home", answer, []).map((one) => one.id)).toEqual([
      "row",
      "ranked",
      "continue",
      "grid",
    ]);
  });
});

describe("answerIsThisPage", () => {
  it("lets home and the kids home, which name no screen, use the answer's own name", () => {
    expect(answerIsThisPage("home", null, "home")).toBe(true);
  });

  it("refuses an answer built for another screen", () => {
    expect(answerIsThisPage("films", "films", "series")).toBe(false);
    expect(answerIsThisPage("films", "films", "films")).toBe(true);
  });

  it("never lets games own an answer", () => {
    expect(answerIsThisPage("games", "games", "games")).toBe(false);
    expect(answerIsThisPage("games", null, "games")).toBe(false);
  });
});

describe("heroOn", () => {
  it("takes the first tile of the first hero section", () => {
    const answer = [section("a", "row"), section("h", "hero", 3)];
    expect(heroOn("home", answer)?.title).toBe("movie:0");
  });

  it("has no hero on games, and none when the hero section is empty", () => {
    expect(heroOn("games", [section("h", "hero", 3)])).toBeNull();
    expect(heroOn("home", [section("h", "hero", 0)])).toBeNull();
  });
});

describe("the page's own name and key", () => {
  it("falls back to home when the route names no screen", () => {
    expect(screenOf(undefined)).toBe("home");
    expect(screenOf(null)).toBe("home");
    expect(screenOf("films")).toBe("films");
  });

  it("files a genre page apart from the tab it hangs under", () => {
    expect(pageKey("p1", "genre", "83")).not.toBe(pageKey("p1", "genre", "34399"));
    // And apart from the same page under another profile.
    expect(pageKey("p1", "home", null)).not.toBe(pageKey("p2", "home", null));
  });
});
