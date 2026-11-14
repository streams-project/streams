// What an address opens, held to the fixture corpus and to the Rust it mirrors.
//
// `src/route.ts` is the largest mirror in this folder, and a second implementation checked only
// against its own expectations is worse than none, because it keeps passing while the rule it
// mirrors moves underneath it. So this suite holds it to two things outside this folder:
//
//   1. `platform/contract/fixtures/2026-11-05/valid/View.route.json` and `View.route_activate.json`
//      — the targets the contract itself carries, read as the same bytes the Rust reads
//      (`platform/contract/tests/fixtures.rs`);
//   2. `platform/core/src/routes.rs`, read as **text**. A fixture carries data and not rules, so
//      it does not notice the day somebody adds an address to the Rust. The two reads below do.
//      It is brittle to a rewrite of that file, which is the price; the alternative was no guard.
//      (The same technique, and the same bargain, as `src/tests/tier.test.ts`.)

import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { RouteTarget } from "../contract";
import { ADDRESS } from "../contract";
import { decode, episodeOf, queryValue, split, targetOf, titleOf } from "../route";

const ROOT = join(__dirname, "..", "..", "..", "..");
const CORPUS = join(ROOT, "platform", "contract", "fixtures", "2026-11-05");
const RULES = readFileSync(join(ROOT, "platform", "core", "src", "routes.rs"), "utf8");

function fixture(name: string): unknown {
  return JSON.parse(readFileSync(join(CORPUS, "valid", name), "utf8")) as unknown;
}

/** The body of one Rust function, by name. */
function body(name: string): string {
  const at = RULES.indexOf("fn " + name + "(");
  if (at < 0) return "";
  const from = RULES.indexOf("{", at);
  let depth = 0;
  for (let scan = from; scan < RULES.length; scan += 1) {
    const mark = RULES[scan];
    if (mark === "{") depth += 1;
    if (mark === "}") {
      depth -= 1;
      if (depth === 0) return RULES.slice(from, scan);
    }
  }
  return "";
}

/** Every plain string literal in a piece of Rust, deduplicated and sorted. */
function words(rust: string, prefix: string): string[] {
  const found = rust.match(new RegExp(prefix + '"[^"]*"', "g")) ?? [];
  const bare = found.map((one) => one.slice(prefix.length + 1, -1));
  return bare.filter((one, at) => bare.indexOf(one) === at).sort();
}

describe("the Rust this mirrors has not moved", () => {
  it("is where this suite thinks it is", () => {
    expect(RULES).toContain("fn place(");
    expect(RULES).toContain("fn member_target(");
    expect(body("place")).toContain("[\"title\", slug]");
  });

  it("reads the same path words as `place` does", () => {
    // Every literal `place` matches a path part against. A word added to the Rust and not to
    // `src/route.ts` fails here, naming the file to read.
    expect(words(body("place"), "")).toEqual([
      "",
      "34399",
      "83",
      "activate",
      "browse",
      "code",
      "coming-soon",
      "genre",
      "kids",
      "latest",
      "login",
      "my-list",
      "original-audio",
      "profiles",
      "settings",
      "title",
      "tv2",
      "viewed",
      "watch",
    ]);
  });

  it("reads the same two names for a season and an episode as `episode_of` does", () => {
    // In the Rust these are a function of their own, which is why they are not in `place` above.
    expect(words(body("episode_of"), "")).toEqual(["e", "episode", "s", "season"]);
  });

  it("reads the same one-word screens as `screen_named` does", () => {
    expect(words(body("screen_named"), "b")).toEqual([
      "clips",
      "games",
      "help",
      "mine",
      "notifications",
      "search",
      "settings",
    ]);
  });
});

describe("the corpus holds the targets this mirror makes", () => {
  it("View.route is an episode of a series, being watched", () => {
    const held = (fixture("View.route.json") as { route: RouteTarget }).route;
    expect(held.target).toBe("watch");
    expect(held.title).toBe("tv:1001");
    expect(held.episode).toEqual({ season: 1, episode: 3 });
  });

  it("is what the long watch address opens", () => {
    const held = (fixture("View.route.json") as { route: RouteTarget }).route;
    expect(targetOf("/watch/tv/1001/1/3")).toEqual(held);
  });

  it("is what the same thing asked for in the query opens", () => {
    const held = (fixture("View.route.json") as { route: RouteTarget }).route;
    expect(targetOf("/watch/tv/1001?season=1&episode=3")).toEqual(held);
  });

  it("is what the short names for them open, which is the half a reader would miss", () => {
    const held = (fixture("View.route.json") as { route: RouteTarget }).route;
    expect(targetOf("/watch/tv/1001?s=1&e=3")).toEqual(held);
  });

  it("View.route_activate is a code that stood in the address", () => {
    const held = (fixture("View.route_activate.json") as { route: RouteTarget }).route;
    expect(targetOf("/activate?code=BCDFGHJK")).toEqual(held);
    expect(targetOf("/tv2?code=BCDFGHJK")).toEqual(held);
  });
});

