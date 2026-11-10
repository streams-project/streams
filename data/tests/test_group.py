"""Group stage on small generated vectors."""

import json
import sqlite3
from dataclasses import replace

import numpy as np
import pytest

from databuild.embed.quant import quantise, unit
from databuild.embed.run import embed
from databuild.errors import StageFailed
from databuild.group.cluster import (
    Neighbours,
    choose_method,
    knn_exact,
    leiden_available,
    merge_small,
    partition,
    place,
    spherical_kmeans,
)
from databuild.group.params import Params
from databuild.group.run import group
from databuild.group.seasons import Event, calendar_events, keyword_pool
from databuild.group.stats import Fact, group_rows, make_fact, split_list
from databuild.group.vibe import build_vibe, centred, format_bucket, mood_pointers
from databuild.group.world import build_layers
from databuild.settings import Embed, Group, Reviews

from .fixtures import make_titles


def blobs(per=30, centres=4, dim=32, seed=1):
    rng = np.random.default_rng(seed)
    points = rng.normal(size=(centres, dim))
    X = np.concatenate([p + rng.normal(scale=0.08, size=(per, dim)) for p in points])
    labels = np.repeat(np.arange(centres), per)
    return unit(X.astype(np.float32)), labels


def same_grouping(a, b):
    pairs = {(int(x), int(y)) for x, y in zip(a, b, strict=True)}
    return len(pairs) == len(set(a.tolist())) == len(set(b.tolist()))


def quiet(_):
    return None


def test_kmeans_finds_clear_groups_and_repeats_exactly():
    X, truth = blobs()
    one = spherical_kmeans(X, 4, 7)
    assert same_grouping(one, truth)
    assert np.array_equal(one, spherical_kmeans(X, 4, 7))


def test_kmeans_with_more_groups_than_rows_still_answers():
    X, _ = blobs(per=1, centres=3)
    assert len(spherical_kmeans(X, 9, 7)) == 3
    assert spherical_kmeans(X[:1], 4, 7).tolist() == [0]


def test_merge_small_joins_the_closest_group():
    X, truth = blobs(per=10, centres=3)
    labels = truth.copy()
    labels[0] = 5
    merged = merge_small(X, labels, 3)
    assert merged[0] == merged[1]
    assert merged.max() == 2


def test_merge_small_keeps_everything_when_no_group_is_big_enough():
    X, truth = blobs(per=2, centres=3)
    assert merge_small(X, truth, 50).max() == 2


def test_exact_neighbours_skip_the_row_itself():
    X, _ = blobs(per=5, centres=2)
    idx, sim = knn_exact(X, 3)
    assert not (idx == np.arange(len(X))[:, None]).any()
    assert (sim[:, 0] >= sim[:, 1]).all()


def test_vote_takes_the_nearest_group_and_keeps_its_share():
    X, truth = blobs(per=20, centres=2)
    picks, fits = place(Neighbours(X[1:]), X[:1], [truth[1:]], 5, 0.05)
    assert picks[0][0] == truth[0]
    assert fits[0][0] > 0.9


def test_choose_method_never_swaps_silently(monkeypatch):
    assert choose_method("kmeans", quiet) == "kmeans"
    with pytest.raises(StageFailed, match="unknown"):
        choose_method("x", quiet)
    monkeypatch.setattr("databuild.group.cluster.leiden_available", lambda: False)
    assert choose_method("auto", quiet) == "kmeans"
    with pytest.raises(StageFailed, match="full extra"):
        choose_method("leiden", quiet)


@pytest.mark.full
def test_leiden_finds_clear_groups():
    if not leiden_available():
        pytest.skip("the full extra is not installed")
    X, truth = blobs()
    labels = partition(X, 4, "leiden", 7, k=8)
    assert same_grouping(labels, truth)


@pytest.mark.full
def test_layers_with_leiden_label_every_title():
    if not leiden_available():
        pytest.skip("the full extra is not installed")
    X, _ = blobs(per=40, centres=5)
    text = np.full(len(X), 200)
    text[::7] = 10
    p = Params.for_titles(len(X))
    layers = build_layers(X, text, p, "leiden", 7, quiet)
    assert (layers.w2 >= 0).all()
    assert layers.wsrc.sum() == (text < p.min_text).sum()


def test_layers_nest_and_thin_titles_are_placed_by_vote():
    X, truth = blobs(per=40, centres=4)
    text = np.full(len(X), 200)
    text[::9] = 10
    p = replace(
        Params.for_titles(len(X)), min_text=100, target_l1=4, min_l1=4, sub_size=15, min_split=30
    )
    layers = build_layers(X, text, p, "kmeans", 7, quiet)
    assert same_grouping(layers.w1, truth)
    for w2 in np.unique(layers.w2):
        assert len(set(layers.w1[layers.w2 == w2].tolist())) == 1
    assert layers.wsrc.sum() == (text < 100).sum()
    assert (layers.wconf[layers.wsrc == 1] > 0).all()
    assert (layers.wconf[layers.wsrc == 0] == 1).all()
    assert layers.th1.max() <= layers.th3.max()


