//! Binary entry point for the DTU clone: `dtu-claude-code-hooks-v1`.
// Stub binary: `main` body is `todo!()` per TDD Red Gate.
// The `todo` lint is suppressed at file scope; this allow is intentional and
// will be removed when the binary is implemented in the TDD implementation step.
#![allow(clippy::todo)]
//!
//! Reads the monocle daemon lock file, resolves the connection details, and starts
//! the axum HTTP server that synthesizes hook POSTs for all 5 endpoints.
//!
//! Built artifact: `target/[debug|release]/dtu-claude-code-hooks-v1`
//! Source: `crates/monocle-test-harness/src/bin/dtu_server.rs`
//!
//! Source authority:
//! - S-DTU-001 AC-005 (Rust binary form; `cargo build --bin dtu-claude-code-hooks-v1`)
//! - S-DTU-001 AC-006 (MONOCLE_HOOK_ENDPOINT_BASE, MONOCLE_NO_AUTOSTART env vars)
//! - dtu-assessment.md v1.7.5 §Packaging Decision (lines 320-343)
//! - BC-HOOK-013 (lock file scan), BC-HOOK-015 (token), BC-HOOK-016 (auth header)

#![forbid(unsafe_code)]

use anyhow::Result;

fn main() -> Result<()> {
    todo!("S-DTU-001 implementation pending; AC-005, AC-006, BC-HOOK-013, BC-HOOK-015")
}
