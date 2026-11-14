// This app has a server and cannot reach it, said out loud, with a way to try again.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Lost.kt:78-143`. What it says
// and why it is in that order is `src/lost.ts`, which is checked without a Roku; this file draws
// it and knows nothing else.
//
// The hard part of this screen is that it may have no words at all, and `src/lost.ts` carries the
// whole of that. One thing follows from it here: the control stays on the screen even when its
// word has not arrived, because this screen exists for that case, so `Show` wraps the word and
// never the control.

import { Show } from "@roku-sdk/solid";

import { lostSays, lostTrouble, tryAgainWord } from "../lost";
import {
  COLOUR,
  PAGE_EDGE,
  SCREEN_HIGH,
  SCREEN_WIDE,
  TITLE_NAME_LINE,
} from "../numbers";
import { PILL } from "../parts";
import type { Shell } from "../ports";
import { screenRemote } from "./remote";

/** The sentence, the address, and the control, stacked on the middle of the screen. */
const BODY_WIDE = 1100;
const ADDRESS_SIZE = 32;

export function Lost(props: { shell: Shell }) {
  const seen = () => props.shell.seen();
  // The address, always: it is the one thing on this screen that did not have to be fetched to be
  // known. A run that was never given one at all has `refused` instead, which is the address this
  // app would not take (`src/host.ts`, `connect`).
  const address = () => (seen().server !== "" ? seen().server : seen().refused);

  const { ref } = screenRemote({
    read: (word) => {
      if (word !== "ok") return false;
      props.shell.connect(address());
      return true;
    },
    rtl: () => seen().rtl,
    says: () => lostSays(seen().words, address()),
  });

  const middle = Math.round((SCREEN_HIGH - TITLE_NAME_LINE * 2) / 2);
  const left = Math.round((SCREEN_WIDE - BODY_WIDE) / 2);

  return (
    <group ref={ref}>
      <rectangle width={SCREEN_WIDE} height={SCREEN_HIGH} color={COLOUR.background} />
      <Show when={lostTrouble(seen().words) !== ""}>
        <label
          text={lostTrouble(seen().words)}
          translation={[left, middle]}
          width={BODY_WIDE}
          color={COLOUR.text}
        />
      </Show>
      <label
        text={address()}
        translation={[left, middle + TITLE_NAME_LINE]}
        width={BODY_WIDE}
        color={COLOUR.muted}
      />
      {/*
        The way out: the same address, asked again. The shape is square here for the reason the
        top bar's pills are square - a Rectangle has no corner radius and the Effect node's field
        list was not read - and it stays on the screen whether its word arrived or not.
      */}
      <rectangle
        width={BODY_WIDE}
        height={PILL.high}
        translation={[left, middle + TITLE_NAME_LINE * 2 + PAGE_EDGE]}
        color={COLOUR.text}
      />
      <Show when={tryAgainWord(seen().words) !== ""}>
        <label
          text={tryAgainWord(seen().words)}
          translation={[
            left + PILL.inset,
            middle + TITLE_NAME_LINE * 2 + PAGE_EDGE + Math.round((PILL.high - ADDRESS_SIZE) / 2),
          ]}
          width={BODY_WIDE - PILL.inset * 2}
          color={COLOUR.background}
        />
      </Show>
    </group>
  );
}

// `BODY_WIDE` is the Kotlin's 1100 (`Lost.kt:108`) and `ADDRESS_SIZE` its 32 (`Lost.kt:119`).
// Neither is a measurement and neither could be: the reference has no screen like this, because a
// browser at a server that has gone shows the browser's own error page.
//
// OPEN: of a Roku Label, only `text`, `color`, `translation`, `width` and `lineSpacing` were read
// on Roku's pages, and nothing that sets a font size or aligns a word inside its box. So these
// three lines are drawn at whatever size a Label is and start on the left of their box, where the
// television centres them. The README's "text that fits" item is the same gap, and it is why
// `ADDRESS_SIZE` is used to place a line and not to size one.
