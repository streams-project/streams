// The profile chooser. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/profiles/Gate.kt` and
// `.../profiles/GateGrid.kt`.
//
// "background art carousel, Ken Burns zoom, title logo, localized gate lines, arc sheet with
// dynamic tint model, manage mode, and PIN flow" (`profiles/Gate.kt:58-61`).
//
// Two things are deliberately not here, and both are named in the note rather than half built:
//
//   * The ARC of the sheet. `profiles/Gate.kt:191` clips it with `ArcSheetShape(sagDp = 15.2,
//     radiusDp = 1465)` -- a 15.2 sag on a 1465 radius, which on a phone's width is a very shallow
//     curve. Reproducing it on ArkUI wants `clipShape` with a `Path`, and neither the attribute's
//     exact form nor the path syntax was read off the reference here, so the sheet is drawn with its
//     top corners rounded instead. The measured numbers are in `Sizes.ets` and the shape is an open
//     item; what is drawn now is the right colour and the right height in the wrong curve.
//   * Adding, editing and deleting a profile. `profiles/EditProfile.kt` is 1194 lines and
//     `profiles/AddProfile.kt` 311, and the gate's manage mode opens them. They are a screen each
//     and this file raises the taps for them; what it does with those taps today is nothing, which
//     is said in the open items rather than drawn as a control that dies.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { LengthMetrics } from '@kit.ArkUI';
import { image } from '@kit.ImageKit';
import { GateTitle, Profile } from '../../contract/contract';
import { enterProfileAction } from '../../contract/effects';
import { good, readHello, readProfileGate } from '../../contract/read';
import { Pictures, Seen, Shell } from '../../ports/Ports';
import { Art, mainColour } from './Pictures';
import { Face } from './Face';
import { logo } from './Pick';
import { fill, number, word } from './Words';
import { SKIP_ALL } from './Heard';
import { argb, BLACK, INK, JOIN } from './Theme';
import { ICON_CHECK, ICON_LOCKED, ICON_PENCIL, ICON_PLUS } from './Icons';
import { GateTint } from './GateTint';
import {
  FACE_BOX, FACE_CORNER, FACE_LETTER_TYPE, FACE_NAME_GAP, GATE_ACROSS, GATE_BLOCK_DROP,
  GATE_BLOCK_LIFT, GATE_COLUMN_GAP, GATE_EVERY_MS, GATE_FADE_MS, GATE_GRID_DROP, GATE_LABEL_EDGE,
  GATE_LABEL_LIFT, GATE_LABEL_TYPE, GATE_ROW_GAP, GATE_ROW_TOP, GATE_ROW_TOP_LEAST,
  GATE_LINE_GAP, GATE_LINE_TYPE, GATE_LOGO_HIGH, GATE_LOGO_WIDE, GATE_NAME_TYPE, GATE_NAME_WIDE,
  GATE_ROW_TOP_ROOM, GATE_SHEET_DROP, GATE_SHEET_SAG, GATE_STILLS, GATE_ZOOM, PROFILES_MAX,
} from './Sizes';

/** What one cell of the grid is. The grid is profiles, then the actions. `profiles/GateGrid.kt:66-72`. */
enum Cell {
  PROFILE = 'profile',
  ADD = 'add',
  EDIT = 'edit',
  DONE = 'done',
  BLANK = 'blank',
}

class Place {
  readonly kind: Cell;
  readonly person: Profile | null;

  constructor(kind: Cell, person: Profile | null) {
    this.kind = kind;
    this.person = person;
  }
}

@ComponentV2
export struct Gate {
  @Require @Param shell: Shell;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Require @Param screenWide: number;
  @Require @Param screenHigh: number;

  @Local titles: GateTitle[] = [];
  @Local at: number = 0;
  @Local tint: GateTint = GateTint.Neutral;
  @Local zoom: number = 1;
  @Local manage: boolean = false;
  @Local profilesMax: number = PROFILES_MAX;
  /** The profile a PIN is being asked for, or null. */
  @Local locked: Profile | null = null;

