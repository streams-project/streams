// Every measured number this surface lays out with, and nothing else.
//
// Not one of them was chosen here. Each carries the file and line of the Kotlin that holds it, and
// that Kotlin in turn carries where it was measured -- the Netflix reference folder in the project's
// notes, outside this repository. A number with no citation below is a number this surface may not
// draw with, and `surfaces/touch/README.md` says so.
//
// ArkUI's unit is `vp`, and "This unit is used for values whose unit is not specified"
// (`ts-pixel-units`), so a bare number here is a vp. `Length` is documented as
// `string | number | Resource` where a number's "default unit is vp" (`ts-types`). Whether a vp is
// the same size as Android's dp is ASSUMED, not verified: no HarmonyOS page mentions dp or Android,
// and both are defined as a density-scaled reference unit. If it turns out not to be, every number
// below scales by one factor and this is the one file that changes.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.

/**
 * The width a screen has to work with, by class, never by device name: compact phones, medium
 * (large phones on their side, small tablets), expanded tablets. Every page states its layout per
 * class. The three classes and their two breakpoints are
 * `apps/android/surfaces/phone/src/main/kotlin/streams/phone/app/Sizes.kt:11-22`, which that file
 * says equal `Sizes.swift`.
 */
export enum WidthClass {
  COMPACT = 'compact',
  MEDIUM = 'medium',
  EXPANDED = 'expanded',
}

/** The class of a window this wide. `Sizes.kt:15-22`. */
export function widthClass(wide: number): WidthClass {
  if (wide < 600) {
    return WidthClass.COMPACT;
  }
  if (wide < 840) {
    return WidthClass.MEDIUM;
  }
  return WidthClass.EXPANDED;
}

/** The side margin of a page: 12 on a phone, wider on tablets. `Sizes.kt:26-30`. */
export function pageEdge(wide: number): number {
  const which = widthClass(wide);
  if (which === WidthClass.COMPACT) {
    return 12;
  }
  if (which === WidthClass.MEDIUM) {
    return 24;
  }
  return 40;
}

/**
 * A tile, every row the same: `browse/Rows.kt:84-85`, measured 2026-11-11 with `uiautomator`, where
 * "row 1 reads 305 x 427 px like every row after it: the first row is not taller". 119 x 166.6 is
 * 5 : 7, and it replaced a 184.6 x 369.2 panel the first row used to draw.
 */
export const TILE_WIDE: number = 119;
export const TILE_HIGH: number = 166.6;

/** Between tiles, and the row's start from the screen's edge. `browse/Rows.kt:88-89`. */
export const TILE_GAP: number = 6;
export const ROW_START: number = 12;

/** The pitch of one tile along a row: the tile and the gap after it. `browse/Empty.kt:79-81`. */
export const TILE_PITCH: number = TILE_WIDE + TILE_GAP;

/** The picture of a Continue Watching tile, which carries its controls under the art. `browse/Rows.kt:92`. */
export const CONTINUE_HIGH: number = 138;

/** The control strip under a Continue tile's art, and one of its two round buttons. `browse/Rows.kt:367,375`. */
export const CONTINUE_STRIP_HIGH: number = 38;
export const CONTINUE_BUTTON: number = 34;

/** A row's name box, and the gap from the name to the tiles and from the tiles to the next row. `browse/Rows.kt:152,186,146`. */
export const ROW_NAME_HIGH: number = 34;
export const ROW_NAME_GAP: number = 4;
export const ROW_GAP: number = 12;

/** A row's name: 18, bold. `browse/Rows.kt:169-170`. */
export const ROW_NAME_TYPE: number = 18;

/** Pictures read ahead of the last tile on screen. `browse/Rows.kt:95`. */
export const WARM_AHEAD: number = 6;

/**
 * Rows read ahead of the viewer, vertically. `browse/Paging.kt:41`, which records that the number
 * is BORROWED: "The two-screen trigger is measured on the reference's *web* page, not its phone".
 * The phone column of that line records only that 23 rows arrive progressively.
 */
