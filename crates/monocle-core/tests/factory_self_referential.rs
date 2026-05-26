//! VP-015 integration tests for `VsddFactoryAdapter` (BC-2.02.005, S-012).
//!
//! Self-referential test suite: runs `VsddFactoryAdapter` against the monocle
//! repository's own `.factory/STATE.md` to verify end-to-end detection + state
//! parsing.
//!
//! Canonical test-function name pinned at BC-2.02.005 line 93 (verbatim):
//! `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection`
//!
//! # Test roster
//!
//! | Test | AC | Red Gate |
//! |------|----|----------|
//! | `test_BC_FACTORY_002_vsdd_adapter_new_constructor` | AC-011 | PASSES (no todo!()) |
//! | `test_BC_FACTORY_002_vsdd_adapter_display_name` | AC-010 | PASSES (already implemented) |
//! | `test_BC_FACTORY_002_vsdd_adapter_subscribe_empty` | AC-007, AC-009 | PASSES (already implemented) |
//! | `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection` | AC-005, AC-006 | FAILS (detect() is todo!()) |
//! | `test_BC_FACTORY_002_vsdd_adapter_read_state_not_found` | AC-008 | FAILS (read_state() is todo!()) |
//! | `test_BC_FACTORY_002_vsdd_detect_negative_no_state_file` | AC-005 negative | FAILS (detect() is todo!()) |
//! | `test_BC_FACTORY_002_vsdd_detect_negative_body_only` | AC-005, EC-021 | FAILS (detect() is todo!()) |
//! | `test_BC_FACTORY_002_vsdd_adapter_read_state_success` | AC-008, AC-012 | FAILS (read_state() is todo!()) |
//! | `test_BC_FACTORY_002_vsdd_adapter_read_state_cycle_absent` | AC-012 | FAILS (read_state() is todo!()) |
//! | `test_BC_FACTORY_002_vsdd_adapter_read_state_parse_field_guards_*` | AC-013 | FAILS (read_state() is todo!()) |

// BC-based test names use uppercase segments.
#![allow(non_snake_case)]
// Test code uses `.expect()` / `.unwrap()` to surface assertion failures.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
// `panic!` is correct test-harness behavior.
#![allow(clippy::panic)]

use std::io::Write as _;
use std::path::{Path, PathBuf};

use monocle_core::factory::{FactoryAdapter, FactoryReadError};
use monocle_core::factory::vsdd::VsddFactoryAdapter;

