// The top of a main page, pinned, and the chip row under it. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Header.kt`.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
// `LengthMetrics` is how a layout mirrors: `LocalizedPadding`, `LocalizedMargin` and
// `LocalizedBorderRadiuses` take `start`/`end` (and `topStart`/`topEnd`) rather than left and right,
// and their fields are typed `LengthMetrics` with no plain-number branch, so a bare number in one of
// them is refused. The reference's RTL guide is explicit: "use the generalized direction terms start
// and end as parameter types instead of absolute terms".
import { LengthMetrics } from '@kit.ArkUI';
import { Pictures, Seen } from '../../ports/Ports';
import { Art } from './Pictures';
import { fill, word } from './Words';
import { SKIP_ALL } from './Heard';
import { ACCENT, argb, CHIP_INK, CORNER_BUTTON, CORNER_CHIP, CORNER_CHIP_END, INK, round } from './Theme';
import { ICON_BACK, ICON_BELL, ICON_CARET_DOWN } from './Icons';
import {
  BAR_BOX, BAR_BOX_EDGE, BAR_ICON, CHIP_EDGE, CHIP_GAP, CHIP_HIGH, CHIP_PAD, CHIP_ROW, CHIP_TYPE,
  MARK, OUTLINE_HIGH, PAGE_NAME_TYPE, TOP_BAR,
} from './Sizes';

/**
 * The top bar: the brand's mark and the page's name at the start, the bell at the end.
 *
 * Reference: mark box 48 x 44 at 3.9, name from 51.9, 44 buttons, drawn at 48 because that is the
 * smallest a finger should be asked for (`browse/Header.kt:52-54`). On a page reached from a chip
 * the mark gives way to a back arrow, as the reference's does.
 */
@ComponentV2
export struct TopBar {
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  /** The page's own name, already translated by the server. */
  @Require @Param name: string;
  /** True on a page that was reached from a chip, where the mark becomes a back arrow. */
  @Param back: boolean = false;
  @Param unread: number = 0;
  @Param hasBell: boolean = true;
  @Event goBack: () => void = () => {};
  @Event goBell: () => void = () => {};

  build() {
    Row() {
      // Either a back arrow, which is a control and needs words, or the brand's mark, which is
      // decoration: "the page's own name is drawn next to it and says the same thing"
      // (`browse/Header.kt:70-72`).
      Stack() {
        if (this.back) {
          Image(ICON_BACK)
            .width(BAR_ICON)
            .height(BAR_ICON)
            .fillColor(INK)
            .scale(this.seen.rtl ? { x: -1 } : { x: 1 })
            .accessibilityLevel(SKIP_ALL)
        } else {
          // The mark comes from the server, not from resources (`browse/Header.kt:87`), so it is a
          // picture like any other and goes through the port.
          Art({
            url: this.seen.server.length === 0 ? null : this.seen.server + '/mark.svg',
            wide: MARK,
            high: MARK,
            pictures: this.pictures,
          })
            .width(MARK)
            .height(MARK)
        }
      }
      .width(BAR_BOX)
      .height(BAR_BOX)
      .margin({ start: LengthMetrics.vp(BAR_BOX_EDGE) })
      .onClick(() => {
        if (this.back) {
          this.goBack();
        }
      })
      .accessibilityGroup(this.back)
      .accessibilityText(this.back ? word(this.seen, 'common.back') : '')
      .accessibilityLevel(this.back ? 'auto' : SKIP_ALL)

      Text(this.name)
        .fontSize(PAGE_NAME_TYPE)
        .fontWeight(FontWeight.Bold)
        .fontColor(INK)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })
        .layoutWeight(1)
        .padding({ end: LengthMetrics.vp(this.hasBell ? 0 : 8) })

      if (this.hasBell) {
        Stack({ alignContent: Alignment.TopEnd }) {
          Image(ICON_BELL)
            .width(BAR_ICON)
            .height(BAR_ICON)
            .fillColor(INK)
            .accessibilityLevel(SKIP_ALL)

          if (this.unread > 0) {
            // The count is drawn on the bell, so the bell says it (`browse/Header.kt:100-105`).
            Text(this.unread > 9 ? '9+' : this.unread.toString())
              .fontSize(10)
              .fontWeight(FontWeight.Bold)
              .fontColor(INK)
              .backgroundColor(ACCENT)
              .borderRadius(round(CORNER_BUTTON, 14))
              .padding({ left: 4, right: 4, top: 1, bottom: 1 })
              .maxLines(1)
              .margin({ top: 8, right: 6 })
              .accessibilityLevel(SKIP_ALL)
          }
        }
        .width(BAR_BOX)
        .height(BAR_BOX)
        .onClick(() => this.goBell())
        .accessibilityGroup(true)
        .accessibilityText(this.bellSays())

        Stack().width(4)
      }
    }
    .width('100%')
    .height(TOP_BAR)
    .alignItems(VerticalAlign.Center)
  }

  private bellSays(): string {
    if (this.unread > 0) {
      return fill(word(this.seen, 'say.notifications_unread'), 'n', this.unread);
    }
    return word(this.seen, 'notifications.title');
  }
}

