//! `dtu-fidelity` xtask subcommand.
//!
//! Runs the DTU fidelity oracle against the 25-fixture corpus at
//! `tests/fixtures/dtu/claude-code-hook-2x/`. Drives the monocle-test-harness
//! DTU clone library in-process (no real binary dependency) to avoid the
//! `todo!()` in the unimplemented `dtu-claude-code-hooks-v1` binary.
//!
//! Exit codes:
//!   0 — mean fidelity score ≥ 0.95 across all 25 fixtures
//!   1 — mean fidelity score < 0.95 or one or more per-fixture failures
//!
//! Source authority:
//! - S-DTU-001 AC-007 (`cargo xtask dtu-fidelity` oracle)
//! - dtu-assessment.md v1.7.5 §Tooling, §DTU Fidelity Measurement Procedure
//! - BC-HOOK-007 (scoring function)
//! - BC-HOOK-034 (notification filter — non-permission_prompt must NOT be forwarded)

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use axum::{body::Body, http::Request};
use clap::Args;
use monocle_test_harness::dtu::{
    endpoints::{build_router, paths},
    lock_reader::LockFileInfo,
    payload::FixtureScore,
    server::CloneState,
};
use serde_json::Value;
use tower::ServiceExt as _;
use tracing::{error, info, warn};

/// Arguments for the `dtu-fidelity` subcommand.
#[derive(Args, Debug)]
pub struct DtuFidelityArgs {
    /// Emit per-fixture scores as JSON to stdout instead of the default table.
    #[arg(long)]
    pub json: bool,

    /// Override the fixture corpus path (default: tests/fixtures/dtu/claude-code-hook-2x/).
    #[arg(long)]
    pub corpus: Option<PathBuf>,

    /// Fidelity threshold (default: 0.95). Exits 1 if mean is below this value.
    #[arg(long, default_value = "0.95")]
    pub threshold: f64,
}

/// Specification for a single fixture to score.
struct FixtureSpec {
    /// Endpoint name (directory under the corpus root).
    endpoint: &'static str,
    /// Clone router path constant.
    path: &'static str,
    /// Fixture filename without `.json` extension.
    name: &'static str,
    /// If true, the mock daemon must receive a POST for this fixture.
    /// If false, the clone MUST NOT forward (BC-HOOK-034 filter case).
    expect_forward: bool,
}