def test_full_size_numbers_are_the_measured_ones():
    p = Params.for_titles(700000)
    assert (p.min_text, p.k_graph, p.k_vote, p.tau) == (100, 15, 25, 0.05)
    assert (p.target_l1, p.min_l1, p.sub_size, p.min_split, p.min_sub) == (60, 200, 150, 300, 30)
    assert p.theme_targets == (150, 400, 1000)


def test_params_ramp_smoothly_between_15000_and_25000():
    p_15k = Params.for_titles(15000)
    p_scaled_15k = Params._scaled_params(15000)
    assert p_15k == p_scaled_15k

    p_25k = Params.for_titles(25000)
    full = Params()
    assert p_25k == full

    p_20k = Params.for_titles(20000)
    # Check that midpoint smoothly sits strictly between scaled and full
    assert p_scaled_15k.min_text < p_20k.min_text < full.min_text
    assert p_20k.min_text == round(0.5 * p_scaled_15k.min_text + 0.5 * full.min_text)
    assert p_20k.target_l1 == round(0.5 * Params._scaled_params(20000).target_l1 + 0.5 * full.target_l1)


def test_stats_measure_shares_and_lifts():
    facts = [
        make_fact(f"T{i}", "1994", "tv" if i < 5 else "movie", "en", "Drama", '["US"]', i)
        for i in range(10)
    ]
    facts += [make_fact(f"U{i}", "2004", "movie", "fr", "Comedy", '["FR"]', i) for i in range(30)]
    rows = group_rows(facts, np.array([0] * 10 + [1] * 30), "world", 1, None, np.ones(40))
    assert [r[4] for r in rows] == [10, 30]
    stats = json.loads(rows[0][7])
    assert stats["genres"] == [["Drama", 1.0, 4.0]]
    assert stats["countries"] == [["US", 1.0, 4.0]]
    assert stats["decades"] == [[1990, 10]]
    assert stats["tv_share"] == 0.5
    assert stats["median_year"] == 1994
    assert stats["examples"][0] == "T9 (1994)"


def test_split_list_reads_json_and_comma_forms():
    assert split_list('["US", "GB"]') == ["US", "GB"]
    assert split_list("Drama, Comedy") == ["Drama", "Comedy"]
    assert split_list(None) == []


def test_negative_labels_stay_out_of_groups():
    facts = [Fact("a", "2000", "movie", "en", (), "", 1.0)] * 3
    rows = group_rows(facts, np.array([0, -1, 0]), "vibe", 1, None, np.ones(3))
    assert [r[4] for r in rows] == [2]


def test_format_buckets():
    assert format_bucket("tv", 5) == 1
    assert format_bucket("movie", 20) == 2
    assert format_bucket("movie", 100) == 0
    assert format_bucket("movie", None) == 0


def test_centring_is_per_bucket_and_skips_small_ones():
    R = unit(np.ones((40, 4), np.float32) + np.random.default_rng(0).normal(size=(40, 4)))
    buckets = np.array([0] * 35 + [1] * 5)
    out = centred(R, buckets)
    assert abs(float(out[:35].mean())) < 0.1
    assert np.allclose(out[35:], R[35:])


def test_vibe_seeds_make_groups_and_others_are_placed():
    X, _ = blobs(per=30, centres=3)
    keys = np.arange(1, len(X) + 1, dtype="<u4")
    seeds = np.arange(0, len(X), 2)
    p = replace(
        Params.for_titles(len(X)),
        vibe_fine_size=15,
        vibe_mid_size=30,
        min_vibe_fine=2,
        min_vibe_mid=2,
    )
    vibe = build_vibe(X, keys, np.zeros(len(X), np.int8), X[seeds], keys[seeds], p, "kmeans", 7)
    assert vibe.seeded.sum() == len(seeds)
    assert (vibe.fine >= 0).all()
    assert vibe.fine_core[seeds].all()
    assert (vibe.fine_fit[seeds] == 1).all()
    assert set(vibe.fine_centres) == set(np.unique(vibe.fine[seeds]).tolist())


def test_vibe_with_no_matching_review_says_so():
    X, _ = blobs(per=4, centres=2)
    keys = np.arange(1, len(X) + 1, dtype="<u4")
    with pytest.raises(StageFailed, match="no review vector"):
        build_vibe(
            X, keys, np.zeros(len(X), np.int8), X, keys + 1000, Params.for_titles(8), "kmeans", 7
        )


