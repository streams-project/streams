// Every number this surface lays out with, each beside where it was taken from.
//
// The television surface measures in reference pixels on a 1920 by 1080 grid
// (`apps/android/surfaces/shared/src/main/kotlin/streams/shared/Measure.kt:9-12`: "the screen
// references measure 1920 by 1080"). A Roku app with `ui_resolutions=fhd` lays out on the same
// 1920 by 1080 grid, so every number below crosses unchanged, with no scale factor. That is the
// one piece of luck in this port.
//
// The rule from `apps/android/surfaces/tv/src/test/kotlin/streams/tv/NumbersTest.kt:20` holds
// here: a number is never asserted against itself. Each line says whether it is a measurement or a
// decision, and where it came from.
//
// A citation written `tv/...` is relative to `apps/android/surfaces/tv/src/main/kotlin/streams/`,
// and one written `NumbersTest.kt` is `apps/android/surfaces/tv/src/test/kotlin/streams/tv/`.
//
// The layout numbers are transcribed because there is nowhere else to take them from: the Kotlin
// holds them as `private const val` and the contract generates no layout values for any language.
// The brand's corners and colours are **not** transcribed - they are read from `brand.json` at the
// foot of this file, the same single source Android reads through `BuildConfig`.

import brand from "../../../brand.json";

/** The design grid. A Roku FHD app and the television surface share it. */
export const SCREEN_WIDE = 1920;
export const SCREEN_HIGH = 1080;

/**
 * The page margin, left and right. A decision, not a measurement: `NumbersTest.kt:238-239` records
 * the reference's own safe margin as 68 px at 1920 (`pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md:191`) and this project
 * choosing 72 in note 188. (`tv/app/Round.kt:50`)
 */
export const PAGE_EDGE = 72;

/** Pages without the top bar keep their words on this column. (`tv/app/Round.kt:53`) */
export const SUB_EDGE = 130;

/** A closed tile in a row. Measured. (`tv/browse/Rows.kt:61`) */
export const TILE_WIDE = 311;
export const TILE_HIGH = 504;

/**
 * The open card of the row that owns the screen. Measured:
 * `notes/netflix-reference/tv/raw/tv-geometry-1920x1080.json` `row_screen.focused_tile` is
 * 906 x 510, matched to 904 x 508 in `00-gaps.md` gap 9. (`tv/browse/Rows.kt:88` and its comment
 * at `Rows.kt:100-102`)
 */
export const CARD_WIDE = 904;
export const CARD_HIGH = 508;

/**
 * How far one tile is from the next along a row. Not a measurement: `NumbersTest.kt:85` says 328 is
 * 311 and 17 added up, so a tile resized without touching this would overlap.
 * (`tv/browse/Rows.kt:44`)
 */
export const STEP = TILE_WIDE + 17;

/** The open card's corner, at 1920. Where the row's posters sit. (`tv/browse/Rows.kt:87`) */
export const CARD_X = PAGE_EDGE;
export const CARD_Y = 200;

/** The closed posters of the open row sit three pixels lower than the card. (`tv/browse/Rows.kt:132-137`) */
export const POSTER_Y = 202;

/** The first poster after the open card. (`tv/browse/Rows.kt:137`) */
export const POSTER_AHEAD_X = PAGE_EDGE + CARD_WIDE + 19;

/** The poster already passed, half off the left edge. (`tv/browse/Rows.kt:132`) */
export const POSTER_BEHIND_X = -264;

/** The open row's name, above its card. (`tv/browse/Rows.kt:128`) */
export const ROW_NAME_Y = 146;
export const ROW_NAME_SIZE = 32;

/** The title's facts and story under the open card. (`tv/browse/Rows.kt:143-153`) */
export const FACTS_Y = 728;
export const STORY_Y = 768;
export const STORY_WIDE = 1000;
export const STORY_SIZE = 30;
export const STORY_LINE = 39;
export const STORY_LINES = 3;

/** The name drawn on the open card, inset from its edges. (`tv/browse/Rows.kt:97-109`) */
export const CARD_NAME_SIZE = 44;
export const CARD_INSET = 24;
export const CARD_NAME_WIDE = CARD_WIDE - CARD_INSET * 2;
export const CARD_NAME_BOTTOM = 16;
export const CARD_BADGE_BOTTOM = 78;

/** The shade at the foot of the open card, and the ring that says it is open. (`tv/browse/Rows.kt:96,110`) */
export const CARD_FOOT_HIGH = 200;
export const FOCUS_RING = 3;

/** A closed tile's badges sit this far inside it. (`tv/browse/Rows.kt:64`) */
export const TILE_BADGE_INSET = 12;

/**
 * The next row, dim, cut by the screen's foot. The two values are the row with a card above it and
 * the row with the hero above it. (`tv/browse/Rows.kt:164-175`)
 */
export const NEXT_ROW_NAME_Y = { underCard: 944, underHero: 946 };
export const NEXT_ROW_TOP = { underCard: 998, underHero: 1001 };

