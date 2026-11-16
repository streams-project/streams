// The player screen and its skip marks. A port of
// `apps/android/surfaces/phone/src/main/kotlin/streams/phone/watch/Watch.kt` and the time window
// reading of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/watch/Moments.kt:33-34`.
//
// Nothing ticks across the core boundary for skip marks: the core hands `PlayView.windows` over in
// `Seen.play`, and the surface compares `Seen.positionMs` against each `TimeWindow` to offer
// `player.skip_intro` or `player.skip_recap`, seeking to `seek_to_ms` (or `end_ms` when absent).
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { LengthMetrics } from '@kit.ArkUI';
import { TimeWindow } from '../../contract/contract';
import { pauseAction, seekAction, unpauseAction } from '../../contract/effects';
import { Seen, Shell } from '../../ports/Ports';
import { fill, number, word } from './Words';
import { READ_ANYWAY, SKIP_ALL } from './Heard';
import { ACCENT, argb, BLACK, CORNER_BUTTON, INK, round } from './Theme';
import {
  CHIP_PAD, HAIRLINE, SEASON_CHIP_HIGH, SEASON_CHIP_TYPE, WATCH_BAR_HIGH, WATCH_CLOCK_BOTTOM,
  WATCH_CLOCK_TYPE, WATCH_CONTROLS_MS, WATCH_PAD,
} from './Sizes';

/** A position in milliseconds as `M:SS` or `H:MM:SS`, written in the language's digits. `watch/Watch.kt:51-55`. */
export function clock(ms: number): string {
  const whole = ms > 0 ? Math.floor(ms / 1000) : 0;
  const minutes = Math.floor(whole / 60) % 60;
  const seconds = whole % 60;
  const tail = (seconds < 10 ? '0' : '') + seconds.toString();
  if (whole >= 3600) {
    const hours = Math.floor(whole / 3600);
    const mid = (minutes < 10 ? '0' : '') + minutes.toString();
    return number(hours.toString() + ':' + mid + ':' + tail);
  }
  return number(minutes.toString() + ':' + tail);
}

/**
 * The skipable `TimeWindow` (`intro` or `recap`) the picture is inside now, or null.
 * `tv/watch/Moments.kt:33-34`.
 */
export function skipWindow(seen: Seen): TimeWindow | null {
  const windows = seen.play?.windows;
  if (windows === undefined) {
    return null;
  }
  for (const window of windows) {
    if ((window.kind === 'intro' || window.kind === 'recap')
      && seen.positionMs >= window.start_ms
      && seen.positionMs < window.end_ms) {
      return window;
    }
  }
  return null;
}

/** Where a skip window moves playback to: `seek_to_ms` when present, else `end_ms`. `tv/watch/Watch.kt:374`. */
export function skipTargetMs(window: TimeWindow): number {
  return window.seek_to_ms ?? window.end_ms;
}

/** Hands the `XComponent` surface id to the player port once the surface is created. `Ports.ets:96-104`. */
class PictureSurface extends XComponentController {
  private readonly shell: Shell;

  constructor(shell: Shell) {
    super();
    this.shell = shell;
  }

  onSurfaceCreated(surfaceId: string): void {
    this.shell.player.picture(surfaceId);
  }
}

@ComponentV2
export struct Watch {
  @Require @Param shell: Shell;
  @Require @Param seen: Seen;

  @Local shown: boolean = true;
  private hideTimer: number = 0;

  aboutToAppear(): void {
    this.wake();
  }

  aboutToDisappear(): void {
    if (this.hideTimer !== 0) {
      clearTimeout(this.hideTimer);
      this.hideTimer = 0;
    }
  }

  private wake(): void {
    this.shown = true;
    if (this.hideTimer !== 0) {
      clearTimeout(this.hideTimer);
    }
    this.hideTimer = setTimeout(() => {
      this.hideTimer = 0;
      this.shown = false;
    }, WATCH_CONTROLS_MS);
  }

  private toggle(): void {
    this.wake();
    const playing = this.seen.play?.state === 'playing';
    this.shell.act(playing ? pauseAction() : unpauseAction());
  }

