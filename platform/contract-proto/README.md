# contract-proto — the contract as a schema, and the generator that makes it the only copy

Note 204's P1 and the constraints half of its P2. The whole contract as proto3, the
buf configuration, and one program that writes every language's value checks from the
field options so a rule cannot be stated twice and disagree with itself.

One conclusion: **drawn for a binary wire, the value half of this contract is three
bounds, one closed enum and a single predicate.** Everything else the contract refuses
is presence, and presence is a field option. The spike's six hand written checks
become 335 generated call sites and 11 lines of hand written Rust.

## Where this sits (landed 2026-11-12)

This crate and `gen/` are workspace members, so `./check` formats, lints, tests and
regenerates them with everything else. It was written in a worktree where each crate
declared its own `[workspace]`, which is why `./check` could not see it; those lines
are gone and this is note 204's P1.

**It arrives beside the JSON contract, not instead of it.** `platform/contract` is
still the live contract: the server speaks it, the four client emitters generate from
it, and the 271 fixtures are JSON. Nothing in this package changes any of that, and
nothing outside it changed except the lines that make `./check` see it — the
workspace manifest, `check`, and `platform/tools/check_proto_tie.py`, which is where
the ledger tie now lives so it reads like every other check in that folder.

Three things were added rather than moved, each because landing it in `./check`
exposed a claim nothing was holding:

* `tests/deps.rs`, the twin of `platform/contract/tests/deps.rs`. The size case rests
  on prost being `no_std` and on nothing else getting in, and a std dependency
  arriving here would have failed no check in this repository.
* `the_three_bounded_numbers_are_uint64_in_the_schema_because_uint32_truncates`,
  which pins the declaration where the test beside it pins the behaviour.
* `check_proto_tie.py --selftest`, which breaks each of the tie's rules on purpose.
  This package exists because of a check that passed while being wrong, so the tie
  check having no test of its own was the same hole in a new place.

## Run it

From the repository root:

    CARGO_BUILD_JOBS=2 nice -n 15 cargo build --locked -p contract-proto        # types
    CARGO_BUILD_JOBS=2 nice -n 15 cargo build --locked -p contract-proto \
        --target wasm32v1-none                                    # no_std, the core's shape
    CARGO_BUILD_JOBS=2 nice -n 15 cargo test  --locked -p contract-proto        # 23 tests

    cargo run --locked -p contract-validate-gen -- write   # writes the validator
    cargo run --locked -p contract-validate-gen -- check   # writes nothing, fails on drift

    cd platform
    python3 -I tools/check_proto_tie.py             # the ledger tie, standard library only
    python3 -I tools/check_proto_tie.py --selftest  # each of its rules, broken on purpose

`size/` is deliberately NOT a member — it is a `cdylib` with its own
`#[panic_handler]` and its own `wasm` profile, built by hand for one target to answer
one question, and the root manifest excludes it. The figures below came from it:

    cd platform/contract-proto/size
    CARGO_BUILD_JOBS=2 nice -n 15 cargo build --offline --profile wasm \
        --target wasm32v1-none
    CARGO_BUILD_JOBS=2 nice -n 15 cargo build --offline --profile wasm \
        --target wasm32v1-none --features validate

## What is measured (2026-11-12, all by build, test or script)