/** How many posters each row draws. (`tv/browse/Rows.kt:45-46`) */
export const AHEAD = 5;
export const NEXT_POSTERS = 6;

/** How long a move takes. (`tv/browse/Rows.kt:47-48`, `tv/browse/Browse.kt:42-43`) */
export const SLIDE_MS = 240;
export const FADE_MS = 200;
export const RISE = 810;
export const RISE_MS = 320;

/**
 * The curve every move on the rows page uses, as four control values.
 * (`tv/browse/Rows.kt:51`: `CubicBezierEasing(0.22f, 0.61f, 0.2f, 1f)`)
 */
export const GLIDE = [0.22, 0.61, 0.2, 1] as const;

/**
 * How many rows past the focused one the page keeps in hand. This surface's own prefetch margin,
 * not a Netflix measurement. (`tv/browse/Browse.kt:45-53`)
 */
export const REACH = 2;

/** The top bar: its centre line, the height of its round parts, and the face. (`tv/browse/TopBar.kt:37-39`) */
export const BAR_MID = 68;
export const BAR_HIGH = 64;
export const FACE = 48;

/** Where search and the tabs start, and the gap between tabs. (`tv/browse/TopBar.kt:91,111,114`) */
export const SEARCH_X = 539;
export const TABS_X = 609;
export const TAB_GAP = 8;
export const TAB_SIZE = 26;
export const TAB_INSET = 32;

/** The mark at the far end of the bar. (`tv/browse/TopBar.kt:117`) */
export const MARK_X = SCREEN_WIDE - PAGE_EDGE - BAR_HIGH;

/** How many tabs fit the bar. Genres live on their own pages. (`tv/browse/TopBar.kt:42`) */
export const TAB_LIMIT = 6;

/** The title page. (`tv/title/TitlePage.kt:196-229`) */
export const TITLE_LOGO = { x: PAGE_EDGE, y: 26, wide: 900, high: 200, maxWide: 600, maxHigh: 200 };
export const TITLE_NAME_SIZE = 76;
export const TITLE_NAME_LINE = 84;
export const TITLE_FACTS_Y = 244;
export const TITLE_STORY = { y: 300, wide: 820, size: 28, line: 38, lines: 4 };
/** The reactions sit over the menu icons: both centre on 122. (`tv/title/TitlePage.kt:227-228`) */
export const TITLE_REACTIONS = { x: PAGE_EDGE + 18, y: 504 };
export const TITLE_MENU = { y: 581, wide: 570 };

/** How much larger a corner is on a television than on a phone. (`tv/app/Round.kt:17-20`) */
const TV_SCALE = 1.5;

function tv(unit: number): number {
  return Math.round(unit * TV_SCALE);
}

/**
 * The brand's corners on a television, in reference pixels: the brand's own unit times 1.5. The
 * units come from `brand.json`, as they do on Android through `BuildConfig`
 * (`apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt:51-60`), so `brand.json`
 * changes them all at once and no radius is written twice. (`tv/app/Round.kt:22-47`)
 */
export const ROUND = {
  /** Posters, stills, episode pictures, the picture grid. */
  card: tv(brand.shape.card_radius),
  /** The hero, the landing cards, menus, panels. */
  panel: tv(brand.shape.panel_radius),
  /** Fields, boxes of text, the code plate, PIN boxes. */
  field: tv(brand.shape.field_radius),
  /** Keys and letters: smaller than a field. (`Round.kt:37`, integer arithmetic) */
  key: Math.trunc((tv(brand.shape.field_radius) * 2) / 3),
  /** Badges and small labels. (`Round.kt:40`) */
  chip: Math.trunc(tv(brand.shape.field_radius) / 2),
};

/** `#rrggbb` as a Roku takes it: `0xrrggbbaa`. */
function colour(hex: string, alpha = "ff"): string {
  return `0x${hex.replace("#", "")}${alpha}`;
}

/**
 * The brand's colours, from `brand.json` `colors`, in the form Roku writes them. Roku's own example
 * writes a colour this way (verified 2026-11-12,
 * https://developer.roku.com/dev/docs/rsg-sdk-usefocusable: `color={... "0x0055DDFF" ...}`).
 */
export const COLOUR = {
  background: colour(brand.colors.background),
  surface: colour(brand.colors.surface),
  raised: colour(brand.colors.raised),
  text: colour(brand.colors.text),
  muted: colour(brand.colors.muted),
  accent: colour(brand.colors.accent),
  join: colour(brand.colors.join),
  /** The open card's ring, and a focused tab's fill. */
  focus: colour(brand.colors.text),
  /** A tab that is the page but not where the remote is: the Kotlin's white at 0.16. (`TopBar.kt:48`) */
  tabHere: colour(brand.colors.text, "29"),
  /** Nothing at all. A Rectangle with no fill. */
  none: "0x00000000",
};
