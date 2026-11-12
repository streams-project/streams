// The title page. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/title/Title.kt`, with its two tabs
// (`title/Episodes.kt`, `title/MoreLike.kt`).
//
// "16:9 art under a 52 bar, title or logo, facts, white play button, expandable synopsis, genres
// line, action buttons row, divider, and Episodes / More Like This tabs" (`title/Title.kt:106-110`).
//
// Three words the Kotlin writes in English and this does not: `title/Title.kt:116` has
// `"Rate"`, `:196` has `"Share"`, `:293` has `"HD"` and `:406,445` have `"… more"`, all typed into
// the surface. There is no key for any of them in `strings/en.json` (301 keys, read 2026-11-12), so
// this draws nothing where a key is missing rather than drawing English. Which keys are wanted is an
// open item for `strings/`, named in the note.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { LengthMetrics } from '@kit.ArkUI';
import { Episode, Season, SeasonSummary, Tile, TitleDetail } from '../../contract/contract';
import { commandAction, listCommand, rateCommand, remindCommand } from '../../contract/effects';
import { good, readSeason } from '../../contract/read';
import { Pictures, Seen, Shell } from '../../ports/Ports';
import { Art } from './Pictures';
import { isSeriesKey, logo, tall, titleAddress, watchAddress, wide } from './Pick';
import { counted, fill, number, word } from './Words';
import { READ_ANYWAY, SKIP_ALL } from './Heard';
import {
  ACCENT, argb, BLACK, CORNER_BUTTON, CORNER_CARD, CORNER_CHIP, CORNER_FIELD, INK, MUTED, PLATE,
  RAISED, round,
} from './Theme';
import {
  ICON_BACK, ICON_BELL, ICON_CAST, ICON_CHECK, ICON_LOVE, ICON_LOVE_FILLED, ICON_PLAY, ICON_PLUS,
  ICON_THUMB_DOWN, ICON_THUMB_DOWN_FILLED, ICON_THUMB_UP, ICON_THUMB_UP_FILLED,
} from './Icons';
import {
  BAR_BOX, BAR_ICON, BAR_ROOM, EPISODES_EDGE, EPISODE_ART_HIGH,
  EPISODE_ART_WIDE, EPISODE_GAP, EPISODE_LENGTH_TYPE, EPISODE_NAME_LINE, EPISODE_NAME_TYPE,
  EPISODE_RING, EPISODE_RING_EDGE, EPISODE_RING_ICON, EPISODE_ROW_GAP, EPISODE_TEXT_LINE,
  EPISODE_TEXT_LINES, EPISODE_TEXT_TYPE, MORE_ACROSS, MORE_EDGE_HIGH, MORE_EDGE_SIDE, MORE_GAP,
  PROGRESS_HIGH, SEASON_CHIP_GAP, SEASON_CHIP_HIGH, SEASON_CHIP_PAD, SEASON_CHIP_TYPE,
  TILE_RATIO, TITLE_ACTION_GAP, TITLE_ACTION_ICON, TITLE_ACTION_TYPE, TITLE_ACTION_WIDE,
  TITLE_ACTION_WORD_GAP, TITLE_AGE_PAD_SIDE, TITLE_AGE_PAD_TOP, TITLE_AGE_TYPE, TITLE_ART_RATIO,
  TITLE_BAR, TITLE_BODY_EDGE, TITLE_FACTS_GAP, TITLE_FACTS_TYPE, TITLE_GENRES_FEW,
  TITLE_GENRES_LINE, TITLE_GENRES_TYPE, TITLE_LOGO_HIGH, TITLE_LOGO_SHARE, TITLE_NAME_LINE,
  TITLE_NAME_TYPE, TITLE_PLAY_HIGH, TITLE_PLAY_ICON, TITLE_PLAY_TYPE, TITLE_SYNOPSIS_LINE,
  TITLE_SYNOPSIS_LINES, TITLE_SYNOPSIS_TYPE, TITLE_TAB_HIGH, TITLE_TAB_LINE,
  TITLE_TAB_PAD, TITLE_TAB_TYPE,
} from './Sizes';

@ComponentV2
export struct Title {
  @Require @Param shell: Shell;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Require @Param screenWide: number;