export const ROWS_AHEAD: number = 9;

/**
 * Titles read ahead of the viewer, sideways. `browse/Paging.kt:47`: a row shows 3.29 tiles, so two
 * rows' worth of travel is 6.6 tiles, so seven. Netflix's own sideways trigger has not been measured.
 */
export const TILES_AHEAD: number = 7;

/** The top bar's height, and the chip row's. `browse/Header.kt:46-47` (reference: 51.9 and 73.4). */
export const TOP_BAR: number = 52;
export const CHIP_ROW: number = 73;

/** A chip, and a chip that stands alone as an outline. `browse/Header.kt:48-49`. */
export const CHIP_HIGH: number = 49;
export const OUTLINE_HIGH: number = 43;

/** The chip row: chips 6 apart, 16 in from each end, words 15 medium. `browse/Header.kt:157,159,207-208`. */
export const CHIP_GAP: number = 6;
export const CHIP_EDGE: number = 16;
export const CHIP_PAD: number = 16;
export const CHIP_TYPE: number = 15;

/** The top bar: a 48 box 4 in, the page's name at 24 bold. `browse/Header.kt:73,94-95`. */
export const BAR_BOX: number = 48;
export const BAR_BOX_EDGE: number = 4;
export const BAR_ICON: number = 24;
export const PAGE_NAME_TYPE: number = 24;
/** The brand's mark inside that box. `browse/Header.kt:87`. */
export const MARK: number = 30;

/** The billboard card: 0.712 wide for every 1 high, 20 in from each side. `browse/Billboard.kt:55-56`. */
export const CARD_RATIO: number = 0.712;
export const CARD_SIDE: number = 20;

/**
 * How much of the screen's height the billboard takes.
 *
 * `browse/Billboard.kt:58-73`, derived there and not measured directly: "On the reference phone the
 * card is 381.7 x 536.2 dp on a 936.6 dp screen, which is 536.2 / 936.6 = 0.572, and that is what
 * leaves the billboard and exactly one row on the first screen". That Kotlin also records that this
 * is the one number it carries across rather than reading off the reference, because no Netflix
 * tablet has been measured, and that a phone reaches the share by filling its width while a wide
 * screen has its height capped at it instead -- measured on a 1280 x 800 tablet, filling the width
 * made the card 218 percent of the screen.
 */
export const CARD_SHARE: number = 0.572;

/** Inside the billboard: 16 in, 14 up from the bottom, the logo 70 percent wide within 108. `browse/Billboard.kt:141,149`. */
export const CARD_PAD: number = 16;
export const CARD_BOTTOM: number = 14;
export const CARD_LOGO_SHARE: number = 0.7;
export const CARD_LOGO_HIGH: number = 108;

/** The billboard's two buttons: 48 high, 16 apart, words 15 semibold, icon 22. `browse/Billboard.kt:165,203,215,219-220`. */
export const WIDE_BUTTON_HIGH: number = 48;
export const WIDE_BUTTON_GAP: number = 16;
export const WIDE_BUTTON_TYPE: number = 15;
export const WIDE_BUTTON_ICON: number = 22;

/** The billboard's name when it has no logo, and its tag line. `browse/Billboard.kt:156,163-164,192`. */
export const CARD_NAME_TYPE: number = 34;
export const CARD_TAGS_TYPE: number = 14;
export const CARD_TAGS_MOST: number = 4;
export const CARD_LOGO_GAP: number = 16;
export const CARD_TAGS_GAP: number = 14;

/** The page's tinted top, behind the header, fading to black. `browse/Home.kt:175`. */
export const TINT_HIGH: number = 640;

