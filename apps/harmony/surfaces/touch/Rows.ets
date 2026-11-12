// One row, and the page a row's name opens. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Rows.kt:75-243,491-583`.
//
// "The row is as deep as the page has read it, and reads further as the viewer flings: the
// reference's standard row runs to 70 titles and this project's server holds 120, against the 15
// one answer carries" (`Rows.kt:103-105`).
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
// `LengthMetrics` is how a layout mirrors: `LocalizedPadding`, `LocalizedMargin` and
// `LocalizedBorderRadiuses` take `start`/`end` (and `topStart`/`topEnd`) rather than left and right,
// and their fields are typed `LengthMetrics` with no plain-number branch, so a bare number in one of
// them is refused. The reference's RTL guide is explicit: "use the generalized direction terms start
// and end as parameter types instead of absolute terms".
import { LengthMetrics } from '@kit.ArkUI';
import { Section, SectionKind, Tile } from '../../contract/contract';
import { Pictures, Seen } from '../../ports/Ports';
import { Depth } from './Paging';
import { ContinueCard, rankOf, RankedTileCard, TileCard } from './Tiles';
import { Art } from './Pictures';
import { tall } from './Pick';
import { warm } from './Pictures';
import { fill, word } from './Words';
import { SKIP_ALL } from './Heard';
import { argb, BACK, CORNER_CARD, INK, RAISED } from './Theme';
import { ICON_BACK } from './Icons';
import {
  BAR_BOX, BAR_ICON, CONTINUE_HIGH, CONTINUE_STRIP_HIGH, GRID_ACROSS, GRID_BAR, GRID_EDGE_HIGH,
  GRID_TITLE_TYPE, ROW_GAP, ROW_NAME_GAP, ROW_NAME_HIGH, ROW_NAME_TYPE, ROW_OPENS_PAST, ROW_START,
  TILE_GAP, TILE_HIGH, TILE_WIDE, WARM_AHEAD,
} from './Sizes';

/** The rows a page shows, in the server's order. `browse/Rows.kt:76`. */
export const ROW_KINDS: SectionKind[] = ['row', 'ranked', 'continue', 'grid'];

export function isRow(section: Section): boolean {
  for (const kind of ROW_KINDS) {
    if (section.kind === kind) {
      return true;
    }
  }
  return false;
}

/**
 * One row: its name over a sideways list of tiles.
 *
 * Reference: the name 34.3 high, bold, 18; 4 to the tiles; 12 to the next row (`Rows.kt:97-99`).
 * A Top 10 row shows large outlined rank numerals beside each tile; Continue Watching shows progress
 * and action buttons; a tap on the row's name opens the whole row as a page when there is more of it
 * than the row shows.
 */
@ComponentV2
export struct SectionRow {
  @Require @Param section: Section;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Require @Param depth: Depth;
  @Event open: (tile: Tile) => void = () => {};
  @Event openInfo: (tile: Tile) => void = () => {};
  @Event remove: (tile: Tile) => void = () => {};
  @Event openGrid: (section: Section, tiles: Tile[]) => void = () => {};

  /** Every title of this row the viewer can reach. The depth answers with more as they fling. */
  @Local tiles: Tile[] = [];
  /**
   * Titles the viewer took out of a Continue row, which go at once rather than waiting for the
   * server to agree. `browse/Rows.kt:118-119`.
   */
  @Local hidden: string[] = [];

  aboutToAppear(): void {
    this.tiles = this.depth.tiles(this.section);
    this.depth.listen(this.section.id, (tiles: Tile[]): void => {
      this.tiles = tiles;
    });
  }

  aboutToDisappear(): void {
    this.depth.forget(this.section.id);
  }

  /**
   * A list reuses a row for the next row of its kind rather than building one, so the section
   * changes under a node that is already on screen. `browse/Home.kt:202-204` relies on that reuse;
   * without this the row would go on showing the titles of whichever section it was built for.
   */
  @Monitor('section')
  moved(): void {
    this.depth.listen(this.section.id, (tiles: Tile[]): void => {
      this.tiles = tiles;
    });
    this.tiles = this.depth.tiles(this.section);
    this.hidden = [];
  }

