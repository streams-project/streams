"""Manifest stage: hashes, model, dimensions and keys, a mix refused, no trust switch."""

import base64
import json
import re
import shutil
import sqlite3
from dataclasses import replace
from datetime import timedelta
from pathlib import Path

import jsonschema
import numpy as np
import pytest

from databuild.embed.run import embed
from databuild.errors import StageFailed
from databuild.export.files import BY_PATH, BY_ROLE, FILES, table
from databuild.export.run import export
from databuild.group.run import group
from databuild.manifest.age import check_age
from databuild.manifest.schema import MANIFEST_SCHEMA, SIGNATURE_SCHEMA, check, load_schema
from databuild.manifest.sign import key_id, read_private, read_public, write_keypair
from databuild.manifest.verify import verify_set
from databuild.manifest.write import sha256_file, unlisted_files, write_manifest
from databuild.name.run import name
from databuild.settings import Age, Contract, Embed, Group, Secret, Sign

from .conftest import NOW
from .fixtures import make_titles

DIM = 64
REPOS = Path(__file__).resolve().parents[2]
CONTRACT_SCHEMAS = REPOS / "platform" / "contract" / "generated" / "schemas"
SERVER_MANIFEST = REPOS / "server" / "crates" / "server-catalog" / "src" / "manifest.rs"
DOCS = Path(__file__).resolve().parents[1] / "docs" / "files.md"


def built(ctx, count=120):
    ctx.settings = replace(ctx.settings, embed=Embed(dimensions=DIM), group=Group(method="kmeans"))
    make_titles(ctx.work / "titles.db", count)
    embed(ctx)
    group(ctx)
    name(ctx)
    export(ctx)
    return ctx


@pytest.fixture
def signed(context, tmp_path):
    """A set signed with a key made for this test."""
    private, public = tmp_path / "sign.key", tmp_path / "sign.pub"
    write_keypair(private, public)
    ctx = built(context())
    ctx.settings = replace(ctx.settings, sign=Sign(str(private), str(public)))
    write_manifest(ctx)
    return ctx


def manifest_of(ctx):
    return json.loads((ctx.out / "manifest.json").read_text())


def rewrite(ctx, change):
    """Changes the manifest and leaves its signature as it was."""
    document = manifest_of(ctx)
    change(document)
    (ctx.out / "manifest.json").write_text(json.dumps(document))


def resign(ctx):
    """A manifest changed by someone who holds the key, so only the later checks can refuse it."""
    key = read_private(Secret(Path(ctx.settings.sign.key_file).read_text()))
    from databuild.manifest.sign import sign, signature_bytes

    data = (ctx.out / "manifest.json").read_bytes()
    (ctx.out / "manifest.sig").write_bytes(signature_bytes(sign(data, key)))


def verify(ctx):
    return verify_set(ctx.out, ctx.settings.sign.public_key_file)


def test_h63_the_manifest_names_model_size_keys_rows_versions_and_a_hash_per_file(signed):
    ctx = signed
    document = manifest_of(ctx)
    assert document["schema_version"] == 1
    assert document["model"] == "hashing-v1"
    assert document["dimensions"] == DIM
    assert document["quantisation"] == "f16"
    assert document["key_scheme"] == "kind_top_bit_u32_le"
    assert document["build_commit"] == "abc123"
    assert document["built_at_ms"] == int(NOW.timestamp() * 1000)
    assert document["dataset_version"].startswith("full-2026-11-05-")
    rows = len(np.fromfile(ctx.out / "vectors.keys", dtype="<u4"))
    assert document["row_count"] == rows
    for role, entry in document["files"].items():
        path = ctx.out / entry["path"]
        assert BY_ROLE[role].path == entry["path"]
        assert entry["sha256"] == sha256_file(path)
        assert entry["bytes"] == path.stat().st_size


def test_h64_the_index_is_not_in_the_set_because_the_server_builds_it(signed):
    assert not any("hnsw" in role or "index" in role for role in manifest_of(signed)["files"])


def test_s219_a_copy_of_the_set_passes_because_hashes_decide(signed, tmp_path):
    ctx = signed
    copy = tmp_path / "copy"
    shutil.copytree(ctx.out, copy)
    assert verify_set(copy, ctx.settings.sign.public_key_file).signed