  /** Episodes for a series, More Like This for a film. `title/Title.kt:118-120`. */
  @Local episodesTab: boolean = false;
  @Local saved: boolean = false;
  @Local reminded: boolean = false;
  @Local rating: string = '';
  @Local rateOpen: boolean = false;
  @Local synopsisOpen: boolean = false;
  @Local genresOpen: boolean = false;
  @Local seasonNumber: number = 0;
  @Local season: Season | null = null;
  @Local seasonSheet: boolean = false;

  aboutToAppear(): void {
    this.settle();
  }

  /** The route moved to another title under this page. */
  @Monitor('seen')
  stepped(): void {
    const data = this.data();
    if (data !== null && data.title !== this.lastTitle) {
      this.settle();
    }
  }

  private lastTitle: string = '';

  private settle(): void {
    const data = this.data();
    if (data === null) {
      return;
    }
    this.lastTitle = data.title;
    this.episodesTab = isSeriesKey(data.title);
    this.saved = data.in_list === true;
    this.reminded = data.reminder_set === true;
    this.rating = data.rating ?? '';
    this.synopsisOpen = false;
    this.genresOpen = false;
    this.season = null;
    const seasons = data.seasons;
    this.seasonNumber = seasons !== undefined && seasons.length > 0 ? seasons[0].number : 0;
    if (this.seasonNumber > 0) {
      this.readSeason(this.seasonNumber);
    }
  }

  /**
   * The facts of the title the route names, and nothing else.
   *
   * `title/Title.kt:113` draws nothing at all until `seen.detail` is the route's title: a page that
   * drew the title before it would show the one the viewer came from.
   */
  private data(): TitleDetail | null {
    const detail = this.seen.detail;
    if (detail === null) {
      return null;
    }
    return detail.title === this.seen.route?.title ? detail : null;
  }

  build() {
    Column() {
      if (this.data() !== null) {
        this.bar()
        Scroll() {
          Column() {
            this.body(this.data()!)
          }
          .width('100%')
          .alignItems(HorizontalAlign.Start)
        }
        .layoutWeight(1)
        .width('100%')
        .scrollBar(BarState.Off)
        .align(Alignment.TopStart)
      }
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BLACK)
  }

  /** The pinned bar: back at the start, cast and close at the end. `title/Title.kt:134-186`. */
  @Builder
  bar() {
    Row() {
      this.barBox(ICON_BACK, word(this.seen, 'common.back'), true, () => this.shell.back())
      Blank()
      this.barBox(ICON_CAST, word(this.seen, 'say.cast'), false, () => {
        // The Kotlin leaves this empty too (`title/Title.kt:161`): casting is not built in either
        // family. The control is drawn because the reference draws it, and it says what it is.
      })
      // The reference's close is two lines, which has nothing for a reader to gather, so it is
      // given words (`title/Title.kt:176-177`).
      Stack() {
        Text('×')
          .fontSize(20)
          .fontColor(INK)
          .accessibilityLevel(SKIP_ALL)
      }
      .width(BAR_BOX)
      .height(BAR_BOX)
      .onClick(() => this.shell.back())
      .accessibilityGroup(true)
      .accessibilityText(word(this.seen, 'say.close'))
    }
    .width('100%')
    .height(TITLE_BAR)
    .padding({ start: LengthMetrics.vp(4), end: LengthMetrics.vp(4) })
    .alignItems(VerticalAlign.Center)
  }

  @Builder
  barBox(icon: Resource, says: string, turns: boolean, go: () => void) {
    Stack() {
      Image(icon)
        .width(BAR_ICON)
        .height(BAR_ICON)
        .fillColor(INK)
        .scale(turns && this.seen.rtl ? { x: -1 } : { x: 1 })
        .accessibilityLevel(SKIP_ALL)
    }
    .width(BAR_BOX)
    .height(BAR_BOX)
    .onClick(go)
    .accessibilityGroup(true)
    .accessibilityText(says)
  }

