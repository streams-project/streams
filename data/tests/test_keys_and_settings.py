"""Keys, settings and the transport rules."""

import json
import tomllib
from dataclasses import fields
from pathlib import Path

import pytest

from databuild import settings as S
from databuild.errors import FeatureOff, StageFailed
from databuild.keys import TV_BIT, item_key, parse_title_key, title_key, unpack
from databuild.net import NetError, Request, UrlTransport
from databuild.scan import scan_own_words, scan_paths

CONTRACT = Path(__file__).resolve().parents[2] / "platform" / "contract"


def test_movie_keeps_its_id_and_a_series_sets_the_top_bit():
    assert item_key("movie", 12) == 12
    assert item_key("tv", 12) == 12 | TV_BIT
    assert unpack(item_key("tv", 12)) == ("tv", 12)
    assert unpack(12) == ("movie", 12)


@pytest.mark.parametrize("bad", ["12", "movie:0", "show:1", "MOVIE:1", "movie:01", "", 12, None])
def test_a_bare_id_or_a_bad_key_does_not_parse(bad):
    with pytest.raises(ValueError):
        parse_title_key(bad)


def test_keys_round_trip():
    assert parse_title_key(title_key("tv", 77)) == ("tv", 77)


def test_every_refused_title_key_in_the_contract_is_refused_here():
    folder = CONTRACT / "fixtures" / "2026-11-05" / "refused"
    if not folder.is_dir():
        pytest.skip("the platform repository is not next to this one")
    files = sorted(folder.glob("TitleKey.*.json"))
    assert files
    for path in files:
        with pytest.raises(ValueError):
            parse_title_key(json.loads(path.read_text()))


def test_no_file_means_every_outside_feature_is_off():
    loaded = S.load(None)
    assert loaded.tmdb.api_base == ""
    assert loaded.tmdb.key_file == ""
    assert loaded.name.endpoint == ""
    assert loaded.open.wikidata_endpoint == ""


def test_a_missing_setting_turns_the_feature_off_and_names_itself():
    with pytest.raises(FeatureOff, match=r"tmdb.api_base"):
        S.need("", "tmdb.api_base")
    with pytest.raises(FeatureOff, match=r"tmdb.key_file"):
        S.read_secret("", "tmdb.key_file")


def test_a_typing_mistake_in_settings_is_refused(tmp_path):
    path = tmp_path / "s.toml"
    path.write_text('[tmdb]\napi_bas = "x"\n')
    with pytest.raises(StageFailed, match="api_bas"):
        S.load(path)
    path.write_text('[nothing]\nx = "y"\n')
    with pytest.raises(StageFailed, match="nothing"):
        S.load(path)


def test_settings_load_every_section(tmp_path):
    path = tmp_path / "s.toml"
    path.write_text(
        '[paths]\nwork = "w"\n[tmdb]\nlanguages = ["en-US", "fr-FR"]\n[embed]\ndimensions = 16\n'
    )
    loaded = S.load(path)
    assert loaded.paths.work == "w"
    assert loaded.tmdb.languages == ("en-US", "fr-FR")
    assert loaded.embed.dimensions == 16


def test_a_secret_never_prints(tmp_path):
    path = tmp_path / "k"
    path.write_text("sensitive-value\n")
    secret = S.read_secret(str(path), "x")
    assert secret.reveal() == "sensitive-value"
    assert "sensitive" not in repr(secret)
    assert "sensitive" not in f"{secret}"


def test_an_empty_key_file_is_a_failure_with_no_content_in_the_text(tmp_path):
    path = tmp_path / "k"
    path.write_text("  \n")
    with pytest.raises(StageFailed, match="empty"):
        S.read_secret(str(path), "x")


def test_a_network_error_text_holds_no_address_and_no_query():
    request = Request("http://127.0.0.1:1", "/x", {"api_key": "sensitive"}, timeout=0.2)
    with pytest.raises(NetError) as caught:
        UrlTransport().fetch(request)
    assert "sensitive" not in str(caught.value)
    assert "127.0.0.1" not in str(caught.value)


def test_the_example_file_lists_every_setting_with_no_address_and_no_key():
    example = Path(__file__).resolve().parents[1] / "config" / "settings.example.toml"
    loaded = S.load(example)
    assert loaded.tmdb.api_base == ""
    assert loaded.open.wikidata_endpoint == ""
    assert loaded.sign.key_file == ""
    raw = tomllib.loads(example.read_text())
    assert set(raw) == set(S.SECTIONS)
    for section, cls in S.SECTIONS.items():
        assert set(raw[section]) == {f.name for f in fields(cls)}, section


def test_h133_a_vocabulary_file_with_a_name_from_a_capture_is_refused(tmp_path):
    folder = tmp_path / "vocabulary"
    folder.mkdir()
    (folder / "topics.json").write_text(json.dumps({"x": {"name": "Love " + "Hurts"}}))
    assert [f.what for f in scan_paths([tmp_path])] == ["name from a capture"]
    assert not list(scan_own_words({"x": {"name": "Quiet Harbour Stories"}}))


def test_h133_the_list_of_capture_names_leaves_out_plain_words_of_our_own():
    assert not list(scan_own_words(["true crime", "sitcoms", "comedies"]))


def test_h133_the_carried_vocabulary_holds_no_name_from_a_capture():
    folder = Path(__file__).resolve().parents[1] / "src" / "databuild" / "vocabulary"
    assert not list(scan_paths([folder]))
    assert not (folder / "topics.json").exists()