  private turning: number = 0;

  aboutToAppear(): void {
    this.shell.ask('/v1/hello', (status: number, text: string): void => {
      if (!good(status)) {
        return;
      }
      const read = readHello(text);
      if (read !== null && read.limits !== undefined) {
        this.profilesMax = read.limits.profiles_max;
      }
    });
    this.shell.ask('/v1/profiles/gate', (status: number, text: string): void => {
      if (!good(status)) {
        return;
      }
      const read = readProfileGate(text);
      if (read !== null) {
        this.titles = read.titles;
        this.turn();
      }
    });
  }

  aboutToDisappear(): void {
    if (this.turning !== 0) {
      clearTimeout(this.turning);
      this.turning = 0;
    }
  }

  /**
   * The carousel: a still every 3850 ms, zooming to 1.08 across it, crossfading in 350.
   * `profiles/Gate.kt:90-103`.
   *
   * The Kotlin runs an `Animatable` in a loop and the ArkUI answer is an attribute that animates
   * itself, so the zoom is set and `.animation()` carries it. A `Swiper` is not used: nobody swipes
   * this, and the reference crossfades rather than sliding -- which on `Swiper` would mean
   * `customContentTransition` cancelling its own slide with a negative translate. Two stacked
   * pictures and an opacity are the same picture for none of the work.
   */
  private turn(): void {
    if (this.titles.length === 0) {
      return;
    }
    this.zoom = 1;
    // The next frame's zoom, so the attribute has somewhere to travel to.
    this.zoom = GATE_ZOOM;
    this.turning = setTimeout(() => {
      const most = this.titles.length < GATE_STILLS ? this.titles.length : GATE_STILLS;
      if (most > 1) {
        this.at = (this.at + 1) % most;
      }
      this.turn();
    }, GATE_EVERY_MS);
  }

  private title(): GateTitle | null {
    if (this.titles.length === 0) {
      return null;
    }
    return this.titles[this.at % this.titles.length];
  }

  // --- where everything sits ---------------------------------------------------------------------
  //
  // `profiles/Gate.kt:152-158`, which calls these "Responsive positioning matching Pixel 8".

  private rowTop(): number {
    const room = this.screenHigh - GATE_ROW_TOP_ROOM;
    const most = room > GATE_ROW_TOP_LEAST ? room : GATE_ROW_TOP_LEAST;
    return GATE_ROW_TOP < most ? GATE_ROW_TOP : most;
  }

  private sheetApex(): number {
    return this.rowTop() + GATE_SHEET_DROP;
  }

  private sheetHigh(): number {
    return this.screenHigh - this.sheetApex();
  }

