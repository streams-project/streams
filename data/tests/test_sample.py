"""The open sample on a made up Wikidata and TVmaze. No real title and no network."""

import io
import json
import tarfile
from dataclasses import replace

import numpy as np
import pytest
from PIL import Image

from databuild import cli
from databuild.clean.catalog import clean
from databuild.embed.run import embed
from databuild.errors import FeatureOff, StageFailed
from databuild.export.run import export
from databuild.group.run import group
from databuild.manifest.verify import verify_set
from databuild.manifest.write import write_manifest
from databuild.name.run import name
from databuild.sample.cards import BACKDROP, POSTER, Card, render, write_cards
from databuild.sample.run import (
    NEUTRAL_RATING,
    OPEN_TEXT_FILE,
    film_row,
    plain_genres,
    pull_open,
    series_genres,
)
from databuild.sample.tvmaze import TvMaze, parse_show, plain
from databuild.sample.wikidata import (
    BATCH,
    Facts,
    Listed,
    Wikidata,
    describe_film,
    item_of,
    parse_list,
    read_facts,
)
from databuild.scan import scan_paths
from databuild.settings import Open

from .conftest import FakeTransport
from .open_fixtures import ENDPOINT, MAZE, FakeOpen, NetError, Reply, make_show, rows_of, settle


def quiet(_):
    return None


def run_all(ctx):
    pull_open(ctx, quiet)
    ctx.votes = False
    clean(ctx)
    embed(ctx, ctx.work / OPEN_TEXT_FILE, primary=True)
    group(ctx)
    name(ctx)
    export(ctx)
    write_manifest(ctx)
    return ctx


def test_no_setting_turns_the_sample_off_names_it_and_calls_nothing(context):
    ctx = context(FakeTransport())
    with pytest.raises(FeatureOff, match=r"open\.wikidata_endpoint"):
        pull_open(ctx, quiet)
    assert ctx.transport.calls == []


@pytest.mark.parametrize(
    ("given", "missing"),
    [
        ({"wikidata_endpoint": ENDPOINT}, r"open\.tvmaze_api_base"),
        ({"wikidata_endpoint": ENDPOINT, "tvmaze_api_base": MAZE}, r"open\.user_agent"),
    ],
)
def test_each_setting_of_the_sample_is_named_when_missing(context, given, missing):
    ctx = context(FakeTransport())
    ctx.settings = replace(ctx.settings, open=Open(**given))
    with pytest.raises(FeatureOff, match=missing):
        pull_open(ctx, quiet)
    assert ctx.transport.calls == []


def test_the_pull_reads_no_key(context):
    ctx = settle(context(), FakeOpen())
    assert ctx.settings.tmdb.key_file == ""
    assert ctx.settings.sign.key_file == ""
    pull_open(ctx, quiet)
    for call in ctx.transport.calls:
        assert "api_key" not in call.params
        assert "Authorization" not in call.headers


def test_every_call_names_the_builder(context):
    ctx = settle(context(), FakeOpen())
    pull_open(ctx, quiet)
    assert ctx.transport.calls
    assert all(c.headers["User-Agent"] == "databuild tests" for c in ctx.transport.calls)


def test_sixty_percent_films_and_forty_percent_series(context):
    ctx = settle(context(), FakeOpen(), titles=50)
    pull_open(ctx, quiet)
    rows = dict(rows_of(ctx.work / "pull.db", "SELECT media_type, count(*) FROM titles GROUP BY 1"))
    assert rows == {"movie": 30, "tv": 20}


def test_about_1000_titles_is_the_default_size():
    assert Open().titles == 1000


def test_titles_go_by_how_widely_they_are_covered(context):
    ctx = settle(context(), FakeOpen(), titles=10)
    pull_open(ctx, quiet)
    held = rows_of(
        ctx.work / "pull.db", "SELECT id, popularity FROM titles WHERE media_type='movie'"
    )
    assert sorted(held) == [
        (101, 199.0),
        (102, 198.0),
        (103, 197.0),
        (104, 196.0),
        (105, 195.0),
        (106, 194.0),
    ]


