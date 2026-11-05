//! The manifest gate on the generated sample.

use server_catalog::manifest::{DatasetManifest, ManifestError, role};
use server_catalog::sample::{SAMPLE_DIMS, sample_titles, special, write_sample};
use server_catalog::{DataSet, DataSlot, DataTrust, SwapError, item_key};

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("manifest_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn rewrite(path: &std::path::Path, change: impl FnOnce(&mut DatasetManifest)) {
    let (mut manifest, _) = DatasetManifest::read(path).unwrap();
    change(&mut manifest);
    std::fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
}

#[test]
fn h63_the_sample_loads_through_its_manifest() {
    let manifest = write_sample(&folder("good"));
    let set = DataSet::load(&manifest).unwrap();
    assert_eq!(set.catalog.len(), sample_titles().len());
    assert_eq!(set.dims(), SAMPLE_DIMS);
    assert!(set.dropped.is_empty());
    assert_eq!(set.data_hash.len(), 64);
    let film = &set.catalog[&item_key("movie", special::SHARED_ID)];
    let series = &set.catalog[&item_key("tv", special::SHARED_ID)];
    assert_eq!(
        (film.title.as_str(), series.title.as_str()),
        ("Twin Film", "Twin Series")
    );
}

#[test]
fn h64_a_wrong_file_swap_is_refused() {
    let dir = folder("swap");
    let manifest = write_sample(&dir);
    let mut bytes = std::fs::read(dir.join("vectors.bin")).unwrap();
    bytes[0] ^= 1;
    std::fs::write(dir.join("vectors.bin"), bytes).unwrap();
    assert!(matches!(
        DataSet::load(&manifest),
        Err(SwapError::Manifest(ManifestError::WrongHash(_)))
    ));
}

#[test]
fn s219_a_copy_keeps_working_because_hashes_decide_and_not_file_times() {
    let dir = folder("copy");
    let manifest = write_sample(&dir);
    let copy = folder("copy_target");
    std::fs::create_dir_all(&copy).unwrap();
    for name in [
        "catalog.db",
        "vectors.bin",
        "vectors.keys",
        "profiles.jsonl",
        "manifest.json",
    ] {
        std::fs::copy(dir.join(name), copy.join(name)).unwrap();
    }
    assert!(manifest.exists());
    assert!(DataSet::load(&copy.join("manifest.json")).is_ok());
}

#[test]
fn h63_a_row_count_that_does_not_fit_the_vectors_is_refused() {
    let dir = folder("shape");
    let manifest = write_sample(&dir);
    rewrite(&manifest, |m| m.row_count += 1);
    assert!(matches!(
        DataSet::load(&manifest),
        Err(SwapError::Manifest(ManifestError::WrongShape(..)))
    ));
}

#[test]
fn h63_an_unknown_schema_and_a_missing_role_are_refused() {
    let dir = folder("schema");
    let manifest = write_sample(&dir);
    rewrite(&manifest, |m| m.schema_version = 99);
    assert!(matches!(
        DataSet::load(&manifest),
        Err(SwapError::Manifest(ManifestError::Schema(99)))
    ));
    let dir = folder("role");
    let manifest = write_sample(&dir);
    rewrite(&manifest, |m| {
        m.files.remove(role::VECTOR_KEYS);
    });
    assert!(matches!(
        DataSet::load(&manifest),
        Err(SwapError::Manifest(ManifestError::MissingRole(_)))
    ));
}

#[test]
fn h69_a_bad_optional_file_is_left_out_and_the_set_loads() {
    let dir = folder("optional");
    let manifest = write_sample(&dir);
    std::fs::write(dir.join("profiles.jsonl"), "changed").unwrap();
    let set = DataSet::load(&manifest).unwrap();
    assert_eq!(set.dropped.len(), 1);
    assert!(set.files.get(role::PROFILES).is_none());
    assert!(!set.profiles.has_profiles());
}

#[test]
fn s151_a_refused_reload_keeps_the_last_good_set() {
    let dir = folder("reload");
    let manifest = write_sample(&dir);
    let slot = DataSlot::open(&manifest).unwrap();
    assert!(!slot.reload().unwrap());
    let hash = slot.load().data_hash.clone();
    rewrite(&manifest, |m| m.dataset_version = String::from("sample-2"));
    std::fs::write(dir.join("vectors.keys"), [0u8; 4]).unwrap();
    assert!(slot.reload().is_err());
    assert_eq!(slot.load().data_hash, hash);
    assert_eq!(slot.load().manifest.dataset_version, "sample-1");
}

