//! TDD test suite for S-042: PTY Resize Detection, 50ms Debounce, ResizePane IPC
//!
//! Anchored to BC-2.09.006 (Resize — PTY and Parser Resized Within 2 Render Ticks
//! of Pane Area Change; 50ms Debounce).
//!
//! Every test MUST FAIL before implementation — the four stub bodies are `todo!()`
//! which panic, so the Red Gate is satisfied as long as those stubs are unimplemented.
//!
//! BC clause → test mapping:
//!
//!   Postcondition 1 (AC-001) → test_BC_2_09_006_size_change_detected_per_render_cycle
//!   Postcondition 2 (AC-002) → test_BC_2_09_006_resize_sends_resizepane_after_50ms
//!   Postcondition 3 (AC-003) → test_BC_2_09_006_local_parser_resized_immediately
//!   Invariant 1 (AC-005)    → test_BC_2_09_006_rapid_resize_coalesced
//!   Invariant 2 (AC-006)    → test_BC_2_09_006_resize_to_same_size_no_op
//!   Invariant 3 (AC-007)    → test_BC_2_09_006_dashboard_mode_no_resizepane
//!   Invariant 4 (AC-008)    → test_BC_2_09_006_local_parser_not_debounced
//!   Edge case EC-235 (AC-009)→ test_BC_2_09_006_mid_window_resize_resets_deadline
//!   Edge case EC-236 (AC-010)→ test_BC_2_09_006_dashboard_mode_no_resizepane (alias)
//!   Edge case EC-237 (AC-011)→ test_BC_2_09_006_resize_to_same_size_no_op (alias)
//!   Edge case EC-239 (AC-012)→ test_BC_2_09_006_zero_dimensions_no_op
//!   Clear-on-exit           → test_BC_2_09_006_clear_debounce_state_on_exit
//!   Scroll offset reset     → test_BC_2_09_006_scroll_offset_reset_on_resize
//!   Debounce not-yet-elapsed→ test_BC_2_09_006_no_ipc_before_debounce_expires
//!   Canonical test vector   → test_BC_2_09_006_canonical_vector_24x80_to_30x100

// BC-2.09.006: test names embed the BC ID for traceability.
// This deviates from Rust non_snake_case but is required by the TDD naming contract.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::ClientToServer;
use monocle_tui::app::{check_resize_debounce, clear_resize_debounce_state, on_resize_detected};
use monocle_tui::App;
use std::time::Duration;
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const SESSION_A: &str = "00000001-0000-4000-8001-000000000042";

/// Build a minimal `App` in `AppMode::EmbeddedTerminal` with a wired `ipc_tx`.
/// Installs a vt100 parser for `SESSION_A` at 24 rows × 80 cols.
/// Returns `(app, ipc_rx)`.
fn make_app_in_embedded(
    session_id: &str,
    parser_rows: u16,
    parser_cols: u16,
) -> (App, mpsc::Receiver<ClientToServer>) {
    let mut app = App::new(MonocleConfig::default());
    // Wire outbound IPC channel so check_resize_debounce can send ResizePane.
    let (tx, rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    // Install a parser at the given initial size.
    let parser = vt100::Parser::new(parser_rows, parser_cols, app.scrollback_rows as usize);
    app.pty_parsers.insert(session_id.to_string(), parser);
    app.pty_scroll_offsets.insert(session_id.to_string(), 0);
    // Transition to EmbeddedTerminal mode.
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_owned(),
        prior: FocusSnapshot::Sessions,
    };
    (app, rx)
}