def test_a_title_keeps_its_tmdb_id_as_its_key(context):
    ctx = settle(context(), FakeOpen(), titles=10)
    pull_open(ctx, quiet)
    lines = (ctx.work / OPEN_TEXT_FILE).read_text().splitlines()
    keys = {json.loads(line)["key"] for line in lines}
    assert "movie:101" in keys
    assert "tv:301" in keys


def test_open_text_leaves_the_title_out(context):
    ctx = settle(context(), FakeOpen(), titles=10)
    pull_open(ctx, quiet)
    text = (ctx.work / OPEN_TEXT_FILE).read_text()
    assert "Made Up Film" not in text
    assert "Made Up Show" not in text
    assert "Genres: " in text


def test_ids_that_are_not_numbers_and_repeated_keys_are_left_out():
    rows = [
        {"item": "https://x.test/Q1", "id": "12", "links": "9"},
        {"item": "https://x.test/Q2", "id": "12", "links": "8"},
        {"item": "https://x.test/Q3", "id": "tv/12", "links": "7"},
        {"item": "https://x.test/Z4", "id": "13", "links": "7"},
    ]
    assert parse_list(rows) == [Listed("Q1", 12, 9)]


def test_item_numbers_are_read_from_the_end_of_an_address():
    assert item_of("https://x.test/entity/Q42") == "Q42"


def test_facts_arrive_in_batches_of_one_property_each():
    fake = FakeOpen(films=BATCH + 1, series=0)
    client = Wikidata(ENDPOINT, "ua", fake.transport(), quiet)
    items = [f"Q{1000 + i}" for i in range(1, BATCH + 2)]
    facts = read_facts(client, items)
    assert len(fake.queries) == 2 * 10
    assert max(q.count("wd:Q") for q in fake.queries) == BATCH
    assert facts["Q1001"].label == "Made Up Film 1"
    assert facts["Q1001"].all("genres") == ["comedy film"]
    assert facts["Q1001"].date == "1971-03-04"
    assert facts["Q1001"].runtime == 81


def test_a_runtime_outside_any_real_length_is_left_out():
    facts = Facts()
    facts.values["runtimes"] = ["2", "9000"]
    assert facts.runtime is None


def test_a_film_sentence_comes_from_facts_and_leaves_the_title_out():
    facts = Facts(label="Hidden Name")
    facts.values["dates"] = ["1984-01-01T00:00:00Z"]
    facts.values["directors"] = ["Person One"]
    facts.values["cast"] = ["Person Two"]
    facts.values["subjects"] = ["ghost"]
    text = describe_film(facts, ["Horror"])
    assert text == "A 1984 horror film directed by Person One, with Person Two. It is about ghost."


def test_a_film_with_too_little_text_is_left_out():
    assert film_row(Listed("Q1", 1, 5), Facts(label="Thin"), 0) is None


def test_an_adult_film_is_left_out(context):
    fake = FakeOpen()
    fake.adult = {"Q1001", "Q5001"}
    ctx = settle(context(), fake, titles=20)
    pull_open(ctx, quiet)
    held = {r[0] for r in rows_of(ctx.work / "pull.db", "SELECT title FROM titles")}
    assert "Made Up Film 1" not in held
    assert "Made Up Series 1" not in held


def test_a_series_with_no_show_id_is_found_by_imdb_id(context):
    fake = FakeOpen()
    fake.maze_less = {"Q5002"}
    ctx = settle(context(), fake, titles=20)
    pull_open(ctx, quiet)
    assert any(
        c.path == "/lookup/shows" and c.params == {"imdb": "tt9002"} for c in ctx.transport.calls
    )
    assert rows_of(ctx.work / "pull.db", "SELECT count(*) FROM titles WHERE id = 302") == [(1,)]


