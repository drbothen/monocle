//! Test helper binary: a fake daemon that ignores SIGTERM and never writes a lock file.
//!
//! Used by `cli_daemon_stop.rs` timeout tests to simulate a daemon that does not
//! respond to SIGTERM within the stop timeout window. This ensures the `daemon stop`
//! command reaches the timeout code path (exit 2) rather than the success code path
//! (exit 0).
//!
//! Usage: spawn this binary, write its PID to the lock file, then invoke
//! `monocle daemon stop`. This process ignores SIGTERM, so stop will time out.
//!
//! The binary exits cleanly on SIGKILL (normal forced termination by test cleanup).
//! It has no child processes, so there are no orphaned processes after cleanup.

// Test helper binary: expect/unwrap are acceptable here (not in production code).
#![allow(clippy::expect_used, clippy::unwrap_used)]

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

    // Sleep for 60 seconds, waking periodically to check for SIGKILL (natural exit).
    // SIGKILL cannot be caught or ignored, so the process will be killed by test cleanup.
    for _ in 0..600 {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