| | number | how |
|---|---|---|
| Schema | 2,331 lines in 14 files | `cat $(find proto -name '*.proto') | wc -l` |
| Messages | 218, including nested union arms | the generator, and `check_proto_tie.py` |
| Enums | 40, one of them closed | the same |
| Fields read | 638 | the generator |
| Generated Rust types | 4,007 lines, 0 `HashMap`, 5 `BTreeMap` | `wc -l`, `rg -c` on `generated/contract.v1.rs` |
| Generated validator | 2,872 lines, 185 functions, 335 refusal sites | `wc -l`, `rg -c` on `generated/validate.rs` |
| Rules declared | 329 `required`, 3 bounds, 2 `closed`, 1 value rule | the generator's own count |
| Hand written rule code | 11 lines (`is_calendar_day` in `src/rules.rs`) | the file |
| Declared losses | 49 repeated fields refuse absence today and cannot on a proto3 wire | the generator |
| Descriptor | 209,121 bytes with source info | `ls -l generated/contract.descriptor.binpb`, re-measured 2026-11-12 after the move; this row read 205,180, and the file is byte-identical to the worktree's copy, so that figure was stale rather than tree-specific |
| Module, 16 types round tripped, prost only | **127,244** bytes (gzip -9 34,092) | `size/`, `wasm` profile, `wasm32v1-none`, stripped |
| The same plus every validator | **156,437** bytes (gzip -9 41,377) | the same, `--features validate` |
| **What the whole constraints half costs** | **29,193 bytes** (gzip -9 7,285) | the difference |
| Of which, the field path strings | 10,016 bytes over 332 unique paths | counted in the generated file |
| Tests | 25, all passing | `cargo test --locked -p contract-proto` |
| Of those, added on landing | 3 | the uint64 declaration pin, and `tests/deps.rs` |

Set beside the spike, which round tripped the same sixteen types out of a sixteen type
module: serde_json 228,543, prost 96,193, plus 386 bytes for its six hand written
checks. The same sixteen types out of the **whole** 218 message module cost 127,244,
so modelling the other 202 messages adds 31,051 bytes for the same round trips. The
validator's 29,193 bytes buy every refusal in the contract, generated, where the
spike's 386 bytes bought six functions a caller had to remember to call. If the module
is ever tight, 10,016 of those bytes are the field path strings and a numeric rule id
would recover them.

Which of these were re-measured after the move, 2026-11-12: the schema lines and
file count, the message, enum, field, validator, refusal-site and rule counts (by
running the generator and `check_proto_tie.py` in the main tree - every figure
reproduced), the generated line counts, `0 HashMap` / `5 BTreeMap`, the 332 unique
paths and their 10,016 bytes, the descriptor size, and the tests. The four wasm size
figures - 127,244, 156,437, their difference and the gzip values - were **not**
re-measured, because they need the `size/` harness built for `wasm32v1-none` and that
is measurement this stage does not need. They still hold: `generated/validate.rs`,
`generated/contract.v1.rs` and the descriptor are byte-identical to the worktree's
copies (`cmp`, all four files), and `size/` was copied unchanged, so the inputs to
those builds did not move.

Not measured here: the real core after the swap. That needs note 204's P3 behind the
feature flag and then `cargo build --locked --profile wasm -p bridge-wasm --target
wasm32v1-none` with `twiggy top`, against today's 695,638 bytes.

## The design: one declaration, and a generator that refuses what it cannot enforce

`proto/contract/constraints/v1/constraints.proto` declares one field option. Every
value rule in the contract is written there, on the field, once. `gen/` reads the
options out of the descriptor and writes `generated/validate.rs`.

The defect this closes is a rule written twice. `platform/contract/src/title.rs:98-103`
states the title id rule as `id == 0 || id > MAX_TITLE_ID`;
`platform/contract/src/title.rs:23-25` states it again as `TITLE_ID_PATTERN`, a ten
branch regular expression hand expanded to mean the same range, and every generated
client compiles that expression into itself. Here the bound is
`{min: 1, max: 2147483647}` on `TitleKey.id` and every language's check comes out of
it, so the second statement does not exist.

**Corrected 2026-11-12, verified.** This paragraph used to say the emitted expression
*accepts* `movie:2147483648` and `tv:9999999999` while the core refuses them. That
was true of `"^(movie|tv):[1-9][0-9]{0,9}$"`, which is what the emitter produced
before `5db018f`. That commit replaced it with the alternation above, added the 230
line `platform/tools/check_patterns.py`, and added the two fixtures. Checked today on
tasks/boundaries: the expression at
`platform/contract/generated/types/Contract.kt:2444` refuses both strings, and
`python3 tools/check_patterns.py` passes with "3 patterned types, 12 fixture strings
judged, proved both ways by fixtures: CalendarDay, TitleKey". So the live argument for
declaring the bound is not that a client is wrong today - it is that one rule is
carried in two forms, a comparison and a hand expanded alternation, and they agree
only because a 230 line check compares them on twelve strings. That check exists
because the two forms disagreed once. Declared on the field there is one form, and
nothing to compare.

