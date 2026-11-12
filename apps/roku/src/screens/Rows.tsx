// The row that owns the screen, and the dim row below it.
//
// A port of `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Rows.kt`. Every number it
// lays out with comes from `src/numbers.ts`, where each one carries the Kotlin line it was read
// from. Nothing here handles a key: the screen does that, once, in `Browse.tsx`.
//
// WHAT IS VERIFIED HERE. This file uses only the elements and fields read on Roku's own pages on
// 2026-11-12, and no others, because a drawing layer written from memory is worse than one with
// gaps:
//   * `<group ref translation>`, `<rectangle width height color>`, `<label text color translation>`
//     - from Roku's own example (https://developer.roku.com/dev/docs/rsg-sdk-usefocusable); JSX
//     element names are the SceneGraph node names in lower case.
//   * `<label lineSpacing>` - the Label node's own field list
//     (https://developer.roku.com/dev/docs/label, page updated 2026-11-10).
//   * `<poster uri width height loadDisplayMode blendColor>` - the Poster node's field list
//     (https://developer.roku.com/dev/docs/poster, page updated 2026-11-04).
//
// WHAT IS MISSING AND WHY. Two things the Kotlin does are left undone rather than guessed:
//   * The brand's 21 px card corner. Poster has no corner radius: its page says rounded corners
//     come from an `effect` node, with `shaderCompatible` set before `uri`. That needs the Effect
//     node's own field list, which was not read.
//   * A line limit on the story and the card's name. Neither `maxLines` nor `wrap` appears in the
//     Label page's field list, and the inherited LabelBase fields were not read.
// Dimming is done with `blendColor` rather than an opacity, because `opacity` is an inherited Group
// field that was not read either. A grey blend multiplies the artwork down, which is what the
// Kotlin's alpha does to it: 0.55 becomes 0x8c8c8cff and 0.5 becomes 0x808080ff.

import { For, Show } from "@roku-sdk/solid";

import { tall, wide } from "../art";
import type { Section, Tile, TitleDetail } from "../contract";
import {
  CARD_FOOT_HIGH,
  CARD_HIGH,
  CARD_INSET,
  CARD_NAME_BOTTOM,
  CARD_NAME_SIZE,
  CARD_WIDE,
  CARD_X,
  CARD_Y,
  COLOUR,
  FOCUS_RING,
  NEXT_POSTERS,
  NEXT_ROW_NAME_Y,
  NEXT_ROW_TOP,
  PAGE_EDGE,
  POSTER_AHEAD_X,
  POSTER_BEHIND_X,
  POSTER_Y,
  ROW_NAME_Y,
  STEP,
  STORY_LINE,
  STORY_SIZE,
  STORY_WIDE,
  STORY_Y,
  TILE_HIGH,
  TILE_WIDE,
} from "../numbers";

/** A grey that multiplies artwork down to the share of itself the Kotlin draws with an alpha. */
const PASSED = "0x8c8c8cff";
const BELOW = "0x808080ff";

/**
 * A tile in a row. The artwork is the title, so the title's name is what a person hears in place of
 * seeing it; `audioGuideText` is the Poster field that carries it. (`Rows.kt:55-66`)
 */
function Poster(props: { tile: Tile; x: number; y: number; blend?: string }) {
  return (
    <group translation={[props.x, props.y]}>
      <rectangle width={TILE_WIDE} height={TILE_HIGH} color={COLOUR.raised} />
      <Show when={tall(props.tile.images)}>
        {(url: string) => (
          <poster
            uri={url}
            width={TILE_WIDE}
            height={TILE_HIGH}
            loadDisplayMode="scaleToFill"
            blendColor={props.blend ?? COLOUR.text}
            audioGuideText={props.tile.name}
          />
        )}
      </Show>
    </group>
  );
}

/**
 * The card stays still and its art changes under it. The name is drawn over the art, and the ring
 * says this is the one card of the row that is open. (`Rows.kt:80-112`)
 */
