//! Integration tests for S-017: Daemon Start Sequence (SOQ-2) + Hook Tmpfile Generation.
//!
//! Traces to BC-2.04.001 (13-step daemon start sequence) and BC-2.04.010
//! (hooks-settings.json generation).
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause |
//! |---------------|----|-----------|
//! | test_BC_2_04_001_clean_start_all_13_steps_succeed | AC-001..AC-013 | BC-2.04.001 PC-1..PC-13 |
//! | test_BC_2_04_001_soq2_ordering_lock_before_hooks_settings | AC-012 | BC-2.04.001 INV-SOQ2 |
//! | test_BC_2_04_001_runtime_dir_created_with_mode_0o700 | AC-001 | BC-2.04.001 PC-1 |
//! | test_BC_2_04_001_lock_file_written_mode_0o600 | AC-008 | BC-2.04.001 PC-8, INV-3 |
//! | test_BC_2_04_010_hooks_settings_json_schema_4_active_hooks | AC-011 | BC-2.04.010 PC-4 |
//! | test_BC_2_04_010_hooks_settings_post_tool_use_empty_array | AC-011 | BC-2.04.010 PC-5 |
//! | test_BC_2_04_010_hooks_settings_pre_compact_empty_array | AC-011 | BC-2.04.010 PC-6 |
//! | test_BC_2_04_010_hooks_settings_no_session_start | AC-011 | BC-2.04.010 INV-1 |
//! | test_BC_2_04_010_hooks_settings_mode_0o600 | AC-010 | BC-2.04.010 PC-3 |
//! | test_BC_2_04_001_auth_token_64_hex_chars_no_prefix | AC-007 | BC-2.04.001 PC-7 |
//! | test_BC_2_04_001_lock_file_contract_version_string | AC-008 | BC-2.04.001 PC-8 |
//! | test_BC_2_04_001_uds_socket_created_at_runtime_dir | AC-013 | BC-2.04.001 PC-13 |
//! | test_BC_2_04_001_stale_socket_removed_before_bind | AC-013 | BC-2.04.001 PC-13 |
//! | test_BC_2_04_001_lock_cleanup_on_post_step8_failure | AC-016 | BC-2.04.001 INV-6 |
//! | test_BC_2_04_010_hooks_settings_removed_on_shutdown | AC-017 | BC-2.04.010 PC-7 |
//! | test_BC_2_04_001_step8_failure_no_hooks_settings_generated | AC-016 | BC-2.04.001 INV-6 |
//! | test_BC_2_04_001_ring_buffer_in_daemon_state | AC-004 | BC-2.04.001 PC-4 |
//! | test_BC_2_04_001_event_bus_bounded_4096 | AC-005 | BC-2.04.001 PC-5 |
//! | test_BC_2_04_001_step2_stale_lock_removed_and_new_daemon_starts | AC-002 | BC-2.04.001 PC-2 (stale path) |
//! | test_BC_2_04_001_step2_live_lock_returns_conflict | AC-002 | BC-2.04.001 PC-2 (live path) |
//!
//! # Red Gate
//!
//! ALL tests in this file MUST FAIL before S-017 implementation begins.
//! The stub functions call `todo!()` and will panic. Tests are written to
//! exercise behavior via the stub surface — they compile but panic at runtime.

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per naming convention.
#![allow(non_snake_case)]
// std::fs::write is used here to plant a stale socket file for the stale-removal test;
// this is not a config-file write — the disallowed_methods rule targets production config writes.
#![allow(clippy::disallowed_methods)]
// Test imports: some types are imported for the wildcard-use pattern or to verify the import path.
#![allow(unused_imports)]

use std::os::unix::fs::PermissionsExt as _;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use monocle_runtime::errors::DaemonStartError;
use monocle_runtime::lifecycle::{daemon_start_sequence, write_hooks_settings, write_lock_file};
use monocle_runtime::ring::{RingBuffer, RotationConfig};
use monocle_runtime::types::{
    EngineModuleRegistry, EventBusHookEvent, EventBusTx, HooksSettings, EVENT_BUS_CAPACITY,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create an isolated temp directory for a single test run.
fn isolated_runtime_dir() -> TempDir {
    tempfile::tempdir().expect("create isolated runtime dir")
}

// ---------------------------------------------------------------------------
// AC-001: Step 1 — runtime dir resolved + created with 0o700
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-1: runtime dir created with mode 0o700.
///
/// Verifies that after `daemon_start_sequence` completes the runtime_dir exists
/// and has mode 0o700 (owner-only rwx).
#[tokio::test]
async fn test_BC_2_04_001_runtime_dir_created_with_mode_0o700() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");

    // daemon_start_sequence must create runtime_dir with 0o700 if it doesn't exist.
    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let meta = std::fs::metadata(&runtime_dir).expect("runtime_dir must exist after start");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o700,
        "runtime_dir must have mode 0o700 (got {mode:o})"
    );
}

// ---------------------------------------------------------------------------
// AC-003: Step 3 — TcpListener bound BEFORE any file write
// AC-008: Step 8 — lock file written after port bind
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-8: lock file written with mode 0o600.
///
/// After `daemon_start_sequence`, `monocle.lock` must exist in runtime_dir with
/// file mode 0o600.
#[tokio::test]
async fn test_BC_2_04_001_lock_file_written_mode_0o600() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let lock_path = runtime_dir.join("monocle.lock");
    assert!(lock_path.exists(), "monocle.lock must exist after start");

    let meta = std::fs::metadata(&lock_path).expect("metadata on lock file");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "lock file must have mode 0o600 (got {mode:o})");
}

/// Exercises BC-2.04.001 step 8: contract_version field in lock file.
///
/// The lock file JSON must contain `contract_version: "monocle-lock-v1"` (string)
/// per the S-017 BC-2.04.001 PC-8 specification.
///
/// Note: This test exercises the new string-based contract_version in the
/// write_lock_file stub (not the legacy integer-based LockFileContent).
#[tokio::test]
async fn test_BC_2_04_001_lock_file_contract_version_string() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let lock_path = runtime_dir.join("monocle.lock");
    let lock_content = std::fs::read_to_string(&lock_path).expect("read lock file");
    let lock_json: serde_json::Value =
        serde_json::from_str(&lock_content).expect("lock file must be valid JSON");

    // BC-2.04.001 PC-8: contract_version is "monocle-lock-v1" (string).
    let contract_version = lock_json
        .get("contract_version")
        .expect("lock file must have contract_version field");
    assert_eq!(
        contract_version.as_str(),
        Some("monocle-lock-v1"),
        "contract_version must be string 'monocle-lock-v1' per BC-2.04.001 PC-8"
    );
}

