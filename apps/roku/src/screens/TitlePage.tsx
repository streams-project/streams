// One title: its artwork, its story, three ratings and a menu of what to do next.
//
// A port of the summary screen of
// `apps/android/surfaces/tv/src/main/kotlin/streams/tv/title/TitlePage.kt:85-231`. The lines, the
// key map and what each line does are `src/title.ts`, apart from the drawing and checked without a
// Roku. Episodes and More Like This are their own screens and are not written yet: the README says
// so.

import { For, Show } from "@roku-sdk/solid";

import { ROUTE, watchAddress } from "../api";
import { logo, wide } from "../art";
import type { Season, TitleDetail } from "../contract";
import {
  COLOUR,
  PAGE_EDGE,
  TITLE_FACTS_Y,
  TITLE_LOGO,
  TITLE_MENU,
  TITLE_NAME_LINE,
  TITLE_NAME_SIZE,
  TITLE_REACTIONS,
  TITLE_STORY,
} from "../numbers";
import type { Shell } from "../ports";
import { ICON, RATING_ICONS, icon } from "../icons";
import { FACES, REACTION_COUNT, TitleFocus, isSeries, lineWord, linesOf } from "../title";
import { say } from "../words";
import type { Word } from "../keys";
import { screenRemote } from "./remote";

/** How far apart the three reactions sit, and how big each one is. (`Reactions.kt:48-49`) */
const REACTION_STEP = 90;
const REACTION_SIZE = 64;

/** A line of the menu: its height and the gap to the next, as `parts/Menu.kt` draws them. */
const LINE_HIGH = 64;
const LINE_GAP = 8;