  build() {
    Stack({ alignContent: Alignment.TopStart }) {
      // 1. The art, zooming, with the band's colour under its foot.
      Stack() {
        Art({
          url: this.art(),
          wide: this.screenWide,
          high: Math.round(this.rowTop()),
          pixels: true,
          pictures: this.pictures,
          tint: (picture: image.PixelMap) => {
            this.tint = GateTint.of(mainColour(picture));
          },
        })
          .scale({ x: this.zoom, y: this.zoom })
          .animation({ duration: GATE_EVERY_MS, curve: Curve.Linear })

        // The bottom fade into the band, at the five stops `profiles/GateArt.kt:129-136` measures:
        // clear to 0.67, the band at 0.85 alpha by 0.88, and full band from 0.98.
        Stack()
          .width('100%')
          .height('100%')
          .linearGradient({
            angle: 180,
            colors: [
              [argb(this.tint.band(), 0), 0],
              [argb(this.tint.band(), 0), 0.67],
              [argb(this.tint.band(), 0.85), 0.88],
              [this.tint.band(), 0.98],
              [this.tint.band(), 1],
            ],
          })
          .accessibilityLevel(SKIP_ALL)
      }
      .width('100%')
      .height(this.rowTop())
      .clip(true)
      .accessibilityLevel(SKIP_ALL)

      // 2. The title's logo and its line, crossfading with the art.
      Column() {
        if (this.gateLogo() !== null) {
          Art({
            url: this.gateLogo(),
            wide: GATE_LOGO_WIDE,
            high: GATE_LOGO_HIGH,
            pictures: this.pictures,
          })
            .constraintSize({ maxWidth: GATE_LOGO_WIDE, maxHeight: GATE_LOGO_HIGH })
        } else if (this.title() !== null) {
          Text(this.title()!.tile.name)
            .fontSize(GATE_NAME_TYPE)
            .fontWeight(FontWeight.Bold)
            .fontColor(INK)
            .textAlign(TextAlign.Center)
            .maxLines(2)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .constraintSize({ maxWidth: GATE_NAME_WIDE })
        }
        if (this.gateLine().length > 0) {
          Text(this.gateLine())
            .fontSize(GATE_LINE_TYPE)
            .fontWeight(FontWeight.Bold)
            .fontColor(INK)
            .textAlign(TextAlign.Center)
            .maxLines(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .margin({ top: GATE_LINE_GAP })
        }
      }
      .width('100%')
      .offset({ y: this.rowTop() - GATE_BLOCK_LIFT - GATE_BLOCK_DROP })
      .alignItems(HorizontalAlign.Center)
      .animation({ duration: GATE_FADE_MS })

      // 3. The arc sheet. Its colour and height are measured; its curve is not reproduced -- see
      //    the note at the head of this file.
      Stack() {
        Stack()
          .width('100%')
          .height('100%')
          .backgroundColor(argb(INK, 0.1))
      }
      .width('100%')
      .height(this.sheetHigh())
      .offset({ y: this.sheetApex() })
      .linearGradient({ angle: 180, colors: this.tint.sheet(this.sheetHigh()) })
      .borderRadius({
        topStart: LengthMetrics.vp(GATE_SHEET_SAG * 2),
        topEnd: LengthMetrics.vp(GATE_SHEET_SAG * 2),
      })
      .clip(true)
      .accessibilityLevel(SKIP_ALL)

      // 4. Who is watching, or Manage profiles.
      Text(this.heading())
        .fontSize(GATE_LABEL_TYPE)
        .fontColor(argb(INK, 0.7))
        .textAlign(TextAlign.Center)
        .width('100%')
        .offset({ y: this.rowTop() - GATE_LABEL_LIFT })
        .padding({ start: LengthMetrics.vp(GATE_LABEL_EDGE), end: LengthMetrics.vp(GATE_LABEL_EDGE) })

      // 5. The grid of faces.
      Column({ space: GATE_ROW_GAP }) {
        Repeat<Place[]>(this.rows())
          .each((row: RepeatItem<Place[]>) => {
            Row({ space: GATE_COLUMN_GAP }) {
              Repeat<Place>(row.item)
                .each((held: RepeatItem<Place>) => {
                  this.cell(held.item);
                })
                .key((place: Place, index: number) => place.kind + '|' + (place.person?.id ?? index.toString()))
            }
            .alignItems(VerticalAlign.Top)
          })
          .key((row: Place[], index: number) => index.toString())
      }
      .width('100%')
      .offset({ y: this.rowTop() + GATE_GRID_DROP })
      .alignItems(HorizontalAlign.Center)
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BLACK)
  }

  @Builder
  cell(place: Place) {
    if (place.kind === Cell.BLANK) {
      Stack().width(FACE_BOX)
    } else if (place.kind === Cell.PROFILE) {
      this.profileCell(place.person!)
    } else {
      this.actionCell(place.kind)
    }
  }

  /** One profile: its face, a kids badge, a padlock, and a pencil in manage mode. `profiles/GateGrid.kt:149-230`. */
  @Builder
  profileCell(person: Profile) {
    Column() {
      Stack() {
        Face({
          person: person,
          server: this.seen.server,
          pictures: this.pictures,
          box: FACE_BOX,
          corner: FACE_CORNER,
          letterType: FACE_LETTER_TYPE,
        })

        if (!this.manage && person.kids) {
          Text(word(this.seen, 'profile.badge.kids'))
            .fontSize(9)
            .fontWeight(FontWeight.Bolder)
            .fontColor(INK)
            .backgroundColor(JOIN)
            .borderRadius(2)
            .padding({ left: 4, right: 4, top: 1, bottom: 1 })
            .offset({ x: -4, y: -4 })
            .accessibilityLevel(SKIP_ALL)
        }

        if (!this.manage && person.has_pin) {
          Image(ICON_LOCKED)
            .width(16)
            .height(16)
            .fillColor(INK)
            .offset({ x: -4, y: -4 })
            .accessibilityLevel(SKIP_ALL)
        }

        if (this.manage) {
          Stack() {
            Image(ICON_PENCIL)
              .width(26)
              .height(26)
              .fillColor(INK)
              .accessibilityLevel(SKIP_ALL)
          }
          .width('100%')
          .height('100%')
          .backgroundColor(argb(BLACK, 0.5))
          .accessibilityLevel(SKIP_ALL)
        }
      }
      .width(FACE_BOX)
      .height(FACE_BOX)
      .borderRadius(FACE_CORNER)
      .clip(true)

      Stack().height(FACE_NAME_GAP)

      Text(person.name)
        .fontSize(13)
        .fontColor(INK)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })
    }
    .width(FACE_BOX)
    .onClick(() => this.chose(person))
    // The name is drawn under the face, so it is gathered rather than given again. A padlock is
    // drawn and nothing else, "which is why it is said here: whoever cannot see the padlock would
    // otherwise reach the profile and be asked for a PIN with no warning"
    // (`profiles/GateGrid.kt:159-162`).
    .accessibilityGroup(true)
    .accessibilityText(
      !this.manage && person.has_pin
        ? fill(word(this.seen, 'say.profile_locked'), 'name', person.name)
        : '',
    )
  }

  /** Edit, Done or Add. `profiles/GateGrid.kt:106-137`. */
  @Builder
  actionCell(kind: Cell) {
    Column() {
      Stack() {
        Image(this.actionIcon(kind))
          .width(this.actionIconSize(kind))
          .height(this.actionIconSize(kind))
          .fillColor(INK)
          .accessibilityLevel(SKIP_ALL)
      }
      .width(FACE_BOX)
      .height(FACE_BOX)
      .backgroundColor(kind === Cell.ADD ? argb(INK, 0.1) : this.tint.editTile())
      .borderRadius(FACE_CORNER)
      .clip(true)

      Stack().height(FACE_NAME_GAP)

      Text(this.actionWord(kind))
        .fontSize(13)
        .fontColor(INK)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })
    }
    .width(FACE_BOX)
    .onClick(() => this.acted(kind))
    .accessibilityGroup(true)
  }

  private actionIcon(kind: Cell): Resource {
    if (kind === Cell.ADD) {
      return ICON_PLUS;
    }
    return kind === Cell.DONE ? ICON_CHECK : ICON_PENCIL;
  }

  /** 28 for Add, 24 for Done, 23 for Edit. `profiles/GateGrid.kt:112,123,133`. */
  private actionIconSize(kind: Cell): number {
    if (kind === Cell.ADD) {
      return 28;
    }
    return kind === Cell.DONE ? 24 : 23;
  }

  private actionWord(kind: Cell): string {
    if (kind === Cell.ADD) {
      return word(this.seen, 'profile.add');
    }
    return word(this.seen, kind === Cell.DONE ? 'common.done' : 'profiles.edit');
  }

  private acted(kind: Cell): void {
    if (kind === Cell.EDIT) {
      this.manage = true;
      return;
    }
    if (kind === Cell.DONE) {
      this.manage = false;
      return;
    }
    // Add. `profiles/AddProfile.kt` is the screen this opens and it is not written here; the open
    // items in the note say so rather than this doing something that looks like it worked.
  }

  /** `profiles/GateGrid.kt:95-103` and `profiles/Gate.kt:231-235`. */
  private chose(person: Profile): void {
    if (this.manage) {
      // `profiles/EditProfile.kt` is the screen this opens. Not written here; see the note.
      return;
    }
    if (person.has_pin) {
      this.locked = person;
      // `profiles/PinSheet.kt` is the sheet that asks. Not written here; see the note. The profile
      // is held so that whoever writes it has the one thing it needs.
      return;
    }
    this.shell.speak(person.language ?? null);
    this.shell.act(enterProfileAction(person.id));
  }

  /** `profiles/Gate.kt:199-201`: the phone's own wording, falling back to the shared one. */
  private heading(): string {
    if (this.manage) {
      return word(this.seen, 'profiles.manage');
    }
    const phone = word(this.seen, 'profiles.choose.phone');
    return phone.length > 0 ? phone : word(this.seen, 'profiles.choose');
  }

  private gateLogo(): string | null {
    const title = this.title();
    return title === null ? null : logo(title.tile.images);
  }

  /**
   * The line under a gate title. A port of `profiles/GateArt.kt:66-93`.
   *
   * `GateLine` carries no text: it carries a kind, and a date or a rank, and the words come from
   * `strings/`. The Kotlin's English fallbacks ("Coming on {date}", "No. {rank} in Films Today")
   * are not carried -- every one of those four keys exists in `strings/en.json`, so a fallback can
   * only ever draw English over a server that is working.
   *
   * THE DATE IS NOT FORMATTED. `profiles/GateArt.kt:55-64` writes it as "d MMMM" in the interface's
   * language with `java.time`. The ArkTS equivalent is `@ohos.intl`'s `DateTimeFormat`, which was
   * not read off the reference here, so this draws the day as the server sent it (YYYY-MM-DD) with
   * its digits in the language's own numerals. That is a visible difference from the Android app
   * and an open item in the note, not a thing to guess at.
   */
  private gateLine(): string {
    const title = this.title();
    if (title === null) {
      return '';
    }
    const line = title.line;
    if (line.kind === 'coming' || line.kind === 'seasons_coming') {
      const day = line.date;
      if (day === undefined) {
        return '';
      }
      const key = line.kind === 'coming' ? 'gate.coming' : 'gate.seasons_coming';
      return fill(word(this.seen, key), 'date', number(day));
    }
    if (line.kind === 'ranked') {
      const rank = line.rank;
      if (rank === undefined) {
        return '';
      }
      const key = line.media === 'tv' ? 'gate.ranked.series' : 'gate.ranked.films';
      return fill(word(this.seen, key), 'rank', rank);
    }
    return '';
  }

  private art(): string | null {
    const title = this.title();
    if (title === null) {
      return null;
    }
    // `profiles/GateArt.kt:44-49`: the art role first, then a poster, then anything wide.
    const images = title.tile.images;
    if (images !== undefined) {
      for (const held of images) {
        if (held.role === 'art') {
          return held.url;
        }
      }
      for (const held of images) {
        if (held.role === 'poster') {
          return held.url;
        }
      }
      for (const held of images) {
        if (held.role === 'backdrop' || held.role === 'still') {
          return held.url;
        }
      }
    }
    return null;
  }

  /**
   * The grid's cells, three across. `profiles/GateGrid.kt:66-86`.
   *
   * In manage mode: the profiles, then Add when there is room, then Done. Otherwise: the profiles,
   * then Edit. Short rows are filled with blanks so the columns line up.
   */
  private rows(): Place[][] {
    const places: Place[] = [];
    for (const person of this.seen.profiles) {
      places.push(new Place(Cell.PROFILE, person));
    }
    if (this.manage) {
      if (this.seen.profiles.length < this.profilesMax) {
        places.push(new Place(Cell.ADD, null));
      }
      places.push(new Place(Cell.DONE, null));
    } else {
      places.push(new Place(Cell.EDIT, null));
    }
    while (places.length % GATE_ACROSS !== 0) {
      places.push(new Place(Cell.BLANK, null));
    }
    const rows: Place[][] = [];
    for (let at = 0; at < places.length; at += GATE_ACROSS) {
      rows.push(places.slice(at, at + GATE_ACROSS));
    }
    return rows;
  }
}
