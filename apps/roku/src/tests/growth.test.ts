// How a page pages and how far ahead it reads, held against the fixture corpus.
//
// `platform/contract/fixtures/2026-11-05/valid/More.section.json` is the answer this rule consumes,
// and `Screen.home.json` is the first answer it starts from. Both are files
// `platform/contract/tests/fixtures.rs` reads with `fs::read_dir` and round-trips through the Rust
// contract types; its first line says they are "plain JSON and every language reads the same
// files".
//
// The fixture is worth more here than an invented answer, and for a reason this file turns into a
// case: `More.section.json` carries a cursor and NO sections. A reading of the rule that followed
// the cursor because it was there would loop forever on that exact file.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { More, Screen, Section } from "../contract";
import { PageFocus } from "../focus";
import { PageGrowth } from "../growth";
import { REACH } from "../numbers";

const ROOT = join(__dirname, "..", "..", "..", "..");
const CORPUS = join(ROOT, "platform", "contract", "fixtures", "2026-11-05");

function fixture(kind: string, name: string): unknown {
  return JSON.parse(readFileSync(join(CORPUS, kind, name), "utf8")) as unknown;
}

// The generated `SectionKind` ends in `(string & {})` so that an unknown kind reads as a plain row
// instead of failing (`platform/contract/README.md:41`), and TypeScript will not widen a `string`
// from a JSON file into it. That is note 219's "an open enum cannot be type-checked in TypeScript",
// and it is why the fixture is read through `unknown` here. The `describe` below is the guard that
// takes the place of the compiler.
const home = fixture("valid", "Screen.home.json") as Screen;
const more = fixture("valid", "More.section.json") as More;

function rows(count: number): Section[] {
  return Array.from({ length: count }, (_unused, at) => ({
    id: `s${at}`,
    kind: "row",
    tiles: [{ name: "t", title: "movie:1" as const }],
  }));
}

describe("the corpus holds what these cases read", () => {
  it("Screen.home carries a cursor for the rest of the page", () => {
    expect(home.cursor).toBe("c-page-2");
    expect(home.sections.length).toBe(2);
  });

  it("More.section carries a cursor and no sections at all", () => {
    expect(more.cursor).toBe("c-row-2");
    expect(more.sections).toBeUndefined();
    expect((more.tiles ?? []).length).toBe(1);
  });
});

describe("a page follows its cursor", () => {
  it("starts from the cursor the first answer came with", () => {
    const growth = new PageGrowth();
    expect(growth.answered(home.cursor)).toBe(true);
    expect(growth.next).toBe("c-page-2");
    expect(growth.extra).toEqual([]);
  });

  it("does not start again for the same answer read twice", () => {
    const growth = new PageGrowth();
    growth.answered(home.cursor);
    growth.took({ cursor: "c-page-3", sections: rows(2) });
    expect(growth.extra.length).toBe(2);
    expect(growth.answered(home.cursor)).toBe(false);
    expect(growth.extra.length).toBe(2);
  });

  it("throws away what it read when a fresh answer arrives", () => {
    const growth = new PageGrowth();
    growth.answered(home.cursor);
    growth.took({ cursor: "c-page-3", sections: rows(2) });
    expect(growth.answered("c-page-9")).toBe(true);
    expect(growth.extra).toEqual([]);
    expect(growth.next).toBe("c-page-9");
  });

  it("stops at an answer with no rows, whatever cursor came with it", () => {
    // This is `More.section.json` itself: a cursor, and nothing behind it.
    const growth = new PageGrowth();
    growth.answered(home.cursor);
    growth.reads(true);
    growth.took(more);
    expect(growth.next).toBeNull();
    expect(growth.extra).toEqual([]);
    expect(growth.reads(true)).toBeNull();
  });

  it("drops a row that came back with no tiles", () => {
    const growth = new PageGrowth();
    growth.answered(home.cursor);
    growth.took({ cursor: "c-page-3", sections: [{ id: "empty", kind: "row" }, ...rows(1)] });
    expect(growth.extra.map((section) => section.id)).toEqual(["s0"]);
  });

  it("reads once, however many times the focus moves while the answer is in flight", () => {
    const growth = new PageGrowth();
    growth.answered(home.cursor);
    expect(growth.reads(true)).toBe("c-page-2");
    expect(growth.reads(true)).toBeNull();
    expect(growth.reads(true)).toBeNull();
    growth.took({ cursor: "c-page-3", sections: rows(1) });
    expect(growth.reads(true)).toBe("c-page-3");
  });

  it("lets the page ask again with the same cursor when a read does not answer", () => {
    const growth = new PageGrowth();
    growth.answered(home.cursor);
    expect(growth.reads(true)).toBe("c-page-2");
    growth.failed();
    expect(growth.reads(true)).toBe("c-page-2");
  });

  it("asks for nothing while the focus is nowhere near the last row", () => {
    const growth = new PageGrowth();
    growth.answered(home.cursor);
    expect(growth.reads(false)).toBeNull();
    expect(growth.busy).toBe(false);
  });
});

describe("how far ahead the focus reads", () => {
  it("asks at once for a page whose first answer carried no rows at all", () => {
    // Such a page can only grow by asking: no focus movement will ever reach a last row.
    expect(new PageFocus([], 4, 0, false).wantsMore()).toBe(true);
  });

  it("does not ask from the bar or the hero of a page that has rows", () => {
    expect(new PageFocus(rows(12), 4, 0, true).wantsMore()).toBe(false);
  });

  it("asks once the focus is within reach of the last row it holds", () => {
    const focus = new PageFocus(rows(12), 4, 0, false);
    const asked: boolean[] = [];
    for (let at = 0; at < 12; at += 1) {
      asked.push(focus.wantsMore());
      focus.press("down");
    }
    // The last REACH + 1 rows ask: the focused one and the REACH below it that are out of sight.
    expect(asked.lastIndexOf(false)).toBe(12 - REACH - 2);
    expect(asked.slice(12 - REACH - 1).every((one) => one)).toBe(true);
  });
});
