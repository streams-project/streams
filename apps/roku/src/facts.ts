// The line of facts about a title: genre, year, length, age, in that order on every screen.
//
// A MIRROR of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/parts/Facts.kt:21-48`, which is
// the behaviour spec for every television surface. See the head of `src/tier.ts` for why this
// folder mirrors rules instead of calling the shared core.
//
// The Kotlin already splits this the right way: `facts()` is a pure function returning a list of
// words, and the drawing is a separate composable that puts a dot between them. Only the list is
// mirrored here. The dot is `src/screens/TitlePage.tsx`'s business, and a separator drawn in code
// is a punctuation decision no language would thank us for.
//
// `src/tests/facts.test.ts` holds this against `platform/contract/fixtures/2026-11-05/valid/`,
// the same files the Rust reads: "Fixtures are plain JSON and every language reads the same files"
// (`platform/contract/tests/fixtures.rs:1`).

import type { TitleDetail } from "./contract";
import { counted, number, say } from "./words";

/**
 * How long a film runs, as hours and minutes. (`Facts.kt:21-35`)
 *
 * The two counted words are joined by `title.length.both`, a pattern from the language rather than
 * a space written here, because the order of the two parts is not the same in every language: `ar`
 * holds `{hours} و{minutes}` and `fa` holds `{hours} و {minutes}` (read from `strings/` on
 * 2026-11-13). The Kotlin's own fallback, `"{hours} {minutes}"`, is kept for a language file that
 * has not got the key.
 */
export function lengthWord(words: Record<string, string>, minutes: number, lang: string): string {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours > 0 && rest > 0) {
    // A plain replace, not `fill`: the two parts are already isolated by `counted`, which ends in
    // `fill("n", n)`, and isolating them again would wrap each in a second pair of U+2068 and
    // U+2069. `Facts.kt:27-29` uses `String.replace` here for the same reason.
    const pattern = words["title.length.both"] ?? "{hours} {minutes}";
    return pattern
      .replace("{hours}", counted(words, "title.length.hours", hours, lang))
      .replace("{minutes}", counted(words, "title.length.minutes", rest, lang));
  }
  if (hours > 0) return counted(words, "title.length.hours", hours, lang);
  return counted(words, "title.length.minutes", rest, lang);
}

/**
 * The facts, in one order on every screen: genre, year, length, age. (`Facts.kt:37-48`)
 *
 * Four rules, each of which can be wrong on its own and is tested on its own:
 *
 *   - the **first** genre only, never the list;
 *   - the year as this language writes its digits;
 *   - a season count **or** a running time, never both: a series carries seasons and takes that
 *     branch even when it also carries a runtime;
 *   - the age mark translated through `age.{label}`, falling back to the label in this language's
 *     digits, because a certificate like `12` is a number in every writing system.
 *
 * A fact with nothing behind it is left out rather than drawn empty, so a title that carries only
 * a name returns an empty list and the line disappears.
 */
export function facts(
  detail: TitleDetail,
  words: Record<string, string>,
  lang: string,
): string[] {
  const digits = say(words, "language.digits");
  const line: string[] = [];
  const genre = (detail.genres ?? [])[0];
  if (genre !== undefined) line.push(genre);
  if (detail.year !== undefined) line.push(number(detail.year, digits));
  const seasons = (detail.seasons ?? []).length;
  const minutes = detail.runtime_min;
  if (seasons > 0) {
    line.push(counted(words, "title.season", seasons, lang));
  } else if (minutes !== undefined && minutes > 0) {
    line.push(lengthWord(words, minutes, lang));
  }
  const label = detail.age?.label;
  if (label !== undefined) {
    line.push(words[`age.${label}`] ?? number(label, digits));
  }
  return line;
}
