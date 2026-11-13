// A main page (Home, Series, Films, a genre, `nav.mine`, search results): the pinned header, the
// billboard, the rows and the floating bar. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Home.kt`.
//
// "laid out and behaving as the reference's. The page behind the header takes the billboard's
// colour at the top and turns black as it scrolls; the chips slide away while the page scrolls down
// and come back as soon as it scrolls up" (`browse/Home.kt:85-89`).
//
// Exactly one row sits under the billboard on the first screen, the top bar does NOT hide on
// scroll, and a tile carries no words -- all three measured
// (`apps/harmony/surfaces/touch/README.md`).
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { LengthMetrics } from '@kit.ArkUI';
import { image } from '@kit.ImageKit';
import { ADDRESS, NavItem, Profile, Section, Tile, TitleDetail } from '../../contract/contract';
// Every action this page raises is built on the TypeScript side of the seam. An object literal in
// ArkTS has to be typed by a named class or interface with no methods
// (`arkts-no-untyped-obj-literals`, 10605038), and the contract writes `Action` and `Command` as
// unions of anonymous shapes -- so no `.ets` file in this folder writes one.
import { commandAction, hideCommand, listCommand } from '../../contract/effects';
import { Pictures, Seen, Shell } from '../../ports/Ports';
import { Billboard } from './Billboard';
import { BarItem, BottomBar } from './BottomBar';
import { Chip, ChipRow, TopBar } from './Header';
import { Depth, Page, pageStamp } from './Paging';
import { DetailStore } from './Details';
import { WaitingRows } from './Empty';
import { isRow, RowGrid, SectionRow } from './Rows';
import { mainColour } from './Pictures';
import { genreAddress, isSeries, tabAddress, titleAddress, watchAddress } from './Pick';
import { word } from './Words';
import { SKIP_ALL } from './Heard';
import { argb, BACK, CATEGORY_IDLE_INK, CORNER_BUTTON, FIRST_TINT, INK, round } from './Theme';
import { ICON_CLIPS, ICON_HOME, ICON_SEARCH, ICON_USER } from './Icons';
import {
  AT_TOP_WITHIN, BAR_LIFT, BAR_ROOM, CHIPS_MS, CHIP_ROW, TINT_HIGH, TINT_MS, TOP_BAR,
} from './Sizes';

/** The pages the chips offer, in the reference's order. `browse/Home.kt:82`. */
const CHIP_SCREENS: string[] = ['series', 'films', 'games', 'new'];

@ComponentV2
export struct Home {
  @Require @Param shell: Shell;
  @Require @Param seen: Seen;
  @Require @Param pictures: Pictures;
  @Require @Param screenWide: number;
  @Require @Param screenHigh: number;

  /** Every section the page can reach: the answer's, plus every part read since. */
  @Local sections: Section[] = [];
  /**
   * The billboard's title, and the rows under it.
   *
   * Held rather than worked out inside `build()`, for two reasons. ArkUI calls `build()` on every
   * change and a method call in it would walk the sections each time; and a `@Param` cannot be
   * given the result of a method that answers `Tile | null`, because narrowing an `if` around a call
   * is not something the compiler can carry into the call below it.
   */
  @Local lead: Tile | null = null;
  @Local rowList: Section[] = [];
  /** The billboard's colour behind the top of the page. */
  @Local tint: string = FIRST_TINT;
  @Local chipsHidden: boolean = false;
  @Local atTop: boolean = true;
  @Local categories: boolean = false;
  /** The row a name opened as a page, and the titles it had when it opened. */
  @Local gridOf: Section | null = null;
  @Local gridTiles: Tile[] = [];
  /** What the viewer just did to the lead title's list state, before the server has agreed. */
  @Local saved: boolean = false;
  @Local sawSaved: boolean = false;
  @Local detail: TitleDetail | null = null;

  private page: Page | null = null;
  private depth: Depth | null = null;
  private store: DetailStore | null = null;
  private stamp: string = '';
  private scroller: Scroller = new Scroller();

  aboutToAppear(): void {
    this.store = new DetailStore(this.shell);
    this.build_page();
  }

