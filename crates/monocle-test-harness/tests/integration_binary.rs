//! Integration tests for the DTU clone binary — CRIT-1 (adversary R1).
//!
//! Verifies:
//! - `cargo build --bin dtu-claude-code-hooks-v1` succeeds (binary compiles)
//! - The binary does not panic on `--help` or equivalent invocation (main() implemented)
//! - AC-005: binary artifact produced in target/
//!
//!
//! Source authority:
//! - S-DTU-001 AC-005 (`cargo build --bin dtu-claude-code-hooks-v1`)
//! - S-DTU-001 AC-006 (MONOCLE_HOOK_ENDPOINT_BASE, MONOCLE_NO_AUTOSTART env vars)
//! - dtu-assessment.md §Packaging Decision (implemented against v1.7.5 at S-DTU-001 authoring time)

// BC-based test function names use BC_S_SS_NNN_ uppercase pattern per factory convention.
// The non_snake_case lint is suppressed at file level to preserve traceability.
#![allow(non_snake_case)]

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn workspace_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR = crates/monocle-test-harness; workspace root is 2 up.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap_or_else(|| panic!("crates/ parent missing"))
        .parent()
        .unwrap_or_else(|| panic!("workspace root parent missing"))
        .to_path_buf()
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-005: Binary compiles
// ──────────────────────────────────────────────────────────────────────────────

/// AC-005: `cargo build --bin dtu-claude-code-hooks-v1` succeeds.
///
/// This test verifies the binary artifact can be compiled from the workspace.
/// If the binary has syntax errors or fails to compile, this test fails.
///
/// Note: compilation is already verified by `cargo test --workspace --locked`.
/// This test explicitly invokes cargo build to produce the binary artifact.
#[test]
fn test_BC_HOOK_001_binary_compiles_via_cargo_build() {
    let root = workspace_root();
    let output = std::process::Command::new("cargo")
        .args(["build", "--bin", "dtu-claude-code-hooks-v1", "--locked"])
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("failed to invoke cargo build: {e}"));

    assert!(
        output.status.success(),
        "cargo build --bin dtu-claude-code-hooks-v1 must succeed; \
         exit={}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-005 / AC-006: Binary does not panic on startup
// ──────────────────────────────────────────────────────────────────────────────

/// AC-005 / AC-006: The `dtu-claude-code-hooks-v1` binary exits 0 on `--help`.
///
/// Verifies that main() is implemented and the binary accepts `--help` without panicking.
///
/// test_BC_HOOK_001_binary_entrypoint_no_panic
#[test]
fn test_BC_HOOK_001_binary_entrypoint_no_panic_on_help() {
    let root = workspace_root();

    // First build the binary (may already be built, cargo is idempotent).
    let build_output = std::process::Command::new("cargo")
        .args(["build", "--bin", "dtu-claude-code-hooks-v1", "--locked"])
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("cargo build failed to execute: {e}"));
    assert!(
        build_output.status.success(),
        "binary must compile before we can run it; stderr={}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let binary_path = root
        .join("target")
        .join("debug")
        .join("dtu-claude-code-hooks-v1");

    assert!(
        binary_path.exists(),
        "binary must exist at {binary_path:?} after cargo build"
    );

    // Run binary with MONOCLE_NO_AUTOSTART=1 to prevent actual daemon connection attempts,
    // plus --help to verify the CLI is responsive.
    // Per AC-006: MONOCLE_NO_AUTOSTART must cause the binary to exit cleanly without connecting.
    let run_output = std::process::Command::new(&binary_path)
        .arg("--help")
        .env("MONOCLE_NO_AUTOSTART", "1")
        .output()
        .unwrap_or_else(|e| panic!("failed to run binary: {e}"));

    assert!(
        run_output.status.success(),
        "dtu-claude-code-hooks-v1 --help must exit 0 (AC-005); \
         exit={}, stderr={}",
        run_output.status,
        String::from_utf8_lossy(&run_output.stderr)
    );
}

/// AC-006: MONOCLE_NO_AUTOSTART env var causes binary to exit 0 without starting a server.
///
/// When MONOCLE_NO_AUTOSTART=1, the binary must exit cleanly (used in CI/test contexts
/// where no daemon is available).
#[test]
fn test_BC_HOOK_001_binary_monocle_no_autostart_exits_cleanly() {
    let root = workspace_root();
    let binary_path = root
        .join("target")
        .join("debug")
        .join("dtu-claude-code-hooks-v1");

    // If binary doesn't exist yet, build it first.
    if !binary_path.exists() {
        let build = std::process::Command::new("cargo")
            .args(["build", "--bin", "dtu-claude-code-hooks-v1", "--locked"])
            .current_dir(&root)
            .output()
            .unwrap_or_else(|e| panic!("cargo build failed: {e}"));
        assert!(
            build.status.success(),
            "binary must compile; stderr={}",
            String::from_utf8_lossy(&build.stderr)
        );
    }

    let run_output = std::process::Command::new(&binary_path)
        .env("MONOCLE_NO_AUTOSTART", "1")
        .output()
        .unwrap_or_else(|e| panic!("failed to run binary: {e}"));

    assert!(
        run_output.status.success(),
        "MONOCLE_NO_AUTOSTART=1 must cause binary to exit 0 without connecting to daemon; \
         exit={}, stderr={}",
        run_output.status,
        String::from_utf8_lossy(&run_output.stderr)
    );
}
