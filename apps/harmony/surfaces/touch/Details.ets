// The facts of a title, read once per run of a screen. A port of
// `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Details.kt`.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { TitleDetail, TitleKey } from '../../contract/contract';
import { good, readTitle } from '../../contract/read';
import { Shell } from '../../ports/Ports';

/** How long the viewer has to settle on a title before its facts are read. `shared/Details.kt:15`. */
const SETTLE_MS: number = 240;

/** What has been read already, for this run of the screen. `shared/Details.kt:17-20`. */
export class DetailStore {
  private readonly kept: Map<string, TitleDetail> = new Map<string, TitleDetail>();
  private readonly shell: Shell;
  /** The settle in flight, so a viewer travelling past ten titles reads none of them. */
  private waiting: number = 0;

  constructor(shell: Shell) {
    this.shell = shell;
  }

  held(title: TitleKey): TitleDetail | null {
    return this.kept.get(title) ?? null;
  }

  /**
   * The facts of whatever the viewer has settled on.
   *
   * `arrived` runs with what was already kept, at once, and again once a read finishes. It does not
   * run when the read is refused: the caller goes on drawing what it had, which is what the Kotlin's
   * null does.
   */
  settle(title: TitleKey, arrived: (detail: TitleDetail) => void): void {
    const already = this.kept.get(title);
    if (already !== undefined) {
      arrived(already);
      return;
    }
    if (this.waiting !== 0) {
      clearTimeout(this.waiting);
    }
    this.waiting = setTimeout(() => {
      this.waiting = 0;
      this.shell.ask('/v1/titles/' + title.replace(':', '/'), (status: number, text: string): void => {
        if (!good(status)) {
          return;
        }
        const read = readTitle(text);
        if (read !== null) {
          this.kept.set(title, read);
          arrived(read);
        }
      });
    }, SETTLE_MS);
  }
}