  /**
   * The page is built again when the answer it was built from changes, and not on every step.
   *
   * `pageStamp` names the answer: the route, the cursor it carried and how many sections it
   * carried. `browse/Paging.kt:59-69` records why both halves of the reading are keyed on the one
   * stamp -- keyed apart, "a refreshed Home kept every row's old cursor while the rows themselves
   * went back to the fifteen tiles the answer carries".
   */
  @Monitor('seen')
  stepped(): void {
    if (pageStamp(this.screenKey(), this.seen.screen) !== this.stamp) {
      this.build_page();
    }
  }

  private build_page(): void {
    this.stamp = pageStamp(this.screenKey(), this.seen.screen);
    const page = new Page(this.shell, this.seen.screen);
    page.listen((sections: Section[]): void => {
      this.sections = sections;
      this.split();
    });
    this.page = page;
    this.depth = new Depth(this.shell);
    this.sections = page.sections();
    this.split();
    this.chipsHidden = false;
    this.sawSaved = false;
    this.detail = null;
    const lead = this.lead;
    const store = this.store;
    if (lead !== null && store !== null) {
      store.settle(lead.title, (detail: TitleDetail): void => {
        this.detail = detail;
      });
    }
  }

  /** The hero and the rows, out of the sections, once per change rather than once per draw. */
  private split(): void {
    let lead: Tile | null = null;
    const rows: Section[] = [];
    for (const section of this.sections) {
      if (section.kind === 'hero') {
        const tiles = section.tiles;
        if (lead === null && tiles !== undefined && tiles.length > 0) {
          lead = tiles[0];
        }
      } else if (isRow(section)) {
        rows.push(section);
      }
    }
    this.lead = lead;
    this.rowList = rows;
  }

