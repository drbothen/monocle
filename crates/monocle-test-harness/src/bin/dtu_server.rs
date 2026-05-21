//! Binary entry point for the DTU clone: `dtu-claude-code-hooks-v1`.
//!
//! Reads the monocle daemon lock file, resolves the connection details, and starts
//! the axum HTTP server that synthesizes hook POSTs for all 5 endpoints.
//!
//! Built artifact: `target/[debug|release]/dtu-claude-code-hooks-v1`
//! Source: `crates/monocle-test-harness/src/bin/dtu_server.rs`
//!
//! Source authority:
//! - S-DTU-001 AC-005 (Rust binary form; `cargo build --bin dtu-claude-code-hooks-v1`)
//! - S-DTU-001 AC-006 (MONOCLE_HOOK_ENDPOINT_BASE, MONOCLE_NO_AUTOSTART env vars)
//! - dtu-assessment.md v1.7.5 §Packaging Decision (lines 320-343)
//! - BC-HOOK-013 (lock file scan), BC-HOOK-015 (token), BC-HOOK-016 (auth header)

#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context as _, Result};

/// Print usage and exit 0.
fn print_help() {
    let msg = concat!(
        "dtu-claude-code-hooks-v1 — monocle DTU clone for Claude Code hook protocol\n",
        "\n",
        "USAGE:\n",
        "  dtu-claude-code-hooks-v1 [OPTIONS]\n",
        "\n",
        "OPTIONS:\n",
        "  --help, -h      Print this help message and exit\n",
        "  --version, -V   Print version and exit\n",
        "\n",
        "ENVIRONMENT:\n",
        "  MONOCLE_NO_AUTOSTART=1      Exit 0 immediately without connecting to daemon\n",
        "  MONOCLE_RUNTIME_DIR=<path>  Override default runtime dir (~/.monocle/)\n",
        "  MONOCLE_HOOK_ENDPOINT_BASE=<url>  Override daemon endpoint base URL\n",
        "  RUST_LOG=<filter>           Tracing log filter (default: info)\n",
    );
    // Use write! to stderr is allowed — but we must not use println!/eprintln!.
    // tracing is not yet initialized here; use std::io::Write directly.
    use std::io::Write as _;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(msg.as_bytes());
}

