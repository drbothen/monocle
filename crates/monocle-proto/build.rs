//! monocle-proto build script.
//!
//! Compiles `proto/monocle/v1/hook_envelope.proto` via prost-build, generating
//! Rust types at compile time. Also emits `file_descriptor_set.bin` into OUT_DIR
//! so that `tests/wire_field_order.rs` can decode it via prost-reflect to assert
//! the field number immutability contract (VP-016 / BC-2.02.006 / FC-05).
//!
//! Phase 1: types are compiled into the binary but no wire path is activated at
//! runtime. Phase 4 activates the wire path without any Phase 4 workspace changes
//! to `monocle-proto`.

// Build scripts have no caller to propagate errors to; panicking on failure is
// the only correct behavior here.
#![allow(clippy::expect_used)]

use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR must be set by cargo"));
    let descriptor_path = out_dir.join("file_descriptor_set.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["proto/monocle/v1/hook_envelope.proto"], &["proto/"])
        .expect("prost-build codegen failed");

    // Emit cargo rerun directives so the build script re-runs when proto files change.
    println!("cargo:rerun-if-changed=proto/monocle/v1/hook_envelope.proto");
    println!("cargo:rerun-if-changed=build.rs");
}
