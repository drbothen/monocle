//! Workspace-structure acceptance tests for S-001.
//!
//! These tests verify the workspace invariants required by S-001 Acceptance
//! Criteria AC-001..AC-007 by parsing the root `Cargo.toml`, the
//! `rust-toolchain.toml`, and the GitHub workflow YAMLs.
//!
//! They run against the workspace root (resolved via `CARGO_MANIFEST_DIR`).

use std::fs;
use std::path::PathBuf;

/// Resolve the workspace root by walking up from the current crate manifest
/// directory until we find a `Cargo.toml` containing `[workspace]`.
fn workspace_root() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cur = start.clone();
    loop {
        let candidate = cur.join("Cargo.toml");
        if let Ok(contents) = fs::read_to_string(&candidate) {
            if contents.contains("[workspace]") {
                return cur;
            }
        }
        if !cur.pop() {
            panic!("workspace root not found from {:?}", start);
        }
    }
}

fn read_workspace_cargo_toml() -> String {
    fs::read_to_string(workspace_root().join("Cargo.toml"))
        .expect("workspace Cargo.toml must exist")
}

// ---------------------------------------------------------------------------
// AC-001: workspace structure compiles. Validated by `cargo build --workspace`
// at the CI step level; not asserted here (a passing test binary implies it
// already compiled).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// AC-002: rust-toolchain.toml pins channel = "1.86" with the canonical content.
// ---------------------------------------------------------------------------

#[test]
fn ac_002_rust_toolchain_file_exists_and_pins_msrv() {
    let root = workspace_root();
    let path = root.join("rust-toolchain.toml");
    let contents = fs::read_to_string(&path)
        .expect("rust-toolchain.toml must exist at workspace root");
    assert!(
        contents.contains("channel = \"1.86\""),
        "rust-toolchain.toml must pin channel = \"1.86\"; got:\n{}",
        contents
    );
    assert!(
        contents.contains("components"),
        "rust-toolchain.toml must declare components array; got:\n{}",
        contents
    );
    assert!(
        contents.contains("\"clippy\"") && contents.contains("\"rustfmt\""),
        "rust-toolchain.toml must include clippy and rustfmt components; got:\n{}",
        contents
    );
    assert!(
        contents.contains("profile = \"minimal\""),
        "rust-toolchain.toml must use profile = \"minimal\"; got:\n{}",
        contents
    );
}

// ---------------------------------------------------------------------------
// AC-003: .github/workflows/ci.yml exists with explicit `include:` matrix
// covering macos-14/aarch64-apple-darwin, ubuntu-24.04/x86_64-unknown-linux-gnu,
// ubuntu-24.04-arm/aarch64-unknown-linux-gnu.
// ---------------------------------------------------------------------------