  build() {
    Stack({ alignContent: Alignment.Top }) {
      // The billboard's colour behind the top of the page, fading to black. `browse/Home.kt:171-177`.
      Stack()
        .width('100%')
        .height(TINT_HIGH)
        .linearGradient({
          angle: 180,
          colors: [[argb(this.tint, 0.95), 0], [BACK, 1]],
        })
        .animation({ duration: TINT_MS })
        .accessibilityLevel(SKIP_ALL)

      // The reference "puts nothing else in an empty body, however long it stays empty"
      // (`browse/Home.kt:118-120`), so the placeholder rows are what an unarrived page shows, and
      // they go on showing.
      if (!this.hasScreen() && this.rowList.length === 0) {
        Column() {
          WaitingRows({ screenWide: this.screenWide })
        }
        .width('100%')
        .padding({ top: TOP_BAR + CHIP_ROW, bottom: BAR_ROOM })
      }

      List() {
        if (this.lead !== null) {
          ListItem() {
            Billboard({
              tile: this.lead,
              seen: this.seen,
              pictures: this.pictures,
              screenWide: this.screenWide,
              screenHigh: this.screenHigh,
              saved: this.inList(),
              play: () => this.playLead(),
              list: () => this.listLead(),
              open: () => this.open(this.lead),
              tint: (picture: image.PixelMap) => {
                const found = mainColour(picture);
                if (found !== null) {
                  this.tint = found;
                }
              },
            })
          }
          ListItem() {
            Stack().height(16)
          }
        }
        Repeat<Section>(this.rowList)
          .each((held: RepeatItem<Section>) => {
            ListItem() {
              this.aRow(held.item);
            }
          })
          .key((section: Section, index: number) => section.id)
          // ArkUI's per-kind reuse pool is `.templateId()` plus a `.template()` per kind, which is
          // what Compose's `contentType` gives `browse/Home.kt:202-208`. It is deliberately NOT
          // wired here: all four row kinds build the same component, so four templates would be
          // four copies of one line, and whether separate pools are worth anything is a question
          // about fling cost on a device nobody has. It is an open item in the note, with the
          // measurement that would settle it.
          .virtualScroll()
        // Room at the foot of the page for the floating bar. `browse/Home.kt:177` gets it from
        // `contentPadding`; `contentStartOffset` is verified on `List` (API 11) and a matching end
        // offset was not read off the reference, so this is one item of empty space, which is the
        // same picture and needs no attribute.
        ListItem() {
          Stack().height(BAR_ROOM)
        }
      }
      .scroller(this.scroller)
      .width('100%')
      .height('100%')
      .scrollBar(BarState.Off)
      .contentStartOffset(TOP_BAR + CHIP_ROW)
      .onDidScroll((x: number, y: number, state: ScrollState) => this.travelled(y))
      .onScrollIndex((start: number, end: number) => {
        const page = this.page;
        if (page !== null) {
          // The billboard and the gap under it are two items of the list, so the rows start two in.
          page.near(this.rowList.length + (this.lead === null ? 0 : 2), end);
        }
      })

      // The pinned header: clear over the colour at the top, black once the page moves. It does NOT
      // hide on scroll -- measured (`surfaces/touch/README.md`).
      Column() {
        TopBar({
          seen: this.seen,
          pictures: this.pictures,
          name: this.pageName(),
          back: !this.onHome(),
          unread: this.seen.unreadNotifications,
          goBack: () => this.shell.back(),
          goBell: () => this.shell.go(ADDRESS.NOTIFICATIONS),
        })
        Stack({ alignContent: Alignment.Top }) {
          ChipRow({
            seen: this.seen,
            chips: this.chips(),
            shown: this.chipsHidden ? 0 : 1,
          })
        }
        .width('100%')
        .height(this.chipsHidden ? 0 : CHIP_ROW)
        .clip(true)
        .animation({ duration: CHIPS_MS })
      }
      .width('100%')
      .backgroundColor(this.atTop ? argb(BACK, 0) : argb(BACK, 0.97))

      // The floating bar, lifted off the foot of the page. Positioned rather than laid out in a
      // full-size overlay, so that only the bar's own strip catches a tap -- an overlay the size of
      // the page "blocks all user interactions with components beneath it".
      Row() {
        BottomBar({
          items: this.bar(),
          server: this.seen.server,
          pictures: this.pictures,
        })
      }
      .width('100%')
      .justifyContent(FlexAlign.Center)
      .position({ bottom: LengthMetrics.vp(BAR_LIFT), start: LengthMetrics.vp(0) })

      if (this.categories) {
        this.categoryPage()
      }

      if (this.gridOf !== null) {
        RowGrid({
          section: this.gridOf,
          tiles: this.gridTiles,
          seen: this.seen,
          pictures: this.pictures,
          depth: this.depth!,
          open: (tile: Tile) => this.open(tile),
          close: () => {
            this.gridOf = null;
          },
        })
      }
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BACK)
  }

  @Builder
  aRow(section: Section) {
    SectionRow({
      section: section,
      seen: this.seen,
      pictures: this.pictures,
      depth: this.depth!,
      open: (tile: Tile) => this.open(tile),
      openInfo: (tile: Tile) => this.shell.go(titleAddress(tile.title)),
      remove: (tile: Tile) => this.hide(tile),
      openGrid: (held: Section, tiles: Tile[]) => {
        this.gridOf = held;
        this.gridTiles = tiles;
      },
    })
  }

