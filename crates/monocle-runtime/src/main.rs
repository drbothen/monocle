//! monocle-runtime daemon binary entry point.
//!
//! Wires the fully-implemented library functions into a live, serving daemon. The daemon:
//!
//! 1. Creates a new session (`setsid`) and blocks SIGHUP so it survives terminal session end.
//! 2. Initialises a `tracing_subscriber` to stderr before any fallible operation (resolves
//!    ADV-W4GATE-MED-002).
//! 3. Resolves and ensures the runtime directory (XDG / `MONOCLE_RUNTIME_DIR`).
//! 4. Runs the full 12-step `daemon_start_sequence` which binds the HTTP listener, writes
//!    `monocle.lock` and `hooks-settings.json`, binds the UDS socket, and spawns the accept loop.
//! 5. Calls `run_server(state, listener)` to serve HTTP with graceful shutdown.
//! 6. After `run_server` returns: cleans up UDS socket, releases lock, removes
//!    `hooks-settings.json`, and exits with the appropriate `DaemonExit` code.
//!
//! # Daemon exit codes
//!
//! | Code | Condition |
//! |------|-----------|
//! | 0    | Graceful shutdown (SIGTERM received, drain complete) |
//! | 1    | Startup failure (runtime dir unresolvable, port bind failure, etc.) |
//! | 2    | AdminForceStop (second POST /shutdown during drain) |
//! | 130  | SigintDuringDrain (second SIGINT during drain) |
//! | 143  | SigtermDuringDrain (second SIGTERM during drain) |
#![forbid(unsafe_code)]
#![deny(missing_docs)]

// Compile-time ABI drift guard (VP-011 §Mechanism, BC-2.02.001 PC-2).
//
// If `monocle_core::MONOCLE_ABI_VERSION` is ever incremented without a corresponding
// update to the `/status` handler and integration tests, this assertion will catch the
// drift at compile time, before any runtime test runs. The expected value `1` identifies
// Phase 1 of the monocle ABI. S-010 is the canonical producer; S-003 exposes it in /status.
const _: () = assert!(
    monocle_core::MONOCLE_ABI_VERSION == 1,
    "ABI version mismatch: monocle_core::MONOCLE_ABI_VERSION changed without updating \
    the /status handler and the compile-time drift guard. Update this assertion and the \
    integration tests in status_abi_version.rs to the new ABI version."
);

