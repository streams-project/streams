// The words, by key. No English appears in this folder.
//
// `strings/en.json` holds 301 keys (counted 2026-11-12 with
// `python3 -c "import json;print(len(json.load(open('strings/en.json'))))"`) and the server hands a
// client one language's worth at `/v1/strings/{lang}` (`platform/contract/src/api.rs:183`). A
// screen asks for a key and draws whatever came back; a key with no word behind it draws nothing,
// which is how a missing translation looks on every other surface too.
//
// The keys below are the ones the screens in this folder ask for. Each was checked against
// `strings/en.json` on 2026-11-12.

/** A word by its key, or an empty string. The television surface reads `words[key].orEmpty()`. */
export function say(words: Record<string, string>, key: string): string {
  return words[key] ?? "";
}

/**
 * A word's `{name}` place filled with a value, isolated (U+2068, U+2069) so a number, a title or an
 * address keeps its own order inside a line of either direction.
 * (`apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt:62-67`)
 */
export function fill(word: string, name: string, value: string | number): string {
  return word.replace(`{${name}}`, `⁨${value}⁩`);
}

/** The keys this surface asks for. */
export const KEY = {
  navSearch: "nav.search",
  play: "common.play",
  moreInfo: "common.more_info",
  gamesSoon: "games.soon",
  mineEmpty: "mine.empty",
  titleResume: "title.resume",
  titleContinueEpisode: "title.continue_episode",
  titlePlayEpisode: "title.play_episode",
  titleEpisodesMore: "title.episodes_more",
  titleMoreLike: "title.more_like",
  listAdd: "list.add",
  listAdded: "list.added",
  badgeNew: "badge.new",
  badgeNewEpisode: "badge.new_episode",
  badgeRanked: "badge.ranked",
  badgeTop10: "badge.top_10",
  /** How the language writes its digits, and whether it wants larger text. (`Television.kt:53-56`) */
  digits: "language.digits",
  textScale: "language.text_scale",
};

/**
 * Two things a person hears one after the other, as one label: a title and the row it sits in. The
 * break is a line end rather than a word or a comma, so no language's punctuation or word order is
 * decided in the code. (`apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Spoken.kt:74-80`,
 * used at `Browse.kt:264`)
 */
export function sayBoth(first: string | null | undefined, second: string | null | undefined): string {
  return [first, second].filter((word) => word !== null && word !== undefined && word !== "").join("\n");
}
