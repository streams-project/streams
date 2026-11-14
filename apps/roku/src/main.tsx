// The one place that picks a Roku screen.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Television.kt:34-105`, cut to
// the screens this folder has. A target with no screen written for it draws nothing rather than
// something wrong, and the README lists every one that is missing.

import { RootFocusBoundary } from "@roku-sdk/navigation";
import { Show, Switch, Match } from "@roku-sdk/solid";

import { COLOUR, SCREEN_HIGH, SCREEN_WIDE } from "./numbers";
import type { Shell } from "./ports";
import { Browse } from "./screens/Browse";
import { Lost } from "./screens/Lost";
import { TitlePage } from "./screens/TitlePage";

export function App(props: { shell: Shell }) {
  const seen = () => props.shell.seen();
  const target = () => seen().route?.target ?? null;
  return (
    <group>
      <rectangle width={SCREEN_WIDE} height={SCREEN_HIGH} color={COLOUR.background} />
      {/*
        A run with no server yet starts at the address screen on the other two families
        (`Television.kt:74`). This family is handed its address instead and has no such screen, so
        a run with none draws the background and nothing else - with one exception below, which is
        the address this app refused. The whole of that decision is the head of `src/door.ts`.
      */}
      <Show when={seen().server !== "" || seen().refused !== ""}>
        <Switch>
          {/*
            Lost outranks every page, deliberately. A television that cannot reach its server has
            a route and a page it is trying to be, and until 2026-11-13 it drew that page's empty
            frame for as long as anyone looked at it (`Lost.kt:39-42`). Here the page gives way.
          */}
          <Match when={seen().lost || seen().refused !== ""}>
            <Lost shell={props.shell} />
          </Match>
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

// WHERE THIS STOPS, and it is now one line rather than a folder.
//
// The host that fills a `Seen` is `src/host.ts` and it is written. What is still missing is the
// app's entry point: `main` from `@roku-sdk/rsg-ts` is named on Roku's getting started page
// (verified 2026-11-12) and its signature was published nowhere that was read, so the call that
// hands `Root` to the SDK cannot be written without inventing it.
//
// Everything that call needs is here and type-checked. `src/host.ts` exports `Host`, which takes
// five ports; `src/adapters.ts` has one for each, three of them written against things Roku
// publishes and two that refuse in the open. What an entry point does, in order, is:
//
//   const clock = new TimerClock();
//   const host = new Host({ network: new FetchNetwork(clock), store: new StandingStore(),
//                           clock, player: new NoPlayer(), launch: <the SDK's parameters> });
//   host.watch(() => { /* the SDK's own way of saying the state moved */ });
//   host.start();
//   // and `Root({ shell: host })` into whatever `main` takes.
//
// That is five lines, and the one thing they need is the SDK. The README says so under what is
// not written, and `src/adapters.ts` says which adapter each of them is waiting on.
