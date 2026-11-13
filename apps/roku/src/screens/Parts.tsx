// The three small parts more than one screen draws: the facts line, the large button, the marks on
// a tile.
//
// Ports of `apps/android/surfaces/tv/src/main/java/streams/tv/parts/Facts.java:30-73`,
// `parts/Badges.java:34-70` and `tv/src/main/kotlin/streams/tv/parts/Pill.kt:30-53`. Every number
// they lay out with is in `src/parts.ts`, beside the line it came from; this file holds none.
//
// WHAT IS VERIFIED HERE. Only the elements and fields read on Roku's own pages on 2026-11-12, as
// `src/screens/Rows.tsx` says at its head: `<group translation>`, `<rectangle width height color>`,
// `<label text color translation>`, `<poster uri width height blendColor audioGuideText>`.
//
// WHAT IS MISSING AND WHY, both the same gap `Rows.tsx` already carries:
//   * The dot between two facts is round in the Kotlin (`Facts.java:63` draws it with
//     `Round.round`) and square here, and so is a badge's plate and a button's pill, because a
//     Rectangle has no corner radius and Poster's page says rounded corners come from an `effect`
//     node whose field list was not read.
//   * A word's width cannot be asked of a Roku before it is drawn, so where the next thing along a
//     line starts is an estimate from `src/type.ts`, in one place, named as one.

import { For, Show } from "@roku-sdk/solid";

import type { Badge, TitleDetail } from "../contract";
import { badgeWords } from "../badges";
import { facts as factsOf } from "../facts";
import { COLOUR } from "../numbers";
import { BADGE, FACTS, PILL, badgeHigh, black } from "../parts";
import { textWide, laidOut } from "../type";

/**
 * The line of facts about a title, in one order on every screen: genre, year, length, age.
 *
 * The order is `src/facts.ts`'s, which mirrors `parts/Facts.kt:37-48` and is held to the contract's
 * own fixtures. Nothing here re-derives it: a view that did would be a second place the order
 * lives (`Facts.java:19-25`).
 *
 * The dots carry no word, so they are drawn and never announced: a line that spoke them would read
 * "Drama dot 2024 dot one hour forty-one minutes".
 */
export function FactsLine(props: {
  detail: TitleDetail;
  words: Record<string, string>;
  language: string;
  x: number;
  y: number;
}) {
  const said = (): string[] => factsOf(props.detail, props.words, props.language);
  const places = (): { word: string; x: number }[] => {
    const line = said();
    const { at } = laidOut(line, FACTS.size, FACTS.side * 2 + FACTS.dot);
    return line.map((word, index) => ({ word, x: at[index] ?? 0 }));
  };
  return (
    <group translation={[props.x, props.y]}>
      <For each={places()}>
        {(place: { word: string; x: number }, index: () => number) => (
          <group>
            <Show when={index() > 0}>
              <rectangle
                translation={[place.x - FACTS.side - FACTS.dot, (FACTS.size - FACTS.dot) / 2]}
                width={FACTS.dot}
                height={FACTS.dot}
                color={COLOUR.muted}
              />
            </Show>
            <label text={place.word} translation={[place.x, 0]} color={COLOUR.text} />
          </group>
        )}
      </For>
    </group>
  );
}

/** How wide the facts line comes to, for a caller that has to fit something beside it. */
export function factsWide(detail: TitleDetail, words: Record<string, string>, language: string): number {
  return laidOut(factsOf(detail, words, language), FACTS.size, FACTS.side * 2 + FACTS.dot).wide;
}

/**
 * A large button. In focus it turns white, as every other television surface draws it.
 *
 * A button with no word at all draws nothing rather than an empty white shape: `Pill.kt:40` returns
 * there, and note 206 measured what the other reading looks like on a real device - a `Pill("")` on
 * black with every word around it missing. The icon beside the word is decoration: it repeats the
 * word, so it carries no reading of its own.
 */
export function Pill(props: { word: string; on: boolean; icon?: string; x: number; y: number }) {
  const wide = (): number =>
    textWide(props.word, PILL.size) +
    PILL.inset * 2 +
    (props.icon === undefined ? 0 : PILL.icon + PILL.gap);
  const ink = (): string => (props.on ? COLOUR.background : COLOUR.text);
  return (
    <Show when={props.word !== "" || props.icon !== undefined}>
      <group translation={[props.x, props.y]}>
        <rectangle
          width={wide()}
          height={PILL.high}
          color={props.on ? COLOUR.focus : COLOUR.tabHere}
        />
        <Show when={props.icon}>
          {(name: string) => (
            <poster
              uri={name}
              width={PILL.icon}
              height={PILL.icon}
              translation={[PILL.inset, (PILL.high - PILL.icon) / 2]}
              blendColor={ink()}
            />
          )}
        </Show>
        <label
          text={props.word}
          translation={[
            PILL.inset + (props.icon === undefined ? 0 : PILL.icon + PILL.gap),
            (PILL.high - PILL.size) / 2,
          ]}
          color={ink()}
        />
      </group>
    </Show>
  );
}

/** How wide a button comes to, so the one beside it knows where to start. */
export function pillWide(word: string, icon: string | null): number {
  return textWide(word, PILL.size) + PILL.inset * 2 + (icon === null ? 0 : PILL.icon + PILL.gap);
}

/**
 * The marks on a tile, as small plates at its bottom left.
 *
 * Handed words and never a `Badge`: turning one into the other is `src/badges.ts`, for the reason
 * `Badges.java:17-27` gives. A tile with no badge that carries a word draws nothing at all, so an
 * empty list leaves no plate on the artwork.
 */
export function TileBadges(props: {
  badges: Badge[] | undefined;
  words: Record<string, string>;
  x: number;
  y: number;
}) {
  const places = (): { word: string; x: number }[] => {
    const said = badgeWords(props.badges, props.words);
    let x = 0;
    return said.map((word) => {
      const place = { word, x };
      x += textWide(word, BADGE.size) + BADGE.across * 2 + BADGE.gap;
      return place;
    });
  };
  return (
    <group translation={[props.x, props.y]}>
      <For each={places()}>
        {(place: { word: string; x: number }) => (
          <group translation={[place.x, 0]}>
            <rectangle
              width={textWide(place.word, BADGE.size) + BADGE.across * 2}
              height={badgeHigh()}
              color={black(BADGE.plate)}
            />
            <label
              text={place.word}
              translation={[BADGE.across, BADGE.down]}
              color={COLOUR.text}
            />
          </group>
        )}
      </For>
    </group>
  );
}
