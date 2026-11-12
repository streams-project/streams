// How a picture reaches this surface. A port of
// `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Art.kt`.
//
// It goes through the `Pictures` port and nowhere else. `Image(src)` on ArkUI accepts a url string
// and then performs the request itself, inside the UI framework, so a screen that wrote a url into
// a component would have the network reaching past its port with no line of code naming it. That is
// the mistake `apps/android/ports/.../Ports.kt:128-140` records being caught: 78 `okhttp3` and 89
// `okio` classes shipped in the APK while a word ban on `okhttp3` passed every run, because no
// surface ever typed it. So no file in this folder passes a string to `Image`.
//
// One import of a HarmonyOS module, and it is a TYPE: `image.PixelMap` is what the port hands back
// as an `Object`, and ArkTS has no structural typing, so the cast has to name the type. Nothing in
// this folder calls `image.createImageSource` or `image.createPixelMap` -- decoding is the
// adapter's, and a ring check for this family should ban those two calls by name rather than the
// kit import, because a type name has to be writable. That is in the note as an open item.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { image } from '@kit.ImageKit';
// Reading one colour out of a picture the port already handed over. Not a fetch and not a decode:
// `effectKit` is given a `PixelMap` that exists. See [mainColour], which is where the risk is named.
import { effectKit } from '@kit.ArkGraphics2D';
import { Picture, Pictures, Stop, Want } from '../../ports/Ports';
import { SKIP_ALL } from './Heard';

/**
 * A picture filling its box.
 *
 * Given the box's size it asks for that size, which is what lets a tile read ahead of the viewer be
 * the same tile that is drawn (`Art.kt:24-35`). Nothing fades in: the reference paints a flat tile
 * and then the artwork, with no shimmer and no crossfade.
 *
 * [says] is the words a screen reader is given. The default is none, which is the right answer for
 * most of them: the art inside a tile is the tile, and the tile says the title's name, so the
 * picture is decoration and marking it so keeps a reader from stopping twice on one thing. A
 * picture that is the only thing carrying its meaning passes its words here.
 */
@ComponentV2
export struct Art {
  /** Where the picture is. Null draws nothing, which leaves the plate behind it showing. */
  @Param url: string | null = null;
  /** The box it will fill, in vp. Zero asks for the picture's own size. */
  @Param wide: number = 0;
  @Param high: number = 0;
  /** True when the caller means to read the pixels back, for a tint. */
  @Param pixels: boolean = false;
  @Param says: string = '';
  @Require @Param pictures: Pictures;
  /** Handed the picture's main colour once, for the page behind it. */
  @Event tint: (picture: image.PixelMap) => void = () => {};

  @Local painted: image.PixelMap | null = null;
  /** The read in flight, so leaving the screen stops it instead of answering into a gone node. */
  private reading: Stop | null = null;

  aboutToAppear(): void {
    this.read();
  }

  aboutToDisappear(): void {
    this.drop();
  }

  /**
   * A row reuses a tile for the next tile of its kind rather than building one, so the url changes
   * under a node that is already on screen. Without this the row would show the first picture it
   * ever read in each slot.
   */
  @Monitor('url')
  moved(): void {
    this.drop();
    this.painted = null;
    this.read();
  }

  private drop(): void {
    const reading = this.reading;
    this.reading = null;
    if (reading !== null) {
      reading.stop();
    }
  }

  private read(): void {
    const url = this.url;
    if (url === null || url.length === 0) {
      return;
    }
    // The box in pixels, which is what the port asks for. `vp2px` on the UI context and not the
    // global one: the reference deprecated the global `vp2px` at API 18 because it "uses the
    // default screen when the UI instance cannot be identified".
    const context = this.getUIContext();
    const want: Want = {
      url: url,
      wide: this.wide > 0 ? context.vp2px(this.wide) : 0,
      high: this.high > 0 ? context.vp2px(this.high) : 0,
      pixels: this.pixels,
    };
    this.reading = this.pictures.fetch(want, (picture: Picture): void => {
      this.reading = null;
      if (picture.kind !== 'painted') {
        // Nothing is drawn and nothing waits. The plate behind stays, which is what the reference
        // shows while a picture is missing.
        return;
      }
      const made = picture.picture as image.PixelMap;
      this.painted = made;
      if (this.pixels) {
        this.tint(made);
      }
    });
  }

  build() {
    Stack() {
      if (this.painted !== null) {
        Image(this.painted)
          .width('100%')
          .height('100%')
          .objectFit(ImageFit.Cover)
          // A picture is not something to drag out of a row. The reference changed this default to
          // true at API 10, so leaving it alone would hand the viewer a drag they never asked for.
          .draggable(false)
          .accessibilityLevel(this.says.length === 0 ? SKIP_ALL : 'auto')
          .accessibilityText(this.says);
      }
    }
    .width('100%')
    .height('100%')
  }
}

/**
 * The main colour of a picture, for the page behind it. Null when it cannot be read.
 *
 * `browse/Billboard.kt:134-135` takes `darkMutedSwatch ?: dominantSwatch` off an `androidx.palette`
 * reading. ArkUI's equivalent is `effectKit`'s colour picker, and the two will not pick the same
 * colour: Palette buckets the picture and prefers a dark muted bucket, while the reference describes
 * `getMainColorSync` as the picture's main colour and nothing more. So this is not a port of that
 * line; it is the nearest thing the platform offers, and the difference is a thing to look at on a
 * device beside the Android app.
 *
 * ASSUMED: `effectKit.createColorPicker`, `ColorPicker.getMainColorSync` and the `Color` it answers
 * with (`red`, `green`, `blue`) were not read off the reference one by one. It is wrapped so that a
 * wrong name costs the page its tint and nothing else -- the billboard's own colours are its own,
 * and the page behind it falls back to the brand's blue, which is what it wears before any picture
 * is read anyway (`browse/Home.kt:78-80`).
 */
export function mainColour(picture: image.PixelMap): string | null {
  try {
    const picked = effectKit.createColorPicker(picture);
    const found = picked.getMainColorSync();
    return '#' + two(found.red) + two(found.green) + two(found.blue);
  } catch (bad) {
    return null;
  }
}

function two(value: number): string {
  const held = value < 0 ? 0 : (value > 255 ? 255 : Math.round(value));
  const text = held.toString(16);
  return text.length < 2 ? '0' + text : text;
}

/**
 * Reads a picture before anything draws it, at the size the box will ask for.
 *
 * A function and not a component, because nothing is drawn: it is called from the effect that
 * watches how far a row has been flung (`browse/Rows.kt:139-144`).
 */
export function warm(pictures: Pictures, url: string | null, wide: number, high: number): void {
  if (url === null || url.length === 0) {
    return;
  }
  const want: Want = { url: url, wide: wide, high: high, pixels: false };
  pictures.warm(want);
}
