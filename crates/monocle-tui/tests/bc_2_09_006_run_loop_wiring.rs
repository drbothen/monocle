//! TDD test suite for BC-2.09.006 run-loop wiring requirements.
//!
//! These tests exercise WIRING — not the helper functions in isolation. The
//! bc_2_09_006_resize_debounce.rs file tests the helper functions themselves.
//! These tests verify that the helpers are actually invoked from the correct call sites.
//!
//! Findings covered:
//!
//!   BLOCKER-001 → test_BC_2_09_006_run_loop_tick_fires_resizepane_without_check_call
//!                 test_BC_2_09_006_run_loop_tick_fires_resizepane_after_resize_event
//!   HIGH-001    → test_BC_2_09_006_per_render_layout_change_triggers_detection
//!   LOW-001     → test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop
//!   (exit cleanup) → test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state
//!   ADV3-HIGH-001 (overlay transition) →
//!       test_BC_2_09_006_overlay_transition_clears_resize_state
//!
//! # Seam contract (BLOCKER-001 / HIGH-001)
//!
//! The production run loop (`App::run()`) is too large to drive directly in unit tests.
//! These tests assert against `tick_resize_debounce(&mut app)` — a public seam that the
//! implementer MUST add to `crates/monocle-tui/src/app.rs` and call from the run loop's
//! post-render step.
//!
//! The seam signature is:
//!
//! ```rust
//! pub fn tick_resize_debounce(app: &mut App) { /* ... */ }
//! ```
//!
//! It must:
//!   1. Read `app.last_pty_pane_area` (set by the render path after each `terminal.draw()`).
//!   2. When in `AppMode::EmbeddedTerminal`, call `on_resize_detected(app, session_id,
//!      area.height, area.width)` to detect layout-change resizes that do NOT produce a
//!      crossterm resize event.
//!   3. Call `check_resize_debounce(app, session_id, current_rows, current_cols)` to
//!      fire the pending ResizePane once the 50ms window has elapsed.
//!
//! The run loop MUST call `tick_resize_debounce(&mut app)` on every tick after render.
//! Without this call, no ResizePane is ever sent (BLOCKER-001: feature is production-inert).
//!
//! # Red Gate summary
//!
//! These tests fail before implementation for the following reasons:
//!
//!   BLOCKER-001 tests: `tick_resize_debounce` does not exist as a public function in
//!                      `monocle_tui::app` — compile error. Once the seam exists, the
//!                      tests fail because `app.last_pty_pane_area` must be populated by
//!                      the render step AND `tick_resize_debounce` must read it; the run
//!                      loop never calls `tick_resize_debounce`.
//!   HIGH-001 test:     `tick_resize_debounce` does not exist — compile error. Once added,
//!                      the test fails because `last_pty_pane_area` is the detection input
//!                      but the seam does not read it yet.
//!   LOW-001 test:      `tick_resize_debounce` does not exist — compile error. Once added,
//!                      the Dashboard mode guard in the seam prevents detection.
//!   exit-cleanup test: `exit_embedded_terminal` does not call `clear_resize_debounce_state`
//!                      — deadline and last_sent_size remain Some after exit.
//!   ADV3-HIGH-001 test: `on_permission_prompt_queued` does not call
//!                      `clear_resize_debounce_state` — deadline and last_sent_size
//!                      remain Some after the EmbeddedTerminal→Overlay transition.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::{ClientToServer, PermissionPromptPayload};
use monocle_tui::app::{
    exit_embedded_terminal, on_permission_prompt_queued, on_resize_detected, tick_resize_debounce,
};
use monocle_tui::App;
use ratatui::layout::Rect;
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const SESSION_A: &str = "00000001-0000-4000-8001-000000000042";