/// The canonical 25-fixture corpus specification.
///
/// Order matches dtu-assessment.md §Fixture Corpus (5 per endpoint × 5 endpoints).
/// Source authority: S-DTU-001 AC-004.
static FIXTURE_CORPUS: &[FixtureSpec] = &[
    // pre-tool-use (5 fixtures)
    FixtureSpec {
        endpoint: "pre-tool-use",
        path: paths::PRE_TOOL_USE,
        name: "bash-simple",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "pre-tool-use",
        path: paths::PRE_TOOL_USE,
        name: "bash-multiline",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "pre-tool-use",
        path: paths::PRE_TOOL_USE,
        name: "read-file",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "pre-tool-use",
        path: paths::PRE_TOOL_USE,
        name: "write-file",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "pre-tool-use",
        path: paths::PRE_TOOL_USE,
        name: "unknown-tool",
        expect_forward: true,
    },
    // notification (5 fixtures)
    FixtureSpec {
        endpoint: "notification",
        path: paths::NOTIFICATION,
        name: "permission-prompt-bash",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "notification",
        path: paths::NOTIFICATION,
        name: "permission-prompt-write",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "notification",
        path: paths::NOTIFICATION,
        name: "permission-prompt-web-fetch",
        expect_forward: true,
    },
    // BC-HOOK-034: non-permission_prompt notifications MUST NOT be forwarded.
    // Scored as 1.0 iff the daemon receives nothing; 0.0 otherwise.
    FixtureSpec {
        endpoint: "notification",
        path: paths::NOTIFICATION,
        name: "non-permission-dropped",
        expect_forward: false,
    },
    FixtureSpec {
        endpoint: "notification",
        path: paths::NOTIFICATION,
        name: "large-message-boundary",
        expect_forward: true,
    },
    // stop (5 fixtures)
    FixtureSpec {
        endpoint: "stop",
        path: paths::STOP,
        name: "stop-reason-normal",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "stop",
        path: paths::STOP,
        name: "stop-reason-error",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "stop",
        path: paths::STOP,
        name: "stop-reason-interrupt",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "stop",
        path: paths::STOP,
        name: "stop-reason-empty",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "stop",
        path: paths::STOP,
        name: "stop-reason-unknown",
        expect_forward: true,
    },
    // session-start (5 fixtures)
    FixtureSpec {
        endpoint: "session-start",
        path: paths::SESSION_START,
        name: "session-start-basic",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "session-start",
        path: paths::SESSION_START,
        name: "session-start-uuid-v4",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "session-start",
        path: paths::SESSION_START,
        name: "session-start-extra-fields",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "session-start",
        path: paths::SESSION_START,
        name: "session-start-missing-session-id",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "session-start",
        path: paths::SESSION_START,
        name: "session-start-null-pid",
        expect_forward: true,
    },
    // prompt-submit (5 fixtures)
    FixtureSpec {
        endpoint: "prompt-submit",
        path: paths::PROMPT_SUBMIT,
        name: "prompt-submit-basic",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "prompt-submit",
        path: paths::PROMPT_SUBMIT,
        name: "prompt-submit-uuid-v4",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "prompt-submit",
        path: paths::PROMPT_SUBMIT,
        name: "prompt-submit-extra-fields",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "prompt-submit",
        path: paths::PROMPT_SUBMIT,
        name: "prompt-submit-missing-session-id",
        expect_forward: true,
    },
    FixtureSpec {
        endpoint: "prompt-submit",
        path: paths::PROMPT_SUBMIT,
        name: "prompt-submit-long-session-id",
        expect_forward: true,
    },
];

/// Resolved per-fixture result for reporting and aggregate scoring.
#[derive(Debug, serde::Serialize)]
struct FixtureResult {
    fixture: String,
    score: f64,
    mismatched_fields: Vec<String>,
    /// True if the fixture was scored (forwarded or correctly filtered).
    /// False if the daemon received nothing when it was expected to receive a POST.
    received: bool,
}

/// Locate the workspace root from the xtask binary's location.
///
/// The xtask binary is run with `cargo run -p xtask`, so `CARGO_MANIFEST_DIR`
/// at runtime points to the xtask crate. We walk up to find the workspace root
/// (the directory containing `[workspace]` in its Cargo.toml).
fn workspace_root() -> Result<PathBuf> {
    // In a cargo run, the current dir is typically the workspace root.
    // Fall back to walking up from the binary location.
    let cwd = std::env::current_dir().context("get current directory")?;

    // Check cwd first (most common case with `cargo run -p xtask` from root).
    if is_workspace_root(&cwd) {
        return Ok(cwd);
    }

    // Walk up from CARGO_MANIFEST_DIR (xtask/).
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut dir = manifest_dir.as_path();
    loop {
        if is_workspace_root(dir) {
            return Ok(dir.to_path_buf());
        }
        dir = dir
            .parent()
            .context("reached filesystem root without finding workspace Cargo.toml")?;
    }
}

/// Returns true if `dir` contains a `Cargo.toml` with `[workspace]`.
fn is_workspace_root(dir: &Path) -> bool {
    let candidate = dir.join("Cargo.toml");
    std::fs::read_to_string(&candidate)
        .map(|s| s.contains("[workspace]"))
        .unwrap_or(false)
}

/// Load a fixture JSON file from the corpus.
fn load_fixture(corpus_root: &Path, endpoint: &str, name: &str) -> Result<Value> {
    let path = corpus_root.join(endpoint).join(format!("{name}.json"));
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("read fixture {endpoint}/{name}.json at {path:?}"))?;
    serde_json::from_str(&content).with_context(|| format!("parse fixture {endpoint}/{name}.json"))
}

