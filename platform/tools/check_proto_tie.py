#!/usr/bin/env python3
"""Tie the ledger to the schema, so neither can drift from the other.

    python3 -I tools/check_proto_tie.py
    python3 -I tools/check_proto_tie.py --selftest
    python3 -I tools/check_proto_tie.py --fixtures /path/to/contract/fixtures

`./check` runs the first two, so the sidecar and the schema cannot drift from each
other without a red run, and so this file cannot stop reporting without a red run
either: `--selftest` breaks each rule on purpose, once, against the real schema, and
fails if the rule stays quiet.

Exit 0 means the tie holds; 1 means it does not, and says why. Standard library
only, like platform/tools/fake_server.py: the descriptor is parsed here, by hand,
in about sixty lines, because there is no protobuf module on this machine and
adding one would make this check need a toolchain the repository does not have.

WHY THE TIE IS A SEPARATE FILE AND NOT A COLUMN IN ledger.csv
-------------------------------------------------------------
Decided 2026-11-12, and this file is the implementation of it: the sidecar stays and
`./check` runs this script, rather than teaching ledger.py an eighth column. Two
reasons. The eighth column cannot survive: `ledger.py generate` rewrites the whole
of ledger.csv from tools/ledger_plan.json, so every hand written cell in a `proto`
column is deleted the first time anyone regenerates the ledger, and four more
readers would have to learn the column first. And the two files answer different
questions - ledger.csv is the behaviour specification and is generated from a plan,
while this mapping is onto an implementation and changes when the schema changes,
not when the behaviour does. A column would tie their lifetimes together for no
gain. Drift is what a column would have bought, and running this script in `./check`
buys the same thing without the coupling.

Note 204's plan was "ledger.csv untouched, plus one new column `proto`". That
cannot be done, and the proof is in two lines of a file this package does not own:

  * platform/tools/ledger.py:191-192 refuses any ledger whose columns are not
    exactly the seven it knows. Adding an eighth makes `ledger.py check` fail.
  * platform/tools/ledger.py:182-184 writes a ledger from those same seven
    columns, and `ledger.py generate` rewrites the whole file from
    tools/ledger_plan.json. Sixty-six hand written cells would be deleted the
    first time anyone regenerated it.

Four more readers would have to learn the column as well: server/scripts/
check_test_ids.py, platform/kit/core/tests/sessions.rs, .../coverage.rs and
data/tests/test_ledger.py.

So the tie lives in `ledger-proto.csv`, keyed by ledger id, beside the ledger. That
is also the better shape on its own merits: ledger.csv is the BEHAVIOUR
specification and it is generated from a plan, while this mapping is onto an
implementation and changes when the schema changes, not when the behaviour does.
A row may appear more than once, because one behaviour can constrain several
messages.

THE TIE, IN BOTH DIRECTIONS
---------------------------
Direction one, declared: a ledger row that constrains a shape names the path, and
this script fails when the path is not in the descriptor. A renamed field or a
deleted message is caught at commit time by the row that pointed at it.

Direction two, computed: every message in the schema must be reachable from a
named path, from a fixture, or from a message that is. An unreachable message is a
shape nobody specified, and it fails. This direction is deliberately NOT declared
in the schema - the obvious alternative, a `ledger` option on each message, would
be the same fact written in two places, which is the defect this whole package
exists to close.
"""

import argparse
import csv
import re
import sys
from pathlib import Path

PLATFORM = Path(__file__).resolve().parent.parent
SCHEMA = PLATFORM / "contract-proto"
LEDGER = PLATFORM / "contract" / "ledger.csv"
FIXTURES = PLATFORM / "contract" / "fixtures"
TIE = SCHEMA / "ledger-proto.csv"
DESCRIPTOR = SCHEMA / "generated" / "contract.descriptor.binpb"
RULES = SCHEMA / "generated" / "rules.json"
CLASSIFICATION = SCHEMA / "tests" / "refusals.rs"

# platform/contract/src/version.rs WIRE_VERSION.
WIRE_VERSION = "2026-11-05"

PACKAGE = "contract.v1"