/** The floating bar at the foot of a page. `parts/BottomBar.kt:55-57,61,64,72-73,91,93,97`. */
export const BAR_HIGH: number = 63;
export const BAR_PAD: number = 7;
export const BAR_ITEM_HIGH: number = 48;
export const BAR_ITEM_WIDE: number = 66;
/** Narrower once the server offers a fifth place. `parts/BottomBar.kt:57`. */
export const BAR_ITEM_WIDE_FULL: number = 58;
export const BAR_ITEM_PAD: number = 6;
export const BAR_FACE: number = 24;
export const BAR_WORD_TYPE: number = 11;
export const BAR_WORD_GAP: number = 2;
/** The bar's own lift off the foot of the page, and the room a page leaves for it. `browse/Home.kt:245,177`. */
export const BAR_LIFT: number = 9;
export const BAR_ROOM: number = 120;

/** Rows a phone screen holds: 936.6 of screen over a 217.0 row pitch is 4.3. `browse/Empty.kt:35-36`. */
export const WAITING_ROWS: number = 4;

/**
 * The placeholder name bar, measured off the reference capture itself. `browse/Empty.kt:38-45`:
 * "`netflix-reference/phone/02-home-scrolled-placeholders.png`, 1080 x 2400 at 2.5625 px/dp: the
 * bar runs y 1543-1621, which is 79 px, and x 31-1048, which starts and ends on the row's own
 * 12.1 dp gutter. So it is 30.8 dp high -- the measured row name box is 30 high inside a 34.3 dp
 * block -- and as wide as the row."
 */
export const BAR_PLACEHOLDER_HIGH: number = 30.8;

/** The title page: a 52 bar, 16:9 art, a 10 body margin. `title/Title.kt:138,199,209`. */
export const TITLE_BAR: number = 52;
export const TITLE_ART_RATIO: number = 16 / 9;
export const TITLE_BODY_EDGE: number = 10;

/** The title page's name, or its logo at 70 percent wide within 80. `title/Title.kt:220-221,228-231`. */
export const TITLE_NAME_TYPE: number = 24;
export const TITLE_NAME_LINE: number = 28;
export const TITLE_LOGO_SHARE: number = 0.7;
export const TITLE_LOGO_HIGH: number = 80;

/** The title page's facts line, and its age badge. `title/Title.kt:243,249,258,264`. */
export const TITLE_FACTS_GAP: number = 8;
export const TITLE_FACTS_TYPE: number = 14;
export const TITLE_AGE_PAD_SIDE: number = 6;
export const TITLE_AGE_PAD_TOP: number = 2;
export const TITLE_AGE_TYPE: number = 12;

/** The title page's play button, synopsis, genres and three actions. `title/Title.kt:327,358,375-376,399-400,440-441,459-460,486-488`. */
export const TITLE_PLAY_HIGH: number = 48;
export const TITLE_PLAY_TYPE: number = 16;
export const TITLE_PLAY_ICON: number = 22;
export const TITLE_SYNOPSIS_TYPE: number = 16;
export const TITLE_SYNOPSIS_LINE: number = 21;
export const TITLE_SYNOPSIS_LINES: number = 3;
/** Past this many characters the synopsis offers more. `title/Title.kt:403`. */
export const TITLE_SYNOPSIS_LONG: number = 120;
export const TITLE_GENRES_TYPE: number = 13;
export const TITLE_GENRES_LINE: number = 18;
/** Genres shown before the line offers more. `title/Title.kt:418`. */
export const TITLE_GENRES_FEW: number = 3;
export const TITLE_ACTION_WIDE: number = 96;
export const TITLE_ACTION_GAP: number = 8;
export const TITLE_ACTION_ICON: number = 24;
export const TITLE_ACTION_WORD_GAP: number = 6;
export const TITLE_ACTION_TYPE: number = 12;

/** The title page's tabs: 48 boxes, a 4 accent line, words 14 bold. `title/Title.kt:616,626,635-636,643-644`. */
export const TITLE_TAB_HIGH: number = 48;
export const TITLE_TAB_PAD: number = 12;
export const TITLE_TAB_LINE: number = 4;
export const TITLE_TAB_TYPE: number = 14;