/// Build a minimal `App` in `AppMode::EmbeddedTerminal` with a wired `ipc_tx`.
/// Installs a vt100 parser for `SESSION_A` at 24 rows x 80 cols.
/// Returns `(app, ipc_rx)`.
fn make_app_in_embedded(
    session_id: &str,
    parser_rows: u16,
    parser_cols: u16,
) -> (App, mpsc::Receiver<ClientToServer>) {
    let mut app = App::new(MonocleConfig::default());
    let (tx, rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    let parser = vt100::Parser::new(parser_rows, parser_cols, app.scrollback_rows as usize);
    app.pty_parsers.insert(session_id.to_string(), parser);
    app.pty_scroll_offsets.insert(session_id.to_string(), 0);
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_owned(),
        prior: FocusSnapshot::Sessions,
    };
    (app, rx)
}

/// Build a minimal `App` in `AppMode::Dashboard`.
fn make_app_in_dashboard() -> App {
    App::new(MonocleConfig::default())
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
// BLOCKER-001 — run loop must call tick_resize_debounce after each render tick.
//
// BC-2.09.006 PC-1/2: ResizePane must be sent after the 50ms debounce window
// expires from the run loop itself, not from any test-helper call.
//
// Seam: `tick_resize_debounce(&mut app)` in monocle_tui::app — the implementer
// MUST add this function AND call it from `App::run()` after every `terminal.draw()`.
//
// Red Gate:
//   - Compile error: `tick_resize_debounce` does not exist in `monocle_tui::app`.
//   - After seam is added: test fails because the run loop tick does not read
//     `last_pty_pane_area` and call `check_resize_debounce` on expiry.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_run_loop_tick_fires_resizepane_without_check_call
///
/// BLOCKER-001 / BC-2.09.006 PC-2:
///   After the per-render path arms the debounce (via `last_pty_pane_area` mismatch
///   detected by `tick_resize_debounce`), a single call to `tick_resize_debounce`
///   after 50ms MUST emit `ClientToServer::ResizePane` WITHOUT any direct call to
///   `check_resize_debounce` in the test.
///
///   The seam `tick_resize_debounce` represents the post-render step in
///   `App::run()` that the implementer will add. It reads `last_pty_pane_area`,
///   calls `on_resize_detected` for layout-change detection, and calls
///   `check_resize_debounce` to fire the debounce. The test only calls the
///   seam — it never touches `check_resize_debounce` or `on_resize_detected`
///   directly. This guards against implementing the helpers as dead code.
///
///   This test routes entirely through the per-render `tick_resize_debounce` /
///   `last_pty_pane_area` path so that removing the redundant crossterm
///   `Event::Resize` arm from `handle_crossterm_event` cannot break it
///   (OBS-001 enablement).
///
///   Red Gate: `tick_resize_debounce` does not exist yet — compile error.
///   Once implemented, the test passes ONLY if the seam reads `last_pty_pane_area`
///   AND calls `check_resize_debounce` internally (i.e., the full pipeline runs).
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_run_loop_tick_fires_resizepane_without_check_call() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Simulate the render step: last_pty_pane_area reflects a new pane size (30x100).
    // Parser is at (24x80) — mismatch causes tick_resize_debounce to arm the debounce.
    // (In production, terminal.draw() sets last_pty_pane_area before tick_resize_debounce runs.)
    app.last_pty_pane_area = Some(Rect {
        x: 0,
        y: 0,
        width: 100, // cols
        height: 30, // rows
    });

    // Step 1: first tick — arms the debounce via pane-area mismatch detection.
    // The test does NOT call on_resize_detected directly (OBS-001 / guards dead code).
    tick_resize_debounce(&mut app);

    // Pre-condition: no ResizePane sent yet (debounce not expired).
    let msgs_before = drain(&mut rx);
    assert!(
        msgs_before.is_empty(),
        "BLOCKER-001 pre-condition: no ResizePane before debounce expires — got {}",
        msgs_before.len()
    );

    // Advance clock past 50ms debounce.
    tokio::time::advance(Duration::from_millis(55)).await;

    // Step 2: call tick_resize_debounce — the post-render seam that the run loop MUST call.
    // This is the ONLY path to ResizePane; the test does NOT call check_resize_debounce.
    //
    // Red Gate: `tick_resize_debounce` does not exist yet — compile error.
    // After implementation: passes only if the seam fires check_resize_debounce internally.
    tick_resize_debounce(&mut app);

    // Assert: ResizePane was sent.
    let msgs = drain(&mut rx);
    let resize_msgs: Vec<_> = msgs
        .iter()
        .filter(|m| matches!(m, ClientToServer::ResizePane { .. }))
        .collect();

    assert!(
        !resize_msgs.is_empty(),
        "BLOCKER-001 / BC-2.09.006 PC-2: tick_resize_debounce must emit ResizePane after \
         50ms debounce — no ResizePane sent. The run loop's post-render step is absent \
         (tick_resize_debounce not wired into App::run())."
    );

    if let Some(ClientToServer::ResizePane {
        session_id,
        rows,
        cols,
    }) = resize_msgs.first()
    {
        assert_eq!(
            session_id.as_str(),
            SESSION_A,
            "BLOCKER-001: ResizePane session_id must be SESSION_A"
        );
        assert_eq!(*rows, 30, "BLOCKER-001: ResizePane rows must be 30");
        assert_eq!(*cols, 100, "BLOCKER-001: ResizePane cols must be 100");
    }
}