  @Builder
  body(data: TitleDetail) {
    // 16:9 art at the top.
    Stack() {
      Art({
        url: wide(data.images),
        wide: this.screenWide,
        high: Math.round(this.screenWide / TITLE_ART_RATIO),
        pictures: this.pictures,
      })
    }
    .width('100%')
    .aspectRatio(TITLE_ART_RATIO)
    .backgroundColor(PLATE)

    Column() {
      // The title's logo is its name as a picture, and the name is drawn as words further down, so
      // the logo is decoration (`title/Title.kt:215-218`).
      if (logo(data.images) !== null) {
        Art({
          url: logo(data.images),
          wide: Math.round((this.screenWide - TITLE_BODY_EDGE * 2) * TITLE_LOGO_SHARE),
          high: TITLE_LOGO_HIGH,
          pictures: this.pictures,
        })
          .width((this.screenWide - TITLE_BODY_EDGE * 2) * TITLE_LOGO_SHARE)
          .height(TITLE_LOGO_HIGH)
      } else {
        Text(data.name)
          .fontSize(TITLE_NAME_TYPE)
          .lineHeight(TITLE_NAME_LINE)
          .fontWeight(FontWeight.Bold)
          .fontColor(INK)
          .maxLines(2)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }

      this.facts(data)

      if (data.available_from !== undefined) {
        this.coming(data)
      } else {
        this.playButton(data)
      }

      this.synopsis(data)
      this.genres(data)
      this.actions(data)
    }
    .width('100%')
    .padding({
      start: LengthMetrics.vp(TITLE_BODY_EDGE),
      end: LengthMetrics.vp(TITLE_BODY_EDGE),
      top: LengthMetrics.vp(TITLE_BODY_EDGE),
      bottom: LengthMetrics.vp(TITLE_BODY_EDGE),
    })
    .alignItems(HorizontalAlign.Start)

    Stack()
      .width('100%')
      .height(1)
      .backgroundColor(argb(INK, 0.15))
      .margin({ top: 16 })

    this.tabs(data)

    if (this.episodesTab && isSeriesKey(data.title)) {
      this.episodes(data)
    } else {
      this.moreLike(data)
    }

    Stack().height(BAR_ROOM)
  }

  /** Year, age badge, seasons or length. `title/Title.kt:237-299`. */
  @Builder
  facts(data: TitleDetail) {
    Row({ space: TITLE_FACTS_GAP }) {
      if (data.year !== undefined) {
        Text(number(data.year))
          .fontSize(TITLE_FACTS_TYPE)
          .fontColor(MUTED)
      }
      if (data.age !== undefined) {
        Text(data.age.label)
          .fontSize(TITLE_AGE_TYPE)
          .fontWeight(FontWeight.Medium)
          .fontColor(argb(INK, 0.9))
          .backgroundColor(argb(INK, 0.15))
          .borderRadius(CORNER_FIELD)
          .padding({
            start: LengthMetrics.vp(TITLE_AGE_PAD_SIDE),
            end: LengthMetrics.vp(TITLE_AGE_PAD_SIDE),
            top: LengthMetrics.vp(TITLE_AGE_PAD_TOP),
            bottom: LengthMetrics.vp(TITLE_AGE_PAD_TOP),
          })
      }
      if (this.length(data).length > 0) {
        Text(this.length(data))
          .fontSize(TITLE_FACTS_TYPE)
          .fontColor(MUTED)
      }
      // `title/Title.kt:287-298` draws an "HD" badge with the letters typed into the surface. There
      // is no key for it in `strings/`, and "HD" is a mark rather than a word in most languages, so
      // what belongs here is a key saying so. Until there is one this draws nothing.
    }
    .width('100%')
    .margin({ top: 8 })
    .alignItems(VerticalAlign.Center)
  }

  /** Seasons for a series, a running time for a film. `title/Title.kt:269-285`. */
  private length(data: TitleDetail): string {
    const seasons = data.seasons;
    if (seasons !== undefined && seasons.length > 0) {
      return counted(this.seen, 'title.season', seasons.length);
    }
    const minutes = data.runtime_min;
    if (minutes === undefined) {
      return '';
    }
    if (minutes >= 60) {
      const held = fill(word(this.seen, 'title.length.hours'), 'h', Math.floor(minutes / 60));
      return fill(held, 'm', minutes % 60);
    }
    return fill(word(this.seen, 'title.length.minutes'), 'm', minutes);
  }