  build() {
    Column() {
      // The name opens the row as a page when there is more of it than the row shows. The chevron
      // that says so is drawn, so it is said too: on its own the merged words would be the row's
      // name and a bare glyph, "which tells a reader nothing about where the control goes"
      // (`browse/Rows.kt:147-149`).
      Stack({ alignContent: Alignment.Start }) {
        Row() {
          Text(this.section.title ?? '')
            .fontSize(ROW_NAME_TYPE)
            .fontWeight(FontWeight.Bold)
            .fontColor(INK)
            .maxLines(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .layoutWeight(1)

          if (this.hasMore()) {
            // Not a word: the glyph the reference draws, turned with the reading direction.
            Text(this.seen.rtl ? '‹' : '›')
              .fontSize(ROW_NAME_TYPE)
              .fontWeight(FontWeight.Bold)
              .fontColor(argb(INK, 0.7))
              .margin({ start: LengthMetrics.vp(6) })
          }
        }
        .width('100%')
      }
      .width('100%')
      .constraintSize({ minHeight: ROW_NAME_HIGH })
      .padding({ start: LengthMetrics.vp(ROW_START), end: LengthMetrics.vp(ROW_START) })
      .onClick(() => {
        if (this.hasMore()) {
          this.openGrid(this.section, this.shown());
        }
      })
      .accessibilityGroup(this.hasMore())
      .accessibilityText(
        this.hasMore() ? fill(word(this.seen, 'say.see_all'), 'row', this.section.title ?? '') : '',
      )

      Stack().height(ROW_NAME_GAP)

      List({ space: TILE_GAP }) {
        Repeat<Tile>(this.shown())
          .each((held: RepeatItem<Tile>) => {
            ListItem() {
              this.one(held.item, held.index);
            }
          })
          .key((tile: Tile, index: number) => tile.title)
          .virtualScroll()
      }
      .listDirection(Axis.Horizontal)
      .scrollBar(BarState.Off)
      .width('100%')
      .height(this.rowHigh())
      // The row starts 12 in and runs to the screen's edge, where the reference cuts its last tile:
      // "the last one cut by the edge as the reference's is" (`browse/Empty.kt:78-81`). So there is
      // a start offset and no end offset -- `contentStartOffset` is API 11 and verified; a matching
      // end offset was not read off the reference, and the row does not want one.
      .contentStartOffset(ROW_START)
      // How far the viewer has flung this row decides two things: when the next part of the row is
      // read, and which pictures are fetched before anything draws them
      // (`browse/Rows.kt:131-144`). Compose reads this out of `layoutInfo`; ArkUI reports it.
      .onScrollIndex((start: number, end: number) => this.reached(end))
    }
    .width('100%')
    .padding({ bottom: ROW_GAP })
    .alignItems(HorizontalAlign.Start)
  }

  /**
   * How tall this row's tiles are. Set rather than left to the content so that a row of one kind
   * reused for the next row of that kind does not jump: a Continue tile carries a 38 strip under
   * its 138 of art (`browse/Rows.kt:336,367`) and every other tile is the measured 166.6.
   */
  private rowHigh(): number {
    return this.section.kind === 'continue' ? CONTINUE_HIGH + CONTINUE_STRIP_HIGH : TILE_HIGH;
  }

  @Builder
  one(tile: Tile, index: number) {
    if (this.section.kind === 'ranked') {
      RankedTileCard({
        tile: tile,
        seen: this.seen,
        pictures: this.pictures,
        // The server's rank when it carries one, else the place in the row. `browse/Rows.kt:198`.
        rank: rankOf(tile) > 0 ? rankOf(tile) : index + 1,
        open: () => this.open(tile),
      })
    } else if (this.section.kind === 'continue') {
      ContinueCard({
        tile: tile,
        seen: this.seen,
        pictures: this.pictures,
        open: () => this.open(tile),
        openInfo: () => this.openInfo(tile),
        remove: () => {
          this.hidden = this.hidden.concat([tile.title]);
          this.remove(tile);
        },
      })
    } else {
      TileCard({
        tile: tile,
        seen: this.seen,
        pictures: this.pictures,
        rank: rankOf(tile),
        open: () => this.open(tile),
      })
    }
  }

  /** The titles this row draws: every one it has read, less the ones the viewer took out. */
  private shown(): Tile[] {
    if (this.hidden.length === 0) {
      return this.tiles;
    }
    const left: Tile[] = [];
    for (const tile of this.tiles) {
      if (this.hidden.indexOf(tile.title) < 0) {
        left.push(tile);
      }
    }
    return left;
  }

  /** `browse/Rows.kt:122`. */
  private hasMore(): boolean {
    return this.section.cursor !== undefined || this.tiles.length > ROW_OPENS_PAST;
  }

  private reached(end: number): void {
    const shown = this.shown();
    this.depth.reach(this.section, shown.length, end);
    for (let place = end + 1; place <= end + WARM_AHEAD; place++) {
      if (place < shown.length) {
        warm(this.pictures, tall(shown[place].images), TILE_WIDE, TILE_HIGH);
      }
    }
  }
}

/**
 * The whole of a row, as a page. A port of `browse/Rows.kt:491-583`.
 *
 * It reads the rest of the row as the viewer scrolls, through the same cursor the row itself uses,
 * "so the grid reaches the whole row and not only the part the page arrived with".
 *
 * The Kotlin draws it in a `Dialog`; here the page that owns the row draws it over itself, which is
 * what `browse/Home.kt:248-321` already does for the categories overlay. That is a change of
 * mechanism and not of behaviour, and it avoids a second answer to what a full-screen dialog is on
 * ArkUI.
 */
@ComponentV2
export struct RowGrid {
  @Require @Param section: Section;
  @Require @Param tiles: Tile[];
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Require @Param depth: Depth;
  @Event open: (tile: Tile) => void = () => {};
  @Event close: () => void = () => {};

