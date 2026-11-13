// What a profile may be shown, held against the fixture corpus AND against the Rust it mirrors.
//
// Two kinds of case here, and the second kind is the one that earns this file:
//
//   1. **Fixtures.** `platform/contract/fixtures/2026-11-05/valid/ProfileLimits.basic.json` and the
//      two `Profile.*.json` files are read from disk. They are the same files
//      `platform/contract/tests/fixtures.rs` reads with `fs::read_dir` and round-trips through the
//      Rust contract types; its first line says the fixtures are "plain JSON and every language
//      reads the same files".
//
//   2. **The Rust source.** `server/crates/server-rules/src/tier.rs` is read as text and its match
//      arms are compared with the tables in `src/tier.ts`. This is the drift guard. Two
//      implementations of one rule drift silently, and a mirror that is only ever checked against
//      its own expectations will keep passing while the rule it mirrors moves underneath it. The
//      day somebody adds a tier, renames a rating or changes a certificate rank in the Rust, this
//      file goes red and names that file.
//
// What the second kind cannot do: it reads the words and the order, not the behaviour. A change to
// what `eligible` does with those words is caught by the Rust's own tests, not here.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import {
  RATINGS,
  TIERS,
  certificateFits,
  isKids,
  maxCert,
  tierOfMaturity,
  tierOfProfile,
  tierOfRating,
} from "../tier";

/** The repository root, four folders up from `apps/roku/src/tests`. */
const ROOT = join(__dirname, "..", "..", "..", "..");
const CORPUS = join(ROOT, "platform", "contract", "fixtures", "2026-11-05");
const RUST = join(ROOT, "server", "crates", "server-rules", "src", "tier.rs");

function fixture(name: string): Record<string, unknown> {
  return JSON.parse(readFileSync(join(CORPUS, "valid", name), "utf8")) as Record<string, unknown>;
}

const limits = fixture("ProfileLimits.basic.json");
const kidsProfile = fixture("Profile.kids.json");
const mainProfile = fixture("Profile.main.json");

describe("the corpus holds what these cases read", () => {
  it("ProfileLimits carries a saved age rating, and the two profiles differ on kids", () => {
    expect(limits["age_rating"]).toBe("15");
    expect(kidsProfile["kids"]).toBe(true);
    expect(mainProfile["kids"]).toBe(false);
  });
});

describe("a profile's tier, from the fixture corpus", () => {
  it("reads the saved age rating of a grown up profile", () => {
    const tier = tierOfProfile(limits["age_rating"] as string, null, mainProfile["kids"] as boolean);
    expect(tier).toBe("fifteen");
    expect(isKids(tier as string)).toBe(false);
  });

  it("never lets a Kids profile rise above PG, whatever is saved on it", () => {
    const tier = tierOfProfile(limits["age_rating"] as string, null, kidsProfile["kids"] as boolean);
    expect(tier).toBe("older_kids");
    expect(isKids(tier as string)).toBe(true);
  });

  it("gives a profile whose limit cannot be read nothing at all, never the widest tier", () => {
    // `tier.rs:17-19`: "A profile whose limit cannot be read gets nothing, and the call answers
    // 503, never Adult." Falling open here would show an adult catalogue to a child.
    expect(tierOfProfile("", "", false)).toBeNull();
    expect(tierOfProfile("R18", "grown_up", false)).toBeNull();
    expect(tierOfProfile(null, null, true)).toBeNull();
  });

  it("falls back to the older coarse level when no rating is saved", () => {
    expect(tierOfProfile(null, "teens", false)).toBe("twelve");
    expect(tierOfProfile("", "little_kids", false)).toBe("little_kids");
  });

  it("lets the saved rating win over the older level", () => {
    expect(tierOfProfile("U", "adult", false)).toBe("little_kids");
  });
});