**The property that makes "one declaration" true is that the generator fails the build
on an option it cannot emit.** Not a warning, not a skip: `gen/src/main.rs` exits 2 and
names the field. A generator that ignores what it does not understand recreates the
defect in a new costume — a rule somebody adds that the core silently does not enforce.
This is also why the option set is ours rather than protovalidate's: with a third party
rule language the generator has to implement a subset of someone else's vocabulary and
pass over the rest, and "the rest" is exactly where the next silent gap would be.
protovalidate is unusable in the core anyway (CEL at run time, std only; note 204
verified it), so the Rust side is generated from options whichever option set is used.

It already earned its keep twice while being written:

* It refused `required` on `CalendarDay.ymd` because the field had no presence. A
  `uint32` with implicit presence cannot tell 0 from absent, so the rule would have
  been a lie. The schema now writes `optional`.
* It refuses `required` on a repeated field, which is the honest answer to the 49
  fields where proto3 cannot carry presence. Those carry `presence_lost: true`, a
  declared loss the tie check counts, rather than a rule that does nothing.

## The sharpest thing measured: a uint32 id would have shipped a correctness defect

`refused/TitleKey.id_ten_digits.json` is `"tv:9999999999"`. Declared as `uint32 id`,
the schema does something worse than either the Kotlin or the Rust: protobuf truncates
a varint that does not fit the declared width, so those exact bytes decode cleanly as
`TitleKey { kind: Tv, id: 1410065407 }` — a **different and entirely valid title**. A
viewer would be shown the wrong film and no fault would be reported anywhere. Measured,
not reasoned: `tests/refusals.rs` builds the varint and decodes it.

So `TitleKey.id`, `CalendarDay.ymd` and `ContractDate.ymd` are `uint64`, which makes
the illegal value representable so a bound can refuse it. Two tests hold it there, by
name: `a_ten_digit_id_arrives_whole_and_is_refused_rather_than_truncated` builds the
varint and proves the behaviour, and
`the_three_bounded_numbers_are_uint64_in_the_schema_because_uint32_truncates` reads the
checked in descriptor and pins the declaration, so the schema edit that would bring the
defect back fails there rather than somewhere downstream. A varint costs the same bytes
for every value that fits, so a real key is not a byte larger. The cost is in the
clients: protobuf-es gives TypeScript a `bigint` and protoc-gen-kotlin a `Long` where
`uint32` gave `number` and `Int`. `[jstype = JS_STRING]` is the standard mitigation and
is deliberately left off until P5 says which it wants.

The same reasoning put a ceiling on `ContractDate.ymd`. `WireDate::parse`
(`platform/contract/src/version.rs:1132-1144`) takes eight digits and nothing else, so
`"123456789"` is refused today; without `{max: 99999999}` the reshape would have quietly
dropped that refusal.

## The 27 refused fixtures

The corpus on `tasks/boundaries` has 27 files, not the 25 note 204 records. Both counts
are right for their tree: the spike worktree is pinned at `cefb33c` (2026-11-10) and has
25, and `TitleKey.id_over_max` and `TitleKey.id_ten_digits` were added afterwards in
`5db018f`. They are the two that prove the Kotlin defect, so the classification has to
see them. `tests/refusals.rs` reads the folder and fails when a file is there that its
table does not name, which is the check that would have caught the difference.

| | count | |
|---|---|---|
| Refused by a generated rule | 21 | 6 `required`, 10 `closed`, 3 bounds, 2 calendar day |
| Refused by the decoder | 4 | a string in a varint field (×3), and a CoreOut read as a CoreIn |
| No longer refused, on purpose | 2 | `CalendarDay.no_dashes`, `TitleKey.upper_case` |
| **Accepted with nothing to catch it** | **0** | |

The two that change are both refusals about text, and there is no text. `"20261120"`
is refused today for having no dashes; packed it is 2026-11-20, a day the project
accepts. `"Movie:1"` is refused for its capital letter; there is no kind word on the
wire, only a number. Each is recorded beside its message in `proto/contract/v1/common.proto`
and asserted in the tests, so neither can be quietly reinstated or quietly forgotten.