/// test_BC_2_09_006_run_loop_tick_fires_resizepane_after_resize_event
///
/// BLOCKER-001 / BC-2.09.006 PC-1/2 (end-to-end sequence via seam):
///   Full pipeline test — debounce arming through the per-render tick path only:
///     1. `last_pty_pane_area` is updated to (30x100), simulating a render cycle
///        that observed a pane size change. The parser was initialized at (24x80),
///        so there is a mismatch.
///     2. `tick_resize_debounce` is called at t=0. It detects the mismatch via
///        `on_resize_detected` internally and arms the debounce. No ResizePane yet.
///     3. At t=49ms, a second tick must NOT fire ResizePane (window not elapsed).
///     4. At t=50ms, a third tick MUST fire ResizePane with the new dimensions.
///
///   This test routes entirely through the per-render `tick_resize_debounce` /
///   `last_pty_pane_area` path — the production-authoritative detection path for
///   layout-change resizes. It does NOT rely on any crossterm event arm so that
///   removing the redundant `Event::Resize` arm from `handle_crossterm_event`
///   cannot break this test (OBS-001 enablement).
///
///   Red Gate: `tick_resize_debounce` does not exist — compile error.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_run_loop_tick_fires_resizepane_after_resize_event() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Simulate the render step writing last_pty_pane_area with a new pane size.
    // Parser is at (24x80); pane area is now (30x100) — mismatch triggers detection.
    app.last_pty_pane_area = Some(Rect {
        x: 0,
        y: 0,
        width: 100, // cols
        height: 30, // rows
    });

    // t=0: first tick — tick_resize_debounce detects the mismatch, arms debounce.
    // No ResizePane should be sent yet (debounce window not elapsed).
    tick_resize_debounce(&mut app);
    let msgs_t0 = drain(&mut rx);
    assert!(
        msgs_t0
            .iter()
            .all(|m| !matches!(m, ClientToServer::ResizePane { .. })),
        "BLOCKER-001 / BC-2.09.006 PC-2: tick_resize_debounce must NOT fire ResizePane \
         at t=0ms (debounce window not elapsed) — got ResizePane immediately"
    );
    assert!(
        app.resize_debounce_deadline.is_some(),
        "BLOCKER-001 / BC-2.09.006 PC-1: debounce deadline must be armed after first \
         tick_resize_debounce detects pane-area mismatch"
    );

    // t=49ms: tick must NOT fire ResizePane.
    tokio::time::advance(Duration::from_millis(49)).await;
    tick_resize_debounce(&mut app);
    let msgs_49ms = drain(&mut rx);
    assert!(
        msgs_49ms
            .iter()
            .all(|m| !matches!(m, ClientToServer::ResizePane { .. })),
        "BLOCKER-001 / BC-2.09.006 PC-2: tick_resize_debounce must NOT fire ResizePane \
         at t=49ms (debounce window not elapsed) — got ResizePane too early"
    );

    // t=50ms: tick MUST fire ResizePane.
    tokio::time::advance(Duration::from_millis(1)).await;
    tick_resize_debounce(&mut app);
    let msgs_50ms = drain(&mut rx);
    let resize_msgs: Vec<_> = msgs_50ms
        .iter()
        .filter(|m| matches!(m, ClientToServer::ResizePane { .. }))
        .collect();

    assert_eq!(
        resize_msgs.len(),
        1,
        "BLOCKER-001 / BC-2.09.006 PC-2: exactly one ResizePane after 50ms via \
         tick_resize_debounce — got {}",
        resize_msgs.len()
    );
    if let Some(ClientToServer::ResizePane { rows, cols, .. }) = resize_msgs.first() {
        assert_eq!(*rows, 30, "BLOCKER-001: rows must be 30");
        assert_eq!(*cols, 100, "BLOCKER-001: cols must be 100");
    }
}

