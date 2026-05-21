//! Shared test utilities for monocle-test-harness integration tests.
//!
//! Provides helpers for:
//! - Creating lock file fixtures (tempfile-based)
//! - Loading fixture JSON from the 25-fixture corpus
//! - Constructing axum test requests
//!
//! Source authority: S-DTU-001 v1.2, BC-HOOK-001..BC-HOOK-041
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;

// ──────────────────────────────────────────────────────────────────────────────
// Lock file fixture helpers
// ──────────────────────────────────────────────────────────────────────────────

/// A valid 64-hex auth token used across tests.
pub const VALID_AUTH_TOKEN: &str =
    "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";

/// Write a monocle lock file to `dir/<port>.lock` with the given JSON content.
/// Returns the path to the written file.
///
/// Uses `tempfile::persist` for atomic write per SS-conventions-anti-patterns.md
/// (`std::fs::write` is disallowed; use `tempfile::persist` instead).
pub fn write_lock_file(dir: &Path, port: u16, content: &Value) -> PathBuf {
    use std::io::Write as _;
    let target_path = dir.join(format!("{port}.lock"));
    let json = serde_json::to_string_pretty(content)
        .unwrap_or_else(|e| panic!("serialize lock JSON: {e}"));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .unwrap_or_else(|e| panic!("NamedTempFile in lock dir: {e}"));
    tmp.write_all(json.as_bytes())
        .unwrap_or_else(|e| panic!("write lock JSON to tempfile: {e}"));
    tmp.persist(&target_path)
        .unwrap_or_else(|e| panic!("persist lock file atomically: {e}"));
    target_path
}

/// Build a canonical monocle lock file JSON object.
///
/// Schema per SS-daemon-lifecycle.md v1.0.33 §Start Sequence lines 491-512.
pub fn monocle_lock_json(port: u16, pid: u32, auth_token: &str) -> Value {
    serde_json::json!({
        "contract_version": 1,
        "pid": pid,
        "port": port,
        "authToken": auth_token,
        "startTimeUtc": "2026-05-20T10:00:00.000Z",
        "app": "monocle",
        "version": "0.1.0"
    })
}

/// Build a lock file JSON for a non-monocle application (e.g., VS Code).
pub fn non_monocle_lock_json(port: u16, pid: u32) -> Value {
    serde_json::json!({
        "contract_version": 1,
        "pid": pid,
        "port": port,
        "authToken": VALID_AUTH_TOKEN,
        "startTimeUtc": "2026-05-20T10:00:00.000Z",
        "app": "vscode",
        "version": "1.90.0"
    })
}

/// Build a lock file JSON with a wrong contract_version (triggers ContractVersionMismatch).
pub fn wrong_version_lock_json(port: u16, pid: u32) -> Value {
    serde_json::json!({
        "contract_version": 99,
        "pid": pid,
        "port": port,
        "authToken": VALID_AUTH_TOKEN,
        "app": "monocle",
        "version": "0.1.0"
    })
}

/// Create a TempDir containing a single valid monocle lock file.
/// Returns (TempDir, lock_path).
pub fn temp_lock_dir_with_valid_lock(port: u16, pid: u32) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("TempDir");
    let content = monocle_lock_json(port, pid, VALID_AUTH_TOKEN);
    let path = write_lock_file(dir.path(), port, &content);
    (dir, path)
}

/// Create a TempDir with no lock files (empty run dir).
pub fn temp_empty_lock_dir() -> TempDir {
    TempDir::new().expect("TempDir")
}

// ──────────────────────────────────────────────────────────────────────────────
// Fixture corpus helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Root of the 25-fixture corpus, relative to workspace root.
///
/// In tests, we use `env!("CARGO_MANIFEST_DIR")` to find the workspace root.
pub fn fixture_corpus_root() -> PathBuf {
    // Cargo sets CARGO_MANIFEST_DIR to the crate's manifest dir.
    // The fixtures live at <workspace_root>/tests/fixtures/dtu/claude-code-hook-2x/.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crate manifest is at crates/monocle-test-harness/Cargo.toml; workspace root is 2 up.
    let workspace_root = manifest_dir
        .parent()
        .expect("parent of crates/monocle-test-harness")
        .parent()
        .expect("workspace root");
    workspace_root.join("tests/fixtures/dtu/claude-code-hook-2x")
}

/// Load a fixture JSON file from the corpus.
///
/// `endpoint` is one of: "pre-tool-use", "notification", "stop",
/// "session-start", "prompt-submit".
/// `name` is the fixture filename without `.json` extension.
pub fn load_fixture(endpoint: &str, name: &str) -> Value {
    let path = fixture_corpus_root()
        .join(endpoint)
        .join(format!("{name}.json"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {endpoint}/{name}.json: {e}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("parse fixture {endpoint}/{name}.json: {e}"))
}

/// List all fixture files for an endpoint. Returns filenames without extension.
pub fn list_fixtures(endpoint: &str) -> Vec<String> {
    let dir = fixture_corpus_root().join(endpoint);
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read fixture dir {endpoint}: {e}"))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            name.strip_suffix(".json").map(|s| s.to_string())
        })
        .collect();
    names.sort();
    names
}