/** The profile gate's tiles: 72 avatars rounded 10.8, names 14.8 under them. `profiles/GateGrid.kt:63,141,167,176,225`. */
export const FACE_BOX: number = 72;
export const FACE_CORNER: number = 10.8;
export const FACE_NAME_GAP: number = 14.8;
export const FACE_LETTER_TYPE: number = 32;
/** Between the gate's columns and between its rows. `profiles/GateGrid.kt:76,81`. */
export const GATE_ROW_GAP: number = 49.2;
export const GATE_COLUMN_GAP: number = 40;
/** The gate's grid is three across. `profiles/GateGrid.kt:72,84`. */
export const GATE_ACROSS: number = 3;

/**
 * Where the gate's first row of faces sits, and the arc sheet under it. `profiles/Gate.kt:153-158`,
 * which calls these "Responsive positioning matching Pixel 8": the default top is 669.7, held to at
 * least 340 and to the screen less 240, the sheet's apex is 15.9 below it, the title block's bottom
 * is 65.2 above it, and the art fills down to it.
 */
export const GATE_ROW_TOP: number = 669.7;
export const GATE_ROW_TOP_LEAST: number = 340;
export const GATE_ROW_TOP_ROOM: number = 240;
export const GATE_SHEET_DROP: number = 15.9;
export const GATE_BLOCK_LIFT: number = 65.2;
/** The arc of the sheet. `profiles/Gate.kt:191`. */
export const GATE_SHEET_SAG: number = 15.2;
export const GATE_SHEET_RADIUS: number = 1465;
/** The gate's "who is watching" line, 25 above the first row, at 14. `profiles/Gate.kt:205,210-211`. */
export const GATE_LABEL_LIFT: number = 25;
export const GATE_LABEL_TYPE: number = 14;
export const GATE_LABEL_EDGE: number = 24;
/** The faces sit 8 under the first row's top, and the title block 45 above its own. `profiles/Gate.kt:176,218`. */
export const GATE_GRID_DROP: number = 8;
export const GATE_BLOCK_DROP: number = 45;

/** The gate's carousel: a still every 3850 ms, zooming to 1.08 across it. `profiles/Gate.kt:55-56`. */
export const GATE_EVERY_MS: number = 3850;
export const GATE_ZOOM: number = 1.08;
/** Stills the carousel runs through. `profiles/Gate.kt:100`. */
export const GATE_STILLS: number = 3;
/** The crossfade between two of them. `profiles/Gate.kt:172`. */
export const GATE_FADE_MS: number = 350;
/** Profiles a server allows when its answer does not say. `profiles/Gate.kt:54`. */
export const PROFILES_MAX: number = 5;

/** The chips slide away in 220 ms and the page's tint follows in 600. `browse/Home.kt:130,129`. */
export const CHIPS_MS: number = 220;
export const TINT_MS: number = 600;
/** Past this much scroll the header stops being clear. `browse/Home.kt:133`. */
export const AT_TOP_WITHIN: number = 24;

/** The full page grid a row's name opens: three across, 8 down from its bar. `browse/Rows.kt:557,559`. */
export const GRID_ACROSS: number = 3;
export const GRID_EDGE_HIGH: number = 8;
/** That grid's own bar, and its title at 20 bold. `browse/Rows.kt:520,544-545`. */
export const GRID_BAR: number = 52;
export const GRID_TITLE_TYPE: number = 20;

/** Tiles past which a row offers the whole of itself as a page. `browse/Rows.kt:122`. */
export const ROW_OPENS_PAST: number = 10;

/** A tile's badges. `browse/Rows.kt:456-458,467-471`. */
export const BADGE_EDGE: number = 6;
export const BADGE_TYPE: number = 12;
export const BADGE_NEW_TYPE: number = 10;
export const BADGE_PAD_SIDE: number = 6;
export const BADGE_PAD_TOP: number = 2;
export const BADGE_NEW_PAD_SIDE: number = 5;

