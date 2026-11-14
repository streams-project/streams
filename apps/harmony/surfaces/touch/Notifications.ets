// The notifications feed. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/notifications/Notifications.kt`, which
// is the only measurement of this screen the project has: there is no television notifications
// screen and no capture of Netflix's own, so every figure here is that file's and nothing in this
// one was chosen. They are in `Sizes.ets` as `NOTICE_*` with the line each came from, and
// `platform/tools/touch_check.py` reads both sides.
//
// `Root.ets` listed this as one of the five routes that draw nothing. It is now four.
//
// `NotificationsScreen` and not `Notifications`: the contract exports an interface of that name,
// which this file reads, and a struct would shadow it -- the same trap `Pick.ets` names for the
// contract's `Image` and `Search.ets` for ArkUI's `Search`.
//
// TWO PLACES THIS DIFFERS FROM THE KOTLIN, both on purpose and both the house rule rather than a
// second opinion:
//
//   1. NO ENGLISH. The Kotlin writes `words["notifications.title"] ?: "Notifications"` and five
//      more fallbacks like it (`:149,173,230-235`). Every one of those keys IS in `strings/en.json`
//      in all eighteen languages, so the fallbacks are dead text that only fires when the words
//      have not arrived -- and when they have not arrived, a word of English on a Japanese phone is
//      worse than a blank. `Words.ets` says so at length, so this draws the key's word or nothing.
//   2. NO SPINNER. The Kotlin draws `CircularProgressIndicator` twice (`:160,215`). This surface
//      has no measured waiting mark of its own -- `Empty.ets` draws placeholder ROWS, which is what
//      the reference does on a browse page and is not what this screen shows -- so the first read
//      draws nothing and the foot of the list draws nothing while the next page is in flight. That
//      is the `apps/roku` rule: a blank is honest and a made-up part is not. Named as an open item.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { LengthMetrics } from '@kit.ArkUI';
import { NoticeItem, Notifications } from '../../contract/contract';
import { good, readNotifications } from '../../contract/read';
import { Pictures, Seen, Shell } from '../../ports/Ports';
import { Art } from './Pictures';
import { tall, titleAddress } from './Pick';
import { word } from './Words';
import { SKIP_ALL } from './Heard';
import {
  ACCENT, BACK, CORNER_BUTTON, CORNER_CARD, CORNER_CHIP, INK, MUTED, PLATE, round,
} from './Theme';
import { ICON_BACK, ICON_BELL } from './Icons';
import {
  BAR_BOX, BAR_ICON, NOTICES_AHEAD, NOTICE_ART_GAP, NOTICE_ART_HIGH, NOTICE_ART_WIDE,
  NOTICE_BAR, NOTICE_BAR_EDGE, NOTICE_DATE_GAP, NOTICE_DATE_TYPE, NOTICE_DOT, NOTICE_DOT_GAP,
  NOTICE_EMPTY_GAP, NOTICE_EMPTY_ICON, NOTICE_EMPTY_TYPE, NOTICE_KIND_GAP, NOTICE_KIND_PAD_HIGH,
  NOTICE_KIND_PAD_SIDE, NOTICE_KIND_TYPE, NOTICE_LIST_PAD, NOTICE_NAME_TYPE, NOTICE_PAD_HIGH,
  NOTICE_PAD_SIDE, NOTICE_TITLE_TYPE,
} from './Sizes';

/**
 * The word for a notice's kind. `notifications/Notifications.kt:228-235`.
 *
 * A function and not a map: ArkTS has no index signature and the contract's `NoticeKind` is a
 * string union with an open member, so a kind the server adds tomorrow falls through to nothing
 * drawn rather than to a key that does not exist.
 */
function kindKey(kind: string): string {
  if (kind === 'coming') {
    return 'notification.coming';
  }
  if (kind === 'reminder_live') {
    return 'notification.reminder_live';
  }
  if (kind === 'new_arrival') {
    return 'notification.new_arrival';
  }
  if (kind === 'new_season') {
    return 'notification.new_season';
  }
  if (kind === 'new_episode') {
    return 'notification.new_episode';
  }
  return '';
}

