// The bar at the foot of every main page, floating as the reference's does. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/parts/BottomBar.kt`.
//
// "It sits in the reading direction, so it turns with a right to left language. All shapes and
// corners come from brand.json, never hardcoded" (`parts/BottomBar.kt:49-52`).
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
// `LengthMetrics` is how a layout mirrors: `LocalizedPadding`, `LocalizedMargin` and
// `LocalizedBorderRadiuses` take `start`/`end` (and `topStart`/`topEnd`) rather than left and right,
// and their fields are typed `LengthMetrics` with no plain-number branch, so a bare number in one of
// them is refused. The reference's RTL guide is explicit: "use the generalized direction terms start
// and end as parameter types instead of absolute terms".
import { LengthMetrics } from '@kit.ArkUI';
import { Profile } from '../../contract/contract';
import { Pictures } from '../../ports/Ports';
import { Face } from './Face';
import { SKIP_ALL } from './Heard';
import { argb, BAR_CHOSEN, BAR_FILL, BAR_IDLE_INK, CORNER_BUTTON, INK, round } from './Theme';
import {
  BAR_FACE, BAR_HIGH, BAR_ITEM_HIGH, BAR_ITEM_PAD, BAR_ITEM_WIDE, BAR_ITEM_WIDE_FULL, BAR_PAD,
  BAR_WORD_GAP, BAR_WORD_TYPE,
} from './Sizes';

/**
 * One place on the bar: its icon, its word, whether it is the page in view, and where it goes.
 * The bar shows the profile's face in place of an icon, as the reference shows the viewer's.
 */
export class BarItem {
  readonly icon: Resource | null;
  readonly word: string;
  readonly here: boolean;
  readonly face: Profile | null;
  readonly go: () => void;

  constructor(icon: Resource | null, word: string, here: boolean, face: Profile | null, go: () => void) {
    this.icon = icon;
    this.word = word;
    this.here = here;
    this.face = face;
    this.go = go;
  }
}

@ComponentV2
export struct BottomBar {
  @Require @Param items: BarItem[];
  @Require @Param server: string;
  @Require @Param pictures: Pictures;

  build() {
    Row() {
      Repeat<BarItem>(this.items)
        .each((held: RepeatItem<BarItem>) => {
          this.one(held.item);
        })
        .key((item: BarItem, index: number) => item.word + '|' + index.toString())
    }
    .height(BAR_HIGH)
    .padding({ start: LengthMetrics.vp(BAR_PAD), end: LengthMetrics.vp(BAR_PAD) })
    .backgroundColor(BAR_FILL)
    .borderRadius(round(CORNER_BUTTON, BAR_HIGH))
    .border({ width: 1, color: argb(INK, 0.1) })
    .justifyContent(FlexAlign.Center)
    .alignItems(VerticalAlign.Center)
  }

  @Builder
  one(item: BarItem) {
    Column() {
      if (item.face !== null) {
        Face({
          person: item.face,
          server: this.server,
          pictures: this.pictures,
          box: BAR_FACE,
          corner: 6,
          letterType: 13,
        })
      } else if (item.icon !== null) {
        Image(item.icon)
          .width(BAR_FACE)
          .height(BAR_FACE)
          .fillColor(item.here ? INK : BAR_IDLE_INK)
          .accessibilityLevel(SKIP_ALL)
      }

      Stack().height(BAR_WORD_GAP)

      Text(item.word)
        .fontSize(BAR_WORD_TYPE)
        .fontColor(item.here ? INK : BAR_IDLE_INK)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })
    }
    .constraintSize({ minWidth: this.itemWide() })
    .height(BAR_ITEM_HIGH)
    .padding({ start: LengthMetrics.vp(BAR_ITEM_PAD), end: LengthMetrics.vp(BAR_ITEM_PAD) })
    .margin({ start: LengthMetrics.vp(2), end: LengthMetrics.vp(2) })
    .backgroundColor(item.here ? BAR_CHOSEN : argb(BAR_FILL, 0))
    .borderRadius(round(CORNER_BUTTON, BAR_ITEM_HIGH))
    .justifyContent(FlexAlign.Center)
    .onClick(item.go)
    // "Each place on the bar draws its own word under its icon. The face shown in place of an icon
    // is decoration, so what is gathered here is the word and nothing else"
    // (`parts/BottomBar.kt:77-79`).
    .accessibilityGroup(true)
  }

  /** Narrower once the server offers a fifth place. `parts/BottomBar.kt:57`. */
  private itemWide(): number {
    return this.items.length > 4 ? BAR_ITEM_WIDE_FULL : BAR_ITEM_WIDE;
  }
}
