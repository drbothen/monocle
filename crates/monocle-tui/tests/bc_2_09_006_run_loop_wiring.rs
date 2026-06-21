//! TDD test suite for BC-2.09.006 run-loop wiring requirements.
//!
//! These tests exercise WIRING — not the helper functions in isolation. The
//! bc_2_09_006_resize_debounce.rs file tests the helper functions themselves.
//! These tests verify that the helpers are actually called from the right call sites.
//!
//! Findings covered:
//!
//!   BLOCKER-001 → test_BC_2_09_006_run_loop_resize_event_arms_debounce
//!                 test_BC_2_09_006_run_loop_resize_sends_resizepane_after_debounce
//!   LOW-001 (EC-236 mode guard via call site) →
//!                 test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop
//!   HIGH-001 (exit_embedded_terminal calls clear) →
//!                 test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state
//!
//! All tests MUST FAIL before implementation:
//!   BLOCKER-001 tests: `handle_crossterm_event` does not handle `Event::Resize` — the
//!                      debounce deadline is never armed.
//!   HIGH-001 test:     `exit_embedded_terminal` does not call `clear_resize_debounce_state`
//!                      — the deadline and last_sent_size remain Some after exit.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use crossterm::event::Event;
use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::ClientToServer;
use monocle_tui::app::{
    build_builtin_binding_layers, check_resize_debounce, exit_embedded_terminal, on_resize_detected,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
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
// BLOCKER-001 — run loop must call on_resize_detected + check_resize_debounce
//
// The TUI's handle_crossterm_event must handle crossterm::event::Event::Resize
// by calling on_resize_detected() + check_resize_debounce() when in
// AppMode::EmbeddedTerminal.
//
// RED GATE: handle_crossterm_event does NOT handle Event::Resize — the debounce
// deadline is never set. The test asserts it IS set after processing the event,
// so it fails because the wiring is absent.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_run_loop_resize_event_arms_debounce
///
/// BLOCKER-001 / BC-2.09.006 PC-1/2:
///   When `crossterm::event::Event::Resize(100, 30)` is processed by
///   `handle_crossterm_event` in `AppMode::EmbeddedTerminal`, the call site must
///   call `on_resize_detected(app, session_id, 30, 100)` (rows, cols in crossterm
///   Resize are (cols, rows) — see crossterm docs). The test asserts that after
///   processing the event, `app.resize_debounce_deadline` is `Some(_)`.
///
///   This test is the WIRING guard: it proves `on_resize_detected` is called from
///   the live event path, not just in isolation. Without this test, an implementer
///   could add `on_resize_detected` to a dead code path that never executes.
///
///   RED GATE: `handle_crossterm_event` does not handle `Event::Resize` — the
///   debounce deadline stays `None` after the call. Assertion fails.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_run_loop_resize_event_arms_debounce() {
    let (mut app, _rx) = make_app_in_embedded(SESSION_A, 24, 80);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Pre-condition: no debounce deadline armed yet.
    assert!(
        app.resize_debounce_deadline.is_none(),
        "BLOCKER-001 pre-condition: resize_debounce_deadline must be None before resize event"
    );

    // Act: dispatch Event::Resize(100, 30) via the real handle_crossterm_event path.
    // crossterm::event::Event::Resize(cols, rows) — note order is (width=cols, height=rows).
    let resize_event = Event::Resize(100, 30);
    let result = monocle_tui::app::handle_crossterm_event(
        &mut app,
        resize_event,
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(
        result.is_ok(),
        "BLOCKER-001: handle_crossterm_event must return Ok — got Err"
    );

    // Assert: debounce deadline was armed by the run loop wiring.
    // RED GATE: This fails because handle_crossterm_event ignores Event::Resize.
    assert!(
        app.resize_debounce_deadline.is_some(),
        "BLOCKER-001 / BC-2.09.006 PC-1: handle_crossterm_event must arm the 50ms debounce \
         deadline when Event::Resize is received in AppMode::EmbeddedTerminal — \
         resize_debounce_deadline is None (on_resize_detected was NOT called from the run loop)"
    );
}

/// test_BC_2_09_006_run_loop_resize_sends_resizepane_after_debounce
///
/// BLOCKER-001 / BC-2.09.006 PC-2:
///   After `Event::Resize` is processed and 50ms passes, calling
///   `check_resize_debounce` (or the run-loop tick that calls it) must send
///   `ClientToServer::ResizePane { session_id, rows: 30, cols: 100 }`.
///
///   This test exercises the full sequence: Event::Resize → handle_crossterm_event
///   arms debounce → time advances 50ms → check_resize_debounce sends ResizePane.
///
///   Note: `check_resize_debounce` is called explicitly here because the run loop
///   is not fully driven. The BLOCKER-001 wiring test above is the primary guard;
///   this test verifies the end-to-end sequence once wiring is in place.
///
///   RED GATE: `handle_crossterm_event` does not handle `Event::Resize` — the
///   debounce deadline stays None, so `check_resize_debounce` sends nothing.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_006_run_loop_resize_sends_resizepane_after_debounce() {
    let (mut app, mut rx) = make_app_in_embedded(SESSION_A, 24, 80);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act 1: dispatch resize event via the real handle_crossterm_event.
    let resize_event = Event::Resize(100, 30); // (cols=100, rows=30)
    monocle_tui::app::handle_crossterm_event(&mut app, resize_event, &layers, &mut sessions_state)
        .await
        .ok();

    // Advance time past the 50ms debounce window.
    tokio::time::advance(Duration::from_millis(55)).await;

    // Act 2: call check_resize_debounce (as the run loop tick would).
    // After implementation, the run loop itself will call this; here we call it
    // explicitly to test the end-to-end IPC emission.
    check_resize_debounce(&mut app, SESSION_A, 30, 100);

    // Assert: ResizePane was sent.
    let msgs = drain(&mut rx);
    let resize_msgs: Vec<_> = msgs
        .iter()
        .filter(|m| matches!(m, ClientToServer::ResizePane { .. }))
        .collect();

    // RED GATE: fails because handle_crossterm_event ignores Event::Resize, so
    // debounce was never armed, so check_resize_debounce sends nothing.
    assert!(
        !resize_msgs.is_empty(),
        "BLOCKER-001 / BC-2.09.006 PC-2: ResizePane must be sent after handle_crossterm_event \
         processes Event::Resize and 50ms elapses — no ResizePane sent (run loop wiring absent)"
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
        assert_eq!(
            *rows, 30,
            "BLOCKER-001: ResizePane rows must be 30 (canonical test vector)"
        );
        assert_eq!(
            *cols, 100,
            "BLOCKER-001: ResizePane cols must be 100 (canonical test vector)"
        );
    }
}