  /** A title that has not arrived yet, with a reminder. `title/Title.kt:301-349`. */
  @Builder
  coming(data: TitleDetail) {
    Text(fill(word(this.seen, 'gate.coming'), 'date', data.available_from!))
      .fontSize(15)
      .fontWeight(FontWeight.Medium)
      .fontColor(INK)
      .margin({ top: 10 })

    Row() {
      Image(this.reminded ? ICON_CHECK : ICON_BELL)
        .width(TITLE_PLAY_ICON)
        .height(TITLE_PLAY_ICON)
        .fillColor(INK)
        .accessibilityLevel(SKIP_ALL)
      Stack().width(8)
      Text(word(this.seen, this.reminded ? 'remind.set' : 'remind.me'))
        .fontSize(TITLE_PLAY_TYPE)
        .fontWeight(FontWeight.Bold)
        .fontColor(INK)
    }
    .width('100%')
    .height(TITLE_PLAY_HIGH)
    .margin({ top: 8 })
    .backgroundColor(this.reminded ? RAISED : ACCENT)
    .borderRadius(round(CORNER_BUTTON, TITLE_PLAY_HIGH))
    .justifyContent(FlexAlign.Center)
    .alignItems(VerticalAlign.Center)
    .onClick(() => {
      const next = !this.reminded;
      this.reminded = next;
      this.shell.act(commandAction(remindCommand(data.title, next)));
      // `title/Title.kt:319-322` also schedules a local notification through `ReminderScheduler`.
      // That is a device capability with no port, and adding one is not this surface's to do: it is
      // an open item in the note. The command reaches the server either way, so the reminder is
      // kept; what is missing is the alarm on the phone itself.
    })
    .accessibilityGroup(true)
  }

  /** `title/Title.kt:351-379`. */
  @Builder
  playButton(data: TitleDetail) {
    Row() {
      Image(ICON_PLAY)
        .width(TITLE_PLAY_ICON)
        .height(TITLE_PLAY_ICON)
        .fillColor(BLACK)
        .accessibilityLevel(SKIP_ALL)
      Stack().width(8)
      Text(word(this.seen, 'common.play'))
        .fontSize(TITLE_PLAY_TYPE)
        .fontWeight(FontWeight.Bold)
        .fontColor(BLACK)
    }
    .width('100%')
    .height(TITLE_PLAY_HIGH)
    .margin({ top: 10 })
    .backgroundColor(INK)
    .borderRadius(round(CORNER_BUTTON, TITLE_PLAY_HIGH))
    .justifyContent(FlexAlign.Center)
    .alignItems(VerticalAlign.Center)
    .onClick(() => this.play(data))
    .accessibilityGroup(true)
  }

  /**
   * Where Play goes. `title/Title.kt:83-104`.
   *
   * A film plays. A series goes to where the viewer left off; with nothing to go back to it reads
   * the first season to find its first episode, because an episode number is not always 1.
   */
  private play(data: TitleDetail): void {
    const path = data.title.replace(':', '/');
    if (!isSeriesKey(data.title)) {
      this.shell.go(watchAddress(data.title));
      return;
    }
    const next = data.resume?.episode;
    if (next !== undefined) {
      this.shell.go(watchAddress(data.title) + '/' + next.season.toString() + '/' + next.episode.toString());
      return;
    }
    const seasons = data.seasons;
    const first = seasons !== undefined && seasons.length > 0 ? seasons[0].number : 1;
    this.shell.ask('/v1/titles/' + path + '/seasons/' + first.toString(), (status: number, text: string): void => {
      let episode = 1;
      if (good(status)) {
        const read = readSeason(text);
        if (read !== null && read.episodes.length > 0) {
          episode = read.episodes[0].number;
        }
      }
      this.shell.go(watchAddress(data.title) + '/' + first.toString() + '/' + episode.toString());
    });
  }

  /**
   * The description, three lines until it is tapped. `title/Title.kt:381-413`.
   *
   * "The whole description is the control: a reader hears it, and then whether tapping shows more of
   * it or less."
   */
  @Builder
  synopsis(data: TitleDetail) {
    if (data.overview !== undefined && data.overview.length > 0) {
      Column() {
        Text(data.overview)
          .fontSize(TITLE_SYNOPSIS_TYPE)
          .lineHeight(TITLE_SYNOPSIS_LINE)
          .fontColor(INK)
          .maxLines(this.synopsisOpen ? -1 : TITLE_SYNOPSIS_LINES)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
        // `title/Title.kt:404-411` draws "… more" here, typed into the surface. There is no key for
        // it; `say.show_more` is the words a reader hears, not words to draw. So nothing is drawn,
        // and the whole block stays tappable, which is how the reference opens it anyway.
      }
      .width('100%')
      .margin({ top: 10 })
      .alignItems(HorizontalAlign.Start)
      .onClick(() => {
        this.synopsisOpen = !this.synopsisOpen;
      })
      .accessibilityGroup(true)
      .accessibilityText(data.overview)
      .accessibilityDescription(word(this.seen, this.synopsisOpen ? 'say.show_less' : 'say.show_more'))
    }
  }