/// Drive the clone router with a fixture POST and capture what the mock daemon receives.
///
/// Returns `Some(received_json)` if the daemon received a POST within 500ms.
/// Returns `None` if the daemon received nothing (filtered or fire-and-forget failed).
///
/// Mirrors the `drive_and_capture` function in `integration_fidelity.rs` so that
/// the xtask oracle produces results comparable to the test suite.
async fn drive_and_capture(endpoint_path: &'static str, fixture: &Value) -> Option<Value> {
    use tokio::net::TcpListener;

    // ── Start a minimal mock daemon on a random loopback port ──────────────────
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|e| panic!("bind mock daemon: {e}"));
    let port = listener
        .local_addr()
        .unwrap_or_else(|e| panic!("local addr: {e}"))
        .port();

    // Channel capacity 25: enough for the aggregate run.
    let (body_tx, mut body_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(25);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let mock_router = {
        let tx = body_tx.clone();
        axum::Router::new().fallback(axum::routing::any(move |req: Request<Body>| {
            let tx = tx.clone();
            async move {
                let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
                    .await
                    .unwrap_or_default();
                let _ = tx.try_send(body_bytes.to_vec());
                axum::http::StatusCode::OK
            }
        }))
    };

    tokio::spawn(async move {
        axum::serve(listener, mock_router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap_or(());
    });

    // ── Build CloneState pointing at the mock daemon ───────────────────────────
    // Auth token is a placeholder; the mock daemon accepts all requests.
    let auth_token = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
    let state = CloneState {
        client: reqwest::Client::new(),
        daemon: LockFileInfo {
            port,
            auth_token: auth_token.to_string(),
            pid: std::process::id(),
        },
        endpoint_base: format!("http://127.0.0.1:{port}"),
    };

    // ── Drive the clone router in-process via tower::ServiceExt::oneshot ───────
    let router = build_router(state);
    let req = match Request::builder()
        .method("POST")
        .uri(endpoint_path)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(fixture).unwrap_or_default()))
    {
        Ok(r) => r,
        Err(e) => {
            error!("failed to build request for {endpoint_path}: {e}");
            return None;
        }
    };

    let response = router.oneshot(req).await.expect("oneshot");
    // The clone router must return 200 (fail-open / echo semantics).
    if response.status() != axum::http::StatusCode::OK {
        warn!(
            status = %response.status(),
            "clone router returned non-200; treating as forwarding failure"
        );
    }

    // ── Wait up to 500ms for the mock daemon to capture the forwarded POST ─────
    let received = tokio::time::timeout(Duration::from_millis(500), body_rx.recv()).await;

    // Signal graceful shutdown of mock daemon.
    let _ = shutdown_tx.send(());

    match received {
        Ok(Some(bytes)) => serde_json::from_slice(&bytes).ok(),
        _ => None,
    }
}