/** Which kinds wear the accent rather than the plate. `notifications/Notifications.kt:277`. */
function kindIsAccent(kind: string): boolean {
  return kind === 'reminder_live' || kind === 'new_arrival';
}

@ComponentV2
export struct NotificationsScreen {
  @Require @Param shell: Shell;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;

  @Local items: NoticeItem[] = [];
  @Local hasMore: boolean = false;
  /** False until the first answer, so an empty list is not drawn before there is one. */
  @Local answered: boolean = false;

  /** The cursor of the page after the one held, and whether one is in flight. */
  private cursor: string = '';
  private reading: boolean = false;

  aboutToAppear(): void {
    this.readPage('', false);
  }

  /** `notifications/Notifications.kt:76-87`. */
  private readPage(cursor: string, append: boolean): void {
    if (this.reading) {
      return;
    }
    this.reading = true;
    const path = cursor.length === 0
      ? '/v1/notifications'
      : '/v1/notifications?cursor=' + encodeURIComponent(cursor);
    this.shell.ask(path, (status: number, text: string): void => {
      this.reading = false;
      this.answered = true;
      if (!good(status)) {
        return;
      }
      const read: Notifications | null = readNotifications(text);
      if (read === null) {
        return;
      }
      this.cursor = read.cursor ?? '';
      this.hasMore = read.has_more;
      this.items = append ? this.items.concat(read.items) : read.items;
    });
  }

  /**
   * The next page, once the viewer is within three of the end.
   * `notifications/Notifications.kt:102-113`, which reads the same trigger off the list's own
   * visible range. ArkUI reports a scroll's index on the `List`, so the test is written where the
   * index arrives rather than derived from a state of its own.
   */
  private reached(last: number): void {
    if (this.hasMore && !this.reading && this.cursor.length > 0
      && last >= this.items.length - NOTICES_AHEAD) {
      this.readPage(this.cursor, true);
    }
  }

  /**
   * A notice opened: it stops being unread here and on the server, and the viewer goes to the
   * title. `notifications/Notifications.kt:190-205`.
   *
   * The read is told to the server up to this notice's id, the count the bell draws is refreshed
   * through the host, and the row is marked here so the dot goes at the tap rather than at the
   * next answer.
   */
  private opened(item: NoticeItem): void {
    if (item.unread) {
      const body = '{"up_to":"' + item.id + '"}';
      this.shell.send('POST', '/v1/notifications/read', body, (status: number, text: string): void => {
        this.shell.refreshNotifications();
      });
      const held: NoticeItem[] = [];
      for (const one of this.items) {
        held.push(one.id === item.id ? this.read(one) : one);
      }
      this.items = held;
    }
    this.shell.go(titleAddress(item.tile.title));
  }

  /** The same notice, read. A new object: a `@Local` array notices an assignment, not a write. */
  private read(item: NoticeItem): NoticeItem {
    return {
      date: item.date,
      id: item.id,
      kind: item.kind,
      tile: item.tile,
      unread: false,
    };
  }

  build() {
    Column() {
      this.bar()

      if (this.items.length > 0) {
        this.feed()
      } else if (this.answered) {
        this.nothing()
      }
      // Before the first answer, nothing. See the head of this file.
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BACK)
    .alignItems(HorizontalAlign.Start)
  }