// ──────────────────────────────────────────────────────────────────────────────
// Axum test request helpers
// ──────────────────────────────────────────────────────────────────────────────

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use tower::ServiceExt;

/// Send a POST request to `path` on `router` with `body` as JSON.
/// Returns the response status code.
pub async fn post_json(router: Router, path: &str, body: &Value) -> StatusCode {
    let req = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(body).expect("serialize body"),
        ))
        .expect("build request");
    let response = router.oneshot(req).await.expect("oneshot");
    response.status()
}

/// Send a GET request to `path` on `router`.
/// Returns the response status code.
pub async fn get(router: Router, path: &str) -> StatusCode {
    let req = Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .expect("build request");
    let response = router.oneshot(req).await.expect("oneshot");
    response.status()
}

/// Send a DELETE request to `path` on `router`.
pub async fn delete(router: Router, path: &str) -> StatusCode {
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(path)
        .body(Body::empty())
        .expect("build request");
    let response = router.oneshot(req).await.expect("oneshot");
    response.status()
}

/// Send a PUT request to `path` on `router`.
pub async fn put_json(router: Router, path: &str, body: &Value) -> StatusCode {
    let req = Request::builder()
        .method(Method::PUT)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(body).expect("serialize body"),
        ))
        .expect("build request");
    let response = router.oneshot(req).await.expect("oneshot");
    response.status()
}

/// Build a `CloneState` using a fake daemon address (no real server needed for
/// router-level tests — handlers will todo!() before any network call).
///
/// The port is a non-listening loopback port; tests exercise the *routing*
/// layer and expect todo!() panics from the handler stubs (Red Gate).
pub fn make_test_clone_state() -> monocle_test_harness::dtu::server::CloneState {
    use monocle_test_harness::dtu::{lock_reader::LockFileInfo, server::CloneState};
    CloneState {
        client: reqwest::Client::new(),
        daemon: LockFileInfo {
            port: 19999,
            auth_token: VALID_AUTH_TOKEN.to_string(),
            pid: std::process::id(),
        },
        endpoint_base: "http://127.0.0.1:19999".to_string(),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Mock daemon helpers for fidelity testing
// ──────────────────────────────────────────────────────────────────────────────

/// Result from starting a mock daemon.
///
/// The mock daemon captures the first POST body it receives and sends it back
/// via the channel. Drop `MockDaemon` to shut down the server.
pub struct MockDaemon {
    /// Port the mock daemon is listening on.
    pub port: u16,
    /// Receives the raw body bytes of the first POST the daemon captures.
    /// If the clone drops the request (e.g. filter), `None` is received after timeout.
    pub receiver: tokio::sync::mpsc::Receiver<Vec<u8>>,
    /// Shutdown signal — send anything to stop the server.
    pub shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

/// Start a mock daemon that captures a single POST body per call.
///
/// The mock daemon starts an axum HTTP server on 127.0.0.1:0 (random port).
/// Every POST body it receives is forwarded through the returned channel.
/// This allows fidelity tests to intercept what the clone actually sends to the daemon.
///
/// AC-004, BC-HOOK-007: used by integration_fidelity.rs tests to verify that
/// the clone forwards the correct payload structure.
pub async fn start_mock_daemon() -> MockDaemon {
    use axum::{body::Body, http::Request};
    use tokio::net::TcpListener;

    // Bind on port 0 → OS assigns a random free port.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|e| panic!("bind mock daemon: {e}"));
    let port = listener
        .local_addr()
        .unwrap_or_else(|e| panic!("local addr: {e}"))
        .port();

    // Channel to deliver captured request bodies to the test.
    // We use capacity 25 so all 25 fixtures can be processed by the aggregate test.
    let (body_tx, body_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(25);
    // Shutdown channel.
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Build a tiny axum router that captures all POST bodies.
    let router = {
        let tx = body_tx.clone();
        axum::Router::new().fallback(axum::routing::any(move |req: Request<Body>| {
            let tx = tx.clone();
            async move {
                let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
                    .await
                    .unwrap_or_default();
                // Send captured body; ignore send error (receiver may be gone).
                let _ = tx.try_send(body_bytes.to_vec());
                axum::http::StatusCode::OK
            }
        }))
    };

    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap_or(());
    });

    MockDaemon {
        port,
        receiver: body_rx,
        shutdown_tx,
    }
}

/// Build a `CloneState` pointing to a running mock daemon.
///
/// Used by fidelity tests to intercept what the clone actually forwards.
pub fn make_clone_state_for_mock(
    daemon_port: u16,
) -> monocle_test_harness::dtu::server::CloneState {
    use monocle_test_harness::dtu::{lock_reader::LockFileInfo, server::CloneState};
    CloneState {
        client: reqwest::Client::new(),
        daemon: LockFileInfo {
            port: daemon_port,
            auth_token: VALID_AUTH_TOKEN.to_string(),
            pid: std::process::id(),
        },
        endpoint_base: format!("http://127.0.0.1:{daemon_port}"),
    }
}