function OpenCard(props: { tile: Tile }) {
  return (
    <group translation={[CARD_X, CARD_Y]}>
      <rectangle width={CARD_WIDE} height={CARD_HIGH} color={COLOUR.raised} />
      <Show when={wide(props.tile.images)}>
        {(url: string) => (
          <poster
            uri={url}
            width={CARD_WIDE}
            height={CARD_HIGH}
            loadDisplayMode="scaleToFill"
            audioGuideText={props.tile.name}
          />
        )}
      </Show>
      {/* The shade the name sits on, at the foot of the card. (`Rows.kt:53,96`) */}
      <rectangle
        translation={[0, CARD_HIGH - CARD_FOOT_HIGH]}
        width={CARD_WIDE}
        height={CARD_FOOT_HIGH}
        color="0x000000b3"
      />
      <label
        text={props.tile.name}
        translation={[CARD_INSET, CARD_HIGH - CARD_NAME_BOTTOM - CARD_NAME_SIZE]}
        color={COLOUR.text}
      />
      {/* The ring, as four edges: a Rectangle has no border. (`Rows.kt:110`) */}
      <rectangle width={CARD_WIDE} height={FOCUS_RING} color={COLOUR.focus} />
      <rectangle
        translation={[0, CARD_HIGH - FOCUS_RING]}
        width={CARD_WIDE}
        height={FOCUS_RING}
        color={COLOUR.focus}
      />
      <rectangle width={FOCUS_RING} height={CARD_HIGH} color={COLOUR.focus} />
      <rectangle
        translation={[CARD_WIDE - FOCUS_RING, 0]}
        width={FOCUS_RING}
        height={CARD_HIGH}
        color={COLOUR.focus}
      />
    </group>
  );
}

/** The row that owns the screen: its focused title opens wide with the next ones beside it. (`Rows.kt:114-156`) */
export function OpenRow(props: {
  section: Section;
  at: number;
  detail: TitleDetail | null;
  /** 1 when lines run left to right, -1 when they run right to left. (`Round.kt:57-59`) */
  along: number;
}) {
  const tiles = (): Tile[] => props.section.tiles ?? [];
  const tile = (): Tile => tiles()[Math.min(props.at, tiles().length - 1)]!;
  const ahead = (): { tile: Tile; index: number }[] =>
    tiles()
      .map((one, index) => ({ tile: one, index }))
      .filter((one) => one.index > props.at);
  return (
    <group>
      <label text={props.section.title ?? ""} translation={[PAGE_EDGE, ROW_NAME_Y]} color={COLOUR.text} />
      <Show when={props.at > 0}>
        <Poster tile={tiles()[props.at - 1]!} x={POSTER_BEHIND_X} y={POSTER_Y} blend={PASSED} />
      </Show>
      {/*
        The posters after the open card. In a right to left language the row runs leftwards, so
        every step along it multiplies by `along`, which is -1 there. (`Rows.kt:120-121,137`)
      */}
      <For each={ahead()}>
        {(one: { tile: Tile; index: number }) => (
          <Poster
            tile={one.tile}
            x={POSTER_AHEAD_X + (one.index - 1 - props.at) * STEP * props.along}
            y={POSTER_Y}
          />
        )}
      </For>
      <OpenCard tile={tile()} />
      <Show when={props.detail}>
        {(detail: TitleDetail) => (
          <label
            text={detail.overview ?? ""}
            translation={[PAGE_EDGE, STORY_Y]}
            width={STORY_WIDE}
            color={COLOUR.muted}
            lineSpacing={STORY_LINE - STORY_SIZE}
          />
        )}
      </Show>
    </group>
  );
}

/**
 * The next row, dim, cut by the screen's edge. It shows where the viewer left it: the kept tile
 * first, the one before it at the edge, as the open row does. (`Rows.kt:158-178`)
 */
export function NextRow(props: { section: Section; high: boolean; at: number }) {
  const tiles = (): Tile[] => props.section.tiles ?? [];
  const top = (): number => (props.high ? NEXT_ROW_TOP.underHero : NEXT_ROW_TOP.underCard);
  const name = (): number => (props.high ? NEXT_ROW_NAME_Y.underHero : NEXT_ROW_NAME_Y.underCard);
  const shown = (): Tile[] => tiles().slice(props.at, props.at + NEXT_POSTERS);
  return (
    <group>
      <label text={props.section.title ?? ""} translation={[PAGE_EDGE, name()]} color={COLOUR.muted} />
      <Show when={props.at > 0 && tiles()[props.at - 1] !== undefined}>
        <Poster tile={tiles()[props.at - 1]!} x={PAGE_EDGE - STEP} y={top()} blend={BELOW} />
      </Show>
      <For each={shown()}>
        {(tile: Tile, index: () => number) => (
          <Poster tile={tile} x={PAGE_EDGE + index() * STEP} y={top()} blend={BELOW} />
        )}
      </For>
    </group>
  );
}
