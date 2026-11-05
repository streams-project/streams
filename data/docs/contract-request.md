# What this repository needs from the contract

The manifest shape belongs in `platform-contract` (note 152, correction 12). The contract does not
hold it yet, so this repository ships the shape in `src/databuild/manifest/`. The server has its own
interim type in `server-catalog`. Both agree today, and a test here reads the server's type to keep
it that way.

## Asked of the contract

Add these wire types, and the generator then writes `generated/schemas/DatasetManifest.json`:

```rust
pub struct FileEntry { path: String, sha256: String, bytes: u64, required: bool }

pub struct OpenSet { model: String, dimensions: u32, recipe: String, row_count: u64 }

pub struct DatasetManifest {
    schema_version: u32,
    dataset_version: String,
    build_commit: String,
    built_at_ms: u64,
    model: String,
    dimensions: u32,
    quantisation: String,
    row_count: u64,
    key_scheme: String,
    recipe: Option<String>,
    source: Option<String>,
    as_of: Option<String>,
    open_set: Option<OpenSet>,
    files: BTreeMap<String, FileEntry>,
}

pub struct DatasetSignature { algorithm: String, key_id: String, signature: String }
```

Once they exist, set `contract.schema_dir` to the contract's `generated/schemas` folder and delete
the two JSON files shipped here.

`built_at_ms` is when the export ran. A daily job rewrites the manifest around a new daily file and
keeps this value, so data age never hides an old catalogue.

## Asked of the server

- Read `manifest.sig` beside `manifest.json`. It is a detached ed25519 signature over the exact
  bytes of `manifest.json`, with a key id. `server-catalog` checks no signature today.
- Keep the accepted public keys in its own settings. A set with no signature is unsigned, and the
  server decides what to do with it.
- Read these roles when wanted: `open_vectors`, `open_vector_keys`, `skip_marks`,
  `content_reasons`, `cards`, `series_text`, `series_credit`, `sources`.
- Take cards from `cards.tar`, unpacked under the image host. Paths in the catalogue start with
  `/poster/` and `/backdrop/`.