# Field numbers, read from
# ~/.cargo/registry/.../protox-0.7.2/protobuf/src/google/protobuf/descriptor.proto
# rather than from memory.
FILE_SET_FILE = 1
FILE_NAME, FILE_PACKAGE, FILE_MESSAGE, FILE_ENUM = 1, 2, 4, 5
MSG_NAME, MSG_FIELD, MSG_NESTED, MSG_ENUM = 1, 2, 3, 4
FIELD_NAME, FIELD_TYPE, FIELD_TYPE_NAME, FIELD_PROTO3_OPTIONAL = 1, 5, 6, 17
ENUM_NAME, ENUM_VALUE = 1, 2
VALUE_NAME, VALUE_NUMBER = 1, 2
TYPE_MESSAGE, TYPE_ENUM = 11, 14


def read_varint(data, index):
    """One varint. Returns the value and the index after it."""
    value = 0
    shift = 0
    while True:
        byte = data[index]
        index += 1
        value |= (byte & 0x7F) << shift
        if byte < 0x80:
            return value, index
        shift += 7


def fields(data):
    """Every (number, payload) of one message. Payload is bytes or an int."""
    index = 0
    out = []
    while index < len(data):
        key, index = read_varint(data, index)
        number, wire = key >> 3, key & 7
        if wire == 0:
            value, index = read_varint(data, index)
            out.append((number, value))
        elif wire == 2:
            length, index = read_varint(data, index)
            out.append((number, data[index : index + length]))
            index += length
        elif wire == 5:
            out.append((number, data[index : index + 4]))
            index += 4
        elif wire == 1:
            out.append((number, data[index : index + 8]))
            index += 8
        else:
            raise ValueError(f"wire type {wire} is not in a descriptor")
    return out


def first(parts, number, default=None):
    for got, value in parts:
        if got == number:
            return value
    return default


def every(parts, number):
    return [value for got, value in parts if got == number]


def text(value):
    return value.decode("utf-8")


class Schema:
    """What this check needs out of a descriptor set."""

    def __init__(self):
        self.messages = {}  # full name -> {field name -> type full name or None}
        self.enums = {}  # full name -> {value name: number}

    def load(self, path):
        data = path.read_bytes()
        for file_bytes in every(fields(data), FILE_SET_FILE):
            file_parts = fields(file_bytes)
            package = first(file_parts, FILE_PACKAGE)
            package = text(package) if package else ""
            if package != PACKAGE:
                continue
            for message in every(file_parts, FILE_MESSAGE):
                self.message(package, fields(message))
            for enumeration in every(file_parts, FILE_ENUM):
                self.enum(package, fields(enumeration))

    def message(self, scope, parts):
        name = f"{scope}.{text(first(parts, MSG_NAME))}"
        if name.endswith("Entry") and name not in self.messages:
            # A map entry message, which protobuf invents. It is not a shape of
            # the contract and no ledger row can name it.
            pass
        self.messages[name] = {
            text(first(field_parts, FIELD_NAME)): self.field_type(field_parts)
            for field_parts in (fields(field) for field in every(parts, MSG_FIELD))
        }
        for nested in every(parts, MSG_NESTED):
            self.message(name, fields(nested))
        for enumeration in every(parts, MSG_ENUM):
            self.enum(name, fields(enumeration))

    @staticmethod
    def field_type(field_parts):
        kind = first(field_parts, FIELD_TYPE, 0)
        if kind in (TYPE_MESSAGE, TYPE_ENUM):
            return text(first(field_parts, FIELD_TYPE_NAME)).lstrip(".")
        return None

    def enum(self, scope, parts):
        name = f"{scope}.{text(first(parts, ENUM_NAME))}"
        self.enums[name] = {
            text(first(value_parts, VALUE_NAME)): first(value_parts, VALUE_NUMBER, 0)
            for value_parts in (fields(value) for value in every(parts, ENUM_VALUE))
        }

    def real_messages(self):
        """Every message of the contract, leaving out protobuf's map entries."""
        return {
            name: shape
            for name, shape in self.messages.items()
            if not re.search(r"\.[A-Za-z0-9]+Entry$", name)
            or not set(shape) <= {"key", "value"}
        }

    def knows(self, path):
        if path in self.messages or path in self.enums:
            return True
        head, _, tail = path.rpartition(".")
        if head in self.messages and tail in self.messages[head]:
            return True
        if head in self.enums and tail in self.enums[head]:
            return True
        return False


def fail(problems):
    for problem in problems:
        print(f"  {problem}")
    print(f"{len(problems)} problems")
    return 1