// ---------------------------------------------------------------------------
// LOW-001 / EC-236 mode guard — resize in Dashboard is a no-op at the CALL SITE
//
// When `Event::Resize` arrives in `AppMode::Dashboard`, `on_resize_detected` must
// NOT be called. The test verifies that the run loop's wiring includes the mode guard.
//
// RED GATE: `handle_crossterm_event` doesn't handle `Event::Resize` at all — so
// the mode guard test passes vacuously (no debounce is armed whether the mode guard
// exists or not). After BLOCKER-001 is fixed, the mode guard becomes testable.
//
// To make this a proper Red Gate test, we use a combined approach:
//   1. Arm the debounce manually (simulating a prior resize detection).
//   2. Call handle_crossterm_event with Event::Resize while in Dashboard mode.
//   3. Assert the debounce was NOT re-armed for the Dashboard session.
//
// After BLOCKER-001 is fixed AND mode guard is added: the test verifies that
// Dashboard mode suppresses resize detection at the call site.
//
// LOW-001 current Red Gate: When in EmbeddedTerminal and handle_crossterm_event
// handles Resize (after BLOCKER-001 fix), the mode-guard in the Resize handler
// must check AppMode::EmbeddedTerminal before calling on_resize_detected.
// Without the mode guard, calling Event::Resize in Dashboard would arm the debounce.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop
///
/// LOW-001 / BC-2.09.006 EC-236:
///   When `AppMode::Dashboard` is active and `handle_crossterm_event` processes
///   `Event::Resize(100, 30)`, `on_resize_detected` must NOT be called — the
///   debounce deadline must remain `None`.
///
///   This test uses a two-step proof to verify the mode guard is at the call site
///   (not vacuously satisfied by the Resize handler being absent):
///
///   Step 1 (BLOCKER-001 guard): First verify that in EmbeddedTerminal mode,
///   `Event::Resize` DOES arm the debounce (the wiring exists).
///
///   Step 2 (LOW-001 guard): Then verify that in Dashboard mode, the SAME event
///   does NOT arm the debounce (the mode guard suppresses it).
///
///   RED GATE: Step 1 fails because `handle_crossterm_event` does not handle
///   `Event::Resize` at all — the EmbeddedTerminal assert fails, proving that
///   Step 2's pass is vacuous and the mode guard has not been tested yet.
///   Once BLOCKER-001 is fixed (Step 1 passes), Step 2 verifies the mode guard.
#[tokio::test]
async fn test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop() {
    let layers = build_builtin_binding_layers();

    // --- Step 1: EmbeddedTerminal mode MUST arm the debounce (BLOCKER-001 sanity check) ---
    // Without this, the Dashboard assertion below is vacuous.
    {
        let (mut app_et, _rx) = make_app_in_embedded(SESSION_A, 24, 80);
        let mut sessions_state = SessionsPanelState::default();

        monocle_tui::app::handle_crossterm_event(
            &mut app_et,
            Event::Resize(100, 30),
            &layers,
            &mut sessions_state,
        )
        .await
        .ok();

        // RED GATE: fails because handle_crossterm_event ignores Event::Resize.
        // Once BLOCKER-001 is fixed, this assertion passes and Step 2 becomes meaningful.
        assert!(
            app_et.resize_debounce_deadline.is_some(),
            "LOW-001 Step 1 (BLOCKER-001 sanity): Event::Resize in EmbeddedTerminal \
             must arm the debounce deadline — got None. The mode guard test (Step 2) \
             is vacuous until this wiring is added to handle_crossterm_event."
        );
    }

    // --- Step 2: Dashboard mode must NOT arm the debounce (mode guard) ---
    {
        let mut app_db = make_app_in_dashboard();
        let mut sessions_state = SessionsPanelState::default();

        monocle_tui::app::handle_crossterm_event(
            &mut app_db,
            Event::Resize(100, 30),
            &layers,
            &mut sessions_state,
        )
        .await
        .ok();

        assert!(
            app_db.resize_debounce_deadline.is_none(),
            "LOW-001 / BC-2.09.006 EC-236: Event::Resize in Dashboard mode must NOT arm \
             the debounce deadline — on_resize_detected must be guarded by mode check"
        );
    }
}