/// Print version and exit 0.
fn print_version() {
    use std::io::Write as _;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(
        out,
        "dtu-claude-code-hooks-v1 {}",
        env!("CARGO_PKG_VERSION")
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    // ── 1. Argument parsing (manual; no clap dep in monocle-test-harness yet) ──
    //    clap IS in workspace deps (v4.6) but monocle-test-harness does not inherit it.
    //    Manual arg parsing is sufficient for these 2 flags; adding clap is surfaced
    //    to architect as a follow-up (see commit message).
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(first) = args.first() {
        match first.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-V" => {
                print_version();
                return Ok(());
            }
            other => {
                // Unknown arg — exit 1 with message.
                // tracing not yet initialized, use stderr directly.
                use std::io::Write as _;
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "dtu-claude-code-hooks-v1: unknown argument: {other}\n\
                     Run with --help for usage."
                );
                std::process::exit(1);
            }
        }
    }

    // ── 2. MONOCLE_NO_AUTOSTART check (AC-006) ───────────────────────────────
    // When set, exit 0 immediately without connecting to the daemon.
    // Used in CI/test contexts where no daemon is available.
    if std::env::var("MONOCLE_NO_AUTOSTART")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        // tracing not yet initialized; write to stderr directly (not println!/eprintln!).
        use std::io::Write as _;
        let _ = writeln!(
            std::io::stderr().lock(),
            "dtu-claude-code-hooks-v1: MONOCLE_NO_AUTOSTART set — exiting cleanly"
        );
        return Ok(());
    }

    // ── 3. Initialize tracing subscriber (Fmt; structured; from RUST_LOG env) ─
    // BC-HOOK-013 / conventions: no println!/eprintln! in production code.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // ── 4. Resolve lock file path (BC-HOOK-013, BC-HOOK-014, AC-006) ──────────
    let runtime_dir = monocle_test_harness::dtu::lock_reader::resolve_runtime_dir();
    let lock_path = runtime_dir.join("monocle.lock");

    tracing::info!(lock_path = %lock_path.display(), "scanning for monocle lock file");

    // ── 5. Read lock file ──────────────────────────────────────────────────────
    let lock_info = monocle_test_harness::dtu::lock_reader::read_lock_file(&lock_path)
        .with_context(|| format!("lock file read failed: {}", lock_path.display()))?;

    tracing::info!(
        port = lock_info.port,
        pid = lock_info.pid,
        "lock file resolved"
    );

    // ── 6. Derive endpoint base (AC-006 env var override) ─────────────────────
    let endpoint_base = monocle_test_harness::dtu::lock_reader::derive_endpoint_base(&lock_info);
    tracing::info!(endpoint_base = %endpoint_base, "daemon endpoint base resolved");

    // ── 7. Build CloneState and start server ───────────────────────────────────
    // 5-second timeout aligns with BC-HOOK-022 hook timeout budget.
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build reqwest client")?;
    let state = monocle_test_harness::dtu::server::CloneState {
        client,
        daemon: lock_info,
        endpoint_base,
    };

    // Derive listen address: 127.0.0.1, port from MONOCLE_DTU_LISTEN_PORT env var
    // with fallback to DTU_DEFAULT_PORT (7860). Allows CI and tests to run multiple
    // DTU instances without port conflicts. AC-006 extensibility.
    //
    // Env var semantics (production-grade; R3 MED adversary finding):
    //   Unset         → fallback to DTU_DEFAULT_PORT (OK)
    //   Set-and-empty → fallback to DTU_DEFAULT_PORT (tolerated; treated as unset)
    //   Set-and-garbage ("abc", "99999", "0") → bail! with actionable message
    //   Set-and-valid u16 in 1..=65535 → use that port
    let port = match std::env::var("MONOCLE_DTU_LISTEN_PORT") {
        Ok(s) if s.is_empty() => monocle_test_harness::dtu::server::DTU_DEFAULT_PORT,
        Ok(s) => {
            let parsed: u16 = s.parse::<u16>().with_context(|| {
                format!(
                    "MONOCLE_DTU_LISTEN_PORT={:?} is not a valid u16 port (expected 1-65535)",
                    s
                )
            })?;
            if parsed == 0 {
                anyhow::bail!(
                    "MONOCLE_DTU_LISTEN_PORT=0 is not a valid listen port (port 0 requests an \
                     OS-assigned ephemeral port, which is not supported for the DTU server; \
                     use a fixed port in 1-65535)"
                );
            }
            parsed
        }
        Err(_) => monocle_test_harness::dtu::server::DTU_DEFAULT_PORT,
    };
    let listen_addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!(addr = %listen_addr, "starting DTU clone server");

    // ── 8. Serve with SIGTERM / Ctrl-C graceful shutdown ──────────────────────
    let server_future = monocle_test_harness::dtu::server::start_server(listen_addr, state);

    tokio::select! {
        result = server_future => {
            result.context("DTU clone server error")?;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("received Ctrl-C — shutting down DTU clone");
        }
        _ = wait_for_sigterm() => {
            tracing::info!("received SIGTERM — shutting down DTU clone");
        }
    }

    tracing::info!("DTU clone server stopped");
    Ok(())
}

/// Wait for SIGTERM on Unix. On non-Unix platforms, wait forever (never returns).
async fn wait_for_sigterm() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut sig) = signal(SignalKind::terminate()) {
            sig.recv().await;
        } else {
            // If we can't register SIGTERM, wait forever — Ctrl-C covers shutdown.
            std::future::pending::<()>().await;
        }
    }
    #[cfg(not(unix))]
    {
        std::future::pending::<()>().await;
    }
}
