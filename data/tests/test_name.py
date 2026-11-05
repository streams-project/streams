"""Name stage on small generated data. A fake text model answers; nothing reaches a network."""

import json
import sqlite3
import struct
from dataclasses import replace

import numpy as np
import pytest

from databuild.embed.quant import unit
from databuild.embed.run import embed
from databuild.errors import FeatureOff, StageFailed
from databuild.group.run import group
from databuild.name.events import build_events
from databuild.name.groups import (
    clean_name,
    decade_label,
    fallback_name,
    fix_name,
    qualify_shared,
    settle_vibe,
)
from databuild.name.llm import HttpChat, make_chat
from databuild.name.profiles import PROFILES_FILE, parse_profile, write_profiles
from databuild.name.rows import Builder, build_rows, passes, repeats, slug
from databuild.name.run import name
from databuild.name.taxonomy import calendar_entries, kids_entries
from databuild.name.topics import category_rule, own_topics, topic_rule, write_topics
from databuild.net import NetError, Reply
from databuild.scan import scan_json_fields, scan_text
from databuild.settings import Embed, Group
from databuild.settings import Name as NameSettings
from databuild.vocab import load

from .conftest import FakeTransport
from .fixtures import make_titles


class FakeChat:
    def __init__(self, answer="Quiet Harbour Stories"):
        self.answer = answer
        self.calls = []

    def ask(self, system, user, max_tokens=16):
        self.calls.append((system, user, max_tokens))
        return self.answer(user) if callable(self.answer) else self.answer


def built(ctx, count=240):
    """Titles, vectors and groups, ready for the name stage."""
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=64), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", count)
    embed(ctx)
    group(ctx)
    return ctx


def stats(genres=(), decades=(), countries=(), examples=()):
    return {
        "genres": list(genres),
        "decades": list(decades),
        "countries": list(countries),
        "examples": list(examples),
    }


def test_a_written_name_keeps_one_line_of_at_most_six_words():
    assert clean_name('Title: "Quiet Harbour Stories."\nmore') == "Quiet Harbour Stories"
    assert clean_name("one two three four five six seven") == "one two three four five six"


def test_a_place_the_model_cut_short_is_repaired():
    assert fix_name("Hong Action") == "Hong Kong Action"
    assert fix_name("Hong Kong Action") == "Hong Kong Action"


def test_a_name_without_a_model_comes_from_the_facts():
    assert decade_label(1980) == "80s"
    assert decade_label(2010) == "2010s"
    facts = stats([("Horror", 0.5, 2.0)], [(1980, 40)])
    assert fallback_name(facts, 50) == "80s Horror"
    facts = stats([("Horror", 0.5, 2.0), ("Comedy", 0.3, 1.5)], [(1980, 10)])
    assert fallback_name(facts, 50) == "Horror & Comedy"
    assert fallback_name(stats([("Horror", 0.5, 2.0)]), 50) == "Horror Picks"
    assert fallback_name(stats(examples=["Made Up (1999)"]), 50) == "Like Made Up"


def test_groups_sharing_a_name_take_a_country_only_they_hold():
    names = {"world|2|1": "Dark Crime", "world|2|2": "Dark Crime"}
    facts = {
        "world|2|1": (stats(countries=[("KR", 0.6, 3.0)]), 10),
        "world|2|2": (stats(countries=[("FR", 0.5, 3.0)]), 10),
    }
    assert qualify_shared(names, facts) == 2
    assert names == {"world|2|1": "Korean Dark Crime", "world|2|2": "French Dark Crime"}


def test_a_shared_vibe_name_stays_with_the_largest_group_and_the_rest_qualify_or_go():
    names = {"vibe|1|1": "Slow Burn", "vibe|1|2": "Slow Burn", "vibe|1|3": "Slow Burn"}
    facts = {
        "vibe|1|1": (stats(), 30),
        "vibe|1|2": (stats(decades=[(1990, 8)]), 10),
        "vibe|1|3": (stats(), 9),
    }
    assert settle_vibe(names, facts) == (1, 1)
    assert names == {"vibe|1|1": "Slow Burn", "vibe|1|2": "1990s Slow Burn"}


def test_stage_without_a_model_names_groups_from_data_and_says_so(context):
    ctx = built(context())
    name(ctx)
    db = sqlite3.connect(ctx.work / "groups.db")
    named = db.execute("SELECT count(*) FROM groups WHERE kind='world' AND name <> ''").fetchone()
    assert named[0] > 0
    assert any("no text model" in line for line in ctx.said)
    assert any("profiles off" in line for line in ctx.said)
    assert any("topics off" in line for line in ctx.said)


def test_a_group_name_stays_while_its_measured_facts_stay(context, monkeypatch):
    from databuild.name import groups as module

    ctx = built(context())
    chat = FakeChat()
    module.name_groups(ctx, chat)
    first = len(chat.calls)
    assert first > 0
    module.name_groups(ctx, chat)
    assert len(chat.calls) == first