def test_a_series_the_service_does_not_hold_is_passed_over(context):
    fake = FakeOpen()
    ctx = settle(context(), fake, titles=10)
    del ctx.transport.routes["/shows/701"]
    pull_open(ctx, quiet)
    assert rows_of(ctx.work / "pull.db", "SELECT count(*) FROM titles WHERE id = 301") == [(0,)]
    assert rows_of(ctx.work / "pull.db", "SELECT count(*) FROM titles WHERE media_type = 'tv'") == [
        (4,)
    ]


def test_no_title_with_text_fails_the_stage(context):
    fake = FakeOpen(films=2, series=0)
    fake.short = {"Q1001", "Q1002"}
    ctx = settle(context(), fake, titles=2)
    with pytest.raises(StageFailed, match="enough text"):
        pull_open(ctx, quiet)


def test_markup_leaves_a_summary():
    assert plain("<p>One <b>two</b> &amp; three.</p>\n") == "One two & three."
    assert plain(None) == ""


def test_a_show_gives_rows_shaped_like_the_pull_file():
    show = parse_show(make_show(1), 301)
    assert show.language == "en"
    assert show.networks == ["Channel 1"]
    assert show.countries == ["GB"]
    assert show.seasons[0] == (301, 1, "Season 1", "", "1991-02-03", "", 3)
    assert show.episodes[0][:4] == (301, 1, 1, "Part 1")
    assert len(show.episodes) == 6


def test_an_episode_with_no_number_is_left_out():
    document = make_show(1)
    document["_embedded"]["episodes"].append({"season": 1, "number": None, "name": "Special"})
    assert len(parse_show(document, 301).episodes) == 6


def test_tvmaze_keeps_under_its_limit():
    waits = []
    clock = [0.0]

    def sleep(seconds):
        waits.append(round(seconds, 3))
        clock[0] += seconds

    transport = FakeTransport({"/shows/1": {"id": 1}})
    maze = TvMaze(MAZE, "ua", transport, 2.0, sleep, lambda: clock[0])
    for _ in range(3):
        maze.show(1)
    assert waits == [0.5, 0.5]


def test_tvmaze_waits_when_told_and_then_goes_on():
    waits = []
    transport = FakeTransport(
        {"/shows/1": [Reply(429, headers={"retry-after": "7"}), {"id": 1}]},
    )
    maze = TvMaze(MAZE, "ua", transport, 1000.0, waits.append)
    assert maze.show(1) == {"id": 1}
    assert 7.0 in waits


def test_a_refused_call_stops_the_pull_with_a_plain_word():
    transport = FakeTransport({"/shows/1": Reply(403)})
    with pytest.raises(StageFailed, match="answered 403"):
        TvMaze(MAZE, "ua", transport, 1000.0, quiet).show(1)


def test_a_service_that_stays_down_gives_nothing_for_that_show():
    transport = FakeTransport({"/shows/1": Reply(503)})
    assert TvMaze(MAZE, "ua", transport, 1000.0, quiet).show(1) is None


def test_wikidata_retries_then_fails_plainly_with_no_address():
    client = Wikidata(ENDPOINT, "ua", FakeTransport({"": NetError("x")}), quiet)
    with pytest.raises(StageFailed) as caught:
        client.select("SELECT")
    assert "fake.test" not in str(caught.value)
    client = Wikidata(ENDPOINT, "ua", FakeTransport({"": Reply(400)}), quiet)
    with pytest.raises(StageFailed, match="answered 400"):
        client.select("SELECT")


def test_wikidata_waits_when_told():
    waits = []
    good = {"results": {"bindings": []}}
    transport = FakeTransport({"": [Reply(429, headers={"retry-after": "3"}), Reply(500), good]})
    assert Wikidata(ENDPOINT, "ua", transport, waits.append).select("SELECT") == []
    assert waits[0] == 3.0


