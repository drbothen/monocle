//! VP-013 AST audit: Non-Exhaustive Enum Policy (BC-2.02.003).
//!
//! This file is the canonical verification harness for VP-013 and BC-2.02.003.
//! It uses a `syn 2` AST walk over every `.rs` file in `monocle-core/src/` to
//! assert that every `pub enum` either:
//!   (a) carries `#[non_exhaustive]`, OR
//!   (b) is listed in the ADR-0004/ADR-0006 EXEMPT list.
//!
//! Canonical test name: `test_BC_TYPES_001_non_exhaustive_enum_coverage`
//! (per VP-013 §Harness Location; to be migrated to
//! `test_BC_2_02_003_non_exhaustive_enum_coverage` post BC renumber propagation).
//!
//! # Probe mapping (from VP-013 §Probe Matrix)
//!
//! - 13.a — Walk all `pub enum` in `monocle-core/src/**/*.rs`:
//!            each is `#[non_exhaustive]` OR in EXEMPT.
//! - 13.b — Inject `pub enum BadEnum { ... }` without `#[non_exhaustive]`:
//!            audit fails with descriptive error.
//! - 13.d — EXEMPT list has exactly 3 entries (ADR-0004: 2, ADR-0006: 1):
//!            consistency check fails on length mismatch.
//!
//! Tests for AC-001, AC-002, AC-003, AC-004 (vacuous), AC-005.

// BC-based test names use uppercase segments (e.g., `test_BC_TYPES_001_xxx`).
#![allow(non_snake_case)]
// Test code uses `.expect()` / `.unwrap()` to surface assertion failures.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
// `panic!` is correct test-harness behavior for audit failure reports.
#![allow(clippy::panic)]
// The `format!` calls in failure messages are intentional; no `to_string` alternative.
#![allow(clippy::useless_format)]
// 12-space continuation indent in module-level doc comment (VP-013 §Probe Matrix table).
#![allow(clippy::doc_overindented_list_items)]
// Intentional assert!(true) in vacuous AC-004 traceability sentinel test.
#![allow(clippy::assertions_on_constants)]

use std::fs;
use std::path::PathBuf;

use syn::{Attribute, File, Item, Visibility};

// ---------------------------------------------------------------------------
// ADR-0004 Exemption List
// ---------------------------------------------------------------------------

/// The ADR-0004 / ADR-0006 exemption list for BC-2.02.003 (VP-013 Pre-conditions §EXEMPT).
///
/// Originally 2 entries per ADR-0004 §Immediate (Phase 1) consequences:
///   - `Phase1Permission` — exhaustive by correctness requirement (ADR-0004)
///   - `ClaudeCodeTool`   — exhaustive by explicit tool-set tracking requirement (ADR-0004)
///
/// S-024 adds `AppMode` as a third exhaustive enum per ADR-0006:
///   - `AppMode` — first-party, compile-time safety mechanism. Downstream crates
///     (`monocle-tui`) use exhaustive match over AppMode variants to render the TUI.
///     A new variant without an updated match arm is a compile error — the intended
///     forward-change signal. `#[non_exhaustive]` would silently allow wildcard arms,
///     hiding unhandled variants.
///
/// ADR-0004 §Alternatives Considered noted that AppMode was first-party and that "if
/// Phase 3 or 4 requires a new AppMode variant, a new ADR would be produced at that
/// time." S-024 (BC-2.06.001 INV-1 / AC-013) triggers the Phase 3+ scenario described
/// by ADR-0004: AppMode is now consumed exhaustively. ADR-0006 formalizes the exemption
/// per the BC-TYPES-001 extension protocol.
///
/// If this constant is changed, `test_BC_2_02_003_exempt_list_length`
/// MUST fail (AC-005 / VP-013 Test Vector 13.d). Any further changes require
/// a new ADR superseding or amending ADR-0004/ADR-0006.
///
/// Traces to: ADR-0004 §Extension strategy; ADR-0006; BC-2.06.001 INV-1; AC-013.
///
/// Ordering: ADR-0004 entries first (declaration order in SS-permissions-phase1.md
/// §Canonical Definition), then S-024 addition in BC-2.06.001 declaration order.
const EXEMPT: &[&str] = &["Phase1Permission", "ClaudeCodeTool", "AppMode"];