def test_mood_pointers_keep_six_and_skip_small_groups():
    rng = np.random.default_rng(0)
    centres = {g: unit(rng.normal(size=(1, 8)).astype(np.float32))[0] for g in range(9)}
    sizes = dict.fromkeys(range(9), 40) | {3: 2}
    moods = [{"id": "a", "title": "A", "late": 1}]
    rows = mood_pointers(
        moods,
        unit(rng.normal(size=(1, 8)).astype(np.float32)),
        unit(rng.normal(size=(20, 8))),
        centres,
        sizes,
        30,
    )
    assert [r[5] for r in rows] == [0, 1, 2, 3, 4, 5]
    assert 3 not in [r[3] for r in rows]
    assert rows[0][4] >= rows[-1][4]


def test_every_calendar_event_has_keywords_and_a_name():
    events = calendar_events()
    assert len(events) > 20
    assert all(e.name and e.titles for e in events)


def test_season_pool_takes_quality_rows_with_the_event_keywords():
    event = Event("x", frozenset({"ghost"}), False, ("X",))
    words = {0: {"ghost"}, 1: {"ghost"}, 2: {"robot"}}
    assert keyword_pool(event, words, np.array([1, 2])) == [1]


def test_central_events_match_by_word_part():
    event = Event("y", frozenset(), True, ("Y",))
    words = {0: {"gay theme"}, 1: {"gayle"}, 2: {"lesbian relationship"}}
    assert keyword_pool(event, words, np.array([0, 1, 2])) == [0, 2]


def run_group(ctx, count=120):
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=64), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", count)
    embed(ctx)
    group(ctx)
    return sqlite3.connect(ctx.work / "groups.db")


def test_group_stage_writes_tables_and_arrays(context):
    ctx = context()
    db = run_group(ctx)
    kinds = dict(db.execute("SELECT kind || level, count(*) FROM groups GROUP BY 1"))
    assert {"world1", "world2", "atom1", "theme1", "theme2", "theme3"} <= set(kinds)
    assert db.execute("SELECT count(*) FROM mood_vectors").fetchone() == (26,)
    arrays = np.load(ctx.work / "groups.npz")
    assert str(arrays["method"]) == "kmeans"
    assert len(arrays["w2"]) == 120


def test_h132_no_review_input_turns_the_vibe_layer_off_and_says_so(context):
    ctx = context()
    db = run_group(ctx)
    assert db.execute("SELECT count(*) FROM groups WHERE kind = 'vibe'").fetchone() == (0,)
    assert db.execute("SELECT count(*) FROM mood_pointers").fetchone() == (0,)
    assert any("vibe layer off" in line for line in ctx.said)


def test_review_vectors_make_the_vibe_tables(context, tmp_path):
    ctx = context()
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=64), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", 120)
    embed(ctx)
    from databuild.embed.run import read_master

    rows, keys, _ = read_master(ctx.work / "vectors")
    (tmp_path / "r.q8").write_bytes(rows[:60].tobytes())
    (tmp_path / "r.keys").write_bytes(keys[:60].tobytes())
    ctx.settings = replace(
        ctx.settings, reviews=Reviews(str(tmp_path / "r.q8"), str(tmp_path / "r.keys"))
    )
    group(ctx)
    db = sqlite3.connect(ctx.work / "groups.db")
    assert db.execute("SELECT count(*) FROM groups WHERE kind = 'vibe'").fetchone()[0] > 0
    assert db.execute("SELECT count(*) FROM group_centroids").fetchone()[0] > 0
    assert db.execute("SELECT count(*) FROM mood_pointers").fetchone()[0] > 0
    assert "vf" in np.load(ctx.work / "groups.npz")


def test_review_keys_that_disagree_with_their_rows_are_refused(context, tmp_path):
    ctx = context()
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=64), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", 20)
    embed(ctx)
    (tmp_path / "r.q8").write_bytes(quantise(unit(np.ones((2, 64), np.float32))).tobytes())
    (tmp_path / "r.keys").write_bytes(np.array([1], dtype="<u4").tobytes())
    ctx.settings = replace(
        ctx.settings, reviews=Reviews(str(tmp_path / "r.q8"), str(tmp_path / "r.keys"))
    )
    with pytest.raises(StageFailed, match="disagree"):
        group(ctx)


def test_h63_vectors_of_a_title_the_titles_file_lacks_are_refused(context):
    ctx = context()
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=64), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", 30)
    embed(ctx)
    make_titles(ctx.work / "titles.db", 20)
    with pytest.raises(StageFailed, match="titles file does not"):
        group(ctx)
