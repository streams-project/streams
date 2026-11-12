// The one place that picks a Roku screen.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Television.kt:34-105`, cut to
// the screens this folder has. A target with no screen written for it draws nothing rather than
// something wrong, and the README lists every one that is missing.

import { RootFocusBoundary } from "@roku-sdk/navigation";
import { Show, Switch, Match } from "@roku-sdk/solid";

import { COLOUR } from "./numbers";
import type { Shell } from "./ports";
import { Browse } from "./screens/Browse";
import { TitlePage } from "./screens/TitlePage";

export function App(props: { shell: Shell }) {
  const seen = () => props.shell.seen();
  const target = () => seen().route?.target ?? null;
  return (
    <group>
      <rectangle width={1920} height={1080} color={COLOUR.background} />
      {/*
        A run with no server yet starts at the address screen (`Television.kt:74`). That screen is
        not written, so there is nothing to draw until the host is given an address.
      */}
      <Show when={seen().server !== ""}>
        <Switch>
          <Match when={target() === "title"}>
            <TitlePage shell={props.shell} />
          </Match>
          <Match when={target() === "home" || target() === "kids_home" || target() === "screen"}>
            <Browse shell={props.shell} />
          </Match>
        </Switch>
      </Show>
    </group>
  );
}

/**
 * The root boundary, which every `useFocusable` must sit inside (verified 2026-11-12,
 * https://developer.roku.com/dev/docs/rsg-sdk-usefocusable: it "must be called inside a component
 * rendered within a `RootFocusBoundary` or `FocusBoundary`").
 */
export function Root(props: { shell: Shell }) {
  return (
    <RootFocusBoundary>
      <App shell={props.shell} />
    </RootFocusBoundary>
  );
}

// WHERE THIS STOPS. The app's entry point is `main` from `@roku-sdk/rsg-ts` (named on Roku's
// getting started page, verified 2026-11-12), and the host that fills a `Seen` - http, storage,
// the player, the address of the server - is `src/host.ts`, which is not written. Both need API
// shapes that were not read on Roku's pages and cannot be guessed: `main`'s own signature, the
// SDK's http and registry APIs, and the Video node's fields. The README says what unblocks each.
