"""Export stage on small generated data."""

import json
import math
import sqlite3
import struct
from dataclasses import replace

import numpy as np
import pytest

from databuild.embed.quant import quantise, restore, unit
from databuild.embed.run import embed, read_master
from databuild.errors import StageFailed
from databuild.export.catalog import APPLICATION_ID, SCHEMA_VERSION
from databuild.export.files import BY_ROLE, EXPORT_RECORD, FILES
from databuild.export.run import export
from databuild.export.vectors import Member, make_seeds, read_seeds, seed_bytes
from databuild.group.run import group
from databuild.keys import TV_BIT, unpack
from databuild.name.run import name
from databuild.settings import Embed, Group
from databuild.settings import Export as ExportSettings

from .fixtures import make_titles

DIM = 64


def built(ctx, count=240):
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=DIM), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", count)
    embed(ctx)
    group(ctx)
    name(ctx)
    return ctx


def opened(ctx):
    return sqlite3.connect(ctx.out / "catalog.db")


def test_s134_vectors_are_the_restored_unit_rows_in_master_order_as_float16(context):
    ctx = built(context())
    export(ctx)
    rows, keys, _ = read_master(ctx.work / "vectors")
    kept = np.fromfile(ctx.out / "vectors.keys", dtype="<u4")
    written = np.fromfile(ctx.out / "vectors.bin", dtype="<f2").reshape(len(kept), DIM)
    positions = {int(k): i for i, k in enumerate(keys)}
    index = [positions[int(k)] for k in kept]
    assert index == sorted(index)
    assert np.array_equal(written, restore(rows[index]).astype("<f2"))
    assert np.allclose(np.linalg.norm(written.astype(np.float32), axis=1), 1.0, atol=1e-2)


def test_s134_only_titles_of_the_view_get_a_vector(context):
    ctx = context()
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=DIM), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", 240)
    db = sqlite3.connect(ctx.work / "titles.db")
    db.execute("UPDATE titles SET poster_path = '' WHERE tmdb_id % 7 = 0")
    db.commit()
    db.close()
    embed(ctx)
    group(ctx)
    name(ctx)
    export(ctx)
    db = opened(ctx)
    shown = {k for (k,) in db.execute("SELECT id FROM catalog")}
    held = db.execute("SELECT count(*) FROM titles").fetchone()[0]
    kept = set(np.fromfile(ctx.out / "vectors.keys", dtype="<u4").tolist())
    assert kept == shown
    assert len(shown) < held
    assert any("vectors left out" in line for line in ctx.said)


def test_s133_the_view_uses_the_build_date_and_titles_stay_in_the_table(context):
    ctx = built(context())
    export(ctx)
    db = opened(ctx)
    view = db.execute("SELECT sql FROM sqlite_master WHERE name = 'catalog'").fetchone()[0]
    assert ">= 2025" in view


def test_the_view_gives_public_ids_and_packed_keys(context):
    ctx = built(context())
    export(ctx)
    for key, title_id, kind in opened(ctx).execute("SELECT id, title_id, media_type FROM catalog"):
        assert unpack(key) == (kind, title_id)
    tv = opened(ctx).execute("SELECT count(*) FROM catalog WHERE id >= ?", (TV_BIT,)).fetchone()
    assert tv[0] > 0


def test_h63_the_file_says_what_it_is(context):
    ctx = built(context())
    export(ctx)
    db = opened(ctx)
    assert db.execute("PRAGMA application_id").fetchone() == (APPLICATION_ID,)
    assert db.execute("PRAGMA user_version").fetchone() == (SCHEMA_VERSION,)
    meta = dict(db.execute("SELECT k, v FROM meta"))
    assert meta["model"] == "hashing-v1"
    assert meta["as_of"] == "2026-11-05"
    assert meta["build_commit"] == "abc123"


def test_h70_the_text_index_exists_and_finds_overview_words(context):
    ctx = built(context())
    export(ctx)
    found = opened(ctx).execute(
        "SELECT c.title FROM catalog_fts f JOIN catalog c ON c.id = f.id"
        " WHERE catalog_fts MATCH 'detective' LIMIT 3"
    )
    assert "Crime" in found.fetchone()[0]


def test_placement_columns_and_group_tables_reach_the_catalogue(context):
    ctx = built(context())
    export(ctx)
    db = opened(ctx)
    placed = db.execute("SELECT count(*) FROM titles WHERE w1 IS NOT NULL").fetchone()[0]
    assert placed == db.execute("SELECT count(*) FROM titles").fetchone()[0]
    assert db.execute("SELECT count(*) FROM mood_vectors").fetchone() == (26,)
    assert db.execute("SELECT count(*) FROM groups WHERE name <> ''").fetchone()[0] > 0


