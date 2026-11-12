// Search.
//
// READ THIS BEFORE CHANGING THE LAYOUT. There is no measured touch search screen in this project.
// The Android phone surface has no search screen at all: `browse/Home.kt:349` sends the viewer to
// `/search` and the server answers with a `Screen`, which `Home` then draws as rows like any other
// page. The only search screen that exists is the television's
// (`apps/android/surfaces/tv/src/main/kotlin/streams/tv/search/Search.kt`, 300 lines), and it is
// built for a remote: letters down the left, genres under them, posters on the right, with focus
// moving between four zones. None of that is touch.
//
// So what is measured here, and what is not:
//
//   measured   the BEHAVIOUR, from the television's: a 350 ms settle after the last keystroke
//              (`tv/search/Search.kt:31`), `/v1/search?q=` for the results and `/v1/screens/home`
//              for what is offered before anything is typed (`:81,91`), twelve of those offered
//              (`:32`), and the results read out of a `Screen`'s sections, flattened, by title
//              (`:54-58`).
//   measured   the GRID a page of tiles is drawn in: three across at the tile's own size, 6 apart,
//              12 in from the edge -- `browse/Rows.kt:556-561`, which is the grid a row's name
//              opens on this very surface.
//   NOT measured   where the field goes, how tall it is, what it says when empty, whether the
//              results are a grid or a list, and whether anything sits above them. Netflix's phone
//              search screen has not been captured by this project.
//
// This screen is therefore built out of the two measured halves and nothing else: the television's
// behaviour, drawn in the surface's own grid, with one field above it. Every number in it comes from
// `Sizes.ets` with its citation, and there is not one number here that was chosen. What would settle
// the rest is one measurement pass on the Netflix phone app's search screen, which is an open item
// in the note.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { LengthMetrics } from '@kit.ArkUI';
import { Screen, Tile } from '../../contract/contract';
import { good, readScreen } from '../../contract/read';
import { Pictures, Seen, Shell } from '../../ports/Ports';
import { Art } from './Pictures';
import { tall, titleAddress } from './Pick';
import { word } from './Words';
import { SKIP_ALL } from './Heard';
import { BACK, CORNER_CARD, CORNER_FIELD, INK, MUTED, RAISED } from './Theme';
import { ICON_BACK } from './Icons';
import {
  BAR_BOX, BAR_ICON, BAR_ROOM, GRID_ACROSS, GRID_BAR, GRID_EDGE_HIGH, GRID_TITLE_TYPE, OFFERED,
  ROW_START, SETTLE_MS, TILE_GAP, TILE_HIGH, TILE_RATIO, TILE_WIDE,
} from './Sizes';

/**
 * `SearchScreen` and not `Search`: `Search` is an ArkUI component, and a struct of that name in this
 * file would shadow the very component the screen is built out of -- the same trap `Pick.ets` names
 * for the contract's `Image`.
 */
@ComponentV2
export struct SearchScreen {
  @Require @Param shell: Shell;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Require @Param screenWide: number;

  @Local typed: string = '';
  @Local results: Tile[] = [];
  @Local offered: Tile[] = [];

  /** The settle in flight. A viewer still typing has not asked for anything yet. */
  private settling: number = 0;

  aboutToAppear(): void {
    // What is offered before anything is typed: the first titles of Home, as the television's
    // search offers (`tv/search/Search.kt:79-83`).
    this.shell.ask('/v1/screens/home', (status: number, text: string): void => {
      if (!good(status)) {
        return;
      }
      const read = readScreen(text);
      if (read !== null) {
        this.offered = this.tilesOf(read, OFFERED);
      }
    });
  }

  aboutToDisappear(): void {
    this.stopSettling();
  }

  private stopSettling(): void {
    if (this.settling !== 0) {
      clearTimeout(this.settling);
      this.settling = 0;
    }
  }

  /** `tv/search/Search.kt:54-58`: every tile of every section, once each, by title. */
  private tilesOf(screen: Screen, limit: number): Tile[] {
    const held: Tile[] = [];
    const seen = new Set<string>();
    for (const section of screen.sections) {
      for (const tile of section.tiles ?? []) {
        if (!seen.has(tile.title) && held.length < limit) {
          seen.add(tile.title);
          held.push(tile);
        }
      }
    }
    return held;
  }

  /** `tv/search/Search.kt:84-93`. */
  private asked(text: string): void {
    this.typed = text;
    this.stopSettling();
    if (text.trim().length === 0) {
      this.results = [];
      return;
    }
    this.settling = setTimeout(() => {
      this.settling = 0;
      const query = encodeURIComponent(text.trim());
      this.shell.ask('/v1/search?q=' + query, (status: number, body: string): void => {
        if (!good(status)) {
          return;
        }
        const read = readScreen(body);
        if (read !== null) {
          // No cap on the results, as the television's has none.
          this.results = this.tilesOf(read, Number.MAX_SAFE_INTEGER);
        }
      });
    }, SETTLE_MS);
  }