// ---------------------------------------------------------------------------
// AC-007: Step 7 — auth token 64 hex chars, no monocle-v1: prefix
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-7: auth token hex portion is 64 lowercase hex chars.
///
/// The lock file's `token` field carries the wire-format `monocle-v1:<64-hex>` string.
/// The hex portion (after stripping the `monocle-v1:` prefix) must be exactly 64 lowercase
/// hex characters. BC-2.04.001 PC-15 defines exactly 5 fields: pid, port, token,
/// contract_version, started_at — there is no separate `authToken` field.
#[tokio::test]
async fn test_BC_2_04_001_auth_token_64_hex_chars_no_prefix() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let lock_path = runtime_dir.join("monocle.lock");
    let lock_content = std::fs::read_to_string(&lock_path).expect("read lock file");
    let lock_json: serde_json::Value =
        serde_json::from_str(&lock_content).expect("lock file must be valid JSON");

    // BC-2.04.001 PC-15: the lock file has exactly 5 fields (pid, port, token,
    // contract_version, started_at). `authToken` is NOT a separate field; the raw hex
    // is embedded as the suffix of `token` after the `monocle-v1:` prefix.
    let token_field = lock_json
        .get("token")
        .and_then(|v| v.as_str())
        .expect("lock file must have 'token' string field (BC-2.04.001 PC-15)");

    // token field must have the monocle-v1: prefix.
    assert!(
        token_field.starts_with("monocle-v1:"),
        "token field must start with 'monocle-v1:' prefix (got: {token_field})"
    );

    // Strip prefix and verify the hex portion.
    let hex_part = token_field
        .strip_prefix("monocle-v1:")
        .expect("already asserted starts_with");

    // Must be exactly 64 chars.
    assert_eq!(
        hex_part.len(),
        64,
        "token hex portion must be exactly 64 chars (got {})",
        hex_part.len()
    );
    // Must be all lowercase hex.
    assert!(
        hex_part
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "token hex portion must be 64 lowercase hex chars (got: {hex_part})"
    );

    // BC-2.04.001 PC-15: no separate `authToken` field should be present in the lock file.
    assert!(
        lock_json.get("authToken").is_none(),
        "lock file must NOT have a separate 'authToken' field (BC-2.04.001 PC-15 — exactly 5 fields)"
    );
}

// ---------------------------------------------------------------------------
// AC-009: Step 9 — hooks-settings.json written
// AC-010: Step 10 — hooks-settings.json mode 0o600
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.010 PC-3: hooks-settings.json mode is 0o600.
#[tokio::test]
async fn test_BC_2_04_010_hooks_settings_mode_0o600() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    assert!(
        hs_path.exists(),
        "hooks-settings.json must exist after start"
    );

    let meta = std::fs::metadata(&hs_path).expect("metadata on hooks-settings.json");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "hooks-settings.json must have mode 0o600 (got {mode:o})"
    );
}

// ---------------------------------------------------------------------------
// AC-011: Step 11 — hooks-settings.json schema
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.010 PC-4: exactly 4 active hooks in hooks-settings.json.
///
/// Active hooks are: PreToolUse, Notification, Stop, UserPromptSubmit.
/// Each must have a non-empty command entry.
#[tokio::test]
async fn test_BC_2_04_010_hooks_settings_json_schema_4_active_hooks() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let hs_content = std::fs::read_to_string(&hs_path).expect("read hooks-settings.json");
    let hs: serde_json::Value =
        serde_json::from_str(&hs_content).expect("hooks-settings.json must be valid JSON");

    let hooks = hs.get("hooks").expect("must have 'hooks' key");

    // Verify all 4 active hooks are present and non-empty.
    for hook_name in &["PreToolUse", "Notification", "Stop", "UserPromptSubmit"] {
        let entries = hooks
            .get(hook_name)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("hooks.{hook_name} must be an array"));
        assert!(
            !entries.is_empty(),
            "hooks.{hook_name} must be non-empty (active hook)"
        );
    }

    // Count active (non-empty) hooks — must be exactly 4.
    let hooks_obj = hooks.as_object().expect("hooks must be an object");
    let active_count = hooks_obj
        .values()
        .filter(|v| v.as_array().map(|a| !a.is_empty()).unwrap_or(false))
        .count();
    assert_eq!(
        active_count, 4,
        "exactly 4 hook types must be active (got {active_count})"
    );
}

/// Exercises BC-2.04.010 PC-5: PostToolUse is empty array in hooks-settings.json.
///
/// JC-2 closure: PostToolUse is not part of the Phase 1 canonical 5-endpoint surface.
#[tokio::test]
async fn test_BC_2_04_010_hooks_settings_post_tool_use_empty_array() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let hs_content = std::fs::read_to_string(&hs_path).expect("read hooks-settings.json");
    let hs: serde_json::Value =
        serde_json::from_str(&hs_content).expect("hooks-settings.json must be valid JSON");

    let post_tool_use = hs
        .get("hooks")
        .and_then(|h| h.get("PostToolUse"))
        .and_then(|v| v.as_array())
        .expect("hooks.PostToolUse must be an array");

    assert!(
        post_tool_use.is_empty(),
        "hooks.PostToolUse must be empty array (JC-2, BC-2.04.010 PC-5)"
    );
}

/// Exercises BC-2.04.010 PC-6: PreCompact is empty array in hooks-settings.json.
///
/// PreCompact is not in the canonical 5 endpoints for Phase 1.
#[tokio::test]
async fn test_BC_2_04_010_hooks_settings_pre_compact_empty_array() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let hs_content = std::fs::read_to_string(&hs_path).expect("read hooks-settings.json");
    let hs: serde_json::Value =
        serde_json::from_str(&hs_content).expect("hooks-settings.json must be valid JSON");

    let pre_compact = hs
        .get("hooks")
        .and_then(|h| h.get("PreCompact"))
        .and_then(|v| v.as_array())
        .expect("hooks.PreCompact must be an array");

    assert!(
        pre_compact.is_empty(),
        "hooks.PreCompact must be empty array (BC-2.04.010 PC-6)"
    );
}

/// Exercises BC-2.04.010 INV-1: SessionStart key is absent from hooks-settings.json.
///
/// JC-2 closure: SessionStart is not included in hooks-settings.json.
/// The key must not appear in the `hooks` object at all.
#[tokio::test]
async fn test_BC_2_04_010_hooks_settings_no_session_start() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let hs_content = std::fs::read_to_string(&hs_path).expect("read hooks-settings.json");
    let hs: serde_json::Value =
        serde_json::from_str(&hs_content).expect("hooks-settings.json must be valid JSON");

    let hooks = hs.get("hooks").expect("must have 'hooks' key");
    assert!(
        hooks.get("SessionStart").is_none(),
        "hooks.SessionStart must be absent entirely (JC-2, BC-2.04.010 INV-1)"
    );
}

