// The three kinds of tile a row draws. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Rows.kt:249-489`.
//
// `Rows.kt:426-431` records what is NOT here and why: "There was a second shape here, a
// 184.6 x 369.2 dp panel for the first row carrying a TOP 10 corner badge and a caption chip
// reading New Episode or Watch Now. The 2026-11-11 reading of the reference has no tall first row
// and no text on a tile at all, so both are gone." A tile is a picture, sometimes a badge over it,
// and no words of its own.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { Tile } from '../../contract/contract';
import { Pictures, Seen } from '../../ports/Ports';
import { Art } from './Pictures';
import { tall } from './Pick';
import { fill, number, word } from './Words';
import { SKIP_ALL } from './Heard';
import {
  ACCENT, argb, CORNER_CARD, INK, RAISED, RANK_FILL, RANK_STROKE_INK, round,
} from './Theme';
import { ICON_INFO, ICON_MORE } from './Icons';
import {
  BADGE_EDGE, BADGE_NEW_PAD_SIDE, BADGE_NEW_TYPE, BADGE_PAD_SIDE, BADGE_PAD_TOP, BADGE_TYPE,
  CONTINUE_BUTTON, CONTINUE_HIGH, CONTINUE_STRIP_HIGH, PROGRESS_HIGH, RANK_EXTRA, RANK_EXTRA_TWO,
  RANK_NUMERAL_HIGH, RANK_NUMERAL_TYPE, RANK_NUMERAL_WIDE, RANK_NUMERAL_WIDE_TWO, RANK_STROKE,
  TILE_HIGH, TILE_WIDE,
} from './Sizes';

/** The rank a tile carries, or zero. `browse/Rows.kt:198,219`. */
export function rankOf(tile: Tile): number {
  if (tile.badges === undefined) {
    return 0;
  }
  for (const badge of tile.badges) {
    if (badge.rank !== undefined) {
      return badge.rank;
    }
  }
  return 0;
}

/** True when the server marked this title new. `browse/Rows.kt:464`. */
export function isNew(tile: Tile): boolean {
  if (tile.badges === undefined) {
    return false;
  }
  for (const badge of tile.badges) {
    if (badge.kind === 'new') {
      return true;
    }
  }
  return false;
}

/**
 * How far in the viewer is: a track at a tenth of white under a fill in the brand's accent.
 * `browse/Rows.kt:348-361`.
 *
 * Two plates and not a `Progress`: the Kotlin draws two boxes, and two boxes is a picture this
 * surface can be certain of. A `Progress` of the linear kind would want its track colour and its
 * fill colour set through attributes whose names were not read off the reference here, and getting
 * one wrong would show as a bar of the wrong colour on a device nobody has.
 */
@ComponentV2
struct Watched {
  @Require @Param percent: number;

  build() {
    Stack({ alignContent: Alignment.Start }) {
      Stack()
        .width(this.held() + '%')
        .height(PROGRESS_HIGH)
        .backgroundColor(ACCENT)
    }
    .width('100%')
    .height(PROGRESS_HIGH)
    // The alpha is baked into the colour and not applied with `.opacity()`: opacity would dim the
    // fill inside this node too, and the reference's fill is the accent at full strength.
    .backgroundColor(argb(INK, 0.3))
    .accessibilityLevel(SKIP_ALL)
  }

  private held(): number {
    return this.percent < 0 ? 0 : (this.percent > 100 ? 100 : this.percent);
  }
}

/**
 * One tile: the title's picture at 119 x 166.6, rounded by the brand.
 *
 * A tile draws a picture and sometimes a badge over it, and no words of its own. The title's name IS
 * the tile: it is the one thing a reader has to hear, and it comes from the answer, so it needs no
 * key (`browse/Rows.kt:443-445`).
 */
@ComponentV2
export struct TileCard {
  @Require @Param tile: Tile;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  /** A rank badge over the art, for a row that is not the ranked kind. Zero draws none. */
  @Param rank: number = 0;
  /** Whole percent watched, drawn as a bar. Below zero draws none. */
  @Param percent: number = -1;
  /** What a reader hears. The title's name unless the row has more to say about it. */
  @Param says: string = '';
  @Event open: () => void = () => {};