export function TitlePage(props: { shell: Shell }) {
  const seen = () => props.shell.seen();
  const data = (): TitleDetail | null => {
    const held = seen().detail;
    if (held === null) return null;
    return held.title === seen().route?.title ? held : null;
  };

  // A new title starts at its summary, and the line opened there is kept for coming back to it.
  // (`TitlePage.kt:89-90`, and note 187's row for this page)
  let focus: TitleFocus | null = null;
  let heldFor: string | null = null;
  function state(): TitleFocus | null {
    const held = data();
    if (held === null) return null;
    if (focus === null || heldFor !== held.title) {
      focus = new TitleFocus(held);
      heldFor = held.title;
    }
    return focus;
  }

  /** Play: a film from where it was left, a series from the episode the viewer is on. (`TitlePage.kt:114-130`) */
  function play(held: TitleDetail): void {
    if (!isSeries(held)) {
      props.shell.go(watchAddress(held.title));
      return;
    }
    const next = held.resume?.episode;
    if (next !== undefined) {
      props.shell.go(`${watchAddress(held.title)}/${next.season}/${next.episode}`);
      return;
    }
    const number = held.seasons?.[0]?.number ?? 1;
    const [kind, id] = held.title.split(":");
    props.shell.ask(ROUTE.season(kind!, id!, number), (status, text) => {
      let first = 1;
      if (status >= 200 && status <= 299) {
        const season = JSON.parse(text) as Season;
        first = season.episodes?.[0]?.number ?? 1;
      }
      props.shell.go(`${watchAddress(held.title)}/${number}/${first}`);
    });
  }

  function read(word: Word): boolean {
    const held = data();
    const at = state();
    if (held === null || at === null) return false;
    const deed = at.press(word);
    switch (deed.kind) {
      case "play":
        play(held);
        return true;
      // A change the viewer made goes through the rules, as `TitlePage.kt:135,140` does, and not
      // as a request this screen builds: the rules queue it, retry it and hold the token.
      case "list":
        props.shell.act({
          action: "command",
          command: { command: "list", in_list: at.saved, title: held.title },
        });
        return true;
      // `rating` is left out rather than sent as null when the viewer took a rating back: "An
      // optional field is left out and never sent as null" (`platform/contract/README.md:40`).
      case "rate":
        props.shell.act({
          action: "command",
          command:
            deed.rating === null
              ? { command: "rate", title: held.title }
              : { command: "rate", rating: deed.rating, title: held.title },
        });
        return true;
      case "back":
        props.shell.back();
        return true;
      // Episodes and More Like This are not written yet, so their lines do nothing rather than
      // opening a screen that is not there.
      default:
        return true;
    }
  }

  const { ref } = screenRemote({ read, rtl: () => seen().rtl });

  return (
    <group ref={ref}>
      <rectangle width={1920} height={1080} color={COLOUR.background} />
      <Show when={data()}>
        {(held: TitleDetail) => (
          <group>
            {/*
              The artwork and its shades. Decoration: the title is named, described and listed on
              top of it, so a Poster that announced itself would say it twice. (`TitlePage.kt:60-69`)
            */}
            <Show when={wide(held.images)}>
              {(url: string) => (
                <poster uri={url} width={1920} height={1080} loadDisplayMode="scaleToFill" />
              )}
            </Show>
            <Show when={logo(held.images)} fallback={
              <label
                text={held.name}
                translation={[TITLE_LOGO.x, TITLE_LOGO.y + TITLE_LOGO.high - TITLE_NAME_LINE * 2]}
                width={TITLE_LOGO.wide}
                color={COLOUR.text}
              />
            }>
              {(url: string) => (
                <poster
                  uri={url}
                  width={TITLE_LOGO.maxWide}
                  height={TITLE_LOGO.maxHigh}
                  translation={[TITLE_LOGO.x, TITLE_LOGO.y]}
                  loadDisplayMode="scaleToFit"
                  audioGuideText={held.name}
                />
              )}
            </Show>
            <label
              text={held.overview ?? ""}
              translation={[PAGE_EDGE, TITLE_STORY.y]}
              width={TITLE_STORY.wide}
              color={COLOUR.muted}
              lineSpacing={TITLE_STORY.line - TITLE_STORY.size}
            />
            {/* The three ratings. Each carries its own word: three faces and no words is a control a
                person cannot hear. (`Reactions.kt:34-39`) */}
            <For each={FACES}>
              {(face: { rating: string; word: string }, index: () => number) => (
                <group
                  translation={[TITLE_REACTIONS.x + index() * REACTION_STEP, TITLE_REACTIONS.y]}
                >
                  <rectangle
                    width={REACTION_SIZE}
                    height={REACTION_SIZE}
                    color={
                      state()?.part === "reactions" && state()?.reaction === index()
                        ? COLOUR.focus
                        : COLOUR.none
                    }
                  />
                  <poster
                    uri={icon(
                      // Filled when this is the rating the viewer gave. (`Reactions.kt:55`)
                      state()?.rating === face.rating
                        ? (RATING_ICONS[index()]?.filled ?? ICON.thumbUp)
                        : (RATING_ICONS[index()]?.plain ?? ICON.thumbUp),
                    )}
                    width={38}
                    height={38}
                    translation={[(REACTION_SIZE - 38) / 2, (REACTION_SIZE - 38) / 2]}
                    audioGuideText={say(seen().words, face.word)}
                  />
                </group>
              )}
            </For>
            {/* The menu. The line the remote is on is filled; the rest are plain. (`parts/Menu.kt`) */}
            <For each={linesOf(held)}>
              {(line, index: () => number) => (
                <group
                  translation={[PAGE_EDGE, TITLE_MENU.y + index() * (LINE_HIGH + LINE_GAP)]}
                >
                  <rectangle
                    width={TITLE_MENU.wide}
                    height={LINE_HIGH}
                    color={
                      state()?.part === "menu" && state()?.line === index()
                        ? COLOUR.tabHere
                        : COLOUR.none
                    }
                  />
                  <label
                    text={lineWord(line, held, state()?.saved ?? false, seen().words)}
                    translation={[16, (LINE_HIGH - 28) / 2]}
                    color={COLOUR.text}
                  />
                </group>
              )}
            </For>
            <Show when={held.runtime_min !== undefined || held.year !== undefined}>
              <label
                text={[held.year, held.runtime_min].filter((one) => one !== undefined).join("  ")}
                translation={[PAGE_EDGE, TITLE_FACTS_Y]}
                color={COLOUR.muted}
              />
            </Show>
          </group>
        )}
      </Show>
    </group>
  );
}

/** The sizes this page's two biggest labels want. */
export const TITLE_TEXT = { name: TITLE_NAME_SIZE, story: TITLE_STORY.size, reactions: REACTION_COUNT };