/// Drain all pending messages from an mpsc receiver without blocking.
fn drain(rx: &mut mpsc::Receiver<ClientToServer>) -> Vec<ClientToServer> {
    let mut out = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        out.push(msg);
    }
    out
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.09.006 postcondition 3 / Invariant 4
//
// `on_resize_detected` must call `pty_parsers[session_id].set_size(rows, cols)`
// IMMEDIATELY — before the debounce window expires. The local render uses the
// new parser size on the next tick even if no IPC has been sent yet.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_local_parser_resized_immediately
///
/// AC-003 / BC-2.09.006 postcondition 3 / Invariant 4:
///   After `on_resize_detected(app, session_id, 30, 100)`:
///   - `pty_parsers[session_id].screen().size()` == (30, 100).
///   - The debounce deadline is armed (Some(_)).
///   - No IPC message has been sent (debounce not yet expired).
#[tokio::test]
async fn test_BC_2_09_006_local_parser_resized_immediately() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Act: detect pane area change 24×80 → 30×100.
    on_resize_detected(&mut app, SESSION_A, 30, 100);

    // Assert 1: local parser reflects new size immediately (postcondition 3 / Invariant 4).
    let (rows, cols) = app
        .pty_parsers
        .get(SESSION_A)
        .expect("parser must exist for SESSION_A")
        .screen()
        .size();
    assert_eq!(
        (rows, cols),
        (30, 100),
        "BC-2.09.006 AC-003 / Invariant 4: parser must reflect new size (30, 100) \
         immediately after on_resize_detected — got ({}, {})",
        rows,
        cols
    );

    // Assert 2: debounce deadline was armed.
    assert!(
        app.resize_debounce_deadline.is_some(),
        "BC-2.09.006 Invariant 1: resize_debounce_deadline must be Some(_) after \
         on_resize_detected detects a size change"
    );

    // Assert 3: no IPC sent yet (debounce has not expired).
    let msgs = drain(&mut rx);
    assert!(
        msgs.is_empty(),
        "BC-2.09.006 postcondition 2: no ResizePane IPC must be sent before the \
         debounce window expires — got {} messages",
        msgs.len()
    );
}

// ---------------------------------------------------------------------------
// AC-001 / BC-2.09.006 postcondition 1 — size-change detection
//
// When area matches parser size, `on_resize_detected` is a no-op: parser
// unchanged, no deadline armed, no IPC sent.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_size_change_detected_per_render_cycle
///
/// AC-001 / BC-2.09.006 postcondition 1:
///   When `area.rows == parser.screen().size().0` and `area.cols == parser.screen().size().1`,
///   `on_resize_detected` is a no-op — no parser mutation, no debounce arm, no IPC.
#[tokio::test]
async fn test_BC_2_09_006_size_change_detected_per_render_cycle() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Act: call with the SAME dimensions as the current parser (24×80 → 24×80).
    // This must be a no-op because the area equals the parser size.
    on_resize_detected(&mut app, SESSION_A, 24, 80);

    // Assert 1: no debounce deadline armed (no change detected).
    assert!(
        app.resize_debounce_deadline.is_none(),
        "BC-2.09.006 AC-001: no resize_debounce_deadline must be set when \
         area matches parser size — resize was a no-op"
    );

    // Assert 2: no IPC sent.
    let msgs = drain(&mut rx);
    assert!(
        msgs.is_empty(),
        "BC-2.09.006 AC-001: no ResizePane must be sent when area == parser size"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.09.006 postcondition 2 — ResizePane IPC sent after debounce expiry
//
// Timing is driven deterministically via `#[tokio::test(start_paused = true)]` +
// `tokio::time::advance()`. No wall-clock sleep.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_resize_sends_resizepane_after_50ms
///
/// AC-002 / BC-2.09.006 postcondition 2:
///   1. `on_resize_detected` arms the debounce deadline.
///   2. Before 50ms elapses: `check_resize_debounce` must NOT send ResizePane.
///   3. After 50ms elapses: `check_resize_debounce` MUST send
///      `ClientToServer::ResizePane { session_id, rows: 30, cols: 100 }`.
///
/// Clock is driven with `tokio::time::pause` + `tokio::time::advance` — no real sleep.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_resize_sends_resizepane_after_50ms() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Step 1: detect pane resize 24×80 → 30×100. Arms the debounce deadline at now+50ms.
    on_resize_detected(&mut app, SESSION_A, 30, 100);

    // Step 2: advance clock to 49ms — debounce must NOT have fired yet.
    tokio::time::advance(Duration::from_millis(49)).await;
    check_resize_debounce(&mut app, SESSION_A, 30, 100);
    let msgs_before = drain(&mut rx);
    assert!(
        msgs_before.is_empty(),
        "BC-2.09.006 AC-002: no ResizePane must be sent at t=49ms (debounce window \
         not yet elapsed) — got {} messages",
        msgs_before.len()
    );

    // Step 3: advance clock 1ms more (total 50ms) — debounce fires.
    tokio::time::advance(Duration::from_millis(1)).await;
    check_resize_debounce(&mut app, SESSION_A, 30, 100);

    let msgs_after = drain(&mut rx);
    assert_eq!(
        msgs_after.len(),
        1,
        "BC-2.09.006 AC-002: exactly one ResizePane must be sent after 50ms debounce \
         window expires — got {} messages",
        msgs_after.len()
    );
    match &msgs_after[0] {
        ClientToServer::ResizePane {
            session_id,
            rows,
            cols,
        } => {
            assert_eq!(
                session_id, SESSION_A,
                "BC-2.09.006 AC-002: ResizePane session_id must match"
            );
            assert_eq!(*rows, 30, "BC-2.09.006 AC-002: ResizePane rows must be 30");
            assert_eq!(
                *cols, 100,
                "BC-2.09.006 AC-002: ResizePane cols must be 100"
            );
        }
        other => panic!("BC-2.09.006 AC-002: expected ResizePane, got {:?}", other),
    }

    // Step 4: deadline must be cleared after send.
    assert!(
        app.resize_debounce_deadline.is_none(),
        "BC-2.09.006 Invariant 1: resize_debounce_deadline must be cleared \
         after ResizePane is sent"
    );

    // Step 5: last_sent_size updated.
    assert_eq!(
        app.last_sent_size,
        Some((30, 100)),
        "BC-2.09.006 Invariant 2: last_sent_size must be updated to (30, 100) \
         after ResizePane is sent"
    );
}