// ---------------------------------------------------------------------------
// HIGH-001 — per-render layout-change detection via tick_resize_debounce.
//
// BC-2.09.006 PC-1 ("at each render cycle") requires that a pane-area change
// detected from `last_pty_pane_area` (set by the render path) triggers resize
// detection even when NO `crossterm::Event::Resize` was received.
//
// This covers panel layout changes (user splits a pane, layout weight changes)
// where the crossterm terminal size is unchanged but the embedded terminal pane
// area shrinks or grows.
//
// Red Gate:
//   - Compile error: `tick_resize_debounce` does not exist.
//   - After seam added: test fails because `tick_resize_debounce` does not read
//     `last_pty_pane_area` and call `on_resize_detected` for layout-change detection.
//     The parser size stays at the old (24x80) value; the debounce is never armed;
//     no ResizePane is sent.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_per_render_layout_change_triggers_detection
///
/// HIGH-001 / BC-2.09.006 PC-1 ("at each render cycle"):
///   When `app.last_pty_pane_area` changes to a different size than the parser
///   (simulating a panel layout change without a crossterm Event::Resize), a call to
///   `tick_resize_debounce` must:
///     1. Detect the mismatch and call `on_resize_detected` internally.
///     2. Resize the local parser immediately to the new area dimensions.
///     3. Arm the 50ms debounce deadline.
///     4. After 50ms, send `ClientToServer::ResizePane` with the new dimensions.
///
///   NO `Event::Resize`, `on_resize_detected`, or `check_resize_debounce` is called
///   directly by the test — only `tick_resize_debounce` via the seam.
///
///   Red Gate: `tick_resize_debounce` does not exist — compile error.
///   After seam added: fails because `last_pty_pane_area` is not read for detection.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_per_render_layout_change_triggers_detection() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Simulate a layout change: the render step captured a new pane area (30x100)
    // that differs from the current parser size (24x80).
    // NO Event::Resize was received — this is a panel layout change only.
    app.last_pty_pane_area = Some(Rect {
        x: 0,
        y: 0,
        width: 100, // cols = 100 (Rect.width)
        height: 30, // rows = 30 (Rect.height)
    });

    // Pre-condition: parser is still at old size (24x80).
    {
        let (rows, cols) = app.pty_parsers.get(SESSION_A).unwrap().screen().size();
        assert_eq!(
            (rows, cols),
            (24, 80),
            "HIGH-001 pre-condition: parser must be at initial size (24, 80)"
        );
    }

    // Pre-condition: no debounce armed.
    assert!(
        app.resize_debounce_deadline.is_none(),
        "HIGH-001 pre-condition: no debounce deadline should be armed before tick"
    );

    // Step 1: tick once — should detect layout change and arm debounce.
    // Red Gate: tick_resize_debounce does not exist — compile error.
    tick_resize_debounce(&mut app);

    // Assert: local parser was resized immediately (BC-2.09.006 postcondition 3).
    // Red Gate (after seam): fails if tick_resize_debounce does not read last_pty_pane_area.
    {
        let (rows, cols) = app.pty_parsers.get(SESSION_A).unwrap().screen().size();
        assert_eq!(
            (rows, cols),
            (30, 100),
            "HIGH-001 / BC-2.09.006 PC-1: parser must be resized to (30, 100) after \
             tick_resize_debounce detects layout change from last_pty_pane_area — \
             got ({}, {}). tick_resize_debounce does not read last_pty_pane_area.",
            rows,
            cols
        );
    }

    // Assert: debounce was armed.
    assert!(
        app.resize_debounce_deadline.is_some(),
        "HIGH-001 / BC-2.09.006 PC-1: debounce deadline must be armed after \
         tick_resize_debounce detects a layout-change resize — got None"
    );

    // Assert: no IPC sent yet (debounce window not elapsed).
    let msgs_before = drain(&mut rx);
    assert!(
        msgs_before.is_empty(),
        "HIGH-001: no ResizePane before debounce expires — got {}",
        msgs_before.len()
    );

    // Step 2: advance 50ms and tick again — ResizePane must be sent.
    tokio::time::advance(Duration::from_millis(55)).await;
    tick_resize_debounce(&mut app);

    let msgs = drain(&mut rx);
    let resize_msgs: Vec<_> = msgs
        .iter()
        .filter(|m| matches!(m, ClientToServer::ResizePane { .. }))
        .collect();

    assert_eq!(
        resize_msgs.len(),
        1,
        "HIGH-001 / BC-2.09.006 PC-1+PC-2: exactly one ResizePane must be sent after \
         tick_resize_debounce detects layout change and 50ms elapses — got {}",
        resize_msgs.len()
    );
    if let Some(ClientToServer::ResizePane {
        session_id,
        rows,
        cols,
    }) = resize_msgs.first()
    {
        assert_eq!(
            session_id.as_str(),
            SESSION_A,
            "HIGH-001: ResizePane session_id must be SESSION_A"
        );
        assert_eq!(
            *rows, 30,
            "HIGH-001: ResizePane rows must be 30 (from last_pty_pane_area.height)"
        );
        assert_eq!(
            *cols, 100,
            "HIGH-001: ResizePane cols must be 100 (from last_pty_pane_area.width)"
        );
    }
}