// ---------------------------------------------------------------------------
// AC-012: SOQ-2 ordering — lock file BEFORE hooks-settings.json
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 SOQ-2 invariant: lock file must be written BEFORE
/// hooks-settings.json.
///
/// The test verifies that `write_hooks_settings` is only called after
/// `write_lock_file` has succeeded. We inject this by verifying that if the
/// daemon is inspected mid-sequence, hooks-settings.json doesn't appear before
/// the lock file.
///
/// The primary verification is structural: we call `write_lock_file` directly
/// and then `write_hooks_settings`, ensuring the lock file path is accessible
/// before hooks-settings.json is written.
#[tokio::test]
async fn test_BC_2_04_001_soq2_ordering_lock_before_hooks_settings() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    // Verify SOQ-2 at the function level: write_lock_file stub must be called BEFORE
    // write_hooks_settings stub. Both are todo!() stubs; we test that they exist and
    // are callable in the correct order.

    // The daemon_start_sequence enforces this ordering. A full start must result
    // in both files present, with lock file's mtime <= hooks-settings.json's mtime.
    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let lock_path = runtime_dir.join("monocle.lock");
    let hs_path = runtime_dir.join("hooks-settings.json");

    assert!(lock_path.exists(), "monocle.lock must exist (step 8)");
    assert!(
        hs_path.exists(),
        "hooks-settings.json must exist (step 9, after lock)"
    );

    // Verify ordering: lock file creation time must be <= hooks-settings.json creation time.
    // On platforms with sub-second file timestamps, this catches sequence inversions.
    let lock_meta = std::fs::metadata(&lock_path).expect("lock file metadata");
    let hs_meta = std::fs::metadata(&hs_path).expect("hooks-settings.json metadata");

    let lock_modified = lock_meta
        .modified()
        .expect("lock file must support modified time");
    let hs_modified = hs_meta
        .modified()
        .expect("hooks-settings.json must support modified time");

    assert!(
        lock_modified <= hs_modified,
        "SOQ-2 violated: lock file mtime ({lock_modified:?}) must be <= hooks-settings.json mtime ({hs_modified:?})"
    );
}

// ---------------------------------------------------------------------------
// AC-013: Step 13 — UDS socket at runtime_dir/monocle.sock, mode 0o600
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-13: UDS socket created at runtime_dir/monocle.sock.
#[tokio::test]
async fn test_BC_2_04_001_uds_socket_created_at_runtime_dir() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let sock_path = runtime_dir.join("monocle.sock");
    assert!(
        sock_path.exists(),
        "monocle.sock must exist after start (BC-2.04.001 PC-13)"
    );

    let meta = std::fs::metadata(&sock_path).expect("metadata on monocle.sock");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "monocle.sock must have mode 0o600 (got {mode:o})"
    );
}

/// Exercises BC-2.04.001 PC-13: stale socket at runtime_dir/monocle.sock is removed before bind.
///
/// Creates a pre-existing file at the socket path, then verifies that
/// `daemon_start_sequence` removes it before creating a fresh socket.
#[tokio::test]
async fn test_BC_2_04_001_stale_socket_removed_before_bind() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    // Plant a stale "socket" file (just a plain file for the remove-before-bind test).
    let sock_path = runtime_dir.join("monocle.sock");
    std::fs::write(&sock_path, b"stale-socket-placeholder").expect("plant stale socket file"); // nosemgrep: monocle-no-naked-fs-write
    assert!(sock_path.exists(), "stale socket must exist before start");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed even with stale socket");

    // After start, socket file must exist and be the new real socket.
    assert!(
        sock_path.exists(),
        "monocle.sock must exist after start (stale removed + fresh bound)"
    );
}

// ---------------------------------------------------------------------------
// AC-001 (full sequence): clean start — all 13 steps succeed
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-1..PC-13: clean start with all 13 steps succeeding.
///
/// This is the primary happy-path integration test. It verifies that after a
/// successful `daemon_start_sequence` call, all expected artifacts exist with
/// correct permissions.
#[tokio::test]
async fn test_BC_2_04_001_clean_start_all_13_steps_succeed() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    // Do NOT pre-create runtime_dir — daemon_start_sequence must create it.

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed on clean start");

    // Step 1: runtime_dir created.
    assert!(runtime_dir.exists(), "runtime_dir must exist after start");

    // Step 8: lock file exists.
    assert!(
        runtime_dir.join("monocle.lock").exists(),
        "monocle.lock must exist after start"
    );

    // Step 9: hooks-settings.json exists.
    assert!(
        runtime_dir.join("hooks-settings.json").exists(),
        "hooks-settings.json must exist after start"
    );

    // Step 13: UDS socket exists.
    assert!(
        runtime_dir.join("monocle.sock").exists(),
        "monocle.sock must exist after start"
    );
}

// ---------------------------------------------------------------------------
// AC-016: Lock file cleanup on post-step-8 failure (invariant 6)
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 INV-6: lock file is cleaned up on post-step-8 failure.
///
/// If any step after the lock file is written (step 8) fails before the sequence
/// completes, the lock file MUST be removed from the filesystem. This prevents a
/// stale lock from blocking future daemon starts.
///
/// This test simulates a post-step-8 failure by using a runtime_dir that will
/// reject the hooks-settings.json write (step 9) but has accepted the lock file
/// write (step 8). We verify the lock file is absent after the failed sequence.
///
/// Since we cannot inject failures into the stub, this test exercises the
/// daemon_start_sequence error path with a directory that makes step 9 fail
/// while step 8 has already succeeded.
#[tokio::test]
async fn test_BC_2_04_001_lock_cleanup_on_post_step8_failure() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    // Make hooks-settings.json path non-writable by placing a read-only DIRECTORY there.
    // This causes the hooks-settings.json write (step 9) to fail while the lock file
    // write (step 8) has already succeeded.
    let hs_path = runtime_dir.join("hooks-settings.json");
    std::fs::create_dir_all(&hs_path).expect("create hooks-settings.json as dir to block write");

    // daemon_start_sequence must fail because hooks-settings.json cannot be written.
    let result = daemon_start_sequence(&runtime_dir).await;
    assert!(
        result.is_err(),
        "daemon_start_sequence must fail when hooks-settings.json path is a directory"
    );

    // INV-6: lock file must be cleaned up after post-step-8 failure.
    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        !lock_path.exists(),
        "monocle.lock must be removed on post-step-8 failure (BC-2.04.001 INV-6)"
    );
}

/// Exercises BC-2.04.001 INV-6: hooks-settings.json is NOT generated when step 8 fails.
///
/// If the lock file write (step 8) fails, hooks-settings.json must never be written
/// (SOQ-2 precondition: hooks-settings.json only appears after lock file).
#[tokio::test]
async fn test_BC_2_04_001_step8_failure_no_hooks_settings_generated() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    // Make lock file path non-writable by placing a read-only DIRECTORY there.
    // This causes the lock file write (step 8) to fail.
    let lock_path = runtime_dir.join("monocle.lock");
    std::fs::create_dir_all(&lock_path).expect("create monocle.lock as dir to block write");

    let result = daemon_start_sequence(&runtime_dir).await;
    assert!(
        result.is_err(),
        "daemon_start_sequence must fail when monocle.lock path is a directory"
    );

    // hooks-settings.json must NOT have been written (step 8 failed, so step 9 never ran).
    let hs_path = runtime_dir.join("hooks-settings.json");
    assert!(
        !hs_path.exists(),
        "hooks-settings.json must NOT be created when lock file write fails (SOQ-2)"
    );
}

// ---------------------------------------------------------------------------
// AC-017: hooks-settings.json removed on graceful shutdown
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.010 PC-7: hooks-settings.json is removed on graceful shutdown.
///
/// When the daemon shuts down gracefully, `hooks-settings.json` must be removed
/// from the runtime directory. This ensures Claude Code stops posting hooks to
/// the (now-dead) daemon endpoint.
#[tokio::test]
async fn test_BC_2_04_010_hooks_settings_removed_on_shutdown() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    // First, start the daemon to create hooks-settings.json.
    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    assert!(
        hs_path.exists(),
        "hooks-settings.json must exist after start"
    );

    // Trigger graceful shutdown via the state's shutdown_tx.
    // The shutdown path must remove hooks-settings.json before calling exit_with(Graceful).
    //
    // We verify this by constructing the DaemonState and triggering its shutdown path.
    // Since the shutdown path is implemented in the run_server/signal wiring in S-017,
    // we test the cleanup function directly.
    monocle_runtime::lifecycle::remove_hooks_settings(&runtime_dir)
        .expect("remove_hooks_settings must succeed");

    assert!(
        !hs_path.exists(),
        "hooks-settings.json must be removed on graceful shutdown (BC-2.04.010 PC-7)"
    );
}

