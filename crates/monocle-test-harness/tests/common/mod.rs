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
pub fn write_lock_file(dir: &Path, port: u16, content: &Value) -> PathBuf {
    let path = dir.join(format!("{port}.lock"));
    let json = serde_json::to_string_pretty(content).expect("serialize lock JSON");
    std::fs::write(&path, json).expect("write lock file");
    path
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