// ---------------------------------------------------------------------------
// LOW-001 / EC-236 — Dashboard mode guard via tick_resize_debounce
//
// When `AppMode::Dashboard` is active, `tick_resize_debounce` must be a no-op.
//
// Red Gate: `tick_resize_debounce` does not exist — compile error.
// After seam added: this test verifies the mode guard in the seam suppresses resize.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop
///
/// LOW-001 / BC-2.09.006 EC-236:
///   When `AppMode::Dashboard` is active and `tick_resize_debounce` is called with
///   a non-None `last_pty_pane_area`, it must NOT arm the debounce or send ResizePane.
///   The seam's first action must be to check the mode and return early in non-EmbeddedTerminal.
///
///   Red Gate: compile error until `tick_resize_debounce` exists.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop() {
    let mut app_db = make_app_in_dashboard();
    // Inject a last_pty_pane_area to ensure tick does not arm resize state in Dashboard mode.
    app_db.last_pty_pane_area = Some(Rect {
        x: 0,
        y: 0,
        width: 100,
        height: 30,
    });

    tick_resize_debounce(&mut app_db);

    assert!(
        app_db.resize_debounce_deadline.is_none(),
        "LOW-001 / BC-2.09.006 EC-236: tick_resize_debounce in Dashboard mode must NOT \
         arm the debounce deadline — got Some(_). Mode guard absent in seam."
    );
}

