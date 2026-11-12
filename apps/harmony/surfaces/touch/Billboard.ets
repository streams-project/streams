// The billboard: a tall card of a title's picture with its logo, a line of tags and two buttons.
// A port of `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Billboard.kt`.
//
// Reference: 381.7 x 536.2, logo about 70 percent wide within 108, tags 14 with the kind in bold,
// two 48 buttons 16 apart, 16 in, 13.7 from the bottom (`browse/Billboard.kt:83-88`). Our brand
// rounds the card and corners both buttons from `brand.json`.
//
// Exactly one row sits under it on the first screen, and the picture is a STILL and not a video:
// both are measured (`surfaces/touch/README.md`).
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
// `LengthMetrics` is how a layout mirrors: `LocalizedPadding`, `LocalizedMargin` and
// `LocalizedBorderRadiuses` take `start`/`end` (and `topStart`/`topEnd`) rather than left and right,
// and their fields are typed `LengthMetrics` with no plain-number branch, so a bare number in one of
// them is refused. The reference's RTL guide is explicit: "use the generalized direction terms start
// and end as parameter types instead of absolute terms".
import { LengthMetrics } from '@kit.ArkUI';
import { image } from '@kit.ImageKit';
import { Tile } from '../../contract/contract';
import { Pictures, Seen } from '../../ports/Ports';
import { Art } from './Pictures';
import { isSeries, logo, tall } from './Pick';
import { word } from './Words';
import { READ_ANYWAY, SKIP_ALL } from './Heard';
import {
  argb, BLACK, CARD_PLATE, CORNER_BUTTON, CORNER_PANEL, INK, round, SECOND_FILL, TAG_INK,
} from './Theme';
import { ICON_CHECK, ICON_PLAY, ICON_PLUS } from './Icons';
import {
  CARD_BOTTOM, CARD_LOGO_GAP, CARD_LOGO_HIGH, CARD_LOGO_SHARE, CARD_NAME_TYPE, CARD_PAD,
  CARD_RATIO, CARD_SHARE, CARD_SIDE, CARD_TAGS_GAP, CARD_TAGS_MOST, CARD_TAGS_TYPE,
  WIDE_BUTTON_GAP, WIDE_BUTTON_HIGH, WIDE_BUTTON_ICON, WIDE_BUTTON_TYPE,
} from './Sizes';
import { WidthClass, widthClass } from './Sizes';

@ComponentV2
export struct Billboard {
  @Require @Param tile: Tile;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  /** The window, so the card can hold its share of the height on a wide screen. */
  @Require @Param screenWide: number;
  @Require @Param screenHigh: number;
  @Param saved: boolean = false;
  @Event play: () => void = () => {};
  @Event list: () => void = () => {};
  @Event open: () => void = () => {};
  /** Handed the picture's main colour, for the page behind the card. */
  @Event tint: (picture: image.PixelMap) => void = () => {};

