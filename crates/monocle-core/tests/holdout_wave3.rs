//! Holdout evaluation tests for Wave 3 (monocle-core).
//!
//! Scenarios covered:
//! - HS-W3-003: VsddFactoryAdapter Detects monocle's Own Factory
//! - HS-W3-005: FactoryAdapter subscribe() Stream is Empty in Phase 1

// Test files: expect/unwrap are idiomatic assertion amplification.
#![allow(clippy::expect_used, clippy::unwrap_used)]
#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use std::io::Write as _;
use std::path::{Path, PathBuf};

use monocle_core::factory::vsdd::VsddFactoryAdapter;
use monocle_core::factory::FactoryAdapter;

// ──────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────

/// Write test-fixture content to a path atomically using tempfile::persist.
fn write_fixture(dest: &Path, content: &[u8]) {
    let dir = dest
        .parent()
        .expect("fixture path must have a parent directory");
    let mut named = tempfile::NamedTempFile::new_in(dir).expect("cannot create NamedTempFile");
    named.write_all(content).expect("cannot write fixture");
    named.persist(dest).expect("cannot persist fixture");
}

/// Resolve the monocle main-repo root from CARGO_MANIFEST_DIR.
fn monocle_repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|p| p.join(".git").exists())
        .expect("cannot find git root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

// ═══════════════════════════════════════════════════════════════════════════
// HS-W3-003: VsddFactoryAdapter Detects monocle's Own Factory
// ═══════════════════════════════════════════════════════════════════════════

/// HS-W3-003 positive: detect(monocle_repo_root) must return Some(...)
/// because monocle has .factory/STATE.md with document_type: pipeline-state.
#[test]
fn test_HS_W3_003_detect_monocle_own_factory() {
    let repo_root = monocle_repo_root();

    // Precondition: .factory/STATE.md must exist
    let state_path = repo_root.join(".factory").join("STATE.md");
    assert!(
        state_path.exists(),
        "HS-W3-003 precondition: {state_path:?} must exist. \
         Ensure the factory-artifacts worktree is mounted."
    );

    let detection = VsddFactoryAdapter::detect(&repo_root);

    assert!(
        detection.is_some(),
        "HS-W3-003: VsddFactoryAdapter::detect(monocle_repo_root) must return Some(...) \
         because .factory/STATE.md contains document_type: pipeline-state. Got None."
    );

    let det = detection.unwrap();
    assert_eq!(
        det.display_name, "VSDD Factory",
        "HS-W3-003: detection display_name must be 'VSDD Factory'"
    );
    assert_eq!(
        det.workspace_root, repo_root,
        "HS-W3-003: detection workspace_root must match provided root"
    );
    assert_eq!(
        det.state_file,
        repo_root.join(".factory").join("STATE.md"),
        "HS-W3-003: detection state_file must be workspace_root/.factory/STATE.md"
    );
}

/// HS-W3-003 negative: a temp dir with .factory/STATE.md but document_type != pipeline-state
/// must cause detect() to return None.
#[test]
fn test_HS_W3_003_detect_wrong_document_type_returns_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let factory_dir = tmp.path().join(".factory");
    std::fs::create_dir_all(&factory_dir).expect("create .factory dir");

    // Write STATE.md with document_type: something-else
    write_fixture(
        &factory_dir.join("STATE.md"),
        concat!(
            "---\n",
            "document_type: something-else\n",
            "phase: test\n",
            "status: active\n",
            "---\n\n",
            "# Not a VSDD factory\n",
        )
        .as_bytes(),
    );

    let detection = VsddFactoryAdapter::detect(tmp.path());

    assert!(
        detection.is_none(),
        "HS-W3-003: detect() must return None when document_type is 'something-else', \
         not 'pipeline-state'. Got Some({detection:?})"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// HS-W3-005: FactoryAdapter subscribe() Stream is Empty in Phase 1
// ═══════════════════════════════════════════════════════════════════════════

/// HS-W3-005: subscribe() must return Ok(stream), and polling it must
/// immediately return None (not block).
#[tokio::test]
async fn test_HS_W3_005_subscribe_stream_is_empty() {
    use futures::StreamExt as _;

    let adapter = VsddFactoryAdapter::new(PathBuf::from("/tmp/test-hs-w3-005"));

    // subscribe() must return Ok
    let stream_result = adapter.subscribe();
    assert!(
        stream_result.is_ok(),
        "HS-W3-005: subscribe() must return Ok(stream); got Err: {:?}",
        stream_result.err()
    );

    let mut stream = stream_result.unwrap();

    // First poll must return None immediately (not block)
    // Use tokio::time::timeout to ensure we don't block forever
    let poll_result =
        tokio::time::timeout(std::time::Duration::from_millis(100), stream.next()).await;

    match poll_result {
        Ok(None) => {
            // Expected: stream returned None immediately
        }
        Ok(Some(_)) => {
            panic!(
                "HS-W3-005: subscribe() Phase 1 stream must be empty — \
                 first poll must return None. Got Some(FactoryState)."
            );
        }
        Err(_timeout) => {
            panic!(
                "HS-W3-005: subscribe() Phase 1 stream must return None immediately, \
                 not block. Timed out after 100ms."
            );
        }
    }
}