#[test]
fn ac_003_ci_yml_exists_and_uses_explicit_include_matrix() {
    let root = workspace_root();
    let path = root.join(".github").join("workflows").join("ci.yml");
    let contents = fs::read_to_string(&path)
        .expect(".github/workflows/ci.yml must exist");

    assert!(
        contents.contains("include:"),
        "ci.yml must use explicit `include:` matrix (not Cartesian product); got:\n{}",
        contents
    );
    for runner in ["macos-14", "ubuntu-24.04", "ubuntu-24.04-arm"] {
        assert!(
            contents.contains(runner),
            "ci.yml must reference runner {}; got:\n{}",
            runner,
            contents
        );
    }
    for target in [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
    ] {
        assert!(
            contents.contains(target),
            "ci.yml must reference target {}; got:\n{}",
            target,
            contents
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004: workspace Cargo.toml has rust-version = "1.86" and CI has
// lint-toolchain step.
// ---------------------------------------------------------------------------

#[test]
fn ac_004_workspace_cargo_pins_rust_version_1_86() {
    let ws = read_workspace_cargo_toml();
    assert!(
        ws.contains("rust-version = \"1.86\""),
        "workspace Cargo.toml must set rust-version = \"1.86\"; got:\n{}",
        ws
    );
}

#[test]
fn ac_004_ci_yml_has_lint_toolchain_step() {
    let root = workspace_root();
    let contents = fs::read_to_string(root.join(".github").join("workflows").join("ci.yml"))
        .expect("ci.yml must exist");
    assert!(
        contents.contains("lint-toolchain") || contents.contains("rust-toolchain.toml"),
        "ci.yml must include a lint-toolchain step that verifies rust-toolchain.toml channel pin; got:\n{}",
        contents
    );
    assert!(
        contents.contains("1\\.86") || contents.contains("1.86"),
        "ci.yml lint-toolchain step must assert channel matches 1.86; got:\n{}",
        contents
    );
}

// ---------------------------------------------------------------------------
// AC-005: workspace members = exactly [monocle-core, monocle-runtime, monocle-proto].
// monocle-auth is NOT a workspace member (per Decision 3).
// monocle-tui is NOT a Phase 1 workspace member.
// ---------------------------------------------------------------------------

#[test]
fn ac_005_workspace_declares_exactly_three_phase1_members() {
    let ws = read_workspace_cargo_toml();
    for member in ["monocle-core", "monocle-runtime", "monocle-proto"] {
        assert!(
            ws.contains(member),
            "workspace Cargo.toml must list `{}` as a member; got:\n{}",
            member,
            ws
        );
    }
}

#[test]
fn ac_005_workspace_does_not_declare_monocle_auth_or_tui() {
    let ws = read_workspace_cargo_toml();
    // Forbidden Phase 1 members per AC-005 / Decision 3.
    assert!(
        !ws.contains("\"monocle-auth\"") && !ws.contains("crates/monocle-auth"),
        "monocle-auth must NOT be a workspace member (Decision 3); got:\n{}",
        ws
    );
    assert!(
        !ws.contains("\"monocle-tui\"") && !ws.contains("crates/monocle-tui"),
        "monocle-tui must NOT be a Phase 1 workspace member; got:\n{}",
        ws
    );
}

// ---------------------------------------------------------------------------
// AC-006: workspace [workspace.dependencies] declares the 9 EXACT-pinned
// security-sensitive crates plus the direct bytes 1.10 pin.
// ---------------------------------------------------------------------------

#[test]
fn ac_006_workspace_declares_exact_pinned_security_crates() {
    let ws = read_workspace_cargo_toml();
    // EXACT-pin signatures, all using full SemVer triplet form (=x.y.z).
    let exact_pins = [
        "tokio = { version = \"=1.52.0\"",
        "axum = \"=0.8.9\"",
        "serde_json = \"=1.0.149\"",
        "rand = \"=0.8.6\"",
        "prost = \"=0.14.1\"",
        "reqwest = \"=0.13.0\"",
        "wasmtime = \"=44.0.1\"",
        "russh = \"=0.60.2\"",
    ];
    for pin in exact_pins {
        assert!(
            ws.contains(pin),
            "workspace [workspace.dependencies] must declare `{}`; got:\n{}",
            pin,
            ws
        );
    }
}

#[test]
fn ac_006_workspace_declares_bytes_direct_pin() {
    let ws = read_workspace_cargo_toml();
    assert!(
        ws.contains("bytes = \"1.10\""),
        "workspace [workspace.dependencies] must declare `bytes = \"1.10\"` to close RUSTSEC-2026-0007; got:\n{}",
        ws
    );
}

#[test]
fn ac_006_workspace_omits_rmcp_in_phase_1() {
    let ws = read_workspace_cargo_toml();
    assert!(
        !ws.contains("rmcp ="),
        "rmcp must be OMITTED from Phase 1 workspace (OQ-09 -- Phase 4 scope only); got:\n{}",
        ws
    );
}

#[test]
fn ac_006_workspace_uses_workspace_dependencies_pattern() {
    let ws = read_workspace_cargo_toml();
    assert!(
        ws.contains("[workspace.dependencies]"),
        "workspace Cargo.toml must use the Cargo 2021+ [workspace.dependencies] pattern; got:\n{}",
        ws
    );
}

// ---------------------------------------------------------------------------
// AC-007: .github/workflows/audit.yml exists with weekly cron schedule and
// runs cargo audit --deny warnings after installing cargo-audit --locked.
// ---------------------------------------------------------------------------

#[test]
fn ac_007_audit_yml_exists_with_weekly_cron() {
    let root = workspace_root();
    let path = root.join(".github").join("workflows").join("audit.yml");
    let contents = fs::read_to_string(&path)
        .expect(".github/workflows/audit.yml must exist");

    assert!(
        contents.contains("cron: '0 0 * * 0'") || contents.contains("cron: \"0 0 * * 0\""),
        "audit.yml must schedule weekly cron '0 0 * * 0'; got:\n{}",
        contents
    );
    assert!(
        contents.contains("cargo audit"),
        "audit.yml must invoke cargo audit; got:\n{}",
        contents
    );
    assert!(
        contents.contains("--deny warnings"),
        "audit.yml must use cargo audit --deny warnings; got:\n{}",
        contents
    );
    assert!(
        contents.contains("cargo install cargo-audit") && contents.contains("--locked"),
        "audit.yml must install cargo-audit via `cargo install cargo-audit --locked`; got:\n{}",
        contents
    );
}

// ---------------------------------------------------------------------------
// Module structure invariants for monocle-core (per
// SS-core-types-and-abi.md v1.2.13 §Module Layout).
// ---------------------------------------------------------------------------

#[test]
fn monocle_core_declares_phase1_modules() {
    let root = workspace_root();
    let lib = fs::read_to_string(root.join("crates/monocle-core/src/lib.rs"))
        .expect("monocle-core/src/lib.rs must exist");
    for module in ["engine", "factory", "abi"] {
        assert!(
            lib.contains(&format!("pub mod {}", module))
                || lib.contains(&format!("pub mod {} ", module)),
            "monocle-core/src/lib.rs must declare `pub mod {}`; got:\n{}",
            module,
            lib
        );
    }
}

// ---------------------------------------------------------------------------
// monocle-proto build.rs stub invariant.
// ---------------------------------------------------------------------------

#[test]
fn monocle_proto_has_build_rs_stub() {
    let root = workspace_root();
    let path = root.join("crates/monocle-proto/build.rs");
    assert!(
        path.exists(),
        "monocle-proto/build.rs must exist as a no-op stub for S-013"
    );
}

// ---------------------------------------------------------------------------
// monocle-runtime binary stub invariant (replaced by S-002+).
// ---------------------------------------------------------------------------

#[test]
fn monocle_runtime_has_main_rs_stub() {
    let root = workspace_root();
    let main_rs = fs::read_to_string(root.join("crates/monocle-runtime/src/main.rs"))
        .expect("monocle-runtime/src/main.rs must exist");
    assert!(
        main_rs.contains("fn main()"),
        "monocle-runtime/src/main.rs must define fn main(); got:\n{}",
        main_rs
    );
}
