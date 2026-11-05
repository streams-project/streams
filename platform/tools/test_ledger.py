#!/usr/bin/env python3
"""Tests for tools/ledger.py. Each planted fault names the id it plants.

Run from anywhere with: python3 tools/test_ledger.py
"""

import csv
import io
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
ROOT = TOOLS.parent
SCRIPT = TOOLS / "ledger.py"
LEDGER = ROOT / "contract" / "ledger.csv"
PLAN = TOOLS / "ledger_plan.json"


def run(*args):
    """Run the real script as a process and return (exit code, all output)."""
    done = subprocess.run(
        [sys.executable, str(SCRIPT), *map(str, args)],
        capture_output=True,
        text=True,
    )
    return done.returncode, done.stdout + done.stderr


def read_rows():
    with LEDGER.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def write_rows(path, rows, fields):
    buffer = io.StringIO()
    writer = csv.DictWriter(buffer, fieldnames=fields, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    path.write_text(buffer.getvalue(), encoding="utf-8")


class Ledger(unittest.TestCase):
    def setUp(self):
        self.folder = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.folder)
        self.copy = self.folder / "ledger.csv"
        self.rows = read_rows()
        self.fields = list(self.rows[0].keys())

    def row(self, wanted):
        return next(row for row in self.rows if row["id"] == wanted)

    def assert_refused(self, words):
        write_rows(self.copy, self.rows, self.fields)
        code, output = run("check", "--ledger", self.copy)
        self.assertEqual(code, 1, output)
        self.assertIn(words, output)

    def test_real_ledger_passes(self):
        code, output = run("check")
        self.assertEqual(code, 0, output)

    def test_counts_match_note_153(self):
        code, output = run("check")
        self.assertEqual(code, 0, output)
        for line in (
            "behaviours 444",
            "high 133",
            "dropped 15",
            "conditional 5",
            "core 51",
            "offline 7",
        ):
            self.assertIn(line, output)

    def test_unchanged_copy_passes(self):
        write_rows(self.copy, self.rows, self.fields)
        code, output = run("check", "--ledger", self.copy)
        self.assertEqual(code, 0, output)

    def test_planted_missing_behaviour_s1_fails(self):
        self.rows.remove(self.row("S1"))
        self.assert_refused("missing id S1")

    def test_planted_missing_high_h133_fails(self):
        self.rows.remove(self.row("H133"))
        self.assert_refused("missing id H133")

    def test_planted_duplicate_c1_fails(self):
        self.rows.append(dict(self.row("C1")))
        self.assert_refused("duplicate id C1")

    def test_planted_unknown_id_s229_fails(self):
        extra = dict(self.row("S1"))
        extra["id"] = "S229"
        self.rows.append(extra)
        self.assert_refused("unknown id S229")

    def test_planted_changed_c18_with_no_text_fails(self):
        self.row("C18")["new_rule"] = ""
        self.assert_refused("C18 is changed with no new rule text")

    def test_planted_changed_h1_with_blank_text_fails(self):
        self.row("H1")["new_rule"] = "   "
        self.assert_refused("H1 is changed with no new rule text")

    def test_planted_carried_row_with_text_fails(self):
        row = self.row("C1")
        self.assertEqual(row["status"], "carried")
        row["new_rule"] = "A rule hidden in a carried row."
        self.assert_refused("C1 is carried but holds new rule text")

    def test_planted_dropped_c83_marked_carried_contradicts_drop_list(self):
        row = self.row("C83")
        row.update(status="carried", home="server", place="server play", reason="")
        self.assert_refused("C83 is on the drop list but has status carried")

    def test_planted_carried_c1_marked_dropped_contradicts_drop_list(self):
        self.row("C1").update(
            status="dropped", home="dropped", place="dropped", reason="Planted."
        )
        self.assert_refused("C1 has status dropped but is not on the drop list")

    def test_planted_dropped_c10_with_no_reason_fails(self):
        self.row("C10")["reason"] = ""
        self.assert_refused("C10 is dropped with no reason")

    def test_planted_home_dropped_on_carried_row_fails(self):
        self.row("C1")["home"] = "dropped"
        self.assert_refused("C1 has home dropped but status carried")

    def test_planted_conditional_p12_marked_dropped_fails(self):
        self.row("P12").update(status="dropped", home="dropped", place="dropped")
        self.assert_refused("P12 is conditional on the owner but has status dropped")

    def test_planted_core_row_moved_to_server_fails(self):
        self.row("C1").update(home="server", place="server play")
        self.assert_refused("core has 50 behaviours, expected 51")

    def test_planted_offline_row_moved_to_server_fails(self):
        self.row("C2").update(home="server", place="server library")
        self.assert_refused("offline has 6 behaviours, expected 7")

    def test_planted_place_outside_its_home_fails(self):
        self.row("S1")["place"] = "core player"
        self.assert_refused("S1 has place core player, which belongs to home core")

    def test_planted_defect_h1_carried_fails(self):
        self.row("H1").update(status="carried", new_rule="")
        self.assert_refused("H1 is a High defect and cannot have status carried")

    def test_planted_new_row_n1_with_no_text_fails(self):
        self.row("N1")["new_rule"] = ""
        self.assert_refused("N1 is changed with no new rule text")

    def test_hand_edit_of_rule_text_fails(self):
        self.row("C18")["new_rule"] = "Some other rule."
        self.assert_refused("C18 differs from the plan")

    def test_plan_missing_a_placement_fails(self):
        plan = json.loads(PLAN.read_text(encoding="utf-8"))
        plan["places"]["web"] = plan["places"]["web"].replace("U7", "")
        planted = self.folder / "plan.json"
        planted.write_text(json.dumps(plan), encoding="utf-8")
        code, output = run("check", "--plan", planted)
        self.assertEqual(code, 1, output)
        self.assertIn("plan places no row for U7", output)

    def notes(self, leave_out=None):
        """Write minimal notes that hold only ids, with one id left out."""
        ids = [row["id"] for row in self.rows if row["kind"] != "new"]
        behaviours = self.folder / "behaviours.md"
        defects = self.folder / "defects.md"
        behaviours.write_text(
            "".join(
                f"| {i} | a |\n"
                for i in ids
                if i[0] != "H" and i != leave_out
            ),
            encoding="utf-8",
        )
        defects.write_text(
            "### High\n"
            + "".join(
                f"| {i} | a |\n"
                for i in ids
                if i[0] == "H" and i != leave_out
            )
            + "### Medium\n| M1 | a |\n",
            encoding="utf-8",
        )
        return behaviours, defects

    def test_generate_rebuilds_identical_bytes_from_ids(self):
        behaviours, defects = self.notes()
        out = self.folder / "out.csv"
        code, output = run(
            "generate",
            "--behaviours", behaviours,
            "--defects", defects,
            "--out", out,
        )
        self.assertEqual(code, 0, output)
        self.assertEqual(out.read_bytes(), LEDGER.read_bytes())

    def test_generate_fails_when_note_lacks_s5(self):
        behaviours, defects = self.notes(leave_out="S5")
        out = self.folder / "out.csv"
        code, output = run(
            "generate",
            "--behaviours", behaviours,
            "--defects", defects,
            "--out", out,
        )
        self.assertEqual(code, 1, output)
        self.assertIn("missing id S5", output)
        self.assertFalse(out.exists())


if __name__ == "__main__":
    unittest.main()
