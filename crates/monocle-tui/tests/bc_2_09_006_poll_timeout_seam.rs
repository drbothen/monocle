//! TDD test suite for F-S042-MED-001: resize-aware run-loop poll timeout.
//!
//! # Finding context
//!
//! BC-2.09.006 PC-8 requires that a one-shot resize fires ResizePane within ≤100ms of
//! pane-area change. The 50ms debounce deadline is armed in `on_resize_detected`.
//! However, the run loop calls `event::poll(tick_rate)` with `tick_rate = 100ms`
//! unconditionally — it blocks for up to 100ms regardless of whether a resize deadline
//! is imminent, so `tick_resize_debounce` cannot be called until the next tick.
//!
//! In the worst case: resize detected at t=0 (deadline at t+50ms), run loop entered
//! `event::poll(100ms)` at t=1ms → blocks until t=101ms → tick_resize_debounce fires
//! at t=101ms (51ms late, total latency 101ms > 100ms limit).
//!
//! # Fix (implementer task)
//!
//! The implementer must add `resize_aware_poll_timeout` as a pure helper function in
//! `crates/monocle-tui/src/app.rs` and use it at the `event::poll(...)` call site:
//!
//! ```text
//! pub fn resize_aware_poll_timeout(
//!     deadline: Option<tokio::time::Instant>,
//!     tick_rate: Duration,
//!     now: tokio::time::Instant,
//! ) -> Duration
//! ```
//!
//! And the call site changes from:
//! ```text
//! if event::poll(tick_rate)?
//! ```
//! to:
//! ```text
//! let poll_timeout = resize_aware_poll_timeout(app.resize_debounce_deadline, tick_rate,
//!                                              tokio::time::Instant::now());
//! if event::poll(poll_timeout)?
//! ```
//!
//! # Test strategy (deterministic, not wall-clock)
//!
//! The PRIMARY Red Gate test (`test_BC_2_09_006_MED001_poll_timeout_shrinks_to_deadline`)
//! exercises the pure `resize_aware_poll_timeout` helper directly. It is fully deterministic —
//! no wall-clock, no sleep, no flaky timing. It fails against current code because the function
//! does not exist (compile error). Once added by the implementer, it directly catches the bug:
//! the current run loop passes `tick_rate` unconditionally; the correct implementation passes
//! the shrunk timeout.
//!
//! Three sub-cases cover the full contract:
//!   (a) No deadline pending → returns tick_rate (idle-path unchanged).
//!   (b) Deadline at now+50ms with tick_rate=100ms → returns ≤50ms (deadline wins).
//!   (c) Deadline already elapsed → returns Duration::ZERO (saturating, immediate wake).
//!
//! # Red Gate failure mode
//!
//! Current code (HEAD 941fb41): `resize_aware_poll_timeout` does not exist in
//! `monocle_tui::app`. The test file fails to COMPILE, which is the correct Red Gate
//! failure — the function is entirely absent, not just incorrect.
//!
//! # Implementer sync
//!
//! The helper name and signature are PINNED by this test. The implementer MUST provide:
//!
//! ```rust
//! // In crates/monocle-tui/src/app.rs
//! pub fn resize_aware_poll_timeout(
//!     deadline: Option<tokio::time::Instant>,
//!     tick_rate: std::time::Duration,
//!     now: tokio::time::Instant,
//! ) -> std::time::Duration
//! ```
//!
//! And lib.rs must re-export it via:
//! ```rust
//! #[doc(hidden)]
//! pub use app::resize_aware_poll_timeout;
//! ```
//!
//! BC traceability: F-S042-MED-001 / BC-2.09.006 PC-8 (ResizePane latency ≤100ms).

// BC-2.09.006: test names embed the BC ID for traceability.
// This deviates from Rust non_snake_case but is required by the TDD naming contract.
#![allow(non_snake_case)]

// Seam import: monocle_tui::app is pub mod in lib.rs; any pub fn in app.rs is accessible here.
// This import creates the compile-error Red Gate: `resize_aware_poll_timeout` does not
// exist in app.rs at HEAD 941fb41. The implementer must add it to make these tests compile.
use monocle_tui::app::resize_aware_poll_timeout;
use std::time::Duration;
use tokio::time::Instant;