  /** `notifications/Notifications.kt:124-155`. */
  @Builder
  bar() {
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

      Text(word(this.seen, 'notifications.title'))
        .fontSize(NOTICE_TITLE_TYPE)
        .fontWeight(FontWeight.Bold)
        .fontColor(INK)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })
        .layoutWeight(1)
    }
    .width('100%')
    .height(NOTICE_BAR)
    .padding({ start: LengthMetrics.vp(NOTICE_BAR_EDGE) })
    .alignItems(VerticalAlign.Center)
  }

  /** `notifications/Notifications.kt:161-179`: a bell beside words that already say there is none. */
  @Builder
  nothing() {
    Column() {
      Image(ICON_BELL)
        .width(NOTICE_EMPTY_ICON)
        .height(NOTICE_EMPTY_ICON)
        .fillColor(MUTED)
        .accessibilityLevel(SKIP_ALL)

      Stack().height(NOTICE_EMPTY_GAP)

      Text(word(this.seen, 'notifications.empty'))
        .fontSize(NOTICE_EMPTY_TYPE)
        .fontColor(MUTED)
    }
    .width('100%')
    .layoutWeight(1)
    .justifyContent(FlexAlign.Center)
    .alignItems(HorizontalAlign.Center)
  }

  /** `notifications/Notifications.kt:180-219`. */
  @Builder
  feed() {
    List() {
      Repeat<NoticeItem>(this.items)
        .each((held: RepeatItem<NoticeItem>) => {
          ListItem() {
            this.notice(held.item)
          }
        })
        .key((item: NoticeItem, index: number) => item.id)
        .virtualScroll()
    }
    .width('100%')
    .layoutWeight(1)
    .scrollBar(BarState.Off)
    .padding({ top: NOTICE_LIST_PAD, bottom: NOTICE_LIST_PAD })
    .onScrollIndex((first: number, last: number) => this.reached(last))
  }

  /**
   * One notice. `notifications/Notifications.kt:222-320`.
   *
   * It is one control: the kind, the title's name and the date are what a reader wants in that
   * order and they are all drawn, so the row gathers its own words rather than being given a second
   * copy of them. Whether it has been read is a dot with no words, so it is said as the control's
   * state (`:245`), and the poster is decoration because the name is drawn beside it (`:257-259`).
   */
  @Builder
  notice(item: NoticeItem) {
    Row() {
      Stack() {
        Art({
          url: tall(item.tile.images),
          wide: NOTICE_ART_WIDE,
          high: NOTICE_ART_HIGH,
          pictures: this.pictures,
        })
      }
      .width(NOTICE_ART_WIDE)
      .height(NOTICE_ART_HIGH)
      .backgroundColor(PLATE)
      .borderRadius(CORNER_CARD)
      .clip(true)
      .accessibilityLevel(SKIP_ALL)

      Stack().width(NOTICE_ART_GAP)

      Column() {
        if (kindKey(item.kind).length > 0) {
          Text(word(this.seen, kindKey(item.kind)))
            .fontSize(NOTICE_KIND_TYPE)
            .fontWeight(FontWeight.Medium)
            .fontColor(INK)
            .backgroundColor(kindIsAccent(item.kind) ? ACCENT : PLATE)
            .borderRadius(CORNER_CHIP)
            .padding({
              left: NOTICE_KIND_PAD_SIDE,
              right: NOTICE_KIND_PAD_SIDE,
              top: NOTICE_KIND_PAD_HIGH,
              bottom: NOTICE_KIND_PAD_HIGH,
            })

          Stack().height(NOTICE_KIND_GAP)
        }

        Text(item.tile.name)
          .fontSize(NOTICE_NAME_TYPE)
          .fontWeight(FontWeight.Bold)
          .fontColor(INK)
          .maxLines(2)
          .textOverflow({ overflow: TextOverflow.Ellipsis })

        Stack().height(NOTICE_DATE_GAP)

        Text(item.date)
          .fontSize(NOTICE_DATE_TYPE)
          .fontColor(MUTED)
      }
      .layoutWeight(1)
      .alignItems(HorizontalAlign.Start)
      .justifyContent(FlexAlign.Center)

      if (item.unread) {
        Stack().width(NOTICE_DOT_GAP)
        Stack()
          .width(NOTICE_DOT)
          .height(NOTICE_DOT)
          .backgroundColor(ACCENT)
          .borderRadius(round(CORNER_BUTTON, NOTICE_DOT))
          .accessibilityLevel(SKIP_ALL)
      }
    }
    .width('100%')
    .padding({
      left: NOTICE_PAD_SIDE,
      right: NOTICE_PAD_SIDE,
      top: NOTICE_PAD_HIGH,
      bottom: NOTICE_PAD_HIGH,
    })
    .alignItems(VerticalAlign.Center)
    .onClick(() => this.opened(item))
    .accessibilityGroup(true)
    .accessibilityDescription(item.unread ? word(this.seen, 'say.unread') : '')
  }
}