describe("every address in the table opens something", () => {
  it("and nothing in it is not found", () => {
    const lost: string[] = [];
    for (const name of Object.keys(ADDRESS)) {
      const address = ADDRESS[name as keyof typeof ADDRESS];
      // The three that are a prefix and not an address are given the rest of themselves.
      const whole =
        name === "GENRE" ? address + "77" : name === "TITLE" ? address + "movie-1001"
        : name === "WATCH" ? address + "movie/1001" : address;
      if (targetOf(whole).target === "not_found") lost.push(name);
    }
    expect(lost).toEqual([]);
  });

  it("opens the tabs at the screens the server builds", () => {
    expect(targetOf(ADDRESS.HOME)).toEqual({ target: "home" });
    expect(targetOf(ADDRESS.KIDS)).toEqual({ target: "kids_home" });
    expect(targetOf(ADDRESS.SERIES)).toEqual({ target: "screen", screen: "series" });
    expect(targetOf(ADDRESS.FILMS)).toEqual({ target: "screen", screen: "films" });
    expect(targetOf(ADDRESS.NEW)).toEqual({ target: "screen", screen: "new" });
    expect(targetOf(ADDRESS.MY_LIST)).toEqual({ target: "screen", screen: "my-list" });
    expect(targetOf(ADDRESS.LANGUAGE)).toEqual({ target: "screen", screen: "language" });
    expect(targetOf(ADDRESS.ACTIVITY)).toEqual({ target: "screen", screen: "activity" });
    expect(targetOf(ADDRESS.MINE)).toEqual({ target: "screen", screen: "mine" });
  });

  it("keeps the two genre ids that are screens of their own apart from every other genre", () => {
    // 83 and 34399 are series and films, and they are the one case where the same shape of
    // address means two different pages (`routes.rs:149-154`).
    expect(targetOf("/browse/genre/83")).toEqual({ target: "screen", screen: "series" });
    expect(targetOf("/browse/genre/77")).toEqual({ target: "screen", screen: "genre", genre: "77" });
  });

  it("opens a title by the kind in its address, because film and series ids overlap", () => {
    expect(targetOf("/title/movie-1001")).toEqual({ target: "title", title: "movie:1001" });
    expect(targetOf("/title/tv-1001")).toEqual({ target: "title", title: "tv:1001" });
  });
});

describe("an address that is not one opens nothing", () => {
  it("rather than the nearest page", () => {
    expect(targetOf("/browse/nonsense").target).toBe("not_found");
    expect(targetOf("/nonsense").target).toBe("not_found");
    expect(targetOf("/title/").target).toBe("not_found");
    expect(targetOf("/title/1001").target).toBe("not_found");
    expect(targetOf("/watch/tv").target).toBe("not_found");
  });

  it("not even when the address names something every object in this language has", () => {
    // Found by reading, not by a failing test, and it is the kind of defect this language has and
    // Rust does not: the one-word screens are a plain object, so `ONE_WORD["constructor"]` is a
    // function rather than `undefined`. A presence test would have opened a screen whose name was
    // the `Object` constructor.
    for (const probe of ["constructor", "toString", "valueOf", "hasOwnProperty", "__proto__"]) {
      const opened = targetOf("/" + probe);
      expect(opened.target).toBe("not_found");
      expect(opened.screen).toBeUndefined();
    }
  });

  it("and an episode of nothing is not an episode", () => {
    // `?season=&episode=` is the case a `Number("")` reading opens as season 0 episode 0.
    expect(targetOf("/watch/tv/1001?season=&episode=").episode).toBeUndefined();
    expect(targetOf("/watch/tv/1001?season=one&episode=three").episode).toBeUndefined();
    expect(targetOf("/watch/tv/1001/x/3").target).toBe("not_found");
  });
});

describe("the pieces an address is read with", () => {
  it("drops the fragment, the query and a trailing slash", () => {
    expect(split("/browse/?x=1#here")).toEqual({ path: "/browse", query: "x=1" });
    expect(split("/")).toEqual({ path: "/", query: "" });
    expect(split("")).toEqual({ path: "/", query: "" });
  });

  it("reads the first of the names it is given", () => {
    expect(queryValue("a=1&b=2", ["b", "a"])).toBe("1");
    expect(queryValue("a=1", ["c"])).toBeNull();
    expect(queryValue("bare", ["bare"])).toBeNull();
  });

  it("answers nothing for an escape that is not one, rather than throwing", () => {
    expect(decode("a%20b")).toBe("a b");
    expect(decode("%zz")).toBeNull();
  });

  it("reads a title out of its address form and refuses a bare id", () => {
    expect(titleOf("tv-1399")).toBe("tv:1399");
    expect(titleOf("1399")).toBeNull();
    expect(titleOf("tv-")).toBeNull();
  });

  it("wants both halves of an episode or neither", () => {
    expect(episodeOf("season=1&episode=3")).toEqual({ season: 1, episode: 3 });
    expect(episodeOf("season=1")).toBeNull();
    expect(episodeOf("")).toBeNull();
  });
});

describe("a title waiting in the address goes along", () => {
  it("onto the page that was opened", () => {
    expect(targetOf("/browse?title=movie%3A1001")).toEqual({
      target: "home",
      title: "movie:1001",
    });
    expect(targetOf("/kids?title=movie%3A1001")).toEqual({
      target: "kids_home",
      title: "movie:1001",
    });
  });

  it("onto a screen page too, which is three of the four places it can land", () => {
    // Home, the kids home and every screen each carry it (`routes.rs:194-207`), and until this
    // case existed a mutation that dropped it from the screen branch alone survived: the two
    // cases above both go through the home branches and saw nothing.
    expect(targetOf("/latest?title=movie%3A1001")).toEqual({
      target: "screen",
      screen: "new",
      title: "movie:1001",
    });
    expect(targetOf("/browse/genre/77?title=tv%3A1001")).toEqual({
      target: "screen",
      screen: "genre",
      genre: "77",
      title: "tv:1001",
    });
  });

  it("and a bare id is left behind, because it has no kind", () => {
    expect(targetOf("/browse?title=1001")).toEqual({ target: "home" });
    expect(targetOf("/latest?title=1001")).toEqual({ target: "screen", screen: "new" });
  });

  it("but details asked for over a page open the title instead", () => {
    // `jbv` is the one query that outranks the path (`routes.rs:134-137`).
    expect(targetOf("/browse?jbv=tv-1001")).toEqual({ target: "title", title: "tv:1001" });
  });
});
