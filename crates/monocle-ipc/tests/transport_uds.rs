//! UDS transport lifecycle tests for `monocle-ipc` (BC-2.05.001).
// Test function names follow the VSDD `test_BC_S_SS_NNN_xxx` convention (uppercase BC).
// Suppress non_snake_case lint for test files — BC_ prefix is intentional for traceability.
#![allow(non_snake_case)]
//!
//! Tests the UDS socket bind, stale socket removal, mode 0o600, path length limit,
//! and graceful shutdown cleanup.
//!
//! All test names follow `test_BC_S_SS_NNN_[description]` convention.
//! These tests call `UdsTransport::bind(...)` which is a `todo!()` stub — they MUST FAIL
//! at runtime to satisfy the Red Gate requirement.

use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use monocle_ipc::uds::UdsTransport;

// ---------------------------------------------------------------------------
// BC-2.05.001 — UDS bind lifecycle
// ---------------------------------------------------------------------------

/// test_BC_2_05_001_uds_bind_creates_socket_with_mode_0o600:
/// Happy path: `UdsTransport::bind` creates the socket file at `<runtime_dir>/monocle.sock`
/// with mode 0o600.
///
/// Traces to BC-2.05.001 PC-1 (bind) and PC-2 (mode 0o600).
///
/// **RED GATE**: This test calls `UdsTransport::bind` which panics with `todo!()`.
#[tokio::test]
async fn test_BC_2_05_001_uds_bind_creates_socket_with_mode_0o600() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = dir.path();

    // Bind the UDS transport — stub will panic with todo!() until implemented.
    let transport = UdsTransport::bind(runtime_dir)
        .await
        .expect("bind should succeed on a fresh tempdir");

    let sock_path = runtime_dir.join("monocle.sock");
    assert!(sock_path.exists(), "monocle.sock must exist after bind");

    let meta = std::fs::metadata(&sock_path).expect("stat monocle.sock");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "monocle.sock permissions must be 0o600 (owner-only), got 0o{mode:03o}"
    );

    // Transport must report the correct sock_path.
    assert_eq!(transport.sock_path(), sock_path);
}

/// test_BC_2_05_001_uds_bind_removes_stale_socket_before_rebind:
/// When `monocle.sock` already exists (stale socket from a prior crash),
/// `UdsTransport::bind` removes it and rebinds.
///
/// Traces to BC-2.05.001 PC-3 / EC-001.
///
/// **RED GATE**: Calls `UdsTransport::bind` which panics with `todo!()`.
#[tokio::test]
async fn test_BC_2_05_001_uds_bind_removes_stale_socket_before_rebind() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = dir.path();
    let sock_path = runtime_dir.join("monocle.sock");

    // Plant a stale socket file.
    std::fs::write(&sock_path, b"stale").expect("write stale socket");
    assert!(sock_path.exists(), "stale socket must exist before bind");

    // Bind should remove the stale file and create a real socket.
    let transport = UdsTransport::bind(runtime_dir)
        .await
        .expect("bind should succeed after removing stale socket");

    // Socket must exist and be a real socket (not the planted regular file).
    assert!(
        sock_path.exists(),
        "monocle.sock must exist after bind (stale removed)"
    );

    // Mode must be 0o600 — not whatever the stale file had.
    let meta = std::fs::metadata(&sock_path).expect("stat");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "mode must be 0o600 after stale-socket rebind");

    // The transport's sock_path must be the expected path.
    assert_eq!(transport.sock_path(), sock_path);
}

/// test_BC_2_05_001_uds_bind_cleanup_removes_socket_on_shutdown:
/// `UdsTransport::cleanup()` removes the socket file from the filesystem.
///
/// Traces to BC-2.05.001 PC-4 (graceful shutdown removal).
///
/// **RED GATE**: Calls `UdsTransport::bind` + `cleanup()` which panic with `todo!()`.
#[tokio::test]
async fn test_BC_2_05_001_uds_bind_cleanup_removes_socket_on_shutdown() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = dir.path();

    let transport = UdsTransport::bind(runtime_dir).await.expect("bind");

    let sock_path = runtime_dir.join("monocle.sock");
    assert!(sock_path.exists(), "socket must exist before cleanup");

    transport.cleanup();

    assert!(
        !sock_path.exists(),
        "monocle.sock must be removed after cleanup()"
    );
}