`refused/Attempt.no_headers.json` is worth naming separately. The spike measured it as
byte identical to its valid twin — 40 bytes each — because a proto3 map has no presence
and no decoder in any language could tell them apart. The `Headers` wrapper message
restores presence for two bytes, and `an_empty_headers_map_is_not_a_missing_one` proves
the twin is still accepted.

## The ledger tie

`ledger-proto.csv`, two columns, `id,proto`, keyed by ledger id.
`platform/tools/check_proto_tie.py` runs both directions and parses the descriptor by
hand with no protobuf module, the way `platform/tools/fake_server.py` uses the standard
library only. `./check` runs it, and runs `--selftest` after it.

Direction one, **declared**: a ledger row that constrains a shape names the path, and the
check fails when the path is not in the descriptor. Proved by deleting one:
`C16: contract.v1.Attempt.provider_name is not in the schema`.

Direction two, **computed**: every message must be reachable from a named path, a fixture,
or a message that is. An unreachable message is a shape nobody specified. Proved by adding
one: `contract.v1.Orphan: no ledger row names it and no fixture covers it`.

Both proofs are now in the file rather than in this paragraph. `--selftest` mutates the
real schema and the real sidecar seven times — a renamed path, an unknown ledger id, a
changed column set, an orphan message, an enum with no zero value, an unclassified refused
fixture, and no mutation at all — and fails when a rule stays quiet. Written because the
reason this package exists is a check that passed while being wrong: the spike's
classification table covered 25 of 27 fixtures and nothing noticed.

Direction two is deliberately **not** declared in the schema. The obvious alternative — a
`ledger` option on each message naming its rows — is the same fact in two files, which is
the defect this package exists to close. The ledger is the specification and it points at
the implementation; the implementation does not point back.

Today, re-run in the main tree on 2026-11-12 after the move and reproducing the worktree
exactly: 218 messages, 40 enums, 0 orphans, 6 ledger rows tied over 11 paths, 381 fields
with a rule, 49 `presence_lost` declarations. The other 60-odd rows note 204 expects to
carry a path are P7's to write; the check reports the coverage rather than pretending.

### Why the tie is not a column in `ledger.csv`

**Decided 2026-11-12: the sidecar stays, and `./check` runs the tie check so the two
cannot drift.** The eighth column cannot survive, because `ledger.py generate` rewrites
the whole of `ledger.csv` from `tools/ledger_plan.json` and would delete every hand
written cell in it; and the two files answer different questions, so a column would tie
their lifetimes together for nothing. Drift is the only thing a column would have bought,
and a check in `./check` buys it without the coupling.

Note 204 plans "ledger.csv untouched, plus one new column `proto`". That cannot be done
as written, and the proof is in a file this package does not own:

* `platform/tools/ledger.py:191-192` **refuses** any ledger whose columns are not exactly
  the seven it knows. Adding an eighth makes `ledger.py check` fail — and `./check:50`
  runs it.
* `platform/tools/ledger.py:182-184` writes the ledger from those same seven columns, and
  `ledger.py generate` rewrites the whole file from `tools/ledger_plan.json`. Sixty-six
  hand written cells would be deleted the first time anyone regenerated it.
* Four more readers would have to learn the column: `server/scripts/check_test_ids.py`,
  `platform/kit/core/tests/sessions.rs`, `platform/kit/core/tests/coverage.rs` and
  `data/tests/test_ledger.py`.

A sidecar is also the better shape on its own merits. `ledger.csv` is the behaviour
specification and it is generated from a plan; this mapping is onto an implementation and
changes when the schema changes, not when the behaviour does.

## buf

`buf.yaml` is `STANDARD` with **no exceptions**, where the spike needed three. Its
exceptions (`ENUM_VALUE_PREFIX`, `ENUM_VALUE_UPPER_SNAKE_CASE`, `ENUM_ZERO_VALUE_SUFFIX`)
existed because its enum value names were the contract's JSON words — `row`, `unknown` —
so that a proto3 JSON decoder could match them. Drawn for a binary wire the names never
travel, so the values are `SECTION_KIND_ROW` and `SECTION_KIND_UNSPECIFIED` and the
exceptions go. The wire words live in the clients' labels, where they always belonged.