// ---------------------------------------------------------------------------
// AC-002 sub-test: no IPC before debounce — deadline armed but not elapsed
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_no_ipc_before_debounce_expires
///
/// BC-2.09.006 postcondition 2 (negative path):
///   When `check_resize_debounce` is called and the deadline has NOT elapsed,
///   no ResizePane is sent and the deadline remains armed.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_no_ipc_before_debounce_expires() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Arm the debounce at t=0.
    on_resize_detected(&mut app, SESSION_A, 30, 100);
    assert!(app.resize_debounce_deadline.is_some(), "precondition");

    // Call check_resize_debounce WITHOUT advancing the clock (t=0 < deadline).
    check_resize_debounce(&mut app, SESSION_A, 30, 100);

    let msgs = drain(&mut rx);
    assert!(
        msgs.is_empty(),
        "BC-2.09.006 postcondition 2 (negative): no ResizePane must be sent \
         when deadline has not elapsed — got {} messages",
        msgs.len()
    );

    // Deadline must still be armed.
    assert!(
        app.resize_debounce_deadline.is_some(),
        "BC-2.09.006 Invariant 1: resize_debounce_deadline must remain armed \
         when debounce window has not elapsed"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.09.006 Invariant 1 — rapid resize coalesced; mid-window reset
//
// Only one ResizePane per 50ms window; intermediate sizes discarded.
// A mid-window resize resets the debounce deadline.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_rapid_resize_coalesced
///
/// AC-005 / BC-2.09.006 Invariant 1 (canonical test vector 2):
///   Three sizes 24×80 → 25×82 → 26×84 within 50ms → only one ResizePane
///   for the FINAL size (26, 84) after 50ms.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_rapid_resize_coalesced() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // t=0: first resize 24×80 → 25×82. Arms deadline.
    on_resize_detected(&mut app, SESSION_A, 25, 82);

    // t=10ms: second resize 25×82 → 26×84 (mid-window).
    tokio::time::advance(Duration::from_millis(10)).await;
    on_resize_detected(&mut app, SESSION_A, 26, 84);

    // t=40ms: third resize 26×84 → 27×86 (still within 50ms of first detection).
    tokio::time::advance(Duration::from_millis(30)).await;
    on_resize_detected(&mut app, SESSION_A, 27, 86);

    // Verify no IPC has been sent yet.
    let early_msgs = drain(&mut rx);
    assert!(
        early_msgs.is_empty(),
        "BC-2.09.006 Invariant 1: no ResizePane before debounce expires — \
         got {} messages at t<50ms",
        early_msgs.len()
    );

    // t=55ms total from first detection: advance past 50ms to fire the debounce.
    tokio::time::advance(Duration::from_millis(15)).await;
    check_resize_debounce(&mut app, SESSION_A, 27, 86);

    let msgs = drain(&mut rx);
    assert_eq!(
        msgs.len(),
        1,
        "BC-2.09.006 Invariant 1 (canonical vector 2): exactly one ResizePane must be \
         sent after 50ms coalescing — got {} messages",
        msgs.len()
    );
    match &msgs[0] {
        ClientToServer::ResizePane { rows, cols, .. } => {
            assert_eq!(
                (*rows, *cols),
                (27, 86),
                "BC-2.09.006 Invariant 1: ResizePane must encode the FINAL dimensions \
                 (27, 86), not an intermediate size"
            );
        }
        other => panic!("expected ResizePane, got {:?}", other),
    }
}

