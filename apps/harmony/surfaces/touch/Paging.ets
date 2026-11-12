// How a page grows. A port of `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/
// Paging.kt`, and the reasoning in that file is the specification: the answer a screen arrives with
// carries the first part only -- twelve rows of fifteen titles -- and the rest is read back through
// the one field the contract already has for it, a screen's `cursor` for more rows and a section's
// `cursor` for more titles, both through `/v1/more/{cursor}`.
//
// Nothing here is new shape: no route, no field, no control on the screen. A page grows as the
// viewer travels, the way the reference's does. `apps/apple/Sources/SurfacesPhone/browse/Home.swift:
// 301` and `Paging.kt:75,120` are the two families that already do this, and a third answer is not
// written here.
//
// Two differences from the Kotlin, both forced by the toolkit and neither a change of behaviour:
//
//   * Compose reads how far a list has been flung out of `layoutInfo.visibleItemsInfo`, which ArkUI
//     has no equivalent of. ArkUI reports the visible range as an EVENT instead --
//     `onScrollIndex(start, end, center)`, API 9, with the centre added at API 10 -- so a row and a
//     page push their position in rather than having it read out of them. The trigger arithmetic is
//     unchanged.
//   * Compose's `remember(stamp)` throws the state away when the stamp changes. Here [Page] and
//     [Depth] are objects a screen holds, and the screen builds a new one when [pageStamp] moves.
//
// There is no lock around the readings, where `Paging.kt:129,139` has `synchronized`. ArkTS runs one
// of these at a time on the thread that owns the screen: a port answers on the thread that asked
// (`Pictures.fetch`), the host steps on one thread because a core handle may be used from one
// thread at a time, and a Worker would be a VM of its own rather than a second thread in this one.
// If OPEN QUESTION Q3 is answered by moving the core to a Worker, this comment is the place that
// says what has to be looked at again.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { More, Screen, Section, Tile } from '../../contract/contract';
import { good, readMore } from '../../contract/read';
import { Shell } from '../../ports/Ports';
import { ROWS_AHEAD, TILES_AHEAD } from './Sizes';

/**
 * A part, or nothing at all: a refusal and a body that does not parse are the same answer here.
 * `Paging.kt:152-155`.
 *
 * The read itself is in `../../contract/read.ts`, on the TypeScript side of the seam: ArkTS bans
 * `any` (`arkts-no-any-unknown`, 10605008) and `JSON.parse` is typed `any`, so the official ArkTS
 * reading of a nested answer is a hand walk of the whole tree. `effects.ts` says why that seam
 * exists and what it costs.
 */
function part(status: number, text: string): More | null {
  return good(status) ? readMore(text) : null;
}

/**
 * Which answer a page's read parts belong to: the route, the cursor that answer carried and how many
 * sections it carried. A page built again gets a new stamp and starts its paging again.
 *
 * Both halves of the reading are keyed on it -- the page's own parts and each row's depth -- because
 * they have to be thrown away together. `Paging.kt:59-69` records what keying them apart did: a
 * refreshed Home kept every row's old cursor while the rows went back to the fifteen tiles the
 * answer carries, so a row either stopped growing or asked again for tiles it was already showing.
 */
export function pageStamp(key: string, screen: Screen | null): string {
  const cursor = screen === null || screen.cursor === undefined ? 'none' : screen.cursor;
  const held = screen === null ? 0 : screen.sections.length;
  return key + '|' + cursor + '|' + held.toString();
}

/**
 * Where a reading is up to: the cursor it would ask for next, and whether it is asking now.
 *
 * The two are one value on purpose. `Paging.kt:49-57` records why: held apart, the end of a read
 * wrote them one after the other, and a draw that fell between the two writes saw "not asking"
 * beside a cursor that had already been spent and asked the server for it a second time. "That was
 * watched happening on a device: two requests for `page-...-36` and two for `-48` in one scroll to
 * the bottom."
 */
class Reading {
  readonly cursor: string | null;
  readonly asking: boolean;

  constructor(cursor: string | null, asking: boolean) {
    this.cursor = cursor;
    this.asking = asking;
  }
}

/** The sections of one page as the viewer can reach them: the answer's, plus every part read since. */
export class Page {
  private readonly shell: Shell;
  private base: Section[] = [];
  private parts: Section[] = [];
  private reading: Reading;
  /** Told when a part arrives, so the screen holding this draws the rows it just gained. */
  private arrived: (sections: Section[]) => void = () => {};

