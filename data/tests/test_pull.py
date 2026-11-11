"""Pull stage, driven by a fake transport."""

import gzip
import json
import sqlite3
import threading

import pytest

from databuild.errors import FeatureOff, StageFailed
from databuild.net import NetError, Reply
from databuild.pull.details import pull_details, pull_refresh
from databuild.pull.exports import read_export
from databuild.pull.fresh import pull_fresh
from databuild.pull.popularity import pull_popularity
from databuild.pull.tmdb import (
    Limiter,
    lost_is_systemic,
    open_pull,
    parse_season,
    parse_title,
    pick_art,
    pick_logo,
)

from .conftest import API, EXPORTS, KEY, NOW, FakeTransport, gz_lines


def movie_doc(i=1, **more):
    doc = {
        "id": i,
        "title": f"Film {i}",
        "original_title": f"Film {i}",
        "release_date": "2020-05-01",
        "runtime": 101,
        "tagline": "A line",
        "overview": "A film about a long journey that goes on for a good while yet.",
        "genres": [{"name": "Drama"}, {"name": "Comedy"}],
        "vote_average": 7.1,
        "vote_count": 300,
        "popularity": 12.5,
        "poster_path": "/p.jpg",
        "backdrop_path": "/b.jpg",
        "original_language": "en",
        "adult": False,
        "status": "Released",
        "imdb_id": "tt1",
        "production_countries": [{"iso_3166_1": "US"}],
        "production_companies": [
            {"name": "One", "logo_path": "/one.png"},
            {"name": "Two", "logo_path": None},
        ],
        "spoken_languages": [{"iso_639_1": "en"}],
        "belongs_to_collection": {"name": "Set"},
        "credits": {
            "cast": [{"name": f"Actor {n}"} for n in range(14)],
            "crew": [
                {"name": "Ann", "job": "Director"},
                {"name": "Bob", "job": "Writer"},
                {"name": "Cy", "job": "Gaffer"},
            ],
        },
        "keywords": {"keywords": [{"name": "journey"}]},
        "release_dates": {
            "results": [
                {
                    "iso_3166_1": "GB",
                    "release_dates": [{"certification": ""}, {"certification": "15"}],
                }
            ]
        },
        "images": {
            "logos": [
                {"iso_639_1": "fr", "file_path": "/fr.png"},
                {"iso_639_1": None, "file_path": "/none.png"},
                {"iso_639_1": "en", "file_path": "/en.png"},
            ]
        },
        "videos": {
            "results": [
                {
                    "site": "video-site",
                    "key": "k1",
                    "type": "Trailer",
                    "name": "Official",
                    "official": True,
                    "size": 1080,
                }
            ]
        },
        "translations": {
            "translations": [
                {
                    "iso_639_1": "fr",
                    "iso_3166_1": "FR",
                    "data": {"title": "Le Film", "overview": "Un film", "tagline": ""},
                },
                {
                    "iso_639_1": "de",
                    "iso_3166_1": "DE",
                    "data": {"title": "Der Film", "overview": "", "tagline": ""},
                },
            ]
        },
    }
    doc.update(more)
    return doc


def tv_doc(i=2):
    return {
        "id": i,
        "name": f"Series {i}",
        "original_name": f"Series {i}",
        "first_air_date": "2019-02-03",
        "episode_run_time": [50, 30, 40],
        "overview": "A series about a long journey that goes on for a good while yet.",
        "genres": [{"name": "Drama"}],
        "vote_average": 6.0,
        "vote_count": 20,
        "popularity": 8.0,
        "poster_path": "/p.jpg",
        "backdrop_path": "/b.jpg",
        "created_by": [{"name": "Dee"}],
        "networks": [
            {"name": "Net", "logo_path": "/net.png"},
            {"name": "Other", "logo_path": None},
        ],
        "origin_country": ["GB"],
        "type": "Scripted",
        "number_of_seasons": 1,
        "number_of_episodes": 2,
        "seasons": [{"season_number": 1}],
        "keywords": {"results": [{"name": "journey"}]},
        "content_ratings": {
            "results": [{"iso_3166_1": "US", "rating": "TV-MA"}, {"iso_3166_1": "GB", "rating": ""}]
        },
        "external_ids": {"imdb_id": "tt2"},
        "credits": {"cast": [{"name": "Eve"}], "crew": []},
        "images": {"logos": []},
    }