// ---------------------------------------------------------------------------
// PRIMARY deterministic test — pure helper contract
// (Red Gate: compile error — function does not exist against 941fb41)
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_MED001_poll_timeout_shrinks_to_deadline
///
/// F-S042-MED-001 / BC-2.09.006 PC-8:
///   `resize_aware_poll_timeout` must return a timeout that allows the run loop
///   to wake at the resize debounce deadline rather than blocking for a full tick_rate.
///
/// Sub-case (a): No pending resize deadline → must return tick_rate unchanged.
///   This ensures the idle path (no resize) is completely unaffected by the fix.
///
/// Sub-case (b): Deadline is `now + 50ms`, tick_rate is 100ms →
///   must return a Duration ≤ 50ms (not the full 100ms).
///   This is the core fix: the run loop must wake at the 50ms deadline, not 100ms.
///
/// Sub-case (c): Deadline is `now - 1ms` (already elapsed) →
///   must return Duration::ZERO (saturating_duration_since → 0, immediate wake).
///   This ensures an already-expired deadline triggers an immediate wake, not a wait.
///
/// # Red Gate
///
/// Fails to COMPILE against 941fb41 — `resize_aware_poll_timeout` is absent from
/// `monocle_tui::app`. Once the implementer adds the function and lib.rs re-exports it,
/// the test catches the logic bug: the current run loop passes `tick_rate` unconditionally.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_MED001_poll_timeout_shrinks_to_deadline() {
    let tick_rate = Duration::from_millis(100);

    // --- Sub-case (a): No pending resize deadline → returns tick_rate (idle path unchanged). ---
    {
        let now = Instant::now();
        let timeout = resize_aware_poll_timeout(None, tick_rate, now);
        assert_eq!(
            timeout, tick_rate,
            "F-S042-MED-001 / BC-2.09.006 PC-8 sub-case (a): \
             resize_aware_poll_timeout(None, 100ms, now) must return tick_rate (100ms). \
             The idle path must be completely unchanged — no resize pending, no poll shrink."
        );
    }

    // --- Sub-case (b): Deadline at now+50ms, tick_rate=100ms → must return ≤50ms. ---
    // The deadline is 50ms in the future; the run loop must not block for the full 100ms
    // or the ResizePane will arrive 50ms late (BC-2.09.006 PC-8 violation).
    {
        let now = Instant::now();
        let deadline = now + Duration::from_millis(50);
        let timeout = resize_aware_poll_timeout(Some(deadline), tick_rate, now);
        assert!(
            timeout <= Duration::from_millis(50),
            "F-S042-MED-001 / BC-2.09.006 PC-8 sub-case (b): \
             resize_aware_poll_timeout(Some(now+50ms), 100ms, now) must return ≤50ms \
             so the run loop wakes at the debounce deadline. Got {:?}. \
             Current bug: tick_rate (100ms) is returned unconditionally; \
             tick_resize_debounce cannot fire until the full 100ms tick elapses.",
            timeout
        );
        // Also verify it does not return zero (would busy-spin).
        // A small positive value (>0) is required: the deadline hasn't elapsed yet.
        // We use > Duration::ZERO as the lower bound.
        assert!(
            timeout > Duration::ZERO,
            "F-S042-MED-001 / BC-2.09.006 PC-8 sub-case (b): \
             resize_aware_poll_timeout must return > 0 when deadline is in the future — \
             got Duration::ZERO which would busy-spin the run loop. Got {:?}.",
            timeout
        );
    }

    // --- Sub-case (c): Deadline already elapsed → returns Duration::ZERO (immediate wake). ---
    // An already-elapsed deadline means tick_resize_debounce should fire immediately;
    // saturating_duration_since(past) → 0 is the correct saturating behavior.
    {
        let now = Instant::now();
        // Deadline is 1ms in the past (elapsed).
        let deadline = now - Duration::from_millis(1);
        let timeout = resize_aware_poll_timeout(Some(deadline), tick_rate, now);
        assert_eq!(
            timeout,
            Duration::ZERO,
            "F-S042-MED-001 / BC-2.09.006 PC-8 sub-case (c): \
             resize_aware_poll_timeout(Some(elapsed_deadline), 100ms, now) must return \
             Duration::ZERO so the run loop wakes immediately. Got {:?}. \
             saturating_duration_since an elapsed deadline must produce 0.",
            timeout
        );
    }
}

// ---------------------------------------------------------------------------
// SECONDARY contract: timeout must never exceed tick_rate.
//
// Even when a deadline is far in the future (e.g., a clock anomaly or very large
// debounce), the returned poll timeout must be capped at tick_rate. The run loop
// depends on tick_rate as the maximum latency for overlay-timer updates
// (BC-2.06.020 AC-009 — 100ms granularity for "Waiting: Ns" display).
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_MED001_poll_timeout_never_exceeds_tick_rate
///
/// F-S042-MED-001 / BC-2.09.006 PC-8:
///   `resize_aware_poll_timeout` must never return a timeout GREATER than tick_rate.
///   The timeout is `min(time_to_deadline, tick_rate)` — the tick_rate is the maximum.
///
///   If the deadline is very far in the future (e.g., now+10s, while tick_rate=100ms),
///   the function must return tick_rate (100ms), not the full 10s.
///   This guards against an implementation that forgets the `.min(tick_rate)` cap.
///
/// # Red Gate
///
/// Compile error until `resize_aware_poll_timeout` exists (same as primary test).
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_MED001_poll_timeout_never_exceeds_tick_rate() {
    let tick_rate = Duration::from_millis(100);
    let now = Instant::now();

    // Deadline very far in the future (10 seconds from now).
    // The function must cap at tick_rate (100ms), not return 10s.
    let far_future_deadline = now + Duration::from_secs(10);
    let timeout = resize_aware_poll_timeout(Some(far_future_deadline), tick_rate, now);

    assert!(
        timeout <= tick_rate,
        "F-S042-MED-001 / BC-2.09.006 PC-8: \
         resize_aware_poll_timeout must NEVER exceed tick_rate regardless of deadline. \
         With deadline=now+10s and tick_rate=100ms, expected ≤100ms, got {:?}. \
         The implementation must apply .min(tick_rate) to the computed deadline gap.",
        timeout
    );

    // The return must also be exactly tick_rate when the deadline is far away — the run
    // loop's timer-update frequency (BC-2.06.020 AC-009) must not be degraded.
    assert_eq!(
        timeout, tick_rate,
        "F-S042-MED-001 / BC-2.09.006 PC-8: \
         resize_aware_poll_timeout(Some(far_future), 100ms, now) must return exactly \
         tick_rate (100ms) when the deadline is far in the future. Got {:?}.",
        timeout
    );
}
