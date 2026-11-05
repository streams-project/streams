#!/usr/bin/env python3
"""Generate and check contract/ledger.csv.

The ledger has one row per behaviour, High defect and new row. Each row says
what happens to that id, and where its one rule lives. Decisions are held in
tools/ledger_plan.json; this script only joins them to the id lists.

  python3 tools/ledger.py check
      Checks the committed ledger against the plan. Needs no other file.

  python3 tools/ledger.py generate --behaviours FILE --defects FILE
      Reads the ids from the two audit lists, refuses any id that is missing
      or unknown, and rewrites the ledger.

Columns:
  id        S, C, P, U behaviour, H High defect, N new row
  kind      behaviour, defect or new
  status    carried, changed, dropped or conditional
  home      server, core, offline, platform_app or dropped
  place     the exact part inside that home
  new_rule  the rule that replaces the old one, for changed and conditional
  reason    why it was dropped or conditional, or where the design states
            the change. Defect rows name their gate.

Exit code 0 means the ledger holds; 1 means it does not, and says why.
"""

import argparse
import csv
import io
import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "contract" / "ledger.csv"
PLAN = ROOT / "tools" / "ledger_plan.json"

COLUMNS = ["id", "kind", "status", "home", "place", "new_rule", "reason"]
STATUSES = ("carried", "changed", "dropped", "conditional")
HOMES = ("server", "core", "offline", "platform_app", "dropped")
ORDER = "SCPUHN"


class Refused(Exception):
    """The ledger or plan breaks a rule. Holds every problem found."""

    def __init__(self, problems):
        super().__init__("\n".join(problems))
        self.problems = problems


def expand(text):
    """Turn 'S37-43 C2' into ['S37', ..., 'S43', 'C2']."""
    ids = []
    for token in text.split():
        match = re.fullmatch(r"([A-Z])(\d+)(?:-(\d+))?", token)
        if match is None:
            raise Refused([f"plan has a bad id token: {token}"])
        first = int(match.group(2))
        last = int(match.group(3) or first)
        ids += [f"{match.group(1)}{n}" for n in range(first, last + 1)]
    return ids


def sort_key(wanted):
    return ORDER.index(wanted[0]), int(wanted[1:])


def known_ids(plan):
    return [f"{p}{n}" for p, top in plan["ids"].items() for n in range(1, top + 1)]


def kind_of(wanted):
    return {"H": "defect", "N": "new"}.get(wanted[0], "behaviour")


def home_of(plan):
    return {place: home for home, places in plan["homes"].items() for place in places}


def dropped_ids(plan):
    return {i: e["reason"] for e in plan["dropped"] for i in expand(e["ids"])}


def conditional_ids(plan):
    return set(expand(plan["conditional"]["ids"]))


def placed_ids(plan):
    """Map each behaviour id to its place. Refuse a gap or a double."""
    places = {}
    problems = []

    def put(wanted, place):
        if wanted in places:
            problems.append(f"plan places {wanted} twice")
        places[wanted] = place

    for place, ids in plan["places"].items():
        for wanted in expand(ids):
            put(wanted, place)
    for wanted in dropped_ids(plan):
        put(wanted, "dropped")
    for wanted in conditional_ids(plan):
        put(wanted, "conditional on the owner")
    for gate in plan["gates"]:
        for place, ids in gate["places"].items():
            for wanted in expand(ids):
                put(wanted, place)
    expected = known_ids(plan)
    problems += [f"plan places no row for {i}" for i in expected if i not in places]
    problems += [f"plan places unknown id {i}" for i in places if i not in expected]
    if problems:
        raise Refused(problems)
    return places


def change_entries(plan):
    """Map each changed behaviour id to its rule and the design section."""
    changes = {}
    problems = []
    blocked = set(dropped_ids(plan)) | conditional_ids(plan)
    for entry in plan["changes"]:
        for wanted in expand(entry["ids"]):
            if wanted in changes:
                problems.append(f"plan changes {wanted} twice")
            if wanted in blocked:
                problems.append(f"plan changes {wanted}, which is dropped or conditional")
            changes[wanted] = entry
    if problems:
        raise Refused(problems)
    return changes