SEASON = {
    "season_number": 1,
    "name": "Season 1",
    "overview": "First",
    "air_date": "2019-02-03",
    "poster_path": "/s.jpg",
    "episodes": [
        {"episode_number": 1, "name": "One", "still_path": "/e1.jpg", "runtime": 48},
        {"episode_number": 2, "name": "Two", "still_path": "/e2.jpg", "runtime": 51},
    ],
}


def exports():
    return {
        "/movie_ids_11_05_2026.json.gz": Reply(
            200,
            gz_lines(
                [
                    {"id": 1, "popularity": 12.5},
                    {"id": 5, "popularity": 1.0},
                    {"id": 6, "popularity": 50.0, "adult": True},
                ]
            ),
        ),
        "/tv_series_ids_11_05_2026.json.gz": Reply(200, gz_lines([{"id": 2, "popularity": 8.0}])),
    }


def full_transport():
    return FakeTransport(
        {**exports(), "/movie/1": movie_doc(), "/tv/2": tv_doc(), "/tv/2/season/1": SEASON}
    )


def test_logo_rule_is_english_then_no_text_then_any():
    assert pick_logo(movie_doc()["images"]["logos"]) == "/en.png"
    assert (
        pick_logo([{"iso_639_1": None, "file_path": "/n"}, {"iso_639_1": "fr", "file_path": "/f"}])
        == "/n"
    )
    assert pick_logo([{"iso_639_1": "fr", "file_path": "/f"}]) == "/f"
    assert pick_logo([]) is None


def test_art_is_the_best_poster_with_no_words():
    posters = [
        {"iso_639_1": "en", "file_path": "/words.jpg", "vote_average": 9.0},
        {"iso_639_1": None, "file_path": "/plain-low.jpg", "vote_average": 3.0, "width": 2000},
        {"iso_639_1": None, "file_path": "/plain-high.jpg", "vote_average": 5.2, "width": 1000},
    ]
    assert pick_art(posters) == "/plain-high.jpg"
    assert pick_art([{"iso_639_1": "en", "file_path": "/words.jpg"}]) is None
    assert pick_art([]) is None
    doc = movie_doc()
    doc["images"]["posters"] = posters
    assert parse_title("movie", doc, ("en-US",), 0)["art_path"] == "/plain-high.jpg"


def test_an_old_pull_file_gains_the_art_column(tmp_path):
    old = sqlite3.connect(tmp_path / "pull.db")
    old.execute("CREATE TABLE titles (media_type TEXT, id INTEGER, PRIMARY KEY (media_type, id))")
    old.commit()
    old.close()
    db = open_pull(tmp_path / "pull.db")
    assert "art_path" in {row[1] for row in db.execute("PRAGMA table_info(titles)")}


def test_film_row_holds_the_old_facts_and_the_new_pulls():
    row = parse_title("movie", movie_doc(), ("en-US", "fr-FR"), 100)
    assert row["title"] == "Film 1"
    assert json.loads(row["cast_members"]) == [f"Actor {n}" for n in range(10)]
    assert json.loads(row["directors"]) == ["Ann"]
    assert json.loads(row["certs"]) == {"GB": "15"}
    assert row["collection"] == "Set"
    assert row["logo_path"] == "/en.png"
    film = parse_title("movie", movie_doc(), ["en-US"], 1)
    assert json.loads(film["company_logos"]) == [{"name": "One", "logo_path": "/one.png"}]
    assert json.loads(row["crew"]) == [
        {"name": "Ann", "job": "Director"},
        {"name": "Bob", "job": "Writer"},
    ]
    assert json.loads(row["videos"])[0]["key"] == "k1"
    assert json.loads(row["translations"]) == {
        "fr-FR": {"title": "Le Film", "overview": "Un film", "tagline": ""}
    }
    assert row["fetched_at"] == 100


def test_series_row_takes_the_middle_runtime_and_network_logos():
    row = parse_title("tv", tv_doc(), ("en-US",), 100)
    assert row["runtime"] == 40
    assert json.loads(row["creators"]) == ["Dee"]
    assert json.loads(row["network_logos"]) == [{"name": "Net", "logo_path": "/net.png"}]
    assert json.loads(row["certs"]) == {"US": "TV-MA"}
    assert row["imdb_id"] == "tt2"


