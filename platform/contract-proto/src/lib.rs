//! The contract as a protobuf schema, with the validator generated from the same
//! field options that every other language's checks come from.
//!
//! `no_std` with `alloc`, which is the shape the core is in
//! (platform/core/src/lib.rs:5, and an in house global allocator at
//! platform/bridges/wasm/src/heap.rs), so "does prost build for the core" is
//! answered by this file compiling for `wasm32v1-none`.
//!
//! What this crate is NOT: the core's domain types. `TitleKey` keeps its private
//! fields and its fallible constructor in platform/contract/src/title.rs, and the
//! `TryFrom` conversions between these generated structs and those types are note
//! 204's P2, in platform/contract/src/convert.rs. These types are the wire, and
//! nothing else.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod contract {
    //! Generated from proto/contract/v1/*.proto by build.rs, with protox. Checked
    //! in under generated/.
    include!("../generated/contract.v1.rs");
}

pub mod rules;

// Generated code is not hand tidied, and these two lints are the whole reason this
// attribute exists. `single_match` fires on the five `match &m.field { None =>
// return Err(..), Some(_) => {} }` bodies and `collapsible_if` on the bounded
// `if let Some(value) ... { if ... }` bodies. Both are the generator emitting ONE
// shape per field kind, which is what makes 2,872 lines readable and reviewable;
// writing a special case into gen/src/main.rs to please a style lint would also
// move every byte figure in README.md, each of which was measured against this
// exact file. Silenced at the boundary, where generated code belongs.
#[allow(clippy::single_match, clippy::collapsible_if)]
pub mod validate {
    //! Generated from the field options by `cargo run -p contract-validate-gen --
    //! write`. Checked in under generated/.
    include!("../generated/validate.rs");
}