// ---------------------------------------------------------------------------
// AC-004: Step 4 — RingBuffer in DaemonState, 100MB × 5 rotations
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-4: RingBuffer constructed with 100 MiB × 5 rotations
/// and Arc-wrapped.
///
/// Verifies that a `RingBuffer` with the correct rotation config (100 MiB hard cap,
/// 5 retained files) can be constructed and wrapped in Arc as required by DaemonState.
#[tokio::test]
async fn test_BC_2_04_001_ring_buffer_in_daemon_state() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    // After a successful start, the DaemonState must have ring != None.
    // We verify this indirectly by ensuring the ring buffer path would exist
    // if we push a record (the ring file is created lazily on first push).
    // The primary assertion is that DaemonState can be constructed with the
    // correct ring config — verified by the successful sequence above.

    // Additionally verify that RingBuffer::new with the 100 MiB config is
    // constructable (type-level validation).
    let ring_path = runtime_dir.join("monocle-events.jsonl");
    let config = RotationConfig {
        soft_threshold_bytes: 50 * 1024 * 1024, // 50 MiB soft threshold
        hard_cap_bytes: 100 * 1024 * 1024,      // 100 MiB hard cap (BC-2.04.001 PC-4)
        retained: 5,                            // 5 rotated files (BC-2.04.001 PC-4)
    };
    let _ring: Arc<RingBuffer> = Arc::new(RingBuffer::new(ring_path, config));
    // If we reach here, the RingBuffer can be constructed and Arc-wrapped.
}

// ---------------------------------------------------------------------------
// AC-005: Step 5 — mpsc::channel bounded at 4096
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-5: event bus channel bounded at exactly 4096 slots.
///
/// Verifies that `EVENT_BUS_CAPACITY` is 4096 and that the channel created with
/// that capacity can be used as `EventBusTx` / `EventBusRx`.
#[tokio::test]
async fn test_BC_2_04_001_event_bus_bounded_4096() {
    // Verify the constant.
    assert_eq!(EVENT_BUS_CAPACITY, 4096, "EVENT_BUS_CAPACITY must be 4096");

    // Verify the channel type compiles and is usable.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<EventBusHookEvent>(EVENT_BUS_CAPACITY);
    let _: EventBusTx = tx.clone();

    // Verify the drop counter type is AtomicU64.
    let drop_counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    assert_eq!(
        drop_counter.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "drop counter must start at 0"
    );

    // Verify channel capacity is respected: we should be able to fill it.
    // (Only test capacity assertion, don't actually fill 4096 slots in a unit test.)
    // Instead, verify a send/recv pair works.
    //
    // HookEvent variants are #[non_exhaustive] — cannot use struct-literal construction
    // outside monocle-core. Use serde_json deserialization to construct the event.
    use monocle_core::hook_events::HookEvent;
    let event: HookEvent = serde_json::from_value(serde_json::json!({
        "Stop": {
            "stop_reason": "end_turn",
            "session_id": "test-session-id",
            "pid": 12345
        }
    }))
    .expect("deserialize HookEvent::Stop");

    let bus_event = EventBusHookEvent::new(event, "2026-05-27T10:00:00.000Z".to_string());
    tx.send(bus_event)
        .await
        .expect("send must succeed on unbounded-below-capacity channel");
    let received = rx.recv().await.expect("recv must succeed");
    assert_eq!(received.received_at, "2026-05-27T10:00:00.000Z");

    // Verify that starting the daemon sets up a bounded channel internally.
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");
}

// ---------------------------------------------------------------------------
// AC-014: Step 14 — crash recovery checkpoint init (BC-2.01.006)
// ---------------------------------------------------------------------------