def test_episode_rows_hold_stills_and_runtimes():
    season, rows = parse_season(2, SEASON)
    assert season[1:3] == (1, "Season 1")
    assert season[6] == 2
    assert rows[0][6:8] == ("/e1.jpg", 48)


def test_limiter_slows_on_429_and_recovers_to_its_target():
    limiter = Limiter(30.0, clock=lambda: 0.0, sleep=lambda _: None)
    limiter.slow()
    assert limiter.rate == pytest.approx(22.5)
    for _ in range(40):
        limiter.fast()
    assert limiter.rate == 30.0
    for _ in range(20):
        limiter.slow()
    assert limiter.rate == 8.0


def test_pull_skips_low_popularity_and_adult_ids_and_stores_episodes(context, key_file):
    transport = full_transport()
    ctx = context(transport, api_base=API, exports_base=EXPORTS, key_file=key_file)
    pull_details(ctx)
    db = sqlite3.connect(ctx.work / "pull.db")
    assert sorted(db.execute("SELECT media_type, id FROM titles")) == [("movie", 1), ("tv", 2)]
    assert db.execute("SELECT count(*) FROM seasons").fetchone() == (1,)
    assert db.execute(
        "SELECT still_path, runtime FROM episodes ORDER BY episode_number"
    ).fetchall() == [
        ("/e1.jpg", 48),
        ("/e2.jpg", 51),
    ]
    assert not [c for c in transport.calls if c.path.startswith("/movie/5")]


def test_a_second_run_resumes_and_fetches_nothing_new(context, key_file):
    transport = full_transport()
    ctx = context(transport, api_base=API, exports_base=EXPORTS, key_file=key_file)
    pull_details(ctx)
    before = len([c for c in transport.calls if c.path.startswith(("/movie/", "/tv/"))])
    pull_details(ctx)
    after = len([c for c in transport.calls if c.path.startswith(("/movie/", "/tv/"))])
    assert after == before


def test_a_refused_key_stops_the_run_with_no_key_in_the_text(context, key_file):
    routes = {**exports(), "/movie/1": Reply(401), "/tv/2": Reply(401)}
    ctx = context(FakeTransport(routes), api_base=API, exports_base=EXPORTS, key_file=key_file)
    with pytest.raises(StageFailed) as caught:
        pull_details(ctx)
    assert KEY not in str(caught.value)
    assert not any(KEY in line for line in ctx.said)


def test_a_rate_limit_answer_is_retried_after_its_wait(context, key_file):
    routes = {
        **exports(),
        "/movie/1": [Reply(429, headers={"retry-after": "0"}), movie_doc()],
        "/tv/2": tv_doc(),
        "/tv/2/season/1": SEASON,
    }
    ctx = context(FakeTransport(routes), api_base=API, exports_base=EXPORTS, key_file=key_file)
    pull_details(ctx)
    db = sqlite3.connect(ctx.work / "pull.db")
    assert db.execute("SELECT count(*) FROM titles").fetchone() == (2,)


def ends_in_time(call, seconds=20.0):
    """Runs the call on a thread. A stage that never ends shows as a failed test here instead of
    stopping the whole run, which is what a worker that dies without a result used to cause."""
    outcome: list[BaseException | None] = []

    def go() -> None:
        try:
            call()
        except BaseException as e:
            outcome.append(e)
        else:
            outcome.append(None)

    thread = threading.Thread(target=go, daemon=True)
    thread.start()
    thread.join(seconds)
    assert not thread.is_alive(), "the stage never ended"
    if outcome[0] is not None:
        raise outcome[0]


def unreadable_routes(answer):
    return {**exports(), "/movie/1": answer, "/tv/2": tv_doc(), "/tv/2/season/1": SEASON}


