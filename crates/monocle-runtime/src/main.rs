//! monocle-runtime binary entry. Stub from S-001; daemon wired in S-002.
//!
//! S-002+ will replace this with full daemon entry point: argument parsing via
//! clap, daemon lifecycle from SS-daemon-lifecycle.md, axum router init.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

// Compile-time ABI drift guard (VP-011 §Mechanism, BC-2.02.001 PC-2).
//
// If `monocle_core::MONOCLE_ABI_VERSION` is ever incremented without a corresponding
// update to the `/status` handler and integration tests, this assertion will catch the
// drift at compile time, before any runtime test runs. The expected value `1` identifies
// Phase 1 of the monocle ABI. S-010 is the canonical producer; S-003 exposes it in /status.
const _: () = assert!(
    monocle_core::MONOCLE_ABI_VERSION == 1,
    "ABI version mismatch: monocle_core::MONOCLE_ABI_VERSION changed without updating \
    the /status handler and the compile-time drift guard. Update this assertion and the \
    integration tests in status_abi_version.rs to the new ABI version."
);

fn main() {
    // Intentional no-op stub. S-002 will wire the daemon entry point.
}
