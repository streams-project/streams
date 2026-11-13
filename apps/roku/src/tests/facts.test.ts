// The facts line, held against the fixtures the Rust reads and the words the server sends.
//
// **This is the point of the file.** `apps/roku/` is a second implementation of rules the Rust core
// holds, and a second implementation tested against its own idea of correctness is worse than none,
// because it looks right. So nothing below invents a title: every case is a file from
// `platform/contract/fixtures/2026-11-05/valid/`, the corpus `platform/contract/tests/fixtures.rs`
// reads with `fs::read_dir` and round-trips through the Rust contract types, and which that file's
// first line calls the files "every language reads". The words are read from `strings/`, the same
// files the server hands out at `/v1/strings/{lang}` (`platform/contract/src/api.rs:183`).
//
// The expected output is derived from `Facts.kt:37-48` and the words in the language file, never
// from running this code and writing down what it said.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { TitleDetail } from "../contract";
import { facts, lengthWord } from "../facts";

/** The repository root, four folders up from `apps/roku/src/tests`. */
const ROOT = join(__dirname, "..", "..", "..", "..");

/** The fixture corpus. `WIRE_VERSION` names the folder, as `fixtures.rs:45-49` does it. */
const CORPUS = join(ROOT, "platform", "contract", "fixtures", "2026-11-05");

function fixture(kind: string, name: string): TitleDetail {
  return JSON.parse(readFileSync(join(CORPUS, kind, name), "utf8")) as TitleDetail;
}

function strings(language: string): Record<string, string> {
  return JSON.parse(readFileSync(join(ROOT, "strings", `${language}.json`), "utf8")) as Record<
    string,
    string
  >;
}

const en = strings("en");

describe("the corpus holds what these cases read", () => {
  // A fixture that quietly lost a field would make every case below pass by drawing nothing. This
  // is the guard against that: it asserts the shape the rule depends on, not the rule.
  it("TitleDetail.film carries a genre, a year and a runtime and no seasons", () => {
    const film = fixture("valid", "TitleDetail.film.json");
    expect(film.genres).toEqual(["Drama"]);
    expect(film.year).toBe(2024);
    expect(film.runtime_min).toBe(101);
    expect(film.seasons).toBeUndefined();
  });

  it("TitleDetail.series carries seasons and neither genre nor year", () => {
    const series = fixture("valid", "TitleDetail.series.json");
    expect((series.seasons ?? []).length).toBe(1);
    expect(series.genres).toBeUndefined();
    expect(series.year).toBeUndefined();
  });

  it("no fixture in the corpus carries an age mark", () => {
    // Recorded rather than assumed: the age branch of `facts` therefore has no fixture to stand
    // on, and is held below by the words instead. A fixture for it is an open item in the note.
    const named = ["TitleDetail.film.json", "TitleDetail.series.json", "TitleDetail.coming.json"];
    const marks = named.map((name) => fixture("valid", name).age);
    expect(marks).toEqual([undefined, undefined, undefined]);
  });
});

describe("facts, from the fixture corpus", () => {
  it("puts a film's genre, year and length in that order", () => {
    // `title.length.hours.one` is "{n} hour", `title.length.minutes.many` is "{n} minutes" and
    // `title.length.both` is "{hours} {minutes}", all read from strings/en.json. 101 minutes is
    // one hour and 41 minutes. The filled values are isolated with U+2068 and U+2069 by `fill`.
    expect(facts(fixture("valid", "TitleDetail.film.json"), en, "en")).toEqual([
      "Drama",
      "2024",
      "⁨1⁩ hour ⁨41⁩ minutes",
    ]);
  });

  it("counts a series' seasons instead of its length, and leaves out what it has not got", () => {
    expect(facts(fixture("valid", "TitleDetail.series.json"), en, "en")).toEqual([
      "⁨1⁩ Season",
    ]);
  });

  it("treats a title that has not arrived yet like any other", () => {
    // `TitleDetail.coming.json` differs by `available_from` and `reminder_set`, neither of which is
    // a fact on this line: 105 minutes is one hour and 45.
    expect(facts(fixture("valid", "TitleDetail.coming.json"), en, "en")).toEqual([
      "Drama",
      "2026",
      "⁨1⁩ hour ⁨45⁩ minutes",
    ]);
  });

  it("writes the year in the language's own digits", () => {
    // ar holds `language.digits` = "arab", so 2024 is written with U+0660..U+0669.
    const line = facts(fixture("valid", "TitleDetail.film.json"), strings("ar"), "ar");
    expect(line[1]).toBe("٢٠٢٤");
  });

  it("takes only the first genre, never the list", () => {
    const film = fixture("valid", "TitleDetail.film.json");
    const many: TitleDetail = { ...film, genres: ["Drama", "Crime", "Thriller"] };
    expect(facts(many, en, "en")[0]).toBe("Drama");
    expect(facts(many, en, "en").length).toBe(3);
  });

  it("prefers a season count over a running time when a title carries both", () => {
    const film = fixture("valid", "TitleDetail.film.json");
    const both: TitleDetail = { ...film, seasons: [{ number: 1, episode_count: 2 }] };
    expect(facts(both, en, "en")[2]).toBe("⁨1⁩ Season");
  });

  it("draws nothing at all for a title that carries only a name", () => {
    const bare: TitleDetail = { name: "A", title: "movie:1" };
    expect(facts(bare, en, "en")).toEqual([]);
  });

  it("translates an age mark through its own key, and falls back to the label", () => {
    const film = fixture("valid", "TitleDetail.film.json");
    const marked: TitleDetail = { ...film, age: { label: "15", tone: "red" } };
    const spoken: Record<string, string> = { ...en, "age.15": "Fifteen" };
    expect(facts(marked, spoken, "en")[3]).toBe("Fifteen");
    expect(facts(marked, en, "en")[3]).toBe("15");
  });
});

describe("how long a title runs", () => {
  it("drops the hours when there are none", () => {
    expect(lengthWord(en, 41, "en")).toBe("⁨41⁩ minutes");
  });

  it("drops the minutes when a title runs to the hour", () => {
    expect(lengthWord(en, 120, "en")).toBe("⁨2⁩ hours");
  });

  it("reaches the word through the fallback chain when the language has no `other`", () => {
    // en carries `title.length.hours.one` and `.many` and no `.other`, so two hours is found only
    // by `counted`'s second and third steps. Read from strings/en.json on 2026-11-13.
    expect(en["title.length.hours.other"]).toBeUndefined();
    expect(en["title.length.hours.many"]).toBe("{n} hours");
  });

  it("joins the two parts with the pattern the language gives, not with a space", () => {
    // ar's `title.length.both` is "{hours} و{minutes}" with no space before the minutes.
    const ar = strings("ar");
    expect(ar["title.length.both"]).toBe("{hours} و{minutes}");
    expect(lengthWord(ar, 101, "ar").indexOf("و")).toBeGreaterThan(0);
  });
});
