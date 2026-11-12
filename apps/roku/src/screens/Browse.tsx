// A page of rows. The parts draw, and this file only knows where the focus is and what a key does.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Browse.kt:154-301`. The
// focus itself is `src/focus.ts` and which rows a page may draw is `src/page.ts`, both apart from
// the drawing and both checked without a Roku.

import { Show, createMemo, createSignal } from "@roku-sdk/solid";

import { ROUTE, tabAddress, titleAddress, watchAddress } from "../api";
import { ADDRESS } from "../contract";
import type { More, Section, TitleDetail } from "../contract";
import { PageFocus, SEARCH_TAB, WHO_TAB } from "../focus";
import { COLOUR, REACH } from "../numbers";
import { answerIsThisPage, heroOn, rowsOn, screenOf } from "../page";
import type { Shell } from "../ports";
import { KEY, say, sayBoth } from "../words";
import type { Word } from "../keys";
import { NextRow, OpenRow } from "./Rows";
import { TopBar, tabsOf } from "./TopBar";
import { screenRemote } from "./remote";

/**
 * The rows a page has past its first answer. The server builds up to 50 rows and hands a screen 12
 * with a cursor to read the rest. (`Browse.kt:60-73`)
 */
class PageGrowth {
  /** The cursor the first answer came with. A fresh answer carries a new one and starts the page again. */
  from: string | null = null;
  /** The next cursor to read, or nothing when the page has given everything it has. */
  next: string | null = null;
  busy = false;
  extra: Section[] = [];
}