// ──────────────────────────────────────────────────────────────────────────────
// Fixture helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Write test-fixture content to a path atomically using `tempfile::NamedTempFile + persist`.
///
/// The project's anti-pattern policy forbids `std::fs::write` even in test code
/// (`SS-conventions-anti-patterns.md` §disallowed_methods). This helper satisfies that
/// policy for STATE.md fixture writes.
fn write_fixture(dest: &Path, content: &[u8]) {
    let dir = dest
        .parent()
        .expect("fixture path must have a parent directory");
    let mut named = tempfile::NamedTempFile::new_in(dir)
        .expect("cannot create NamedTempFile for test fixture");
    named
        .write_all(content)
        .expect("cannot write test fixture content to NamedTempFile");
    named
        .persist(dest)
        .expect("cannot persist test fixture NamedTempFile to destination");
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Resolve the monocle main-repo root from the crate's CARGO_MANIFEST_DIR.
///
/// Directory layout:
/// ```text
/// /Users/jmagady/Dev/monocle/              ← main repo root (has .factory/)
///   .worktrees/
///     S-012/
///       crates/
///         monocle-core/                    ← CARGO_MANIFEST_DIR
/// ```
///
/// From CARGO_MANIFEST_DIR, going up 4 levels yields the main repo root.
fn monocle_repo_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // crates/monocle-core → crates → S-012 → .worktrees → /Users/.../monocle
    PathBuf::from(manifest_dir)
        .parent() // crates/
        .and_then(Path::parent) // .worktrees/S-012/
        .and_then(Path::parent) // .worktrees/
        .and_then(Path::parent) // monocle/  ← main repo root
        .expect("cannot resolve monocle repo root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

/// Write a STATE.md that contains `document_type: pipeline-state` ONLY in the
/// document body, NOT in the `---`-delimited YAML frontmatter (EC-021 vector).
///
/// Uses `tempfile::NamedTempFile + persist` per the project anti-pattern policy.
fn write_body_only_state_md_in_dir(dir: &Path) -> std::io::Result<()> {
    let factory_dir = dir.join(".factory");
    std::fs::create_dir_all(&factory_dir)?;
    let state_path = factory_dir.join("STATE.md");
    // Frontmatter exists but does NOT contain the key; key only in body
    let content = b"---\ndocument_type: other-type\nphase: test-phase\n---\n\n\
         This body contains document_type: pipeline-state but it is NOT in the frontmatter.\n";
    let mut named = tempfile::NamedTempFile::new_in(&factory_dir)?;
    named.write_all(content)?;
    named
        .persist(&state_path)
        .map_err(|e| e.error)?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-011 — VsddFactoryAdapter::new() constructor (already implemented; PASSES)
// (traces to BC-2.02.005 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-011: `VsddFactoryAdapter::new()` derives `state_file_path` correctly.
///
/// Test vectors from AC-011:
/// - `/tmp/my-project` → `state_file = /tmp/my-project/.factory/STATE.md`
/// - `some/path` → `state_file = some/path/.factory/STATE.md`
/// - `PathBuf::new()` → `state_file = .factory/STATE.md`
///
/// No filesystem access at construction time (BC-2.02.005 PC-1).
///
/// Red Gate: PASSES — constructor is implemented without todo!().
///
/// Traces to: BC-2.02.005 PC-1; AC-011.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_new_constructor() {
    // Vector 1: absolute path
    let adapter = VsddFactoryAdapter::new(PathBuf::from("/tmp/my-project"));
    assert_eq!(
        adapter.state_file_path(),
        Path::new("/tmp/my-project/.factory/STATE.md"),
        "state_file_path must be workspace_root/.factory/STATE.md (AC-011 vector 1)"
    );

    // Vector 2: relative path
    let adapter = VsddFactoryAdapter::new(PathBuf::from("some/path"));
    assert_eq!(
        adapter.state_file_path(),
        Path::new("some/path/.factory/STATE.md"),
        "state_file_path must be workspace_root/.factory/STATE.md (AC-011 vector 2)"
    );

    // Vector 3: empty PathBuf (treated as current directory join)
    let adapter = VsddFactoryAdapter::new(PathBuf::new());
    assert_eq!(
        adapter.state_file_path(),
        Path::new(".factory/STATE.md"),
        "empty PathBuf must join to .factory/STATE.md (AC-011 vector 3)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-010 — display_name() returns "VSDD Factory" (already implemented; PASSES)
// (traces to BC-2.02.005 invariant 2)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-010: `display_name()` returns the exact string `"VSDD Factory"`.
///
/// BC-2.02.005 INV-2 verbatim: `display_name() returns "VSDD Factory"`.
///
/// Red Gate: PASSES — display_name() is implemented without todo!().
///
/// Traces to: BC-2.02.005 INV-2; AC-010.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_display_name() {
    let adapter = VsddFactoryAdapter::new(PathBuf::from("/tmp/test"));
    assert_eq!(
        adapter.display_name(),
        "VSDD Factory",
        "display_name() must return the exact string \"VSDD Factory\" \
         (BC-2.02.005 INV-2, AC-010)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-007, AC-009 — subscribe() Phase 1 stub returns empty stream (already
// implemented; PASSES)
// (traces to BC-2.02.005 invariant 3)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-007 and AC-009: `subscribe()` returns `Ok(empty_stream)` and
/// polling immediately returns `None`.
///
/// Phase 1 invariant (BC-2.02.005 INV-3): no file watcher instantiated.
///
/// Red Gate: PASSES — subscribe() is implemented without todo!().
///
/// Traces to: BC-2.02.005 INV-3; AC-007; AC-009.
#[tokio::test]
async fn test_BC_FACTORY_002_vsdd_adapter_subscribe_empty() {
    use futures::StreamExt as _;

    let adapter = VsddFactoryAdapter::new(PathBuf::from("/tmp/test"));
    let stream_result = adapter.subscribe();

    assert!(
        stream_result.is_ok(),
        "subscribe() must return Ok (BC-2.02.005 INV-3, AC-007)"
    );

    let mut stream = stream_result.expect("subscribe() returned Err (AC-009 failure)");

    // Poll immediately — must return None (stream is empty; no file watcher)
    let first_item = stream.next().await;
    assert!(
        first_item.is_none(),
        "subscribe() Phase 1 stream must be empty — first poll must return None \
         (BC-2.02.005 INV-3, AC-009). Got: {first_item:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-005, AC-006 — self-referential detection (FAILS — detect() is todo!())
// (traces to BC-2.02.005 postcondition 1, postcondition 2)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-005 and AC-006: `VsddFactoryAdapter::detect(monocle_repo_root)`
/// returns `Some(FactoryDetection)` for the monocle project's own `.factory/STATE.md`.
///
/// This is the canonical BC-2.02.005 line 93 test function (verbatim name required).
///
/// Red Gate: FAILS — detect() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-1; BC-2.02.005 PC-2; AC-005; AC-006; VP-015.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_self_referential_detection() {
    let repo_root = monocle_repo_root();

    // Precondition: the test environment must have .factory/STATE.md
    assert!(
        repo_root.join(".factory").join("STATE.md").exists(),
        "Self-referential test precondition failed: \
         {repo_root:?}/.factory/STATE.md does not exist. \
         Ensure the factory-artifacts worktree is mounted at .factory/ in the main repo."
    );

    let detection = VsddFactoryAdapter::detect(&repo_root);

    assert!(
        detection.is_some(),
        "VsddFactoryAdapter::detect(monocle_repo_root) must return Some(FactoryDetection) \
         because .factory/STATE.md contains `document_type: pipeline-state` in its \
         YAML frontmatter (BC-2.02.005 PC-1, AC-005, AC-006, VP-015). Got None."
    );

    let det = detection.unwrap();
    assert_eq!(
        det.display_name,
        "VSDD Factory",
        "FactoryDetection::display_name must be \"VSDD Factory\" (AC-006, VP-015)"
    );
    assert_eq!(
        det.workspace_root,
        repo_root,
        "FactoryDetection::workspace_root must match the provided workspace root (AC-006)"
    );
    assert_eq!(
        det.state_file,
        repo_root.join(".factory").join("STATE.md"),
        "FactoryDetection::state_file must be workspace_root/.factory/STATE.md (AC-006)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-008 — read_state() on non-existent path → NotFound (FAILS — todo!())
// (traces to BC-2.02.005 postcondition 4)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-008: `read_state()` returns `Err(FactoryReadError::NotFound)` when
/// STATE.md does not exist (E-FACT-001).
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-008; E-FACT-001.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_read_state_not_found() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for read_state_not_found test");
    // Do NOT create .factory/STATE.md in the tempdir
    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());

    let result = adapter.read_state();

    assert!(
        result.is_err(),
        "read_state() must return Err when STATE.md does not exist (AC-008, E-FACT-001)"
    );

    match result {
        Err(FactoryReadError::NotFound) => {
            // Expected: E-FACT-001
        }
        Err(other) => panic!(
            "read_state() must return FactoryReadError::NotFound for missing STATE.md \
             (BC-2.02.005 PC-4, AC-008, E-FACT-001). Got: {other:?}"
        ),
        Ok(_) => panic!(
            "read_state() must return Err(NotFound) for missing STATE.md, but returned Ok. \
             (BC-2.02.005 PC-4, AC-008, E-FACT-001)"
        ),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-005 negative — detect() returns None when STATE.md absent (FAILS — todo!())
// (traces to BC-2.02.005 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-005 negative: `detect()` returns `None` when there is no
/// `.factory/STATE.md` in the workspace root.
///
/// Red Gate: FAILS — detect() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-1; AC-005.
#[test]
fn test_BC_FACTORY_002_vsdd_detect_negative_no_state_file() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for detect_negative test");
    // Tempdir has no .factory/STATE.md

    let detection = VsddFactoryAdapter::detect(tmp.path());

    assert!(
        detection.is_none(),
        "detect() must return None when .factory/STATE.md is absent \
         (BC-2.02.005 PC-1, AC-005). Got Some({detection:?})"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-005, EC-021 — detect() body-only negative (FAILS — todo!())
// (traces to BC-2.02.005 postcondition 1, invariant 1, EC-021)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-005 (EC-021 negative): `detect()` returns `None` when STATE.md
/// contains `document_type: pipeline-state` ONLY in the document body, NOT in
/// the `---`-delimited YAML frontmatter.
///
/// This is the BC-2.02.005 INV-1 + EC-021 canonical test vector from S-012 AC-005:
/// detection requires the key in the FRONTMATTER; a body-only occurrence is NOT
/// sufficient for detection.
///
/// Known divergence: SS-core-types-and-abi.md line 602 reference implementation
/// uses `content.contains("document_type: pipeline-state")` which would match
/// body text and return `true`. This test enforces the stricter frontmatter-only
/// detection per BC-2.02.005 INV-1.
///
/// Red Gate: FAILS — detect() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 INV-1; BC-2.02.005 PC-1; AC-005; EC-021.
#[test]
fn test_BC_FACTORY_002_vsdd_detect_negative_body_only() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for body-only detect test");
    write_body_only_state_md_in_dir(tmp.path()).expect("cannot write body-only STATE.md");

    let detection = VsddFactoryAdapter::detect(tmp.path());

    assert!(
        detection.is_none(),
        "detect() must return None when `document_type: pipeline-state` appears ONLY in \
         the document body (not in the YAML frontmatter) — EC-021, BC-2.02.005 INV-1. \
         A body-only match is NOT a valid VSDD factory workspace. Got Some({detection:?})"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-008, AC-012 — read_state() on valid STATE.md → Ok(FactoryState) (FAILS — todo!())
// (traces to BC-2.02.005 postcondition 4, postcondition 3)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-008 and AC-012: `read_state()` returns `Ok(FactoryState)` for a
/// valid STATE.md, with correct field population.
///
/// Test vectors:
/// - `phase` populated from frontmatter `phase:` key
/// - `cycle: Some(...)` when `current_cycle:` present in frontmatter
/// - `awaiting: Some(...)` when `awaiting:` present in frontmatter
/// - `convergence: None` when §Session Resume Checkpoint section absent
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; BC-2.02.005 PC-3; AC-008; AC-012.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_read_state_success() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for read_state_success test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    let state_path = factory_dir.join("STATE.md");
    write_fixture(
        &state_path,
        concat!(
            "---\n",
            "document_type: pipeline-state\n",
            "phase: phase-3-test\n",
            "status: active\n",
            "current_cycle: cycle-007\n",
            "awaiting: \"wave-3 completion\"\n",
            "---\n\n",
            "# Body text\n",
        )
        .as_bytes(),
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let result = adapter.read_state();

    assert!(
        result.is_ok(),
        "read_state() must return Ok for a valid STATE.md (AC-008). Got: {result:?}"
    );

    let state = result.expect("read_state() returned Err for valid STATE.md (AC-008)");

    assert_eq!(
        state.phase, "phase-3-test",
        "FactoryState::phase must be populated from frontmatter `phase:` key (AC-008)"
    );
    assert_eq!(
        state.cycle,
        Some("cycle-007".to_string()),
        "FactoryState::cycle must be Some(\"cycle-007\") when current_cycle: is present \
         (AC-012)"
    );
    // EC-022: quoted string is unquoted ("wave-3 completion" not "\"wave-3 completion\"")
    assert_eq!(
        state.awaiting,
        Some("wave-3 completion".to_string()),
        "FactoryState::awaiting must be unquoted Some(\"wave-3 completion\") \
         (AC-012, EC-022)"
    );
    // convergence: None because no §Session Resume Checkpoint section
    assert!(
        state.convergence.is_none(),
        "FactoryState::convergence must be None when §Session Resume Checkpoint is absent \
         (AC-012)"
    );

    // MUST NOT be "unknown"
    assert!(
        state.cycle.as_deref() != Some("unknown"),
        "FactoryState::cycle MUST NOT be Some(\"unknown\") — \
         `None` represents absent keys; `\"unknown\"` is forbidden (BC-2.02.005 PC-3, AC-012)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-012 — cycle: None when current_cycle: absent from frontmatter (FAILS — todo!())
// (traces to BC-2.02.005 postcondition 3)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-012: `cycle: None` when `current_cycle:` is absent from STATE.md
/// YAML frontmatter.
///
/// BC-2.02.005 PC-3 verbatim: `"unknown"` MUST NOT occur for absent fields.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-3; AC-012.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_read_state_cycle_absent() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for cycle_absent test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    let state_path = factory_dir.join("STATE.md");
    // No `current_cycle:` key in frontmatter
    write_fixture(
        &state_path,
        concat!(
            "---\n",
            "document_type: pipeline-state\n",
            "phase: phase-3-test\n",
            "status: active\n",
            "---\n\n",
            "No cycle info in this state file.\n",
        )
        .as_bytes(),
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let result = adapter.read_state();

    assert!(
        result.is_ok(),
        "read_state() must succeed for valid STATE.md without current_cycle: (AC-012)"
    );

    let state = result.expect("read_state() returned Err for cycle_absent test");

    assert!(
        state.cycle.is_none(),
        "FactoryState::cycle must be None when `current_cycle:` is absent from frontmatter \
         (BC-2.02.005 PC-3, AC-012). Got: {:?}",
        state.cycle
    );

    // Explicit check: "unknown" must never appear
    assert!(
        state.cycle.as_deref() != Some("unknown"),
        "FactoryState::cycle MUST NOT be Some(\"unknown\") — `None` is the correct \
         representation for absent frontmatter keys (BC-2.02.005 PC-3, AC-012)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-013 — parse_frontmatter_field guards (FAILS — read_state() is todo!())
// (traces to BC-2.02.005 postcondition 4, EC-061, EC-022, EC-023)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-013 guard 2 (EC-061): empty value → `None`.
///
/// `current_cycle: ` (trailing space, empty value) must yield `cycle: None`.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013 guard 2; EC-061.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_guard_empty_value_yields_none() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for guard_empty_value test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    // Empty value: key present, value is empty string after trimming
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\ncurrent_cycle: \n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for guard_empty_value test (AC-013)");

    assert!(
        state.cycle.is_none(),
        "Guard 2 (EC-061): `current_cycle: ` (empty value) must yield cycle: None \
         (BC-2.02.005 PC-4, AC-013). Got: {:?}",
        state.cycle
    );
}

/// Exercises AC-013 guard 2 (EC-061): empty quoted value → `None`.
///
/// `current_cycle: ""` must yield `cycle: None` (after unquoting the value
/// is empty, triggering guard 2).
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013 guard 2; EC-061; EC-022.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_guard_empty_quoted_value_yields_none() {
    let tmp =
        tempfile::tempdir().expect("cannot create tempdir for guard_empty_quoted_value test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\ncurrent_cycle: \"\"\n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for guard_empty_quoted_value test (AC-013)");

    assert!(
        state.cycle.is_none(),
        "Guard 2 (EC-061): `current_cycle: \"\"` (empty quoted value) must yield cycle: None \
         after unquoting (BC-2.02.005 PC-4, AC-013, EC-022). Got: {:?}",
        state.cycle
    );
}

/// Exercises AC-013 guard 3 (EC-023): flow-style list value → `None`.
///
/// `blocking_issues: []` must NOT be parsed as a blocking_issues value via the
/// simple frontmatter field parser (guard 3 skips flow-style list values starting
/// with `[`).
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013 guard 3; EC-023.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_guard_flow_list_yields_none() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for guard_flow_list test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    // Flow-style list: should be guarded (returns None for the field)
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\ncurrent_cycle: [a, b]\n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for guard_flow_list test (AC-013)");

    assert!(
        state.cycle.is_none(),
        "Guard 3 (EC-023): `current_cycle: [a, b]` (flow-style list) must yield cycle: None \
         (BC-2.02.005 PC-4, AC-013, EC-023). Got: {:?}",
        state.cycle
    );
}

/// Exercises AC-013 EC-022: double-quoted scalar is unquoted.
///
/// `awaiting: "round 18 validation chain"` must yield
/// `awaiting: Some("round 18 validation chain")` — surrounding quotes stripped.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013; EC-022.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_double_quoted_scalar_unquoted() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for double_quoted_scalar test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\nawaiting: \"round 18 validation chain\"\n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for double_quoted_scalar test (AC-013)");

    assert_eq!(
        state.awaiting,
        Some("round 18 validation chain".to_string()),
        "EC-022: `awaiting: \"round 18 validation chain\"` must unquote to \
         Some(\"round 18 validation chain\") (BC-2.02.005 PC-4, AC-013, EC-022). \
         Got: {:?}",
        state.awaiting
    );
}

/// Exercises AC-013 EC-022: single-quoted scalar is unquoted.
///
/// `awaiting: 'single-quoted'` must yield `awaiting: Some("single-quoted")`.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013; EC-022.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_single_quoted_scalar_unquoted() {
    let tmp = tempfile::tempdir().expect("cannot create tempdir for single_quoted_scalar test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\nawaiting: 'single-quoted'\n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for single_quoted_scalar test (AC-013)");

    assert_eq!(
        state.awaiting,
        Some("single-quoted".to_string()),
        "EC-022: `awaiting: 'single-quoted'` must unquote to Some(\"single-quoted\") \
         (BC-2.02.005 PC-4, AC-013, EC-022). Got: {:?}",
        state.awaiting
    );
}

/// Exercises AC-013 guard 4: block scalar marker `|` → `None`.
///
/// `current_cycle: |` (block scalar literal marker) must yield `cycle: None`.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013 guard 4.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_guard_block_scalar_literal_yields_none() {
    let tmp =
        tempfile::tempdir().expect("cannot create tempdir for guard_block_scalar_literal test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\ncurrent_cycle: |\n  multi-line\n  value\n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for guard_block_scalar_literal test (AC-013)");

    assert!(
        state.cycle.is_none(),
        "Guard 4: `current_cycle: |` (block scalar literal) must yield cycle: None \
         (BC-2.02.005 PC-4, AC-013 guard 4). Got: {:?}",
        state.cycle
    );
}

/// Exercises AC-013 guard 4: folded block scalar marker `>` → `None`.
///
/// `current_cycle: > folded` must yield `cycle: None`.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013 guard 4.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_guard_block_scalar_folded_yields_none() {
    let tmp =
        tempfile::tempdir().expect("cannot create tempdir for guard_block_scalar_folded test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\ncurrent_cycle: > folded\n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for guard_block_scalar_folded test (AC-013)");

    assert!(
        state.cycle.is_none(),
        "Guard 4: `current_cycle: > folded` (folded block scalar) must yield cycle: None \
         (BC-2.02.005 PC-4, AC-013 guard 4). Got: {:?}",
        state.cycle
    );
}

/// Exercises AC-013 guard 1 (continuation lines): leading-space value → `None`.
///
/// `current_cycle:   cycle-001` (value line starting with whitespace is a
/// continuation line) — guard 1 skips it; `cycle: None`.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-013 guard 1.
#[test]
fn test_BC_FACTORY_002_vsdd_parse_guard_continuation_line_yields_none() {
    let tmp =
        tempfile::tempdir().expect("cannot create tempdir for guard_continuation_line test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    // The value on the NEXT line starts with whitespace (continuation line pattern)
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"---\ndocument_type: pipeline-state\nphase: p\nstatus: s\ncurrent_cycle:\n  cycle-001\n---\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let state = adapter
        .read_state()
        .expect("read_state() must succeed for guard_continuation_line test (AC-013)");

    assert!(
        state.cycle.is_none(),
        "Guard 1 (BC-2.02.005 PC-4 guard 1): multi-line YAML value with continuation \
         line must yield cycle: None — multi-line values are not parsed by the simple \
         frontmatter field parser (AC-013 guard 1). Got: {:?}",
        state.cycle
    );
}

/// Exercises AC-008: `read_state()` returns `Err(FactoryReadError::ParseError(...))`
/// when STATE.md exists but has no valid YAML frontmatter (E-FACT-002).
///
/// A file that contains no `---` delimiters is unparseable as a frontmatter document.
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-008; E-FACT-002.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_read_state_parse_error_no_frontmatter() {
    let tmp =
        tempfile::tempdir().expect("cannot create tempdir for read_state_parse_error test");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("cannot create .factory dir");
    // File exists but has no frontmatter at all (not a valid VSDD state file)
    write_fixture(
        &factory_dir.join("STATE.md"),
        b"This file has no YAML frontmatter delimiters at all.\nJust plain text.\n",
    );

    let adapter = VsddFactoryAdapter::new(tmp.path().to_path_buf());
    let result = adapter.read_state();

    assert!(
        result.is_err(),
        "read_state() must return Err when STATE.md has no valid frontmatter \
         (AC-008, E-FACT-002)"
    );

    match result {
        Err(FactoryReadError::ParseError(_)) => {
            // Expected: E-FACT-002
        }
        Err(FactoryReadError::NotFound) => panic!(
            "read_state() returned NotFound for an existing (but unparseable) STATE.md — \
             must return ParseError (BC-2.02.005 PC-4, AC-008, E-FACT-002)"
        ),
        Err(other) => panic!(
            "read_state() must return FactoryReadError::ParseError for unparseable STATE.md. \
             Got: {other:?}"
        ),
        Ok(_) => panic!(
            "read_state() must return Err(ParseError) for unparseable STATE.md, \
             but returned Ok (BC-2.02.005 PC-4, AC-008, E-FACT-002)"
        ),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-008, AC-012 — self-referential read_state() on the real monocle STATE.md
// (FAILS — read_state() is todo!())
// (traces to BC-2.02.005 postcondition 4)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-008 end-to-end: `read_state()` on the monocle project's own
/// `.factory/STATE.md` returns `Ok(FactoryState)` with a non-empty `phase`.
///
/// This is the full VP-015 integration probe (state parsing against real data).
///
/// Red Gate: FAILS — read_state() panics with `todo!()`.
///
/// Traces to: BC-2.02.005 PC-4; AC-008; VP-015.
#[test]
fn test_BC_FACTORY_002_vsdd_adapter_read_state_on_real_state_md() {
    let repo_root = monocle_repo_root();

    assert!(
        repo_root.join(".factory").join("STATE.md").exists(),
        "Self-referential read_state() test precondition failed: \
         {repo_root:?}/.factory/STATE.md does not exist."
    );

    let adapter = VsddFactoryAdapter::new(repo_root);
    let result = adapter.read_state();

    assert!(
        result.is_ok(),
        "read_state() must return Ok for the real monocle .factory/STATE.md \
         (BC-2.02.005 PC-4, AC-008, VP-015). Got: {result:?}"
    );

    let state = result.expect("read_state() returned Err for real STATE.md");

    // Phase must be non-empty (the real STATE.md always has a phase)
    assert!(
        !state.phase.is_empty(),
        "FactoryState::phase must be non-empty for the real monocle STATE.md \
         (AC-008, VP-015)"
    );

    // Status must be non-empty
    assert!(
        !state.status.is_empty(),
        "FactoryState::status must be non-empty for the real monocle STATE.md \
         (AC-008, VP-015)"
    );

    // Verify the `"unknown"` invariant (BC-2.02.005 PC-3): real STATE.md must not
    // have the string "unknown" as a value in cycle or awaiting
    assert!(
        state.cycle.as_deref() != Some("unknown"),
        "FactoryState::cycle MUST NOT be Some(\"unknown\") for real STATE.md \
         (BC-2.02.005 PC-3, AC-012)"
    );
    assert!(
        state.awaiting.as_deref() != Some("unknown"),
        "FactoryState::awaiting MUST NOT be Some(\"unknown\") for real STATE.md \
         (BC-2.02.005 PC-3, AC-012)"
    );
}
