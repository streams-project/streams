#!/usr/bin/env python3
"""Every id a test names must be a real row of the contract ledger.

A test function here is named after the rules it proves, for example
`s72_quality_gates_for_rows_and_for_judged_pages`. The ledger
(`../platform/contract/ledger.csv`) is the only list of behaviour and defect
ids, and the report of what this server covers is read off these names, so a
name that claims an id the ledger does not hold, or holds for another layer,
reports coverage that does not exist.

Two other id lists exist beside the ledger: the threat model and the failure
matrix of the cross project plan. Their ids reuse the ledger's letters (the
plan's own critic asked for a prefix), so a test that proves one of those rows
writes it with its namespace: `tm` for a threat row and `fm` for a failure
matrix row, for example `tmt13_...` for threat T13 and `fmg5_...` for gate G5.
The two sets below are frozen from that plan and are the only ids those
prefixes may carry.

    python3 scripts/check_test_ids.py [--ledger PATH] [--report]

Exit 0 when every id in every test name resolves, 1 otherwise.
"""

from __future__ import annotations

import argparse
import csv
import os
import re
import sys
from pathlib import Path

# Threat model rows (90). Frozen from the cross project plan, 2026-11-05.
THREAT_IDS = (
    {f"T{n}" for n in range(1, 20)}
    | {f"P{n}" for n in range(1, 9)}
    | {f"K{n}" for n in range(1, 16)}
    | {f"F{n}" for n in range(1, 13)}
    | {f"S{n}" for n in range(1, 7)}
    | {f"O{n}" for n in range(1, 9)}
    | {f"C{n}" for n in range(1, 10)}
    | {f"R{n}" for n in range(1, 14)}
)

# Failure matrix rows and gates. Frozen from the same plan.
FAILURE_IDS = {f"F{n}" for n in range(1, 15)} | {f"G{n}" for n in range(1, 11)}

# A word of a test name that reads as an id: one letter and a number, under an
# optional namespace prefix. `utf16` and the like are words, not ids.
ID_WORD = re.compile(r"^(tm|fm)?([a-z])(\d{1,3})$")
FN = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([a-z][a-z0-9_]*)")
ATTRIBUTE = re.compile(r"^\s*#\[")


def ledger_ids(path: Path) -> set[str]:
    """Every id in the ledger."""
    with path.open(newline="", encoding="utf-8") as handle:
        return {row["id"].strip() for row in csv.DictReader(handle) if row.get("id")}


def test_functions(root: Path) -> list[tuple[Path, int, str]]:
    """Every `#[test]` or `#[tokio::test]` function under `root`, with its place."""
    found: list[tuple[Path, int, str]] = []
    for file in sorted(root.rglob("*.rs")):
        attributes: list[str] = []
        for number, line in enumerate(file.read_text(encoding="utf-8").splitlines(), 1):
            if ATTRIBUTE.match(line):
                attributes.append(line)
                continue
            name = FN.match(line)
            if name:
                if any("test]" in a for a in attributes):
                    found.append((file, number, name.group(1)))
            if line.strip() and not ATTRIBUTE.match(line):
                attributes = []
    return found


def check(name: str, ledger: set[str]) -> list[str]:
    """What is wrong with the ids in this test name."""
    wrong = []
    for word in name.split("_"):
        parts = ID_WORD.match(word)
        if not parts:
            continue
        prefix, letter, number = parts.groups()
        id_text = f"{letter.upper()}{int(number)}"
        if prefix == "tm":
            if id_text not in THREAT_IDS:
                wrong.append(f"{word}: no threat row TM-{id_text}")
        elif prefix == "fm":
            if id_text not in FAILURE_IDS:
                wrong.append(f"{word}: no failure matrix row FM-{id_text}")
        elif id_text not in ledger:
            hint = ""
            if id_text in THREAT_IDS:
                hint = f" (threat row TM-{id_text} is written tm{id_text.lower()})"
            elif id_text in FAILURE_IDS:
                hint = f" (failure row FM-{id_text} is written fm{id_text.lower()})"
            wrong.append(f"{word}: {id_text} is not a ledger row{hint}")
    return wrong


def main() -> int:
    here = Path(__file__).resolve().parent.parent
    default = os.environ.get("LEDGER") or here.parent / "platform" / "contract" / "ledger.csv"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", default=str(default), help="path of ledger.csv")
    parser.add_argument(
        "--report", action="store_true", help="print the ledger ids the tests name"
    )
    args = parser.parse_args()

    path = Path(args.ledger)
    if not path.is_file():
        print(f"no ledger at {path}: check out the platform repository beside this one")
        return 1
    ledger = ledger_ids(path)
    if not ledger:
        print(f"{path} holds no ids")
        return 1

    tests = test_functions(here / "crates")
    if not tests:
        print("no tests found: this check would pass for the wrong reason")
        return 1

    faults = 0
    named: set[str] = set()
    for file, number, name in tests:
        for fault in check(name, ledger):
            print(f"{file.relative_to(here)}:{number}: {name}: {fault}")
            faults += 1
        for word in name.split("_"):
            parts = ID_WORD.match(word)
            if parts and not parts.group(1):
                id_text = f"{parts.group(2).upper()}{int(parts.group(3))}"
                if id_text in ledger:
                    named.add(id_text)

    print(f"{len(tests)} tests, {len(named)} ledger ids named, {faults} faults")
    if args.report:
        for id_text in sorted(named, key=lambda i: (i[0], int(i[1:]))):
            print(id_text)
    return 1 if faults else 0


if __name__ == "__main__":
    sys.exit(main())