`breaking` is `FILE`, the strictest. What it buys against this schema, concretely: a
renamed field, a changed field number, a changed type, a deleted message, a reordered
enum value and a field moved in or out of a oneof are each refused at commit time. Those
are the six ways the JSON contract can drift silently today, and nothing catches them,
because `platform/tools/check_types.py` has never once run past Swift.

**Neither command has been run.** buf is not on this machine and installing it needs a
download the sandbox refuses. Every rule above was reasoned against the schema, not
observed. The first thing to do on a machine with the toolchain is `buf lint`, and the
second is `buf breaking --against` the previous commit's descriptor.

## The 376 JSON files under platform/contract

271 fixtures, 104 standalone JSON Schemas, and `openapi.json`.

### The 271 fixtures become protobuf text format

Proved in `tests/corpus_format.rs`, which round trips a message through text format and
asserts identical bytes back. Two facts decided it:

* **`prost` has no text format at all** — no reader, no writer. A corpus in text format
  cannot be loaded by the crate the core links, which is the right way round: the core
  reads bytes.
* **`prost-reflect` has both**, over a `DynamicMessage` driven by the checked in
  descriptor, so the corpus is readable by a std tool with no protoc and no network.

It is genuinely readable and genuinely diffable, because the writer puts the enum value
*name* in, not the number:

    name: SCREEN_NAME_MY_LIST
    sections: [{
      id: "s"
      kind: SECTION_KIND_ROW
      tiles: [{
        title {
          kind: TITLE_KIND_MOVIE
          id: 550
        }
        name: "Fight Club"
      }]
    }]
    ttl_s: 60

One trap, costly and silent: `{:#}` is the text format writer and `{:#?}` is Rust's
`Debug`, which prints the internal map keyed by field number. The test names it.

What is **not** done here: converting the existing 271 JSON files. That needs the shape
mapping — a title key's text to `{kind, id}`, a date's text to a packed number, a
flattened union to a oneof — which is the conversion layer (P2), and the corpus itself is
P4. This package's job was to say what the format is and prove it holds.

### `openapi.json` is regenerated, not retired, and it carries less

343,637 bytes (`ls -l`, tasks/boundaries at 1bc1ef8, 2026-11-12), 65 paths, 71
operations, 189 schemas, generated from the 831 line `api.rs`. **Correction:** this
line read 342,957 bytes, and that number reproduces in neither tree - the spike
worktree's own copy is 342,811. It is a generated file and it moves with `api.rs`, so
quote it with the commit or not at all. The path, operation and schema counts below
were taken from the worktree's copy and are not re-counted here; treat them as that
tree's, not this one's. Read by `platform/tools/check_patterns.py`, `check_private.py`,
`fake_server.py`, `platform/contract/gen/src/{openapi,schemas,lib}.rs` and
`gen/tests/generated.rs`.

Keep it, because **protobuf says nothing about any of what it is for**: 65 paths, 71
operations, 31 of them carrying a path or query parameter, 12 cacheable GETs, one route
answering `text/vtt`, and 5 auth classes of which 4 become security schemes. Counted
today, not quoted: the paths, operations, parameters, schemas and the `text/vtt` route
out of `openapi.json` itself, and the cacheable routes from the twelve `cached(read(`
call sites in `platform/contract/src/api.rs`. Those are `api.rs`'s facts. Retiring the
file deletes the
only machine readable description of them, and buf offers no replacement without
`google.api.http` annotations plus a transcoding gateway — which is a new runtime in the
server, and note 204 lists that as the owner's open question.

What changes is the body half. The core writes its own HTTP bodies (29 `serde_json` call
sites in `platform/core/src/lib.rs`, per note 204), so if the REST surface kept a JSON
encoding, `serde_json` would stay in the core and the entire size prize would be lost.
The bodies therefore go binary, and `openapi.json` stops carrying 189 inline JSON Schemas
and carries `application/x-protobuf` with the message's full name instead — a reference
into `generated/contract.descriptor.binpb`, which is one file rather than 189 copies of
shapes the schema already states. `gen/src/shape.rs` and `gen/src/schemas.rs` (453 lines)
are replaced by a descriptor pass, or deleted. That work is P6's; this is the answer it
needs, not the change.

