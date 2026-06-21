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
//!      `crossterm::Event::Resize` event.
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

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use crossterm::event::Event;
use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::ClientToServer;
use monocle_tui::app::{
    build_builtin_binding_layers, exit_embedded_terminal, on_resize_detected, tick_resize_debounce,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use monocle_tui::App;
use ratatui::layout::Rect;
use std::time::Duration;
use tokio::sync::mpsc;

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
///   After `Event::Resize(100, 30)` arms the debounce, a single call to
///   `tick_resize_debounce` after 50ms MUST emit `ClientToServer::ResizePane`
///   WITHOUT any direct call to `check_resize_debounce` in the test.
///
///   The seam `tick_resize_debounce` represents the post-render step in
///   `App::run()` that the implementer will add. It reads `last_pty_pane_area`,
///   calls `on_resize_detected` for layout-change detection, and calls
///   `check_resize_debounce` to fire the debounce. The test only calls the
///   seam — it never touches `check_resize_debounce` or `on_resize_detected`
///   directly. This guards against implementing the helpers as dead code.
///
///   Red Gate: `tick_resize_debounce` does not exist yet — compile error.
///   Once implemented, the test passes ONLY if the seam reads `last_pty_pane_area`
///   AND calls `check_resize_debounce` internally (i.e., the full pipeline runs).
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_run_loop_tick_fires_resizepane_without_check_call() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Step 1: dispatch Event::Resize via the real handle_crossterm_event path.
    // crossterm::event::Event::Resize(cols, rows) — crossterm order is (width, height).
    let resize_event = Event::Resize(100, 30); // cols=100, rows=30
    monocle_tui::app::handle_crossterm_event(&mut app, resize_event, &layers, &mut sessions_state)
        .await
        .ok();

    // The event path called on_resize_detected which armed the debounce and updated
    // the parser. Simulate the post-Event::Resize pane area that a real render would set.
    // (In production, terminal.draw() sets last_pty_pane_area before tick_resize_debounce runs.)
    app.last_pty_pane_area = Some(Rect {
        x: 0,
        y: 0,
        width: 100,
        height: 30,
    });

    // Pre-condition: no ResizePane sent yet.
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
///   Full pipeline test:
///     1. `Event::Resize(100, 30)` arms the debounce via `handle_crossterm_event`.
///     2. `last_pty_pane_area` is updated to reflect the rendered terminal area.
///     3. After 50ms, `tick_resize_debounce` fires ResizePane.
///
///   This test also verifies that `tick_resize_debounce` does NOT fire before 50ms.
///
///   Red Gate: `tick_resize_debounce` does not exist — compile error.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_run_loop_tick_fires_resizepane_after_resize_event() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Arm debounce via Event::Resize.
    monocle_tui::app::handle_crossterm_event(
        &mut app,
        Event::Resize(100, 30),
        &layers,
        &mut sessions_state,
    )
    .await
    .ok();

    // Set last_pty_pane_area as the render step would after terminal.draw().
    app.last_pty_pane_area = Some(Rect {
        x: 0,
        y: 0,
        width: 100,
        height: 30,
    });

    // At t=49ms: tick must NOT fire ResizePane.
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

    // At t=50ms: tick MUST fire ResizePane.
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