def build_rows(plan):
    """Return the full ledger the plan describes, in fixed order."""
    places = placed_ids(plan)
    homes = home_of(plan)
    drops = dropped_ids(plan)
    conditional = conditional_ids(plan)
    changes = change_entries(plan)
    gates = {i: g for g in plan["gates"] for ids in g["places"].values() for i in expand(ids)}
    rows = []
    for wanted in sorted(places, key=sort_key):
        place = places[wanted]
        row = dict.fromkeys(COLUMNS, "")
        row.update(id=wanted, kind=kind_of(wanted), place=place, home=homes[place])
        if wanted in drops:
            row.update(status="dropped", reason=drops[wanted])
        elif wanted in conditional:
            entry = plan["conditional"]
            row.update(status="conditional", new_rule=entry["rule"], reason=entry["reason"])
        elif wanted in gates:
            gate = gates[wanted]
            row.update(status="changed", new_rule=gate["rule"], reason=f"gate {gate['gate']}")
        elif wanted in changes:
            entry = changes[wanted]
            row.update(status="changed", new_rule=entry["rule"], reason=entry["basis"])
        else:
            row["status"] = "carried"
        rows.append(row)
    for entry in plan["new"]:
        row = dict.fromkeys(COLUMNS, "")
        row.update(
            id=entry["id"],
            kind="new",
            status="changed",
            home=homes[entry["place"]],
            place=entry["place"],
            new_rule=entry["rule"],
            reason=entry["basis"],
        )
        rows.append(row)
    return rows


