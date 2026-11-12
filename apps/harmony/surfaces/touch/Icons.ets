// Every icon this surface draws, in one place.
//
// THE FILES DO NOT EXIST YET, and that is the honest state of this: the twenty-three names below
// are exactly the `R.drawable.icon_*` the Android phone surface draws, read off
// `apps/android/surfaces/phone` and `apps/android/surfaces/shared` on 2026-11-12, and they are
// written as the `$r('app.media.*')` references a HarmonyOS module resolves. What has to exist for
// them to resolve is a module `resources/base/media` folder, which is part of OPEN QUESTION Q6 --
// there is no DevEco project here, so there is nowhere for a resource to live.
//
// They are named here rather than spelled at each call site so that wiring them up later is one
// file and not thirty edits, and so that a missing icon fails in one place with a name on it.
//
// The brand's MARK is not here. `browse/Header.kt:87` reads it from the server --
// `AsyncImage("$server/mark.svg", ...)` -- so it is a picture like any other and goes through the
// `Pictures` port, not through resources.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.

export const ICON_BACK: Resource = $r('app.media.icon_back');
export const ICON_BELL: Resource = $r('app.media.icon_bell');
export const ICON_CARET_DOWN: Resource = $r('app.media.icon_caret_down');
export const ICON_CAST: Resource = $r('app.media.icon_cast');
export const ICON_CHECK: Resource = $r('app.media.icon_check');
export const ICON_CLIPS: Resource = $r('app.media.icon_clips');
export const ICON_HOME: Resource = $r('app.media.icon_home');
export const ICON_INFO: Resource = $r('app.media.icon_info');
export const ICON_LOCK_OPEN: Resource = $r('app.media.icon_lock_open');
export const ICON_LOCKED: Resource = $r('app.media.icon_locked');
export const ICON_LOVE: Resource = $r('app.media.icon_love');
export const ICON_LOVE_FILLED: Resource = $r('app.media.icon_love_filled');
export const ICON_MORE: Resource = $r('app.media.icon_more');
export const ICON_PENCIL: Resource = $r('app.media.icon_pencil');
export const ICON_PLAY: Resource = $r('app.media.icon_play');
export const ICON_PLUS: Resource = $r('app.media.icon_plus');
export const ICON_SEARCH: Resource = $r('app.media.icon_search');
export const ICON_SHARE: Resource = $r('app.media.icon_share');
export const ICON_THUMB_DOWN: Resource = $r('app.media.icon_thumb_down');
export const ICON_THUMB_DOWN_FILLED: Resource = $r('app.media.icon_thumb_down_filled');
export const ICON_THUMB_UP: Resource = $r('app.media.icon_thumb_up');
export const ICON_THUMB_UP_FILLED: Resource = $r('app.media.icon_thumb_up_filled');
export const ICON_USER: Resource = $r('app.media.icon_user');