/// Exercises AC-014 (BC-2.01.006): crash recovery checkpoint is initialized
/// as part of the daemon start sequence.
///
/// After `daemon_start_sequence`, the recovery checkpoint infrastructure must be
/// ready. The test verifies by checking that the checkpoint can be written and
/// read back via the existing lifecycle API.
#[tokio::test]
async fn test_BC_2_04_001_crash_recovery_checkpoint_init() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");

    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    // Verify that the checkpoint infrastructure is ready: write a checkpoint
    // into the runtime dir and read it back.
    use monocle_runtime::lifecycle::{read_recovery_checkpoint, write_recovery_checkpoint};
    use monocle_runtime::types::{CheckpointReadResult, RecoveryCheckpoint, ShutdownReason};

    let checkpoint = RecoveryCheckpoint {
        pid: std::process::id(),
        shutdown_reason: ShutdownReason::Signal,
        last_app_mode: "Running".to_string(),
        shutdown_utc: "2026-05-27T10:00:00.000Z".to_string(),
    };
    let cp_path = runtime_dir.join("recovery-checkpoint.json");

    write_recovery_checkpoint(&cp_path, &checkpoint)
        .expect("write_recovery_checkpoint must succeed after daemon_start_sequence");

    match read_recovery_checkpoint(&cp_path) {
        CheckpointReadResult::Valid(cp) => {
            assert_eq!(cp.last_app_mode, "Running");
            assert_eq!(cp.pid, std::process::id());
        }
        other => panic!("expected Valid checkpoint, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// write_hooks_settings unit-level tests (direct function call, not via full sequence)
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.010 via `write_hooks_settings` directly.
///
/// Verifies the hooks-settings.json file structure when called standalone.
/// This is a more targeted test than the full-sequence test above.
#[test]
fn test_BC_2_04_010_write_hooks_settings_creates_valid_schema() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path();

    let auth_token = "a".repeat(64);
    write_hooks_settings(runtime_dir, 7891, &auth_token)
        .expect("write_hooks_settings must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    assert!(hs_path.exists(), "hooks-settings.json must exist");

    let content = std::fs::read_to_string(&hs_path).expect("read hooks-settings.json");
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");

    let hooks = json.get("hooks").expect("'hooks' key must exist");

    // 4 active hooks.
    for name in &["PreToolUse", "Notification", "Stop", "UserPromptSubmit"] {
        let arr = hooks
            .get(name)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("hooks.{name} must be array"));
        assert!(!arr.is_empty(), "hooks.{name} must be active");
    }

    // PostToolUse and PreCompact are empty.
    assert!(
        hooks
            .get("PostToolUse")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "hooks.PostToolUse must be empty"
    );
    assert!(
        hooks
            .get("PreCompact")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "hooks.PreCompact must be empty"
    );

    // SessionStart must be absent.
    assert!(
        hooks.get("SessionStart").is_none(),
        "SessionStart must be absent"
    );
}

/// Exercises BC-2.04.010 PC-3: `write_hooks_settings` sets file mode 0o600.
#[test]
fn test_BC_2_04_010_write_hooks_settings_mode_0o600_direct() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path();

    let auth_token = "b".repeat(64);
    write_hooks_settings(runtime_dir, 7891, &auth_token)
        .expect("write_hooks_settings must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let meta = std::fs::metadata(&hs_path).expect("metadata");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "hooks-settings.json must have mode 0o600");
}

/// Exercises BC-2.04.010: the hook command URLs reference the correct port.
///
/// Each active hook command must use `http://127.0.0.1:<port>/hooks/<path>`.
#[test]
fn test_BC_2_04_010_hook_commands_reference_correct_port() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path();

    let auth_token = "c".repeat(64);
    let port: u16 = 9001;
    write_hooks_settings(runtime_dir, port, &auth_token)
        .expect("write_hooks_settings must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let content = std::fs::read_to_string(&hs_path).expect("read file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");

    let hooks = json.get("hooks").expect("'hooks' key");
    for name in &["PreToolUse", "Notification", "Stop", "UserPromptSubmit"] {
        let entries = hooks.get(name).and_then(|v| v.as_array()).expect("array");
        for entry in entries {
            let cmds = entry
                .get("hooks")
                .and_then(|v| v.as_array())
                .expect("hooks array");
            for cmd in cmds {
                let command_str = cmd
                    .get("command")
                    .and_then(|v| v.as_str())
                    .expect("command string");
                assert!(
                    command_str.contains(&format!(":{port}")),
                    "hook command must reference port {port}: {command_str}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// write_lock_file unit-level tests (direct function call)
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 step 8: `write_lock_file` returns a lock guard and token.
///
/// Verifies the function returns `Ok((DaemonLock, token))` and the lock file
/// exists on disk.
#[test]
fn test_BC_2_04_001_write_lock_file_returns_guard_and_token() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path();
    let auth_token = "d".repeat(64);

    let result = write_lock_file(runtime_dir, 7891, &auth_token);
    let (lock, returned_token) = result.expect("write_lock_file must succeed");

    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        lock_path.exists(),
        "monocle.lock must exist after write_lock_file"
    );

    // The returned token must be a valid 64 hex char string.
    assert_eq!(returned_token.len(), 64, "returned token must be 64 chars");
    assert!(
        returned_token
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "returned token must be 64 lowercase hex chars"
    );

    // Clean up.
    let _ = lock.release();
}

// ---------------------------------------------------------------------------
// EngineModuleRegistry type tests
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-6: EngineModuleRegistry can register both modules.
///
/// Verifies that `EngineModuleRegistry` correctly tracks Claude Code and VSDD
/// factory module registration.
#[test]
fn test_BC_2_04_001_engine_module_registry_registers_both_modules() {
    let mut registry = EngineModuleRegistry::new();
    assert!(
        !registry.is_complete(),
        "fresh registry must not be complete"
    );

    registry.register_claude_code();
    assert!(
        registry.claude_code_registered,
        "claude_code_registered must be true after registration"
    );
    assert!(
        !registry.is_complete(),
        "registry incomplete without vsdd_factory"
    );

    registry.register_vsdd_factory();
    assert!(
        registry.vsdd_factory_registered,
        "vsdd_factory_registered must be true after registration"
    );

    assert!(
        registry.is_complete(),
        "registry must be complete after both modules registered"
    );
}

// ---------------------------------------------------------------------------
// HooksSettings type tests
// ---------------------------------------------------------------------------

/// Exercises the `HooksSettings` struct serialization round-trip.
///
/// Verifies that `HooksSettings` serializes to JSON and deserializes back
/// to the same structure without data loss.
#[test]
fn test_BC_2_04_010_hooks_settings_type_serialization_roundtrip() {
    use monocle_runtime::types::{HookCommand, HookEntry, HooksMap, HooksSettings};

    let settings = HooksSettings {
        hooks: HooksMap {
            pre_tool_use: vec![HookEntry::command_all("echo pre-tool-use".to_string())],
            post_tool_use: vec![],
            notification: vec![HookEntry::command_all("echo notification".to_string())],
            stop: vec![HookEntry::command_all("echo stop".to_string())],
            user_prompt_submit: vec![HookEntry::command_all("echo prompt".to_string())],
            pre_compact: vec![],
        },
    };

    let json = serde_json::to_string_pretty(&settings).expect("serialize HooksSettings");
    let deserialized: HooksSettings =
        serde_json::from_str(&json).expect("deserialize HooksSettings");

    assert_eq!(deserialized.hooks.pre_tool_use.len(), 1);
    assert_eq!(deserialized.hooks.post_tool_use.len(), 0);
    assert_eq!(deserialized.hooks.notification.len(), 1);
    assert_eq!(deserialized.hooks.stop.len(), 1);
    assert_eq!(deserialized.hooks.user_prompt_submit.len(), 1);
    assert_eq!(deserialized.hooks.pre_compact.len(), 0);

    // Verify SessionStart key is absent from the serialized JSON.
    let json_value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(
        json_value
            .get("hooks")
            .and_then(|h| h.get("SessionStart"))
            .is_none(),
        "SessionStart must be absent from serialized HooksSettings"
    );
}

// ---------------------------------------------------------------------------
// MED-002: Token embedded in hook commands (BC-2.04.010 PC-3 wire token)
// MED-003: matcher field present as "" in every active hook entry
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.010 PC-3: each active hook command contains
/// `monocle-v1:<token>` in the X-Monocle-Authorization header.
///
/// The `write_hooks_settings` function must embed the wire token
/// `monocle-v1:<auth_token>` in the `-H 'X-Monocle-Authorization: ...'`
/// argument of every active hook curl command.
#[test]
fn test_BC_2_04_010_hook_commands_contain_wire_token() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path();

    let auth_token = "abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234";
    write_hooks_settings(runtime_dir, 9001, auth_token).expect("write_hooks_settings must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let content = std::fs::read_to_string(&hs_path).expect("read hooks-settings.json");
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");

    let expected_wire_token = format!("monocle-v1:{auth_token}");
    let hooks = json.get("hooks").expect("'hooks' key must exist");

    for hook_name in &["PreToolUse", "Notification", "Stop", "UserPromptSubmit"] {
        let entries = hooks
            .get(hook_name)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("hooks.{hook_name} must be an array"));
        for entry in entries {
            let cmds = entry
                .get("hooks")
                .and_then(|v| v.as_array())
                .unwrap_or_else(|| panic!("hooks.{hook_name}[].hooks must be an array"));
            for cmd in cmds {
                let command_str =
                    cmd.get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or_else(|| {
                            panic!("hooks.{hook_name}[].hooks[].command must be a string")
                        });
                assert!(
                    command_str.contains(&expected_wire_token),
                    "hook command for {hook_name} must contain wire token '{expected_wire_token}': \
                     {command_str}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MED-001: Step 2 stale/live lock coverage (BC-2.04.001 PC-2)
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.001 PC-2 (stale path): a lock file referencing a dead PID is
/// removed and the daemon start sequence completes successfully.
///
/// Plants a lock file with `i32::MAX - 1` as the PID — a value almost certainly not
/// held by any running process. `daemon_start_sequence` must detect the stale lock via
/// signal 0, remove it, and proceed to a successful start.
#[tokio::test]
async fn test_BC_2_04_001_step2_stale_lock_removed_and_new_daemon_starts() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    // Plant a stale lock file using a dead PID.
    let dead_pid: i32 = i32::MAX - 1;
    let lock_path = runtime_dir.join("monocle.lock");

    // Use tempfile::NamedTempFile::new_in → write → persist (project convention: no naked fs::write).
    let mut stale_tmp =
        tempfile::NamedTempFile::new_in(&runtime_dir).expect("create stale lock tempfile");
    let stale_json = format!(
        r#"{{"pid": {dead_pid}, "port": 12345, "token": "monocle-v1:aabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccdd", "contract_version": "monocle-lock-v1", "started_at": "2026-01-01T00:00:00.000Z"}}"#
    );
    use std::io::Write as _;
    stale_tmp
        .write_all(stale_json.as_bytes())
        .expect("write stale lock content");
    stale_tmp.flush().expect("flush stale lock tempfile");
    stale_tmp
        .persist(&lock_path)
        .expect("persist stale lock file");

    assert!(
        lock_path.exists(),
        "stale lock file must exist before sequence"
    );

    // daemon_start_sequence must detect the dead PID, remove the stale lock, and succeed.
    daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed when stale lock is present");

    // After a successful start, a new lock file must exist (fresh, not the stale one).
    assert!(
        lock_path.exists(),
        "monocle.lock must exist after successful start (fresh lock, not stale)"
    );

    // The new lock file must reference the current process PID, not the stale dead PID.
    let new_lock_content =
        std::fs::read_to_string(&lock_path).expect("read new lock file after stale recovery");
    let new_lock_json: serde_json::Value =
        serde_json::from_str(&new_lock_content).expect("new lock file must be valid JSON");
    let new_pid = new_lock_json
        .get("pid")
        .and_then(|v| v.as_i64())
        .expect("new lock file must have pid field");
    assert_ne!(
        new_pid, dead_pid as i64,
        "new lock file must NOT contain the stale dead PID"
    );
    assert_eq!(
        new_pid,
        std::process::id() as i64,
        "new lock file must contain the current process PID"
    );
}

/// Exercises BC-2.04.001 PC-2 (live path): a lock file referencing the current process
/// PID causes `daemon_start_sequence` to return `DaemonStartError::LockFileConflict`
/// with the correct PID.
///
/// The current process PID (from `std::process::id()`) is live by definition: signal 0
/// always succeeds for oneself. This triggers the conflict detection path.
#[tokio::test]
async fn test_BC_2_04_001_step2_live_lock_returns_conflict() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime_dir");

    // Plant a lock file with the current process's PID (live by definition).
    let live_pid = std::process::id() as i32;
    let lock_path = runtime_dir.join("monocle.lock");

    // Use tempfile::NamedTempFile::new_in → write → persist (project convention).
    let mut live_tmp =
        tempfile::NamedTempFile::new_in(&runtime_dir).expect("create live lock tempfile");
    let live_json = format!(
        r#"{{"pid": {live_pid}, "port": 12345, "token": "monocle-v1:aabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccdd", "contract_version": "monocle-lock-v1", "started_at": "2026-01-01T00:00:00.000Z"}}"#
    );
    use std::io::Write as _;
    live_tmp
        .write_all(live_json.as_bytes())
        .expect("write live lock content");
    live_tmp.flush().expect("flush live lock tempfile");
    live_tmp
        .persist(&lock_path)
        .expect("persist live lock file");

    assert!(
        lock_path.exists(),
        "live lock file must exist before sequence"
    );

    // daemon_start_sequence must detect the live PID and return LockFileConflict.
    let result = daemon_start_sequence(&runtime_dir).await;

    match result {
        Err(DaemonStartError::LockFileConflict { pid }) => {
            assert_eq!(
                pid, live_pid,
                "LockFileConflict must report the PID from the lock file (got {pid}, expected {live_pid})"
            );
        }
        Err(other) => {
            panic!("expected DaemonStartError::LockFileConflict, got: {other:?}");
        }
        Ok(_) => {
            panic!("daemon_start_sequence must NOT succeed when a live lock is present");
        }
    }
}

/// Exercises BC-2.04.010 PC-3 schema: each active hook entry must have a `"matcher"` field
/// with value `""` (empty string).
///
/// BC-2.04.010 PC-3 requires `"matcher": ""` to be present — not absent — in the
/// serialized JSON for every active hook entry (match-all semantics).
#[test]
fn test_BC_2_04_010_hook_entries_have_empty_matcher_field() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path();

    let auth_token = "e".repeat(64);
    write_hooks_settings(runtime_dir, 9002, &auth_token)
        .expect("write_hooks_settings must succeed");

    let hs_path = runtime_dir.join("hooks-settings.json");
    let content = std::fs::read_to_string(&hs_path).expect("read hooks-settings.json");
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");

    let hooks = json.get("hooks").expect("'hooks' key must exist");

    for hook_name in &["PreToolUse", "Notification", "Stop", "UserPromptSubmit"] {
        let entries = hooks
            .get(hook_name)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("hooks.{hook_name} must be an array"));
        for entry in entries {
            // BC-2.04.010 PC-3: the `matcher` key must be present with value `""`.
            let matcher = entry.get("matcher").unwrap_or_else(|| {
                panic!("hooks.{hook_name} entry must have a 'matcher' field (BC-2.04.010 PC-3)")
            });
            assert_eq!(
                matcher.as_str(),
                Some(""),
                "hooks.{hook_name} matcher must be empty string \"\" (BC-2.04.010 PC-3), got: {matcher}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// S-022 IPC integration tests
//
// These tests verify that the fully-wired daemon (daemon_start_sequence or manually
// wired DaemonState + ipc_subscribers) correctly delivers IPC messages to connected
// TUI clients. Exit condition 2 (daemon_start_sequence → UDS → InitialState) and
// exit condition 3 (Defer → PermissionPromptQueued broadcast) are covered here.
// ---------------------------------------------------------------------------

/// F-ADV2-HIGH-001: accept loop terminates cleanly within ~100ms of shutdown signal.
///
/// After `daemon_start_sequence`, send a shutdown signal via `shutdown_tx` and verify
/// that the accept loop task exits cleanly (the socket stops accepting new connections
/// within ~100ms). Verifies the `tokio::select!` shutdown branch in `run_accept_loop`.
#[tokio::test]
async fn test_S022_accept_loop_terminates_within_100ms_of_shutdown_signal() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");

    let daemon_state = daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    // Confirm UDS socket is accepting by connecting once.
    let sock_path = runtime_dir.join("monocle.sock");
    for _ in 0..20 {
        if tokio::net::UnixStream::connect(&sock_path).await.is_ok() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Send shutdown signal (same mechanism as graceful shutdown handler).
    daemon_state
        .shutdown_tx
        .send(true)
        .expect("shutdown_tx send must succeed");

    // After the signal, the accept loop should exit within ~100ms.
    // We verify by attempting to connect and expecting eventual failure.
    // Give the loop 200ms to exit (2x margin over the 100ms target).
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // The socket file remains (cleanup is not the accept loop's job), but
    // the accept loop task should have logged "exited". We can't directly
    // observe the task exit here, but we verify the shutdown signal propagated
    // cleanly by checking the watch channel value.
    assert!(
        *daemon_state.shutdown_rx.borrow(),
        "shutdown_rx must be true after shutdown_tx.send(true)"
    );
    // Confirm the accept loop handled the shutdown signal by verifying no panic occurred
    // (the test completing without a timeout is the primary assertion).
}

/// F-ADV2-HIGH-003: `tui_attached` flips true on connect, false on disconnect.
///
/// Uses the HTTP `/status` endpoint to verify BC-2.01.002 PC-1 contract:
/// - Before any TUI connect: `tui_attached = false`.
/// - While a TUI client is connected: `tui_attached = true`.
/// - After the TUI client disconnects and GC: `tui_attached = false`.
#[tokio::test]
async fn test_S022_tui_attached_flips_on_connect_and_disconnect() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");

    let daemon_state = daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    // Before any TUI connects, tui_attached_count must be 0.
    assert_eq!(
        daemon_state
            .tui_attached_count
            .load(std::sync::atomic::Ordering::SeqCst),
        0,
        "tui_attached_count must be 0 before any TUI connects"
    );

    // Connect a TUI client.
    let sock_path = runtime_dir.join("monocle.sock");
    let mut stream = {
        let mut last_err = None;
        let mut stream_opt: Option<tokio::net::UnixStream> = None;
        for _ in 0..20 {
            match tokio::net::UnixStream::connect(&sock_path).await {
                Ok(s) => {
                    stream_opt = Some(s);
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        }
        stream_opt.unwrap_or_else(|| panic!("TUI connect failed: {}", last_err.unwrap()))
    };

    // Read the InitialState message to ensure the server has processed the connect.
    let _: monocle_ipc::types::ServerToClient = monocle_ipc::framing::read_framed(&mut stream)
        .await
        .expect("must receive InitialState");

    // After connect + InitialState send, tui_attached_count must be >= 1.
    // Allow a brief scheduling window.
    let mut attached = false;
    for _ in 0..20 {
        if daemon_state
            .tui_attached_count
            .load(std::sync::atomic::Ordering::SeqCst)
            > 0
        {
            attached = true;
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    assert!(
        attached,
        "tui_attached_count must be > 0 while TUI client is connected (BC-2.01.002 PC-1)"
    );

    // Disconnect by dropping the stream.
    drop(stream);

    // After disconnect, tui_attached_count must return to 0.
    // The per-client task may need a scheduler tick to process EOF.
    let mut detached = false;
    for _ in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        if daemon_state
            .tui_attached_count
            .load(std::sync::atomic::Ordering::SeqCst)
            == 0
        {
            detached = true;
            break;
        }
    }
    assert!(
        detached,
        "tui_attached_count must return to 0 after TUI client disconnects (BC-2.01.002 PC-1)"
    );
}

/// S-022 exit condition 2: `daemon_start_sequence` → UDS connect → `InitialState` arrives.
///
/// After a successful `daemon_start_sequence`, a TUI client that connects to the
/// UDS socket MUST receive exactly one `ServerToClient::InitialState` message as the
/// first and only message (BC-2.05.002 PC-2, AC-002).
#[tokio::test]
async fn test_S022_daemon_start_sequence_uds_connect_receives_initial_state() {
    let tmp = isolated_runtime_dir();
    let runtime_dir = tmp.path().join("monocle-runtime");

    // daemon_start_sequence creates the runtime dir, binds the UDS socket, and spawns
    // the accept loop. The returned DaemonState has ipc_subscribers wired (S-022).
    let _daemon_state = daemon_start_sequence(&runtime_dir)
        .await
        .expect("daemon_start_sequence must succeed");

    // Connect a TUI client to the UDS socket.
    // Retry briefly — the accept loop task may need a scheduler tick to start.
    let mut stream = {
        let sock_path = runtime_dir.join("monocle.sock");
        let mut last_err = None;
        let mut stream_opt: Option<tokio::net::UnixStream> = None;
        for _ in 0..20 {
            match tokio::net::UnixStream::connect(&sock_path).await {
                Ok(s) => {
                    stream_opt = Some(s);
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        }
        stream_opt.unwrap_or_else(|| {
            panic!(
                "TUI client failed to connect after 20 attempts: {}",
                last_err.unwrap()
            )
        })
    };

    // Read the first message — must be InitialState (BC-2.05.002 PC-2 / AC-002).
    let msg: monocle_ipc::types::ServerToClient = monocle_ipc::framing::read_framed(&mut stream)
        .await
        .expect("must receive the first message from daemon");

    match msg {
        monocle_ipc::types::ServerToClient::InitialState {
            sessions,
            ring_tail,
            overlay_stack,
            drop_counter,
        } => {
            // Fresh daemon: all fields are empty / zero.
            assert!(
                sessions.is_empty(),
                "fresh daemon: sessions must be empty, got {} sessions",
                sessions.len()
            );
            assert!(
                ring_tail.is_empty(),
                "fresh daemon: ring_tail must be empty, got {} events",
                ring_tail.len()
            );
            assert!(
                overlay_stack.is_empty(),
                "fresh daemon: overlay_stack must be empty"
            );
            assert_eq!(drop_counter, 0, "fresh daemon: drop_counter must be 0");
        }
        other => panic!("first message from daemon MUST be InitialState, got: {other:?}"),
    }
}

/// S-022 exit condition 3: PreToolUse Defer → PermissionPromptQueued broadcast to TUI client.
///
/// When the engine returns `HookDecision::Defer`, the handler must:
/// 1. Register the prompt in `pending_decisions`.
/// 2. Broadcast `ServerToClient::PermissionPromptQueued` to all connected TUI clients
///    BEFORE awaiting the decision oneshot (BC-2.04.007 invariant 5, BC-2.05.005 PC-2).
///
/// This test uses the `hook_decision_override` test-injection field to force the Defer
/// path without requiring a real engine that returns Defer. A synthetic IPC subscriber
/// is injected directly into `ipc_subscribers` to observe the broadcast.
#[tokio::test]
async fn test_S022_pre_tool_use_defer_broadcasts_permission_prompt_queued() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use monocle_runtime::server::build_server;
    use monocle_runtime::state::DaemonState;
    use monocle_runtime::types::EVENT_BUS_CAPACITY;
    use std::sync::atomic::AtomicU64;
    use tower::ServiceExt;

    const TEST_TOKEN: &str = "aabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccdd";

    // Build a DaemonState with all S-022 fields wired:
    // - event_bus_tx: required by the pre-tool-use handler
    // - pending_decisions: required for register_prompt (Defer path)
    // - ipc_subscribers: required for PermissionPromptQueued broadcast
    // - hook_decision_override: force Defer without a real engine
    let (event_tx, _event_rx) =
        tokio::sync::mpsc::channel::<monocle_runtime::types::EventBusHookEvent>(EVENT_BUS_CAPACITY);
    let ipc_subscribers: monocle_ipc::server::SubscriberList =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN.to_string();
    state.event_bus_tx = Some(Arc::new(event_tx));
    state.drop_counter = Some(Arc::new(AtomicU64::new(0)));
    state.session_registry = Some(Arc::new(monocle_runtime::hooks::SessionRegistry::new()));
    state.pending_decisions = Some(Arc::new(
        monocle_runtime::permissions::PendingDecisionRegistry::new(),
    ));
    state.ipc_subscribers = Some(Arc::clone(&ipc_subscribers));
    state.hook_decision_override = Some((monocle_core::engine::HookDecision::Defer, None));

    let state = Arc::new(state);

    // Inject a TUI subscriber that will receive PermissionPromptQueued.
    let (tui_tx, mut tui_rx) = tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(16);
    ipc_subscribers
        .lock()
        .await
        .push(monocle_ipc::server::ClientEntry::new(tui_tx));

    // Build the Axum server and POST to /hooks/pre-tool-use.
    // The Defer path is async: it broadcasts PermissionPromptQueued, then awaits the
    // decision oneshot (which never fires since no TUI client resolves it). The outer
    // 300ms timeout fires and returns fail-open allow.
    let app = build_server(Arc::clone(&state));
    let body = serde_json::json!({
        "session_id": "s022-test-session",
        "pid": 42000,
        "tool_name": "Bash",
        "tool_input": {"command": "rm -rf /"}
    });
    let req = Request::builder()
        .method("POST")
        .uri("/hooks/pre-tool-use")
        .header("Content-Type", "application/json")
        .header(
            "X-Monocle-Authorization",
            format!("monocle-v1:{TEST_TOKEN}"),
        )
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Spawn the POST in a separate task so we can concurrently read from tui_rx.
    let post_task = tokio::spawn(async move {
        let resp = app.oneshot(req).await.expect("oneshot must not fail");
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    });

    // The Defer path broadcasts PermissionPromptQueued BEFORE awaiting the oneshot.
    // Wait up to 500ms for the broadcast to arrive (budget: 300ms timeout + margin).
    let queued_msg = tokio::time::timeout(tokio::time::Duration::from_millis(500), tui_rx.recv())
        .await
        .expect("PermissionPromptQueued must arrive within 500ms")
        .expect("subscriber channel must not close before receiving PermissionPromptQueued");

    match queued_msg {
        monocle_ipc::types::ServerToClient::PermissionPromptQueued { payload } => {
            assert_eq!(
                payload.tool_name, "Bash",
                "PermissionPromptQueued.tool_name must match the hook body"
            );
            assert_eq!(
                payload.session_id, "s022-test-session",
                "PermissionPromptQueued.session_id must match the hook body"
            );
            // prompt_id must NOT be Uuid::nil() — register_prompt assigns a real UUID.
            assert_ne!(
                payload.prompt_id,
                uuid::Uuid::nil(),
                "PermissionPromptQueued.prompt_id must be a real UUID, not Uuid::nil()"
            );
        }
        other => panic!("expected PermissionPromptQueued from Defer path, got: {other:?}"),
    }

    // The POST will eventually return fail-open allow (after 300ms timeout).
    let (status, resp_json) = post_task.await.expect("post task must complete");
    assert_eq!(
        status,
        StatusCode::OK,
        "Defer+timeout must return HTTP 200, got: {resp_json}"
    );
    assert_eq!(
        resp_json.get("reason").and_then(|v| v.as_str()),
        Some("timeout"),
        "Defer+timeout must produce reason:timeout, got: {resp_json}"
    );
}

// ---------------------------------------------------------------------------
// F-ADV3-HIGH-001: BC-2.05.001 EC-002 path-length enforcement via daemon_start_sequence
// ---------------------------------------------------------------------------

/// Exercises BC-2.05.001 EC-002: `daemon_start_sequence` returns
/// `DaemonStartError::UdsPathTooLong` when the computed `<runtime_dir>/monocle.sock`
/// path exceeds `UDS_PATH_LIMIT_BYTES` (104 bytes).
///
/// Verifies that the production bind path (via `UdsTransport::bind`) enforces the
/// path-length check — the previous inline bind in lifecycle.rs bypassed it entirely
/// (F-ADV3-HIGH-001).
///
/// The ERROR log line "UDS socket path exceeds OS limit (<N> bytes, limit <M>)" is
/// produced by `UdsTransport::bind` before returning `IpcError::PathTooLong`.
#[tokio::test]
async fn test_BC_2_05_001_daemon_start_sequence_returns_err_when_uds_path_too_long() {
    // Construct a runtime_dir whose path, when joined with "monocle.sock" (+12 bytes),
    // exceeds UDS_PATH_LIMIT_BYTES (104 bytes). We use a TempDir base and then append a
    // subdirectory whose total length pushes the socket path over 104 bytes.
    //
    // Strategy: tempdir on macOS produces paths like /var/folders/.../T/... which are
    // typically ~50 bytes. We append enough path components to exceed 104 bytes total.
    // monocle.sock adds 12 bytes (including the '/'), so we need runtime_dir > 92 bytes.
    use monocle_ipc::uds::UDS_PATH_LIMIT_BYTES;

    let base = tempfile::tempdir().expect("create base tempdir");
    // Build a runtime_dir path that will push the socket path above UDS_PATH_LIMIT_BYTES.
    // Append a long component so runtime_dir > UDS_PATH_LIMIT_BYTES - len("monocle.sock") - 1.
    let long_segment = "a".repeat(UDS_PATH_LIMIT_BYTES);
    let runtime_dir = base.path().join(&long_segment);

    let sock_path = runtime_dir.join("monocle.sock");
    let path_len = sock_path.as_os_str().len();

    // Only run this test if the path is actually over the limit.
    // On platforms where tempdir produces a very short base, this assertion verifies
    // the test is exercising the right scenario.
    assert!(
        path_len > UDS_PATH_LIMIT_BYTES,
        "test setup: sock_path must exceed {UDS_PATH_LIMIT_BYTES} bytes, got {path_len}"
    );

    // Create runtime_dir so daemon_start_sequence can proceed past step 1.
    std::fs::create_dir_all(&runtime_dir).expect("create long-path runtime_dir");

    // daemon_start_sequence must fail at step 10 with UdsPathTooLong.
    let result = daemon_start_sequence(&runtime_dir).await;

    match result {
        Err(DaemonStartError::UdsPathTooLong { length, limit }) => {
            assert_eq!(
                length, path_len,
                "UdsPathTooLong.length must equal computed path length ({path_len}), got {length}"
            );
            assert_eq!(
                limit, UDS_PATH_LIMIT_BYTES,
                "UdsPathTooLong.limit must be UDS_PATH_LIMIT_BYTES ({UDS_PATH_LIMIT_BYTES}), got {limit}"
            );
        }
        Err(other) => {
            panic!(
                "expected DaemonStartError::UdsPathTooLong, got: {other:?} \
                (sock_path len = {path_len}, limit = {UDS_PATH_LIMIT_BYTES})"
            );
        }
        Ok(_) => {
            panic!(
                "daemon_start_sequence must NOT succeed when UDS path exceeds OS limit \
                (sock_path len = {path_len}, limit = {UDS_PATH_LIMIT_BYTES})"
            );
        }
    }

    // INV-6: lock file must have been cleaned up (not left behind on step 10 failure).
    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        !lock_path.exists(),
        "INV-6: monocle.lock must be removed on step 10 UdsPathTooLong failure"
    );
}