### The 104 standalone schemas are retired with the JSON surface

Three things read `generated/schemas/`:

* `platform/tools/check_patterns.py`, whose whole job is proving that "every patterned
  type's expression is the same text in the standalone schema" for "three string types
  carry a pattern today". Those three are `TitleKey`, `CalendarDay` and `WireDate` — and
  all three stop being patterned strings in this schema. **The check and the problem it
  polices disappear together**, which is the cleanest thing in this package: 29 of the 104
  files carry a pattern transitively, and none of them will. `TITLE_ID_PATTERN` survives
  only for the id inside a route path and a deep link, which is text and stays text.
* `platform/contract/gen/tests/generated.rs`, which can read the descriptor instead.
* `data/src/databuild/manifest/schema.py`, which reads `contract.schema_dir` for
  `DatasetManifest` — and there is no `DatasetManifest.json` in that folder. Its own
  docstring says so: "Until the contract holds the type, this repository ships the same
  shape beside this file." So `data/` does not depend on any of the 104 files today.

None of those three files is this package's to change. `check_patterns.py` is named in
`./check:75`, so its removal is a `./check` edit, which is P7's.

## TitleKind stays closed

The owner's decision, and the schema cannot hold it: proto3 enums are open over numbers
by design, and that openness is exactly what the tolerated corpus needs from the other 39
enums. So the closure is a refusal declared on the **field**, with `closed: true`, at
every place a kind is carried — `TitleKey.kind` and `GateLine.media`. It refuses the zero
value and any number the schema does not declare.

`an_unknown_title_kind_is_refused_and_every_other_unknown_word_is_kept` is the test that
holds both halves at once: kind 7 is refused, section kind 99 is carried.

## Stage two, with paths

Stage one is this package, landed and checked. Stage two is three pieces of work in
a fixed order, and none of it is done here.

**1. The Kotlin emitter is not first; the Java one is.** The owner's decision of
2026-11-12 is Java with native Android Views, so `platform/contract/gen/src/kotlin.rs`
(280 lines) is emitting checks for a language the Android app is moving off. Writing
the generated-check emitter for Kotlin now would be work thrown away. Write
**Swift first** - `platform/contract/gen/src/swift.rs`, 363 lines, the largest
emitter, the one language whose compiler this machine actually has at
`xcrun -f swiftc`, and the surface the owner is continuing with - then Java, then
TypeScript, then whatever is left of Kotlin. The thing being written in each case is
the same pass the Rust validator already has: read `(contract.constraints.v1.field)`
off the descriptor and emit the check, so no client is born more forgiving than the
core, and no client needs a 230 line checker to prove it.
`platform/contract/gen/src/kotlin.rs:106-113` emits `Regex(...)` from a pattern
string, so each emitter's correctness is in its escaping and in whatever
`TITLE_ID_PATTERN` happens to say - which is the shape that went wrong once, in the
pattern rather than in the emitter, and the shape a declared bound removes.
Sequencing note, because it is the one place this ordering can bite: do not touch
`platform/tools/check_patterns.py` until the emitters are written, since it is the
only thing proving the four languages agree today.

**2. The core's swap off serde_json** touches `platform/core/src/lib.rs`, where
`serde_json` appears on **29 lines** (`rg -c serde_json`, 2026-11-12, verified), and
it needs `platform/contract/src/convert.rs`, which does not exist yet: the `TryFrom`
pair between these generated wire structs and the core's domain types. Three facts
decide its shape, and all three are already measured here rather than open:
`TitleKey.id` is `uint64` on the wire and `u32` in
`platform/contract/src/title.rs:98` (`TitleKey::new(kind, id: u32)`), so the
conversion is where the bound is enforced a second time, in the one place that is
allowed to state it twice because one side is the wire and the other is the domain;
`CalendarDay` and `ContractDate` arrive packed and parse out, not in;
and `Features.accounts_required`, `Strings.dir` and `Language.dir` have a default
that is not the zero value, so the reader applies the default and implicit presence
would turn the safe answer into the unsafe one silently. Behind note 204's feature
flag, and the gate is the size measurement against today's 695,638 bytes of
`bridge-wasm`, which is the one benchmark still worth running because it decides
whether the swap happens at all.

