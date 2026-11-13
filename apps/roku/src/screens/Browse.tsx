// A page of rows. The parts draw, and this file only knows where the focus is and what a key does.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Browse.kt:154-301` and of
// the view that draws it today, `tv/src/main/java/streams/tv/browse/BrowseView.java:139-280`. The
// focus itself is `src/focus.ts`, which rows a page may draw is `src/page.ts`, and how a page
// follows its cursor is `src/growth.ts` - all three apart from the drawing and all three checked
// without a Roku.
//
// **What changed on 2026-11-13.** This file carried its own `PageGrowth` class and its own reading
// of the paging rule, written the day before `src/growth.ts` mirrored that rule from the Kotlin and
// held it to `platform/contract/fixtures/2026-11-05/valid/More.section.json`. Two readings of one
// rule is the drift the whole mirror exists to prevent, and the fixture-held one is the one that
// survives: the local class is gone and this page calls `PageGrowth` and `PageFocus.wantsMore`.
// The one case that fixture carries - a cursor with no sections behind it - is why it matters: a
// page that followed a cursor because it was there would read that file forever.

import { Show, createMemo, createSignal } from "@roku-sdk/solid";

import { ROUTE, tabAddress, titleAddress, watchAddress } from "../api";
import { ADDRESS } from "../contract";
import type { More, Section, Tile, TitleDetail } from "../contract";
import { PageFocus, SEARCH_TAB, WHO_TAB } from "../focus";
import { PageGrowth } from "../growth";
import { COLOUR, PAGE_EDGE, ROW_NAME_SIZE, SCREEN_HIGH, SCREEN_WIDE } from "../numbers";
import { answerIsThisPage, heroOn, rowsOn, screenOf } from "../page";
import type { Shell } from "../ports";
import { KEY, say, sayBoth } from "../words";
import type { Word } from "../keys";
import { Hero } from "./Hero";
import { NextRow, OpenRow } from "./Rows";
import { TopBar, tabsOf } from "./TopBar";
import { screenRemote } from "./remote";

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
   * gesture. Every branch of the rule is `src/growth.ts`; this is the part that has a network.
   * (`Browse.kt:196-216`)
   */
  function grow(): void {
    if (!mine()) return;
    const screen = data();
    if (screen === null) return;
    // A fresh answer is a fresh page: what was read past its first rows belongs to the answer it
    // came from. `answered` says when that happened, and the drawn rows follow it.
    if (growth.answered(screen.cursor)) setExtra([]);
    const cursor = growth.reads(focus.wantsMore());
    if (cursor === null) return;
    props.shell.ask(ROUTE.more(cursor), (status, text) => {
      if (status < 200 || status > 299) {
        growth.failed();
        return;
      }
      let more: More;
      try {
        more = JSON.parse(text) as More;
      } catch {
        growth.failed();
        return;
      }
      growth.took(more);
      setExtra(growth.extra);
      again();
    });
  }

  /**
   * The title the remote is on: the open row's tile, or the page's hero when the remote is above
   * the rows. Its own facts arrive after the move. (`BrowseView.java:165-176`, `Browse.kt:131-137`)
   */
  function onTile(): Tile | null {
    if (focus.where === "row") return (rows()[focus.row]?.tiles ?? [])[focus.column] ?? null;
    return lead();
  }

  /** Reads what is known about that title. (`Rows.kt:123`) */
  function look(): void {
    const tile = onTile();
    if (tile === null) {
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
  /** 0 Play, 1 More info, and -1 when the remote is elsewhere. (`BrowseView.java:140-141,256`) */
  const onButton = createMemo(() => {
    void moved();
    return focus.where === "hero" ? focus.button : -1;
  });

  grow();
  look();

  return (
    <group ref={ref}>
      <rectangle width={SCREEN_WIDE} height={SCREEN_HIGH} color={COLOUR.background} />
      <TopBar
        tabs={tabs()}
        active={active()}
        focus={focus.where === "top" ? focus.tab : -1}
        person={person()}
        server={seen().server}
        words={seen().words}
      />
      {/*
        The hero is the page's own card and the open row takes its place: a page shows one or the
        other, never both. (`BrowseView.java:253-256`)
      */}
      <Show when={inRow() ? null : lead()}>
        {(tile: Tile) => (
          <Hero
            lead={tile}
            detail={detail()}
            words={seen().words}
            language={seen().language}
            button={onButton()}
          />
        )}
      </Show>
      <Show when={inRow() ? open() : null}>
        {(section: Section) => (
          <OpenRow
            section={section}
            at={focus.column}
            detail={detail()}
            words={seen().words}
            language={seen().language}
            along={seen().rtl ? -1 : 1}
          />
        )}
      </Show>
      {/*
        A page with no hero and no rows of its own says so, in the viewer's language. Which of the
        two words it is, is this page's business and not the screen's. (`Browse.kt:115-120`,
        `BrowseView.java:259-260`)
      */}
      <Show when={!inRow() && lead() === null && rows().length === 0}>
        <label
          text={say(seen().words, current() === "games" ? KEY.gamesSoon : KEY.mineEmpty)}
          // `BrowseView.java:117` lays this across the whole page and centres it both ways. Of a
          // Roku Label, only `text`, `color`, `translation`, `width` and `lineSpacing` were read,
          // and nothing that aligns a word inside its box, so the line starts on the page margin
          // and sits on the middle of the screen.
          translation={[PAGE_EDGE, Math.round((SCREEN_HIGH - ROW_NAME_SIZE) / 2)]}
          width={SCREEN_WIDE - PAGE_EDGE * 2}
          color={COLOUR.muted}
        />
      </Show>
      <Show when={below()}>
        {(section: Section) => (
          <NextRow
            section={section}
            high={!inRow()}
            at={focus.columnOf(inRow() ? focus.row + 1 : 0)}
            words={seen().words}
          />
        )}
      </Show>
    </group>
  );
}