def check_rows(rows, columns, schema, ledger_ids, name=None):
    """Direction one. Returns the rows keyed by ledger id, and what is wrong.

    Takes rows rather than a path so `--selftest` can hand it a mutated copy.
    """
    problems = []
    tied = {}
    if columns != ["id", "proto"]:
        problems.append(f"{name} columns are {columns}, expected ['id', 'proto']")
    for row in rows:
        row_id, named = row["id"].strip(), row["proto"].strip()
        if not named:
            problems.append(f"{row_id}: an empty path. Leave the row out instead.")
            continue
        if ledger_ids and row_id not in ledger_ids:
            problems.append(f"{row_id}: no such ledger row")
        if not schema.knows(named):
            problems.append(f"{row_id}: {named} is not in the schema")
        tied.setdefault(row_id, []).append(named)
    return tied, problems


def read_tie(path):
    """The sidecar, as rows and column names."""
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        return list(reader), reader.fieldnames


def reach(messages, tied, fixture_names):
    """Direction two. Every message reachable from a named path or a fixture."""
    seeds = set()
    for paths in tied.values():
        for named in paths:
            seeds.add(named if named in messages else named.rpartition(".")[0])
    seeds |= {name for name in fixture_names if name in messages}

    reached = set()
    queue = [name for name in seeds if name in messages]
    while queue:
        name = queue.pop()
        if name in reached:
            continue
        reached.add(name)
        for target in messages[name].values():
            if target in messages and target not in reached:
                queue.append(target)
        # A nested message of a reached message is part of its shape.
        for other in messages:
            if other.startswith(f"{name}.") and other not in reached:
                queue.append(other)
    return reached


def audit(schema, messages, tied, fixture_names, refused_stems, table):
    """Both directions plus the two smaller rules, with nothing read from disk.

    Separated from `main` so `--selftest` can run it against a mutated copy of the
    real schema. A check nobody has watched fail is a check nobody knows works:
    the spike's classification table covered 25 of 27 fixtures and every test it
    had still passed.
    """
    problems = []
    reached = reach(messages, tied, fixture_names)
    orphans = sorted(set(messages) - reached)
    for orphan in orphans:
        problems.append(
            f"{orphan}: no ledger row names it and no fixture covers it. "
            "A shape nobody specified."
        )

    # Every enum declares its zero value, which is what stands in for the word a
    # build does not know.
    for name, values in sorted(schema.enums.items()):
        if 0 not in values.values():
            problems.append(f"{name}: no value is number 0, so there is no unknown")

    # Every refused fixture is classified. This is the check that would have caught
    # the spike's table covering 25 of 27.
    if table is not None:
        for stem in sorted(refused_stems):
            if f'"{stem}"' not in table:
                problems.append(f"refused/{stem}.json is not named in tests/refusals.rs")

    return problems, reached, orphans


def fixtures_in(fixtures_root):
    """The message name each fixture is about, and the refused fixtures' stems."""
    names = set()
    refused = set()
    for kind in ("valid", "tolerated", "refused"):
        folder = fixtures_root / kind
        if not folder.is_dir():
            continue
        for fixture in folder.glob("*.json"):
            names.add(f"{PACKAGE}.{fixture.name.split('.')[0]}")
            if kind == "refused":
                refused.add(fixture.stem)
    return names, refused


def load(ledger_path, fixtures_path):
    """Everything this check reads off the disk."""
    schema = Schema()
    schema.load(DESCRIPTOR)
    ledger_ids = set()
    problems = []
    if ledger_path.exists():
        with ledger_path.open(newline="", encoding="utf-8") as handle:
            ledger_ids = {row["id"] for row in csv.DictReader(handle)}
    else:
        problems.append(f"{ledger_path} is missing, so the tie cannot be checked")
    rows, columns = read_tie(TIE)
    tied, found = check_rows(rows, columns, schema, ledger_ids, TIE.name)
    problems += found
    names, refused = fixtures_in(fixtures_path / WIRE_VERSION)
    table = CLASSIFICATION.read_text(encoding="utf-8") if CLASSIFICATION.exists() else None
    return schema, tied, names, refused, table, problems