  build() {
    Stack({ alignContent: Alignment.TopStart }) {
      Art({
        url: tall(this.tile.images),
        wide: TILE_WIDE,
        high: TILE_HIGH,
        pictures: this.pictures,
      })

      if (this.rank > 0) {
        Text(fill(word(this.seen, 'badge.ranked'), 'rank', this.rank))
          .fontSize(BADGE_TYPE)
          .fontWeight(FontWeight.Bold)
          .fontColor(INK)
          .backgroundColor(ACCENT)
          .borderRadius(BADGE_EDGE)
          .padding({ left: BADGE_PAD_SIDE, right: BADGE_PAD_SIDE, top: BADGE_PAD_TOP, bottom: BADGE_PAD_TOP })
          .margin(BADGE_EDGE);
      }

      if (isNew(this.tile)) {
        Text(word(this.seen, 'badge.new'))
          .fontSize(BADGE_NEW_TYPE)
          .fontWeight(FontWeight.Bold)
          .fontColor(INK)
          .backgroundColor(ACCENT)
          .borderRadius(4)
          .padding({ left: BADGE_NEW_PAD_SIDE, right: BADGE_NEW_PAD_SIDE, top: BADGE_PAD_TOP, bottom: BADGE_PAD_TOP })
          .margin(BADGE_EDGE);
      }

      if (this.percent >= 0) {
        Watched({ percent: this.percent })
          .offset({ y: TILE_HIGH - PROGRESS_HIGH });
      }
    }
    .width(TILE_WIDE)
    .height(TILE_HIGH)
    .backgroundColor(RAISED)
    .borderRadius(CORNER_CARD)
    .clip(true)
    .onClick(() => this.open())
    .accessibilityGroup(true)
    .accessibilityText(this.says.length === 0 ? this.tile.name : this.says)
  }
}

/**
 * Top 10 row item: a large outlined rank numeral beside the tile, with the poster over it.
 * `browse/Rows.kt:245-308`. Digits follow the language.
 *
 * The numeral is one glyph drawn twice over itself, filled and then stroked. ArkUI's `Text` has no
 * stroke, so it is drawn on a `Canvas`. ASSUMED: `fillText`, `strokeText`, `font`, `lineWidth`,
 * `fillStyle` and `strokeStyle` are the ordinary members of a 2D canvas and ArkUI's
 * `CanvasRenderingContext2D` is documented as one, but these six were not read off the reference
 * one by one. If any is named differently the numeral is the only thing affected.
 *
 * It is decoration either way: "One numeral is drawn twice over itself, as a fill and as a stroke,
 * so read as it stands it is said twice. It is decoration: the place is part of what the tile itself
 * says" (`browse/Rows.kt:267-269`).
 */
@ComponentV2
export struct RankedTileCard {
  @Require @Param tile: Tile;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Require @Param rank: number;
  @Event open: () => void = () => {};

  private painter: CanvasRenderingContext2D = new CanvasRenderingContext2D(new RenderingContextSettings(true));

  build() {
    Stack({ alignContent: Alignment.BottomEnd }) {
      Stack({ alignContent: Alignment.Start }) {
        Canvas(this.painter)
          .width(this.rank >= 10 ? RANK_NUMERAL_WIDE_TWO : RANK_NUMERAL_WIDE)
          .height(RANK_NUMERAL_HIGH)
          .onReady(() => this.paint())
      }
      .width(this.rank >= 10 ? RANK_NUMERAL_WIDE_TWO : RANK_NUMERAL_WIDE)
      .height(RANK_NUMERAL_HIGH)
      .accessibilityLevel(SKIP_ALL)

      // The poster at the end, over the numeral. It carries the place, since the numeral does not.
      TileCard({
        tile: this.tile,
        seen: this.seen,
        pictures: this.pictures,
        says: fill(fill(word(this.seen, 'say.ranked'), 'rank', number(this.rank)), 'title', this.tile.name),
        open: () => this.open(),
      })
    }
    .width(TILE_WIDE + (this.rank >= 10 ? RANK_EXTRA_TWO : RANK_EXTRA))
    .height(TILE_HIGH)
  }

