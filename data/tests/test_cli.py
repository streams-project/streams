"""The one entry point. Nothing here reaches a network: every command run stops before one."""

import subprocess
from pathlib import Path

import pytest

from databuild import cli


def test_a_scan_of_a_clean_folder_passes(tmp_path, capsys):
    (tmp_path / "a.txt").write_text("plain words\n")
    assert cli.main(["scan", str(tmp_path)]) == 0
    assert "0 findings" in capsys.readouterr().out


def test_h133_a_scan_finding_ends_the_job_with_a_failure(tmp_path, capsys):
    (tmp_path / "a.txt").write_text("see https://" + "real-host.org/page\n")
    assert cli.main(["scan", str(tmp_path)]) == cli.EXIT_FAILED
    assert "address in a file" in capsys.readouterr().out


def test_a_missing_setting_turns_the_stage_off_and_names_itself(tmp_path, monkeypatch, capsys):
    monkeypatch.chdir(tmp_path)
    assert cli.main(["pull"]) == cli.EXIT_OFF
    assert "tmdb" in capsys.readouterr().err


def test_a_stage_with_no_input_fails_with_a_plain_word(tmp_path, monkeypatch, capsys):
    monkeypatch.chdir(tmp_path)
    assert cli.main(["clean"]) == cli.EXIT_FAILED
    assert "no pull file" in capsys.readouterr().err


def test_an_unreadable_settings_file_fails_plainly(tmp_path, capsys):
    assert cli.main(["--settings", str(tmp_path / "no.toml"), "clean"]) == cli.EXIT_FAILED
    assert "cannot read" in capsys.readouterr().err


def test_a_bad_date_is_refused_at_the_door(capsys):
    with pytest.raises(SystemExit):
        cli.main(["--as-of", "soon", "clean"])


def test_folders_given_on_the_command_line_win(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    assert cli.main(["--work", "w", "--out", "o", "clean"]) == cli.EXIT_FAILED
    assert (tmp_path / "w").is_dir()
    assert (tmp_path / "o").is_dir()


def test_outside_a_repository_the_commit_reads_unknown(monkeypatch):
    monkeypatch.setattr(cli, "_git", lambda *a: "")
    assert cli.build_commit() == "unknown"


def test_a_tree_with_other_changes_is_marked(monkeypatch):
    answers = {"rev-parse": "abc", "status": " M x"}
    monkeypatch.setattr(cli, "_git", lambda *a: answers[a[0]])
    assert cli.build_commit() == "abc-dirty"


JOBS = Path(__file__).resolve().parents[1] / "jobs"


def _fake_uv(folder, fail_on="no such step"):
    log = folder / "calls"
    tool = folder / "uv"
    body = f'''#!/bin/sh
echo "$*" >> "{log}"
case "$*" in *"{fail_on}"*) exit 1;; esac
'''
    tool.write_text(body)
    tool.chmod(0o755)
    return log


def _run_job(folder, word):
    env = {
        "PATH": f"{folder}:/usr/bin:/bin",
        "DATABUILD_HOME": str(folder),
        "DATABUILD_SETTINGS": str(folder / "s.toml"),
    }
    return subprocess.run(
        [JOBS / "run", word], env=env, capture_output=True, text=True, check=False
    )


def test_s151_new_titles_run_every_six_hours_at_minute_23():
    assert "OnCalendar=*-*-* 00/6:23:00 UTC" in (JOBS / "databuild-fresh.timer").read_text()


def test_h62_each_job_ends_by_proving_the_set_and_a_failure_reaches_the_alarm(tmp_path):
    log = _fake_uv(tmp_path)
    assert _run_job(tmp_path, "daily").returncode == 0
    steps = [line.split()[-1] for line in log.read_text().splitlines()]
    assert steps == ["popularity", "manifest", "verify", "age"]
    for unit in ("databuild-fresh.service", "databuild-daily.service"):
        assert "OnFailure=databuild-alarm.service" in (JOBS / unit).read_text()


def test_h62_a_stale_set_fails_the_job_so_no_green_check_hides_it(tmp_path):
    _fake_uv(tmp_path, fail_on="age")
    assert _run_job(tmp_path, "fresh").returncode != 0


def test_h99_a_failed_pull_stops_before_a_new_manifest(tmp_path):
    log = _fake_uv(tmp_path, fail_on="popularity")
    assert _run_job(tmp_path, "daily").returncode != 0
    assert "manifest" not in log.read_text()


def test_a_job_with_no_settings_names_what_is_missing(tmp_path):
    done = subprocess.run(
        [JOBS / "run", "fresh"],
        env={"PATH": "/usr/bin:/bin"},
        capture_output=True,
        text=True,
        check=False,
    )
    assert done.returncode != 0
    assert "DATABUILD_HOME" in done.stderr


def test_an_alarm_with_no_command_says_so_and_a_set_one_runs(tmp_path):
    off = subprocess.run([JOBS / "alarm"], env={}, capture_output=True, text=True, check=False)
    assert off.returncode != 0
    assert "ALARM_COMMAND" in off.stderr
    mark = tmp_path / "rang"
    on = subprocess.run(
        [JOBS / "alarm"], env={"ALARM_COMMAND": f"touch {mark}"}, capture_output=True, check=False
    )
    assert on.returncode == 0
    assert mark.exists()
