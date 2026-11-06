"""Clean stage and the catalogue rule."""

import json
import sqlite3
from datetime import date

import pytest

from databuild.clean.catalog import clean, insert_titles, is_anime, open_titles, other_names
from databuild.clean.rule import make_catalog_view, rule_sql
from databuild.errors import StageFailed
from databuild.keys import TV_BIT

from .conftest import API, EXPORTS, NOW
from .test_pull import full_transport


def shown(rows, as_of=date(2026, 11, 5), votes=True):
    db = sqlite3.connect(":memory:")
    db.execute(
        "CREATE TABLE titles (id INTEGER, tmdb_id INTEGER, media_type TEXT, release_year TEXT,"
        " vote_average REAL, vote_count INTEGER, adult INTEGER, poster_path TEXT, overview TEXT)"
    )
    for i, (kind, year, rating, votes_held) in enumerate(rows):
        db.execute(
            "INSERT INTO titles VALUES (?,?,?,?,?,?,0,'/p','" + "x" * 60 + "')",
            (i, i, kind, year, rating, votes_held),
        )
    make_catalog_view(db, as_of, votes)
    return sorted(r[0] for r in db.execute("SELECT id FROM catalog"))


@pytest.mark.parametrize(
    ("row", "kept"),
    [
        (("movie", "2025", 0.0, 0), True),
        (("movie", "2024", 0.0, 0), False),
        (("movie", "2000", 6.5, 10), True),
        (("movie", "2000", 6.4, 10), False),
        (("movie", "2000", 6.5, 9), False),
        (("movie", "2000", 5.0, 100), True),
        (("movie", "2000", 5.0, 99), False),
        (("tv", "2025", 0.0, 0), True),
        (("tv", "2024", 0.0, 0), False),
        (("tv", "2000", 5.0, 10), True),
        (("tv", "2000", 4.9, 10), False),
        (("tv", "2000", 5.0, 9), False),
        (("tv", "2000", 9.0, 5000), True),
    ],
)
def test_s133_thresholds(row, kept):
    assert shown([row]) == ([0] if kept else [])


@pytest.mark.parametrize(
    "fields",
    [
        ("1", "/p", "x" * 60),
        ("0", "", "x" * 60),
        ("0", None, "x" * 60),
        ("0", "/p", "x" * 59),
        ("0", "/p", None),
    ],
)
def test_s133_adult_poster_and_overview_are_refused_first(fields):
    db = sqlite3.connect(":memory:")
    db.execute(
        "CREATE TABLE titles (id INTEGER, tmdb_id INTEGER, media_type TEXT, release_year TEXT,"
        " vote_average REAL, vote_count INTEGER, adult INTEGER, poster_path TEXT, overview TEXT)"
    )
    db.execute("INSERT INTO titles VALUES (1,1,'movie','2026',9.0,5000,?,?,?)", fields)
    make_catalog_view(db, date(2026, 11, 5))
    assert db.execute("SELECT count(*) FROM catalog").fetchone() == (0,)


def test_s133_the_clock_arrives_as_input_so_a_built_view_never_moves():
    row = [("movie", "2025", 0.0, 0)]
    assert shown(row, date(2026, 12, 31)) == [0]
    assert shown(row, date(2027, 1, 1)) == []
    assert "strftime" not in rule_sql(date(2026, 1, 1))
    assert "now" not in rule_sql(date(2026, 1, 1))


def test_s133_the_list_clause_is_gone():
    assert "SELECT" not in rule_sql(date(2026, 1, 1))


def test_open_rule_keeps_only_the_base_clause():
    assert shown([("movie", "1950", 0.0, 0)], votes=False) == [0]


def test_anime_rule():
    assert is_anime("Animation, Drama", ["JP"], "en", [])
    assert is_anime("Animation", [], "ja", [])
    assert is_anime("Animation", [], "en", ["Anime"])
    assert not is_anime("Drama", ["JP"], "ja", ["anime"])
    assert not is_anime("Animation", ["US"], "en", [])


def pulled(context, key_file):
    from databuild.pull.details import pull_details

    ctx = context(full_transport(), api_base=API, exports_base=EXPORTS, key_file=key_file)
    pull_details(ctx)
    return ctx


def test_clean_turns_pulled_rows_into_titles_with_packed_keys(context, key_file):
    ctx = pulled(context, key_file)
    clean(ctx)
    db = sqlite3.connect(ctx.work / "titles.db")
    rows = db.execute(
        "SELECT id, tmdb_id, media_type, release_year, director, cast_members, logo_path, source"
        " FROM titles ORDER BY media_type"
    ).fetchall()
    assert rows[0][:5] == (1, 1, "movie", "2020", "Ann")
    assert rows[0][5] == ", ".join(f"Actor {n}" for n in range(8))
    assert rows[0][6:] == ("/en.png", "tmdb")
    assert rows[1][:5] == (2 | TV_BIT, 2, "tv", "2019", "Dee")
    assert db.execute("SELECT count(*) FROM episodes").fetchone() == (2,)


def test_clean_leaves_out_rows_past_the_refresh_limit(context, key_file):
    ctx = pulled(context, key_file)
    db = sqlite3.connect(ctx.work / "pull.db")
    db.execute(
        "UPDATE titles SET fetched_at = ? WHERE media_type = 'tv'",
        (int(NOW.timestamp()) - 190 * 86400,),
    )
    db.commit()
    clean(ctx)
    out = sqlite3.connect(ctx.work / "titles.db")
    assert out.execute("SELECT media_type FROM titles").fetchall() == [("movie",)]
    assert out.execute("SELECT count(*) FROM episodes").fetchone() == (0,)
    assert any("1 older than the refresh limit" in line for line in ctx.said)


def test_clean_with_no_pull_says_what_to_do(context):
    with pytest.raises(StageFailed, match="no pull file"):
        clean(context())


def test_an_empty_titles_file_is_new_each_time(tmp_path):
    path = tmp_path / "t.db"
    db = open_titles(path)
    insert_titles(db, [{"id": 1, "tmdb_id": 1, "media_type": "movie"}])
    db.commit()
    db.close()
    db = open_titles(path)
    assert db.execute("SELECT count(*) FROM titles").fetchone() == (0,)


def test_the_view_gives_the_public_id_beside_the_packed_key(tmp_path):
    db = sqlite3.connect(tmp_path / "t.db")
    db.execute(
        "CREATE TABLE titles (id INTEGER, tmdb_id INTEGER, media_type TEXT, release_year TEXT,"
        " vote_average REAL, vote_count INTEGER, adult INTEGER, poster_path TEXT, overview TEXT)"
    )
    db.execute(
        "INSERT INTO titles VALUES (?,12,'tv','2000',9.0,500,0,'/p',?)", (12 | TV_BIT, "x" * 60)
    )
    make_catalog_view(db, date(2026, 11, 5))
    assert db.execute("SELECT id, title_id FROM catalog").fetchall() == [(12 | TV_BIT, 12)]


def test_other_names_hold_the_original_and_translated_names_once():
    translated = json.dumps({"fr-FR": {"title": "Le Port"}, "de-DE": {"title": "Harbour"}})
    assert json.loads(other_names("Harbour", "Hafen", translated)) == ["Hafen", "Le Port"]
    assert json.loads(other_names("Harbour", None, "not json")) == []