def test_two_groups_of_one_level_in_a_season_never_share_a_name(context):
    from databuild.name.groups import name_seasons

    ctx = built(context())
    db = sqlite3.connect(ctx.work / "groups.db")
    db.execute("DELETE FROM season_members")
    rows = [("halloween", sub, "movie", i, i, 1) for sub in (0, 1) for i in range(1, 13) if i % 4]
    db.executemany("INSERT INTO season_members VALUES (?,?,?,?,?,?)", rows)
    db.commit()
    db.close()
    name_seasons(ctx, FakeChat("Spooky Nights"))
    db = sqlite3.connect(ctx.work / "groups.db")
    assert db.execute("SELECT count(*) FROM season_groups").fetchone() == (1,)


def test_s107_events_hold_label_window_keywords_genres_and_languages():
    events = build_events()
    assert events
    for event in events.values():
        assert event["label"]
        assert set(event["season"]) == {"start", "end"}
        assert event["keywords"] == sorted(event["keywords"])


def test_rule_checks_each_fact_it_names():
    title = {
        "genres": {"Drama"},
        "keywords": {"heist"},
        "year": 1994,
        "language": "fr",
        "countries": {"FR"},
        "anime": False,
    }
    assert passes(title, {"genres_any": ["Drama"], "year_min": 1990, "year_max": 1999})
    assert not passes(title, {"genres_any": ["Comedy"]})
    assert not passes(title, {"genres_none": ["Drama"]})
    assert not passes(title, {"keywords_any": ["wedding"]})
    assert not passes(title, {"year_min": 2000})
    assert not passes(title, {"languages_any": ["en"]})
    assert not passes(title, {"countries_any": ["US"]})
    assert not passes(title, {"anime": True})
    assert not passes({**title, "year": None}, {"year_max": 1999})


def test_a_noun_already_in_the_phrase_is_not_repeated():
    assert repeats("Crime Stories", "Stories")
    assert repeats("Family Comedies", "Comedy")
    assert not repeats("Heist", "Films")
    assert slug("Sci-Fi & Fantasy") == "sci_fi_fantasy"


GROUPS = (
    ("Horror", "ghost", 1985),
    ("Comedy", "wedding", 2015),
    ("Crime", "heist", 1995),
    ("Romance", "first love", 2005),
    ("Drama", "family relationships", 1975),
    ("Science Fiction", "robot", 2010),
)


def _pool(n=60, dim=32):
    """Six tight groups of titles, every second title a series."""
    rng = np.random.default_rng(1)
    centres = rng.normal(size=(len(GROUPS), dim))
    rows = [centres[i % 6] + rng.normal(scale=0.05, size=dim) for i in range(n)]
    vectors = unit(np.stack(rows).astype(np.float32))
    keys = np.arange(1, n + 1, dtype="<u4")
    titles = {}
    for i, k in enumerate(keys):
        genre, word, year = GROUPS[i % 6]
        titles[int(k)] = {
            "kind": "movie" if i % 12 < 6 else "tv",
            "genres": {genre},
            "keywords": {word},
            "year": year,
            "language": "en",
            "countries": {"US"},
            "anime": False,
            "quality": True,
        }
    return titles, keys, vectors


def test_a_row_needs_enough_quality_titles():
    titles, keys, vectors = _pool()
    builder = Builder(titles, keys, vectors, min_support=11, top=5)
    assert builder.measure({"genres_any": ["Horror"]}) is None
    builder = Builder(titles, keys, vectors, min_support=10, top=5)
    facts = builder.measure({"genres_any": ["Horror"]})
    assert facts["support"] == 10
    assert facts["movie_share"] == 0.5
    assert facts["genre_share"] == {"Horror": 1.0}


def test_row_file_is_the_shape_the_row_reader_takes():
    titles, keys, vectors = _pool(240)
    document = build_rows(titles, keys, vectors, {}, min_support=10, top=10)
    assert document["version"] == 3
    assert document["requests"]
    for row in document["requests"]:
        assert {"id", "name", "kind", "rules", "page", "format"} <= set(row)
        assert row["page"] in ("films", "series", "home")
        assert {"films": "movie", "series": "tv", "home": "both"}[row["page"]] == row["format"]
    names = [r["name"].lower() for r in document["requests"]]
    assert len(names) == len(set(names))


def test_h133_no_row_holds_a_capture_field_or_a_refused_word(context):
    ctx = built(context())
    name(ctx)
    text = (ctx.work / "export" / "rows.json").read_text()
    document = json.loads(text)
    assert document["requests"]
    assert not list(scan_json_fields(document))
    assert not list(scan_text(text))


def test_a_catalogue_with_no_row_support_fails_the_stage():
    titles, keys, vectors = _pool(8)
    with pytest.raises(StageFailed, match="no row definition"):
        build_rows(titles, keys, vectors, {}, min_support=30, top=10)


def test_calendar_and_kids_entries_keep_their_query_text():
    assert calendar_entries()
    assert all(e["kind"] == "calendar" for e in calendar_entries())
    kids = kids_entries()
    assert kids
    assert all(e["vibe_query"] and e["type"] == e["kind"] for e in kids)