// ---------------------------------------------------------------------------
// HIGH-001 — exit_embedded_terminal must call clear_resize_debounce_state
//
// When exit_embedded_terminal() is called, any pending debounce state must be
// cleared so that the next EmbeddedTerminal session starts clean.
//
// RED GATE: exit_embedded_terminal() does NOT call clear_resize_debounce_state.
// The debounce deadline and last_sent_size remain Some after exit. The assertions
// fail because they expect None.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state
///
/// HIGH-001 / BC-2.09.006 Invariant 1 (exit cleanup):
///   When `exit_embedded_terminal(app, session_id)` is called with an armed debounce
///   deadline and a non-None `last_sent_size`, both fields must be cleared to `None`.
///   This ensures the next `enter_embedded_terminal` starts with clean resize state.
///
///   RED GATE: `exit_embedded_terminal` does NOT call `clear_resize_debounce_state`.
///   After `exit_embedded_terminal`, `resize_debounce_deadline` and `last_sent_size`
///   remain `Some(_)` — the assertions on `is_none()` fail.
#[tokio::test]
async fn test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state() {
    let (mut app, _rx) = make_app_in_embedded(SESSION_A, 24, 80);

    // Arm the debounce state: simulate a resize detection that set the deadline
    // and a previous sent size.
    on_resize_detected(&mut app, SESSION_A, 30, 100);

    // Manually set last_sent_size to simulate a previously-sent resize.
    app.last_sent_size = Some((30, 100));

    // Verify pre-condition: state is armed.
    assert!(
        app.resize_debounce_deadline.is_some(),
        "HIGH-001 pre-condition: resize_debounce_deadline must be Some(_) after on_resize_detected"
    );
    assert!(
        app.last_sent_size.is_some(),
        "HIGH-001 pre-condition: last_sent_size must be Some(_) after manual assignment"
    );

    // Act: call the real exit_embedded_terminal production path.
    exit_embedded_terminal(&mut app, SESSION_A);

    // Assert: both resize state fields are cleared (None).
    //
    // RED GATE: fails because exit_embedded_terminal does NOT call
    // clear_resize_debounce_state(). The deadline and last_sent_size remain Some.
    assert!(
        app.resize_debounce_deadline.is_none(),
        "HIGH-001 / BC-2.09.006: exit_embedded_terminal must clear resize_debounce_deadline \
         to None — got Some(_). exit_embedded_terminal does not call \
         clear_resize_debounce_state()."
    );
    assert!(
        app.last_sent_size.is_none(),
        "HIGH-001 / BC-2.09.006: exit_embedded_terminal must clear last_sent_size to None — \
         got Some({:?}). exit_embedded_terminal does not call clear_resize_debounce_state().",
        app.last_sent_size
    );

    // Also verify the mode was correctly restored to Dashboard.
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "HIGH-001 post-condition: exit_embedded_terminal must restore AppMode::Dashboard"
    );
}
