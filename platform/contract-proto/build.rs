//! Compiles the schema to Rust and keeps the descriptor on disk.
//!
//! No `protoc`. protox is a protobuf front end written in Rust, so the schema
//! compiles with nothing installed but cargo. That is one fewer tool than this
//! migration was assumed to add, and it is why the Rust half of this package can be
//! proven on a machine with no protobuf toolchain at all.
//!
//! `btree_map(["."])` is the whole answer to the determinism question. prost's
//! default map type is `HashMap`, whose iteration order is randomised per process,
//! and platform/kit/core/tests/boundary.rs compares transcript BYTES across three
//! host languages. With this line every map field is a `BTreeMap`, which is what
//! the contract uses today.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("generated");
    std::fs::create_dir_all(&out)?;

    // One list, read by this file and by gen/, so a file cannot be in the contract
    // for one of them and not the other.
    let text = std::fs::read_to_string(root.join("proto/FILES"))?;
    let files: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    // The Compiler rather than the one line `protox::compile`, because only this
    // path keeps the CUSTOM OPTIONS. `prost_types::FieldOptions` is a prost struct
    // and prost drops unknown fields on decode, so a FileDescriptorSet that has
    // been through it has lost every `(contract.constraints.v1.field)` in the
    // schema. Measured the hard way: the generator read 638 fields and found 0
    // rules. `encode_file_descriptor_set` writes the bytes the options are in.
    let mut compiler = protox::Compiler::new([root.join("proto")])?;
    compiler.include_source_info(true).include_imports(true);
    compiler.open_files(&files)?;

    // The descriptor set, on disk and checked in. Three things read it: the
    // validator generator (gen/), the ledger tie check (tools/check_proto_tie.py)
    // and `buf breaking`, which compares this file across commits.
    std::fs::write(
        out.join("contract.descriptor.binpb"),
        compiler.encode_file_descriptor_set(),
    )?;
    let fds = compiler.file_descriptor_set();

    // prost-build writes one file per PACKAGE, and the set carries two packages
    // besides ours: google.protobuf, because descriptor.proto is imported to extend
    // FieldOptions, and contract.constraints.v1, the options themselves. Neither is
    // a wire type and neither may appear in the module the core links, so they are
    // written to the build directory and only contract.v1.rs is copied out.
    let scratch = PathBuf::from(std::env::var("OUT_DIR")?);
    let mut config = prost_build::Config::new();
    config.out_dir(&scratch);
    config.btree_map(["."]);
    config.compile_fds(fds)?;
    std::fs::copy(scratch.join("contract.v1.rs"), out.join("contract.v1.rs"))?;

    println!("cargo:rerun-if-changed=proto/FILES");
    for file in &files {
        println!("cargo:rerun-if-changed=proto/{file}");
    }
    println!("cargo:rerun-if-changed=proto/contract/constraints/v1/constraints.proto");
    Ok(())
}