def test_genres_read_labels_by_whole_words():
    assert plain_genres(["science fiction film", "warrior film"]) == ["Science Fiction"]
    assert plain_genres(["war film", "Science-Fiction", "animated film"]) == [
        "Animation",
        "Science Fiction",
        "War",
    ]
    names = {"Science Fiction": "Sci-Fi & Fantasy"}
    assert series_genres(["Science-Fiction"], names) == ["Sci-Fi & Fantasy"]


def test_a_card_is_a_png_of_its_size_and_repeats_exactly():
    card = Card("movie", 12, "A Made Up Title That Runs Over Several Lines Of A Card", "1984")
    one = render(card, POSTER)
    assert one == render(card, POSTER)
    assert Image.open(io.BytesIO(one)).size == POSTER
    other = render(Card("tv", 12, "Other", ""), BACKDROP)
    assert Image.open(io.BytesIO(other)).size == BACKDROP
    assert one != other


def test_the_card_file_repeats_exactly_and_has_two_images_per_title(tmp_path):
    cards = [Card("tv", 2, "B", "2001"), Card("movie", 1, "A", "2000")]
    write_cards(tmp_path / "a.tar", cards)
    write_cards(tmp_path / "b.tar", list(reversed(cards)))
    assert (tmp_path / "a.tar").read_bytes() == (tmp_path / "b.tar").read_bytes()
    with tarfile.open(tmp_path / "a.tar") as tar:
        members = tar.getmembers()
    assert [m.name for m in members] == [
        "poster/movie-1.png",
        "backdrop/movie-1.png",
        "poster/tv-2.png",
        "backdrop/tv-2.png",
    ]
    assert all(m.mtime == 0 for m in members)


def test_series_text_sits_in_its_own_folder_with_credit(context):
    ctx = settle(context(), FakeOpen(), titles=20)
    pull_open(ctx, quiet)
    lines = (ctx.out / "tvmaze" / "series.jsonl").read_text().splitlines()
    assert len(lines) == 8
    first = json.loads(lines[0])
    assert first["key"].startswith("tv:")
    assert first["url"]
    assert "<p>" not in first["summary"]
    credit = (ctx.out / "tvmaze" / "CREDIT.md").read_text()
    assert "CC BY-SA 4.0" in credit
    assert "TVmaze" in credit
    assert "CC0" in (ctx.out / "SOURCES.md").read_text()


def test_the_sample_builds_a_set_that_passes_every_check(context):
    ctx = run_all(settle(context(), FakeOpen()))
    done = verify_set(ctx.out)
    manifest = done.manifest
    assert manifest["source"] == "open"
    assert manifest["recipe"] == "open-text-v1"
    assert manifest["model"] == "hashing-v1"
    assert manifest["dataset_version"].startswith("open-2026-11-05-")
    assert {"cards", "series_text", "series_credit", "sources"} <= set(manifest["files"])
    assert manifest["row_count"] > 80


def test_no_text_or_image_in_the_sample_comes_from_tmdb(context):
    ctx = run_all(settle(context(), FakeOpen()))
    sources = {r[0] for r in rows_of(ctx.out / "catalog.db", "SELECT DISTINCT source FROM titles")}
    assert sources == {"tvmaze", "wikidata"}
    paths = rows_of(ctx.out / "catalog.db", "SELECT poster_path, backdrop_path FROM titles")
    assert all(p.startswith("/poster/") and b.startswith("/backdrop/") for p, b in paths)


def test_sample_titles_pass_the_page_gates_with_the_neutral_rating(context):
    ctx = run_all(settle(context(), FakeOpen()))
    rows = rows_of(
        ctx.out / "catalog.db", "SELECT vote_average, vote_count, popularity FROM catalog"
    )
    assert rows
    assert all(r[0] == NEUTRAL_RATING and r[1] == 0 and r[2] >= 2 for r in rows)