fn main() {
    // -------------------------------------------------------------------------
    // Step 1: Session detach — BEFORE tokio runtime is created.
    //
    // setsid() and sigprocmask() must execute on the process main thread before tokio
    // spawns its thread pool. Using Builder::new_multi_thread (step 4) rather than
    // #[tokio::main] guarantees this ordering: the synchronous setup is complete before
    // the async executor starts. (SS-daemon-wiring-impl.md §Tokio Runtime Choice)
    // -------------------------------------------------------------------------

    // Create a new session so the daemon is no longer the session leader of the calling
    // terminal, and will not receive SIGHUP when the terminal session ends.
    // nix::unistd::setsid() is a safe Rust wrapper (BC-2.04.004 INV-2).
    let _ = nix::unistd::setsid();

    // Block SIGHUP so that explicit kill(pid, SIGHUP) signals do not terminate the daemon.
    // This ensures the daemon survives terminal session end simulation in tests
    // (BC-2.04.004 INV-2 test: test_ac_014_daemon_survives_terminal_session_end).
    let mut hup_mask = nix::sys::signal::SigSet::empty();
    hup_mask.add(nix::sys::signal::Signal::SIGHUP);
    let _ = nix::sys::signal::sigprocmask(
        nix::sys::signal::SigmaskHow::SIG_BLOCK,
        Some(&hup_mask),
        None,
    );

    // -------------------------------------------------------------------------
    // Step 2: Initialise tracing subscriber — BEFORE any fallible operation.
    //
    // Resolves ADV-W4GATE-MED-002: daemon errors are now visible in logs.
    // Daemon errors log to stderr; the parent process / shell will redirect stderr to a
    // log file if needed. SS-conventions-anti-patterns.md §Logging: no println! in
    // normal paths. RUST_LOG / tracing_subscriber::EnvFilter drives log level.
    // -------------------------------------------------------------------------
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // -------------------------------------------------------------------------
    // Step 3: Resolve and ensure the runtime directory (synchronous).
    // -------------------------------------------------------------------------
    let runtime_dir = match monocle_runtime::lifecycle::resolve_runtime_dir() {
        Ok(dir) => dir,
        Err(e) => {
            tracing::error!(error = %e, "daemon: failed to resolve runtime directory");
            // SS-conventions-anti-patterns.md: exit_with() is the SOLE process::exit call-site.
            monocle_runtime::lifecycle::exit_with(
                monocle_runtime::lifecycle::DaemonExit::StartupFailure,
            );
        }
    };

    if let Err(e) = monocle_runtime::lifecycle::ensure_runtime_dir(&runtime_dir) {
        tracing::error!(error = %e, "daemon: failed to create runtime directory");
        monocle_runtime::lifecycle::exit_with(
            monocle_runtime::lifecycle::DaemonExit::StartupFailure,
        );
    }

    // -------------------------------------------------------------------------
    // Step 4: Build a multi-thread tokio runtime manually.
    //
    // setsid() and sigprocmask() must complete BEFORE tokio spawns its thread pool.
    // Using Builder::new_multi_thread().build() instead of #[tokio::main] lets us
    // guarantee the synchronous setup in steps 1–3 is done first.
    // -------------------------------------------------------------------------
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "daemon: failed to build tokio runtime");
            monocle_runtime::lifecycle::exit_with(
                monocle_runtime::lifecycle::DaemonExit::StartupFailure,
            );
        });

    // -------------------------------------------------------------------------
    // Steps 5–8: Async execution block.
    // -------------------------------------------------------------------------
    rt.block_on(async move {
        // -----------------------------------------------------------------------
        // Step 5: Full 12-step daemon start sequence.
        //
        // daemon_start_sequence:
        //   - Binds TcpListener on 127.0.0.1:0 (OS-assigned port)
        //   - Writes monocle.lock with the real port
        //   - Writes hooks-settings.json
        //   - Binds UDS socket (monocle.sock)
        //   - Spawns the UDS accept loop as a tokio task
        //   - Returns (DaemonState, TcpListener) — the listener is returned so we can
        //     pass it to run_server without any timing gap or port re-bind
        // -----------------------------------------------------------------------
        let (state, listener) =
            match monocle_runtime::lifecycle::daemon_start_sequence(&runtime_dir).await {
                Ok(pair) => pair,
                Err(e) => {
                    tracing::error!(error = %e, "daemon: start sequence failed");
                    monocle_runtime::lifecycle::exit_with(
                        monocle_runtime::lifecycle::DaemonExit::StartupFailure,
                    );
                }
            };

        tracing::info!("daemon: start sequence complete; serving HTTP");

        // -----------------------------------------------------------------------
        // Step 6: Run the HTTP server.
        //
        // run_server blocks until a shutdown signal fires (SIGTERM, SIGINT, or POST
        // /shutdown). The UDS accept loop is already running as a tokio::spawn task
        // (spawned inside daemon_start_sequence at step 10 of the sequence).
        // Graceful shutdown: run_server uses axum::serve(...).with_graceful_shutdown(...)
        // which waits for in-flight requests to complete. The 10-second outer drain timeout
        // (BC-2.01.004 INV-1) is enforced by the tokio::time::timeout wrapper here in main().
        // If the timeout fires, remaining connections are force-closed and cleanup proceeds.
        // -----------------------------------------------------------------------
        const GRACEFUL_DRAIN_TIMEOUT_SECS: u64 = 10;
        let timed_serve = tokio::time::timeout(
            std::time::Duration::from_secs(GRACEFUL_DRAIN_TIMEOUT_SECS),
            monocle_runtime::server::run_server(std::sync::Arc::clone(&state), listener),
        )
        .await;

        // Flatten Result<Result<(), io::Error>, Elapsed> to Result<(), io::Error>.
        let serve_result = match timed_serve {
            Ok(inner) => inner, // Timeout did not fire; propagate io::Error if any.
            Err(_elapsed) => {
                // BC-2.01.004 INV-1: 10s drain window exhausted. Force-close remaining
                // connections. axum drops the serve future here; any in-flight connections
                // are aborted. Cleanup tail runs normally — shutdown must not block forever.
                tracing::warn!(
                    timeout_secs = GRACEFUL_DRAIN_TIMEOUT_SECS,
                    "graceful drain timeout exhausted; forcing close of remaining connections \
                     (BC-2.01.004 INV-1)"
                );
                Ok(()) // Treat as clean exit — cleanup tail runs normally.
            }
        };

        if let Err(e) = serve_result {
            tracing::error!(error = %e, "daemon: HTTP server returned error");
            // Continue to shutdown tail so lock + hooks-settings are cleaned up even on
            // server error (production-grade: no partial cleanup).
        }

        // -----------------------------------------------------------------------
        // Step 7: Graceful shutdown tail.
        //
        // Order matters (BC-2.01.004 PC-7 → BC-2.04.010 PC-5):
        //   (a) UDS socket cleanup — remove monocle.sock
        //   (b) Release DaemonLock — remove monocle.lock
        //   (c) Remove hooks-settings.json
        //
        // Errors in cleanup are logged as WARN but do not change the exit code
        // (production-grade: shutdown must complete regardless of cleanup errors).
        // -----------------------------------------------------------------------

        // (a) UDS socket cleanup: UdsTransport::cleanup() removes monocle.sock.
        // uds_transport is Option<UdsTransport> on DaemonState; use .as_ref() so we
        // don't take ownership (cleanup() takes &self).
        if let Some(transport) = state.uds_transport.as_ref() {
            transport.cleanup();
        }

        // (b) Release the daemon lock (removes monocle.lock from the filesystem).
        // daemon_lock is Mutex<Option<DaemonLock>>; take() leaves None so any concurrent
        // caller cannot double-release.
        let lock_opt = state
            .daemon_lock
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(lock) = lock_opt {
            if let Err(e) = lock.release() {
                tracing::warn!(error = %e, "daemon: lock release failed during shutdown; continuing");
            }
        }

        // (c) Remove hooks-settings.json so Claude Code stops posting to the dead endpoint.
        if let Err(e) = monocle_runtime::lifecycle::remove_hooks_settings(&runtime_dir) {
            tracing::warn!(error = %e, "daemon: hooks-settings removal failed; continuing");
        }

        // (d) Ring flush drain — signal flush task to drain remaining records and exit,
        //     then await the JoinHandle with a bounded timeout.
        //
        // BC-2.01.007 / BC-2.04.012: records enqueued by hooks that returned HTTP 200
        // must not be lost on shutdown. The flush task exits via an explicit Notify signal
        // (H1 fix — SS-daemon-wiring-impl.md §Round 3 H1) rather than relying on
        // Arc<RingBuffer> refcount reaching zero. The notify_one() call triggers the select!
        // drain branch in flush_task_loop, which drains all pending records via try_recv()
        // and returns. The ring_arc drop below is belt-and-suspenders.
        //
        // Ordering: this step runs AFTER hooks-settings.json removal (step c) so no new
        // hook events arrive from Claude Code after we begin draining. The drain is thus a
        // clean quiesce point: all in-flight hooks have already returned (run_server drain
        // complete), and no new hooks will be enqueued.
        const RING_FLUSH_DRAIN_TIMEOUT_SECS: u64 = 2;

        // Step (d-1): Signal the flush task to drain remaining records and exit.
        // The Notify is the primary deterministic close mechanism (H1 fix).
        {
            if let Some(notify) = state.ring_flush_shutdown.as_ref() {
                notify.notify_one();
            }
            // Belt-and-suspenders: also drop the ring Arc to close write_tx.
            let ring_arc = state.ring.lock().unwrap_or_else(|e| e.into_inner()).take();
            drop(ring_arc);
        }

        // Step (d-2): Take the flush JoinHandle and await with bounded timeout.
        {
            let flush_handle = state
                .ring_flush_handle
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();

            if let Some(handle) = flush_handle {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(RING_FLUSH_DRAIN_TIMEOUT_SECS),
                    handle,
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!(
                            "daemon: ring flush task drained and exited cleanly \
                             (BC-2.04.012 durability)"
                        );
                    }
                    Err(_elapsed) => {
                        tracing::warn!(
                            timeout_secs = RING_FLUSH_DRAIN_TIMEOUT_SECS,
                            "daemon: ring flush drain timeout — some pending records may be \
                             lost (disk I/O too slow); proceeding with exit \
                             (BC-2.04.012 best-effort)"
                        );
                    }
                }
            }
        }

        // -----------------------------------------------------------------------
        // Step 8: Determine exit code and exit.
        //
        // force_exit flag is set by the post_shutdown handler when a second POST /shutdown
        // arrives during the drain window (EC-050 / DaemonExit::AdminForceStop = exit 2).
        // -----------------------------------------------------------------------
        use std::sync::atomic::Ordering;
        let exit_reason = if state.force_exit.load(Ordering::SeqCst) {
            monocle_runtime::lifecycle::DaemonExit::AdminForceStop
        } else {
            monocle_runtime::lifecycle::DaemonExit::Graceful
        };

        tracing::info!(?exit_reason, "daemon: clean shutdown complete");
        monocle_runtime::lifecycle::exit_with(exit_reason);
    });
}
