//! Test helper binary: a fake daemon that ignores SIGTERM and never writes a lock file.
//!
//! Used by `cli_daemon_stop.rs` timeout tests to simulate a daemon that does not
//! respond to SIGTERM within the stop timeout window. This ensures the `daemon stop`
//! command reaches the timeout code path (exit 2) rather than the success code path
//! (exit 0).
//!
//! Usage: spawn this binary with `Command::new(...).stdout(Stdio::piped()).spawn()`,
//! read one byte from stdout (the ready signal), then write the PID to the lock file
//! and invoke `monocle daemon stop`. This process ignores SIGTERM, so stop will time out.
//!
//! The binary exits cleanly on SIGKILL (normal forced termination by test cleanup).
//! It has no child processes, so there are no orphaned processes after cleanup.
//!
//! # Ready Protocol
//!
//! The binary writes a single `b'R'` byte to stdout immediately after installing the
//! SIGTERM ignore handler. The parent process MUST read this byte before sending
//! SIGTERM (via `monocle daemon stop`) to eliminate the race window between `spawn()`
//! and `SIG_IGN` installation.

// Test helper binary: expect/unwrap are acceptable here (not in production code).
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    // Install a no-op SIGTERM handler: ignore the signal entirely.
    // This simulates a daemon that is draining and not yet ready to exit.
    let running = Arc::new(AtomicBool::new(true));

    // SAFETY: signal handlers are inherently unsafe in Rust. We use a simple
    // atomic flag approach here. No allocations or complex operations in the handler.
    unsafe {
        nix::sys::signal::signal(
            nix::sys::signal::Signal::SIGTERM,
            nix::sys::signal::SigHandler::SigIgn,
        )
        .expect("install SIGTERM ignore handler");
    }

    // Ready signal: write a single 'R' byte to stdout AFTER SIG_IGN is installed.
    // The parent reads this byte to confirm the handler is active before invoking
    // `monocle daemon stop`, eliminating the SIGTERM race window.
    std::io::stdout()
        .write_all(b"R")
        .expect("write ready signal to stdout");
    std::io::stdout().flush().expect("flush ready signal");

    // Sleep for 60 seconds, waking periodically to check for SIGKILL (natural exit).
    // SIGKILL cannot be caught or ignored, so the process will be killed by test cleanup.
    for _ in 0..600 {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
