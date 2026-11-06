"""Every ledger row this repository owns has a named test."""

import csv
import re
from pathlib import Path

from .siblings import within

ROOT = Path(__file__).resolve().parent


def test_every_ledger_row_of_this_repository_has_a_named_test():
    ledger = within("platform", "contract", "ledger.csv")
    with open(ledger, encoding="utf-8", newline="") as source:
        owned = [row["id"] for row in csv.DictReader(source) if row["place"].lower() == "data"]
    assert owned
    names = re.findall(r"def (test_[a-z0-9_]+)", "".join(p.read_text() for p in ROOT.glob("*.py")))
    missing = [i for i in owned if not any(n.startswith(f"test_{i.lower()}_") for n in names)]
    assert missing == []