  private paint(): void {
    const shown = number(this.rank);
    this.painter.font = RANK_NUMERAL_TYPE.toString() + 'vp sans-serif';
    this.painter.fillStyle = RANK_FILL;
    this.painter.fillText(shown, 0, RANK_NUMERAL_HIGH * 0.8);
    this.painter.lineWidth = RANK_STROKE;
    this.painter.strokeStyle = RANK_STROKE_INK;
    this.painter.strokeText(shown, 0, RANK_NUMERAL_HIGH * 0.8);
  }
}

/**
 * Continue Watching tile: art with a progress bar, and info and menu buttons under the art.
 * `browse/Rows.kt:311-424`.
 *
 * The bar is drawn, so how far in the viewer is has to be said: "a reader otherwise hears the same
 * words for a title half watched and one not started" (`browse/Rows.kt:330-331`).
 */
@ComponentV2
export struct ContinueCard {
  @Require @Param tile: Tile;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Event open: () => void = () => {};
  @Event openInfo: () => void = () => {};
  @Event remove: () => void = () => {};

  build() {
    Column() {
      Stack({ alignContent: Alignment.BottomStart }) {
        Art({
          url: tall(this.tile.images),
          wide: TILE_WIDE,
          high: CONTINUE_HIGH,
          pictures: this.pictures,
        })
        Watched({ percent: this.tile.percent ?? 0 })
      }
      .width('100%')
      .height(CONTINUE_HIGH)
      .onClick(() => this.open())
      .accessibilityGroup(true)
      .accessibilityText(this.watched())

      Row() {
        Stack() {
          Image(ICON_INFO)
            .width(20)
            .height(20)
            .fillColor(INK)
            .accessibilityLevel(SKIP_ALL)
        }
        .width(CONTINUE_BUTTON)
        .height(CONTINUE_BUTTON)
        .borderRadius(round(999, CONTINUE_BUTTON))
        .onClick(() => this.openInfo())
        .accessibilityGroup(true)
        .accessibilityText(word(this.seen, 'common.more_info'))

        Blank()

        Stack() {
          Image(ICON_MORE)
            .width(20)
            .height(20)
            .fillColor(INK)
            .accessibilityLevel(SKIP_ALL)
        }
        .width(CONTINUE_BUTTON)
        .height(CONTINUE_BUTTON)
        .borderRadius(round(999, CONTINUE_BUTTON))
        .accessibilityGroup(true)
        .accessibilityText(word(this.seen, 'say.more_options'))
        // The one-argument form, which opens on a tap of the node it is bound to. ASSUMED: the
        // attribute is `bindMenu(content: Array<MenuElement> | CustomBuilder)` and `MenuElement`
        // carries `value` and `action`; neither was read off the reference here. The Kotlin's
        // `DropdownMenu` is held open by state of its own, which this form does not need.
        .bindMenu(this.away())
      }
      .width('100%')
      .height(CONTINUE_STRIP_HIGH)
      .padding({ left: 4, right: 4 })
      .backgroundColor(RAISED)
    }
    .width(TILE_WIDE)
    .backgroundColor(RAISED)
    .borderRadius(CORNER_CARD)
    .clip(true)
  }

  /** The one thing that menu offers: taking this title out of the row. `browse/Rows.kt:407-419`. */
  private away(): MenuElement[] {
    return [
      {
        value: word(this.seen, 'row.continue.remove'),
        action: () => this.remove(),
      },
    ];
  }

  private watched(): string {
    const percent = this.tile.percent;
    if (percent === undefined) {
      return this.tile.name;
    }
    const held = percent < 0 ? 0 : (percent > 100 ? 100 : percent);
    return fill(fill(word(this.seen, 'say.watched'), 'title', this.tile.name), 'percent', held);
  }
}
