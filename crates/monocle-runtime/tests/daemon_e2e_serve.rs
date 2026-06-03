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
//! | AC-E2E-004 | POST /hooks/pre-tool-use with auth → 200 + ring ingestion verified |
//! | AC-E2E-005 | monocle.sock exists and is a socket |
//! | AC-E2E-006 | SIGTERM → exit 0 + hooks-settings.json + monocle.lock removed |
//! | AC-E2E-007 | Drain timeout fires ≤11s under a held in-flight request (C2 fix) |
//! | AC-E2E-008 | Ring flush drain-join is non-vacuous: 500ms flush delay + immediate SIGTERM → record durable (CRITICAL-1 fix) |

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

    // I2: Verify end-to-end ring ingestion — the handler appends a record to the JSONL ring
    // independently of the event bus. The ring uses an async write queue, so the file may not
    // appear synchronously with the 200 response. Poll up to 2s for the file to appear and
    // contain a PreToolUse record (SS-daemon-wiring-impl.md §Fix Addendum I2).
    let ring_path = runtime_dir.join("monocle-events.jsonl");
    let ring_content = {
        let start = std::time::Instant::now();
        let max_wait = Duration::from_secs(2);
        loop {
            if ring_path.exists() {
                let content = std::fs::read_to_string(&ring_path)
                    .expect("AC-E2E-004 (I2): read monocle-events.jsonl");
                let found = content.lines().any(|line| {
                    serde_json::from_str::<serde_json::Value>(line)
                        .ok()
                        .and_then(|rec| {
                            rec.get("hook_type")
                                .and_then(|v| v.as_str())
                                .map(|s| s == "PreToolUse")
                        })
                        .unwrap_or(false)
                });
                if found {
                    break content;
                }
            }
            if start.elapsed() >= max_wait {
                let content = if ring_path.exists() {
                    std::fs::read_to_string(&ring_path).unwrap_or_default()
                } else {
                    String::from("<file not found>")
                };
                panic!(
                    "AC-E2E-004 (I2): monocle-events.jsonl did not contain a PreToolUse record \
                     within 2s after POST /hooks/pre-tool-use; content was:\n{content}"
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    assert!(
        ring_content.contains("PreToolUse"),
        "AC-E2E-004 (I2): sanity check — ring_content must contain 'PreToolUse'"
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

/// AC-E2E-007 — Drain timeout fires within ~11s under a held in-flight handler.
///
/// Verifies BC-2.01.004 INV-1: when a hook handler is holding an in-flight HTTP request
/// during the graceful-shutdown drain window (simulated via MONOCLE_HOOK_DELAY_MS=15000
/// which sets `hook_outer_delay_ms` on DaemonState), the daemon force-closes after ≤10s
/// and completes cleanup within 11s total. Tests the `tokio::time::timeout` wrapper in
/// main() (C2 fix — SS-daemon-wiring-impl.md §Fix Addendum C2).
///
/// The `MONOCLE_HOOK_DELAY_MS` env var sets `hook_outer_delay_ms` in `DaemonState`,
/// which is checked at the OUTER handler level (before the 300ms inner timeout budget).
/// This creates a genuinely slow in-flight request that holds axum's graceful shutdown open.
///
/// Test setup:
/// 1. Spawn daemon with MONOCLE_HOOK_DELAY_MS=15000.
/// 2. Issue a POST /hooks/pre-tool-use with valid auth in a background thread.
///    The handler sleeps 15s before returning (outer delay), creating a true in-flight request.
/// 3. Send SIGTERM while the request is in-flight.
/// 4. Assert daemon exits within ~11s (10s drain timeout + 1s tolerance).
/// 5. Assert exit code 0 (DaemonExit::Graceful — drain timeout is not an error).
/// 6. Assert monocle.lock and hooks-settings.json are absent (cleanup ran).
///
/// # Feature gate
///
/// This test requires the `e2e-test-affordances` cargo feature so the daemon binary is
/// compiled with `MONOCLE_HOOK_DELAY_MS` support. Run via:
///   `cargo test -p monocle-runtime --test daemon_e2e_serve --features e2e-test-affordances`
/// HIGH-1 fix (SS-daemon-wiring-impl.md §Fix Addendum Round 2 HIGH-1).
#[test]
#[cfg(feature = "e2e-test-affordances")]
fn test_daemon_e2e_drain_timeout_AC_E2E_007() {
    let binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_monocle-runtime"));
    assert!(binary.exists(), "monocle-runtime binary not found");

    let tmp = tempfile::tempdir().expect("create tmpdir for drain timeout test");
    let runtime_dir = tmp.path().to_path_buf();

    // MONOCLE_HOOK_DELAY_MS=15000 sets hook_outer_delay_ms on DaemonState.
    // The pre_tool_use outer handler sleeps 15s before entering the 300ms inner budget,
    // creating a genuinely in-flight request that holds axum's graceful shutdown open.
    let mut child = std::process::Command::new(&binary)
        .env("MONOCLE_RUNTIME_DIR", &runtime_dir)
        .env("RUST_LOG", "monocle_runtime=info")
        .env("MONOCLE_HOOK_DELAY_MS", "15000")
        .spawn()
        .expect("spawn monocle-runtime binary for drain timeout test");

    let daemon_pid = child.id();

    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        wait_for_file(&lock_path, Duration::from_secs(5)),
        "AC-E2E-007: monocle.lock did not appear within 5s"
    );

    let lock_content = std::fs::read_to_string(&lock_path).expect("AC-E2E-007: read monocle.lock");
    let lock_json: serde_json::Value =
        serde_json::from_str(&lock_content).expect("AC-E2E-007: monocle.lock must be valid JSON");

    let port = lock_json
        .get("port")
        .and_then(|v| v.as_u64())
        .and_then(|p| u16::try_from(p).ok())
        .expect("AC-E2E-007: monocle.lock must have a u16 'port' field");

    let wire_token = lock_json
        .get("token")
        .and_then(|v| v.as_str())
        .expect("AC-E2E-007: lock file must have 'token'");
    let hex_token = wire_token
        .strip_prefix("monocle-v1:")
        .expect("token must start with monocle-v1: prefix")
        .to_owned();

    // Wait for HTTP server readiness.
    assert!(
        wait_for_healthz(port, Duration::from_secs(5)),
        "AC-E2E-007: daemon did not become healthy within 5s"
    );

    // Issue a POST /hooks/pre-tool-use with valid auth in a background thread.
    // The outer handler sleeps 15s (hook_outer_delay_ms=15000) before any processing,
    // creating a genuinely in-flight request that holds axum's graceful shutdown open.
    let hook_url = format!("http://127.0.0.1:{port}/hooks/pre-tool-use");
    let slow_request_thread = std::thread::spawn(move || {
        let hook_body = serde_json::json!({
            "hook_event_name": "PreToolUse",
            "session_id": "drain-timeout-test-session",
            "tool_name": "Read",
            "tool_input": {}
        });
        // Use a 30s read timeout — longer than the 10s drain window + 15s delay.
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(30))
            .timeout_connect(Duration::from_secs(5))
            .build();
        let _ = agent
            .post(&hook_url)
            .set("Content-Type", "application/json")
            .set(
                "X-Monocle-Authorization",
                &format!("monocle-v1:{hex_token}"),
            )
            .send_json(hook_body);
        // The connection will be force-closed by the drain timeout. That's expected.
    });

    // Give the outer delay a moment to start sleeping inside the handler.
    std::thread::sleep(Duration::from_millis(300));

    // Record time before SIGTERM.
    let sigterm_time = std::time::Instant::now();

    // Send SIGTERM while the slow handler is in-flight.
    unsafe {
        libc::kill(daemon_pid as libc::pid_t, libc::SIGTERM);
    }

    // Assert the daemon exits within ~11s (10s drain timeout + 1s tolerance).
    // Without the tokio::time::timeout wrapper in main(), the daemon would hang for
    // 15s (the full hook_outer_delay_ms). With it, it exits within ~10s.
    let max_exit_wait = Duration::from_secs(11);
    let deadline = sigterm_time + max_exit_wait;
    let exit_status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    child.kill().ok();
                    let elapsed = sigterm_time.elapsed();
                    panic!(
                        "AC-E2E-007: daemon did not exit within 11s after SIGTERM with \
                         in-flight slow handler (hook_outer_delay_ms=15000; elapsed: {elapsed:.1?}). \
                         This indicates the tokio::time::timeout drain wrapper in main() \
                         did not fire (BC-2.01.004 INV-1)."
                    );
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => panic!("AC-E2E-007: wait() failed: {e}"),
        }
    };

    let elapsed = sigterm_time.elapsed();
    assert!(
        exit_status.success(),
        "AC-E2E-007: daemon must exit 0 after drain timeout (force-close is not an error; \
         got: {exit_status:?}, elapsed: {elapsed:.1?})"
    );

    let hs_path = runtime_dir.join("hooks-settings.json");
    assert!(
        !hs_path.exists(),
        "AC-E2E-007: hooks-settings.json must be removed after drain timeout (elapsed: {elapsed:.1?})"
    );
    assert!(
        !lock_path.exists(),
        "AC-E2E-007: monocle.lock must be removed after drain timeout (elapsed: {elapsed:.1?})"
    );

    // Cleanup the background thread.
    let _ = slow_request_thread.join();
}

/// C1 — drop_counter stays 0 under normal load with a live event-bus consumer.
///
/// Issues 10 hook POSTs to the running daemon with stderr captured. Verifies that:
/// 1. All requests succeed (HTTP 200).
/// 2. Daemon stderr contains NO "event bus closed" WARN (which fires when event_rx is
///    dropped and try_send hits TrySendError::Closed). Absence of this message proves
///    the fan-out task holds event_rx so the channel is never Closed (C1 fix —
///    SS-daemon-wiring-impl.md §Fix Addendum C1).
/// 3. The ring file has exactly 10 PreToolUse records (all try_publish_event calls
///    succeeded with no silent discard due to a closed channel).
#[test]
fn test_daemon_e2e_drop_counter_zero_under_normal_load_C1() {
    let binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_monocle-runtime"));
    assert!(binary.exists(), "monocle-runtime binary not found");

    let tmp = tempfile::tempdir().expect("create tmpdir for drop counter test");
    let runtime_dir = tmp.path().to_path_buf();

    // Capture stderr so we can assert the absence of "event bus closed" WARN.
    let mut child = std::process::Command::new(&binary)
        .env("MONOCLE_RUNTIME_DIR", &runtime_dir)
        // Use warn level: "event bus closed" WARN fires at warn; info won't show it.
        .env("RUST_LOG", "monocle_runtime=warn")
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn monocle-runtime binary for drop counter test");

    let daemon_pid = child.id();

    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        wait_for_file(&lock_path, Duration::from_secs(5)),
        "C1 test: monocle.lock did not appear within 5s"
    );

    let lock_content = std::fs::read_to_string(&lock_path).expect("C1 test: read monocle.lock");
    let lock_json: serde_json::Value =
        serde_json::from_str(&lock_content).expect("C1 test: monocle.lock must be valid JSON");

    let port = lock_json
        .get("port")
        .and_then(|v| v.as_u64())
        .and_then(|p| u16::try_from(p).ok())
        .expect("C1 test: monocle.lock must have a u16 'port' field");

    let wire_token = lock_json
        .get("token")
        .and_then(|v| v.as_str())
        .expect("C1 test: lock file must have 'token'");
    let hex_token = wire_token
        .strip_prefix("monocle-v1:")
        .expect("token must start with monocle-v1: prefix");

    assert!(
        wait_for_healthz(port, Duration::from_secs(5)),
        "C1 test: daemon did not become healthy within 5s"
    );

    let hook_url = format!("http://127.0.0.1:{port}/hooks/pre-tool-use");
    // Issue 10 hook events.
    for i in 0..10u32 {
        let hook_body = serde_json::json!({
            "hook_event_name": "PreToolUse",
            "session_id": format!("c1-test-session-{i}"),
            "tool_name": "Read",
            "tool_input": {}
        });
        let resp = ureq::post(&hook_url)
            .set("Content-Type", "application/json")
            .set(
                "X-Monocle-Authorization",
                &format!("monocle-v1:{hex_token}"),
            )
            .send_json(hook_body)
            .unwrap_or_else(|e| panic!("C1 test: hook POST {i} failed at transport layer: {e}"));
        assert_eq!(resp.status(), 200, "C1 test: hook POST {i} must return 200");
    }

    // Assert ring file has exactly 10 PreToolUse records — all try_publish_event calls
    // succeeded and ring.append() was reached for each (ring path is independent of event bus).
    // Poll up to 3s for all 10 records to land (async ring flush task has a queue).
    let ring_path = runtime_dir.join("monocle-events.jsonl");
    let pre_tool_use_count = {
        let start = std::time::Instant::now();
        let max_wait = Duration::from_secs(3);
        loop {
            let count = if ring_path.exists() {
                std::fs::read_to_string(&ring_path)
                    .unwrap_or_default()
                    .lines()
                    .filter(|line| {
                        serde_json::from_str::<serde_json::Value>(line)
                            .ok()
                            .and_then(|v| {
                                v.get("hook_type")
                                    .and_then(|t| t.as_str())
                                    .map(|s| s == "PreToolUse")
                            })
                            .unwrap_or(false)
                    })
                    .count()
            } else {
                0
            };
            if count >= 10 {
                break count;
            }
            if start.elapsed() >= max_wait {
                break count;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    };
    assert_eq!(
        pre_tool_use_count, 10,
        "C1 test: ring file must have exactly 10 PreToolUse records after 10 hook POSTs \
         (polled up to 3s for ring flush task to write all records)"
    );

    // Graceful shutdown.
    unsafe {
        libc::kill(daemon_pid as libc::pid_t, libc::SIGTERM);
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    child.kill().ok();
                    panic!("C1 test: daemon did not exit within 5s after SIGTERM");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => panic!("C1 test: wait() failed: {e}"),
        }
    }

    // After shutdown, read captured stderr and assert no "event bus closed" WARN.
    // This message fires when event_rx is dropped and try_send gets TrySendError::Closed.
    // Its absence proves the fan-out task held event_rx throughout the test run (C1 fix).
    let stderr_output = {
        let mut stderr = child
            .stderr
            .take()
            .expect("C1 test: stderr was piped; take() should succeed");
        let mut buf = String::new();
        use std::io::Read as _;
        stderr
            .read_to_string(&mut buf)
            .expect("C1 test: read daemon stderr");
        buf
    };
    assert!(
        !stderr_output.contains("event bus closed"),
        "C1 test: daemon stderr must NOT contain 'event bus closed' WARN — this indicates \
         event_rx was dropped (the channel became Closed) which means the fan-out task was \
         not spawned. C1 fix ensures the fan-out task holds event_rx. Stderr:\n{stderr_output}"
    );
}

/// AC-E2E-008 — Ring durability: flush-delay drain proves the drain-join is non-vacuous.
///
/// # What this test verifies (non-vacuity argument)
///
/// The ring flush task (`flush_task_loop` in ring.rs) writes each record to disk IMMEDIATELY
/// on receipt when running without a flush delay — so in the absence of `MONOCLE_RING_FLUSH_DELAY_MS`,
/// any record enqueued before SIGTERM is already on disk by the time the drain-join runs.
/// That makes the drain-join vacuous: it passes whether or not the drain-join exists.
///
/// This test activates `MONOCLE_RING_FLUSH_DELAY_MS=500` to inject a 500ms artificial delay
/// BEFORE each disk write in the flush task. The timeline is then:
///
/// ```text
/// t=0ms   POST /hooks/pre-tool-use → handler returns 200 → record enqueued to write-queue
/// t=~5ms  SIGTERM sent (immediately after confirming POST was received by the handler)
/// t=~5ms  Shutdown sequence begins: HTTP drain completes (no in-flight requests), then
///         hooks-settings.json removed, then ring Arc dropped (closing write-queue channel)
/// t=~5ms  flush_task_loop: currently sleeping 500ms for the enqueued record's pre-write delay
/// t=~505ms flush_task_loop: write + flush to disk; rx.recv() returns None → task exits
/// t=~505ms drain-join (2s timeout) sees task exited → cleanup continues
/// t=~510ms daemon exits 0
/// ```
///
/// Without the drain-join in main.rs step (d):
/// - `process::exit` would fire at ~5ms (after HTTP drain)
/// - The flush task is still sleeping; the 500ms write never executes
/// - Record is LOST → test FAILS with "monocle-events.jsonl does not contain PreToolUse"
///
/// With the drain-join (2s timeout > 500ms delay):
/// - Shutdown waits for the flush task to complete the 500ms sleep and execute the write
/// - Record is DURABLE → test PASSES
///
/// # Feature gate
///
/// Requires `e2e-test-affordances` (same as AC-E2E-007). The daemon binary compiled with
/// `--features e2e-test-affordances` supports `MONOCLE_RING_FLUSH_DELAY_MS`. Run via:
///   `cargo test -p monocle-runtime --test daemon_e2e_serve --features e2e-test-affordances`
///
/// # Timing parameters
///
/// - `MONOCLE_RING_FLUSH_DELAY_MS=500`: flush delay. Must be > typical daemon shutdown time
///   (~5ms) so the record is provably NOT on disk at SIGTERM, and < the 2s drain-join timeout
///   so the drain completes deterministically.
/// - SIGTERM sent: immediately after confirming POST was received (≤10ms wait for 200 response).
///   No 100ms pre-SIGTERM sleep — unlike the vacuous version, we don't need to let the flush
///   complete before SIGTERM; the delay guarantees it hasn't.
/// - Exit budget: 15s (10s HTTP drain + 2s ring-flush drain + 3s buffer).
#[test]
#[cfg(feature = "e2e-test-affordances")]
fn test_daemon_e2e_ring_durability_immediate_sigterm_AC_E2E_008() {
    // Flush delay: 500ms. Drain-join timeout in main.rs: 2000ms.
    // Invariant: FLUSH_DELAY_MS < DRAIN_JOIN_TIMEOUT_MS so the drain always completes.
    // Invariant: FLUSH_DELAY_MS >> typical daemon-shutdown-path duration (~5ms on loopback)
    //            so the record is provably NOT flushed at the moment SIGTERM fires.
    const FLUSH_DELAY_MS: u64 = 500;

    let binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_monocle-runtime"));
    assert!(
        binary.exists(),
        "AC-E2E-008: monocle-runtime binary not found at {binary:?}"
    );

    let tmp = tempfile::tempdir().expect("AC-E2E-008: create tmpdir");
    let runtime_dir = tmp.path().to_path_buf();

    // Spawn the daemon WITH the flush delay affordance.
    // MONOCLE_RING_FLUSH_DELAY_MS=500 causes flush_task_loop to sleep 500ms BEFORE
    // writing each record to disk. Combined with an immediate SIGTERM after the POST,
    // the record will be ENQUEUED but NOT YET FLUSHED when shutdown begins.
    let mut child = std::process::Command::new(&binary)
        .env("MONOCLE_RUNTIME_DIR", &runtime_dir)
        .env("RUST_LOG", "monocle_runtime=info")
        .env("MONOCLE_RING_FLUSH_DELAY_MS", FLUSH_DELAY_MS.to_string())
        .spawn()
        .expect("AC-E2E-008: spawn monocle-runtime binary with MONOCLE_RING_FLUSH_DELAY_MS");

    let daemon_pid = child.id();

    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        wait_for_file(&lock_path, Duration::from_secs(5)),
        "AC-E2E-008: monocle.lock did not appear within 5s"
    );

    let lock_content = std::fs::read_to_string(&lock_path).expect("AC-E2E-008: read monocle.lock");
    let lock_json: serde_json::Value =
        serde_json::from_str(&lock_content).expect("AC-E2E-008: monocle.lock must be valid JSON");

    let port = lock_json
        .get("port")
        .and_then(|v| v.as_u64())
        .and_then(|p| u16::try_from(p).ok())
        .expect("AC-E2E-008: monocle.lock must have a u16 'port' field");

    let wire_token = lock_json
        .get("token")
        .and_then(|v| v.as_str())
        .expect("AC-E2E-008: lock file must have 'token'");
    let hex_token = wire_token
        .strip_prefix("monocle-v1:")
        .expect("AC-E2E-008: token must start with monocle-v1: prefix")
        .to_owned();

    // Wait for HTTP server readiness.
    assert!(
        wait_for_healthz(port, Duration::from_secs(5)),
        "AC-E2E-008: daemon did not become healthy within 5s"
    );

    // POST the hook and wait for the HTTP 200 response to confirm the record was ENQUEUED
    // to the write-queue. We use a synchronous (blocking) POST here — once we get 200,
    // we know the handler ran and called ring.append(), which enqueued the record to the
    // async write-queue. The flush task has then started sleeping for 500ms BEFORE the write.
    //
    // Non-vacuity: at the moment SIGTERM fires (immediately below), the record is:
    //   - ENQUEUED in the write-queue (ring.append() returned Ok)
    //   - NOT YET ON DISK (flush task is sleeping for FLUSH_DELAY_MS ms)
    // The shutdown drain-join (2s timeout) is the ONLY mechanism that waits for the flush
    // task to finish sleeping and execute the disk write. Without it, the record is lost.
    let hook_url = format!("http://127.0.0.1:{port}/hooks/pre-tool-use");
    let hook_body = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "session_id": "e2e-durability-flush-delay-session",
        "tool_name": "Bash",
        "tool_input": {"command": "echo durability-flush-delay-test"}
    });
    let hook_resp = ureq::post(&hook_url)
        .set("Content-Type", "application/json")
        .set(
            "X-Monocle-Authorization",
            &format!("monocle-v1:{hex_token}"),
        )
        .send_json(hook_body)
        .expect("AC-E2E-008: POST /hooks/pre-tool-use must succeed");
    assert_eq!(
        hook_resp.status(),
        200,
        "AC-E2E-008: POST /hooks/pre-tool-use must return 200 (record must be enqueued)"
    );

    // SIGTERM fires IMMEDIATELY after the 200 response.
    // At this point: record is ENQUEUED but NOT YET WRITTEN (flush task is sleeping 500ms).
    // The drain-join in main.rs step (d) must wait for the 500ms sleep to complete and the
    // write to execute before process::exit fires.
    unsafe {
        libc::kill(daemon_pid as libc::pid_t, libc::SIGTERM);
    }

    // Wait for daemon exit.
    // Budget: 10s HTTP drain (no in-flight requests so this is fast) + 500ms flush delay
    // + 2s drain-join timeout + 2s buffer = ~15s total.
    let sigterm_time = std::time::Instant::now();
    let deadline = sigterm_time + Duration::from_secs(15);
    let exit_status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    child.kill().ok();
                    let elapsed = sigterm_time.elapsed();
                    panic!(
                        "AC-E2E-008: daemon did not exit within 15s after SIGTERM \
                         (MONOCLE_RING_FLUSH_DELAY_MS={FLUSH_DELAY_MS}; elapsed: {elapsed:.1?}). \
                         The drain-join in main.rs step (d) may not be awaiting the flush task \
                         (BC-2.01.007 / BC-2.04.012 CRITICAL-1)."
                    );
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => panic!("AC-E2E-008: wait() failed: {e}"),
        }
    };

    assert!(
        exit_status.success(),
        "AC-E2E-008: daemon must exit 0 on graceful shutdown (got: {exit_status:?})"
    );

    // The ring file MUST exist and contain the PreToolUse record.
    //
    // Non-vacuity assertion: if the drain-join were absent (or the drain-join timeout were
    // < FLUSH_DELAY_MS), process::exit would fire before the flush task writes the record.
    // The file would either not exist or not contain the PreToolUse entry. This is the
    // discriminating assertion that makes this test non-vacuous: it can only pass if the
    // shutdown drain-join successfully waited for the delayed flush to complete.
    let ring_path = runtime_dir.join("monocle-events.jsonl");
    assert!(
        ring_path.exists(),
        "AC-E2E-008: monocle-events.jsonl must exist after daemon exit \
         (record lost — drain-join did not wait for the {FLUSH_DELAY_MS}ms-delayed flush; \
         BC-2.01.007 / BC-2.04.012 CRITICAL-1 fix violated)"
    );

    let ring_content =
        std::fs::read_to_string(&ring_path).expect("AC-E2E-008: read monocle-events.jsonl");
    let has_pre_tool_use = ring_content.lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|rec| {
                rec.get("hook_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "PreToolUse")
            })
            .unwrap_or(false)
    });
    assert!(
        has_pre_tool_use,
        "AC-E2E-008: monocle-events.jsonl must contain a PreToolUse record after \
         immediate-SIGTERM shutdown with MONOCLE_RING_FLUSH_DELAY_MS={FLUSH_DELAY_MS}. \
         The record was ENQUEUED (HTTP 200 returned) but the flush task sleeps {FLUSH_DELAY_MS}ms \
         before writing. The drain-join (2s timeout) in main.rs step (d) must have waited for \
         the flush to complete. If the record is absent, the drain-join did not wait \
         (BC-2.01.007 / BC-2.04.012 CRITICAL-1 fix). Ring content:\n{ring_content}"
    );
}
