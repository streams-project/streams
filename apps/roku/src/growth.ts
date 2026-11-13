// How a page of rows pages, and how far ahead it reads.
//
// A MIRROR of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Browse.kt:32-46` and
// `:139-168`, the `PageGrowth` class and the two effects that drive it. See the head of
// `src/tier.ts` for why this folder mirrors rules instead of calling the shared core.
//
// What the rule is, in one paragraph, because it is easy to get almost right: the server builds up
// to 50 rows and hands a screen the first 12 with a cursor for the rest (`Browse.kt:34-37`). The
// page follows that cursor as the focus comes within `REACH` of the last row it holds, so the rows
// are there before the viewer reaches them: a remote cannot flick past twelve rows the way a thumb
// can, so arriving-on-demand would show an empty page every time. Until this existed, nothing
// outside `apps/web` followed the cursor and 38 built rows could not be reached from a television
// at all (`notes/netflix-reference/00-gaps.md`, gap 1).
//
// Three things it has to get right, each of which is a separate case in `src/tests/growth.test.ts`:
//
//   1. a fresh answer is a fresh page: what was read past the first rows belongs to the answer it
//      came from, so a new cursor throws the extra rows away rather than appending to them;
//   2. one read at a time, however many times the focus moves while it is in flight;
//   3. an answer with no rows left is the end, **whatever cursor came with it** — a cursor is not
//      a promise that anything is behind it.
//
// `REACH` is not restated here. It lives in `src/numbers.ts:119-123` with the Kotlin line it came
// from, which is where every number this surface lays out with lives.

import type { Cursor, More, Section } from "./contract";

/** A section with no tiles is not a row: a row with no titles in it is an empty line under a name. */
function filled(sections: Section[]): Section[] {
  return sections.filter((section) => section.tiles !== undefined && section.tiles.length > 0);
}

/** The rows a page has past its first answer. (`Browse.kt:38-46`) */
export class PageGrowth {
  /** The cursor the first answer came with. A fresh answer carries a new one and starts again. */
  from: Cursor | null = null;

  /** The next cursor to read, or nothing when the page has given everything it has. */
  next: Cursor | null = null;

  /** True while a read is in flight. */
  busy = false;

  /** The rows read past the first answer, in the order they arrived. */
  extra: Section[] = [];

  /**
   * A fresh answer arrived for this page. True when it started the page again. (`Browse.kt:140-148`)
   *
   * The comparison is on the answer's own cursor and not on its rows: the same page re-read gives
   * the same rows and a new cursor, and a page that kept its extra rows across that would show them
   * twice.
   */
  answered(cursor: Cursor | null | undefined): boolean {
    const given = cursor ?? null;
    if (this.from === given) return false;
    this.from = given;
    this.next = given;
    this.extra = [];
    this.busy = false;
    return true;
  }

  /**
   * The cursor to read now, or nothing. Asking marks the page busy, so a focus that keeps moving
   * while the answer is in flight does not start a second read. (`Browse.kt:150-158`)
   *
   * `wants` is the page's own answer to "is the focus within reach of the last row I hold", which
   * is `PageFocus.wantsMore()`.
   */
  reads(wants: boolean): Cursor | null {
    if (this.next === null || this.busy || !wants) return null;
    this.busy = true;
    return this.next;
  }

  /**
   * What came back. An answer with no rows left is the end, whatever cursor came with it, and rows
   * with no tiles are dropped on the way in. (`Browse.kt:159-167`)
   */
  took(more: More): void {
    const rows = filled(more.sections ?? []);
    this.next = rows.length === 0 ? null : (more.cursor ?? null);
    this.extra = this.extra.concat(rows);
    this.busy = false;
  }

  /**
   * The read did not answer. The page is free to ask again, with the same cursor.
   * (`Browse.kt:166-168`: the `finally` that clears `busy` whichever way the body left.)
   */
  failed(): void {
    this.busy = false;
  }
}