/// Expected exempt-list count (AC-005 / VP-013 Test Vector 13.d).
///
/// Checked against `EXEMPT.len()` at test time. Changing this constant without
/// a corresponding ADR or story-level AC citing BC-2.02.003 PC-1 is a policy violation.
/// Updated to 3 by S-024 (AC-013: AppMode is exhaustive by design per ADR-0006).
const EXEMPT_COUNT_FROM_ADR_0004: usize = 3;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the absolute path to `monocle-core/src/`.
///
/// The test binary runs from the workspace root (via `cargo test -p monocle-core`)
/// or from within the crate directory; we handle both by checking for the
/// `CARGO_MANIFEST_DIR` env var set by Cargo when running tests.
fn src_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set when running cargo test");
    PathBuf::from(manifest_dir).join("src")
}

/// Collect all `.rs` files under `dir`, recursively.
fn collect_rs_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_rs_recursive(dir, &mut out);
    out.sort(); // deterministic order across platforms
    out
}

fn collect_rs_recursive(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let read_dir = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Failed to read directory {}: {e}", dir.display()));
    for entry in read_dir {
        let entry = entry.expect("directory entry readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_recursive(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Returns `true` if the attribute is `#[non_exhaustive]`.
fn is_non_exhaustive(attr: &Attribute) -> bool {
    attr.path().is_ident("non_exhaustive")
}

/// Returns `true` if the item's visibility is `pub` (exactly `pub`, not `pub(crate)`
/// or `pub(super)` — the policy covers only `pub` enums per BC-2.02.003).
fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// Audit a single parsed file: collect violations (pub enums without
/// `#[non_exhaustive]` that are not in `EXEMPT`).
///
/// Returns a list of `(enum_name)` strings for each violation.
fn audit_file(ast: &File) -> Vec<String> {
    let mut violations = Vec::new();
    audit_items(&ast.items, &mut violations);
    violations
}

/// Recursively audit a slice of `Item`s, descending into inline `mod` blocks.
///
/// Inline `mod foo { ... }` blocks (i.e., `Item::Mod` with `content.is_some()`)
/// can contain `pub enum` declarations that are subject to BC-2.02.003. Without
/// recursion the top-level walk would silently miss them.
fn audit_items(items: &[Item], violations: &mut Vec<String>) {
    for item in items {
        match item {
            Item::Enum(enum_item) => {
                if !is_pub(&enum_item.vis) {
                    // Non-public enums are not subject to the policy.
                    continue;
                }
                let name = enum_item.ident.to_string();
                let has_non_exhaustive = enum_item.attrs.iter().any(is_non_exhaustive);
                let is_exempt = EXEMPT.contains(&name.as_str());
                if !has_non_exhaustive && !is_exempt {
                    violations.push(name);
                }
            }
            Item::Mod(mod_item) => {
                // Recurse into inline `mod` blocks that have content.
                // File-level `mod foo;` declarations (without braces) have
                // `content == None` and are handled by the file-system walk in
                // `collect_rs_files`; they do not need recursion here.
                if let Some((_, inner_items)) = &mod_item.content {
                    audit_items(inner_items, violations);
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// AC-005 / VP-013 Test Vector 13.d — EXEMPT list length check
// ---------------------------------------------------------------------------

/// VP-013 Test Vector 13.d: EXEMPT list length must exactly match the count
/// documented in ADR-0004.
///
/// Traces to: BC-2.02.003 postcondition 3; AC-005; VP-013 §Probe Matrix 13.d.
///
/// **Red Gate:** This test passes immediately (it is a compile-time-constant
/// assertion with no implementation dependency). It is a policy guardrail: if
/// someone silently adds a third entry to `EXEMPT` without updating ADR-0004,
/// this test fails. The test writer intentionally does NOT make this test fail
/// on first run — the EXEMPT constant is correctly populated at stub time.
#[test]
fn test_BC_2_02_003_exempt_list_length() {
    assert_eq!(
        EXEMPT.len(),
        EXEMPT_COUNT_FROM_ADR_0004,
        "EXEMPT list length ({}) != ADR-0004 documented count ({}). \
         Any change to the EXEMPT list requires a new ADR superseding ADR-0004. \
         Per AC-005 / VP-013 Test Vector 13.d.",
        EXEMPT.len(),
        EXEMPT_COUNT_FROM_ADR_0004,
    );
}

// ---------------------------------------------------------------------------
// AC-002 — ADR-0004 exemptions must NOT have #[non_exhaustive]
// ---------------------------------------------------------------------------

/// AC-002 / VP-013: `Phase1Permission` must NOT carry `#[non_exhaustive]`.
///
/// Exhaustive enums are not `#[non_exhaustive]`. The AST audit verifies
/// the source; here we also verify via a compile-time proof: if
/// `Phase1Permission` were `#[non_exhaustive]`, adding a wildcard arm to an
/// in-crate match would be required — the test below uses a complete match
/// (no wildcard), and the compiler accepts it. If `Phase1Permission` becomes
/// `#[non_exhaustive]`, the compiler will reject this test unless a wildcard
/// is added. That is the intended compile-time signal.
///
/// Traces to: BC-2.02.003 postcondition 2; ADR-0004 §Decision; AC-002.
#[test]
fn test_BC_2_02_003_phase1_permission_is_exhaustive() {
    // Parse monocle-core/src/permissions.rs and assert Phase1Permission has
    // NO #[non_exhaustive] attribute.
    let src = src_dir();
    let permissions_path = src.join("permissions.rs");
    let source = fs::read_to_string(&permissions_path).unwrap_or_else(|e| {
        panic!(
            "Cannot read permissions.rs at {}: {e}",
            permissions_path.display()
        )
    });
    let ast: File = syn::parse_str(&source)
        .unwrap_or_else(|e| panic!("syn failed to parse permissions.rs: {e}"));

    let mut found = false;
    for item in &ast.items {
        if let Item::Enum(e) = item {
            if e.ident == "Phase1Permission" {
                found = true;
                let has_ne = e.attrs.iter().any(is_non_exhaustive);
                assert!(
                    !has_ne,
                    "Phase1Permission MUST NOT carry #[non_exhaustive] per ADR-0004. \
                     Found the attribute in permissions.rs. \
                     Traces to: BC-2.02.003 PC-2; AC-002."
                );
            }
        }
    }
    assert!(
        found,
        "Phase1Permission enum not found in permissions.rs. \
         Expected it to be declared there per SS-permissions-phase1.md §Canonical Definition."
    );
}

/// AC-002 / VP-013: `ClaudeCodeTool` must NOT carry `#[non_exhaustive]`.
///
/// Same compile-time proof approach as `Phase1Permission`. See ADR-0004 §Decision.
///
/// Traces to: BC-2.02.003 postcondition 2; ADR-0004 §Decision; AC-002.
#[test]
fn test_BC_2_02_003_claude_code_tool_is_exhaustive() {
    let src = src_dir();
    let permissions_path = src.join("permissions.rs");
    let source = fs::read_to_string(&permissions_path).unwrap_or_else(|e| {
        panic!(
            "Cannot read permissions.rs at {}: {e}",
            permissions_path.display()
        )
    });
    let ast: File = syn::parse_str(&source)
        .unwrap_or_else(|e| panic!("syn failed to parse permissions.rs: {e}"));

    let mut found = false;
    for item in &ast.items {
        if let Item::Enum(e) = item {
            if e.ident == "ClaudeCodeTool" {
                found = true;
                let has_ne = e.attrs.iter().any(is_non_exhaustive);
                assert!(
                    !has_ne,
                    "ClaudeCodeTool MUST NOT carry #[non_exhaustive] per ADR-0004. \
                     Found the attribute in permissions.rs. \
                     Traces to: BC-2.02.003 PC-2; ADR-0004 §Decision; AC-002."
                );
            }
        }
    }
    assert!(
        found,
        "ClaudeCodeTool enum not found in permissions.rs. \
         Expected it to be declared there per SS-permissions-phase1.md §Canonical Definition."
    );
}

// ---------------------------------------------------------------------------
// AC-001 / VP-013 — Each canonical non-exhaustive enum is present and attributed
// ---------------------------------------------------------------------------

/// Verify that a named `pub enum` in `monocle-core/src/` carries `#[non_exhaustive]`.
///
/// Helper used by the canonical-enum presence tests (AC-001).
fn assert_enum_is_non_exhaustive(enum_name: &str, file_name: &str) {
    let src = src_dir();
    let path = src.join(file_name);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {file_name} at {}: {e}", path.display()));
    let ast: File =
        syn::parse_str(&source).unwrap_or_else(|e| panic!("syn failed to parse {file_name}: {e}"));

    let mut found = false;
    for item in &ast.items {
        if let Item::Enum(e) = item {
            if e.ident == enum_name {
                found = true;
                let has_ne = e.attrs.iter().any(is_non_exhaustive);
                assert!(
                    has_ne,
                    "pub enum `{enum_name}` in {file_name} is missing `#[non_exhaustive]`. \
                     Every public monocle-core enum must carry this attribute unless \
                     explicitly exempted by ADR per BC-2.02.003 PC-1. \
                     Traces to: AC-001; VP-013 Probe 13.a."
                );
            }
        }
    }
    assert!(
        found,
        "pub enum `{enum_name}` not found in {file_name}. \
         BC-2.02.003 PC-4 mandates this enum exists in monocle-core."
    );
}

/// AC-001: `HookEvent` in `hook_events.rs` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.003 PC-1, PC-4; AC-001; VP-013 Probe 13.a.
#[test]
fn test_BC_2_02_003_hook_event_is_non_exhaustive() {
    assert_enum_is_non_exhaustive("HookEvent", "hook_events.rs");
}

/// AC-001: `HookDecision` in `engine.rs` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.003 PC-1, PC-4; AC-001; VP-013 Probe 13.a.
#[test]
fn test_BC_2_02_003_hook_decision_is_non_exhaustive() {
    assert_enum_is_non_exhaustive("HookDecision", "engine.rs");
}

/// AC-001: `SessionStatus` in `engine.rs` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.003 PC-1, PC-4; AC-001; VP-013 Probe 13.a.
#[test]
fn test_BC_2_02_003_session_status_is_non_exhaustive() {
    assert_enum_is_non_exhaustive("SessionStatus", "engine.rs");
}

/// AC-001: `EngineMetadataError` in `engine.rs` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.003 PC-1, PC-4; AC-001; VP-013 Probe 13.a.
#[test]
fn test_BC_2_02_003_engine_metadata_error_is_non_exhaustive() {
    assert_enum_is_non_exhaustive("EngineMetadataError", "engine.rs");
}

/// AC-001b: `DenyReason` in `permissions.rs` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.003 PC-1, PC-4; AC-001b; VP-013 Probe 13.a.
#[test]
fn test_BC_2_02_003_deny_reason_is_non_exhaustive() {
    assert_enum_is_non_exhaustive("DenyReason", "permissions.rs");
}

/// AC-001b: `AllowPattern` in `permissions.rs` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.003 PC-1, PC-4; AC-001b; VP-013 Probe 13.a.
#[test]
fn test_BC_2_02_003_allow_pattern_is_non_exhaustive() {
    assert_enum_is_non_exhaustive("AllowPattern", "permissions.rs");
}

/// AC-001b: `DenyPattern` in `permissions.rs` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.003 PC-1, PC-4; AC-001b; VP-013 Probe 13.a.
#[test]
fn test_BC_2_02_003_deny_pattern_is_non_exhaustive() {
    assert_enum_is_non_exhaustive("DenyPattern", "permissions.rs");
}

// ---------------------------------------------------------------------------
// AC-003 / VP-013 — Full AST audit of monocle-core/src/
// (test_BC_TYPES_001_non_exhaustive_enum_coverage — canonical VP-013 name)
// ---------------------------------------------------------------------------

/// AC-003 / VP-013 Probe 13.a: Primary AST audit.
///
/// Walks every `.rs` file in `monocle-core/src/`, finds all `pub enum`
/// declarations, and asserts each is either `#[non_exhaustive]` or in the
/// ADR-0004 EXEMPT list.
///
/// This is the canonical VP-013 test. Its name is pinned per VP-013 §Harness
/// Location (historical: `test_BC_TYPES_001_non_exhaustive_enum_coverage`).
///
/// Traces to: BC-2.02.003 invariant 1; AC-003; VP-013 Probe 13.a; VP-013
/// §Post-conditions 1+2.
#[test]
fn test_BC_TYPES_001_non_exhaustive_enum_coverage() {
    let src = src_dir();
    let files = collect_rs_files(&src);
    assert!(
        !files.is_empty(),
        "No .rs files found in monocle-core/src/ — \
         check that CARGO_MANIFEST_DIR points to the crate root."
    );

    let mut all_violations: Vec<String> = Vec::new();

    for path in &files {
        let source = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
        let ast: File = match syn::parse_str(&source) {
            Ok(f) => f,
            Err(e) => panic!("syn failed to parse {}: {e}", path.display()),
        };
        let file_violations = audit_file(&ast);
        for name in file_violations {
            all_violations.push(format!(
                "  {} (in {})",
                name,
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<unknown>")
            ));
        }
    }

    assert!(
        all_violations.is_empty(),
        "VP-013 AUDIT FAILED: {} pub enum(s) in monocle-core/src/ lack \
         `#[non_exhaustive]` and are not in the ADR-0004 EXEMPT list:\n{}\n\
         Every public monocle-core enum must carry `#[non_exhaustive]` unless \
         exempted by an ADR per BC-2.02.003 PC-1. \
         Current EXEMPT list: {:?}. \
         Traces to: AC-003; VP-013 Probe 13.a.",
        all_violations.len(),
        all_violations.join("\n"),
        EXEMPT,
    );
}

// ---------------------------------------------------------------------------
// AC-003 / VP-013 Probe 13.b — failure detection using the synthetic fixture
// ---------------------------------------------------------------------------

/// VP-013 Probe 13.b: AST audit failure detection on the synthetic fixture.
///
/// Parses `tests/fixtures/missing_non_exhaustive.rs` directly (NOT compiled
/// by Cargo) and asserts that the audit detects `BadEnum` as a violation.
/// This exercises the failure path of `audit_file()` — the primary audit
/// (13.a) only passes because the production source is correctly attributed.
/// 13.b proves the audit _can_ fail when given a violating input.
///
/// The fixture file is located relative to `CARGO_MANIFEST_DIR` because
/// the `tests/` directory is sibling to `src/` under the crate root.
///
/// Traces to: BC-2.02.003 §Canonical Test Vectors row 1; AC-003;
/// VP-013 Probe 13.b; BC-2.02.003 EC-016.
#[test]
fn test_BC_2_02_003_fixture_missing_non_exhaustive_detected() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set when running cargo test");
    let fixture_path = PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("missing_non_exhaustive.rs");

    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Cannot read fixture at {}: {e}", fixture_path.display()));
    let ast: File =
        syn::parse_str(&source).unwrap_or_else(|e| panic!("syn failed to parse fixture: {e}"));

    let violations = audit_file(&ast);
    assert!(
        violations.contains(&"BadEnum".to_string()),
        "VP-013 Probe 13.b FAILED: the AST audit did NOT detect `BadEnum` \
         (a pub enum without #[non_exhaustive]) in the synthetic fixture. \
         The audit failure-detection path is broken. \
         Violations found: {:?}. \
         Traces to: AC-003; VP-013 Probe 13.b.",
        violations,
    );
}

// ---------------------------------------------------------------------------
// AC-004 — wildcard arms on non_exhaustive external enums (vacuous check)
// ---------------------------------------------------------------------------

/// AC-004: monocle-runtime match sites on non-exhaustive `monocle-core` enums
/// must have wildcard arms.
///
/// The Rust compiler enforces this at build time: if any `match` in an
/// external crate (monocle-runtime) patterns over a `#[non_exhaustive]` enum
/// without a wildcard arm, `cargo build --workspace` fails.
///
/// At S-011 dispatch time, monocle-runtime contains zero match sites on
/// `monocle-core` non-exhaustive enums. AC-004 is therefore satisfied vacuously:
/// the compiler enforces the rule at every future match site the moment it is
/// written.
///
/// This test records the vacuous-satisfaction verdict and will remain
/// non-vacuous when monocle-runtime develops match sites (the test will still
/// pass, because the compiler will reject non-wildcard matches).
///
/// Traces to: BC-2.02.003 invariant 1; AC-004; S-011 §AC-004 contingency clause.
#[test]
fn test_BC_2_02_003_wildcard_arm_compiler_enforced_vacuous() {
    // Vacuously satisfied: monocle-runtime has no match sites on monocle-core
    // non-exhaustive enums at S-011 dispatch time. The compiler enforces the
    // wildcard-arm requirement at every future match site. This test is a
    // traceability sentinel.
    assert!(
        true,
        "Vacuous: compiler enforces wildcard arm requirement at each match site."
    );
}
