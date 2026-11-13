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

/**
 * The plural form a count takes in a language, as CLDR names them: zero, one, two, few, many,
 * other. A mirror of `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt:99-128`,
 * branch for branch.
 *
 * This is not decoration. Four of the eighteen language files carry keys English has not —
 * `ar` 310, `he` 307, `pl` 304, `ru` 304 against `en` 301 (counted 2026-11-12, note 219) — and
 * every extra one is a plural form of `title.length.hours` or `title.season`. A surface that
 * appended a word instead of choosing a form would be wrong in those four languages and nowhere
 * else, which is the kind of wrong nobody sees.
 */
export function pluralOf(n: number, lang: string): string {
  const base = lang.split("-")[0] ?? lang;
  switch (base) {
    case "ar":
      if (n === 0) return "zero";
      if (n === 1) return "one";
      if (n === 2) return "two";
      if (n % 100 >= 3 && n % 100 <= 10) return "few";
      if (n % 100 >= 11 && n % 100 <= 99) return "many";
      return "other";
    case "ru":
    case "uk":
      if (n % 10 === 1 && n % 100 !== 11) return "one";
      if (n % 10 >= 2 && n % 10 <= 4 && !(n % 100 >= 12 && n % 100 <= 14)) return "few";
      return "many";
    case "pl":
      if (n === 1) return "one";
      if (n % 10 >= 2 && n % 10 <= 4 && !(n % 100 >= 12 && n % 100 <= 14)) return "few";
      return "many";
    case "he":
    case "iw":
      if (n === 1) return "one";
      if (n === 2) return "two";
      return "other";
    case "ja":
    case "ko":
    case "zh":
      return "other";
    case "fa":
      return n === 1 ? "one" : "other";
    case "fr":
    case "pt":
    case "hi":
      return n <= 1 ? "one" : "other";
    default:
      return n === 1 ? "one" : "other";
  }
}

/**
 * A counted word: `{base}.{form}` for the count's plural form, else `other`, `many`, then `one`.
 * (`Theme.kt:130-134`)
 *
 * The fallback chain is load-bearing and is why this is not a one-liner: English holds
 * `title.season.one` and `title.season.many` and no `title.season.other`, so two seasons in English
 * reach the word only through the second step of the chain.
 */
export function counted(
  words: Record<string, string>,
  base: string,
  n: number,
  lang: string,
): string {
  const forms = [pluralOf(n, lang), "other", "many", "one"];
  for (const form of forms) {
    const word = words[`${base}.${form}`];
    if (word !== undefined) return fill(word, "n", n);
  }
  return "";
}

/**
 * A number as the interface's language writes it. (`Theme.kt:88-97`)
 *
 * The digit set is a word like any other: `language.digits` carries `latn`, `arab` or `arabext`,
 * so no language is named here.
 */
export function number(value: string | number, digits: string): string {
  const plain = String(value);
  const zero = digits === "arab" ? 0x0660 : digits === "arabext" ? 0x06f0 : 0;
  if (zero === 0) return plain;
  let out = "";
  for (const mark of plain) {
    const at = "0123456789".indexOf(mark);
    out += at < 0 ? mark : String.fromCharCode(zero + at);
  }
  return out;
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
