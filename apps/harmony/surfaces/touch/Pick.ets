// Small readings of the contract: which picture a place wants, and where a tap goes. Ports of
// `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Art.kt:14-22` and
// `.../shared/Addresses.kt`.
//
// It is a file of its own for one reason, and it is a trap worth naming: the contract exports an
// interface called `Image`, and `Image` is also an ArkUI component. In a `.ets` file that draws,
// `import { Image } from '../../contract/contract'` would shadow the component and the file would
// stop compiling somewhere that looks unrelated. So the contract's `Image` is read here, under an
// alias, and no file that draws ever imports it.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { ADDRESS, Image as Art, ImageRole, Tile, TitleKey } from '../../contract/contract';

function find(images: Art[] | undefined, role: ImageRole): string | null {
  if (images === undefined) {
    return null;
  }
  for (const held of images) {
    if (held.role === role) {
      return held.url;
    }
  }
  return null;
}

/** The picture for a wide place: a backdrop, else a still, else the poster. `Art.kt:18-19`. */
export function wide(images: Art[] | undefined): string | null {
  return find(images, 'backdrop') ?? find(images, 'still') ?? find(images, 'poster');
}

/** The picture for a tall place: the poster, else whatever a wide place would take. `Art.kt:22`. */
export function tall(images: Art[] | undefined): string | null {
  return find(images, 'poster') ?? wide(images);
}

/** A title's own logo, which is its name as a picture. `browse/Billboard.kt:144`. */
export function logo(images: Art[] | undefined): string | null {
  return find(images, 'logo');
}

/**
 * A title's page. `Addresses.kt:8`.
 *
 * `TitleKey` is a plain string on the wire -- the generated type is
 * `` `movie:${number}` | `tv:${number}` `` -- so there is no `.value` here where the Kotlin has one,
 * because that is Kotlin's value class and not a field of the contract.
 */
export function titleAddress(key: TitleKey): string {
  return ADDRESS.TITLE + key.replace(':', '-');
}

/** Where a title plays. `Addresses.kt:11`. */
export function watchAddress(key: TitleKey): string {
  return ADDRESS.WATCH + key.replace(':', '/');
}

/**
 * Where a tab lives, by the screen's word. `Addresses.kt:14`, which calls `Address.of(screen)`.
 *
 * The generator emits that function for Kotlin (`platform/contract/generated/types/Contract.kt:
 * 43-61`) and NOT for TypeScript, which gets the `ADDRESS` table alone. So it is written here, out
 * of the table's own fields and never out of a string -- `platform/tools/check_rings.py` bans an
 * address literal in a surface for every family it scans, and rightly: a surface that types "/browse"
 * has a second copy of the address table in it.
 *
 * An open item either way: the generator should emit this for TypeScript as it does for Kotlin, and
 * then this function is deleted. `apps/web` and `apps/tv` read the same generated file.
 */
export function tabAddress(screen: string): string {
  if (screen === 'home') {
    return ADDRESS.HOME;
  }
  if (screen === 'kids') {
    return ADDRESS.KIDS;
  }
  if (screen === 'series') {
    return ADDRESS.SERIES;
  }
  if (screen === 'films') {
    return ADDRESS.FILMS;
  }
  if (screen === 'new') {
    return ADDRESS.NEW;
  }
  if (screen === 'games') {
    return ADDRESS.GAMES;
  }
  if (screen === 'my-list') {
    return ADDRESS.MY_LIST;
  }
  if (screen === 'mine') {
    return ADDRESS.MINE;
  }
  if (screen === 'language') {
    return ADDRESS.LANGUAGE;
  }
  if (screen === 'search') {
    return ADDRESS.SEARCH;
  }
  if (screen === 'notifications') {
    return ADDRESS.NOTIFICATIONS;
  }
  if (screen === 'activity') {
    return ADDRESS.ACTIVITY;
  }
  if (screen === 'clips') {
    return ADDRESS.CLIPS;
  }
  if (screen === 'help') {
    return ADDRESS.HELP;
  }
  if (screen === 'settings') {
    return ADDRESS.SETTINGS;
  }
  return ADDRESS.HOME;
}

/** A genre's page. `browse/Home.kt:271`. */
export function genreAddress(genre: string): string {
  return ADDRESS.GENRE + genre;
}

/** True for a series. `Addresses.kt:17`. */
export function isSeries(tile: Tile): boolean {
  return tile.title.startsWith('tv:');
}

/** True for a series, read off a title's own key. `title/Title.kt:115`. */
export function isSeriesKey(key: TitleKey): boolean {
  return key.startsWith('tv:');
}