**3. `platform/contract/generated/openapi.json` is regenerated, not retired**, and
the body half is what changes. Protobuf says nothing about 65 paths, 71 operations,
the cacheable GETs or the `text/vtt` route, and buf offers no replacement without
`google.api.http` annotations plus a transcoding gateway, which is a new runtime in
the server and the owner's open question. So the file stays and stops carrying 189
inline JSON Schemas: it carries `application/x-protobuf` and the message's full
name, a reference into `generated/contract.descriptor.binpb`, which is one file
rather than 189 copies of shapes the schema already states.
`platform/contract/gen/src/shape.rs` (212 lines) and
`platform/contract/gen/src/schemas.rs` (241 lines) become a descriptor pass or are
deleted, `platform/contract/gen/src/openapi.rs` (232 lines) learns the content type,
and `platform/tools/check_patterns.py` disappears with the problem it polices,
because all three patterned string types stop being patterned strings - which is a
`./check:75` edit. None of that may start before the emitters, because
`check_patterns.py` is what currently proves the four languages agree.

## Decisions for the owner

1. **`More` keeps two repeated lists and does not become a oneof.** A oneof would make
   "one route, two shapes" unambiguous, and it would also refuse an answer carrying
   neither list, which the Rust accepts today. That is a visible change to what the
   server may send, so it needs the owner, not a tidy-up. (`screen.proto`)
2. **`ProfileLimits.blocked_titles` stays `repeated string`.** It is the one field that
   carries title key text without parsing it. `TitleKey::parse_list` deliberately skips
   junk and keeps going, so a stored list may hold an entry this build cannot parse;
   `repeated TitleKey` would turn it into a kind-unset key, which the closed rule then
   refuses, and a profile with one bad stored entry would stop being readable. Assumed,
   not verified: the server's storage was not inspected. (`account.proto`)
3. **`Features.accounts_required`, `Strings.dir` and `Language.dir` have a default that is
   not the zero value** — `true`, `Ltr`, `Ltr`. proto3 refuses field defaults, so all three
   carry explicit presence and the reader applies the default. Implicit presence would read
   `accounts_required` as `false`, turning the safe default into the unsafe one silently
   for every server that leaves it out. The reader is the conversion layer's (P2).
4. **`uint64` on the three bounded numbers, and whether the clients want
   `[jstype = JS_STRING]`** with it. (P5)
5. **Whether the REST bodies really go binary**, which is what decides the `openapi.json`
   answer above. (P6)

## What could not be run here

`buf`, `protoc`, `protoc-gen-swift`, `protoc-gen-kotlin`, `protoc-gen-es`, `node`, `npm`,
`tsc` and `kotlinc` are all absent, and the sandbox refuses the protoc release download.
`swiftc` is at `xcrun -f swiftc` but the generated Swift needs the SwiftProtobuf runtime,
which is also a download. So:

* `buf lint` and `buf breaking` — **not run**. Written and reasoned, nothing observed.
* The three client languages — **not generated and not compiled**. `buf.gen.yaml` is
  written and ready.
* The client side of the generator — **designed, not written**. `gen/` emits Rust today.
  The same pass writes Swift, Kotlin and TypeScript checks from the same options, and
  that is the piece that finally closes the Kotlin defect in the Kotlin. It cannot be
  tested here, and writing an untestable emitter is how `"^(movie|tv):[1-9][0-9]{0,9}$"`
  shipped and stood until `5db018f`.
* The real core's size after the swap — **not measured**, and it is P3's gate.

Rust is the only one of the four languages that can be generated and compiled on this
machine, which is the same gap that let the Kotlin escaping defect ship:
`platform/tools/check_types.py` has never run past Swift. The format change widens the
gap by adding buf and protoc to the list, which is why note 204's P0 installs `tsc` and
`kotlinc` before anything else.