@pytest.mark.parametrize(
    "answer",
    [
        pytest.param(Reply(200, b"<html>not json</html>"), id="not json"),
        pytest.param(
            {**movie_doc(), "credits": {"cast": [{"profile_path": "/x.jpg"}], "crew": []}},
            id="a credits entry with no name",
        ),
    ],
)
def test_an_answer_this_build_cannot_read_fails_that_fetch_and_the_pull_ends(
    context, key_file, answer
):
    ctx = context(
        FakeTransport(unreadable_routes(answer)),
        api_base=API,
        exports_base=EXPORTS,
        key_file=key_file,
    )
    ends_in_time(lambda: pull_details(ctx))
    db = sqlite3.connect(ctx.work / "pull.db")
    assert sorted(db.execute("SELECT media_type, id FROM titles")) == [("tv", 2)]
    # no fetch log row for the film, so a later run asks for it again
    assert sorted(db.execute("SELECT media_type, id FROM fetch_log")) == [("tv", 2)]
    assert any("cannot read" in line for line in ctx.said), ctx.said
    assert any("1 failed" in line for line in ctx.said), ctx.said


@pytest.mark.parametrize(
    "answer,why",
    [
        pytest.param(Reply(200, b"<html>not json</html>"), "unreadable", id="a changed shape"),
        pytest.param(Reply(500), "unanswered", id="an upstream that stopped answering"),
    ],
)
def test_a_run_that_stores_nothing_fails_the_build(context, key_file, monkeypatch, answer, why):
    # One lost title counts as a failed fetch and the run goes on (the test above). A run that
    # stored nothing has not pulled a catalogue: it must not return, because pull.db would keep
    # yesterday's rows, the export would rebuild yesterday's catalogue with a fresh built_at_ms,
    # and server-catalog's refuse_older accepts a set whose build time moved forward. A changed
    # answer shape and an upstream that stopped answering leave the same frozen set, so both
    # count against the one ceiling.
    # One attempt per title, so the six real waits between retries do not slow this test down.
    monkeypatch.setattr("databuild.pull.tmdb.ATTEMPTS", 1)
    routes = {**exports(), "/movie/1": answer, "/tv/2": answer}
    ctx = context(
        FakeTransport(routes),
        api_base=API,
        exports_base=EXPORTS,
        key_file=key_file,
    )
    with pytest.raises(StageFailed, match="nothing this build could store") as caught:
        ends_in_time(lambda: pull_details(ctx))
    assert f"2 {why}" in str(caught.value)
    db = sqlite3.connect(ctx.work / "pull.db")
    assert db.execute("SELECT count(*) FROM titles").fetchone() == (0,)
    assert db.execute("SELECT count(*) FROM fetch_log").fetchone() == (0,)


def test_the_lost_ceiling_has_a_floor_so_a_small_run_is_not_flaky():
    # A big run fails at a fifth; a run shorter than the floor has to lose every title.
    assert not lost_is_systemic(0, 0)
    assert not lost_is_systemic(1, 2)
    assert lost_is_systemic(2, 2)
    assert not lost_is_systemic(19, 20)
    assert lost_is_systemic(20, 20)
    assert not lost_is_systemic(25, 1_000)
    assert not lost_is_systemic(199, 1_000)
    assert lost_is_systemic(200, 1_000)


def test_pull_is_off_with_no_address_or_key(context, key_file):
    with pytest.raises(FeatureOff, match=r"tmdb.api_base"):
        pull_details(context(full_transport()))
    with pytest.raises(FeatureOff, match=r"tmdb.key_file"):
        pull_details(context(full_transport(), api_base=API))


def test_six_month_refresh_fetches_old_titles_and_purges_what_is_gone(context, key_file):
    transport = full_transport()
    ctx = context(transport, api_base=API, exports_base=EXPORTS, key_file=key_file)
    pull_details(ctx)
    db = sqlite3.connect(ctx.work / "pull.db")
    old = int(NOW.timestamp()) - 200 * 86400
    db.execute("UPDATE titles SET fetched_at = ?", (old,))
    db.commit()
    transport.routes["/tv/2"] = Reply(404)
    pull_refresh(ctx)
    assert db.execute("SELECT media_type, id, fetched_at FROM titles").fetchall() == [
        ("movie", 1, int(NOW.timestamp()))
    ]
    assert db.execute("SELECT count(*) FROM seasons").fetchone() == (0,)
    assert db.execute("SELECT count(*) FROM episodes").fetchone() == (0,)


def test_refresh_leaves_titles_inside_the_limit(context, key_file):
    transport = full_transport()
    ctx = context(transport, api_base=API, exports_base=EXPORTS, key_file=key_file)
    pull_details(ctx)
    calls = len(transport.calls)
    pull_refresh(ctx)
    assert len(transport.calls) == calls