  build() {
    Column() {
      Row() {
        Stack() {
          Image(ICON_BACK)
            .width(BAR_ICON)
            .height(BAR_ICON)
            .fillColor(INK)
            // The arrow points the way Back goes, so it turns with the reading direction.
            .scale(this.seen.rtl ? { x: -1 } : { x: 1 })
            .accessibilityLevel(SKIP_ALL)
        }
        .width(BAR_BOX)
        .height(BAR_BOX)
        .onClick(() => this.close())
        .accessibilityGroup(true)
        .accessibilityText(word(this.seen, 'common.back'))

        Stack().width(8)

        Text(this.section.title ?? '')
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

      Grid() {
        Repeat<Tile>(this.tiles)
          .each((held: RepeatItem<Tile>) => {
            GridItem() {
              Stack() {
                Art({
                  url: tall(held.item.images),
                  wide: TILE_WIDE,
                  high: TILE_HIGH,
                  pictures: this.pictures,
                })
              }
              .width('100%')
              .height(TILE_HIGH)
              .backgroundColor(RAISED)
              .borderRadius(CORNER_CARD)
              .clip(true)
              .onClick(() => {
                this.close();
                this.open(held.item);
              })
              .accessibilityGroup(true)
              .accessibilityText(held.item.name)
            }
          })
          .key((tile: Tile, index: number) => tile.title)
          .virtualScroll()
      }
      // Three across, as the reference's own grid is (`browse/Rows.kt:557`). Written from
      // GRID_ACROSS rather than typed, so the number lives in one file.
      .columnsTemplate(this.columns())
      .columnsGap(TILE_GAP)
      .rowsGap(TILE_GAP)
      .padding({
        start: LengthMetrics.vp(ROW_START),
        end: LengthMetrics.vp(ROW_START),
        top: LengthMetrics.vp(GRID_EDGE_HIGH),
        bottom: LengthMetrics.vp(GRID_EDGE_HIGH),
      })
      .layoutWeight(1)
      .width('100%')
      .onScrollIndex((start: number, end: number) => {
        this.depth.reach(this.section, this.tiles.length, end);
      })
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BACK)
  }

  private columns(): string {
    const parts: string[] = [];
    for (let at = 0; at < GRID_ACROSS; at++) {
      parts.push('1fr');
    }
    return parts.join(' ');
  }
}