  /**
   * Every genre the server has, over the page. `browse/Home.kt:248-321`.
   *
   * The Kotlin sorts them with a `Collator` for the interface's language, which ArkTS has no
   * equivalent of in this folder -- `@ohos.i18n` has one, and reaching it from a surface is the
   * network-past-its-port mistake in a quieter form. So the server's order stands here, and that is
   * an OPEN ITEM in the note rather than a decision: the right answer is probably that the server
   * sorts `/v1/nav`, because it already knows the language and every other family would stop
   * carrying a collator.
   */
  @Builder
  categoryPage() {
    Column() {
      List() {
        Repeat<NavItem>(this.genres())
          .each((held: RepeatItem<NavItem>) => {
            ListItem() {
              Stack() {
                Text(held.item.label)
                  .fontSize(21)
                  .fontWeight(this.chosenGenre(held.item) ? FontWeight.Bold : FontWeight.Medium)
                  .fontColor(this.chosenGenre(held.item) ? INK : CATEGORY_IDLE_INK)
                  .textAlign(TextAlign.Center)
              }
              .width('100%')
              .padding({ top: 12, bottom: 12, left: 24, right: 24 })
              .onClick(() => {
                this.categories = false;
                this.shell.go(genreAddress(held.item.genre ?? ''));
              })
              .accessibilityGroup(true)
            }
          })
          .key((item: NavItem, index: number) => item.id)
          .virtualScroll()
      }
      .width('100%')
      .layoutWeight(1)
      .scrollBar(BarState.Off)
      .contentStartOffset(40)

      // A cross drawn on a canvas has nothing for a reader to gather, so this control is given
      // words (`browse/Home.kt:296-298`).
      Stack() {
        Text('×')
          .fontSize(28)
          .fontColor(BACK)
          .accessibilityLevel(SKIP_ALL)
      }
      .width(56)
      .height(56)
      .backgroundColor(INK)
      .borderRadius(round(CORNER_BUTTON, 56))
      .margin({ bottom: 24 })
      .onClick(() => {
        this.categories = false;
      })
      .accessibilityGroup(true)
      .accessibilityText(word(this.seen, 'say.close'))
    }
    .width('100%')
    .height('100%')
    .backgroundColor(argb(BACK, 0.9))
    .alignItems(HorizontalAlign.Center)
  }

  // --- what the page is -------------------------------------------------------------------------

  private here(): string {
    const screen = this.seen.route?.screen;
    return screen === undefined ? 'home' : screen;
  }

  private onHome(): boolean {
    const here = this.here();
    return here === 'home' || here === 'kids';
  }

  /** `browse/Home.kt:102`. */
  private screenKey(): string {
    if (this.onHome()) {
      return 'home';
    }
    return this.here() + ':' + (this.seen.route?.genre ?? '');
  }

  /** The genre the route names, when it names one. `browse/Home.kt:95-97`. */
  private pickedGenre(): NavItem | null {
    if (this.here() !== 'genre') {
      return null;
    }
    for (const item of this.seen.nav) {
      if (item.screen === 'genre' && item.genre === this.seen.route?.genre) {
        return item;
      }
    }
    return null;
  }

  /**
   * The page's name, which the server translated. `browse/Home.kt:98-100`.
   *
   * The Kotlin's last fallback is `words["app.title"]`, and there is NO `app.title` in
   * `strings/en.json` -- checked on 2026-11-12, 301 keys, no such key. So that fallback draws an
   * empty string on Android too, and this draws one here rather than inventing a word. It is an open
   * item for `strings/`, not something to paper over in a surface.
   */
  private pageName(): string {
    const picked = this.pickedGenre();
    if (picked !== null) {
      return picked.label;
    }
    const here = this.here();
    for (const item of this.seen.nav) {
      if (item.screen === here) {
        return item.label;
      }
    }
    return word(this.seen, 'app.title');
  }

  /**
   * Whether what is on screen is this route's page. `browse/Home.kt:117-125`.
   *
   * Until it is, the page draws the placeholder rows and goes on drawing them.
   */
  private hasScreen(): boolean {
    const screen = this.seen.screen;
    if (screen === null) {
      return false;
    }
    const target = this.seen.route?.target;
    let wanted: string | null = null;
    if (target === 'home') {
      wanted = 'home';
    } else if (target === 'kids_home') {
      wanted = 'kids';
    } else {
      const named = this.seen.route?.screen;
      wanted = named === undefined ? null : named;
    }
    return wanted === null || screen.name === wanted;
  }

  private genres(): NavItem[] {
    const held: NavItem[] = [];
    for (const item of this.seen.nav) {
      if (item.screen === 'genre') {
        held.push(item);
      }
    }
    return held;
  }

  private chosenGenre(item: NavItem): boolean {
    return this.here() === 'genre' && item.genre === this.seen.route?.genre;
  }

  // --- what the viewer did ----------------------------------------------------------------------

  private open(tile: Tile | null): void {
    if (tile !== null) {
      this.shell.go(titleAddress(tile.title));
    }
  }