  private skip(window: TimeWindow): void {
    this.wake();
    this.shell.act(seekAction(skipTargetMs(window)));
  }

  private position(): string {
    if (this.seen.lengthMs > 0) {
      const at = fill(word(this.seen, 'say.position'), 'at', clock(this.seen.positionMs));
      return fill(at, 'length', clock(this.seen.lengthMs));
    }
    return clock(this.seen.positionMs);
  }

  private donePercent(): string {
    if (this.seen.lengthMs <= 0) {
      return '0%';
    }
    const share = this.seen.positionMs / this.seen.lengthMs;
    const clamped = share < 0 ? 0 : (share > 1 ? 1 : share);
    return (clamped * 100).toString() + '%';
  }

  build() {
    Stack({ alignContent: Alignment.BottomStart }) {
      XComponent({
        type: XComponentType.SURFACE,
        controller: new PictureSurface(this.shell),
      })
        .width('100%')
        .height('100%')
        .accessibilityLevel(SKIP_ALL)

      if (skipWindow(this.seen) !== null || this.shown) {
        Column() {
          if (skipWindow(this.seen) !== null) {
            this.skipPill(skipWindow(this.seen)!)
          }

          if (skipWindow(this.seen) !== null && this.shown) {
            Stack().height(WATCH_CLOCK_BOTTOM)
          }

          if (this.shown) {
            this.bar()
          }
        }
        .width('100%')
        .padding({
          start: LengthMetrics.vp(WATCH_PAD),
          end: LengthMetrics.vp(WATCH_PAD),
          top: LengthMetrics.vp(WATCH_PAD),
          bottom: LengthMetrics.vp(WATCH_PAD),
        })
        .alignItems(HorizontalAlign.Start)
      }
    }
    .width('100%')
    .height('100%')
    .backgroundColor(BLACK)
    .direction(Direction.Ltr)
    .onClick(() => this.toggle())
    .accessibilityGroup(true)
    .accessibilityText(word(this.seen, this.seen.play?.state === 'playing' ? 'say.pause' : 'common.play'))
    .accessibilityDescription(this.position())
  }

  @Builder
  skipPill(window: TimeWindow) {
    Row() {
      Row() {
        Text(word(this.seen, window.kind === 'intro' ? 'player.skip_intro' : 'player.skip_recap'))
          .fontSize(SEASON_CHIP_TYPE)
          .fontWeight(FontWeight.Bold)
          .fontColor(INK)
          .maxLines(1)
      }
      .height(SEASON_CHIP_HIGH)
      .padding({
        start: LengthMetrics.vp(CHIP_PAD),
        end: LengthMetrics.vp(CHIP_PAD),
      })
      .backgroundColor(argb(BLACK, 0.75))
      .borderRadius(round(CORNER_BUTTON, SEASON_CHIP_HIGH))
      .border({ width: HAIRLINE, color: argb(INK, 0.6) })
      .alignItems(VerticalAlign.Center)
      .onClick(() => this.skip(window))
      .accessibilityGroup(true)
      .accessibilityLevel(READ_ANYWAY)
    }
    .width('100%')
    .justifyContent(FlexAlign.End)
  }

  @Builder
  bar() {
    Column() {
      Text(clock(this.seen.positionMs) + ' / ' + clock(this.seen.lengthMs))
        .fontSize(WATCH_CLOCK_TYPE)
        .fontColor(INK)
        .margin({ bottom: WATCH_CLOCK_BOTTOM })

      Stack({ alignContent: Alignment.TopStart }) {
        Stack()
          .width(this.donePercent())
          .height(WATCH_BAR_HIGH)
          .backgroundColor(ACCENT)
      }
      .width('100%')
      .height(WATCH_BAR_HIGH)
      .backgroundColor(argb(INK, 0.33))
    }
    .width('100%')
    .alignItems(HorizontalAlign.Start)
    .accessibilityGroup(true)
    .accessibilityText(this.position())
  }
}