/// Main entry point for the `dtu-fidelity` subcommand.
///
/// Loads each of the 25 fixtures, drives the clone router, scores the result,
/// and prints a summary table (or JSON if `--json` is passed).
/// Exits 0 iff mean score ≥ threshold (default 0.95).
pub async fn run(args: DtuFidelityArgs) -> Result<()> {
    let root = workspace_root()?;

    let corpus_root = args
        .corpus
        .unwrap_or_else(|| root.join("tests/fixtures/dtu/claude-code-hook-2x"));

    if !corpus_root.exists() {
        anyhow::bail!(
            "fixture corpus not found at {:?} — \
             ensure tests/fixtures/dtu/claude-code-hook-2x/ exists",
            corpus_root
        );
    }

    info!("corpus root: {:?}", corpus_root);
    info!(
        "running dtu-fidelity oracle over {} fixtures",
        FIXTURE_CORPUS.len()
    );

    let mut results: Vec<FixtureResult> = Vec::with_capacity(FIXTURE_CORPUS.len());
    let mut total_score = 0.0_f64;
    let mut failures: Vec<String> = Vec::new();

    for spec in FIXTURE_CORPUS {
        let fixture_key = format!("{}/{}.json", spec.endpoint, spec.name);

        let fixture = load_fixture(&corpus_root, spec.endpoint, spec.name)
            .with_context(|| format!("load fixture {fixture_key}"))?;

        if !spec.expect_forward {
            // BC-HOOK-034 filter case: daemon must receive NOTHING.
            let received = drive_and_capture(spec.path, &fixture).await;
            let (score, received_flag) = if received.is_none() {
                // Correctly filtered — score 1.0.
                (1.0_f64, false)
            } else {
                // Filter broken — daemon received payload it should not have.
                error!(fixture = %fixture_key, "BC-HOOK-034 VIOLATION: daemon received payload that should have been filtered");
                failures.push(format!(
                    "{fixture_key}: filter broken, daemon received payload"
                ));
                (0.0_f64, true)
            };
            total_score += score;
            results.push(FixtureResult {
                fixture: fixture_key,
                score,
                mismatched_fields: vec![],
                received: received_flag,
            });
            continue;
        }

        // Normal forward case.
        let received = drive_and_capture(spec.path, &fixture).await;
        match received {
            None => {
                error!(fixture = %fixture_key, "daemon received nothing — clone did not forward");
                failures.push(format!(
                    "{fixture_key}: daemon received nothing (not forwarded)"
                ));
                // Score 0.0: the clone failed to forward.
                results.push(FixtureResult {
                    fixture: fixture_key,
                    score: 0.0,
                    mismatched_fields: vec!["<not-forwarded>".to_string()],
                    received: false,
                });
                // Do NOT add to total_score — contributes 0.
            }
            Some(clone_json) => {
                let fs = FixtureScore::compute(fixture_key.clone(), &fixture, &clone_json);
                if fs.score < args.threshold {
                    failures.push(format!(
                        "{fixture_key}: score={:.4}, mismatched={:?}",
                        fs.score, fs.mismatched_fields
                    ));
                }
                total_score += fs.score;
                results.push(FixtureResult {
                    fixture: fixture_key,
                    score: fs.score,
                    mismatched_fields: fs.mismatched_fields,
                    received: true,
                });
            }
        }
    }

    let count = FIXTURE_CORPUS.len();
    let mean = if count > 0 {
        total_score / count as f64
    } else {
        0.0
    };

    // User-facing CLI output — writeln! to stdout is intentional; not log noise.
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if args.json {
        // Emit structured JSON for CI artifact upload.
        // User-facing CLI output; stdout is the correct sink (not tracing).
        let output = serde_json::json!({
            "fixture_count": count,
            "mean_score": mean,
            "threshold": args.threshold,
            "pass": mean >= args.threshold && failures.is_empty(),
            "failures": failures,
            "results": results,
        });
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&output).context("serialize JSON output")?
        )
        .context("write JSON output")?;
    } else {
        // Human-readable table — user-facing CLI output; stdout is the correct sink.
        writeln!(out, "\n{:<60} {:>8}  Mismatched", "Fixture", "Score")
            .context("write table header")?;
        writeln!(out, "{}", "-".repeat(90)).context("write separator")?;
        for r in &results {
            let mismatch = if r.mismatched_fields.is_empty() {
                String::new()
            } else {
                r.mismatched_fields.join(", ")
            };
            let marker = if r.score >= args.threshold { " " } else { "!" };
            writeln!(out, "{marker}{:<59} {:>8.4}  {}", r.fixture, r.score, mismatch)
                .context("write fixture row")?;
        }
        writeln!(out, "{}", "-".repeat(90)).context("write separator")?;
        writeln!(
            out,
            "Mean fidelity: {mean:.4}  (threshold: {:.2})",
            args.threshold
        )
        .context("write mean fidelity")?;

        if failures.is_empty() {
            writeln!(
                out,
                "\nPASS — all {count} fixtures meet the {:.2} threshold.",
                args.threshold
            )
            .context("write PASS line")?;
        } else {
            writeln!(
                out,
                "\nFAIL — {}/{count} fixtures below threshold:",
                failures.len()
            )
            .context("write FAIL line")?;
            for f in &failures {
                writeln!(out, "  - {f}").context("write failure entry")?;
            }
        }
    }

    if mean < args.threshold || !failures.is_empty() {
        anyhow::bail!(
            "dtu-fidelity FAIL: mean={mean:.4} < {:.2} or {} fixture(s) failed",
            args.threshold,
            failures.len()
        );
    }

    Ok(())
}