## Corrections to note 204, recorded so they are not repeated

* **"ledger.csv untouched, plus one new column `proto`"** cannot be done:
  `platform/tools/ledger.py:191-192` refuses an eighth column and `:182-184` would delete
  it on the next generate. A sidecar file instead.
* **"25 refused fixtures"** is 27 on `tasks/boundaries`. The spike's tree really did have
  25; two were added in `5db018f` after it was cut, and they are the two that prove the
  defect the constraints work exists to close.
* **The spike's `checks::contract_date`** applied the calendar predicate and the
  `WIRE_MIN` floor to a wire date. `WireDate::parse` applies neither, and the floor lives
  in `VersionRange::fit`, a comparison the core makes. Both would have been new refusals.
  `a_contract_date_is_not_held_to_the_calendar_or_to_wire_min` exists so nobody adds them
  back by tidying.
* **"the descriptor path is not byte stable, which is the path any ledger-to-schema tie
  check would use"** — true about encoding, and it does not reach the tie check, which
  only reads names and never encodes anything.
* **The spike's six checks** are four rules, three of them declarative. Only the calendar
  day needs code.
* **`Attempt.no_headers` is catchable.** The spike's "no decoder in any language can ever
  separate them" was measured against its JSON-shaped schema. With the wrapper message it
  is an ordinary presence refusal.
* **The spike's enum wrapper messages and renamed union arms** (`PlayAction`,
  `CommandAction`, `ActionEvent`) were both JSON artifacts. Prefixed value names remove
  the wrappers; nesting arm messages inside their union removes the renames, except for
  two real traps worth knowing: a nested `Action.Command` would shadow the top level
  `Command` so its own payload field resolves to its parent, and Swift has an `Error`
  protocol, so `DriverEvent.Error` is named `ErrorEvent`.
* **Two of note 204's openapi figures do not reproduce.** It records "61 operations
  carry path or query parameters" and "13 cacheable GETs"; counted today from
  `openapi.json` and from the `cached(read(` call sites in `api.rs`, they are 31 and 12.
  The 61 is probably every operation with any parameter, headers included; the 13 is
  probably the twelve call sites plus the `const fn cached` definition. Neither changes
  any decision, and both are recorded because the figures are quoted elsewhere.
* **The descriptor is 209,121 bytes, not 205,180.** Re-measured on landing, and the
  file is byte-identical to the worktree's copy, so the old figure was stale rather
  than a difference between trees. Same shape of mistake as the `openapi.json` size
  above: a generated file quoted without the commit it was generated at.

* **"the generated Kotlin accepts `movie:2147483648`" is no longer true**, and this
  package was written and named as though it were. Verified 2026-11-12 on
  tasks/boundaries: `5db018f` replaced `"^(movie|tv):[1-9][0-9]{0,9}$"` with the ten
  branch alternation at `platform/contract/src/title.rs:23-25`, added the 230 line
  `platform/tools/check_patterns.py`, and added the two fixtures that prove it;
  `platform/contract/generated/types/Contract.kt:2444` refuses both keys and
  `check_patterns.py` passes. The defect was real and is closed. What the schema
  still removes is the second statement of the rule, and the 230 lines that keep the
  two statements agreeing. Every place this file, `tests/refusals.rs`, `buf.gen.yaml`
  and `check` said otherwise has been corrected rather than deleted, and the test
  that was called `the_two_keys_the_generated_kotlin_accepts_are_refused_here` is now
  `the_two_keys_a_patterned_string_gets_wrong_are_refused_by_a_bound_here`.

* **Custom options do not survive `prost_types`.** `protox::compile` returns a
  `prost_types::FileDescriptorSet`, prost drops unknown fields on decode, and every
  `(contract.constraints.v1.field)` in the schema is an unknown field to it. The
  generator read 638 fields and found 0 rules until both `build.rs` and `gen/` moved to
  `protox::Compiler` and `encode_file_descriptor_set`. `the_checked_in_descriptor_still_carries_the_options`
  is the test that notices if anyone simplifies that line back.
