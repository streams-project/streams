"""The one entry point. Nothing here reaches a network: every command run stops before one."""

import pytest

from databuild import cli


def test_c13_a_scan_of_a_clean_folder_passes(tmp_path, capsys):
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