  /** `title/Title.kt:415-453`. */
  @Builder
  genres(data: TitleDetail) {
    if (data.genres !== undefined && data.genres.length > 0) {
      Text(this.genreLine(data))
        .fontSize(TITLE_GENRES_TYPE)
        .lineHeight(TITLE_GENRES_LINE)
        .fontColor(MUTED)
        .width('100%')
        .margin({ top: 8 })
        .onClick(() => {
          if (data.genres!.length > TITLE_GENRES_FEW) {
            this.genresOpen = !this.genresOpen;
          }
        })
        .accessibilityGroup(data.genres.length > TITLE_GENRES_FEW)
        .accessibilityText(data.genres.join(', '))
        .accessibilityDescription(
          data.genres.length > TITLE_GENRES_FEW
            ? word(this.seen, this.genresOpen ? 'say.show_less' : 'say.show_more')
            : '',
        )
    }
  }

  private genreLine(data: TitleDetail): string {
    const genres = data.genres ?? [];
    if (this.genresOpen || genres.length <= TITLE_GENRES_FEW) {
      return genres.join(', ');
    }
    return genres.slice(0, TITLE_GENRES_FEW).join(', ');
  }

  /** My List, Rate and Share. `title/Title.kt:455-501`. */
  @Builder
  actions(data: TitleDetail) {
    Row({ space: TITLE_ACTION_GAP }) {
      this.action(
        this.saved ? ICON_CHECK : ICON_PLUS,
        word(this.seen, this.saved ? 'list.added' : 'list.add'),
        () => {
          this.saved = !this.saved;
          this.shell.act(commandAction(listCommand(data.title, this.saved)));
        },
      )
      this.action(this.rateIcon(), '', () => {
        this.rateOpen = !this.rateOpen;
      })
      // `title/Title.kt:179-200` draws a Share control whose word is typed into the surface, and
      // whose action is empty in both families. There is no key and no port for it, so it is left
      // out rather than drawn dead: a control that does nothing is worse than one that is not
      // there, and a reader would be told it exists. Named in the note as an open item.
    }
    .width('100%')
    .margin({ top: 16, left: 4 })
    .alignItems(VerticalAlign.Top)

    if (this.rateOpen) {
      this.rateRow(data)
    }
  }

  @Builder
  action(icon: Resource, says: string, go: () => void) {
    Column() {
      Image(icon)
        .width(TITLE_ACTION_ICON)
        .height(TITLE_ACTION_ICON)
        .fillColor(INK)
        .accessibilityLevel(SKIP_ALL)
      Stack().height(TITLE_ACTION_WORD_GAP)
      if (says.length > 0) {
        Text(says)
          .fontSize(TITLE_ACTION_TYPE)
          .fontColor(MUTED)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }
    }
    .width(TITLE_ACTION_WIDE)
    .padding({ top: 4, bottom: 4 })
    .onClick(go)
    // The word under the icon already says which way the control stands ("My List" against "In My
    // List"), so it is gathered, not replaced (`title/Title.kt:70-72`).
    .accessibilityGroup(true)
  }

  private rateIcon(): Resource {
    if (this.rating === 'down') {
      return ICON_THUMB_DOWN_FILLED;
    }
    if (this.rating === 'up') {
      return ICON_THUMB_UP_FILLED;
    }
    if (this.rating === 'love') {
      return ICON_LOVE_FILLED;
    }
    return ICON_THUMB_UP;
  }

