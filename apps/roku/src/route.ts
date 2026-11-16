// What an address opens, for a family that cannot run the core.
//
// A MIRROR of `platform/core/src/routes.rs`, and the largest one this folder has. See the head of
// `src/tier.ts` for why this folder mirrors rules at all, and section 2 of the README for the
// question underneath it: the shared Rust core does not run on a Roku, `POST /v1/core/step` is the
// route that would let it run on the server, and `server/crates/server-api/tests/routes.rs:13`
// still lists that route as unbuilt. Until it answers, an address has to become a target here or
// no key press on this surface goes anywhere.
//
// WHAT IS MIRRORED: `place` (`routes.rs:132-172`), which reads an address into one of nine places,
// and `member_target` (`routes.rs:190-222`), which turns a place into a `RouteTarget`. Those two
// are the whole of what a signed-in viewer's address does.
//
// WHAT IS DELIBERATELY NOT, each with what it would need:
//
//   * **the moved table** (`decide_from` step 1). It is data from `Hello.moved`, and no screen in
//     this folder reads `Hello` yet. A mirror of it would be a rule with no caller.
//   * **who may open it** (`decide_from` step 3): the visitor rules, `accounts_required`, and the
//     `Target::Wait` that stands for "the answer is not in yet". This surface has no sign-in
//     screen, no profile gate and no door, so every one of those branches would decide a screen
//     that is not here. `src/host.ts` asks the server for the page and draws what comes back; a
//     page a profile may not have answers 403 and the screen stays empty, which is the honest
//     behaviour for a surface with nothing to send the viewer to.
//   * **`next`** (`safe_next`, `routes.rs:226-238`). It exists to carry a viewer back across a
//     sign-in this family cannot perform.
//
// So this file answers for a member. `src/tests/route.test.ts` holds it to the fixture corpus and
// reads `routes.rs` as text, the way `src/tests/tier.test.ts` reads `tier.rs`: the day somebody
// adds an address to the Rust, this folder goes red naming that file.

import type { EpisodeRef, RouteTarget, ScreenName, Target, TitleKey } from "./contract";

/** A path and its query, with the fragment, the trailing slash and the empty path handled. (`routes.rs:69-75`) */
export function split(address: string): { path: string; query: string } {
  const noFragment = address.split("#")[0] ?? "";
  const cut = noFragment.indexOf("?");
  const path = cut < 0 ? noFragment : noFragment.slice(0, cut);
  const query = cut < 0 ? "" : noFragment.slice(cut + 1);
  const trimmed = path.replace(/\/+$/, "");
  return { path: trimmed === "" ? "/" : trimmed, query };
}

/** The first value of any of these names in a query. (`routes.rs:77-83`) */
export function queryValue(query: string, names: string[]): string | null {
  for (const pair of query.split("&")) {
    const cut = pair.indexOf("=");
    if (cut < 0) continue;
    if (names.indexOf(pair.slice(0, cut)) >= 0) return pair.slice(cut + 1);
  }
  return null;
}

/**
 * Percent decoding, and nothing when the text is not valid. (`routes.rs:91-106`)
 *
 * `decodeURIComponent` throws on a broken escape and on bytes that are not UTF-8, which is the
 * same answer the Rust gives by handing back `None`: a malformed address decides nothing.
 */
export function decode(text: string): string | null {
  try {
    return decodeURIComponent(text);
  } catch {
    return null;
  }
}

/**
 * A title key, `kind:id`. The contract's own `TitleKey` is `movie:1001` or `tv:1001`
 * (`fixtures/2026-11-05/valid/TitleKey.film.json`), and a key with no kind or no id is not one.
 * (`routes.rs:108-110`, `TitleKey::parse`)
 */
export function titleKey(kind: string, id: string): TitleKey | null {
  if (kind === "" || id === "") return null;
  if (!/^[a-z]+$/.test(kind)) return null;
  if (!/^[A-Za-z0-9_-]+$/.test(id)) return null;
  return `${kind}:${id}` as TitleKey;
}

/** A title from its address form, `tv-1399`. (`routes.rs:174-178`) */
export function titleOf(slug: string): TitleKey | null {
  const cut = slug.indexOf("-");
  if (cut < 0) return null;
  return titleKey(slug.slice(0, cut), slug.slice(cut + 1));
}

/** The screens that live at a one-word address. (`routes.rs:112-124`) */
const ONE_WORD: Record<string, ScreenName> = {
  mine: "mine",
  search: "search",
  notifications: "notifications",
  games: "games",
  clips: "clips",
  help: "help",
  settings: "settings",
};

/** Season and episode of a watch address, both required. (`routes.rs:126-130`) */
export function episodeOf(query: string): EpisodeRef | null {
  const said = queryValue(query, ["season", "s"]);
  const toldEpisode = queryValue(query, ["episode", "e"]);
  if (said === null || toldEpisode === null) return null;
  return whole(said, toldEpisode);
}