def test_vocabulary_file_is_a_count_then_unit_float32_rows_and_names_its_model(context):
    ctx = built(context())
    name(ctx)
    raw = (ctx.work / "export" / "taxonomy.bin").read_bytes()
    count = struct.unpack("<I", raw[:4])[0]
    document = json.loads((ctx.work / "export" / "taxonomy.json").read_text())
    assert count == len(document["entries"])
    assert document["model"] == "hashing-v1"
    assert document["dimensions"] == 64
    rows = np.frombuffer(raw[4:], dtype="<f4").reshape(count, 64)
    assert np.allclose(np.linalg.norm(rows, axis=1), 1.0, atol=1e-3)


def test_a_profile_answer_is_read_or_left_empty():
    assert parse_profile('Sure: {"summary": "x"} done') == {"summary": "x"}
    assert parse_profile("no json here") == {}
    assert parse_profile("[1, 2]") == {}


def test_profiles_resume_and_skip_titles_already_written(context):
    ctx = built(context(), 60)
    chat = FakeChat('{"summary": "a quiet story", "audience": "family"}')
    write_profiles(ctx, chat)
    lines = (ctx.work / PROFILES_FILE).read_text().splitlines()
    assert lines
    first = json.loads(lines[0])
    assert first["k"].split(":")[0] in ("movie", "tv")
    assert first["p"]["audience"] == "family"
    done = len(chat.calls)
    write_profiles(ctx, chat)
    assert len(chat.calls) == done


def test_category_and_topic_rules_read_facts_only():
    row = {
        "keywords": {"stand-up comedy"},
        "title": "A Night",
        "genres": {"Comedy"},
        "countries": {"US"},
        "overview": "two rivals plan a heist",
    }
    assert category_rule({"parts": ["stand-up"]}, row)
    assert not category_rule({"parts": ["stand-up"], "needs_genres": ["Drama"]}, row)
    assert topic_rule({"genres_any": [], "terms": ["heist"]}, row)
    assert not topic_rule({"genres_any": ["Drama"], "terms": ["heist"]}, row)
    assert topic_rule({"countries": {"US": 1}, "genres_any": [], "terms": []}, row)


def test_h133_topics_come_from_the_project_s_own_row_words():
    topics = own_topics()
    vocab = load("rows")
    allowed = {p for p, t in vocab["themes"].items() if t["standalone"]}
    allowed |= {e["name"] for e in vocab["extra"]}
    assert topics
    assert {t["name"] for t in topics.values()} <= allowed
    for topic in topics.values():
        assert topic["terms"]
        assert topic["media"] == ["movie", "tv"]


def test_only_titles_the_judge_accepts_are_kept(context):
    ctx = built(context(), 120)
    chat = FakeChat(lambda user: "yes" if "ghost" in user else "no")
    write_topics(ctx, chat)
    judged = json.loads((ctx.work / "export" / "topics.json").read_text())
    keys = [key for members in judged.values() for key in members]
    assert chat.calls
    for key in keys:
        assert key.split(":")[0] in ("movie", "tv")
    meta = json.loads((ctx.work / "export" / "topics_meta.json").read_text())
    assert set(judged) <= set(meta)


def test_no_endpoint_means_no_model_client(context):
    assert make_chat(context()) is None


def test_a_set_endpoint_with_a_named_key_file_reads_only_that_file(context, key_file):
    ctx = context()
    ctx.settings = replace(
        ctx.settings, name=NameSettings("https://llm.fake.test/v1", "small", key_file)
    )
    chat = make_chat(ctx)
    assert isinstance(chat, HttpChat)
    assert "not-a-real" not in repr(chat.key)


def test_chat_sends_the_model_and_reads_the_first_choice():
    transport = FakeTransport(
        {"/chat/completions": {"choices": [{"message": {"content": "Hello"}}]}}
    )
    chat = HttpChat("https://llm.fake.test/v1", "small", transport)
    assert chat.ask("sys", "user", max_tokens=3) == "Hello"
    body = json.loads(transport.calls[0].body)
    assert body["model"] == "small"
    assert body["max_tokens"] == 3
    assert body["temperature"] == 0


def test_chat_failures_are_plain(context):
    down = HttpChat(
        "https://llm.fake.test/v1", "small", FakeTransport({"/chat/completions": NetError("x")})
    )
    with pytest.raises(StageFailed, match="no answer"):
        down.ask("s", "u")
    refused = HttpChat(
        "https://llm.fake.test/v1", "small", FakeTransport({"/chat/completions": Reply(500)})
    )
    with pytest.raises(StageFailed, match="answered 500"):
        refused.ask("s", "u")


def test_a_key_file_setting_without_an_endpoint_changes_nothing(context, tmp_path):
    ctx = context()
    ctx.settings = replace(ctx.settings, name=NameSettings(key_file=str(tmp_path / "absent")))
    assert make_chat(ctx) is None


def test_a_missing_setting_for_the_http_embedder_stops_the_name_stage_with_its_name(context):
    ctx = context()
    ctx.settings = replace(ctx.settings, embed=Embed(backend="http"))
    from databuild.embed.backends import make_embedder

    with pytest.raises(FeatureOff, match=r"embed\.endpoint"):
        make_embedder(ctx)
