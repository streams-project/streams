// The card at the top of a page: the title the page opens with, and the two things to do with it.
//
// A port of `apps/android/surfaces/tv/src/main/java/streams/tv/browse/HeroView.java:36-168` and the
// behaviour spec it cites, `tv/src/main/kotlin/streams/tv/browse/HeroCard.kt:46-99`. Where every
// part sits is `src/hero.ts`, apart from the drawing and checked without a Roku; this file holds no
// number and no English.
//
// `button` is 0 for Play, 1 for More info, and -1 when the remote is elsewhere on the page, which
// is `HeroView.java:140-141`'s own three answers: the card is still drawn then, with neither
// button lit.
//
// WHAT IS MISSING AND WHY. Two shades. `HeroCard.kt:43-44` draws a gradient dark on the side the
// words start from and a second up the card from 0.55 of its height; a Roku Rectangle takes one
// flat colour and no gradient node was read on Roku's pages. So the card draws the foot shade flat,
// over the part the words sit on, exactly as `src/screens/Rows.tsx` does for the open card's foot,
// and draws no side shade at all rather than a flat band that would darken the artwork evenly.

import { Show } from "@roku-sdk/solid";

import { wide } from "../art";
import type { Tile, TitleDetail } from "../contract";
import { facts as factsOf } from "../facts";
import { BUTTON_GAP, HERO, HERO_NAME, SHADE, heroLogo, heroStack, shadeTop } from "../hero";
import { ICON, icon } from "../icons";
import { COLOUR, STORY_LINE, STORY_SIZE, TITLE_LOGO } from "../numbers";
import { black, white } from "../parts";
import { KEY, say } from "../words";
import { FactsLine, Pill, pillWide } from "./Parts";

export function Hero(props: {
  lead: Tile;
  detail: TitleDetail | null;
  words: Record<string, string>;
  language: string;
  /** 0 Play, 1 More info, -1 the remote is somewhere else on the page. */
  button: number;
}) {
  const mark = (): string | null => heroLogo(props.lead, props.detail);
  const story = (): string => props.detail?.overview ?? "";
  /**
   * Whether there is a facts line at all, which is not the same as whether the details arrived: a
   * title with no genre, no year, no length and no age gives an empty line, and the stack closes
   * the gap rather than leaving a hole. (`HeroView.java:159-164`)
   */
  const hasFacts = (): boolean => {
    const held = props.detail;
    return held !== null && factsOf(held, props.words, props.language).length > 0;
  };
  const stack = (): { head: number; facts: number; story: number; buttons: number } =>
    heroStack(mark() !== null, hasFacts());
  const playWord = (): string => say(props.words, KEY.play);
  return (
    <group translation={[HERO.x, HERO.y]}>
      <rectangle width={HERO.wide} height={HERO.high} color={COLOUR.surface} />
      {/*
        The artwork is the title that is named, pictured and described on top of it, so it is
        decoration and carries no reading of its own. (`HeroView.java:31-33`)
      */}
      <Show when={wide(props.lead.images)}>
        {(url: string) => (
          <poster uri={url} width={HERO.wide} height={HERO.high} loadDisplayMode="scaleToFill" />
        )}
      </Show>
      <rectangle
        translation={[0, shadeTop()]}
        width={HERO.wide}
        height={HERO.high - shadeTop()}
        color={black(SHADE.strength)}
      />
      {/* The card's white edge, as four sides: a Rectangle has no border. (`HeroView.java:70`) */}
      <rectangle width={HERO.wide} height={HERO.edge} color={white(HERO.edgeShare)} />
      <rectangle
        translation={[0, HERO.high - HERO.edge]}
        width={HERO.wide}
        height={HERO.edge}
        color={white(HERO.edgeShare)}
      />
      <rectangle width={HERO.edge} height={HERO.high} color={white(HERO.edgeShare)} />
      <rectangle
        translation={[HERO.wide - HERO.edge, 0]}
        width={HERO.edge}
        height={HERO.high}
        color={white(HERO.edgeShare)}
      />
      {/*
        The lettering is the title's name drawn as a picture, so it says the name and the written
        name is not drawn as well. Without one, the name is words. (`HeroView.java:146-151`)
      */}
      <Show
        when={mark()}
        fallback={
          <label
            text={props.lead.name}
            translation={[HERO.inset, stack().head]}
            width={HERO.column}
            color={COLOUR.text}
            lineSpacing={HERO_NAME.line - HERO_NAME.size}
          />
        }
      >
        {(url: string) => (
          <poster
            uri={url}
            width={TITLE_LOGO.maxWide}
            height={TITLE_LOGO.maxHigh}
            translation={[HERO.inset, stack().head]}
            loadDisplayMode="scaleToFit"
            audioGuideText={props.lead.name}
          />
        )}
      </Show>
      <Show when={props.detail}>
        {(detail: TitleDetail) => (
          <FactsLine
            detail={detail}
            words={props.words}
            language={props.language}
            x={HERO.inset}
            y={stack().facts}
          />
        )}
      </Show>
      <label
        text={story()}
        translation={[HERO.inset, stack().story]}
        width={HERO.column}
        color={COLOUR.muted}
        lineSpacing={STORY_LINE - STORY_SIZE}
      />
      <Pill
        word={playWord()}
        on={props.button === 0}
        icon={icon(ICON.play)}
        x={HERO.inset}
        y={stack().buttons}
      />
      <Pill
        word={say(props.words, KEY.moreInfo)}
        on={props.button === 1}
        x={HERO.inset + pillWide(playWord(), icon(ICON.play)) + BUTTON_GAP}
        y={stack().buttons}
      />
    </group>
  );
}