def test_export_walks_back_five_days_then_fails(context):
    routes = {"/movie_ids_11_03_2026.json.gz": Reply(200, gz_lines([{"id": 1, "popularity": 3.0}]))}
    ctx = context(FakeTransport(routes), exports_base=EXPORTS)
    assert read_export(ctx, "movie").day.isoformat() == "2026-11-03"
    with pytest.raises(StageFailed, match="no tv_series_ids"):
        read_export(ctx, "tv")


def test_export_survives_a_network_error_on_one_day(context):
    routes = {
        "/movie_ids_11_05_2026.json.gz": NetError("x"),
        "/movie_ids_11_04_2026.json.gz": Reply(200, gz_lines([{"id": 1, "popularity": 3.0}])),
    }
    ctx = context(FakeTransport(routes), exports_base=EXPORTS)
    assert read_export(ctx, "movie").day.isoformat() == "2026-11-04"


def popularity_catalogue(ctx):
    db = sqlite3.connect(ctx.out / "catalog.db")
    db.execute("CREATE TABLE titles (media_type TEXT, tmdb_id INTEGER)")
    db.executemany("INSERT INTO titles VALUES (?, ?)", [("movie", 1), ("tv", 2)])
    db.commit()


def test_h62_popularity_writes_the_date_and_the_titles(context):
    ctx = context(FakeTransport(exports()), exports_base=EXPORTS)
    popularity_catalogue(ctx)
    pull_popularity(ctx)
    db = sqlite3.connect(ctx.out / "popularity.db")
    assert db.execute("SELECT v FROM meta WHERE k = 'asof'").fetchone() == ("2026-11-05",)
    assert sorted(db.execute("SELECT media_type, id, popularity FROM pop")) == [
        ("movie", 1, 12.5),
        ("tv", 2, 8.0),
    ]


def test_h62_a_missing_catalogue_fails_the_job(context):
    ctx = context(FakeTransport(exports()), exports_base=EXPORTS)
    with pytest.raises(StageFailed, match="no catalogue"):
        pull_popularity(ctx)


def test_h62_a_missing_export_fails_the_job_and_keeps_the_old_file(context):
    ctx = context(FakeTransport({}), exports_base=EXPORTS)
    popularity_catalogue(ctx)
    (ctx.out / "popularity.db").write_bytes(b"old")
    with pytest.raises(StageFailed):
        pull_popularity(ctx)
    assert (ctx.out / "popularity.db").read_bytes() == b"old"


def fresh_routes():
    page = {
        "total_pages": 1,
        "results": [
            {
                "id": 9,
                "title": "New",
                "genre_ids": [1, 99],
                "release_date": "2026-10-30",
                "popularity": 4.0,
            },
            {"id": 3, "name": "Show", "media_type": "tv", "first_air_date": "2026-11-20"},
            {"id": 4, "media_type": "person"},
        ],
    }
    routes = {
        "/genre/movie/list": {"genres": [{"id": 1, "name": "Drama"}]},
        "/genre/tv/list": {"genres": []},
    }
    for path in (
        "/movie/now_playing",
        "/movie/upcoming",
        "/discover/movie",
    ):
        routes[path] = page
    for path in ("/tv/on_the_air", "/tv/airing_today", "/discover/tv"):
        routes[path] = page
    routes["/trending/all/day"] = {"total_pages": 1, "results": page["results"][1:]}
    routes["/trending/all/week"] = {"total_pages": 1, "results": []}
    return routes


def test_s150_lists_dates_and_ranks(context, key_file):
    transport = FakeTransport(fresh_routes())
    ctx = context(transport, api_base=API, key_file=key_file)
    pull_fresh(ctx, sleep=lambda _: None)
    doc = json.loads((ctx.out / "fresh.json").read_text())
    assert doc["generated"] == "2026-11-05T12:00:00+00:00"
    assert [(e["media_type"], e["id"]) for e in doc["items"]][:2] == [("movie", 3), ("movie", 4)]
    keys = [(e["media_type"], e["id"]) for e in doc["items"]]
    assert keys == sorted(keys)
    film = next(e for e in doc["items"] if e["id"] == 9 and e["media_type"] == "movie")
    assert film["genres"] == ["Drama"]
    assert film["ranks"]["now_playing"] == 0
    dates = [c.params for c in transport.calls if c.path == "/discover/movie"]
    assert dates[0]["primary_release_date.gte"] == "2026-08-07"
    assert dates[0]["primary_release_date.lte"] == "2026-11-05"
    assert dates[1]["primary_release_date.gte"] == "2026-11-06"
    assert dates[1]["primary_release_date.lte"] == "2027-05-04"
    assert all(c.params.get("language", "en-US") == "en-US" for c in transport.calls)