def render(rows):
    buffer = io.StringIO()
    writer = csv.DictWriter(buffer, fieldnames=COLUMNS, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return buffer.getvalue()


def read_ledger(path):
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames != COLUMNS:
            raise Refused([f"ledger columns are {reader.fieldnames}, expected {COLUMNS}"])
        return list(reader)


def check_ids(rows, plan):
    expected = known_ids(plan) + [entry["id"] for entry in plan["new"]]
    seen = Counter(row["id"] for row in rows)
    problems = [f"missing id {i}" for i in expected if i not in seen]
    problems += [f"duplicate id {i}" for i, n in seen.items() if n > 1]
    problems += [f"unknown id {i}" for i in seen if i not in expected]
    return problems


def check_row(row, homes, drops, conditional):
    wanted, status, home, place = row["id"], row["status"], row["home"], row["place"]
    problems = []
    if status not in STATUSES:
        problems.append(f"{wanted} has unknown status {status}")
    if home not in HOMES:
        problems.append(f"{wanted} has unknown home {home}")
    if row["kind"] != kind_of(wanted):
        problems.append(f"{wanted} has kind {row['kind']}, expected {kind_of(wanted)}")
    if place not in homes:
        problems.append(f"{wanted} has unknown place {place}")
    elif homes[place] != home:
        problems.append(f"{wanted} has place {place}, which belongs to home {homes[place]}")

    has_rule = bool(row["new_rule"].strip())
    has_reason = bool(row["reason"].strip())
    if status in ("changed", "conditional") and not has_rule:
        problems.append(f"{wanted} is {status} with no new rule text")
    if status == "carried" and has_rule:
        problems.append(f"{wanted} is carried but holds new rule text")
    if status in ("dropped", "conditional") and not has_reason:
        problems.append(f"{wanted} is {status} with no reason")
    if (home == "dropped") != (status == "dropped"):
        problems.append(f"{wanted} has home {home} but status {status}")
    if row["kind"] == "defect" and status != "changed":
        problems.append(f"{wanted} is a High defect and cannot have status {status}")

    if wanted in drops and status != "dropped":
        problems.append(f"{wanted} is on the drop list but has status {status}")
    if wanted not in drops and status == "dropped":
        problems.append(f"{wanted} has status dropped but is not on the drop list")
    if wanted in conditional and status != "conditional":
        problems.append(f"{wanted} is conditional on the owner but has status {status}")
    if wanted not in conditional and status == "conditional":
        problems.append(f"{wanted} has status conditional but is not on the owner's list")
    return problems


def tally(rows):
    behaviours = [row for row in rows if row["kind"] == "behaviour"]
    return {
        "behaviours": len(behaviours),
        "high": sum(row["kind"] == "defect" for row in rows),
        "dropped": sum(row["status"] == "dropped" for row in rows),
        "conditional": sum(row["status"] == "conditional" for row in rows),
        "core": sum(row["home"] == "core" for row in behaviours),
        "offline": sum(row["home"] == "offline" for row in behaviours),
    }


def check_counts(rows, plan):
    problems = []
    counts = tally(rows)
    labels = {"core": "core has {} behaviours", "offline": "offline has {} behaviours"}
    for name, expected in plan["expect"].items():
        if counts[name] != expected:
            label = labels.get(name, name + " count is {}").format(counts[name])
            problems.append(f"{label}, expected {expected}")
    return problems


def check_against_plan(rows, plan):
    expected = {row["id"]: row for row in build_rows(plan)}
    return [
        f"{row['id']} differs from the plan; change tools/ledger_plan.json and regenerate"
        for row in rows
        if row["id"] in expected and row != expected[row["id"]]
    ]


def check(rows, plan):
    placed_ids(plan)
    homes = home_of(plan)
    drops = dropped_ids(plan)
    conditional = conditional_ids(plan)
    problems = check_ids(rows, plan)
    for row in rows:
        problems += check_row(row, homes, drops, conditional)
    problems += check_counts(rows, plan)
    problems += check_against_plan(rows, plan)
    if problems:
        raise Refused(problems)


def report(rows):
    counts = tally(rows)
    for name, value in counts.items():
        print(f"{name} {value}")
    print(f"new {sum(row['kind'] == 'new' for row in rows)}")
    print(f"rows {len(rows)}")
    for field in ("status", "home"):
        for kind in ("behaviour", "defect", "new"):
            found = Counter(row[field] for row in rows if row["kind"] == kind)
            print(f"{kind} {field}: " + ", ".join(f"{k} {v}" for k, v in sorted(found.items())))


def read_note_ids(path, prefixes):
    pattern = re.compile(rf"^\| ([{prefixes}]\d+) \|", re.MULTILINE)
    return pattern.findall(path.read_text(encoding="utf-8"))


def check_notes(plan, behaviours, defects):
    """Refuse when the audit lists and the plan disagree on any id."""
    found = read_note_ids(behaviours, "SCPU") + read_note_ids(defects, "H")
    expected = known_ids(plan)
    problems = [f"missing id {i} in the audit lists" for i in expected if i not in found]
    problems += [f"unknown id {i} in the audit lists" for i in found if i not in expected]
    problems += [f"duplicate id {i} in the audit lists" for i, n in Counter(found).items() if n > 1]
    if problems:
        raise Refused(problems)


def main():
    parser = argparse.ArgumentParser(description="Generate and check contract/ledger.csv.")
    parser.add_argument("--plan", type=Path, default=PLAN)
    commands = parser.add_subparsers(dest="command", required=True)

    run_check = commands.add_parser("check")
    run_check.add_argument("--ledger", type=Path, default=LEDGER)
    run_check.add_argument("--plan", type=Path, default=argparse.SUPPRESS)

    run_generate = commands.add_parser("generate")
    run_generate.add_argument("--behaviours", type=Path, required=True)
    run_generate.add_argument("--defects", type=Path, required=True)
    run_generate.add_argument("--out", type=Path, default=LEDGER)
    run_generate.add_argument("--plan", type=Path, default=argparse.SUPPRESS)

    args = parser.parse_args()
    plan = json.loads(args.plan.read_text(encoding="utf-8"))
    try:
        if args.command == "generate":
            check_notes(plan, args.behaviours, args.defects)
            rows = build_rows(plan)
            check(rows, plan)
            args.out.write_text(render(rows), encoding="utf-8")
            print(f"wrote {args.out}")
        else:
            rows = read_ledger(args.ledger)
            check(rows, plan)
            print("ledger holds")
        report(rows)
    except Refused as refused:
        for problem in refused.problems:
            print(f"refused: {problem}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
