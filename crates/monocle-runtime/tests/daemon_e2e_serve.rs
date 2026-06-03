//! E2E live-binary integration test: daemon serves HTTP + UDS.
//!
//! Traces to SS-daemon-wiring-impl.md §E2E Verification Contract.
//!
//! Spawns the real `monocle-runtime` binary with an isolated MONOCLE_RUNTIME_DIR
//! and asserts the full startup → serve → shutdown lifecycle:
//!
//! | AC       | Contract |
//! |----------|---------|
//! | AC-E2E-001 | Lock file appears with real OS-assigned (non-39001) port |
//! | AC-E2E-002 | hooks-settings.json references the real port |
//! | AC-E2E-003 | GET /healthz → 200 |
//! | AC-E2E-004 | POST /hooks/pre-tool-use with auth → 200 |
//! | AC-E2E-005 | monocle.sock exists and is a socket |
//! | AC-E2E-006 | SIGTERM → exit 0 + hooks-settings.json + monocle.lock removed |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per naming convention.
#![allow(non_snake_case)]

use std::os::unix::fs::FileTypeExt as _;
use std::time::Duration;

/// Poll for a path to appear on the filesystem.
///
/// Returns `true` if the file appears within `max_wait`, `false` on timeout.
fn wait_for_file(path: &std::path::Path, max_wait: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < max_wait {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

/// Poll for the daemon to respond to GET /healthz with HTTP 200.
///
/// Returns `true` if /healthz returns 200 within `max_wait`.
fn wait_for_healthz(port: u16, max_wait: Duration) -> bool {
    let url = format!("http://127.0.0.1:{port}/healthz");
    let start = std::time::Instant::now();
    while start.elapsed() < max_wait {
        if let Ok(resp) = ureq::get(&url).call() {
            if resp.status() == 200 {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

/// Full lifecycle E2E test for the monocle-runtime binary.
///
/// Asserts AC-E2E-001 through AC-E2E-006: startup, HTTP service, UDS socket,
/// SIGTERM graceful shutdown, and filesystem cleanup.
///
/// This test spawns the REAL binary built by cargo and exercises the true
/// production path. It replaces any verification that relied on the stub's
/// sleep loop.
#[test]
fn test_daemon_e2e_serve_lifecycle() {
    // Locate the binary. CARGO_BIN_EXE_monocle-runtime is set by cargo when running
    // integration tests for a crate that has a [[bin]] target named "monocle-runtime".
    let binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_monocle-runtime"));
    assert!(
        binary.exists(),
        "monocle-runtime binary not found at {binary:?}; run `cargo build -p monocle-runtime` first"
    );

    // Create an isolated temp directory for this test run.
    let tmp = tempfile::tempdir().expect("create tmpdir for E2E test");
    let runtime_dir = tmp.path().to_path_buf();

    // Spawn the daemon.
    let mut child = std::process::Command::new(&binary)
        .env("MONOCLE_RUNTIME_DIR", &runtime_dir)
        // Suppress daemon stderr so test output is clean; set RUST_LOG for visibility on failure.
        .env("RUST_LOG", "monocle_runtime=info")
        .spawn()
        .expect("spawn monocle-runtime binary");

    let daemon_pid = child.id();

    // -----------------------------------------------------------------------
    // AC-E2E-001: Lock file appears with a real (non-zero, non-39001) port.
    // -----------------------------------------------------------------------
    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        wait_for_file(&lock_path, Duration::from_secs(5)),
        "AC-E2E-001 FAIL: monocle.lock did not appear within 5s in {runtime_dir:?}"
    );

    let lock_content = std::fs::read_to_string(&lock_path).expect("AC-E2E-001: read monocle.lock");
    let lock_json: serde_json::Value =
        serde_json::from_str(&lock_content).expect("AC-E2E-001: monocle.lock must be valid JSON");

    let port = lock_json
        .get("port")
        .and_then(|v| v.as_u64())
        .and_then(|p| u16::try_from(p).ok())
        .expect("AC-E2E-001: monocle.lock must have a u16 'port' field");

    assert_ne!(port, 0, "AC-E2E-001: port must not be 0");
    assert_ne!(
        port, 39001,
        "AC-E2E-001: port must not be the hardcoded 39001"
    );

    let contract_version = lock_json
        .get("contract_version")
        .and_then(|v| v.as_str())
        .expect("AC-E2E-001: lock file must have 'contract_version'");
    assert_eq!(
        contract_version, "monocle-lock-v1",
        "AC-E2E-001: contract_version must be 'monocle-lock-v1'"
    );

    let lock_pid = lock_json
        .get("pid")
        .and_then(|v| v.as_u64())
        .expect("AC-E2E-001: lock file must have 'pid'");
    assert_eq!(
        lock_pid, daemon_pid as u64,
        "AC-E2E-001: lock file pid must match spawned process pid"
    );

    // Extract the raw hex token (strip monocle-v1: prefix).
    let wire_token = lock_json
        .get("token")
        .and_then(|v| v.as_str())
        .expect("AC-E2E-001: lock file must have 'token'");
    assert!(
        wire_token.starts_with("monocle-v1:"),
        "AC-E2E-001: token must start with 'monocle-v1:' prefix"
    );
    let hex_token = wire_token
        .strip_prefix("monocle-v1:")
        .expect("just asserted starts_with");

    // -----------------------------------------------------------------------
    // AC-E2E-002: hooks-settings.json references the real port.
    // -----------------------------------------------------------------------
    let hs_path = runtime_dir.join("hooks-settings.json");
    assert!(
        hs_path.exists(),
        "AC-E2E-002: hooks-settings.json must exist after startup"
    );

    let hs_content =
        std::fs::read_to_string(&hs_path).expect("AC-E2E-002: read hooks-settings.json");
    assert!(
        hs_content.contains(&format!(":{port}/")),
        "AC-E2E-002: hooks-settings.json must contain ':{port}/' (the real OS port)"
    );
    assert!(
        !hs_content.contains("39001"),
        "AC-E2E-002: hooks-settings.json must NOT contain the hardcoded port 39001"
    );

    // -----------------------------------------------------------------------
    // AC-E2E-003: GET /healthz returns 200.
    // -----------------------------------------------------------------------
    assert!(
        wait_for_healthz(port, Duration::from_secs(5)),
        "AC-E2E-003: GET http://127.0.0.1:{port}/healthz did not return 200 within 5s"
    );

    // Confirm the body.
    let healthz_resp = ureq::get(&format!("http://127.0.0.1:{port}/healthz"))
        .call()
        .expect("AC-E2E-003: GET /healthz must succeed");
    assert_eq!(
        healthz_resp.status(),
        200,
        "AC-E2E-003: GET /healthz must return HTTP 200"
    );
    let body = healthz_resp
        .into_string()
        .expect("AC-E2E-003: read /healthz body");
    // Body must contain "status" field per healthz handler spec (returns "alive" when Running).
    assert!(
        body.contains("status"),
        "AC-E2E-003: /healthz response body must contain 'status' field, got: {body}"
    );

    // -----------------------------------------------------------------------
    // AC-E2E-004: POST /hooks/pre-tool-use with auth → 200.
    // -----------------------------------------------------------------------
    let hook_url = format!("http://127.0.0.1:{port}/hooks/pre-tool-use");
    // Minimal valid HookEnvelope for pre-tool-use.
    let hook_body = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "session_id": "e2e-test-session",
        "tool_name": "Read",
        "tool_input": {}
    });

    let hook_resp = ureq::post(&hook_url)
        .set("Content-Type", "application/json")
        .set(
            "X-Monocle-Authorization",
            &format!("monocle-v1:{hex_token}"),
        )
        .send_json(hook_body)
        .expect("AC-E2E-004: POST /hooks/pre-tool-use must not fail at transport layer");

    assert_eq!(
        hook_resp.status(),
        200,
        "AC-E2E-004: POST /hooks/pre-tool-use with valid auth must return 200"
    );

    // -----------------------------------------------------------------------
    // AC-E2E-005: monocle.sock exists and is a Unix socket.
    // -----------------------------------------------------------------------
    let sock_path = runtime_dir.join("monocle.sock");
    assert!(
        sock_path.exists(),
        "AC-E2E-005: monocle.sock must exist after startup"
    );
    let sock_meta = std::fs::metadata(&sock_path).expect("AC-E2E-005: stat monocle.sock");
    assert!(
        sock_meta.file_type().is_socket(),
        "AC-E2E-005: monocle.sock must be a Unix socket (is_socket() returned false)"
    );

    // -----------------------------------------------------------------------
    // AC-E2E-006: SIGTERM → exit 0 + cleanup of hooks-settings.json + monocle.lock.
    // -----------------------------------------------------------------------
    // Send SIGTERM.
    unsafe {
        libc::kill(daemon_pid as libc::pid_t, libc::SIGTERM);
    }

    // Wait for exit (up to 5 seconds for the graceful drain to complete).
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let exit_status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    child.kill().ok();
                    panic!("AC-E2E-006: daemon did not exit within 5s after SIGTERM");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => panic!("AC-E2E-006: wait() failed: {e}"),
        }
    };

    assert!(
        exit_status.success(),
        "AC-E2E-006: daemon must exit 0 on SIGTERM (got: {exit_status:?})"
    );

    assert!(
        !hs_path.exists(),
        "AC-E2E-006: hooks-settings.json must be removed on graceful shutdown (BC-2.04.010 PC-5)"
    );
    assert!(
        !lock_path.exists(),
        "AC-E2E-006: monocle.lock must be removed on graceful shutdown (BC-2.01.004 PC-7)"
    );
}
