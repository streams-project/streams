// A title's own page: which lines its menu has, what each key does, and what pressing a line means.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/title/TitlePage.kt:52-171` and the
// three reactions of `title/Reactions.kt:19-39`. Apart from the drawing, so it can be checked
// without a screen: `src/tests/title.test.ts`.
//
// Note 187's rules for this page (`notes/187-tv-navigation-per-page-2026-11-08.md:35`): Left on the
// menu goes back, the line opened is kept when coming back, and Down at the last line stays.

import type { Rating, TitleDetail } from "./contract";
import type { Word } from "./keys";
import { KEY, fill, say } from "./words";

/** The two places the remote can be on a title's page. (`TitlePage.kt:52`) */
export type Part = "reactions" | "menu";

/** The lines the menu can have, in the order they are drawn. (`TitlePage.kt:54`) */
export type Line = "play" | "episodes" | "more" | "list";

/**
 * The three ways to rate a title, in the order the row draws them. The words are keys: three faces
 * and no words at all is the clearest case on this surface of a control a person cannot hear.
 * (`Reactions.kt:22-26,34-39`)
 */
export const FACES: { rating: Rating; word: string }[] = [
  { rating: "down", word: "player.down" },
  { rating: "up", word: "player.up" },
  { rating: "love", word: "player.love" },
];

export const REACTION_COUNT = FACES.length;

/** What a line will do, for the caller to carry out. */
export type Deed =
  | { kind: "play" }
  | { kind: "episodes" }
  | { kind: "more" }
  | { kind: "list" }
  | { kind: "rate"; rating: Rating | null }
  | { kind: "back" }
  | { kind: "nothing" };

/** A series, by its key. (`TitlePage.kt:56`) */
export function isSeries(data: TitleDetail): boolean {
  return data.title.startsWith("tv:");
}

/**
 * The lines this title's menu has. A film has no episodes; a title with nothing like it has no
 * grid. (`TitlePage.kt:101-106`)
 */
export function linesOf(data: TitleDetail): Line[] {
  const lines: Line[] = ["play"];
  if (data.seasons !== undefined && data.seasons.length > 0) lines.push("episodes");
  if (data.more_like_this !== undefined && data.more_like_this.length > 0) lines.push("more");
  lines.push("list");
  return lines;
}

/**
 * The first line of the page says what pressing it will do: play, resume, or carry on with an
 * episode by its number. (`TitlePage.kt:72-83`)
 */
export function playWord(data: TitleDetail, words: Record<string, string>): string {
  const resume = data.resume;
  const episode = resume?.episode;
  let word: string;
  if (!isSeries(data) && resume !== undefined) word = say(words, KEY.titleResume);
  else if (!isSeries(data)) word = say(words, KEY.play);
  else if (episode !== undefined && (resume?.position_s ?? 0) > 0) word = say(words, KEY.titleContinueEpisode);
  else word = say(words, KEY.titlePlayEpisode);
  return fill(word, "n", episode?.episode ?? 1);
}

/** What each line of the menu is called, and whether the list line shows as saved. (`TitlePage.kt:173-186`) */
export function lineWord(line: Line, data: TitleDetail, saved: boolean, words: Record<string, string>): string {
  switch (line) {
    case "play":
      return playWord(data, words);
    case "episodes":
      return say(words, KEY.titleEpisodesMore);
    case "more":
      return say(words, KEY.titleMoreLike);
    case "list":
      return say(words, saved ? KEY.listAdded : KEY.listAdd);
    default:
      return "";
  }
}

/** Where the remote is on a title's page, and what each key does there. */
export class TitleFocus {
  part: Part = "menu";
  /** Which line of the menu. Kept by the page so coming back lands on it. (`TitlePage.kt:90`) */
  line: number;
  /** Which of the three reactions. The middle one is where the row opens. (`TitlePage.kt:109`) */
  reaction = 1;
  /** True once the viewer has saved the title. (`TitlePage.kt:111`) */
  saved: boolean;
  rating: Rating | null;

  private readonly lines: Line[];

  constructor(data: TitleDetail, line = 0) {
    this.lines = linesOf(data);
    this.line = Math.min(line, this.lines.length - 1);
    this.saved = data.in_list === true;
    this.rating = data.rating ?? null;
  }

  get here(): Line | null {
    return this.lines[this.line] ?? null;
  }

  /**
   * A key press. The answer is what to do: `nothing` when the press only moved the focus, and
   * `back` when the page should leave. Every branch is `TitlePage.kt:143-171`.
   */
  press(word: Word): Deed {
    switch (word) {
      case "up":
        if (this.part === "menu" && this.line > 0) this.line -= 1;
        else if (this.part === "menu") this.part = "reactions";
        return { kind: "nothing" };
      case "down":
        if (this.part === "reactions") this.part = "menu";
        else if (this.line < this.lines.length - 1) this.line += 1;
        return { kind: "nothing" };
      // The menu sits at the left edge: Left from it goes back, as Back does. (`TitlePage.kt:154`)
      case "left":
        if (this.part === "reactions" && this.reaction > 0) this.reaction -= 1;
        else if (this.part === "menu") return { kind: "back" };
        return { kind: "nothing" };
      case "right":
        if (this.part === "reactions" && this.reaction < REACTION_COUNT - 1) this.reaction += 1;
        return { kind: "nothing" };
      case "ok":
        if (this.part === "reactions") return { kind: "rate", rating: this.rate() };
        if (this.here === "play") return { kind: "play" };
        if (this.here === "episodes") return { kind: "episodes" };
        if (this.here === "list") {
          this.saved = !this.saved;
          return { kind: "list" };
        }
        return { kind: "more" };
      // From the ratings Back returns to the menu first, then leaves the title. (`TitlePage.kt:167-168`)
      case "back":
        if (this.part === "reactions") {
          this.part = "menu";
          return { kind: "nothing" };
        }
        return { kind: "back" };
      default:
        return { kind: "nothing" };
    }
  }

  /** Pressing a reaction that is already chosen takes it back. (`TitlePage.kt:132-136`) */
  private rate(): Rating | null {
    const asked = FACES[this.reaction]?.rating ?? null;
    this.rating = this.rating === asked ? null : asked;
    return this.rating;
  }
}
