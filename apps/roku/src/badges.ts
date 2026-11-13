// What a mark on a tile says.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/parts/Badge.kt:23-30`, and of the
// reason its Views twin states at `tv/src/main/java/streams/tv/parts/Badges.java:17-27`: the view
// that draws badges is handed words and never a `Badge`, so reading the contract happens once, in
// one place, and a surface cannot end up drawing a badge whose words came from the page before.
//
// Two kinds carry a word and the rest carry none. `BadgeKind` ends in `(string & {})` so that a
// kind this build does not know reads as a plain string instead of failing
// (`platform/contract/README.md:41`), which is exactly the case the empty answer is for: a tile
// marked with something this build has never heard of draws no pill rather than an empty one.
//
// `src/tests/badges.test.ts` holds this against the tiles in
// `platform/contract/fixtures/2026-11-05/`, the corpus `platform/contract/tests/fixtures.rs` reads,
// and against the words in `strings/`.

import type { Badge } from "./contract";
import { KEY, fill, say } from "./words";

/**
 * The word this badge carries, or nothing at all.
 *
 * A ranked badge with no rank behind it answers nothing rather than a word with a hole in it:
 * `Badge.kt:26-27` returns early for exactly that, and `rank` is optional on the wire.
 */
export function badgeWord(badge: Badge, words: Record<string, string>): string {
  if (badge.kind === "new") return say(words, KEY.badgeNew);
  if (badge.kind === "ranked") {
    const rank = badge.rank;
    if (rank === undefined) return "";
    return fill(say(words, KEY.badgeRanked), "rank", rank);
  }
  return "";
}

/**
 * The words a tile's badges carry, in the order they are drawn, with the wordless ones left out.
 *
 * A tile with no badges, or with none that carry a word, gives an empty list, and the part that
 * draws them draws nothing at all: `Badge.kt:34-35` returns before it opens a row, so an empty
 * badge list leaves no gap on the tile rather than an empty plate.
 */
export function badgeWords(
  badges: Badge[] | undefined,
  words: Record<string, string>,
): string[] {
  return (badges ?? []).map((badge) => badgeWord(badge, words)).filter((word) => word !== "");
}