  /** The three ratings. `title/Title.kt:123-175`. */
  @Builder
  rateRow(data: TitleDetail) {
    Row({ space: 4 }) {
      this.rateOne(data, 'down', ICON_THUMB_DOWN, ICON_THUMB_DOWN_FILLED, 'player.down')
      this.rateOne(data, 'up', ICON_THUMB_UP, ICON_THUMB_UP_FILLED, 'player.up')
      this.rateOne(data, 'love', ICON_LOVE, ICON_LOVE_FILLED, 'player.love')
    }
    .margin({ top: 8 })
    .padding({ start: LengthMetrics.vp(8), end: LengthMetrics.vp(8), top: LengthMetrics.vp(4), bottom: LengthMetrics.vp(4) })
    .backgroundColor(RAISED)
    .borderRadius(round(CORNER_BUTTON, 40))
    .alignItems(VerticalAlign.Center)
  }

  @Builder
  rateOne(data: TitleDetail, which: string, idle: Resource, set: Resource, key: string) {
    Stack() {
      Image(this.rating === which ? set : idle)
        .width(TITLE_ACTION_ICON)
        .height(TITLE_ACTION_ICON)
        .fillColor(INK)
        .accessibilityLevel(SKIP_ALL)
    }
    .width(BAR_BOX)
    .height(BAR_BOX)
    .onClick(() => {
      const next = this.rating === which ? '' : which;
      this.rating = next;
      this.shell.act(commandAction(rateCommand(data.title, next.length === 0 ? undefined : next)));
      this.rateOpen = false;
    })
    .accessibilityGroup(true)
    .accessibilityText(word(this.seen, key))
    .accessibilityLevel(READ_ANYWAY)
  }

  /** `title/Title.kt:213-274` of the tabs block. */
  @Builder
  tabs(data: TitleDetail) {
    Row() {
      if (isSeriesKey(data.title)) {
        this.tab(word(this.seen, 'title.episodes_more'), this.episodesTab, () => {
          this.episodesTab = true;
        })
      }
      this.tab(word(this.seen, 'title.more_like'), !this.episodesTab, () => {
        this.episodesTab = false;
      })
    }
    .width('100%')
    .height(TITLE_TAB_HIGH)
    .padding({ start: LengthMetrics.vp(TITLE_BODY_EDGE), end: LengthMetrics.vp(TITLE_BODY_EDGE) })
    .justifyContent(FlexAlign.Start)
  }

  @Builder
  tab(says: string, chosen: boolean, go: () => void) {
    Stack({ alignContent: Alignment.Top }) {
      if (chosen) {
        Stack()
          .width('100%')
          .height(TITLE_TAB_LINE)
          .backgroundColor(ACCENT)
          .accessibilityLevel(SKIP_ALL)
      }
      Text(says)
        .fontSize(TITLE_TAB_TYPE)
        .fontWeight(FontWeight.Bold)
        .fontColor(chosen ? INK : MUTED)
    }
    .height(TITLE_TAB_HIGH)
    .padding({ start: LengthMetrics.vp(TITLE_TAB_PAD), end: LengthMetrics.vp(TITLE_TAB_PAD) })
    .onClick(go)
    .accessibilityGroup(true)
  }

  /**
   * The More Like This tab: a grid of tall tiles, three across. `title/MoreLike.kt:30-69`.
   *
   * "A poster and nothing else, as in a row: the title's name is the tile."
   */
  @Builder
  moreLike(data: TitleDetail) {
    Grid() {
      Repeat<Tile>(data.more_like_this ?? [])
        .each((held: RepeatItem<Tile>) => {
          GridItem() {
            Stack() {
              Art({
                url: tall(held.item.images),
                wide: this.moreWide(),
                high: Math.round(this.moreWide() / TILE_RATIO),
                pictures: this.pictures,
              })
            }
            .width('100%')
            .aspectRatio(TILE_RATIO)
            .backgroundColor(PLATE)
            .borderRadius(CORNER_CARD)
            .clip(true)
            .onClick(() => this.shell.go(titleAddress(held.item.title)))
            .accessibilityGroup(true)
            .accessibilityText(held.item.name)
          }
        })
        .key((tile: Tile, index: number) => tile.title)
    }
    .columnsTemplate(this.moreColumns())
    .columnsGap(MORE_GAP)
    .rowsGap(MORE_GAP)
    .width('100%')
    .padding({
      start: LengthMetrics.vp(MORE_EDGE_SIDE),
      end: LengthMetrics.vp(MORE_EDGE_SIDE),
      top: LengthMetrics.vp(MORE_EDGE_HIGH),
      bottom: LengthMetrics.vp(MORE_EDGE_HIGH),
    })
  }