def selftest(ledger_path, fixtures_path):
    """Make each rule fail on purpose, once, and fail if it does not.

    Every case below is a mutation of the real schema, so the mutation is the only
    difference between a run that passes and a run that reports.
    """
    schema, tied, names, refused, table, problems = load(ledger_path, fixtures_path)
    if problems:
        print("selftest needs a tie that holds first:")
        return fail(problems)
    messages = schema.real_messages()
    cases = []

    # Direction one: a ledger row naming a path the schema does not have. This is
    # the renamed field and the deleted message, caught at commit time by the row
    # that pointed at it. `C16` really does name `contract.v1.Attempt`, so the
    # mutation is one field name on a real row.
    rows, columns = read_tie(TIE)
    renamed = [dict(row) for row in rows]
    renamed[0]["proto"] = f"{renamed[0]['proto']}.provider_name"
    _, found = check_rows(renamed, columns, schema, set(), TIE.name)
    cases.append(("a named path that is not in the schema", found, True))

    # And a row naming a ledger id that does not exist.
    unknown = [dict(row) for row in rows]
    unknown[0]["id"] = "ZZ999"
    _, found = check_rows(unknown, columns, schema, {row["id"] for row in rows}, TIE.name)
    cases.append(("a tie row with no ledger row", found, True))

    # And the wrong columns, which is how a sidecar quietly stops being read.
    _, found = check_rows([], ["id", "proto", "extra"], schema, set(), TIE.name)
    cases.append(("the wrong columns in the sidecar", found, True))

    # Direction two: a message nobody specified.
    with_orphan = dict(messages)
    with_orphan[f"{PACKAGE}.Orphan"] = {}
    found, _, _ = audit(schema, with_orphan, tied, names, refused, table)
    cases.append(("an unreachable message", [p for p in found if "Orphan" in p], True))

    # An enum with no zero value, so a build that does not know a word has nothing
    # to read it as.
    bare = Schema()
    bare.messages, bare.enums = dict(schema.messages), dict(schema.enums)
    bare.enums[f"{PACKAGE}.NoUnknown"] = {"NO_UNKNOWN_ONE": 1}
    found, _, _ = audit(bare, messages, tied, names, refused, table)
    cases.append(
        ("an enum with no zero value", [p for p in found if "NoUnknown" in p], True)
    )

    # A refused fixture the classification table does not name: the exact defect
    # the spike shipped.
    found, _, _ = audit(schema, messages, tied, names, refused | {"Ghost.unclassified"}, table)
    cases.append(("an unclassified refused fixture", [p for p in found if "Ghost" in p], True))

    # And the last case is the control: with nothing mutated, every rule above is
    # quiet. Without it a check that reports everything would pass this selftest.
    found, _, _ = audit(schema, messages, tied, names, refused, table)
    cases.append(("the tie as it stands", found, False))

    bad = []
    for what, reported, must_report in cases:
        if must_report and not reported:
            bad.append(f"{what}: the check did not report it")
        elif not must_report and reported:
            bad.append(f"{what}: reported {len(reported)}, first {reported[0]}")
        else:
            print(f"  ok   {what}: {'reported' if must_report else 'nothing reported'}")
    if bad:
        print("the selftest does not hold:")
        return fail(bad)
    print(f"selftest: {len(cases)} cases, each behaving as it must")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", default=str(FIXTURES))
    parser.add_argument("--ledger", default=str(LEDGER))
    parser.add_argument(
        "--selftest",
        action="store_true",
        help="make each rule fail on purpose and fail if it does not",
    )
    arguments = parser.parse_args()

    if not DESCRIPTOR.exists():
        print(f"{DESCRIPTOR} is missing: run `cargo build` first")
        return 1

    ledger_path = Path(arguments.ledger)
    fixtures_path = Path(arguments.fixtures)
    if arguments.selftest:
        return selftest(ledger_path, fixtures_path)

    schema, tied, names, refused, table, problems = load(ledger_path, fixtures_path)
    messages = schema.real_messages()
    found, reached, orphans = audit(schema, messages, tied, names, refused, table)
    problems += found

    print(f"messages {len(messages)}  enums {len(schema.enums)}")
    print(f"ledger rows tied {len(tied)}  paths {sum(len(p) for p in tied.values())}")
    print(f"messages reached {len(reached)}  orphans {len(orphans)}")
    if RULES.exists():
        rules = RULES.read_text(encoding="utf-8")
        print(f"fields with a rule {rules.count(': [')}")
        print(f"presence_lost declarations {rules.count('presence_lost')}")
    else:
        problems.append(f"{RULES} is missing: run the generator")

    if problems:
        print("the tie does not hold:")
        return fail(problems)
    print("the tie holds")
    return 0


if __name__ == "__main__":
    sys.exit(main())