// ---------------------------------------------------------------------------
// exit_embedded_terminal must call clear_resize_debounce_state (no seam needed here)
//
// Red Gate: exit_embedded_terminal() does NOT call clear_resize_debounce_state.
// The debounce deadline and last_sent_size remain Some after exit. The assertions
// fail because they expect None.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state
///
/// BC-2.09.006 Invariant 1 (exit cleanup):
///   When `exit_embedded_terminal(app, session_id)` is called with an armed debounce
///   deadline and a non-None `last_sent_size`, both fields must be cleared to `None`.
///   This ensures the next `enter_embedded_terminal` starts with clean resize state.
///
///   Red Gate: `exit_embedded_terminal` does NOT call `clear_resize_debounce_state`.
///   After `exit_embedded_terminal`, `resize_debounce_deadline` and `last_sent_size`
///   remain `Some(_)` — the assertions on `is_none()` fail.
#[tokio::test]
async fn test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state() {
    let (mut app, _rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Arm the debounce state.
    on_resize_detected(&mut app, SESSION_A, 30, 100);
    app.last_sent_size = Some((30, 100));

    assert!(
        app.resize_debounce_deadline.is_some(),
        "exit-cleanup pre-condition: resize_debounce_deadline must be Some after on_resize_detected"
    );
    assert!(
        app.last_sent_size.is_some(),
        "exit-cleanup pre-condition: last_sent_size must be Some after manual assignment"
    );

    // Act: call the real exit_embedded_terminal production path.
    exit_embedded_terminal(&mut app, SESSION_A);

    // Assert: both resize state fields are cleared.
    // Red Gate: fails if exit_embedded_terminal does not call clear_resize_debounce_state.
    assert!(
        app.resize_debounce_deadline.is_none(),
        "BC-2.09.006: exit_embedded_terminal must clear resize_debounce_deadline to None — \
         got Some(_). exit_embedded_terminal does not call clear_resize_debounce_state()."
    );
    assert!(
        app.last_sent_size.is_none(),
        "BC-2.09.006: exit_embedded_terminal must clear last_sent_size to None — \
         got Some({:?}). exit_embedded_terminal does not call clear_resize_debounce_state().",
        app.last_sent_size
    );

    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "exit-cleanup: exit_embedded_terminal must restore AppMode::Dashboard"
    );
}