def test_s154_seed_mass_counts_only_rated_titles_with_votes():
    rows = unit(np.eye(4, dtype=np.float32))
    keys = np.array([1, 2, 3, 4], dtype="<u4")
    groups = np.array([0, 0, 1, 1])
    members = {
        1: Member(10.0, 7.0, 60),
        2: Member(10.0, 5.9, 60),
        3: Member(99.0, 9.0, 49),
        4: Member(1.0, 6.0, 50),
    }
    seeds = make_seeds(rows, keys, groups, members)
    assert [s.group for s in seeds] == [0, 1]
    assert seeds[0].mass == pytest.approx(math.log1p(10.0))
    assert seeds[1].mass == pytest.approx(math.log1p(1.0))
    assert np.linalg.norm(seeds[0].centre) == pytest.approx(1.0, abs=1e-5)


def test_s154_an_open_set_with_no_votes_counts_every_member():
    rows = unit(np.eye(2, dtype=np.float32))
    keys = np.array([1, 2], dtype="<u4")
    members = {1: Member(3.0, 6.0, 0), 2: Member(1.0, 6.0, 0)}
    seeds = make_seeds(rows, keys, np.array([0, 1]), members, votes=False)
    assert [s.group for s in seeds] == [0, 1]
    assert seeds[0].mass > seeds[1].mass > 0


def test_s154_groups_of_equal_mass_keep_a_fixed_order():
    rows = unit(np.eye(2, dtype=np.float32))
    keys = np.array([1, 2], dtype="<u4")
    seeds = make_seeds(rows, keys, np.array([5, 3]), {})
    assert [s.group for s in seeds] == [3, 5]


def test_s154_seed_file_lists_every_shown_title_and_its_names_align(context):
    ctx = built(context())
    export(ctx)
    width, seeds = read_seeds((ctx.out / "seeds.bin").read_bytes())
    assert width == DIM
    members = np.concatenate([m for _, m in seeds])
    assert sorted(members.tolist()) == sorted(
        np.fromfile(ctx.out / "vectors.keys", dtype="<u4").tolist()
    )
    names = json.loads((ctx.out / "seeds.names.json").read_text())
    ids = json.loads((ctx.out / "seeds.ids.json").read_text())
    assert len(names) == len(ids) == len(seeds)
    db = sqlite3.connect(ctx.work / "groups.db")
    held = dict(db.execute("SELECT id, name FROM groups WHERE kind = 'world' AND level = 2"))
    assert names == [held[i] or "" for i in ids]


def test_a_cut_seed_file_is_refused_when_read():
    seeds = make_seeds(unit(np.eye(2, dtype=np.float32)), np.array([1, 2], "<u4"), np.zeros(2), {})
    data = seed_bytes(seeds, 2)
    assert read_seeds(data)[0] == 2
    with pytest.raises(ValueError):
        read_seeds(data[:-1])
    with pytest.raises(ValueError):
        read_seeds(data + b"\0")
    with pytest.raises(ValueError):
        read_seeds(struct.pack("<I", 1))


def test_h63_groups_from_other_vectors_are_refused(context):
    ctx = built(context())
    (ctx.work / "vectors" / "master.keys").write_bytes(
        (np.fromfile(ctx.work / "vectors" / "master.keys", dtype="<u4") + 1).tobytes()
    )
    with pytest.raises(StageFailed, match="other vectors"):
        export(ctx)


def test_h63_vocabulary_from_another_model_is_refused(context):
    ctx = built(context())
    path = ctx.work / "export" / "taxonomy.json"
    document = json.loads(path.read_text())
    document["model"] = "another-model"
    path.write_text(json.dumps(document))
    with pytest.raises(StageFailed, match="another model"):
        export(ctx)


def test_a_stage_that_never_ran_is_named(context):
    ctx = context()
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=DIM), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", 60)
    embed(ctx)
    with pytest.raises(StageFailed, match="group stage"):
        export(ctx)


def test_operator_removed_rows_leave_the_season_tables_and_reach_the_file(context, tmp_path):
    ctx = built(context())
    db = sqlite3.connect(ctx.work / "groups.db")
    db.execute("DELETE FROM season_groups")
    db.execute("DELETE FROM season_members")
    db.execute("INSERT INTO season_groups VALUES ('halloween', 1, 'Spooky Nights')")
    db.execute("INSERT INTO season_groups VALUES ('halloween', 2, 'Kept Nights')")
    db.execute("INSERT INTO season_members VALUES ('halloween', 1, 'movie', 1, 0, 1)")
    db.execute("INSERT INTO season_members VALUES ('halloween', 2, 'movie', 2, 0, 1)")
    db.commit()
    db.close()
    listed = tmp_path / "removed.json"
    listed.write_text('["spooky nights"]')
    ctx.settings = replace(ctx.settings, export=ExportSettings(removed_rows=str(listed)))
    export(ctx)
    built_db = opened(ctx)
    assert built_db.execute("SELECT name FROM season_groups").fetchall() == [("Kept Nights",)]
    assert built_db.execute("SELECT sub FROM season_members").fetchall() == [(2,)]
    assert json.loads((ctx.out / "removed_rows.json").read_text()) == ["spooky nights"]


