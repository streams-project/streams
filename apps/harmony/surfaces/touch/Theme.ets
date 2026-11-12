// The brand's colours and corners. `brand.json` at the repository root is the source for both, as
// it is for `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt:16-21,51-60`,
// where `streams build android` writes them into `BuildConfig`.
//
// There is no generator for this family yet, so these are copied by hand from `brand.json` and the
// value is written beside each one. That is the one honest option: inventing a radius would put a
// number in this folder that no file in the project agrees with. What it costs is an open item in
// the note -- `streams build harmony` has to write this file, or these six numbers drift.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.

/**
 * `brand.json` `colors`. The names are that file's, not this surface's.
 *
 * Written as `#rrggbb`, which `ResourceColor` accepts. Where the Kotlin applies an alpha with
 * `Color.copy(alpha = ...)` this surface uses `.opacity()` on a node that has no children, which is
 * the same picture and needs no second colour format; the two gradients that need a per-stop alpha
 * use [argb], which is marked ASSUMED below.
 */
export const BACK: string = '#0b0b10';
export const PLATE: string = '#16161d';
export const RAISED: string = '#24242e';
export const INK: string = '#ffffff';
export const MUTED: string = '#a6a6b3';
export const ACCENT: string = '#2f6bff';
export const JOIN: string = '#e5352b';
export const BLACK: string = '#000000';

/**
 * `brand.json` `shape`. 999 means "as round as this thing is tall", which is why every call site
 * takes the smaller of the brand's radius and half the height, exactly as the Kotlin does
 * (`parts/BottomBar.kt:55-56`, `browse/Billboard.kt:203`, `browse/Header.kt:154`).
 */
export const CORNER_BUTTON: number = 999;
export const CORNER_CARD: number = 14;
export const CORNER_CHIP: number = 12;
export const CORNER_CHIP_END: number = 999;
export const CORNER_FIELD: number = 12;
export const CORNER_PANEL: number = 24;

/** The brand's radius, or half of this height, whichever is less. */
export function round(corner: number, high: number): number {
  const half = high / 2;
  return corner < half ? corner : half;
}

/**
 * A colour with an alpha, as `#aarrggbb`.
 *
 * ASSUMED, not verified: the ArkUI reference documents `ResourceColor` as taking a string colour
 * and the alpha-first `#aarrggbb` order is what every example of it uses, but no page was found
 * that states the order in so many words. It is used only where a gradient stop needs its own
 * alpha; everywhere else this surface uses `.opacity()`, which is verified. If the order is wrong
 * the two gradients come out the wrong colour and nothing else in this folder is affected.
 */
export function argb(rgb: string, alpha: number): string {
  const whole = Math.round(alpha * 255);
  const held = whole < 0 ? 0 : (whole > 255 ? 255 : whole);
  const two = held.toString(16);
  const pair = two.length < 2 ? '0' + two : two;
  return '#' + pair + rgb.substring(1);
}

// Greys the reference was measured at, each one the Kotlin's own constant. These are not in
// `brand.json`: they are what Netflix draws, read off the capture, and the Kotlin names them in the
// file that uses them.

/** The billboard's tag line. `browse/Billboard.kt:76`. */
export const TAG_INK: string = '#d4d4d1';
/** The billboard's second button over the tinted page. `browse/Billboard.kt:79` (at 0.9 alpha). */
export const SECOND_FILL: string = '#3a3f47';
/** The chip row's words. `browse/Header.kt:137` -- the reference's grey (191, 191, 190). */
export const CHIP_INK: string = '#bfbfbe';
/** The floating bar's plate, a chosen place on it, and its idle words. `parts/BottomBar.kt:44-46`. */
export const BAR_FILL: string = '#2d2d2d';
export const BAR_CHOSEN: string = '#383838';
export const BAR_IDLE_INK: string = '#c0c0c0';
/** The gate's edit and action tiles. `profiles/GateGrid.kt:44`. */
export const TILE_GREY: string = '#333333';
/** A Top 10 numeral: filled this, stroked that. `browse/Rows.kt:282,292`. */
export const RANK_FILL: string = '#1e1e1e';
export const RANK_STROKE_INK: string = '#9e9e9e';
/** The page the chips open, and the "more" that opens a synopsis. `browse/Home.kt:278`, `title/Title.kt:406`. */
export const CATEGORY_IDLE_INK: string = '#c0c0c0';
export const MORE_INK: string = '#e7e7e7';
/** Before the billboard's colours are read, the page wears our blue, darkened. `browse/Home.kt:79-80`. */
export const FIRST_TINT: string = '#1a2a55';

/** The billboard's plate under its picture. `browse/Billboard.kt:119`. */
export const CARD_PLATE: string = '#16161d';