def test_open_rule_keeps_only_the_base_clause_so_titles_are_shown_with_no_votes(context):
    ctx = run_all(settle(context(), FakeOpen()))
    assert rows_of(ctx.out / "catalog.db", "SELECT count(*) FROM catalog")[0][0] > 80


def test_seeds_of_an_open_set_are_ordered_by_coverage(context):
    ctx = run_all(settle(context(), FakeOpen()))
    from databuild.export.vectors import read_seeds

    _, seeds = read_seeds((ctx.out / "seeds.bin").read_bytes())
    assert seeds
    assert sum(len(m) for _, m in seeds) == len(np.fromfile(ctx.out / "vectors.keys", dtype="<u4"))


def test_series_keep_their_seasons_and_episodes(context):
    ctx = run_all(settle(context(), FakeOpen()))
    assert rows_of(ctx.out / "catalog.db", "SELECT count(*) FROM seasons")[0][0] == 40 * 2
    assert rows_of(ctx.out / "catalog.db", "SELECT count(*) FROM episodes")[0][0] == 40 * 6


def test_a_set_built_twice_holds_the_same_files(context, tmp_path):
    first = run_all(settle(context(), FakeOpen()))
    manifest = json.loads((first.out / "manifest.json").read_text())
    hashes = {role: entry["sha256"] for role, entry in manifest["files"].items()}
    run_all(first)
    again = json.loads((first.out / "manifest.json").read_text())
    assert {role: e["sha256"] for role, e in again["files"].items()} == hashes


def test_h133_a_built_sample_holds_no_refused_word_or_field(context):
    ctx = run_all(settle(context(), FakeOpen()))
    found = [f for f in scan_paths([ctx.out]) if "address" not in f.what]
    assert found == []


def test_h132_a_stranger_builds_the_sample_through_the_one_entry_point(
    tmp_path, monkeypatch, capsys
):
    fake = FakeOpen()
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(cli, "UrlTransport", fake.transport)
    settings = tmp_path / "s.toml"
    settings.write_text(
        f"""
[embed]
dimensions = 64
[group]
method = "kmeans"
[open]
wikidata_endpoint = "{ENDPOINT}"
tvmaze_api_base = "{MAZE}"
tvmaze_rps = 100000.0
user_agent = "databuild tests"
titles = 100
"""
    )
    code = cli.main(["--settings", str(settings), "--as-of", "2026-11-05", "sample"])
    printed = capsys.readouterr().out
    assert code == 0, printed
    assert "unsigned" in printed
    assert "signature not checked" in printed
    assert "verify: open-2026-11-05" in printed
    assert verify_set(tmp_path / "build" / "out").manifest["source"] == "open"


def test_the_entry_point_runs_every_stage_from_a_pull_file(tmp_path, monkeypatch, capsys):
    import sqlite3

    fake = FakeOpen()
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(cli, "UrlTransport", fake.transport)
    settings = tmp_path / "s.toml"
    settings.write_text(
        f"""
[embed]
dimensions = 64
[group]
method = "kmeans"
[open]
wikidata_endpoint = "{ENDPOINT}"
tvmaze_api_base = "{MAZE}"
tvmaze_rps = 100000.0
user_agent = "databuild tests"
titles = 100
"""
    )
    base = ["--settings", str(settings), "--as-of", "2026-11-05"]
    assert cli.main([*base, "open-pull"]) == 0
    db = sqlite3.connect(tmp_path / "build" / "work" / "pull.db")
    db.execute("UPDATE titles SET vote_average = 7.5, vote_count = 500")
    db.commit()
    db.close()
    for leftover in ("tvmaze/series.jsonl", "tvmaze/CREDIT.md", "SOURCES.md", "cards.tar"):
        (tmp_path / "build" / "out" / leftover).unlink()
    assert cli.main([*base, "all"]) == 0, capsys.readouterr().err
    done = verify_set(tmp_path / "build" / "out")
    assert done.manifest["source"] == "full"
    assert done.manifest["recipe"] == "genres-tagline-overview-people-v1"