// ---------------------------------------------------------------------------
// ADV3-HIGH-001 — overlay transition must clear resize debounce state
//
// BC-2.09.006 Invariants 1/2/3 and S-042 Tasks ("clear resize_debounce_deadline
// and last_sent_size on AppMode exit from EmbeddedTerminal"):
//
// The invariant covers ALL exit paths from EmbeddedTerminal — not only
// `exit_embedded_terminal`, but also the permission-overlay transition driven by
// `on_permission_prompt_queued`. When a permission prompt arrives while the TUI
// is in EmbeddedTerminal mode, `on_permission_prompt_queued` transitions
// `app.mode` to `AppMode::Overlay` (app.rs ~1606-1618). This path does NOT
// currently call `clear_resize_debounce_state`, leaving stale debounce state
// that causes cross-session resize suppression and bypass bugs.
//
// Red Gate: `on_permission_prompt_queued` does NOT call `clear_resize_debounce_state`.
// After the transition, `resize_debounce_deadline` and `last_sent_size` remain
// `Some(_)` — the `is_none()` assertions fail.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_overlay_transition_clears_resize_state
///
/// ADV3-HIGH-001 / BC-2.09.006 Invariants 1/2/3:
///   When a permission prompt arrives while in `AppMode::EmbeddedTerminal` with
///   an armed resize debounce and a set `last_sent_size`, `on_permission_prompt_queued`
///   must clear both `resize_debounce_deadline` and `last_sent_size` as part of the
///   EmbeddedTerminal→Overlay mode transition.
///
///   Scenario:
///     1. App is in EmbeddedTerminal for SESSION_A.
///     2. `on_resize_detected` arms the debounce (resize_debounce_deadline = Some(...)).
///     3. `last_sent_size` is set to Some((30, 100)) (simulates a previously sent resize).
///     4. `on_permission_prompt_queued` is called — the production IPC path that
///        transitions EmbeddedTerminal → AppMode::Overlay (app.rs ~1606-1618).
///     5. Post-transition: resize_debounce_deadline MUST be None.
///        Post-transition: last_sent_size MUST be None.
///
///   Without the fix, stale resize state persists across sessions:
///   - A subsequent re-entry into EmbeddedTerminal for a different session B may
///     suppress its first ResizePane (Invariant 2 dedup fires on stale last_sent_size).
///   - Or the armed debounce fires for the wrong session after mode transition.
///
///   Red Gate: `on_permission_prompt_queued` does NOT call `clear_resize_debounce_state`.
///   Both `resize_debounce_deadline` and `last_sent_size` remain `Some(_)` after the
///   transition. The `is_none()` assertions fail.
#[tokio::test]
async fn test_BC_2_09_006_overlay_transition_clears_resize_state() {
    let (mut app, _rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Step 1: arm the debounce — simulates a resize that occurred while in EmbeddedTerminal.
    on_resize_detected(&mut app, SESSION_A, 30, 100);
    // Also set last_sent_size to simulate a previously confirmed resize.
    app.last_sent_size = Some((30, 100));

    // Pre-condition: both resize fields are armed.
    assert!(
        app.resize_debounce_deadline.is_some(),
        "ADV3-HIGH-001 pre-condition: resize_debounce_deadline must be Some after \
         on_resize_detected"
    );
    assert!(
        app.last_sent_size.is_some(),
        "ADV3-HIGH-001 pre-condition: last_sent_size must be Some after manual assignment"
    );
    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "ADV3-HIGH-001 pre-condition: app must be in EmbeddedTerminal before transition"
    );

    // Step 2: drive the production permission-overlay transition path.
    // This is the real on_permission_prompt_queued (app.rs ~1606-1618) that transitions
    // EmbeddedTerminal → AppMode::Overlay. It does NOT go through exit_embedded_terminal.
    let payload = PermissionPromptPayload {
        prompt_id: Uuid::new_v4(),
        session_id: SESSION_A.to_string(),
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({"command": "echo hello"}),
        old_content: None,
        new_content: None,
    };
    on_permission_prompt_queued(&mut app, payload);

    // Assert: mode transitioned to Overlay.
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "ADV3-HIGH-001: on_permission_prompt_queued must transition mode to AppMode::Overlay"
    );

    // Assert: resize debounce state is cleared.
    // Red Gate: on_permission_prompt_queued does NOT call clear_resize_debounce_state.
    // Both fields remain Some(_). These assertions fail against the current code.
    assert!(
        app.resize_debounce_deadline.is_none(),
        "ADV3-HIGH-001 / BC-2.09.006 Invariant 1: on_permission_prompt_queued must clear \
         resize_debounce_deadline to None on EmbeddedTerminal→Overlay transition — \
         got Some(_). on_permission_prompt_queued does not call clear_resize_debounce_state()."
    );
    assert!(
        app.last_sent_size.is_none(),
        "ADV3-HIGH-001 / BC-2.09.006 Invariant 2: on_permission_prompt_queued must clear \
         last_sent_size to None on EmbeddedTerminal→Overlay transition — \
         got Some({:?}). on_permission_prompt_queued does not call clear_resize_debounce_state().",
        app.last_sent_size
    );
}