def test_h63_a_good_set_passes_with_its_signature(signed):
    done = verify(signed)
    assert done.signed
    assert len(done.manifest_sha256) == 64


def test_h63_a_changed_file_is_refused_by_hash(signed):
    path = signed.out / "vectors.bin"
    data = bytearray(path.read_bytes())
    data[0] ^= 1
    path.write_bytes(bytes(data))
    with pytest.raises(StageFailed, match="wrong hash"):
        verify(signed)


def test_h63_a_cut_file_is_refused_by_size(signed):
    path = signed.out / "vectors.keys"
    path.write_bytes(path.read_bytes()[:-4])
    with pytest.raises(StageFailed, match="wrong size"):
        verify(signed)


def test_h63_a_missing_file_is_refused(signed):
    (signed.out / "seeds.bin").unlink()
    with pytest.raises(StageFailed, match="does not exist"):
        verify(signed)


def test_h63_a_file_swapped_in_from_another_build_is_refused(signed, context, tmp_path):
    other = tmp_path / "other"
    shutil.copytree(signed.out, other)
    data = bytearray((other / "catalog.db").read_bytes())
    data[-1] ^= 1
    (signed.out / "catalog.db").write_bytes(bytes(data))
    with pytest.raises(StageFailed, match="wrong hash"):
        verify(signed)


def test_h63_an_edited_manifest_fails_its_signature_first(signed):
    rewrite(signed, lambda m: m.update(row_count=m["row_count"] + 1))
    with pytest.raises(StageFailed, match="signature does not match"):
        verify(signed)


def test_h63_a_row_count_that_does_not_fit_is_refused_even_when_signed(signed):
    rewrite(signed, lambda m: m.update(row_count=m["row_count"] + 1))
    resign(signed)
    with pytest.raises(StageFailed, match="do not fit"):
        verify(signed)


def test_h63_dimensions_that_do_not_fit_are_refused_even_when_signed(signed):
    rewrite(signed, lambda m: m.update(dimensions=DIM + 1))
    resign(signed)
    with pytest.raises(StageFailed, match="do not fit"):
        verify(signed)


def test_h63_a_model_that_differs_from_the_vocabulary_is_refused(signed):
    rewrite(signed, lambda m: m.update(model="another-model"))
    resign(signed)
    with pytest.raises(StageFailed, match="another model"):
        verify(signed)


def test_h63_a_catalogue_from_another_set_is_refused_even_with_good_hashes(signed):
    rewrite(signed, lambda m: m.update(dataset_version="full-other"))
    resign(signed)
    with pytest.raises(StageFailed, match="another set"):
        verify(signed)


def test_h63_an_unknown_schema_or_key_scheme_is_refused(signed):
    rewrite(signed, lambda m: m.update(schema_version=99))
    resign(signed)
    with pytest.raises(StageFailed, match="schema 99"):
        verify(signed)
    rewrite(signed, lambda m: m.update(schema_version=1, key_scheme="other"))
    resign(signed)
    with pytest.raises(StageFailed, match="scheme"):
        verify(signed)


def test_a_key_outside_the_catalogue_is_refused(signed):
    path = signed.out / "vectors.keys"
    keys = np.fromfile(path, dtype="<u4")
    keys[0] = 999_999
    path.write_bytes(keys.tobytes())
    rewrite(signed, lambda m: m["files"]["vector_keys"].update(sha256=sha256_file(path)))
    resign(signed)
    with pytest.raises(StageFailed, match="not in the catalogue"):
        verify(signed)


def test_two_rows_with_one_key_are_refused(signed):
    path = signed.out / "vectors.keys"
    keys = np.fromfile(path, dtype="<u4")
    keys[1] = keys[0]
    path.write_bytes(keys.tobytes())
    rewrite(signed, lambda m: m["files"]["vector_keys"].update(sha256=sha256_file(path)))
    resign(signed)
    with pytest.raises(StageFailed, match="two rows"):
        verify(signed)


def test_a_vocabulary_file_of_the_wrong_length_is_refused(signed):
    path = signed.out / "taxonomy.bin"
    path.write_bytes(path.read_bytes()[:-4])
    size = path.stat().st_size
    rewrite(
        signed,
        lambda m: m["files"]["taxonomy_vectors"].update(sha256=sha256_file(path), bytes=size),
    )
    resign(signed)
    with pytest.raises(StageFailed, match="does not fit its entries"):
        verify(signed)


