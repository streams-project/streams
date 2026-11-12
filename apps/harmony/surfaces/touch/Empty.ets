// What a page shows before it has anything. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Empty.kt`.
//
// That file records what changed and why: "Until today this surface drew its two bars and nothing
// else. The reference, measured on 2026-11-11, draws a row's name and its tiles as flat rounded
// rectangles at the final 119 x 166.6 dp, with no shimmer, and for a row further down a plain bar
// where the name goes. That is what is here, and that is all that is here: the reference shows no
// message, no retry and no sign of waiting while a page is empty, so neither does this."
//
// Nothing here says anything to a reader, because there is nothing here to say: it is the shape of
// a page that has not arrived. The whole block is marked as decoration.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
// `LengthMetrics` is how a layout mirrors: `LocalizedPadding`, `LocalizedMargin` and
// `LocalizedBorderRadiuses` take `start`/`end` (and `topStart`/`topEnd`) rather than left and right,
// and their fields are typed `LengthMetrics` with no plain-number branch, so a bare number in one of
// them is refused. The reference's RTL guide is explicit: "use the generalized direction terms start
// and end as parameter types instead of absolute terms".
import { LengthMetrics } from '@kit.ArkUI';
import { SKIP_ALL } from './Heard';
import { CORNER_CARD, RAISED } from './Theme';
import {
  BAR_PLACEHOLDER_HIGH, ROW_GAP, ROW_NAME_GAP, ROW_NAME_HIGH, ROW_START, TILE_GAP, TILE_HIGH,
  TILE_PITCH, TILE_WIDE, WAITING_ROWS,
} from './Sizes';

/**
 * A page with nothing in it yet: names as bars, tiles flat at their final size. Nothing moves, as
 * nothing moves on the reference.
 */
@ComponentV2
export struct WaitingRows {
  /** The window's width, so the row holds as many tiles as it really would. */
  @Require @Param screenWide: number;

  build() {
    Column() {
      Repeat<number>(this.rows())
        .each((row: RepeatItem<number>) => {
          Column() {
            // The row's name as a plain bar, measured off the capture itself: 30.8 high and as wide
            // as the row (`browse/Empty.kt:38-45`).
            Stack({ alignContent: Alignment.Start }) {
              Stack()
                .width('100%')
                .height(BAR_PLACEHOLDER_HIGH)
                .backgroundColor(RAISED)
                .borderRadius(CORNER_CARD)
            }
            .width('100%')
            .height(ROW_NAME_HIGH)
            .padding({ start: LengthMetrics.vp(ROW_START), end: LengthMetrics.vp(ROW_START) })

            Stack().height(ROW_NAME_GAP)

            Row({ space: TILE_GAP }) {
              // As many tiles as the row holds, the last one cut by the edge as the reference's is:
              // "the screen less the row's 12.1 dp start, over the 124.9 dp tile pitch, rounded up.
              // On the reference's 421.5 dp phone that is four -- three full and a cut fourth,
              // which is the measured 3.29 visible" (`browse/Empty.kt:78-81`).
              Repeat<number>(this.tiles())
                .each((tile: RepeatItem<number>) => {
                  Stack()
                    .width(TILE_WIDE)
                    .height(TILE_HIGH)
                    .backgroundColor(RAISED)
                    .borderRadius(CORNER_CARD)
                })
                .key((held: number, index: number) => index.toString())
            }
            .width('100%')
            .padding({ start: LengthMetrics.vp(ROW_START) })
          }
          .width('100%')
          .padding({ bottom: ROW_GAP })
          .alignItems(HorizontalAlign.Start)
        })
        .key((held: number, index: number) => index.toString())
    }
    .width('100%')
    .alignItems(HorizontalAlign.Start)
    .accessibilityLevel(SKIP_ALL)
  }

  /** Rows a phone screen holds: 936.6 of screen over a 217.0 row pitch is 4.3. `browse/Empty.kt:35-36`. */
  private rows(): number[] {
    const held: number[] = [];
    for (let at = 0; at < WAITING_ROWS; at++) {
      held.push(at);
    }
    return held;
  }

  /** `browse/Empty.kt:54`. */
  private tiles(): number[] {
    const across = Math.ceil((this.screenWide - ROW_START) / TILE_PITCH);
    const most = across < 1 ? 1 : across;
    const held: number[] = [];
    for (let at = 0; at < most; at++) {
      held.push(at);
    }
    return held;
  }
}
