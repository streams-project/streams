// Where the viewer is on a page of rows, and what each key does there. It knows nothing about
// drawing.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Browse.kt:55-152` and the
// `press` and `choose` of `Browse.kt:218-253`. The television surface keeps this apart from the
// drawing so a key map can be checked without a screen; `src/tests/focus.test.ts` is that check,
// and it needs no Roku.
//
// The rules it carries are note 187's: Back walks out the way the viewer came in, one level at a
// time, and only a root page leaves the app
// (`notes/187-tv-navigation-per-page-2026-11-08.md:16-19`, and its row "Browse" at line 44).

import type { Section, Tile, TitleKey } from "./contract";
import type { Word } from "./keys";

/** The place of the search button, to the left of the first tab. (`TopBar.kt:31`) */
export const SEARCH_TAB = -2;

/** The place of the profile, to the left of search. (`TopBar.kt:34`) */
export const WHO_TAB = -3;

/** The three places the remote can be on a page of rows. (`Browse.kt:55`) */
export type Where = "top" | "hero" | "row";

/** What pressing OK on a page of rows means. The caller turns it into an address. */
export type Choice =
  | { kind: "who" }
  | { kind: "search" }
  | { kind: "tab"; at: number }
  | { kind: "title"; title: TitleKey }
  | { kind: "watch"; title: TitleKey }
  | { kind: "nothing" };

/** A series, by its key. `movie:` and `tv:` are the only two kinds. (`TitlePage.kt:56`) */
function isSeries(title: TitleKey): boolean {
  return title.startsWith("tv:");
}

export class PageFocus {
  rows: Section[];
  tabs: number;
  hero: boolean;
  where: Where;
  row = 0;
  tab: number;
  button = 0;

  /** The tab this page is: Back brings a wandering focus home to it before leaving. (`Browse.kt:76`) */
  private readonly active: number;

  /**
   * Where each row was left, by the row's id: the place and the title on it. A row the viewer comes
   * back to, or sees below, opens there; a fresh answer that moved the title keeps the title.
   * (`Browse.kt:82-86`, and note 187's "Rows keep their place" at lines 74-84)
   */
  private readonly columns = new Map<string, { at: number; title: TitleKey | null }>();

  constructor(rows: Section[], tabs: number, active: number, hero: boolean) {
    this.rows = rows;
    this.tabs = tabs;
    this.hero = hero;
    this.active = active;
    this.tab = active;
    this.where = hero || rows.length === 0 ? this.above() : "row";
  }

  /** The place above the rows: the hero when the page has one, the bar when it has not. (`Browse.kt:77`) */
  private above(): Where {
    return this.hero ? "hero" : "top";
  }

  /** The tile a row was left on, or its first. (`Browse.kt:88-96`) */
  columnOf(index: number): number {
    const section = this.rows[index];
    if (section === undefined) return 0;
    const tiles = section.tiles ?? [];
    const kept = this.columns.get(section.id);
    if (kept === undefined) return 0;
    if (tiles[kept.at]?.title === kept.title) return kept.at;
    const moved = tiles.findIndex((tile: Tile) => tile.title === kept.title);
    if (moved >= 0) return moved;
    return Math.min(Math.max(kept.at, 0), Math.max(tiles.length - 1, 0));
  }

  get column(): number {
    return this.columnOf(this.row);
  }

  private keep(at: number): void {
    const section = this.rows[this.row];
    if (section === undefined) return;
    this.columns.set(section.id, { at, title: (section.tiles ?? [])[at]?.title ?? null });
  }

  private lastColumn(): number {
    return (this.rows[this.row]?.tiles ?? []).length - 1;
  }

  /** The last row that exists, or -1 when the page has none. */
  private lastRow(): number {
    return this.rows.length - 1;
  }

  /** True when the key moved the focus. False means the key was not used here. (`Browse.kt:107-151`) */
  press(word: Word): boolean {
    switch (word) {
      case "up":
        if (this.where === "row" && this.row > 0) this.row -= 1;
        else if (this.where === "row") this.where = this.above();
        else if (this.where === "hero") this.where = "top";
        else return false;
        return true;
      case "down":
        if (this.where === "top" && this.hero) this.where = "hero";
        else if ((this.where === "hero" || this.where === "top") && this.rows.length > 0) {
          this.where = "row";
          this.row = 0;
        } else if (this.where === "row" && this.row < this.lastRow()) this.row += 1;
        else return false;
        return true;
      case "left":
        if (this.where === "top" && this.tab > 0) this.tab -= 1;
        else if (this.where === "top" && this.tab === 0) this.tab = SEARCH_TAB;
        else if (this.where === "top" && this.tab === SEARCH_TAB) this.tab = WHO_TAB;
        else if (this.where === "hero") this.button = 0;
        else if (this.where === "row" && this.column > 0) this.keep(this.column - 1);
        else return false;
        return true;
      case "right":
        if (this.where === "top" && this.tab === WHO_TAB) this.tab = SEARCH_TAB;
        else if (this.where === "top" && this.tab === SEARCH_TAB) this.tab = 0;
        else if (this.where === "top" && this.tab < this.tabs - 1) this.tab += 1;
        else if (this.where === "hero") this.button = 1;
        else if (this.where === "row" && this.column < this.lastColumn()) this.keep(this.column + 1);
        else return false;
        return true;
      case "back":
        if (this.where === "row") this.where = this.above();
        else if (this.where === "hero") this.where = "top";
        // Focus wandered along the bar without choosing: Back brings it home to this page's tab
        // first, and only then leaves the page. (note 187, the Browse row at line 44)
        else if (this.tab !== this.active) this.tab = this.active;
        else return false;
        return true;
      case "ok":
        return false;
      default:
        return false;
    }
  }

  /**
   * What OK means where the remote is now. The hero's first button plays a film and opens a series,
   * because a series has an episode to pick. (`Browse.kt:218-230`)
   */
  choose(lead: Tile | null): Choice {
    if (this.where === "top" && this.tab === WHO_TAB) return { kind: "who" };
    if (this.where === "top" && this.tab === SEARCH_TAB) return { kind: "search" };
    if (this.where === "top") return { kind: "tab", at: this.tab };
    if (this.where === "row") {
      const tile = (this.rows[this.row]?.tiles ?? [])[this.column];
      return tile === undefined ? { kind: "nothing" } : { kind: "title", title: tile.title };
    }
    if (lead === null) return { kind: "nothing" };
    if (this.button === 1 || isSeries(lead.title)) return { kind: "title", title: lead.title };
    return { kind: "watch", title: lead.title };
  }

  /**
   * Whether the page should read its next rows: the focus has come within `reach` of the last row
   * the page holds, or the page has no rows at all and can only grow by asking.
   * (`Browse.kt:196-204`)
   */
  withinReach(reach: number): boolean {
    if (this.rows.length === 0) return true;
    return this.where === "row" && this.row >= this.lastRow() - reach;
  }
}