/**
 * A season and episode pair from two pieces of text, or nothing when either is not a whole number.
 *
 * The emptiness check is not decoration: `Number("")` is 0 and `Number(" 2 ")` is 2, where the
 * Rust `"".parse::<u32>()` and `" 2 ".parse::<u32>()` both fail. Without it `?season=&episode=`
 * would open episode 0 of season 0, which is an episode no series has.
 */
function whole(first: string, second: string): EpisodeRef | null {
  if (!/^[0-9]+$/.test(first) || !/^[0-9]+$/.test(second)) return null;
  return { season: Number(first), episode: Number(second) };
}

/** A title waiting in the address goes along. A bare id has no kind and is left behind. (`routes.rs:184-188`) */
function pendingTitle(query: string): TitleKey | null {
  const held = queryValue(query, ["title"]);
  if (held === null) return null;
  const plain = decode(held);
  if (plain === null) return null;
  const cut = plain.indexOf(":");
  if (cut < 0) return null;
  return titleKey(plain.slice(0, cut), plain.slice(cut + 1));
}

/** A target with nothing else set. (`routes.rs:56-66`) */
export function target(name: Target): RouteTarget {
  return { target: name };
}

/**
 * What an address opens for a member. `not_found` for an address the table has no row for, which
 * is what `routes.rs:258` answers and what a screen draws nothing for.
 *
 * The order inside is the Rust's own: details asked for over any page win, then the path table.
 */
export function targetOf(address: string): RouteTarget {
  const { path, query } = split(address);
  // Details asked for over any page. A device has no page behind them, so the title opens.
  // (`routes.rs:134-137`, and `address.rs:34` for the query's name)
  const over = queryValue(query, ["jbv"]);
  if (over !== null) {
    const held = titleOf(over);
    return held === null ? target("not_found") : { target: "title", title: held };
  }
  const parts = path.replace(/^\/+/, "").split("/");
  const onScreen = (name: ScreenName, genre?: string): RouteTarget => {
    const route: RouteTarget = { target: "screen", screen: name };
    if (genre !== undefined) route.genre = genre;
    const waiting = pendingTitle(query);
    if (waiting !== null) route.title = waiting;
    return route;
  };
  const one = parts[0] ?? "";
  if (parts.length === 1) {
    if (one === "") return target("start");
    if (one === "login") return target("sign_in");
    if (one === "profiles") return target("profiles");
    if (one === "browse") {
      const route: RouteTarget = { target: "home" };
      const waiting = pendingTitle(query);
      if (waiting !== null) route.title = waiting;
      return route;
    }
    if (one === "kids") {
      const route: RouteTarget = { target: "kids_home" };
      const waiting = pendingTitle(query);
      if (waiting !== null) route.title = waiting;
      return route;
    }
    if (one === "activate" || one === "tv2") {
      const code = queryValue(query, ["code"]);
      const plain = code === null ? null : decode(code);
      return plain === null ? target("activate") : { target: "activate", code: plain };
    }
    if (one === "latest") return onScreen("new");
    // Read as a value and not as a presence. `ONE_WORD` is a plain object, so `/constructor`,
    // `/toString` and `/valueOf` all find a function off its prototype, and an `=== undefined`
    // test would hand that function on as the name of a screen. The Rust cannot have this
    // defect - `screen_named` is a `match` on bytes - and neither `strict` nor
    // `noUncheckedIndexedAccess` can see it, because the type says `ScreenName | undefined` and
    // is right about every value anybody wrote.
    const named = ONE_WORD[one];
    return typeof named === "string" ? onScreen(named) : target("not_found");
  }
  if (parts.length === 2 && one === "browse") {
    const second = parts[1] ?? "";
    if (second === "coming-soon") return onScreen("new");
    if (second === "my-list") return onScreen("my-list");
    if (second === "original-audio") return onScreen("language");
    return target("not_found");
  }
  if (parts.length === 2 && one === "settings" && parts[1] === "viewed") return onScreen("activity");
  if (parts.length === 2 && one === "title") {
    const held = titleOf(parts[1] ?? "");
    return held === null ? target("not_found") : { target: "title", title: held };
  }
  if (parts.length === 3 && one === "browse" && parts[1] === "genre") {
    const id = parts[2] ?? "";
    if (id === "") return target("not_found");
    if (id === "83") return onScreen("series");
    if (id === "34399") return onScreen("films");
    return onScreen("genre", id);
  }
  if (one === "watch" && (parts.length === 3 || parts.length === 5)) {
    const held = titleKey(parts[1] ?? "", parts[2] ?? "");
    if (held === null) return target("not_found");
    const route: RouteTarget = { target: "watch", title: held };
    if (parts.length === 5) {
      const pair = whole(parts[3] ?? "", parts[4] ?? "");
      if (pair === null) return target("not_found");
      route.episode = pair;
    } else {
      const pair = episodeOf(query);
      if (pair !== null) route.episode = pair;
    }
    return route;
  }
  return target("not_found");
}