def test_a_cut_seed_file_is_refused(signed):
    path = signed.out / "seeds.bin"
    path.write_bytes(path.read_bytes()[:-2])
    size = path.stat().st_size
    rewrite(signed, lambda m: m["files"]["seeds"].update(sha256=sha256_file(path), bytes=size))
    resign(signed)
    with pytest.raises(StageFailed, match="seed file"):
        verify(signed)


def test_a_path_outside_the_folder_is_refused_by_the_schema(signed):
    for bad in ("../vectors.bin", "/etc/hosts", "a/../../b"):
        rewrite(signed, lambda m, bad=bad: m["files"]["vectors"].update(path=bad))
        resign(signed)
        with pytest.raises(StageFailed, match="does not fit"):
            verify(signed)


def test_h64_no_check_can_be_turned_off():
    from inspect import signature

    assert list(signature(verify_set).parameters) == ["folder", "public_key_file", "schema_dir"]


def test_a_set_with_a_public_key_set_and_no_signature_is_refused(signed):
    (signed.out / "manifest.sig").unlink()
    with pytest.raises(StageFailed, match="not signed"):
        verify(signed)


def test_a_signature_by_another_key_is_refused(signed, tmp_path):
    other_private, other_public = tmp_path / "o.key", tmp_path / "o.pub"
    write_keypair(other_private, other_public)
    with pytest.raises(StageFailed, match="does not hold"):
        verify_set(signed.out, str(other_public))


def test_a_signature_that_is_not_base64_is_refused(signed):
    path = signed.out / "manifest.sig"
    held = json.loads(path.read_text())
    held["signature"] = "not base64 at all!"
    path.write_text(json.dumps(held))
    with pytest.raises(StageFailed, match="signature does not match"):
        verify(signed)


def test_a_flipped_signature_is_refused(signed):
    path = signed.out / "manifest.sig"
    held = json.loads(path.read_text())
    raw = bytearray(base64.b64decode(held["signature"]))
    raw[0] ^= 1
    held["signature"] = base64.b64encode(bytes(raw)).decode()
    path.write_text(json.dumps(held))
    with pytest.raises(StageFailed, match="signature does not match"):
        verify(signed)


def test_no_signing_key_leaves_the_set_unsigned_and_says_so(context):
    ctx = built(context())
    write_manifest(ctx)
    assert not (ctx.out / "manifest.sig").exists()
    assert any("unsigned" in line and "sign.key_file" in line for line in ctx.said)
    assert not verify_set(ctx.out).signed


def test_an_old_signature_does_not_stay_beside_an_unsigned_manifest(signed):
    signed.settings = replace(signed.settings, sign=Sign())
    write_manifest(signed)
    assert not (signed.out / "manifest.sig").exists()


def test_keygen_never_overwrites_and_the_private_file_is_owner_only(tmp_path):
    private, public = tmp_path / "k", tmp_path / "p"
    made = write_keypair(private, public)
    assert private.stat().st_mode & 0o777 == 0o600
    assert made == key_id(read_public(str(public)))
    with pytest.raises(StageFailed, match="exists"):
        write_keypair(private, tmp_path / "other")


def test_a_key_of_the_wrong_kind_is_refused_and_its_text_never_prints(tmp_path):
    with pytest.raises(StageFailed) as caught:
        read_private(Secret("not a key at all"))
    assert "not a key" not in str(caught.value)
    path = tmp_path / "p"
    path.write_text("junk")
    with pytest.raises(StageFailed, match="public key cannot be read"):
        read_public(str(path))
    with pytest.raises(StageFailed, match="cannot read the public key"):
        read_public(str(tmp_path / "absent"))


def test_h132_a_file_outside_the_table_stops_the_manifest(context):
    ctx = built(context())
    (ctx.out / "stray.bin").write_bytes(b"x")
    with pytest.raises(StageFailed, match="not in the sidecar table"):
        write_manifest(ctx)


def test_a_folder_listing_ignores_only_bookkeeping_files(context):
    ctx = built(context())
    write_manifest(ctx)
    assert unlisted_files(ctx.out) == []


