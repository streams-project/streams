// What a profile may be shown: its age tier, and the certificate a title carries.
//
// A MIRROR, and the only folder in this project that has one. Every other client calls the shared
// Rust core; Roku cannot (note 219 and note 222: Roku publishes no native extension, no FFI and no
// WebAssembly runtime, verified as an absence on 2026-11-12, and confirmed by a second reader sent
// out blind). The owner's decision on 2026-11-12 was to write the rules again, in this language,
// and to accept two homes for them.
//
// The Rust this mirrors:
//
//   `server/crates/server-rules/src/tier.rs:7-120`   the tier, its words, and its certificate rank
//   `server/crates/server-rules/src/eligibility.rs:47-88`   the one eligibility function
//
// Two things keep the mirror honest, and neither is a comment:
//
//   1. **Nothing here is a loose number.** The five certificate ranks of `tier.rs:95-103` are not
//      transcribed: `TIERS` is the one ordered list, narrowest first, exactly as the Rust enum is
//      declared ("A tier, narrowest first", `tier.rs:7`), and a rank is a position in it. The same
//      list orders the rating words, so U..18 and 1..5 cannot disagree with each other here.
//   2. **`src/tests/tier.test.ts` holds this against the fixture corpus the Rust reads**,
//      `platform/contract/fixtures/2026-11-05/`, which `platform/contract/tests/fixtures.rs:1` says
//      "every language reads the same files", and against the word lists in the Rust source itself,
//      read from disk. A mirror checked only against its own idea of correctness is worse than no
//      mirror, because it looks right.
//
// **What this does NOT mirror, and why.** `eligible()` also reads a catalogue item's adult flag,
// its suggestive flag, its genres, its keywords and, when it has no certificate at all, a scored
// profile model loaded from a file on the server. None of those five reach a client: the wire gives
// a `Tile` no certificate and no genres (`platform/contract/generated/types/contract.ts:1592-1601`)
// and a `TitleDetail` an `AgeMark` whose `label` is the rating word and nothing else
// (`platform/contract/src/screen.rs:313-316`). So this file answers only the part a screen can
// answer from what it was handed, and says `null` for the rest rather than guessing. The server
// stays the one that decides what a page contains; this decides what this screen may draw of it.

/**
 * The tiers, narrowest first, in the order `server/crates/server-rules/src/tier.rs:7-15` declares
 * them. Every rank below is a position in this list, so the order has one home.
 */
export const TIERS = ["little_kids", "older_kids", "twelve", "fifteen", "adult"];

/** One of `TIERS`. */
export type Tier = string;

/**
 * The British rating each tier is saved as, in the same order as `TIERS`, from the match arms of
 * `Tier::from_rating` (`tier.rs:36-46`). The order ties the two lists together: a word's rank and
 * its tier are the same index, so a rating cannot be given a rank that disagrees with its tier.
 */
export const RATINGS = ["U", "PG", "12", "15", "18"];

/** Words that mean a rating already in `RATINGS`. (`tier.rs:41`, `tier.rs:28`) */
const SAME_AS = {
  "12A": "12",
  teens: "twelve",
};

/** The tier of a saved age limit: U, PG, 12, 12A, 15 or 18. Null when the word is not one. */
export function tierOfRating(word: string | null | undefined): Tier | null {
  if (word === null || word === undefined) return null;
  const plain = word.trim().toUpperCase();
  const named = SAME_AS[plain as keyof typeof SAME_AS] ?? plain;
  const at = RATINGS.indexOf(named);
  return at < 0 ? null : (TIERS[at] ?? null);
}

/** The tier of the coarse level kept on older rows. Null when the word is not one. (`tier.rs:23-33`) */
export function tierOfMaturity(word: string | null | undefined): Tier | null {
  if (word === null || word === undefined) return null;
  const named = SAME_AS[word as keyof typeof SAME_AS] ?? word;
  return TIERS.indexOf(named) < 0 ? null : named;
}

/**
 * The tier in force for a profile: the saved limit wins over the older level, and a Kids profile
 * never rises above PG. Null when neither word can be read, which `tier.rs:17-19` is emphatic
 * about: such a profile "gets nothing, and the call answers 503, never Adult".
 * (`tier.rs:66-74`)
 */
export function tierOfProfile(
  ageLimit: string | null | undefined,
  maturity: string | null | undefined,
  kids: boolean,
): Tier | null {
  const tier = tierOfRating(ageLimit) ?? tierOfMaturity(maturity);
  if (tier === null) return null;
  const ceiling = "older_kids";
  return kids && rankOf(tier) > rankOf(ceiling) ? ceiling : tier;
}

/** The highest certificate rank this tier may see. (`tier.rs:94-103`) */
export function maxCert(tier: Tier): number {
  return rankOf(tier);
}

/** The two tiers of children's profiles. (`tier.rs:106-108`) */
export function isKids(tier: Tier): boolean {
  return rankOf(tier) > 0 && rankOf(tier) <= rankOf("older_kids");
}

// `Tier::of_account` (`tier.rs:78-90`) is deliberately NOT mirrored. It picks the widest tier among
// an account's profiles, and it needs each profile's own age limit. The wire gives a client a
// `Profile` with `id`, `name`, `avatar`, `kids`, `main` and `has_pin` and no limit at all
// (`platform/contract/generated/types/contract.ts`, `Profile`), and `ProfileLimits` arrives for one
// profile at a time. So a screen in this folder cannot compute it, and a mirror that pretended to
// would be a rule with no caller and no way to be wrong.

/**
 * May a profile on this tier be shown a title carrying this certificate?
 *
 * The certificate reaches a client as `TitleDetail.age.label`, the rating word
 * (`platform/contract/src/screen.rs:313-316`), so the same `RATINGS` order gives its rank and
 * `eligible`'s one comparison, `item.cert > tier.max_cert()` (`eligibility.rs:67-69`), is the whole
 * of the rule on this side.
 *
 * - `true`  the certificate is within this tier.
 * - `false` it is above it.
 * - `null`  **not decidable here.** The title carries no certificate, or one this list does not
 *   name. The Rust falls back to a scored profile model that lives on the server
 *   (`eligibility.rs:54-66`), so a screen must not invent an answer: it draws what the server sent.
 */
export function certificateFits(tier: Tier, label: string | null | undefined): boolean | null {
  if (rankOf(tier) === 0) return null;
  if (label === null || label === undefined) return null;
  const named = SAME_AS[label.trim().toUpperCase() as keyof typeof SAME_AS] ?? label.trim().toUpperCase();
  const cert = RATINGS.indexOf(named) + 1;
  if (cert === 0) return null;
  return cert <= maxCert(tier);
}

/** A tier's position in `TIERS`, counting from 1. 0 when the word is not a tier. */
function rankOf(tier: Tier): number {
  return TIERS.indexOf(tier) + 1;
}
