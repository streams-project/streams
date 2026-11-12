// The colour the profile chooser wears, taken from the title behind it. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/profiles/GateTint.kt`.
//
// Every number here is that file's, and that file calls them "the measured brightness model": the
// hue and saturation come from the picture, and the three brightnesses are fixed at 0.13 (the band
// and the top of the sheet), 0.40 (the foot of the sheet) and 0.44 (the edit tile), with the
// saturation held at 0.20 or more so a grey picture still tints.
//
// All arithmetic, no platform call: HSV and RGB convert by hand, so this is the one part of the
// gate that needs nothing checked. Where the Android version reads `Palette`'s dark muted swatch
// this reads the picture's main colour (see `mainColour` in `Pictures.ets`), which is a different
// pick off the same picture -- named as an open item in the note.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.

/** The grey the gate wears when no picture has been read. `profiles/GateTint.kt:34`. */
const NEUTRAL_BAND: string = '#1a1a1a';
/** And its edit tile. `profiles/GateTint.kt:39`. */
const NEUTRAL_TILE: string = '#333333';
/** Saturation floor, so a grey picture still tints. `profiles/GateTint.kt:28`. */
const LEAST_SATURATION: number = 0.2;
/** The band and the top of the sheet. `profiles/GateTint.kt:35,62`. */
const BAND_VALUE: number = 0.13;
/** The foot of the sheet. `profiles/GateTint.kt:63`. */
const FOOT_VALUE: number = 0.4;
/** The edit tile, over a fifth of white. `profiles/GateTint.kt:42,44-47`. */
const TILE_VALUE: number = 0.44;
const TILE_WHITE: number = 0.2;
/** Where the sheet's gradient holds its first colour: 40 over the sheet's height, held in range. `profiles/GateTint.kt:56-60`. */
const STOP_AT: number = 40;
const STOP_LEAST: number = 0.04;
const STOP_MOST: number = 0.25;

function two(value: number): string {
  const held = value < 0 ? 0 : (value > 255 ? 255 : Math.round(value));
  const text = held.toString(16);
  return text.length < 2 ? '0' + text : text;
}

/** An HSV colour as `#rrggbb`. The same arithmetic as `android.graphics.Color.HSVToColor`. */
function hsv(h: number, s: number, v: number): string {
  const sector = (h % 360) / 60;
  const which = Math.floor(sector);
  const part = sector - which;
  const p = v * (1 - s);
  const q = v * (1 - s * part);
  const t = v * (1 - s * (1 - part));
  let r = v;
  let g = t;
  let b = p;
  if (which === 1) {
    r = q;
    g = v;
    b = p;
  } else if (which === 2) {
    r = p;
    g = v;
    b = t;
  } else if (which === 3) {
    r = p;
    g = q;
    b = v;
  } else if (which === 4) {
    r = t;
    g = p;
    b = v;
  } else if (which === 5) {
    r = v;
    g = p;
    b = q;
  }
  return '#' + two(r * 255) + two(g * 255) + two(b * 255);
}

/** The hue and saturation of a `#rrggbb`. The same arithmetic as `Color.colorToHSV`. */
function hueAndSaturation(rgb: string): number[] {
  const r = Number.parseInt(rgb.substring(1, 3), 16) / 255;
  const g = Number.parseInt(rgb.substring(3, 5), 16) / 255;
  const b = Number.parseInt(rgb.substring(5, 7), 16) / 255;
  const most = Math.max(r, Math.max(g, b));
  const least = Math.min(r, Math.min(g, b));
  const span = most - least;
  let h = 0;
  if (span > 0) {
    if (most === r) {
      h = 60 * (((g - b) / span) % 6);
    } else if (most === g) {
      h = 60 * ((b - r) / span + 2);
    } else {
      h = 60 * ((r - g) / span + 4);
    }
  }
  if (h < 0) {
    h += 360;
  }
  const s = most === 0 ? 0 : span / most;
  return [h, s];
}

/** The hue and saturation of a title's art, with the measured brightness model over them. */
export class GateTint {
  readonly h: number;
  readonly s: number;
  readonly neutral: boolean;

  private constructor(h: number, s: number, neutral: boolean) {
    this.h = h;
    this.s = s;
    this.neutral = neutral;
  }

  /** Before any picture has been read. `profiles/GateTint.kt:17`. */
  static readonly Neutral: GateTint = new GateTint(0, 0, true);

  /** From the main colour of a title's art. `profiles/GateTint.kt:23-29`. */
  static of(rgb: string | null): GateTint {
    if (rgb === null || rgb.length < 7) {
      return GateTint.Neutral;
    }
    const read = hueAndSaturation(rgb);
    return new GateTint(read[0], Math.max(read[1], LEAST_SATURATION), false);
  }

  /** The strip between the art and the arc, and the bottom fade's target. */
  band(): string {
    return this.neutral ? NEUTRAL_BAND : hsv(this.h, this.s, BAND_VALUE);
  }

  /** The edit and action tiles of the grid, over a fifth of white. */
  editTile(): string {
    if (this.neutral) {
      return NEUTRAL_TILE;
    }
    const base = hsv(this.h, this.s, TILE_VALUE);
    const r = Number.parseInt(base.substring(1, 3), 16) * (1 - TILE_WHITE) + 255 * TILE_WHITE;
    const g = Number.parseInt(base.substring(3, 5), 16) * (1 - TILE_WHITE) + 255 * TILE_WHITE;
    const b = Number.parseInt(base.substring(5, 7), 16) * (1 - TILE_WHITE) + 255 * TILE_WHITE;
    return '#' + two(r) + two(g) + two(b);
  }

  /**
   * The sheet's gradient: the band's brightness from the apex down, then rising to the foot's.
   * `profiles/GateTint.kt:51-69`.
   *
   * The Kotlin takes the sheet's height in pixels; this takes it in vp, because the stop is a share
   * of the height either way and 40 is a measured distance, not a pixel count.
   */
  sheet(sheetHigh: number): Array<[string, number]> {
    if (this.neutral) {
      return [[NEUTRAL_BAND, 0], [NEUTRAL_BAND, 1]];
    }
    let stop = 0.08;
    if (sheetHigh > 0) {
      stop = STOP_AT / sheetHigh;
      stop = stop < STOP_LEAST ? STOP_LEAST : (stop > STOP_MOST ? STOP_MOST : stop);
    }
    const top = hsv(this.h, this.s, BAND_VALUE);
    return [[top, 0], [top, stop], [hsv(this.h, this.s, FOOT_VALUE), 1]];
  }
}