describe("the certificate a title carries", () => {
  it("lets a tier see its own certificate and everything below it", () => {
    const tier = tierOfProfile(limits["age_rating"] as string, null, false) as string;
    for (const label of RATINGS.slice(0, maxCert(tier))) {
      expect(certificateFits(tier, label)).toBe(true);
    }
  });

  it("keeps a tier away from everything above it", () => {
    const tier = tierOfProfile(limits["age_rating"] as string, null, false) as string;
    for (const label of RATINGS.slice(maxCert(tier))) {
      expect(certificateFits(tier, label)).toBe(false);
    }
  });

  it("stops at PG for the Kids profile in the corpus, so a 12 is refused", () => {
    const tier = tierOfProfile(limits["age_rating"] as string, null, true) as string;
    expect(certificateFits(tier, "PG")).toBe(true);
    expect(certificateFits(tier, "12")).toBe(false);
  });

  it("answers nothing, not yes, when the title carries no certificate this list names", () => {
    // `eligibility.rs:54-66` sends the no-certificate case to a scored profile model that lives on
    // the server. A screen must not invent the answer.
    expect(certificateFits("fifteen", undefined)).toBeNull();
    expect(certificateFits("fifteen", "NC-17")).toBeNull();
    expect(certificateFits("not_a_tier", "15")).toBeNull();
  });

  it("reads 12A as 12, the way a saved limit is read", () => {
    expect(certificateFits("twelve", "12A")).toBe(true);
    expect(certificateFits("older_kids", "12a")).toBe(false);
  });
});

// ---------------------------------------------------------------------------------------------
// The drift guard: the same rule, read out of the Rust that owns it.
// ---------------------------------------------------------------------------------------------

const rust = readFileSync(RUST, "utf8");

/** `LittleKids` -> `little_kids`, which is the word `Tier::as_str` gives. (`tier.rs:111-119`) */
function snake(variant: string): string {
  return variant.replace(/([a-z])([A-Z])/g, "$1_$2").toLowerCase();
}

/** The body of a named function or enum in the Rust, from its opening brace to the matching one. */
function body(after: string): string {
  const at = rust.indexOf(after);
  if (at < 0) throw new Error(`${after} is no longer in ${RUST}`);
  const open = rust.indexOf("{", at);
  let depth = 0;
  for (let i = open; i < rust.length; i += 1) {
    if (rust[i] === "{") depth += 1;
    if (rust[i] === "}") {
      depth -= 1;
      if (depth === 0) return rust.slice(open + 1, i);
    }
  }
  throw new Error(`${after} has no closing brace`);
}

/** Every `"word" | "word" => Some(Self::Variant)` arm of a match, as pairs. */
function wordArms(source: string): [string, string][] {
  const pairs: [string, string][] = [];
  for (const line of source.split("\n")) {
    const arm = /^\s*((?:"[^"]*"\s*\|?\s*)+)=>\s*Some\(Self::(\w+)\)/.exec(line);
    if (arm === null) continue;
    for (const word of (arm[1] ?? "").match(/"([^"]*)"/g) ?? []) {
      pairs.push([word.slice(1, -1), snake(arm[2] ?? "")]);
    }
  }
  return pairs;
}

describe("the Rust this mirrors, read from disk", () => {
  it("declares the same tiers in the same order", () => {
    const variants = (body("pub enum Tier").match(/^\s*(\w+),/gm) ?? []).map((line) =>
      snake(line.trim().replace(",", "")),
    );
    expect(variants).toEqual(TIERS);
  });

  it("maps every saved rating to the tier this file gives it", () => {
    const arms = wordArms(body("pub fn from_rating"));
    expect(arms.length).toBeGreaterThanOrEqual(RATINGS.length);
    for (const [word, tier] of arms) {
      expect(tierOfRating(word)).toBe(tier);
    }
    // ... and nothing here accepts a word the Rust refuses.
    const known = arms.map(([word]) => word.toUpperCase());
    for (const label of RATINGS) {
      expect(known).toContain(label);
    }
  });

  it("maps every coarse level to the tier this file gives it", () => {
    const arms = wordArms(body("pub fn from_maturity"));
    expect(arms.length).toBeGreaterThanOrEqual(TIERS.length);
    for (const [word, tier] of arms) {
      expect(tierOfMaturity(word)).toBe(tier);
    }
  });

  it("gives every tier the certificate rank this file derives from its position", () => {
    const arms = body("pub const fn max_cert").match(/Self::(\w+)\s*=>\s*(\d+)/g) ?? [];
    expect(arms.length).toBe(TIERS.length);
    for (const arm of arms) {
      const parts = /Self::(\w+)\s*=>\s*(\d+)/.exec(arm);
      expect(maxCert(snake(parts?.[1] ?? ""))).toBe(Number(parts?.[2]));
    }
  });

  it("clamps a Kids profile to the same ceiling", () => {
    const parts = body("fn of_parts");
    // `tier.rs:70-73`: `if kind == ProfileKind::Kids && tier > Self::OlderKids { Self::OlderKids }`
    const ceiling = /tier > Self::(\w+)/.exec(parts);
    expect(ceiling).not.toBeNull();
    expect(tierOfProfile("18", null, true)).toBe(snake(ceiling?.[1] ?? ""));
  });
});