def test_a_required_file_that_is_missing_stops_the_manifest(context):
    ctx = built(context())
    (ctx.out / "taxonomy.bin").unlink()
    with pytest.raises(StageFailed, match=r"required file taxonomy\.bin"):
        write_manifest(ctx)


def test_no_export_record_is_named(context):
    with pytest.raises(StageFailed, match="run the export stage"):
        write_manifest(context())


def test_h133_a_capture_field_in_a_built_file_stops_the_manifest(context):
    ctx = built(context())
    path = ctx.out / "rows.json"
    document = json.loads(path.read_text())
    document["requests"][0]["depth"] = 40
    path.write_text(json.dumps(document))
    with pytest.raises(StageFailed, match="capture field"):
        write_manifest(ctx)


def test_h133_a_field_named_after_a_capture_stops_the_manifest(context):
    ctx = built(context())
    path = ctx.out / "rows.json"
    document = json.loads(path.read_text())
    document["requests"][0]["n" + "etflix_name"] = "x"
    path.write_text(json.dumps(document))
    with pytest.raises(StageFailed, match="capture field"):
        write_manifest(ctx)


def test_h133_a_refused_word_in_own_words_stops_the_manifest(context):
    ctx = built(context())
    path = ctx.out / "rows.json"
    document = json.loads(path.read_text())
    document["requests"][0]["name"] = "N" + "etflix Picks"
    path.write_text(json.dumps(document))
    with pytest.raises(StageFailed, match="refused word"):
        write_manifest(ctx)


def test_h133_a_refused_file_name_stops_the_manifest(context):
    ctx = built(context())
    (ctx.out / ("n" + "etflix_tags.json")).write_text("{}")
    with pytest.raises(StageFailed, match="refused file"):
        write_manifest(ctx)


def test_h133_capture_tags_in_the_catalogue_stop_the_manifest(context):
    ctx = built(context())
    db = sqlite3.connect(ctx.out / "catalog.db")
    db.execute("UPDATE titles SET keywords = ? WHERE tmdb_id = 1", ('["' + "nf:" + 'heist"]',))
    db.commit()
    db.close()
    with pytest.raises(StageFailed, match="capture tags"):
        write_manifest(ctx)


def test_a_column_from_a_refused_input_stops_the_manifest(context):
    ctx = built(context())
    db = sqlite3.connect(ctx.out / "catalog.db")
    db.execute("ALTER TABLE titles ADD COLUMN " + "n" + "etflix_listed INTEGER")
    db.commit()
    db.close()
    with pytest.raises(StageFailed, match="refused input"):
        write_manifest(ctx)


def test_h62_a_new_set_passes_the_age_check_and_an_old_one_fails(signed):
    ctx = signed
    check_age(ctx)
    assert any(line.startswith("age: set") for line in ctx.said)
    ctx.now = lambda: NOW + timedelta(days=201)
    with pytest.raises(StageFailed, match="days old"):
        check_age(ctx)


def test_h62_a_daily_manifest_does_not_make_an_old_catalogue_look_new(signed):
    ctx = signed
    built_at = manifest_of(ctx)["built_at_ms"]
    ctx.now = lambda: NOW + timedelta(days=201)
    write_manifest(ctx)
    assert manifest_of(ctx)["built_at_ms"] == built_at
    with pytest.raises(StageFailed, match="days old"):
        check_age(ctx)


def test_h62_a_frozen_popularity_file_ends_the_job_with_a_failure(signed):
    ctx = signed
    db = sqlite3.connect(ctx.out / "popularity.db")
    db.execute("CREATE TABLE meta (k TEXT PRIMARY KEY, v TEXT)")
    db.execute("INSERT INTO meta VALUES ('written', ?)", (NOW.isoformat("T", "seconds"),))
    db.commit()
    db.close()
    check_age(ctx)
    ctx.now = lambda: NOW + timedelta(hours=37)
    ctx.settings = replace(ctx.settings, age=Age(set_days=999))
    with pytest.raises(StageFailed, match=r"popularity is 37\.0 hours old"):
        check_age(ctx)