def test_s150_a_series_that_has_aired_carries_its_next_season(context, key_file):
    routes = fresh_routes()
    aired = [
        {"id": 5, "name": "Back Again", "first_air_date": "2020-01-01"},
        {"id": 6, "name": "Weekly", "first_air_date": "2019-01-01"},
    ]
    routes["/tv/on_the_air"] = {"total_pages": 1, "results": aired}
    routes["/tv/5"] = {
        "next_episode_to_air": {"season_number": 3, "episode_number": 1, "air_date": "2026-11-21"}
    }
    routes["/tv/6"] = {
        "next_episode_to_air": {"season_number": 4, "episode_number": 6, "air_date": "2026-11-07"}
    }
    transport = FakeTransport(routes)
    ctx = context(transport, api_base=API, key_file=key_file)
    pull_fresh(ctx, sleep=lambda _: None)
    doc = json.loads((ctx.out / "fresh.json").read_text())
    by_id = {(e["media_type"], e["id"]): e for e in doc["items"]}
    assert by_id[("tv", 5)]["next_season"] == {"number": 3, "air_date": "2026-11-21"}
    assert "next_season" not in by_id[("tv", 6)]
    assert "next_season" not in by_id[("tv", 3)]
    asked = {c.path for c in transport.calls}
    assert "/tv/3" not in asked, "a series that has not aired has no next season to look up"
    airing = [
        c.params for c in transport.calls if c.path == "/discover/tv" and "air_date.gte" in c.params
    ]
    assert airing[0]["air_date.gte"] == "2026-11-06"
    assert airing[0]["air_date.lte"] == "2027-01-04"


def test_s150_pages_follow_the_table(context, key_file):
    routes = fresh_routes()
    routes["/movie/now_playing"] = {"total_pages": 99, "results": []}
    transport = FakeTransport(routes)
    ctx = context(transport, api_base=API, key_file=key_file)
    pull_fresh(ctx, sleep=lambda _: None)
    assert len([c for c in transport.calls if c.path == "/movie/now_playing"]) == 5


def test_s220_four_attempts_with_growing_backoff_then_the_job_fails(context, key_file):
    routes = fresh_routes()
    routes["/movie/now_playing"] = NetError("x")
    naps: list[float] = []
    ctx = context(FakeTransport(routes), api_base=API, key_file=key_file)
    with pytest.raises(StageFailed, match="/movie/now_playing"):
        pull_fresh(ctx, sleep=naps.append)
    assert naps == [1.5, 3.0, 4.5, 6.0]


def test_s151_a_pull_with_no_items_keeps_the_old_file(context, key_file):
    routes = fresh_routes()
    empty = {"total_pages": 1, "results": []}
    for path in list(routes):
        if not path.startswith("/genre"):
            routes[path] = empty
    ctx = context(FakeTransport(routes), api_base=API, key_file=key_file)
    (ctx.out / "fresh.json").write_text("old")
    with pytest.raises(StageFailed, match="no items"):
        pull_fresh(ctx, sleep=lambda _: None)
    assert (ctx.out / "fresh.json").read_text() == "old"


def test_gzip_helper_reads_back():
    assert gzip.decompress(gz_lines([{"id": 1}])) == b'{"id": 1}'


def test_n4_network_and_company_logos_join_the_pull():
    row = parse_title("tv", tv_doc(), ["en-US"], 1)
    assert json.loads(row["networks"]) == ["Net", "Other"]
    assert json.loads(row["network_logos"]) == [{"name": "Net", "logo_path": "/net.png"}]
    film = parse_title("movie", movie_doc(), ["en-US"], 1)
    assert json.loads(film["company_logos"]) == [{"name": "One", "logo_path": "/one.png"}]