  build() {
    Stack({ alignContent: Alignment.Top }) {
      Stack({ alignContent: Alignment.Bottom }) {
        Art({
          url: tall(this.tile.images),
          wide: this.wide(),
          high: this.high(),
          // The page behind the card takes this picture's main colour, so the pixels are read back.
          pixels: true,
          pictures: this.pictures,
          tint: (picture: image.PixelMap) => this.tint(picture),
        })

        // The fade the words sit on. Transparent to 45 percent, then black at 0.88
        // (`browse/Billboard.kt:81`).
        Stack()
          .width('100%')
          .height('100%')
          .linearGradient({
            angle: 180,
            colors: [[argb(BLACK, 0), 0.45], [argb(BLACK, 0.88), 1]],
          })
          .accessibilityLevel(SKIP_ALL)

        Column() {
          if (logo(this.tile.images) !== null) {
            Art({
              url: logo(this.tile.images),
              wide: Math.round(this.wide() * CARD_LOGO_SHARE),
              high: CARD_LOGO_HIGH,
              pictures: this.pictures,
            })
              .width(this.wide() * CARD_LOGO_SHARE)
              .height(CARD_LOGO_HIGH)
          } else {
            Text(this.tile.name)
              .fontSize(CARD_NAME_TYPE)
              .fontWeight(FontWeight.Bolder)
              .fontColor(INK)
              .textAlign(TextAlign.Center)
              .maxLines(2)
          }

          Stack().height(CARD_LOGO_GAP)

          // The title's kind in bold, then its tags, each apart by a dot.
          // `browse/Billboard.kt:181-197`.
          Text() {
            Span(word(this.seen, isSeries(this.tile) ? 'title.kind.series' : 'title.kind.film'))
              .fontWeight(FontWeight.Bold)
              .fontColor(INK)
            Span(this.tags())
          }
          .fontSize(CARD_TAGS_TYPE)
          .fontColor(TAG_INK)
          .textAlign(TextAlign.Center)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
          .padding({ start: LengthMetrics.vp(8), end: LengthMetrics.vp(8) })

          Stack().height(CARD_TAGS_GAP)

          Row({ space: WIDE_BUTTON_GAP }) {
            this.button(
              word(this.seen, 'common.play'),
              ICON_PLAY,
              INK,
              BLACK,
              () => this.play(),
            )
            this.button(
              word(this.seen, 'list.add'),
              this.saved ? ICON_CHECK : ICON_PLUS,
              argb(SECOND_FILL, 0.9),
              INK,
              () => this.list(),
            )
          }
          .width('100%')
        }
        .width('100%')
        .padding({
          start: LengthMetrics.vp(CARD_PAD),
          end: LengthMetrics.vp(CARD_PAD),
          bottom: LengthMetrics.vp(CARD_BOTTOM),
        })
        .alignItems(HorizontalAlign.Center)
      }
      .width(this.wide())
      .height(this.high())
      .backgroundColor(CARD_PLATE)
      .borderRadius(CORNER_PANEL)
      .clip(true)
      .border({ width: 1, color: argb(INK, 0.28) })
      .onClick(() => this.open())
      // "The card is one control that opens the title. Its picture, its logo and its tag line carry
      // the same name between them, so the name is said once, here, and the two buttons inside it
      // are reached separately with their own words" (`browse/Billboard.kt:121-124`).
      .accessibilityGroup(true)
      .accessibilityText(this.tile.name)
    }
    .width('100%')
    .padding({ start: LengthMetrics.vp(CARD_SIDE), end: LengthMetrics.vp(CARD_SIDE) })
  }

  /**
   * One of the billboard's two buttons: 48 high, cornered by `brand.json` `button_radius`.
   * `browse/Billboard.kt:199-226`.
   *
   * Marked to be read even though the card around it is one group: a button that a reader could not
   * reach would leave Play unreachable on the first screen of the app.
   */
  @Builder
  button(says: string, icon: Resource, fill: string, ink: string, go: () => void) {
    Row() {
      Image(icon)
        .width(WIDE_BUTTON_ICON)
        .height(WIDE_BUTTON_ICON)
        .fillColor(ink)
        .accessibilityLevel(SKIP_ALL)
      Stack().width(6)
      Text(says)
        .fontSize(WIDE_BUTTON_TYPE)
        .fontWeight(FontWeight.Medium)
        .fontColor(ink)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })
    }
    .layoutWeight(1)
    .height(WIDE_BUTTON_HIGH)
    .padding({ start: LengthMetrics.vp(8), end: LengthMetrics.vp(8) })
    .backgroundColor(fill)
    .borderRadius(round(CORNER_BUTTON, WIDE_BUTTON_HIGH))
    .justifyContent(FlexAlign.Center)
    .alignItems(VerticalAlign.Center)
    .onClick(go)
    // The word is drawn in the button, so the button gathers it rather than being given a second
    // copy from a key of its own (`browse/Billboard.kt:207-210`).
    .accessibilityGroup(true)
    .accessibilityLevel(READ_ANYWAY)
  }

  /** The tags after the kind, each apart by a dot, at most four. `browse/Billboard.kt:188`. */
  private tags(): string {
    const tags = this.tile.tags;
    if (tags === undefined) {
      return '';
    }
    let made = '';
    const most = tags.length < CARD_TAGS_MOST ? tags.length : CARD_TAGS_MOST;
    for (let at = 0; at < most; at++) {
      made += '  •  ' + tags[at];
    }
    return made;
  }

  /**
   * The card's size.
   *
   * "A phone reaches this share by filling its width; a wide screen does not... So above the
   * expanded breakpoint the height is held to this share and the width follows the ratio"
   * (`browse/Billboard.kt:65-68`). The number the share is derived from and the reading it came
   * from are both in `Sizes.ets`.
   */
  private high(): number {
    const filled = this.screenWide - CARD_SIDE * 2;
    const byWidth = filled / CARD_RATIO;
    if (widthClass(this.screenWide) !== WidthClass.EXPANDED) {
      return byWidth;
    }
    const byShare = this.screenHigh * CARD_SHARE;
    return byWidth < byShare ? byWidth : byShare;
  }

  private wide(): number {
    const filled = this.screenWide - CARD_SIDE * 2;
    const byRatio = this.high() * CARD_RATIO;
    return filled < byRatio ? filled : byRatio;
  }
}