def test_no_removed_rows_setting_gives_an_empty_list_and_says_so(context):
    ctx = built(context())
    export(ctx)
    assert json.loads((ctx.out / "removed_rows.json").read_text()) == []
    assert any("removed_rows is not set" in line for line in ctx.said)


@pytest.mark.parametrize("text", ["not json", '{"a": 1}', "[1]"])
def test_a_bad_removed_rows_file_stops_the_export(context, tmp_path, text):
    ctx = built(context())
    listed = tmp_path / "removed.json"
    listed.write_text(text)
    ctx.settings = replace(ctx.settings, export=ExportSettings(removed_rows=str(listed)))
    with pytest.raises(StageFailed, match="removed rows"):
        export(ctx)


def test_h132_a_private_input_passes_through_only_when_named(context, tmp_path):
    ctx = built(context())
    export(ctx)
    assert not (ctx.out / "awards.json").exists()
    assert any("awards_file is not set" in line for line in ctx.said)
    held = tmp_path / "held.json"
    held.write_text('{"emmy": {}}')
    ctx.settings = replace(ctx.settings, export=ExportSettings(awards_file=str(held)))
    export(ctx)
    assert (ctx.out / "awards.json").read_text() == '{"emmy": {}}'


def test_a_named_private_input_that_is_absent_stops_the_export(context, tmp_path):
    ctx = built(context())
    ctx.settings = replace(ctx.settings, export=ExportSettings(marks_file=str(tmp_path / "none")))
    with pytest.raises(StageFailed, match="marks_file"):
        export(ctx)


def test_a_file_from_an_earlier_build_does_not_stay(context):
    ctx = built(context())
    (ctx.out / "topics.json").write_text("{}")
    (ctx.out / "fresh.json").write_text("{}")
    export(ctx)
    assert not (ctx.out / "topics.json").exists()
    assert (ctx.out / "fresh.json").exists()


def test_the_same_inputs_give_the_same_bytes(context):
    ctx = built(context())
    export(ctx)
    first = {s.path: (ctx.out / s.path).read_bytes() for s in FILES if (ctx.out / s.path).exists()}
    assert "catalog.db" in first
    export(ctx)
    for path, data in first.items():
        assert (ctx.out / path).read_bytes() == data, path


def test_the_record_names_model_size_rows_and_key_scheme(context):
    ctx = built(context())
    export(ctx)
    record = json.loads((ctx.out / EXPORT_RECORD).read_text())
    assert record["model"] == "hashing-v1"
    assert record["dimensions"] == DIM
    assert record["quantisation"] == "f16"
    assert record["row_count"] == len(np.fromfile(ctx.out / "vectors.keys", dtype="<u4"))
    assert record["key_scheme"] == "kind_top_bit_u32_le"
    assert record["dataset_version"].startswith("full-2026-11-05-")
    assert record["source"] == "full"


def test_a_second_set_from_open_text_goes_to_its_own_folder(context, tmp_path):
    ctx = built(context())
    db = sqlite3.connect(ctx.work / "titles.db")
    keys = [(kind, i) for kind, i in db.execute("SELECT media_type, tmdb_id FROM titles")][:30]
    db.close()
    lines = [json.dumps({"key": f"{k}:{i}", "text": f"open text {i}"}) for k, i in keys]
    source = tmp_path / "open.jsonl"
    source.write_text("\n".join(lines))
    embed(ctx, source)
    export(ctx)
    count = len(np.fromfile(ctx.out / BY_ROLE["open_vector_keys"].path, dtype="<u4"))
    assert 0 < count <= 30
    size = (ctx.out / BY_ROLE["open_vectors"].path).stat().st_size
    assert size == count * DIM * 2
    record = json.loads((ctx.out / EXPORT_RECORD).read_text())
    assert record["open_set"]["recipe"] == "open-text-v1"
    assert record["open_set"]["row_count"] == count


def test_quantised_rows_restore_close_to_their_source():
    source = unit(np.random.default_rng(0).normal(size=(5, DIM)).astype(np.float32))
    restored = restore(quantise(source))
    assert np.allclose((source * restored).sum(axis=1), 1.0, atol=1e-3)