  private moreColumns(): string {
    const parts: string[] = [];
    for (let at = 0; at < MORE_ACROSS; at++) {
      parts.push('1fr');
    }
    return parts.join(' ');
  }

  /** One column of the More grid, for the size the port is asked for. */
  private moreWide(): number {
    const room = this.screenWide - MORE_EDGE_SIDE * 2 - MORE_GAP * (MORE_ACROSS - 1);
    return Math.round(room / MORE_ACROSS);
  }

  /** The Episodes tab. `title/Episodes.kt:96-160`. */
  @Builder
  episodes(data: TitleDetail) {
    Column() {
      // The season picker, when there is more than one.
      if ((data.seasons ?? []).length > 1) {
        Row({ space: SEASON_CHIP_GAP }) {
          Text(fill(word(this.seen, 'title.season.number'), 'n', this.seasonNumber))
            .fontSize(SEASON_CHIP_TYPE)
            .fontWeight(FontWeight.Medium)
            .fontColor(INK)
          Text(this.seen.rtl ? '‹' : '›')
            .fontSize(SEASON_CHIP_TYPE)
            .fontColor(INK)
            .accessibilityLevel(SKIP_ALL)
        }
        .height(SEASON_CHIP_HIGH)
        .margin({ top: 12, bottom: 12 })
        .padding({ start: LengthMetrics.vp(SEASON_CHIP_PAD), end: LengthMetrics.vp(SEASON_CHIP_PAD) })
        .backgroundColor(RAISED)
        .borderRadius(CORNER_CHIP)
        .alignItems(VerticalAlign.Center)
        .onClick(() => {
          this.seasonSheet = true;
        })
        .accessibilityGroup(true)
        .accessibilityText(word(this.seen, 'say.season_pick'))
        .accessibilityDescription(fill(word(this.seen, 'title.season.number'), 'n', this.seasonNumber))
      } else if (this.seasonNumber > 0) {
        Text(fill(word(this.seen, 'title.season.number'), 'n', this.seasonNumber))
          .fontSize(SEASON_CHIP_TYPE)
          .fontColor(MUTED)
          .margin({ top: 12, bottom: 12 })
      }

      Column({ space: EPISODE_GAP }) {
        Repeat<Episode>(this.season?.episodes ?? [])
          .each((held: RepeatItem<Episode>) => {
            this.episode(data, held.item);
          })
          .key((episode: Episode, index: number) => episode.number.toString())
      }
      .width('100%')
      .alignItems(HorizontalAlign.Start)

      if (this.seasonSheet) {
        this.seasonPicker(data)
      }
    }
    .width('100%')
    .padding({ start: LengthMetrics.vp(EPISODES_EDGE), end: LengthMetrics.vp(EPISODES_EDGE) })
    .alignItems(HorizontalAlign.Start)
  }