  constructor(shell: Shell, screen: Screen | null) {
    this.shell = shell;
    // `Screen.sections` is not optional on the wire, so there is nothing to default here.
    this.base = screen === null ? [] : screen.sections;
    const cursor = screen === null || screen.cursor === undefined ? null : screen.cursor;
    this.reading = new Reading(cursor, false);
  }

  listen(arrived: (sections: Section[]) => void): void {
    this.arrived = arrived;
  }

  /**
   * Every section the viewer can reach.
   *
   * Two parts could name one section if a page were built again mid-read, and a list with the same
   * key twice throws, so the id decides. `Paging.kt:99-101`.
   */
  sections(): Section[] {
    const all: Section[] = [];
    const seen = new Set<string>();
    for (const held of this.base) {
      if (!seen.has(held.id)) {
        seen.add(held.id);
        all.push(held);
      }
    }
    for (const held of this.parts) {
      if (!seen.has(held.id)) {
        seen.add(held.id);
        all.push(held);
      }
    }
    return all;
  }

  /**
   * The page says how far down the viewer is; the next part is read once the end is near.
   *
   * `held` is how many items the page has and `lastVisible` is the last one on screen, which is
   * what `onScrollIndex`'s `end` is. `Paging.kt:78-84`.
   */
  near(held: number, lastVisible: number): void {
    if (held <= 0 || lastVisible < held - ROWS_AHEAD) {
      return;
    }
    const next = this.reading.cursor;
    if (next === null || this.reading.asking) {
      return;
    }
    this.reading = new Reading(next, true);
    this.shell.ask('/v1/more/' + next, (status: number, text: string): void => {
      const more = part(status, text);
      if (more !== null && more.sections !== undefined) {
        this.parts = this.parts.concat(more.sections);
        this.arrived(this.sections());
      }
      // A part that does not arrive stops this page. Asking the same cursor again on the next frame
      // would be a loop against a server that has already said no.
      this.reading = new Reading(more === null || more.cursor === undefined ? null : more.cursor, false);
    });
  }
}

/**
 * How deep each row of a page has been read.
 *
 * A row is drawn inside the page's list and is disposed when it scrolls off, so its depth is held
 * here, at the page: a row the viewer comes back to is as deep as they left it. `Paging.kt:104-108`.
 */
export class Depth {
  private readonly shell: Shell;
  private readonly parts: Map<string, Tile[]> = new Map<string, Tile[]>();
  private readonly reading: Map<string, Reading> = new Map<string, Reading>();
  /** One per row on screen. A row registers when it appears and drops it when it goes. */
  private readonly listeners: Map<string, (tiles: Tile[]) => void> = new Map<string, (tiles: Tile[]) => void>();

  constructor(shell: Shell) {
    this.shell = shell;
  }

  listen(id: string, arrived: (tiles: Tile[]) => void): void {
    this.listeners.set(id, arrived);
  }

  forget(id: string): void {
    this.listeners.delete(id);
  }

  /** Every title of this row the viewer can reach. `Paging.kt:120-123`. */
  tiles(section: Section): Tile[] {
    const own = section.tiles === undefined ? [] : section.tiles;
    const got = this.parts.get(section.id);
    if (got === undefined) {
      return own;
    }
    const all: Tile[] = [];
    const seen = new Set<string>();
    for (const held of own.concat(got)) {
      if (!seen.has(held.title)) {
        seen.add(held.title);
        all.push(held);
      }
    }
    return all;
  }

  /** The row says how far the viewer has flung; the next part is read once the end is near. `Paging.kt:126-141`. */
  reach(section: Section, held: number, lastVisible: number): void {
    if (held <= 0 || lastVisible < held - TILES_AHEAD) {
      return;
    }
    const id = section.id;
    const now = this.reading.get(id) ??
      new Reading(section.cursor === undefined ? null : section.cursor, false);
    const next = now.cursor;
    if (now.asking || next === null) {
      return;
    }
    this.reading.set(id, new Reading(next, true));
    this.shell.ask('/v1/more/' + next, (status: number, text: string): void => {
      const more = part(status, text);
      if (more !== null && more.tiles !== undefined) {
        this.parts.set(id, (this.parts.get(id) ?? []).concat(more.tiles));
        const told = this.listeners.get(id);
        if (told !== undefined) {
          told(this.tiles(section));
        }
      }
      // As with a page: what did not arrive is not asked for twice.
      this.reading.set(id, new Reading(more === null || more.cursor === undefined ? null : more.cursor, false));
    });
  }
}