#[test]
fn s142_s138_the_index_opens_only_on_its_own_data_set() {
    let dir = folder("index");
    let manifest = write_sample(&dir);
    let set = DataSet::load(&manifest).unwrap();
    let path = dir.join("nearest.index");
    assert!(!set.open_index(&path), "no index yet");
    let row = set.vectors.get_f32(0);
    let exact = set.nearest(&row, 5);
    set.build_index(&path).unwrap();
    assert_eq!(
        set.index.load().as_ref().map(|i| i.len()),
        Some(set.catalog.len())
    );
    let by_index = set.nearest(&row, 5);
    assert_eq!(by_index[0].0, exact[0].0, "both paths rank the same way");
    // Another data set, same files: the stamp names the old manifest hash.
    rewrite(&manifest, |m| m.dataset_version = String::from("sample-2"));
    let other = DataSet::load(&manifest).unwrap();
    assert!(!other.open_index(&path));
    let reopened = DataSet::load(&manifest).unwrap();
    reopened.build_index(&path).unwrap();
    assert!(reopened.open_index(&path));
    // A damaged index file no longer matches its stamp.
    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    let fresh = DataSet::load(&manifest).unwrap();
    assert!(!fresh.open_index(&path));
}

#[test]
fn h70_a_catalogue_with_no_text_index_is_refused_at_load() {
    let dir = folder("no_text_index");
    let manifest = write_sample(&dir);
    let db = dir.join("catalog.db");
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch("DROP TABLE catalog_fts;").unwrap();
    drop(conn);
    rewrite(&manifest, |m| {
        let entry = m.files.get_mut(server_catalog::role::CATALOG).unwrap();
        entry.sha256 = server_catalog::manifest::sha256_file(&db).unwrap();
        entry.bytes = std::fs::metadata(&db).unwrap().len();
    });
    assert!(matches!(DataSet::load(&manifest), Err(SwapError::Load(_))));
}

#[test]
fn s151_the_sample_reads_its_seeds_events_and_fresh_list() {
    let dir = folder("extras");
    let set = DataSet::load(&write_sample(&dir)).unwrap();
    assert_eq!(set.seeds.seeds.len(), 12);
    assert!(!set.fresh.items.is_empty());
    assert_eq!(set.events.all().len(), 2);
    assert!(set.dropped.is_empty());
}

fn signed(dir: &std::path::Path, manifest: &std::path::Path, key: &ed25519_dalek::SigningKey) {
    use base64::Engine;
    use ed25519_dalek::Signer;
    let signature = key.sign(&std::fs::read(manifest).unwrap());
    let text = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    let body = serde_json::json!({ "algorithm": "ed25519", "key_id": "x", "signature": text });
    std::fs::write(dir.join("manifest.sig"), body.to_string()).unwrap();
}

#[test]
fn c3_a_held_key_refuses_an_unsigned_or_wrongly_signed_set() {
    let dir = folder("signed");
    let manifest = write_sample(&dir);
    let key = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
    let trust = DataTrust::from_keys(vec![key.verifying_key().to_bytes()]);
    assert!(matches!(
        DataSet::load_trusted(&manifest, &trust),
        Err(SwapError::Manifest(ManifestError::Signature))
    ));
    signed(&dir, &manifest, &key);
    assert!(DataSet::load_trusted(&manifest, &trust).is_ok());
    let other = ed25519_dalek::SigningKey::from_bytes(&[8; 32]);
    signed(&dir, &manifest, &other);
    assert!(DataSet::load_trusted(&manifest, &trust).is_err());
    signed(&dir, &manifest, &key);
    let text = std::fs::read(&manifest).unwrap();
    std::fs::write(&manifest, [text.as_slice(), b" "].concat()).unwrap();
    assert!(
        DataSet::load_trusted(&manifest, &trust).is_err(),
        "the signature covers the bytes"
    );
}

#[test]
fn c3_a_pem_public_key_reads() {
    let key = ed25519_dalek::SigningKey::from_bytes(&[9; 32]);
    use base64::Engine;
    let mut der = vec![
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ];
    der.extend_from_slice(&key.verifying_key().to_bytes());
    let body = base64::engine::general_purpose::STANDARD.encode(der);
    let pem = format!("-----BEGIN PUBLIC KEY-----\n{body}\n-----END PUBLIC KEY-----\n");
    assert_eq!(
        DataTrust::from_pem(&[pem]).unwrap(),
        DataTrust::from_keys(vec![key.verifying_key().to_bytes()])
    );
}

#[test]
fn h63_a_key_scheme_or_a_path_outside_the_set_is_refused() {
    let dir = folder("scheme");
    let manifest = write_sample(&dir);
    rewrite(&manifest, |m| m.key_scheme = String::from("other"));
    assert!(matches!(
        DataSet::load(&manifest),
        Err(SwapError::Manifest(ManifestError::KeyScheme(_)))
    ));
    let dir = folder("outside");
    let manifest = write_sample(&dir);
    rewrite(&manifest, |m| {
        m.files.get_mut(role::FRESH).unwrap().path = String::from("../secret");
    });
    assert!(matches!(
        DataSet::load(&manifest),
        Err(SwapError::Manifest(ManifestError::OutsideSet(_)))
    ));
}