  /** What is on screen: the results once something is typed, else what is offered. */
  private shown(): Tile[] {
    return this.typed.trim().length === 0 ? this.offered : this.results;
  }

  build() {
    Column() {
      // The bar, as the row grid's is -- the one bar this surface has measured for a page of tiles
      // (`browse/Rows.kt:517-550`).
      Row() {
        Stack() {
          Image(ICON_BACK)
            .width(BAR_ICON)
            .height(BAR_ICON)
            .fillColor(INK)
            .scale(this.seen.rtl ? { x: -1 } : { x: 1 })
            .accessibilityLevel(SKIP_ALL)
        }
        .width(BAR_BOX)
        .height(BAR_BOX)
        .onClick(() => this.shell.back())
        .accessibilityGroup(true)
        .accessibilityText(word(this.seen, 'common.back'))

        Stack().width(8)

        Text(word(this.seen, 'nav.search'))
          .fontSize(GRID_TITLE_TYPE)
          .fontWeight(FontWeight.Bold)
          .fontColor(INK)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
          .layoutWeight(1)
      }
      .width('100%')
      .height(GRID_BAR)
      .padding({ start: LengthMetrics.vp(4), end: LengthMetrics.vp(4) })
      .alignItems(VerticalAlign.Center)

      // The field. `Search` is ArkUI's own, so the glyph, the clear control and the keyboard's
      // search key come from the platform rather than being drawn here. Its height is not a
      // measured number and so is not set: the component's own is what it gets.
      Search({
        value: this.typed,
        placeholder: word(this.seen, 'nav.search'),
      })
        .width('100%')
        .margin({ left: ROW_START, right: ROW_START })
        .backgroundColor(RAISED)
        .borderRadius(CORNER_FIELD)
        .placeholderColor(MUTED)
        .fontColor(INK)
        .caretColor(INK)
        .onChange((value: string) => this.asked(value))
        .onSubmit((value: string) => this.asked(value))

      // Before anything is typed, the server's own words for what is offered. The key exists
      // (`search.recommended`), so the line is drawn from it and from nothing else.
      if (this.typed.trim().length === 0 && this.offered.length > 0) {
        Text(word(this.seen, 'search.recommended'))
          .fontSize(GRID_TITLE_TYPE)
          .fontWeight(FontWeight.Bold)
          .fontColor(INK)
          .width('100%')
          .margin({ top: GRID_EDGE_HIGH })
          .padding({ start: LengthMetrics.vp(ROW_START), end: LengthMetrics.vp(ROW_START) })
      }

      Grid() {
        Repeat<Tile>(this.shown())
          .each((held: RepeatItem<Tile>) => {
            GridItem() {
              Stack() {
                Art({
                  url: tall(held.item.images),
                  wide: this.cellWide(),
                  high: Math.round(this.cellWide() / TILE_RATIO),
                  pictures: this.pictures,
                })
              }
              .width('100%')
              .height(TILE_HIGH)
              .backgroundColor(RAISED)
              .borderRadius(CORNER_CARD)
              .clip(true)
              .onClick(() => this.shell.go(titleAddress(held.item.title)))
              // A poster and nothing else, as in a row: the title's name is the tile.
              .accessibilityGroup(true)
              .accessibilityText(held.item.name)
            }
          })
          .key((tile: Tile, index: number) => tile.title)
          .virtualScroll()
      }
      .columnsTemplate(this.columns())
      .columnsGap(TILE_GAP)
      .rowsGap(TILE_GAP)
      .width('100%')
      .layoutWeight(1)
      .padding({
        start: LengthMetrics.vp(ROW_START),
        end: LengthMetrics.vp(ROW_START),
        top: LengthMetrics.vp(GRID_EDGE_HIGH),
        bottom: LengthMetrics.vp(BAR_ROOM),
      })
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BACK)
    .alignItems(HorizontalAlign.Start)
  }

  private columns(): string {
    const parts: string[] = [];
    for (let at = 0; at < GRID_ACROSS; at++) {
      parts.push('1fr');
    }
    return parts.join(' ');
  }

  /** One column of the grid, for the size the port is asked for. */
  private cellWide(): number {
    const room = this.screenWide - ROW_START * 2 - TILE_GAP * (GRID_ACROSS - 1);
    const each = Math.round(room / GRID_ACROSS);
    return each < TILE_WIDE ? each : TILE_WIDE;
  }
}
