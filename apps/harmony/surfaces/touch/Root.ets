// The one place that picks a screen. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/app/Root.kt`.
//
// It is also the one place that holds `Seen`, measures the window, and sets the language's digits.
// Everything under it is handed what it needs and reaches nothing itself.
//
// WHAT IS NOT BEHIND THIS SWITCH. Five of the twelve targets have no screen in this folder, and
// they are listed by name rather than sent somewhere that looks like it worked:
//
//   `start`                     `door/Doors.kt` (152 lines)
//   `sign_in`                   the phone's sign in form, and the television's code screen
//   `watch`                     `watch/Watch.kt` (96 lines) plus the player's own controls
//   `screen` = notifications    `notifications/Notifications.kt` (322 lines)
//   the server address screen   drawn when `seen.server` is empty, before anything else
//   the opening                 `app/Opening.kt` (309 lines), which covers the first screen while
//                               it builds and lifts as it hands over
//
// A target with no screen draws nothing. That is deliberate: a placeholder would be the `apps/roku`
// mistake -- something that looks like a surface and is not -- and a viewer who reaches a blank page
// has at least not been told a lie about it. Each one is an open item in the note with the Kotlin
// that is its specification.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { Seen, Shell } from '../../ports/Ports';
import { Home } from './Home';
import { Gate } from './Gate';
import { SearchScreen } from './Search';
import { Title } from './Title';
import { Speaking } from './Words';
import { BACK } from './Theme';

@ComponentV2
export struct Root {
  @Require @Param shell: Shell;

  /**
   * Everything a screen knows.
   *
   * One `@Local`, and the host hands it a fresh object on every change (`Seen.copy`), which is what
   * makes this assignment a redraw. `@ObservedV2`/`@Trace` on `Seen` would do the same without the
   * copy and is refused in `ports/Ports.ets` for putting a UI decorator in the ports ring. This is
   * the whole of OPEN QUESTION Q4's answer.
   */
  @Local seen: Seen = new Seen();
  /** The window, in vp, measured rather than asked of a display. */
  @Local wide: number = 0;
  @Local high: number = 0;

  aboutToAppear(): void {
    this.seen = this.shell.seen.copy();
    Speaking.read(this.seen);
    this.shell.watch((seen: Seen): void => {
      this.seen = seen;
      // The language decides digits and plural forms, as on the television; read once per change
      // rather than per draw.
      Speaking.read(seen);
    });
  }

  build() {
    Stack({ alignContent: Alignment.TopStart }) {
      if (this.wide > 0) {
        if (this.seen.server.length === 0) {
          // The address screen. Not written -- see the head of this file.
        } else {
          this.screen()
        }
      }
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BACK)
    // The interface mirrors for a right to left language and the player never does. ArkUI activates
    // mirroring per component: `Direction.Rtl` turns this node, `Direction.Auto` follows the system
    // language. The reference lists `List`, `Grid`, `Scroll`, `Row`, `Column`, `Flex`, `Stack`,
    // `Text`, `TextInput` and `Search` among the components that mirror, and the universal
    // attributes `padding`, `margin`, `borderRadius`, `position` and `offset` among those that do --
    // which is why every one of those in this folder is written with `start`/`end` through
    // `LengthMetrics`, as the RTL guide says to.
    //
    // It is set from the LANGUAGE THE SERVER SENT (`Seen.rtl`, out of `/v1/strings/{lang}`'s `dir`)
    // and not from `Direction.Auto`, because the profile's language is the interface's language and
    // the device's is not: a viewer reading Arabic on an English phone gets a mirrored interface,
    // and `Direction.Auto` would give them an unmirrored one.
    //
    // ASSUMED, and it is the one assumption in this folder that would be visible on every screen:
    // the reference never says `direction` is inherited down a subtree, only that mirroring is
    // "activated on a component" when its own `direction` is `Rtl`. If it turns out not to
    // inherit, every mirroring container in this folder needs the attribute of its own, which is
    // one line per container and no change of shape. The note says what would settle it.
    //
    // One rule from the reference that this layout already obeys: "The setting does not take effect
    // in the Column component." The root here is a `Stack`.
    .direction(this.seen.rtl ? Direction.Rtl : Direction.Ltr)
    .onAreaChange((old: Area, now: Area) => {
      this.wide = Number.parseFloat(now.width.toString());
      this.high = Number.parseFloat(now.height.toString());
    })
  }

  @Builder
  screen() {
    if (this.target() === 'profiles') {
      Gate({
        shell: this.shell,
        seen: this.seen,
        pictures: this.shell.pictures,
        screenWide: this.wide,
        screenHigh: this.high,
      })
    } else if (this.target() === 'title') {
      Title({
        shell: this.shell,
        seen: this.seen,
        pictures: this.shell.pictures,
        screenWide: this.wide,
      })
    } else if (this.isSearch()) {
      SearchScreen({
        shell: this.shell,
        seen: this.seen,
        pictures: this.shell.pictures,
        screenWide: this.wide,
      })
    } else if (this.isBrowse()) {
      Home({
        shell: this.shell,
        seen: this.seen,
        pictures: this.shell.pictures,
        screenWide: this.wide,
        screenHigh: this.high,
      })
    }
    // Every other target draws nothing. See the head of this file for which, and the note for the
    // Kotlin that specifies each.
  }

  private target(): string {
    const target = this.seen.route?.target;
    return target === undefined ? '' : target;
  }

  /**
   * `/search` is a `screen` target whose name is `search`. The Android phone surface draws it as
   * rows through `Home`; this family draws it as a search screen, because a touch surface can carry
   * a field and the television cannot carry one at all. `Search.ets` says what in it is measured.
   */
  private isSearch(): boolean {
    return this.target() === 'screen' && this.seen.route?.screen === 'search';
  }

  /** `app/Root.kt:71-72`: home, the kids' home, and every other server-built page. */
  private isBrowse(): boolean {
    const target = this.target();
    if (target === 'home' || target === 'kids_home') {
      return true;
    }
    // `notifications` is a `screen` target too, and it has a screen of its own in the Android
    // family that is not written here, so it is not sent to `Home`.
    return target === 'screen' && this.seen.route?.screen !== 'notifications';
  }
}