  /** A series opens its page; a film plays. `browse/Home.kt:191`. */
  private playLead(): void {
    const lead = this.lead;
    if (lead === null) {
      return;
    }
    this.shell.go(isSeries(lead) ? titleAddress(lead.title) : watchAddress(lead.title));
  }

  /** `browse/Home.kt:192-196`. */
  private listLead(): void {
    const lead = this.lead;
    if (lead === null) {
      return;
    }
    const next = !this.inList();
    this.saved = next;
    this.sawSaved = true;
    this.shell.act(commandAction(listCommand(lead.title, next)));
  }

  /** What the viewer did if they did anything, else what the server says. `browse/Home.kt:146-147`. */
  private inList(): boolean {
    if (this.sawSaved) {
      return this.saved;
    }
    return this.detail?.in_list ?? false;
  }

  private hide(tile: Tile): void {
    this.shell.act(commandAction(hideCommand(tile.title)));
  }

  /**
   * The page moved. The chips slide away while it scrolls down and come back as soon as it scrolls
   * up, and the header stops being clear once the page is off its top. `browse/Home.kt:133-142`.
   */
  private travelled(y: number): void {
    if (y > 2) {
      this.chipsHidden = true;
    } else if (y < -2) {
      this.chipsHidden = false;
    }
    // `currentOffset()` is documented as able to answer `undefined` when the scroller is not bound
    // to a component yet, and the typed-as-non-null `offset()` that replaces it is API 23. So this
    // reads it defensively rather than trusting the declared type.
    const now = this.scroller.currentOffset();
    this.atTop = now === undefined ? true : now.yOffset < AT_TOP_WITHIN;
  }

  // --- the chips and the bar --------------------------------------------------------------------

  /** On Home the pages the reference lists, on a page the Categories menu alone. `browse/Home.kt:324-343`. */
  private chips(): Chip[] {
    const held: Chip[] = [];
    if (this.onHome()) {
      for (const screen of CHIP_SCREENS) {
        for (const item of this.seen.nav) {
          if (item.screen === screen) {
            held.push(new Chip(item.label, false, false, () => this.shell.go(tabAddress(screen))));
            break;
          }
        }
      }
      held.push(new Chip(word(this.seen, 'nav.categories'), false, true, () => {
        this.categories = true;
      }));
      return held;
    }
    const picked = this.pickedGenre();
    const label = picked !== null ? picked.label : word(this.seen, 'nav.categories');
    held.push(new Chip(label, false, true, () => {
      this.categories = true;
    }));
    return held;
  }

  /**
   * Home, Clips when the server offers it, Search, and the personal tab with the profile's face.
   * `browse/Home.kt:345-355`.
   */
  private bar(): BarItem[] {
    const held: BarItem[] = [];
    held.push(new BarItem(ICON_HOME, this.navLabel('home'), this.onHome(), null, () => {
      this.shell.go(ADDRESS.HOME);
    }));
    const clips = this.navOf('clips');
    if (clips !== null) {
      held.push(new BarItem(ICON_CLIPS, clips.label, false, null, () => {
        this.shell.go(ADDRESS.CLIPS);
      }));
    }
    held.push(new BarItem(ICON_SEARCH, word(this.seen, 'nav.search'), false, null, () => {
      this.shell.go(ADDRESS.SEARCH);
    }));
    const mine = this.navOf('mine');
    if (mine !== null) {
      held.push(new BarItem(ICON_USER, mine.label, false, this.person(), () => {
        this.shell.go(ADDRESS.MINE);
      }));
    }
    return held;
  }

  private navOf(screen: string): NavItem | null {
    for (const item of this.seen.nav) {
      if (item.screen === screen) {
        return item;
      }
    }
    return null;
  }

  private navLabel(screen: string): string {
    const found = this.navOf(screen);
    return found === null ? '' : found.label;
  }

  private person(): Profile | null {
    for (const profile of this.seen.profiles) {
      if (profile.id === this.seen.active) {
        return profile;
      }
    }
    return this.seen.profiles.length > 0 ? this.seen.profiles[0] : null;
  }
}