  /** One episode: its still with a play ring, its number and name, its length, its description. */
  @Builder
  episode(data: TitleDetail, episode: Episode) {
    Column() {
      Row({ space: EPISODE_ROW_GAP }) {
        Stack() {
          Art({
            url: wide(episode.images),
            wide: EPISODE_ART_WIDE,
            high: EPISODE_ART_HIGH,
            pictures: this.pictures,
          })
          Stack() {
            Image(ICON_PLAY)
              .width(EPISODE_RING_ICON)
              .height(EPISODE_RING_ICON)
              .fillColor(INK)
              .accessibilityLevel(SKIP_ALL)
          }
          .width(EPISODE_RING)
          .height(EPISODE_RING)
          .border({ width: EPISODE_RING_EDGE, color: argb(INK, 0.8) })
          .borderRadius(round(CORNER_BUTTON, EPISODE_RING))
          .accessibilityLevel(SKIP_ALL)

          if (episode.percent !== undefined) {
            Stack({ alignContent: Alignment.Start }) {
              Stack()
                .width(this.watched(episode.percent) + '%')
                .height(PROGRESS_HIGH)
                .backgroundColor(ACCENT)
            }
            .width('100%')
            .height(PROGRESS_HIGH)
            .backgroundColor(argb(INK, 0.3))
            .offset({ y: EPISODE_ART_HIGH - PROGRESS_HIGH })
            .accessibilityLevel(SKIP_ALL)
          }
        }
        .width(EPISODE_ART_WIDE)
        .height(EPISODE_ART_HIGH)
        .backgroundColor(PLATE)
        .borderRadius(CORNER_CARD)
        .clip(true)

        Column() {
          Text(number(episode.number) + '. ' + episode.name)
            .fontSize(EPISODE_NAME_TYPE)
            .lineHeight(EPISODE_NAME_LINE)
            .fontWeight(FontWeight.Bold)
            .fontColor(INK)
            .maxLines(2)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
          if (episode.runtime_min !== undefined && episode.runtime_min > 0) {
            // `title/Episodes.kt:296` writes "${number(rm)}m", with the "m" typed into the surface.
            // The key for it exists -- `title.length.minutes` is "{m}m" -- so this uses it.
            Text(fill(word(this.seen, 'title.length.minutes'), 'm', episode.runtime_min))
              .fontSize(EPISODE_LENGTH_TYPE)
              .fontColor(MUTED)
              .margin({ top: 2 })
          }
        }
        .layoutWeight(1)
        .alignItems(HorizontalAlign.Start)
      }
      .width('100%')
      .alignItems(VerticalAlign.Top)

      if (episode.overview !== undefined && episode.overview.length > 0) {
        Text(episode.overview)
          .fontSize(EPISODE_TEXT_TYPE)
          .lineHeight(EPISODE_TEXT_LINE)
          .fontColor(MUTED)
          .maxLines(EPISODE_TEXT_LINES)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
          .margin({ top: 8 })
      }
    }
    .width('100%')
    .padding({ top: 4, bottom: 4 })
    .alignItems(HorizontalAlign.Start)
    .onClick(() => {
      this.shell.go(
        watchAddress(data.title) + '/' + this.seasonNumber.toString() + '/' + episode.number.toString(),
      );
    })
    .accessibilityGroup(true)
  }

  private watched(percent: number): number {
    return percent < 0 ? 0 : (percent > 100 ? 100 : percent);
  }

  /** The season sheet. `title/Episodes.kt:160-215`. */
  @Builder
  seasonPicker(data: TitleDetail) {
    Column() {
      Column() {
        Repeat<SeasonSummary>(data.seasons ?? [])
          .each((held: RepeatItem<SeasonSummary>) => {
            Stack() {
              Text(fill(word(this.seen, 'title.season.number'), 'n', held.item.number))
                .fontSize(held.item.number === this.seasonNumber ? SEASON_TYPE_CHOSEN : SEASON_TYPE_IDLE)
                .fontColor(held.item.number === this.seasonNumber ? INK : MUTED)
            }
            .width(SEASON_ROW_WIDE)
            .height(SEASON_ROW_HIGH)
            .onClick(() => {
              this.seasonNumber = held.item.number;
              this.season = null;
              this.seasonSheet = false;
              this.readSeason(held.item.number);
            })
            .accessibilityGroup(true)
          })
          .key((season: SeasonSummary, index: number) => season.number.toString())
      }
      .layoutWeight(1)
      .justifyContent(FlexAlign.Center)
      .padding({ top: SEASON_SHEET_TOP, bottom: SEASON_SHEET_BOTTOM })

      Stack() {
        Text('×')
          .fontSize(30)
          .fontColor(BLACK)
          .accessibilityLevel(SKIP_ALL)
      }
      .width(SEASON_CLOSE)
      .height(SEASON_CLOSE)
      .backgroundColor(INK)
      .borderRadius(round(CORNER_BUTTON, SEASON_CLOSE))
      .margin({ bottom: 20 })
      .onClick(() => {
        this.seasonSheet = false;
      })
      .accessibilityGroup(true)
      .accessibilityText(word(this.seen, 'say.close'))
    }
    .width('100%')
    .height('100%')
    .backgroundColor(argb(BLACK, 0.9))
    .alignItems(HorizontalAlign.Center)
  }

  private readSeason(which: number): void {
    const data = this.data();
    if (data === null) {
      return;
    }
    const path = data.title.replace(':', '/');
    this.shell.ask('/v1/titles/' + path + '/seasons/' + which.toString(), (status: number, text: string): void => {
      if (!good(status)) {
        return;
      }
      const read = readSeason(text);
      if (read !== null) {
        this.season = read;
      }
    });
  }
}
