// Who is watching, search, the tabs and the mark, at the top of every signed in page.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/TopBar.kt`. The bar draws
// and nothing else: which tab the remote is on is the page's state, handed in as `focus`.
//
// `words` is here for the two parts that are a picture and nothing else - the profile and search -
// because a picture with no words is unreachable from a remote. The tabs carry the names the server
// sent with them, and the mark is decoration. (`TopBar.kt:63-70`)
//
// OPEN: the brand's pill. A focused tab is a white pill and a Rectangle has no corner radius, so
// the fill below is square. The two ways out are a 9-patch image shipped in the package or the
// Effect node named on Poster's page; neither was read, so neither is written. See the README.
//
// OPEN: the mark. Every other surface draws `{server}/mark.svg`
// (`apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Header.kt:87`,
// the deleted television page's `topbar.js` (gone 2026-11-12; `apps/android/surfaces/tv/.../TopBar.kt:37-39` holds these numbers)) and the brand ships SVG
// (`brand/streams/README.md:8-9`). Whether a Roku Poster reads SVG at all was not established, so
// the address below asks for PNG. Either the server grows one or the package carries it.
//
// The search glyph is not a server address: it is a name from the shared icon set, through
// `src/icons.ts`. Nothing produces the file yet - see that file.

import { For, Show } from "@roku-sdk/solid";

import type { NavItem, Profile } from "../contract";
import {
  BAR_HIGH,
  BAR_MID,
  COLOUR,
  FACE,
  FOCUS_RING,
  MARK_X,
  PAGE_EDGE,
  SEARCH_X,
  TABS_X,
  TAB_GAP,
  TAB_INSET,
  TAB_LIMIT,
  TAB_SIZE,
} from "../numbers";
import { ICON, icon } from "../icons";
import { KEY, say } from "../words";
import { SEARCH_TAB, WHO_TAB } from "../focus";

/** The tabs that fit the bar. Genres live on their own pages. (`TopBar.kt:42`) */
export function tabsOf(nav: NavItem[]): NavItem[] {
  return nav.filter((item) => item.screen !== "genre").slice(0, TAB_LIMIT);
}

/** How wide a tab's pill is: its word, with the inset on each side. An estimate until measured on a set. */
function tabWide(label: string): number {
  return Math.round(label.length * TAB_SIZE * 0.56) + TAB_INSET * 2;
}

function Tab(props: { item: NavItem; active: boolean; on: boolean; x: number }) {
  const back = (): string => {
    if (props.on) return COLOUR.focus;
    if (props.active) return COLOUR.tabHere;
    return COLOUR.none;
  };
  return (
    <group translation={[props.x, BAR_MID - BAR_HIGH / 2]}>
      <rectangle width={tabWide(props.item.label)} height={BAR_HIGH} color={back()} />
      <label
        text={props.item.label}
        translation={[TAB_INSET, (BAR_HIGH - TAB_SIZE) / 2]}
        color={props.on ? COLOUR.background : COLOUR.text}
      />
    </group>
  );
}

export function TopBar(props: {
  tabs: NavItem[];
  active: number;
  /** Which part the remote is on, or -1 when it is elsewhere on the page. */
  focus: number;
  person: Profile | null;
  server: string;
  words: Record<string, string>;
}) {
  const places = (): { item: NavItem; x: number; at: number }[] => {
    let x = TABS_X;
    return props.tabs.map((item, at) => {
      const place = { item, x, at };
      x += tabWide(item.label) + TAB_GAP;
      return place;
    });
  };
  return (
    <group>
      {/* Who is watching. The face is a picture, so its words come from the profile's name. */}
      <group translation={[PAGE_EDGE - FOCUS_RING, BAR_MID - FACE / 2 - FOCUS_RING]}>
        {/* The ring the Kotlin draws as a 3 px border, as a plate behind the face. (`TopBar.kt:85`) */}
        <rectangle
          width={FACE + FOCUS_RING * 2}
          height={FACE + FOCUS_RING * 2}
          color={props.focus === WHO_TAB ? COLOUR.focus : COLOUR.none}
        />
        <Show when={props.person}>
          {(person: Profile) => (
            <poster
              uri={person.avatar ?? ""}
              width={FACE}
              height={FACE}
              translation={[FOCUS_RING, FOCUS_RING]}
              audioGuideText={person.name}
              blendColor={COLOUR.text}
            />
          )}
        </Show>
      </group>
      {/* Search. A picture with no words, so it carries the key's word. */}
      <group translation={[SEARCH_X, BAR_MID - BAR_HIGH / 2]}>
        <rectangle
          width={BAR_HIGH}
          height={BAR_HIGH}
          color={props.focus === SEARCH_TAB ? COLOUR.focus : COLOUR.none}
        />
        <poster
          uri={icon(ICON.search)}
          width={34}
          height={34}
          translation={[(BAR_HIGH - 34) / 2, (BAR_HIGH - 34) / 2]}
          audioGuideText={say(props.words, KEY.navSearch)}
          blendColor={props.focus === SEARCH_TAB ? COLOUR.background : COLOUR.text}
        />
      </group>
      <For each={places()}>
        {(place: { item: NavItem; x: number; at: number }) => (
          <Tab
            item={place.item}
            active={place.at === props.active}
            on={place.at === props.focus}
            x={place.x}
          />
        )}
      </For>
      {/* The mark: the same on every page, and it says nothing a viewer needs. (`TopBar.kt:117`) */}
      <poster
        uri={`${props.server}/mark.png`}
        width={BAR_HIGH}
        height={BAR_HIGH}
        translation={[MARK_X, BAR_MID - BAR_HIGH / 2]}
      />
    </group>
  );
}