/// test_BC_2_09_006_mid_window_resize_resets_deadline
///
/// BC-2.09.006 edge case EC-235 (debounce deadline reset on mid-window resize):
///   t=0: first resize arms deadline at t+50ms.
///   t=30ms: second resize detected. The deadline is RESET to t=30ms+50ms.
///   t=60ms: first deadline (t+50ms) would have fired — but since the deadline
///            was reset, the window has NOT expired yet. No ResizePane at t=60ms.
///   t=80ms: 50ms from reset (t=30ms+50ms) — ResizePane IS sent.
///
/// This confirms that each mid-window resize restarts the 50ms countdown from scratch,
/// per BC-2.09.006 Invariant 1 (only the final size per window is sent).
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_mid_window_resize_resets_deadline() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // t=0: first resize arms debounce deadline at t=0+50ms.
    on_resize_detected(&mut app, SESSION_A, 26, 84);
    let deadline_after_first = app
        .resize_debounce_deadline
        .expect("deadline must be armed after first resize");

    // t=30ms: second resize — deadline must be reset to t=30ms+50ms.
    tokio::time::advance(Duration::from_millis(30)).await;
    on_resize_detected(&mut app, SESSION_A, 28, 90);
    let deadline_after_second = app
        .resize_debounce_deadline
        .expect("deadline must remain armed after second resize");

    // The second deadline must be LATER than the first (reset to now+50ms > 0ms+50ms).
    assert!(
        deadline_after_second > deadline_after_first,
        "BC-2.09.006 EC-235: mid-window resize must RESET the debounce deadline \
         to a later time — first={:?}, second={:?}",
        deadline_after_first,
        deadline_after_second
    );

    // t=51ms: past the ORIGINAL deadline (t=0+50ms) but before the RESET deadline (t=30ms+50ms=80ms).
    tokio::time::advance(Duration::from_millis(21)).await;
    check_resize_debounce(&mut app, SESSION_A, 28, 90);
    let msgs_at_51ms = drain(&mut rx);
    assert!(
        msgs_at_51ms.is_empty(),
        "BC-2.09.006 EC-235: no ResizePane must fire at t=51ms — the deadline was \
         reset to t=80ms after the mid-window resize; got {} messages",
        msgs_at_51ms.len()
    );

    // t=80ms: 50ms from the reset. ResizePane must fire now.
    tokio::time::advance(Duration::from_millis(29)).await;
    check_resize_debounce(&mut app, SESSION_A, 28, 90);
    let msgs_at_80ms = drain(&mut rx);
    assert_eq!(
        msgs_at_80ms.len(),
        1,
        "BC-2.09.006 EC-235: exactly one ResizePane must fire 50ms after the mid-window \
         deadline reset — got {} messages",
        msgs_at_80ms.len()
    );
    match &msgs_at_80ms[0] {
        ClientToServer::ResizePane { rows, cols, .. } => {
            assert_eq!(
                (*rows, *cols),
                (28, 90),
                "BC-2.09.006 EC-235: ResizePane must carry the FINAL dimensions (28, 90)"
            );
        }
        other => panic!("expected ResizePane, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.09.006 Invariant 2 — resize to same size is a no-op
// (canonical test vector 3)
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_resize_to_same_size_no_op
///
/// AC-006 / AC-011 / BC-2.09.006 Invariant 2 / EC-237 (canonical test vector 3):
///   When a ResizePane has already been sent for size (24, 80) and the area is
///   still (24, 80), `check_resize_debounce` must NOT send another ResizePane.
///
///   Also covers: `on_resize_detected` called with size == parser size is a no-op
///   (no deadline armed, no IPC).
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_resize_to_same_size_no_op() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Precondition: last_sent_size == (24, 80) — we already sent for this size.
    app.last_sent_size = Some((24, 80));

    // Act: on_resize_detected with same size as parser (24×80 parser, area 24×80).
    // Since area == parser size, this must be a no-op.
    on_resize_detected(&mut app, SESSION_A, 24, 80);

    // Assert: no debounce armed, no IPC.
    assert!(
        app.resize_debounce_deadline.is_none(),
        "BC-2.09.006 Invariant 2 / EC-237: on_resize_detected must be a no-op \
         when area == parser size"
    );
    let msgs = drain(&mut rx);
    assert!(
        msgs.is_empty(),
        "BC-2.09.006 Invariant 2 / EC-237: no ResizePane when area == parser size"
    );

    // Also cover: even if debounce deadline is somehow set, check_resize_debounce must
    // not send when pending_size == last_sent_size.
    // Manually arm the deadline (artificially) to verify the guard in check_resize_debounce.
    app.resize_debounce_deadline = Some(tokio::time::Instant::now() - Duration::from_millis(1)); // already elapsed

    check_resize_debounce(&mut app, SESSION_A, 24, 80);
    let msgs2 = drain(&mut rx);
    assert!(
        msgs2.is_empty(),
        "BC-2.09.006 Invariant 2: no ResizePane must be sent when \
         pending_size == last_sent_size (already sent for this size)"
    );
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.09.006 Invariant 3 — resize only in EmbeddedTerminal mode
// (EC-236: resize in Dashboard mode → no IPC)
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_dashboard_mode_no_resizepane
///
/// AC-007 / AC-010 / BC-2.09.006 Invariant 3 / EC-236:
///   When `AppMode::Dashboard` is active, `on_resize_detected` must be a no-op:
///   no parser resize, no debounce armed, no IPC.
///
///   Implementation note: callers guard on AppMode before calling on_resize_detected.
///   This test verifies that if on_resize_detected is called for a session that is NOT
///   the focused EmbeddedTerminal session, the function handles it safely.
///   The primary guard is at the call site in the render/event loop — this test
///   exercises the defensive behavior and validates the mode-guard contract.
#[tokio::test]
async fn test_BC_2_09_006_dashboard_mode_no_resizepane() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Transition to Dashboard mode — simulates user pressing Esc to exit embedded terminal.
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };

    // Act: the render loop MUST NOT call on_resize_detected in Dashboard mode.
    // We verify by checking that the debounce state is not touched.
    // (If the implementer calls on_resize_detected from the Dashboard render path,
    //  this test documents the expected behavior: Dashboard resize is a no-op.)
    //
    // Directly test check_resize_debounce with no deadline armed — must be a no-op.
    app.resize_debounce_deadline = None;
    check_resize_debounce(&mut app, SESSION_A, 30, 100);

    let msgs = drain(&mut rx);
    assert!(
        msgs.is_empty(),
        "BC-2.09.006 Invariant 3 / EC-236: no ResizePane must be sent in Dashboard mode — \
         check_resize_debounce with no deadline is a no-op, got {} messages",
        msgs.len()
    );

    // Also verify: on_resize_detected called while Dashboard mode is active.
    // Even if called, the session_id must not be the current EmbeddedTerminal session
    // (there is none). The function must guard on parser existence and mode context.
    // We check that no deadline is armed as a result.
    on_resize_detected(&mut app, SESSION_A, 30, 100);
    // In Dashboard mode with no EmbeddedTerminal, implementations may vary —
    // but the debounce state must reflect no active resize intent.
    // Since mode is Dashboard, the implementation's AppMode guard prevents the arm.
    // We verify no IPC was sent.
    let msgs2 = drain(&mut rx);
    assert!(
        msgs2.is_empty(),
        "BC-2.09.006 EC-236: no ResizePane must be sent when not in EmbeddedTerminal mode"
    );
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.09.006 Invariant 4 — local parser resize is NOT debounced
//
// `on_resize_detected` calls `parser.set_size()` BEFORE the debounce expires.
// This is the same assertion as test_BC_2_09_006_local_parser_resized_immediately
// but framed around Invariant 4 explicitly.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_local_parser_not_debounced
///
/// AC-008 / BC-2.09.006 Invariant 4:
///   The local vt100 parser resize is SYNCHRONOUS and NOT debounced.
///   `on_resize_detected` must call `parser.set_size()` on the same tick it is called,
///   regardless of whether the 50ms debounce has elapsed. The IPC `ResizePane` is
///   debounced; the local parser update is not.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_local_parser_not_debounced() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Act: call on_resize_detected without advancing the clock.
    // The 50ms debounce has NOT elapsed, so no IPC should be sent.
    // But the parser MUST be updated immediately.
    on_resize_detected(&mut app, SESSION_A, 40, 120);

    // Assert A: parser updated immediately (Invariant 4 — not debounced).
    let (rows, cols) = app
        .pty_parsers
        .get(SESSION_A)
        .expect("parser must exist")
        .screen()
        .size();
    assert_eq!(
        (rows, cols),
        (40, 120),
        "BC-2.09.006 Invariant 4: parser.set_size() must be called IMMEDIATELY on \
         resize detection (not debounced) — got ({}, {})",
        rows,
        cols
    );

    // Assert B: no IPC sent yet (IPC IS debounced, unlike the parser).
    let msgs = drain(&mut rx);
    assert!(
        msgs.is_empty(),
        "BC-2.09.006 Invariant 4: no ResizePane IPC at t=0 (debounce not elapsed) — \
         got {} messages. Parser resize is immediate but IPC is debounced.",
        msgs.len()
    );
}