/** The progress bar under a tile. `browse/Rows.kt:352,358`. */
export const PROGRESS_HIGH: number = 3;

/** A Top 10 numeral: 92 high, 48 or 68 wide, drawn at 90 with a 3 stroke. `browse/Rows.kt:258-261,277,280,290`. */
export const RANK_NUMERAL_HIGH: number = 92;
export const RANK_NUMERAL_WIDE: number = 48;
export const RANK_NUMERAL_WIDE_TWO: number = 68;
export const RANK_NUMERAL_TYPE: number = 90;
export const RANK_STROKE: number = 3;
/** How much wider a ranked item is than the tile inside it. `browse/Rows.kt:258-259`. */
export const RANK_EXTRA: number = 26;
export const RANK_EXTRA_TWO: number = 34;

/**
 * How long search waits after a keystroke before asking. `apps/android/surfaces/tv/src/main/kotlin/
 * streams/tv/search/Search.kt:31`, which is the only surface in this project that has a search
 * screen -- the phone surface routes `/search` to the server and draws the answer as a page. So the
 * settle is measured, and the LAYOUT of a touch search screen is not: see the note in `Search.ets`.
 */
export const SETTLE_MS: number = 350;
/** Titles the television's search offers before anything is typed. `tv/search/Search.kt:32`. */
export const OFFERED: number = 12;

/** The More Like This tab: three across, 8 apart, 10 in, 12 down. `title/MoreLike.kt:40-52`. */
export const MORE_ACROSS: number = 3;
export const MORE_GAP: number = 8;
export const MORE_EDGE_SIDE: number = 10;
export const MORE_EDGE_HIGH: number = 12;
/** A More Like This tile keeps the row tile's ratio and takes whatever width three columns leave. */
export const TILE_RATIO: number = TILE_WIDE / TILE_HIGH;

/** The Episodes tab: 10 in, a 36 season chip, 16 between episodes. `title/Episodes.kt:98,103,109,111,138`. */
export const EPISODES_EDGE: number = 10;
export const SEASON_CHIP_HIGH: number = 36;
export const SEASON_CHIP_PAD: number = 14;
export const SEASON_CHIP_GAP: number = 6;
export const SEASON_CHIP_TYPE: number = 14;
export const EPISODE_GAP: number = 16;

/** One episode: a 130 x 80 still, 12 to its words. `title/Episodes.kt:234,238`. */
export const EPISODE_ART_WIDE: number = 130;
export const EPISODE_ART_HIGH: number = 80;
export const EPISODE_ROW_GAP: number = 12;
/** Its play ring, 36 across with a 1.5 edge and a 16 glyph. `title/Episodes.kt:249-258`. */
export const EPISODE_RING: number = 36;
export const EPISODE_RING_EDGE: number = 1.5;
export const EPISODE_RING_ICON: number = 16;
/** Its name, its length and its description. `title/Episodes.kt:286-288,296-298,306-310`. */
export const EPISODE_NAME_TYPE: number = 14;
export const EPISODE_NAME_LINE: number = 18;
export const EPISODE_LENGTH_TYPE: number = 12;
export const EPISODE_TEXT_TYPE: number = 13;
export const EPISODE_TEXT_LINE: number = 18;
export const EPISODE_TEXT_LINES: number = 4;

/** The season sheet: a 248 x 52 row per season, 22 chosen and 18 not. `title/Episodes.kt:172-184`. */
export const SEASON_ROW_WIDE: number = 248;
export const SEASON_ROW_HIGH: number = 52;
export const SEASON_TYPE_CHOSEN: number = 22;
export const SEASON_TYPE_IDLE: number = 18;
export const SEASON_SHEET_TOP: number = 90;
export const SEASON_SHEET_BOTTOM: number = 110;
export const SEASON_CLOSE: number = 64;

