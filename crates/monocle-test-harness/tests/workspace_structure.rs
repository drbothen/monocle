//! Workspace structure tests — CRIT-2 and CRIT-3 (adversary R1).
//!
//! Verifies:
//! - CRIT-2: `xtask` crate exists at `xtask/` and is a workspace member
//! - CRIT-3: `.github/workflows/dtu-fidelity.yml` exists and has expected structure
//!
//!
//! Source authority:
//! - S-DTU-001 AC-007 (`cargo xtask dtu-fidelity` oracle)
//! - dtu-assessment.md v1.7.5 §Xtask Integration
//! - S-001 (CI/DevOps setup — creates .github/workflows/)

// BC-based test function names use BC_S_SS_NNN_ uppercase pattern per factory convention.
// The non_snake_case lint is suppressed at file level to preserve traceability.
#![allow(non_snake_case)]

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn workspace_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap_or_else(|| panic!("crates/ parent missing"))
        .parent()
        .unwrap_or_else(|| panic!("workspace root parent missing"))
        .to_path_buf()
}

// ──────────────────────────────────────────────────────────────────────────────
// CRIT-2: xtask crate existence and workspace membership
// ──────────────────────────────────────────────────────────────────────────────

/// CRIT-2: `xtask/` directory exists at workspace root.
///
/// The `cargo xtask dtu-fidelity` oracle requires an `xtask` crate.
///
/// test_workspace_xtask_crate_exists
#[test]
fn test_workspace_xtask_crate_exists() {
    let root = workspace_root();
    let xtask_dir = root.join("xtask");
    assert!(
        xtask_dir.exists() && xtask_dir.is_dir(),
        "xtask/ directory must exist at workspace root {root:?} — \
         devops-engineer must create it (S-DTU-001 AC-007)"
    );
}

/// CRIT-2: `xtask/Cargo.toml` exists.
#[test]
fn test_workspace_xtask_cargo_toml_exists() {
    let root = workspace_root();
    let xtask_toml = root.join("xtask").join("Cargo.toml");
    assert!(
        xtask_toml.exists(),
        "xtask/Cargo.toml must exist — devops-engineer must create xtask crate"
    );
}

/// CRIT-2: `xtask` is listed in `[workspace.members]` in the root Cargo.toml.
///
/// Per Cargo workspace rules, xtask must be a member for `cargo run -p xtask` to work.
#[test]
fn test_workspace_xtask_is_workspace_member() {
    let root = workspace_root();
    let cargo_toml_path = root.join("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|e| panic!("read workspace Cargo.toml: {e}"));

    assert!(
        cargo_toml_content.contains("\"xtask\"") || cargo_toml_content.contains("'xtask'"),
        "xtask must be listed in [workspace.members] in Cargo.toml; \
         current content does not contain xtask member entry.\n\
         devops-engineer must add xtask to workspace."
    );
}

/// CRIT-2: `cargo run -p xtask -- dtu-fidelity` subcommand is recognized.
///
/// The xtask binary must accept a `dtu-fidelity` subcommand and exit 0
/// when all fidelity tests pass (or exit non-zero on failure, but must not panic).
#[test]
fn test_workspace_xtask_dtu_fidelity_subcommand_exists() {
    let root = workspace_root();

    // First verify the crate exists (guard to give useful error).
    let xtask_dir = root.join("xtask");
    if !xtask_dir.exists() {
        panic!(
            "xtask/ crate does not exist — devops-engineer must create it before this test can run"
        );
    }

    // Attempt `cargo run -p xtask -- dtu-fidelity --help`.
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--locked",
            "-p",
            "xtask",
            "--",
            "dtu-fidelity",
            "--help",
        ])
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("invoke cargo run -p xtask: {e}"));

    assert!(
        output.status.success(),
        "`cargo run -p xtask -- dtu-fidelity --help` must exit 0; \
         exit={}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// CRIT-3: .github/workflows/dtu-fidelity.yml existence and structure
// ──────────────────────────────────────────────────────────────────────────────

/// CRIT-3: `.github/workflows/dtu-fidelity.yml` exists.
///
/// test_workflow_dtu_fidelity_yml_exists
#[test]
fn test_workflow_dtu_fidelity_yml_exists() {
    let root = workspace_root();
    let workflow_path = root
        .join(".github")
        .join("workflows")
        .join("dtu-fidelity.yml");

    assert!(
        workflow_path.exists(),
        ".github/workflows/dtu-fidelity.yml must exist — \
         devops-engineer must create it (S-001 CI/DevOps setup)"
    );
}

/// CRIT-3: `dtu-fidelity.yml` contains a `jobs:` key with at least one job.
#[test]
fn test_workflow_dtu_fidelity_yml_has_jobs() {
    let root = workspace_root();
    let workflow_path = root
        .join(".github")
        .join("workflows")
        .join("dtu-fidelity.yml");

    if !workflow_path.exists() {
        panic!(".github/workflows/dtu-fidelity.yml does not exist — cannot check jobs");
    }

    let content = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read dtu-fidelity.yml: {e}"));

    assert!(
        content.contains("jobs:"),
        "dtu-fidelity.yml must contain a `jobs:` key; got:\n{content}"
    );
}

/// CRIT-3: `dtu-fidelity.yml` triggers on pull_request events.
///
/// The workflow must run on PRs to catch fidelity regressions before merge.
#[test]
fn test_workflow_dtu_fidelity_yml_triggers_on_pull_request() {
    let root = workspace_root();
    let workflow_path = root
        .join(".github")
        .join("workflows")
        .join("dtu-fidelity.yml");

    if !workflow_path.exists() {
        panic!(".github/workflows/dtu-fidelity.yml does not exist");
    }

    let content = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read dtu-fidelity.yml: {e}"));

    assert!(
        content.contains("pull_request"),
        "dtu-fidelity.yml must trigger on pull_request events; got:\n{content}"
    );
}

/// CRIT-3: `dtu-fidelity.yml` invokes `cargo xtask dtu-fidelity` or equivalent.
///
/// The workflow must run the dtu-fidelity oracle to validate fidelity scores.
#[test]
fn test_workflow_dtu_fidelity_yml_invokes_xtask_dtu_fidelity() {
    let root = workspace_root();
    let workflow_path = root
        .join(".github")
        .join("workflows")
        .join("dtu-fidelity.yml");

    if !workflow_path.exists() {
        panic!(".github/workflows/dtu-fidelity.yml does not exist");
    }

    let content = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read dtu-fidelity.yml: {e}"));

    assert!(
        content.contains("dtu-fidelity"),
        "dtu-fidelity.yml must invoke `dtu-fidelity` (via cargo xtask or cargo test); \
         got:\n{content}"
    );
}