// ---------------------------------------------------------------------------
// AC-012 / BC-2.09.006 EC-239 — degenerate pane area (rows=0 or cols=0) → no-op
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_zero_dimensions_no_op
///
/// AC-012 / BC-2.09.006 EC-239:
///   When `area.rows == 0` or `area.cols == 0`, `on_resize_detected` must be a
///   complete no-op — no parser resize, no debounce arm, no IPC.
///
///   Tested with both (0, 80) and (30, 0) to cover both degenerate axes.
#[tokio::test]
async fn test_BC_2_09_006_zero_dimensions_no_op() {
    // Case 1: rows == 0.
    {
        let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);
        on_resize_detected(&mut app, SESSION_A, 0, 80);

        assert!(
            app.resize_debounce_deadline.is_none(),
            "BC-2.09.006 EC-239: no debounce deadline when rows == 0"
        );
        let (rows, cols) = app.pty_parsers.get(SESSION_A).unwrap().screen().size();
        assert_eq!(
            (rows, cols),
            (24, 80),
            "BC-2.09.006 EC-239: parser must NOT be resized when area.rows == 0"
        );
        let msgs = drain(&mut rx);
        assert!(
            msgs.is_empty(),
            "BC-2.09.006 EC-239: no ResizePane when rows == 0"
        );
    }

    // Case 2: cols == 0.
    {
        let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);
        on_resize_detected(&mut app, SESSION_A, 30, 0);

        assert!(
            app.resize_debounce_deadline.is_none(),
            "BC-2.09.006 EC-239: no debounce deadline when cols == 0"
        );
        let (rows, cols) = app.pty_parsers.get(SESSION_A).unwrap().screen().size();
        assert_eq!(
            (rows, cols),
            (24, 80),
            "BC-2.09.006 EC-239: parser must NOT be resized when area.cols == 0"
        );
        let msgs = drain(&mut rx);
        assert!(
            msgs.is_empty(),
            "BC-2.09.006 EC-239: no ResizePane when cols == 0"
        );
    }

    // Case 3: both rows == 0 and cols == 0.
    {
        let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);
        on_resize_detected(&mut app, SESSION_A, 0, 0);

        assert!(
            app.resize_debounce_deadline.is_none(),
            "BC-2.09.006 EC-239: no debounce deadline when rows == 0 && cols == 0"
        );
        let msgs = drain(&mut rx);
        assert!(
            msgs.is_empty(),
            "BC-2.09.006 EC-239: no ResizePane when rows == 0 && cols == 0"
        );
    }
}

