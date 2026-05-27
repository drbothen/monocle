//! monocle-runtime daemon binary entry point.
//!
//! Wired in S-016 as the daemon subprocess spawned by `monocle daemon start`.
//! The daemon:
//! 1. Creates a new session (setsid) and blocks SIGHUP so it survives terminal session end.
//! 2. Resolves the runtime directory via `monocle_runtime::lifecycle::resolve_runtime_dir`.
//! 3. Ensures the runtime directory exists with mode 0o700.
//! 4. Acquires the lock file via `monocle_runtime::lock::DaemonLock::acquire`.
//! 5. Loops indefinitely, servicing incoming connections (HTTP server wired in S-017).
//!
//! The lock file serves as the readiness signal for `monocle daemon start` — the CLI polls
//! for the lock file's existence and exits 0 once it appears (BC-2.04.004 PC-3/PC-4).
//!
//! # Daemon exit codes
//!
//! | Code | Condition |
//! |------|-----------|
//! | 0    | Graceful shutdown (SIGTERM received, reserved for future drain logic) |
//! | 1    | Startup failure (runtime dir unresolvable, lock write failure, etc.) |
//!
//! Exit code 1 on startup failure causes the CLI poller to detect early daemon exit
//! and map it to CLI exit code 71 (internal error) per BC-2.04.004 PC-8.
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
    // --- Step 1: Detach from calling terminal session ---
    //
    // Call setsid() to create a new session. This daemon is no longer the session leader
    // of the calling terminal, so it will not receive SIGHUP when the terminal session ends.
    // nix::unistd::setsid() is a safe Rust function (BC-2.04.004 INV-2 / AC-014).
    let _ = nix::unistd::setsid();

    // Block SIGHUP so that explicit kill(pid, SIGHUP) signals do not terminate the daemon.
    // This ensures the daemon survives terminal session end simulation in tests
    // (BC-2.04.004 INV-2 test: `test_ac_014_daemon_survives_terminal_session_end`).
    let mut hup_mask = nix::sys::signal::SigSet::empty();
    hup_mask.add(nix::sys::signal::Signal::SIGHUP);
    let _ = nix::sys::signal::sigprocmask(
        nix::sys::signal::SigmaskHow::SIG_BLOCK,
        Some(&hup_mask),
        None,
    );

    // --- Step 2: Resolve and create the runtime directory ---
    let runtime_dir = match monocle_runtime::lifecycle::resolve_runtime_dir() {
        Ok(dir) => dir,
        Err(e) => {
            tracing::error!(error = %e, "daemon: failed to resolve runtime directory");
            // SS-conventions-anti-patterns.md: use exit_with(), not std::process::exit().
            // exit_with(StartupFailure) emits the shutdown tracing log then exits 1.
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

    // --- Step 3: Acquire lock file ---
    //
    // DaemonLock::acquire writes the lock file atomically. On failure (read-only dir,
    // existing live lock, I/O error), exit 1 so the CLI poller detects early exit and
    // reports exit code 71 (internal error) to the user.
    let _lock = match monocle_runtime::lock::DaemonLock::acquire(&runtime_dir, 39_001) {
        Ok((lock, _token)) => lock,
        Err(e) => {
            tracing::error!(error = %e, "daemon: failed to acquire lock file");
            monocle_runtime::lifecycle::exit_with(
                monocle_runtime::lifecycle::DaemonExit::StartupFailure,
            );
        }
    };

    // --- Step 4: Daemon main loop ---
    //
    // In S-016, the daemon loop is a simple sleep loop. The HTTP server (axum, routes,
    // state management) will be wired in S-017. SIGTERM uses the default signal handler
    // (process terminates), which is correct for the stub — the daemon exits cleanly when
    // `monocle daemon stop` sends SIGTERM.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