def test_h62_old_fresh_titles_end_the_job_with_a_failure(signed):
    ctx = signed
    (ctx.out / "fresh.json").write_text(json.dumps({"generated": NOW.isoformat(), "items": []}))
    check_age(ctx)
    ctx.now = lambda: NOW + timedelta(hours=13)
    with pytest.raises(StageFailed, match=r"fresh titles are 13\.0 hours old"):
        check_age(ctx)


def test_h62_no_manifest_fails_the_age_check(context):
    with pytest.raises(StageFailed, match="no manifest"):
        check_age(context())


def test_a_daily_file_joins_the_set_through_a_new_manifest(signed):
    ctx = signed
    db = sqlite3.connect(ctx.out / "popularity.db")
    db.execute("CREATE TABLE meta (k TEXT PRIMARY KEY, v TEXT)")
    db.commit()
    db.close()
    write_manifest(ctx)
    assert "popularity" in manifest_of(ctx)["files"]
    assert verify(ctx).signed


def test_a_daily_rerun_needs_only_the_output_folder(signed, tmp_path):
    ctx = signed
    shutil.rmtree(ctx.work)
    ctx.work.mkdir()
    write_manifest(ctx)
    assert verify(ctx).signed


def test_the_schema_is_a_valid_draft_2020_12_schema():
    for schema in (MANIFEST_SCHEMA, SIGNATURE_SCHEMA):
        jsonschema.Draft202012Validator.check_schema(load_schema(schema))


def test_the_wire_rules_hold_no_float_and_integers_under_2_to_the_53(signed):
    def walk(value):
        assert not isinstance(value, float)
        if isinstance(value, int) and not isinstance(value, bool):
            assert value < 2**53
        if isinstance(value, dict):
            for item in value.values():
                walk(item)
        if isinstance(value, list):
            for item in value:
                walk(item)

    walk(manifest_of(signed))


def test_unknown_fields_are_tolerated_as_on_every_wire_type(signed):
    document = manifest_of(signed)
    document["a_field_from_later"] = {"x": 1}
    document["files"]["catalog"]["also_later"] = 1
    check(document, MANIFEST_SCHEMA)


def test_a_named_schema_folder_is_used_and_a_missing_one_is_refused(tmp_path, context):
    schema = {"type": "object", "required": ["only_this"]}
    (tmp_path / MANIFEST_SCHEMA).write_text(json.dumps(schema))
    with pytest.raises(StageFailed, match="only_this"):
        check({}, MANIFEST_SCHEMA, str(tmp_path))
    with pytest.raises(StageFailed, match="holds no"):
        check({}, MANIFEST_SCHEMA, str(tmp_path / "absent"))
    ctx = built(context())
    ctx.settings = replace(ctx.settings, contract=Contract(str(tmp_path)))
    with pytest.raises(StageFailed, match="only_this"):
        write_manifest(ctx)


def test_the_contract_holds_this_schema_or_none_yet():
    """The shape lives in the contract once it is there, so the two copies can never differ."""
    if not CONTRACT_SCHEMAS.is_dir():
        pytest.skip("the platform repository is not next to this one")
    path = CONTRACT_SCHEMAS / MANIFEST_SCHEMA
    if path.exists():
        contract = json.loads(path.read_text())
        assert set(load_schema(MANIFEST_SCHEMA)["required"]) <= set(contract["required"])


def test_every_field_and_role_the_server_reads_is_written_here(signed):
    if not SERVER_MANIFEST.is_file():
        pytest.skip("the server repository is not next to this one")
    text = SERVER_MANIFEST.read_text()
    roles = re.findall(r'pub const [A-Z_]+: &str = "([a-z_]+)";', text)
    assert roles
    assert set(roles) <= set(BY_ROLE)
    body = text[text.index("pub struct DatasetManifest") :]
    fields = re.findall(r"pub ([a-z_]+):", body[: body.index("}")])
    assert set(fields) <= set(manifest_of(signed))
    assert 'self.quantisation != "f16"' in text
    assert re.search(r"SCHEMA_VERSION: u32 = 1;", text)


def test_h132_the_table_lists_each_file_once_and_docs_match_it():
    assert len({f.role for f in FILES}) == len(FILES)
    assert len(BY_PATH) == len(FILES)
    assert DOCS.read_text() == table()


def test_h132_every_private_input_says_what_a_build_gives_without_it():
    for spec in FILES:
        assert spec.writer and spec.inputs and spec.stranger and spec.missing