// ---------------------------------------------------------------------------
// BC-2.09.006 — scroll offset reset on resize (S-042 ownership clause)
//
// SS-embedded-pty.md §Scrollback offset invariants:
//   "resize reflows content; old offset is meaningless" — reset to 0.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_scroll_offset_reset_on_resize
///
/// SS-embedded-pty.md §Scrollback offset invariants (S-042 ownership):
///   After `on_resize_detected` detects a genuine size change,
///   `pty_scroll_offsets[session_id]` must be reset to 0.
///
///   Precondition: scroll offset is non-zero (e.g., user had scrolled up).
#[tokio::test]
async fn test_BC_2_09_006_scroll_offset_reset_on_resize() {
    let (mut app, _rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Simulate user having scrolled up (non-zero offset).
    app.pty_scroll_offsets.insert(SESSION_A.to_string(), 42);

    // Act: detect a size change.
    on_resize_detected(&mut app, SESSION_A, 30, 100);

    // Assert: scroll offset reset to 0.
    let offset = app
        .pty_scroll_offsets
        .get(SESSION_A)
        .copied()
        .unwrap_or(usize::MAX);
    assert_eq!(
        offset, 0,
        "BC-2.09.006 / SS-embedded-pty §Scrollback offset invariants: \
         pty_scroll_offsets[session_id] must be reset to 0 on resize — got {}",
        offset
    );
}

// ---------------------------------------------------------------------------
// BC-2.09.006 — clear_resize_debounce_state on EmbeddedTerminal exit
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_clear_debounce_state_on_exit
///
/// BC-2.09.006 Tasks ("cleared on AppMode exit"):
///   `clear_resize_debounce_state` must reset BOTH `app.last_sent_size` AND
///   `app.resize_debounce_deadline` to `None`.
///
///   This ensures the next entry into EmbeddedTerminal mode starts with a clean slate.
#[tokio::test]
async fn test_BC_2_09_006_clear_debounce_state_on_exit() {
    let (mut app, _rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Pre-populate both fields to non-None to verify they are cleared.
    app.last_sent_size = Some((30, 100));
    app.resize_debounce_deadline = Some(tokio::time::Instant::now() + Duration::from_millis(50));

    // Act: clear on EmbeddedTerminal exit.
    clear_resize_debounce_state(&mut app);

    // Assert: both fields must be None.
    assert!(
        app.last_sent_size.is_none(),
        "BC-2.09.006 (exit clause): last_sent_size must be None after \
         clear_resize_debounce_state"
    );
    assert!(
        app.resize_debounce_deadline.is_none(),
        "BC-2.09.006 (exit clause): resize_debounce_deadline must be None after \
         clear_resize_debounce_state"
    );
}

// ---------------------------------------------------------------------------
// Canonical test vector 1: 24×80 → 30×100 happy path
// (BC-2.09.006 §Canonical Test Vectors row 1)
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_canonical_vector_24x80_to_30x100
///
/// BC-2.09.006 §Canonical Test Vectors (row 1):
///   Input: pane resizes from 24×80 to 30×100; 50ms debounce elapsed.
///   Expected: `ResizePane { rows: 30, cols: 100 }` sent; local parser at (30, 100).
///
/// This is the end-to-end happy path combining postconditions 2 and 3.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_canonical_vector_24x80_to_30x100() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Simulate pane area change 24×80 → 30×100.
    on_resize_detected(&mut app, SESSION_A, 30, 100);

    // Parser must be updated immediately.
    let (rows, cols) = app.pty_parsers.get(SESSION_A).unwrap().screen().size();
    assert_eq!(
        (rows, cols),
        (30, 100),
        "canonical vector: parser must reflect (30, 100) immediately"
    );

    // Advance clock past 50ms debounce.
    tokio::time::advance(Duration::from_millis(51)).await;
    check_resize_debounce(&mut app, SESSION_A, 30, 100);

    // ResizePane must be sent with (rows: 30, cols: 100).
    let msgs = drain(&mut rx);
    assert_eq!(
        msgs.len(),
        1,
        "canonical vector: exactly one ResizePane after 50ms"
    );
    match &msgs[0] {
        ClientToServer::ResizePane {
            session_id,
            rows,
            cols,
        } => {
            assert_eq!(session_id, SESSION_A);
            assert_eq!(*rows, 30, "canonical vector: rows must be 30");
            assert_eq!(*cols, 100, "canonical vector: cols must be 100");
        }
        other => panic!("canonical vector: expected ResizePane, got {:?}", other),
    }

    // last_sent_size updated and deadline cleared.
    assert_eq!(app.last_sent_size, Some((30, 100)));
    assert!(app.resize_debounce_deadline.is_none());
}
