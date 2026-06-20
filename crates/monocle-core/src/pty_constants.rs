//! Pure-core PTY pipeline constants (S-039 adversarial Pass-4, F-PASS4-MED-001).
//!
//! These constants are pure arithmetic — no I/O, no tokio, no side effects.
//! Lives in monocle-core per the Module Purity table in SS-embedded-pty.md.
//!
//! Consumers: monocle-tui `on_pty_output` (cap enforcement), `enter_embedded_terminal`
//! (timeout spawn), and `on_dump_window_timeout` (guard + cleanup).

/// Maximum cumulative byte volume buffered in `pending_pty_bytes` per session.
///
/// When the total byte count across all buffered `Vec<u8>` entries for a session
/// exceeds this limit, `on_pty_output` drops the OLDEST entry (not the incoming
/// one) and increments `pending_pty_drop_count` for that session.
///
/// 512 KiB is large enough to absorb a typical full scrollback dump without
/// discarding data, while bounding worst-case memory growth per-session.
///
/// F-PASS4-MED-001 / BC-2.09.001 Invariant 5 cap.
pub const MAX_PENDING_PTY_BYTES: usize = 512 * 1024;

/// Maximum number of buffered `PtyOutput` messages per session.
///
/// Each entry in `pending_pty_bytes[session_id]` is one `PtyOutput` message.
/// When the entry count exceeds this limit, `on_pty_output` drops the OLDEST
/// entry and increments `pending_pty_drop_count` for that session.
///
/// 4096 matches the daemon-side `RAM_RING_CAPACITY` so the buffer cap does not
/// artificially truncate a full ring replay.
///
/// F-PASS4-MED-001 / BC-2.09.001 Invariant 5 cap.
pub const MAX_PENDING_PTY_MESSAGES: usize = 4096;

/// Maximum wall-clock time a scrollback dump window may remain open.
///
/// If `ScrollbackDumpComplete` is NOT received within this window after
/// `enter_embedded_terminal` successfully sends `AttachSession`, the dump is
/// considered lost and `on_dump_window_timeout` fires to clean up the in-flight
/// state, log a WARN, and allow the user to re-enter and retry.
///
/// 10 seconds is chosen to be larger than any realistic scrollback transmission
/// time on localhost, while preventing an indefinite memory leak when the daemon
/// dies mid-dump.
///
/// F-PASS4-MED-001 / BC-2.09.001 Invariant 5 timeout.
pub const DUMP_WINDOW_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