/** The gate's logo box and its name when it has none. `profiles/GateArt.kt:168-178`. */
export const GATE_LOGO_WIDE: number = 167.4;
export const GATE_LOGO_HIGH: number = 72.2;
export const GATE_NAME_WIDE: number = 220;
export const GATE_NAME_TYPE: number = 20;
/** Its line, 8.2 under the logo. `profiles/GateArt.kt:185-189`. */
export const GATE_LINE_TYPE: number = 14;
export const GATE_LINE_GAP: number = 8.2;

// ---------------------------------------------------------------------------------------------
// Added 2026-11-13. Every one of these was a bare number written at its call site until that day,
// which is 86 lengths across nine files and exactly the thing this file's head forbids: a number
// with no citation is a number this surface may not draw with, and `surfaces/touch/README.md`
// claimed none existed. `platform/tools/touch_check.py` now reads every length a screen draws with
// and fails on any that is not one of the names here, so the claim is held rather than made.
//
// They are grouped by what they belong to, not by the file they were found in.

/**
 * The thinnest line this surface draws: the billboard's second button, the floating bar's edge, a
 * chip's outline and the title page's rule under its facts. One device unit, not a figure off a
 * capture -- `browse/Header.kt:197`, `title/Title.kt:608` and the two borders write `1.dp`.
 */
export const HAIRLINE: number = 1;

/** The gap between a wide button's icon and its word. `browse/Billboard.kt:216`. */
export const WIDE_BUTTON_WORD_GAP: number = 6;

/** The count drawn on the bell: 10 bold, 4 by 1 inside, 8 down and 6 in. `browse/Header.kt:117-124`. */
export const BELL_COUNT_TYPE: number = 10;
export const BELL_COUNT_PAD_SIDE: number = 4;
export const BELL_COUNT_PAD_TOP: number = 1;
export const BELL_COUNT_TOP: number = 8;
export const BELL_COUNT_END: number = 6;

/** After the last box of the top bar, so the row ends on a gap and not on the edge. `browse/Header.kt:130`. */
export const BAR_BOX_GAP: number = 4;

/** A chip that drops: the gap before its caret, and the caret. `browse/Header.kt:213-214`. */
export const CHIP_CARET_GAP: number = 4;
export const CHIP_CARET: number = 18;

/**
 * The kids badge on a face: 9 extra bold, cornered 2, 4 by 1 inside, and inset 4 from the corner
 * it sits in. `profiles/GateGrid.kt:194-202`.
 */
export const KIDS_BADGE_TYPE: number = 9;
export const KIDS_BADGE_CORNER: number = 2;
export const KIDS_BADGE_PAD_SIDE: number = 4;
export const KIDS_BADGE_PAD_TOP: number = 1;
export const BADGE_INSET: number = 4;

/** The pencil over a face in manage mode. `profiles/GateGrid.kt:218`. */
export const GATE_PENCIL: number = 26;

/**
 * A face's name, and the padlock under it when the profile has a PIN: 14, four under the name.
 * `profiles/GateGrid.kt:228,240-246,287`.
 *
 * CORRECTED 2026-11-13. This surface drew the name at 13 and the padlock at 16 laid over the
 * face's bottom corner. The Kotlin draws the name at 14 and the padlock at 14 BELOW the name, and
 * neither number nor place was a decision recorded anywhere here -- they were a transcription. The
 * citation check could not see it, because the sizes were not in this file at all.
 */
export const FACE_NAME_TYPE: number = 14;
export const FACE_LOCK: number = 14;
export const FACE_LOCK_GAP: number = 4;

/** Between the billboard and the first row. `browse/Home.kt:232`. */
export const BILLBOARD_GAP: number = 16;