/// test_BC_2_05_001_uds_bind_returns_error_on_path_too_long:
/// When the computed UDS socket path exceeds `UDS_PATH_LIMIT_BYTES`, `bind` returns
/// `IpcError::PathTooLong` without attempting any filesystem operations.
///
/// Traces to BC-2.05.001 EC-002 / AC-016.
///
/// **RED GATE**: Calls `UdsTransport::bind` which panics with `todo!()`.
#[tokio::test]
async fn test_BC_2_05_001_uds_bind_returns_error_on_path_too_long() {
    // Construct a runtime_dir whose path + "/monocle.sock" (12 bytes) exceeds 104 bytes.
    // We need runtime_dir itself to be >= 93 bytes (93 + 1 separator + 11 = 105 bytes).
    let dir = tempfile::tempdir().expect("tempdir");
    // Construct a subdirectory path that's artificially long.
    // Use a path component that is long enough to push past the limit.
    let long_component = "a".repeat(100); // 100 bytes
    let long_dir = dir.path().join(&long_component);
    std::fs::create_dir_all(&long_dir).expect("create long dir");

    // long_dir path + "/monocle.sock" should exceed UDS_PATH_LIMIT_BYTES.
    let sock_path = long_dir.join("monocle.sock");
    let sock_path_str = sock_path.to_string_lossy();
    let path_len = sock_path_str.len();

    // Only run this test if the constructed path actually exceeds the limit.
    if path_len <= monocle_ipc::uds::UDS_PATH_LIMIT_BYTES {
        // Skip: tempdir path on this system is too short to construct a long enough path.
        // This is a test environment constraint, not a code defect.
        return;
    }

    let result = UdsTransport::bind(&long_dir).await;

    assert!(
        result.is_err(),
        "bind must return an error when socket path exceeds OS limit"
    );
    match result.unwrap_err() {
        monocle_ipc::error::IpcError::PathTooLong { length, limit } => {
            assert!(
                length > limit,
                "reported length ({length}) must exceed reported limit ({limit})"
            );
        }
        other => panic!("expected IpcError::PathTooLong, got {other:?}"),
    }
}

/// test_BC_2_05_001_uds_bind_socket_path_construction_uses_path_join:
/// `UdsTransport::bind` constructs the socket path via `Path::new(runtime_dir).join("monocle.sock")`
/// (never via string concatenation).
///
/// This is validated by verifying that the resulting sock_path matches the expected join.
///
/// **RED GATE**: Calls `UdsTransport::bind` which panics with `todo!()`.
#[tokio::test]
async fn test_BC_2_05_001_uds_bind_socket_path_construction_uses_path_join() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = dir.path();
    let expected_sock_path = Path::new(runtime_dir).join("monocle.sock");

    let transport = UdsTransport::bind(runtime_dir).await.expect("bind");
    assert_eq!(
        transport.sock_path(),
        expected_sock_path,
        "sock_path must equal Path::new(runtime_dir).join('monocle.sock')"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.008 — Transport trait compile-time assertion
// ---------------------------------------------------------------------------

/// test_BC_2_05_008_uds_only_constraint_no_forbidden_files_in_src:
/// The monocle-ipc/src directory must not contain any files with forbidden patterns.
///
/// This is a static analysis test that reads source files and asserts the absence of
/// forbidden shared-memory symbols.
///
/// BC-2.05.008 PC-4 / AC-014.
#[test]
fn test_BC_2_05_008_uds_only_constraint_no_forbidden_files_in_src() {
    // Read each source file in crates/monocle-ipc/src/ and check for forbidden patterns.
    let forbidden = [
        "libc::mmap",
        "nix::sys::mman",
        "shm_open",
        "mmap_rs",
        "memmap2",
        "shared_memory",
        "raw-sync",
        "ipc-channel",
    ];

    // Resolve the crate's src directory relative to the test binary location.
    // In cargo test, the manifest dir is available via the CARGO_MANIFEST_DIR env.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = std::path::Path::new(manifest_dir).join("src");

    for entry in walkdir_src(&src_dir) {
        let contents = std::fs::read_to_string(&entry)
            .unwrap_or_else(|e| panic!("read {}: {e}", entry.display()));
        for pattern in &forbidden {
            assert!(
                !contents.contains(pattern),
                "monocle-ipc/src/{} must not contain forbidden pattern '{pattern}' (BC-2.05.008 PC-4)",
                entry.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }
}

/// Walk the src directory and return all .rs files.
fn walkdir_src(src_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

/// test_BC_2_05_008_forbid_unsafe_code_attribute_present:
/// The `#![forbid(unsafe_code)]` attribute must be the first crate-level attribute in lib.rs.
///
/// This is a static analysis test that reads the lib.rs source file.
/// BC-2.05.008 PC-1 / AC-012.
#[test]
fn test_BC_2_05_008_forbid_unsafe_code_attribute_present() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_rs = std::path::Path::new(manifest_dir).join("src/lib.rs");
    let contents = std::fs::read_to_string(&lib_rs).expect("read src/lib.rs — file must exist");

    // Find the first non-whitespace, non-comment content.
    // The first attribute must be #![forbid(unsafe_code)].
    let first_attr_line = contents
        .lines()
        .find(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .expect("lib.rs must have non-comment content");

    assert_eq!(
        first_attr_line.trim(),
        "#![forbid(unsafe_code)]",
        "first non-comment, non-blank line of lib.rs must be `#![forbid(unsafe_code)]`"
    );
}