export function Browse(props: { shell: Shell }) {
  const seen = () => props.shell.seen();
  const growth = new PageGrowth();
  const [extra, setExtra] = createSignal<Section[]>([]);
  const [detail, setDetail] = createSignal<TitleDetail | null>(null);

  const current = createMemo(() => screenOf(seen().route?.screen));
  const asked = createMemo(() => seen().route?.screen ?? null);
  const data = createMemo(() => seen().screen);
  const mine = createMemo(() => {
    const screen = data();
    return screen !== null && answerIsThisPage(current(), asked(), screen.name);
  });
  const rows = createMemo(() => {
    const screen = data();
    if (screen === null) return [];
    return rowsOn(current(), asked(), screen.name, screen.sections, extra());
  });
  const lead = createMemo(() => heroOn(current(), data()?.sections ?? []));
  const tabs = createMemo(() => tabsOf(seen().nav));
  const active = createMemo(() => {
    const at = tabs().findIndex((item) => item.screen === current());
    return at < 0 ? 0 : at;
  });
  const person = createMemo(() => {
    const all = seen().profiles;
    return all.find((one) => one.id === seen().active) ?? all[0] ?? null;
  });

  // The page's focus. It is built once for the page and then told what the page now holds, so a
  // fresh answer that reordered the rows does not throw away where the viewer was. (`Browse.kt:176-181`)
  const focus = new PageFocus(rows(), tabs().length, active(), lead() !== null);
  // `PageFocus` is plain mutable state, deliberately: it is the part that had to be checkable
  // without a renderer. SolidJS cannot see into it, so every read of it goes through this counter
  // and every press bumps it. One signal for the whole screen is also what the surface draws like:
  // a press moves the row, the card, the posters and the story at once.
  const [moved, setMoved] = createSignal(0);
  const again = (): void => setMoved(moved() + 1);

  /**
   * The page reads its next rows as the focus comes within reach of the last one it holds. A remote
   * cannot flick past twelve rows, so the rows arrive before they are wanted rather than on a
   * gesture. (`Browse.kt:196-216`)
   */
  function grow(): void {
    if (!mine()) return;
    const screen = data();
    if (screen === null) return;
    if (growth.from !== (screen.cursor ?? null)) {
      growth.from = screen.cursor ?? null;
      growth.next = screen.cursor ?? null;
      growth.extra = [];
      growth.busy = false;
      setExtra([]);
    }
    const cursor = growth.next;
    if (cursor === null || growth.busy || !focus.withinReach(REACH)) return;
    growth.busy = true;
    props.shell.ask(ROUTE.more(cursor), (status, text) => {
      growth.busy = false;
      if (status < 200 || status > 299) return;
      let more: More;
      try {
        more = JSON.parse(text) as More;
      } catch {
        return;
      }
      const sections = (more.sections ?? []).filter((one) => (one.tiles ?? []).length > 0);
      // An answer with no rows left is the end, whatever cursor came with it.
      growth.next = (more.sections ?? []).length === 0 ? null : (more.cursor ?? null);
      growth.extra = growth.extra.concat(sections);
      setExtra(growth.extra);
      again();
    });
  }

  /** The open title's own facts, which arrive after the row has moved. (`Rows.kt:123`) */
  function look(): void {
    const tile = (rows()[focus.row]?.tiles ?? [])[focus.column];
    if (tile === undefined) {
      setDetail(null);
      return;
    }
    const [kind, id] = tile.title.split(":");
    setDetail(null);
    props.shell.ask(ROUTE.title(kind!, id!), (status, text) => {
      if (status < 200 || status > 299) return;
      const got = JSON.parse(text) as TitleDetail;
      if (got.title === tile.title) setDetail(got);
    });
  }

  /** What OK means where the remote is. (`Browse.kt:218-230`) */
  function choose(): void {
    const choice = focus.choose(lead());
    switch (choice.kind) {
      case "who":
        props.shell.go(ADDRESS.PROFILES);
        return;
      case "search":
        props.shell.go(tabAddress("search"));
        return;
      case "tab": {
        const item = tabs()[choice.at];
        if (item !== undefined) props.shell.go(tabAddress(item.screen, item.genre));
        return;
      }
      case "title":
        props.shell.go(titleAddress(choice.title));
        return;
      case "watch":
        props.shell.go(watchAddress(choice.title));
        return;
      default:
        return;
    }
  }

  function read(word: Word): boolean {
    focus.rows = rows();
    focus.tabs = tabs().length;
    focus.hero = lead() !== null;
    if (word === "ok") {
      choose();
      return true;
    }
    const was = `${focus.where} ${focus.row} ${focus.column}`;
    if (focus.press(word)) {
      again();
      if (`${focus.where} ${focus.row} ${focus.column}` !== was) look();
      grow();
      return true;
    }
    // Back from the top of any page but Home goes Home, and on Home it stops here: the app stays
    // open. The owner holds whether Back on Home should exit (`Browse.kt:238-251`, gap 7).
    if (word === "back") {
      if (current() !== "home") props.shell.go(ADDRESS.HOME);
      return true;
    }
    return false;
  }

  const { ref } = screenRemote({
    read,
    rtl: () => seen().rtl,
    says: () => here(),
  });

  /**
   * What a person hears as the remote moves. A television draws its own focus, so without this
   * nothing announces where the remote has arrived. (`Browse.kt:255-266`)
   */
  function here(): string {
    void moved();
    const words = seen().words;
    if (focus.where === "top" && focus.tab === WHO_TAB) return person()?.name ?? "";
    if (focus.where === "top" && focus.tab === SEARCH_TAB) return say(words, KEY.navSearch);
    if (focus.where === "top") return tabs()[focus.tab]?.label ?? "";
    if (focus.where === "hero") return say(words, focus.button === 1 ? KEY.moreInfo : KEY.play);
    const row = rows()[focus.row];
    return sayBoth((row?.tiles ?? [])[focus.column]?.name, row?.title);
  }

  const inRow = createMemo(() => {
    void moved();
    return focus.where === "row";
  });
  const open = createMemo(() => {
    void moved();
    return rows()[focus.row] ?? null;
  });
  const below = createMemo(() => {
    void moved();
    return rows()[inRow() ? focus.row + 1 : 0] ?? null;
  });

  grow();
  look();

  return (
    <group ref={ref}>
      <rectangle width={1920} height={1080} color={COLOUR.background} />
      <TopBar
        tabs={tabs()}
        active={active()}
        focus={focus.where === "top" ? focus.tab : -1}
        person={person()}
        server={seen().server}
        words={seen().words}
      />
      <Show when={inRow() ? open() : null}>
        {(section: Section) => (
          <OpenRow
            section={section}
            at={focus.column}
            detail={detail()}
            along={seen().rtl ? -1 : 1}
          />
        )}
      </Show>
      <Show when={!inRow() && rows().length === 0}>
        <label
          text={say(seen().words, current() === "games" ? KEY.gamesSoon : KEY.mineEmpty)}
          translation={[72, 500]}
          color={COLOUR.muted}
        />
      </Show>
      <Show when={below()}>
        {(section: Section) => (
          <NextRow section={section} high={!inRow()} at={focus.columnOf(inRow() ? focus.row + 1 : 0)} />
        )}
      </Show>
    </group>
  );
}