/** The categories page: 21 a line, 12 by 24 around each, 40 down from the top, 120 clear at the foot. `browse/Home.kt:298,311,317`. */
export const CATEGORY_TYPE: number = 21;
export const CATEGORY_PAD_HIGH: number = 12;
export const CATEGORY_PAD_SIDE: number = 24;
export const CATEGORY_TOP: number = 40;
export const CATEGORY_BOTTOM: number = 120;

/** Its round close, and how far it sits off the foot. `browse/Home.kt:329-330`. */
export const CLOSE_BOX: number = 56;
export const CLOSE_LIFT: number = 24;

/** The gap between the back arrow and the name on a page a row's name opens. `browse/Rows.kt:540`. */
export const GRID_BAR_GAP: number = 8;

/** The New badge's corner. `browse/Rows.kt:470`. */
export const BADGE_NEW_CORNER: number = 4;

/** A Continue tile's two round buttons: the glyph inside, and the room beside its word. `browse/Rows.kt:369,385,400`. */
export const CONTINUE_ICON: number = 20;
export const CONTINUE_WORD_PAD: number = 4;

/** The title page's facts line, 8 under the name. `title/Title.kt:239`. */
export const TITLE_FACTS_TOP: number = 8;

/** A title that has not arrived: its line at 15, 10 down, and the reminder 8 under that. `title/Title.kt:308-310,324`. */
export const TITLE_COMING_TYPE: number = 15;
export const TITLE_COMING_TOP: number = 10;
export const TITLE_REMIND_TOP: number = 8;

/** Between a wide title-page button's icon and its word. `title/Title.kt:339`. */
export const TITLE_BUTTON_WORD_GAP: number = 8;

/** The play button, 10 under the facts; the synopsis 10 under that; the genres 8 under that. `title/Title.kt:310,409,448`. */
export const TITLE_PLAY_TOP: number = 10;
export const TITLE_SYNOPSIS_TOP: number = 10;
export const TITLE_GENRES_TOP: number = 8;

/** The row of actions: 16 down, 4 in, each 4 deep inside. `title/Title.kt:457,497`. */
export const TITLE_ACTIONS_TOP: number = 16;
export const TITLE_ACTIONS_START: number = 4;
export const TITLE_ACTION_PAD_HIGH: number = 4;

/** The three ratings: 4 apart, 8 under the actions, 8 by 4 inside. `title/Title.kt:481,497,513`. */
export const RATE_GAP: number = 4;
export const RATE_TOP: number = 8;
export const RATE_PAD_SIDE: number = 8;
export const RATE_PAD_HIGH: number = 4;

/** The season chip, 12 clear above and below. `title/Episodes.kt:102,132`. */
export const SEASON_CHIP_TOP: number = 12;

/** One episode: its length 2 under its name, its description 8 under that, 4 deep in all. `title/Episodes.kt:229,297,312`. */
export const EPISODE_LENGTH_TOP: number = 2;
export const EPISODE_TEXT_TOP: number = 8;
export const EPISODE_PAD_HIGH: number = 4;

/** The season sheet's close, off the foot. `title/Episodes.kt:194`. */
export const SEASON_CLOSE_LIFT: number = 20;

// ---------------------------------------------------------------------------------------------
// NOT MEASURED. Three numbers this surface chose, and the only three in this file.
//
// Each one is a substitution rather than a port: the Android surface draws its close as two lines
// on a `Canvas` (`browse/Home.kt:340-341`, `title/Title.kt:179-180`, `title/Episodes.kt:202`), and
// this surface draws the multiplication sign at a size that fills the same box, because ArkUI's
// canvas text was already the one `ASSUMED` call in `Tiles.ets` and a glyph needs none of it. The
// Kotlin's own numbers are the canvas box -- 20, 20 and 16 -- and a type size is not a box size,
// so citing them here would be a false citation.
//
// `platform/tools/touch_check.py` holds this list exactly: a fourth number cannot join it quietly,
// and taking one away without giving it a citation fails too. What would settle all three is one
// look at the drawn screen, which wants the SDK.

