//! monocle library — pub re-exports for integration testing.
//!
//! This `[lib]` target exists solely so that integration tests in `tests/` can
//! import pub items from modules like `auto_start` without going through the
//! binary entry point (`main.rs`). All real logic lives in the modules declared
//! here; `main.rs` only parses CLI args and dispatches.
//!
//! # Module visibility policy
//!
//! Modules declared here are `pub` for testability. They are NOT intended to be
//! used by other crates — the `monocle` binary crate is a leaf node in the
//! workspace dependency graph. The `[lib]` target is a test-infrastructure
//! artifact only.

#![deny(missing_docs)]
#![forbid(unsafe_code)]
// Allow test-pattern lints in library since this lib exists for test surface only.
// The actual warnings live in main.rs and the individual module files.
#![allow(clippy::module_inception)]

/// Daemon auto-start decision sequence (BC-2.04.002, BC-2.04.003).
pub mod auto_start;