/** One chip of the row: what it says, whether it is chosen, and whether it drops a menu. */
export class Chip {
  readonly word: string;
  readonly chosen: boolean;
  readonly drops: boolean;
  readonly go: () => void;

  constructor(word: string, chosen: boolean, drops: boolean, go: () => void) {
    this.word = word;
    this.chosen = chosen;
    this.drops = drops;
    this.go = go;
  }
}

/**
 * The chip row under the top bar, as the reference's: 49 chips from 16 in, 6 apart, 16 padding, a
 * tenth of white under a 1 edge of a tenth of white, words 15 medium (`browse/Header.kt:142-148`).
 *
 * "The row reads as one group: the first chip's outer side and the last chip's are round
 * (`brand.json` `chip_end_radius`), every other corner takes `chip_radius`. Start and end follow the
 * reading direction, so the group mirrors in Arabic and Hebrew."
 *
 * That mirroring is the whole reason the corners are set with `LocalizedBorderRadiuses` and not with
 * `topLeft`/`topRight`: the reference's RTL guide says to "use the generalized direction terms start
 * and end as parameter types instead of absolute terms", and `borderRadius` is on its list of
 * universal attributes that mirror.
 */
@ComponentV2
export struct ChipRow {
  @Require @Param seen: Seen;
  @Require @Param chips: Chip[];
  /** 1 while the row is shown, 0 once the page has scrolled it away. */
  @Param shown: number = 1;

  build() {
    List({ space: CHIP_GAP }) {
      Repeat<Chip>(this.chips)
        .each((held: RepeatItem<Chip>) => {
          ListItem() {
            this.one(held.item, held.index);
          }
        })
        // By place, not by word: "the row always opens at its first chip, however the menu
        // arrives. The word is the key -- two chips of one row never carry the same one"
        // (`browse/Header.kt:161-164`).
        .key((chip: Chip, index: number) => chip.word)
        .virtualScroll()
    }
    .listDirection(Axis.Horizontal)
    .scrollBar(BarState.Off)
    .width('100%')
    .height(CHIP_ROW)
    .contentStartOffset(CHIP_EDGE)
    .opacity(this.shown)
  }

  @Builder
  one(chip: Chip, index: number) {
    Row() {
      Text(chip.word)
        .fontSize(CHIP_TYPE)
        .fontWeight(FontWeight.Medium)
        .fontColor(this.ink(chip))
        .maxLines(1)

      if (chip.drops) {
        Stack().width(4)
        Image(ICON_CARET_DOWN)
          .width(18)
          .height(18)
          .fillColor(this.ink(chip))
          .accessibilityLevel(SKIP_ALL)
      }
    }
    .height(this.high())
    .padding({ start: LengthMetrics.vp(CHIP_PAD), end: LengthMetrics.vp(CHIP_PAD) })
    .backgroundColor(this.fillOf(chip))
    .borderRadius(this.corners(index))
    .border({ width: 1, color: this.edgeOf(chip) })
    .onClick(chip.go)
    // A chip draws its own word, and the caret beside it is inside this node, so the word is
    // gathered rather than given again (`browse/Header.kt:199-200`).
    .accessibilityGroup(true)
  }

  /** A single chip that drops a menu is drawn as an outline, and shorter. `browse/Header.kt:151-152`. */
  private outline(): boolean {
    return this.chips.length === 1 && this.chips[0].drops;
  }

  private high(): number {
    return this.outline() ? OUTLINE_HIGH : CHIP_HIGH;
  }

  /**
   * The group's corners. `browse/Header.kt:169-180`: an outline chip is cornered all round; in a
   * group the first chip's outer side and the last chip's are round and every other corner takes
   * the chip radius.
   */
  private corners(index: number): LocalizedBorderRadiuses {
    const inner = LengthMetrics.vp(CORNER_CHIP);
    const outer = LengthMetrics.vp(round(CORNER_CHIP_END, this.high()));
    if (this.outline()) {
      return { topStart: inner, topEnd: inner, bottomStart: inner, bottomEnd: inner };
    }
    const first = index === 0;
    const last = index === this.chips.length - 1;
    return {
      topStart: first ? outer : inner,
      bottomStart: first ? outer : inner,
      topEnd: last ? outer : inner,
      bottomEnd: last ? outer : inner,
    };
  }

  private fillOf(chip: Chip): string {
    if (chip.chosen) {
      return INK;
    }
    return this.outline() ? argb(INK, 0) : argb(INK, 0.1);
  }

  private edgeOf(chip: Chip): string {
    if (chip.chosen) {
      return argb(INK, 0);
    }
    return this.outline() ? argb(INK, 0.4) : argb(INK, 0.1);
  }

  private ink(chip: Chip): string {
    return chip.chosen ? '#000000' : CHIP_INK;
  }
}
