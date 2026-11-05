"""Embed stage. Runs the hashing back end, so no model and no network."""

import json
from dataclasses import replace

import numpy as np
import pytest

from databuild.embed import backends, run
from databuild.embed.backends import HashingEmbedder, HttpEmbedder, make_embedder
from databuild.embed.quant import quantise, restore, row_type, unit
from databuild.embed.run import embed, embed_items, read_master
from databuild.embed.text import full_text
from databuild.errors import FeatureOff, StageFailed
from databuild.net import NetError, Reply
from databuild.settings import Embed

from .conftest import FakeTransport
from .fixtures import make_titles


def small(ctx, dimensions=32):
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=dimensions))
    return ctx


def test_full_text_recipe_is_genres_tagline_overview_people():
    text = full_text("movie", "Drama", "A line", "Overview", "Ann", "A, B, C, D, E, F")
    assert text == "Genres: Drama | A line | Overview | Directed by: Ann | Cast: A, B, C, D, E"
    assert full_text("tv", "", "", "O", "Dee", "").endswith("Created by: Dee")
    assert full_text("tv", "", "", "O", "", "") == "O"


def test_full_text_has_size_limits():
    text = full_text("movie", "Q" * 200, "X" * 200, "Z" * 900, "K" * 100, "")
    assert text.count("Q") == 100
    assert text.count("X") == 150
    assert text.count("Z") == 800
    assert text.count("K") == 80


def test_s134_master_row_is_1026_bytes_and_restores_to_a_unit_vector():
    assert row_type(1024).itemsize == 1026
    x = unit(np.random.default_rng(0).normal(size=(4, 1024)).astype(np.float32))
    rows = quantise(x)
    back = restore(rows)
    assert back.dtype == np.float32
    assert np.allclose(np.linalg.norm(back, axis=1), 1.0, atol=1e-4)
    assert (back * x).sum(1).min() > 0.99


def test_s134_value_is_q_times_s_over_norm_plus_small_number():
    rows = np.zeros(1, dtype=row_type(3))
    rows["q"] = [[3, 4, 0]]
    rows["s"] = [2.0]
    want = np.array([6.0, 8.0, 0.0]) / (10.0 + 1e-9)
    assert np.allclose(restore(rows)[0], want)


def test_hashing_is_repeatable_and_similar_text_scores_higher():
    e = HashingEmbedder(256)
    a, b, c = unit(e.encode(["ghost in the attic", "ghost in the cellar", "wedding cake picnic"]))
    assert np.array_equal(e.encode(["ghost in the attic"])[0], e.encode(["ghost in the attic"])[0])
    assert a @ b > a @ c


def test_hashing_gives_a_vector_for_empty_text():
    assert np.linalg.norm(HashingEmbedder(8).encode([""])[0]) == 1.0


def test_embed_writes_master_keys_and_record(context):
    ctx = small(context())
    rows = make_titles(ctx.work / "titles.db", 30)
    embed(ctx)
    master, keys, info = read_master(ctx.work / "vectors")
    assert len(master) == len(keys) == 30
    assert sorted(keys.tolist()) == sorted(r["id"] for r in rows)
    assert info == {
        "model": "hashing-v1",
        "dimensions": 32,
        "quantisation": "int8-with-f16-scale",
        "recipe": "genres-tagline-overview-people-v1",
        "rows": 30,
    }


class Counting(HashingEmbedder):
    calls = 0

    def encode(self, texts):
        Counting.calls += 1
        return super().encode(texts)


def test_a_second_run_resumes_from_finished_blocks(context, monkeypatch):
    ctx = small(context())
    make_titles(ctx.work / "titles.db", 30)
    monkeypatch.setattr(run, "BLOCK", 10)
    items = run.full_items(ctx.work / "titles.db")
    Counting.calls = 0
    embed_items(ctx, Counting(32), items, "r", ctx.work / "v")
    assert Counting.calls == 3
    embed_items(ctx, Counting(32), items, "r", ctx.work / "v")
    assert Counting.calls == 3