/** The categories page's close glyph, in a 56 box. `browse/Home.kt:339` draws a 20 canvas instead. */
export const CLOSE_GLYPH_TYPE: number = 28;
/** The title page's close glyph, in a 48 box. `title/Title.kt:179` draws a 16 canvas instead. */
export const TITLE_CLOSE_GLYPH_TYPE: number = 20;
/** The season sheet's close glyph, in a 64 box. `title/Episodes.kt:202` draws a 20 canvas instead. */
export const SEASON_CLOSE_GLYPH_TYPE: number = 30;

/** The rule under the title page's actions, 16 down from them. `title/Title.kt:606`. */
export const TITLE_RULE_TOP: number = 16;

// ---------------------------------------------------------------------------------------------
// The notifications screen, added 2026-11-13 with the screen itself. Every figure is
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/notifications/Notifications.kt`, which
// is the only measurement of this screen the project has.

/** Its bar: 52 high with a 48 box 4 in, a 24 glyph and the page's name at 22 bold. `notifications/Notifications.kt:127,132-133,142,152`. */
export const NOTICE_BAR: number = 52;
export const NOTICE_BAR_EDGE: number = 4;
export const NOTICE_TITLE_TYPE: number = 22;

/** When there is nothing: a 48 bell, 16 of room, and a line at 16. `notifications/Notifications.kt:169-176`. */
export const NOTICE_EMPTY_ICON: number = 48;
export const NOTICE_EMPTY_GAP: number = 16;
export const NOTICE_EMPTY_TYPE: number = 16;

/** The list: 8 clear top and bottom, and 16 around the waiting mark at its foot. `notifications/Notifications.kt:184,212`. */
export const NOTICE_LIST_PAD: number = 8;
export const NOTICE_MORE_PAD: number = 16;

/** One notice: 16 by 10 around it, a 72 x 104 poster, 16 to its words. `notifications/Notifications.kt:248,253,268`. */
export const NOTICE_PAD_SIDE: number = 16;
export const NOTICE_PAD_HIGH: number = 10;
export const NOTICE_ART_WIDE: number = 72;
export const NOTICE_ART_HIGH: number = 104;
export const NOTICE_ART_GAP: number = 16;

/** Its kind, as a chip: 8 by 2 inside, 11 semibold, 6 under it. `notifications/Notifications.kt:282,287,291`. */
export const NOTICE_KIND_PAD_SIDE: number = 8;
export const NOTICE_KIND_PAD_HIGH: number = 2;
export const NOTICE_KIND_TYPE: number = 11;
export const NOTICE_KIND_GAP: number = 6;

/** Its name at 16 bold, then its date at 13 four under it. `notifications/Notifications.kt:297,303,308`. */
export const NOTICE_NAME_TYPE: number = 16;
export const NOTICE_DATE_GAP: number = 4;
export const NOTICE_DATE_TYPE: number = 13;

/** The unread dot, 12 clear of the words. `notifications/Notifications.kt:313,316`. */
export const NOTICE_DOT_GAP: number = 12;
export const NOTICE_DOT: number = 8;

/**
 * How close to the end of the list the viewer comes before the next page is asked for.
 * `notifications/Notifications.kt:105`: `lastVisible >= items.size - 3`.
 */
export const NOTICES_AHEAD: number = 3;

// ---------------------------------------------------------------------------------------------
// The player screen, read from `apps/android/surfaces/phone/src/main/kotlin/streams/phone/watch/Watch.kt`.

/** The controls stay up for 3000 ms, 24 in from the edge, the clock 14 above a 4 bar. `watch/Watch.kt:46,105,109-110`. */
export const WATCH_CONTROLS_MS: number = 3000;
export const WATCH_PAD: number = 24;
export const WATCH_CLOCK_BOTTOM: number = 14;
export const WATCH_CLOCK_TYPE: number = 14;
export const WATCH_BAR_HIGH: number = 4;
