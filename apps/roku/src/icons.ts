// The icons, by name, from the one set the whole project draws from.
//
// `platform/icons/icons.json` holds 50 icons (counted 2026-11-12) and
// `platform/tools/make_icons.py` writes them out per family: `television()` emits
// the deleted television page's `icon-data.js` (gone 2026-11-12), `android()` emits Android vector XML, `apple()` emits
// an asset catalog. **There is no `roku()`**, so nothing produces the files the paths below name.
// That is the finding, not a thing to work around here: the names come from the shared set, and the
// generator has to grow a Roku emitter. Roku draws a Poster from a bitmap, so that emitter has to
// rasterise rather than copy SVG.
//
// `pkg:/` is how a path inside an app package is written, as Roku's own manifest examples show
// (verified 2026-11-12,
// https://developer.roku.com/dev/docs/rsg-sdk-standing-docs-core-concepts-manifest-manifest-processing).

/** The icons the screens in this folder draw. Every name is a key of `platform/icons/icons.json`. */
export const ICON = {
  search: "search",
  play: "play",
  plus: "plus",
  check: "check",
  grid: "grid",
  episodes: "episodes",
  caretDown: "caret-down",
  thumbDown: "thumb-down",
  thumbDownFilled: "thumb-down-filled",
  thumbUp: "thumb-up",
  thumbUpFilled: "thumb-up-filled",
  love: "love",
  loveFilled: "love-filled",
} as const;

/** Where an icon sits inside the package. */
export function icon(name: string): string {
  return `pkg:/images/icons/${name}.png`;
}

/**
 * The filled and plain icon of each rating, in the order the reactions row draws them.
 * (`apps/android/surfaces/tv/src/main/kotlin/streams/tv/title/Reactions.kt:22-26`)
 */
export const RATING_ICONS = [
  { plain: ICON.thumbDown, filled: ICON.thumbDownFilled },
  { plain: ICON.thumbUp, filled: ICON.thumbUpFilled },
  { plain: ICON.love, filled: ICON.loveFilled },
];
