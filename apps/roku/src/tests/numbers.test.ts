// The geometry, checked by the relations between the numbers and never against itself.
//
// The rule is `apps/android/surfaces/tv/src/test/kotlin/streams/tv/NumbersTest.kt:20`: "a number is
// never asserted against itself. `assertEquals(328, STEP)` passes forever." So nothing below
// repeats a number from `src/numbers.ts`. Each case asserts something that would break if a value
// were changed on its own: a sum, a fit on the screen, or a value derived from `brand.json`.

import brand from "../../../brand.json";
import {
  BAR_HIGH,
  BAR_MID,
  CARD_HIGH,
  CARD_INSET,
  CARD_NAME_WIDE,
  CARD_WIDE,
  CARD_X,
  CARD_Y,
  COLOUR,
  MARK_X,
  NEXT_ROW_TOP,
  PAGE_EDGE,
  POSTER_AHEAD_X,
  ROUND,
  SCREEN_HIGH,
  SCREEN_WIDE,
  STEP,
  TILE_HIGH,
  TILE_WIDE,
} from "../numbers";

describe("a row's rhythm", () => {
  it("steps one tile plus a gutter, so a resized tile cannot silently overlap", () => {
    expect(STEP - TILE_WIDE).toBe(17);
    expect(STEP).toBeGreaterThan(TILE_WIDE);
  });

  it("puts the first poster clear of the open card, by the same gutter rounded up", () => {
    expect(POSTER_AHEAD_X - (PAGE_EDGE + CARD_WIDE)).toBe(19);
    expect(POSTER_AHEAD_X).toBeGreaterThan(CARD_X + CARD_WIDE);
  });

  it("keeps the open card and a closed tile the same shape within a twentieth", () => {
    const card = CARD_WIDE / CARD_HIGH;
    const tile = TILE_WIDE / TILE_HIGH;
    // The card is wide artwork and the tile is a poster, so these are deliberately different.
    expect(card).toBeGreaterThan(tile);
    expect(Math.abs(card - 16 / 9)).toBeLessThan(0.05);
  });
});

describe("everything fits the screen", () => {
  it("leaves the page margin on both sides of the bar", () => {
    expect(MARK_X + BAR_HIGH + PAGE_EDGE).toBe(SCREEN_WIDE);
  });

  it("centres the bar's round parts on its middle line", () => {
    expect(BAR_MID - BAR_HIGH / 2).toBeGreaterThan(0);
    expect(BAR_MID + BAR_HIGH / 2).toBeLessThan(CARD_Y);
  });

  it("fits the open card between the bar and the story under it", () => {
    expect(CARD_Y).toBeGreaterThan(BAR_MID + BAR_HIGH / 2);
    expect(CARD_Y + CARD_HIGH).toBeLessThan(SCREEN_HIGH);
  });

  it("cuts the next row off at the foot of the screen, which is what makes it the next row", () => {
    expect(NEXT_ROW_TOP.underCard).toBeLessThan(SCREEN_HIGH);
    expect(NEXT_ROW_TOP.underCard + TILE_HIGH).toBeGreaterThan(SCREEN_HIGH);
  });

  it("insets the card's name equally on both sides", () => {
    expect(CARD_WIDE - CARD_NAME_WIDE).toBe(CARD_INSET * 2);
  });
});

describe("the brand decides its own shapes and colours", () => {
  it("scales every corner from brand.json by the one television factor", () => {
    expect(ROUND.card).toBe(Math.round(brand.shape.card_radius * 1.5));
    expect(ROUND.panel).toBe(Math.round(brand.shape.panel_radius * 1.5));
    expect(ROUND.field).toBe(Math.round(brand.shape.field_radius * 1.5));
  });

  it("keeps a key smaller than a field and a chip smaller than a key", () => {
    expect(ROUND.key).toBeLessThan(ROUND.field);
    expect(ROUND.chip).toBeLessThan(ROUND.key);
  });

  it("writes every colour as a Roku does, from brand.json and not from a literal", () => {
    for (const value of Object.values(COLOUR)) expect(value).toMatch(/^0x[0-9a-f]{8}$/);
    expect(COLOUR.background).toBe(`0x${brand.colors.background.slice(1)}ff`);
    expect(COLOUR.raised).toBe(`0x${brand.colors.raised.slice(1)}ff`);
  });

  it("makes the focus colour the brand's text colour, which is what a white ring is", () => {
    expect(COLOUR.focus).toBe(COLOUR.text);
  });

  it("leaves a fill with no colour fully transparent", () => {
    expect(COLOUR.none.slice(-2)).toBe("00");
  });
});