def test_h63_changed_text_or_another_model_is_never_mixed_in(context, monkeypatch):
    ctx = small(context())
    make_titles(ctx.work / "titles.db", 30)
    monkeypatch.setattr(run, "BLOCK", 10)
    items = run.full_items(ctx.work / "titles.db")
    embed_items(ctx, Counting(32), items, "r", ctx.work / "v")
    Counting.calls = 0
    items[0] = run.Item(items[0].key, "new text")
    embed_items(ctx, Counting(32), items, "r", ctx.work / "v")
    assert Counting.calls == 1
    Counting.calls = 0
    embed_items(ctx, Counting(32), items, "another recipe", ctx.work / "v")
    assert Counting.calls == 3


def test_a_model_with_too_few_dimensions_is_refused(context):
    ctx = small(context(), dimensions=64)
    with pytest.raises(StageFailed, match="shape"):
        embed_items(ctx, HashingEmbedder(32), [run.Item(1, "x")], "r", ctx.work / "v")


def test_two_rows_with_one_key_are_refused(context):
    ctx = small(context())
    items = [run.Item(1, "a"), run.Item(1, "b")]
    with pytest.raises(StageFailed, match="one key"):
        embed_items(ctx, HashingEmbedder(32), items, "r", ctx.work / "v")


def test_read_master_refuses_rows_and_keys_that_disagree(context):
    ctx = small(context())
    make_titles(ctx.work / "titles.db", 12)
    embed(ctx)
    keys = ctx.work / "vectors" / "master.keys"
    keys.write_bytes(keys.read_bytes()[:-4])
    with pytest.raises(StageFailed, match="disagree"):
        read_master(ctx.work / "vectors")


def test_open_text_builds_a_second_set_from_that_file_alone(context, tmp_path):
    ctx = small(context())
    path = tmp_path / "open.jsonl"
    lines = [{"key": "tv:2", "text": "series"}, {"key": "movie:9", "text": "film"}]
    path.write_text("\n".join(json.dumps(x) for x in lines))
    embed(ctx, open_text=path)
    _, keys, info = read_master(ctx.work / "vectors-open")
    assert keys.tolist() == [9, 2 | (1 << 31)]
    assert info["recipe"] == "open-text-v1"
    assert not (ctx.work / "vectors").exists()


def test_open_text_refuses_a_bare_id(context, tmp_path):
    ctx = small(context())
    path = tmp_path / "open.jsonl"
    path.write_text(json.dumps({"key": "9", "text": "film"}))
    with pytest.raises(ValueError):
        embed(ctx, open_text=path)


def test_http_back_end_orders_rows_by_index_and_cuts_nothing_itself():
    answer = {"data": [{"index": 1, "embedding": [0, 1]}, {"index": 0, "embedding": [1, 0]}]}
    e = HttpEmbedder("http://model.test/v1", "m", FakeTransport({"/embeddings": answer}))
    assert e.encode(["a", "b"]).tolist() == [[1.0, 0.0], [0.0, 1.0]]


def test_http_back_end_failures_are_plain(tmp_path):
    down = HttpEmbedder("http://model.test", "m", FakeTransport({"/embeddings": NetError("x")}))
    with pytest.raises(StageFailed, match="no answer"):
        down.encode(["a"])
    bad = HttpEmbedder("http://model.test", "m", FakeTransport({"/embeddings": Reply(500)}))
    with pytest.raises(StageFailed, match="500"):
        bad.encode(["a"])


def test_query_form_is_the_instruct_form():
    transport = FakeTransport({"/embeddings": {"data": [{"index": 0, "embedding": [1]}]}})
    HttpEmbedder("http://model.test", "m", transport).encode_queries(["x"], "Find", 64)
    assert json.loads(transport.calls[0].body)["input"] == ["Instruct: Find\nQuery: x"]


def test_back_ends_with_no_setting_are_off(context):
    ctx = context()
    with pytest.raises(FeatureOff, match=r"embed\.endpoint"):
        make_embedder(ctx, Embed(backend="http"))
    with pytest.raises(FeatureOff, match=r"embed\.model"):
        make_embedder(ctx, Embed(backend="local-model"))
    with pytest.raises(StageFailed, match="unknown"):
        make_embedder(ctx, Embed(backend="x"))
    assert isinstance(make_embedder(ctx), backends.HashingEmbedder)


def test_stages_run_out_of_order_name_the_stage_to_run(context):
    ctx = small(context())
    with pytest.raises(StageFailed, match="clean stage"):
        embed(ctx)
    with pytest.raises(StageFailed, match="embed stage"):
        read_master(ctx.work / "vectors")
