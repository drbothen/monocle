//! TDD test suite for BC-2.09.003: Mouse dispatch wiring, conversion boundary,
//! and scoped mouse capture lifecycle tests.
//!
//! # Coverage mapping
//!
//! | Test name | AC / BC clause |
//! |-----------|----------------|
//! | test_BC_2_09_003_crossterm_mouse_to_pty_left_down      | AC-007 (conversion boundary) |
//! | test_BC_2_09_003_crossterm_mouse_to_pty_right_release  | AC-007 (conversion boundary) |
//! | test_BC_2_09_003_crossterm_mouse_to_pty_scroll_up      | AC-007 (conversion boundary) |
//! | test_BC_2_09_003_crossterm_mouse_to_pty_drag           | AC-007 (conversion boundary) |
//! | test_BC_2_09_003_crossterm_mouse_to_pty_ctrl_modifier  | AC-007 (modifier preservation) |
//! | test_BC_2_09_003_ratatui_rect_to_pty_fields_copied     | AC-007 (rect conversion) |
//! | test_BC_2_09_003_mouse_dispatch_forwards_keyinput      | AC-003/AC-005/AC-007 (dispatch) |
//! | test_BC_2_09_003_mouse_dispatch_out_of_pane_no_ipc     | AC-006/EC-221 (dispatch) |
//! | test_BC_2_09_003_scoped_mouse_capture_lifecycle        | AC-001/AC-002/Invariant 1 |
//!
//! # Dispatch tests design (BLOCKER-001 lesson from S-043)
//!
//! The dispatch tests drive the SAME run-loop entry point (`handle_crossterm_event`)
//! used in bc_2_09_wiring_tests.rs — NOT by calling `mouse_event_to_pty_bytes` directly.
//! Calling the pure function directly would be a tautology test (it tests the helper,
//! not the wiring). The dispatch test proves the entire path:
//!   crossterm Event::Mouse → handle_crossterm_event → dispatch_embedded_terminal_mouse
//!   → crossterm_mouse_to_pty (todo!) → mouse_event_to_pty_bytes (todo!) → KeyInput IPC.
//!
//! The test sets `app.last_pty_pane_area` (required since S-039 added this field)
//! so the production path can find the pane area. Without it, handle_crossterm_event
//! returns early with a TRACE log (not an error) and the event is dropped — this
//! would make the dispatch test a false negative. Setting it explicitly is correct
//! (mirroring how the render loop populates it in production).
//!
//! # Lifecycle test
//!
//! `scoped_mouse_capture_enter()` and `scoped_mouse_capture_exit()` are private
//! to `app.rs`. The lifecycle test asserts their observable effects through the
//! public `enter_embedded_terminal()` and `exit_embedded_terminal()` API.
//!
//! Since both functions write to a real stdout (terminal device I/O), capturing the
//! exact byte output in a unit test is not feasible without a mock terminal backend
//! — crossterm writes directly to `stdout()`, not through a trait object.
//! The approach used here:
//!   - Assert the function transitions mode (enter) or restores mode (exit) correctly.
//!   - Assert the function does NOT panic (the todo!() stubs panic: Red Gate confirmed).
//!   - Document this as a "lifecycle seam test": the structural invariant (mode transitions
//!     happen IFF scoped capture succeeds) is verified through the production API.
//!   - A TODO marks the terminal-write verification as a future enhancement gated on a
//!     mock-terminal seam (if the project adds one for S-044 or later).
//!
//! # Red Gate
//!
//! All dispatch and lifecycle tests fail against the current stubs:
//! - `crossterm_mouse_to_pty` is `todo!()` — panics on any mouse dispatch
//! - `ratatui_rect_to_pty` is `todo!()` — panics on any mouse dispatch
//! - `scoped_mouse_capture_enter` is `todo!()` — panics on `enter_embedded_terminal`
//! - `scoped_mouse_capture_exit` is `todo!()` — panics on `exit_embedded_terminal`
//!
//! The conversion boundary tests (crossterm_mouse_to_pty, ratatui_rect_to_pty) also
//! fail because the functions are `todo!()`.
//!
//! # No version-pin literals
//!
//! This file contains NO dependency version strings per POL-11.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use monocle_config::MonocleConfig;
use monocle_core::keyboard::{PtyKeyModifiers, PtyMouseButton, PtyMouseEventKind};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::{ClientToServer, PermissionPromptPayload};
use monocle_tui::app::{
    on_permission_prompt_queued, on_permission_prompt_resolved, on_transport_event, TransportEvent,
};
use monocle_tui::keyboard_conv::{crossterm_mouse_to_pty, ratatui_rect_to_pty};
use monocle_tui::{build_builtin_binding_layers, handle_crossterm_event, App};
use ratatui::layout::Rect;
use tokio::sync::mpsc;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build an `App` in `EmbeddedTerminal` mode with a wired `ipc_tx` and a known
/// `last_pty_pane_area`. Returns `(app, ipc_rx)`.
///
/// `last_pty_pane_area` is required for the mouse dispatch path:
/// `handle_crossterm_event` drops mouse events when `last_pty_pane_area == None`
/// (transient condition on first tick). Setting it explicitly bypasses that guard.
fn make_app_embedded_with_pane(
    session_id: &str,
    pane: Rect,
) -> (App, mpsc::Receiver<ClientToServer>) {
    let mut app = App::new(MonocleConfig::default());
    let (tx, rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_owned(),
        prior: FocusSnapshot::Sessions,
    };
    app.last_pty_pane_area = Some(pane);
    (app, rx)
}

/// Drain all messages from an mpsc receiver without blocking.
fn drain_channel(rx: &mut mpsc::Receiver<ClientToServer>) -> Vec<ClientToServer> {
    let mut out = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        out.push(msg);
    }
    out
}

/// Build a crossterm `MouseEvent` with the given kind and coordinates.
/// Uses `crossterm::event::KeyModifiers::NONE` for modifiers.
fn make_mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: crossterm::event::KeyModifiers::NONE,
    }
}

/// Build a crossterm `MouseEvent` with CONTROL modifier.
fn make_mouse_event_ctrl(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: crossterm::event::KeyModifiers::CONTROL,
    }
}

// ---------------------------------------------------------------------------
// Conversion boundary tests — crossterm_mouse_to_pty (AC-007)
//
// These verify that the conversion seam in keyboard_conv.rs correctly maps
// crossterm types to core-owned Pty* types. They do NOT test the encoding —
// just that the type transformation is correct and the modifier bits are
// preserved. The pure function tests handle encoding.
// ---------------------------------------------------------------------------

/// AC-007 / BC-2.09.003 Invariant 2 — Down(Left) crossterm → PtyMouseEventKind::Down(Left)
///
/// crossterm::MouseEventKind::Down(MouseButton::Left) must produce
/// PtyMouseEventKind::Down(PtyMouseButton::Left) with the correct col/row fields.
#[test]
fn test_BC_2_09_003_crossterm_mouse_to_pty_left_down() {
    let ct_event = make_mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 5);
    let pty_event = crossterm_mouse_to_pty(ct_event);
    assert_eq!(
        pty_event.kind,
        PtyMouseEventKind::Down(PtyMouseButton::Left),
        "Down(Left) must convert to PtyMouseEventKind::Down(PtyMouseButton::Left)"
    );
    assert_eq!(pty_event.column, 10, "column must be preserved");
    assert_eq!(pty_event.row, 5, "row must be preserved");
    assert_eq!(
        pty_event.modifiers,
        PtyKeyModifiers::NONE,
        "modifiers must be NONE when no crossterm modifiers"
    );
}

/// AC-007 — Up(Right) crossterm → PtyMouseEventKind::Up(Right)
#[test]
fn test_BC_2_09_003_crossterm_mouse_to_pty_right_release() {
    let ct_event = make_mouse_event(MouseEventKind::Up(MouseButton::Right), 3, 7);
    let pty_event = crossterm_mouse_to_pty(ct_event);
    assert_eq!(
        pty_event.kind,
        PtyMouseEventKind::Up(PtyMouseButton::Right),
        "Up(Right) must convert to PtyMouseEventKind::Up(PtyMouseButton::Right)"
    );
    assert_eq!(pty_event.column, 3, "column must be preserved");
    assert_eq!(pty_event.row, 7, "row must be preserved");
}

/// AC-007 — ScrollUp crossterm → PtyMouseEventKind::ScrollUp
#[test]
fn test_BC_2_09_003_crossterm_mouse_to_pty_scroll_up() {
    let ct_event = make_mouse_event(MouseEventKind::ScrollUp, 20, 10);
    let pty_event = crossterm_mouse_to_pty(ct_event);
    assert_eq!(
        pty_event.kind,
        PtyMouseEventKind::ScrollUp,
        "MouseEventKind::ScrollUp must convert to PtyMouseEventKind::ScrollUp"
    );
    assert_eq!(pty_event.column, 20, "column must be preserved");
    assert_eq!(pty_event.row, 10, "row must be preserved");
}

/// AC-007 — Drag(Left) crossterm → PtyMouseEventKind::Drag(Left)
#[test]
fn test_BC_2_09_003_crossterm_mouse_to_pty_drag() {
    let ct_event = make_mouse_event(MouseEventKind::Drag(MouseButton::Left), 5, 2);
    let pty_event = crossterm_mouse_to_pty(ct_event);
    assert_eq!(
        pty_event.kind,
        PtyMouseEventKind::Drag(PtyMouseButton::Left),
        "Drag(Left) must convert to PtyMouseEventKind::Drag(PtyMouseButton::Left)"
    );
}

/// AC-007 — CONTROL modifier is preserved through the conversion
///
/// crossterm CONTROL bit → PtyKeyModifiers::CONTROL
/// This verifies that the modifier remapping in crossterm_mouse_to_pty is correct.
#[test]
fn test_BC_2_09_003_crossterm_mouse_to_pty_ctrl_modifier() {
    let ct_event = make_mouse_event_ctrl(MouseEventKind::Down(MouseButton::Left), 0, 0);
    let pty_event = crossterm_mouse_to_pty(ct_event);
    assert!(
        pty_event.modifiers.contains(PtyKeyModifiers::CONTROL),
        "CONTROL modifier from crossterm must map to PtyKeyModifiers::CONTROL"
    );
}

// ---------------------------------------------------------------------------
// Conversion boundary tests — ratatui_rect_to_pty (AC-007)
// ---------------------------------------------------------------------------

/// AC-007 — ratatui::layout::Rect fields are copied 1:1 to PtyRect
///
/// x, y, width, height must all be preserved exactly.
#[test]
fn test_BC_2_09_003_ratatui_rect_to_pty_fields_copied() {
    let ratatui_rect = Rect {
        x: 10,
        y: 5,
        width: 80,
        height: 24,
    };
    let pty_rect = ratatui_rect_to_pty(ratatui_rect);
    assert_eq!(pty_rect.x, 10, "x must be copied");
    assert_eq!(pty_rect.y, 5, "y must be copied");
    assert_eq!(pty_rect.width, 80, "width must be copied");
    assert_eq!(pty_rect.height, 24, "height must be copied");
}

// ---------------------------------------------------------------------------
// Dispatch path tests — CRITICAL (S-043 dead-trigger BLOCKER lesson applied)
//
// These tests drive handle_crossterm_event (the same run-loop entry point used
// in bc_2_09_wiring_tests.rs), NOT the helpers directly. This ensures the entire
// wiring path is exercised: Event::Mouse arm → dispatch_embedded_terminal_mouse
// → crossterm_mouse_to_pty → ratatui_rect_to_pty → mouse_event_to_pty_bytes
// → KeyInput IPC send.
// ---------------------------------------------------------------------------

/// AC-003 / AC-005 / AC-007 / BC-2.09.003 PC-1/PC-3 — Mouse event in EmbeddedTerminal
/// sends KeyInput over IPC with the correct SGR-encoded bytes.
///
/// Drives handle_crossterm_event with a Left-button press INSIDE the pane.
/// Expects exactly one KeyInput with the SGR-encoded bytes for that coordinate.
///
/// Pane: x=0, y=0, width=80, height=24 (stored in app.last_pty_pane_area)
/// Event: Down(Left) at (column=5, row=3)
/// Expected bytes: `\x1b[<0;6;4M` (Px=6, Py=4 — 1-indexed, pane at origin)
#[tokio::test]
async fn test_BC_2_09_003_mouse_dispatch_forwards_keyinput() {
    // Pane at origin 80×24 — same as the canonical test vector in BC-2.09.003.
    let pane = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    };
    let (mut app, mut rx) = make_app_embedded_with_pane("session-mouse-001", pane);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = Default::default();

    // Construct a crossterm Event::Mouse(Down(Left)) at (column=5, row=3).
    let mouse_event = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 3,
        modifiers: KeyModifiers::NONE,
    });

    let result = handle_crossterm_event(&mut app, mouse_event, &layers, &mut sessions_state).await;

    assert!(
        result.is_ok(),
        "handle_crossterm_event must not return Err for a mouse event"
    );

    // Expect exactly one KeyInput with SGR bytes for Down(Left) at (5,3) in 80x24@origin.
    // Expected: \x1b[<0;6;4M (Px=5+1=6, Py=3+1=4, base_Ps=0 for Left, terminator M)
    let msgs = drain_channel(&mut rx);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one KeyInput must be sent for a mouse event inside pane; got {}",
        msgs.len()
    );
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            assert_eq!(
                bytes.as_slice(),
                b"\x1b[<0;6;4M",
                "Down(Left) at (5,3) in 80x24@origin must encode as \\x1b[<0;6;4M"
            );
            assert_eq!(
                session_id, "session-mouse-001",
                "session_id must match the focused session"
            );
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }

    // Mode must remain EmbeddedTerminal — mouse events do not change the mode.
    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "mode must remain EmbeddedTerminal after mouse event (not a navigation event)"
    );
}

/// AC-006 / EC-221 / BC-2.09.003 PC-5 — Mouse event OUTSIDE the pane sends no IPC
///
/// Drives handle_crossterm_event with a click outside the pane area.
/// Expects NO KeyInput sent (mouse_event_to_pty_bytes returns None for out-of-pane events).
///
/// Pane: x=0, y=0, width=80, height=24
/// Event: Down(Left) at (column=200, row=200) — well outside the pane
#[tokio::test]
async fn test_BC_2_09_003_mouse_dispatch_out_of_pane_no_ipc() {
    let pane = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    };
    let (mut app, mut rx) = make_app_embedded_with_pane("session-mouse-002", pane);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = Default::default();

    // Click at (200, 200) — far outside 80x24 pane.
    let mouse_event = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 200,
        row: 200,
        modifiers: KeyModifiers::NONE,
    });

    let result = handle_crossterm_event(&mut app, mouse_event, &layers, &mut sessions_state).await;

    assert!(
        result.is_ok(),
        "out-of-pane mouse event must not return Err"
    );

    let msgs = drain_channel(&mut rx);
    assert!(
        msgs.is_empty(),
        "out-of-pane click must NOT send any KeyInput; got {} message(s)",
        msgs.len()
    );
}

/// AC-003 / BC-2.09.003 PC-2 — ScrollUp event dispatched via run-loop produces correct bytes
///
/// Scroll events are forwarded just like button events. This exercises the Ps=64 path
/// through the full dispatch chain.
///
/// Pane: x=0, y=0, width=80, height=24
/// Event: ScrollUp at (column=10, row=5)
/// Expected: `\x1b[<64;11;6M` (Px=10+1=11, Py=5+1=6, base_Ps=64)
#[tokio::test]
async fn test_BC_2_09_003_mouse_dispatch_scroll_up_forwarded() {
    let pane = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    };
    let (mut app, mut rx) = make_app_embedded_with_pane("session-scroll-001", pane);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = Default::default();

    let mouse_event = Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    });

    handle_crossterm_event(&mut app, mouse_event, &layers, &mut sessions_state)
        .await
        .expect("ScrollUp dispatch must not return Err");

    let msgs = drain_channel(&mut rx);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one KeyInput must be sent for ScrollUp"
    );
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, .. } => {
            assert_eq!(
                bytes.as_slice(),
                b"\x1b[<64;11;6M",
                "ScrollUp at (10,5) must encode as \\x1b[<64;11;6M (Ps=64)"
            );
        }
        other => panic!("expected KeyInput, got {:?}", other),
    }
}

/// BC-2.09.003 Invariant 2 — Mouse events do NOT trigger ExitEmbeddedTerminal
///
/// Only a bare Esc key exits EmbeddedTerminal. Mouse events — even if they are at
/// pane origin — must not change the AppMode.
#[tokio::test]
async fn test_BC_2_09_003_mouse_event_does_not_exit_embedded_terminal() {
    let pane = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    };
    let (mut app, _rx) = make_app_embedded_with_pane("session-no-exit", pane);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = Default::default();

    // ScrollUp in pane — should NOT change the mode.
    let mouse_event = Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 5,
        row: 5,
        modifiers: KeyModifiers::NONE,
    });

    handle_crossterm_event(&mut app, mouse_event, &layers, &mut sessions_state)
        .await
        .expect("mouse event must not Err");

    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "AppMode must remain EmbeddedTerminal after mouse event"
    );
}

// ---------------------------------------------------------------------------
// Lifecycle tests — AC-001 / AC-002 / BC-2.09.003 Invariant 1
//
// scoped_mouse_capture_enter() and scoped_mouse_capture_exit() are private.
// We test through the public enter_embedded_terminal / exit_embedded_terminal API.
//
// LIMITATION: These functions write to a real stdout (crossterm::execute! and
// print!) — the exact bytes written cannot be captured in a unit test without a
// mock terminal seam. The tests below assert the observable STRUCTURAL effects:
// - enter_embedded_terminal transitions AppMode to EmbeddedTerminal (requires
//   scoped_mouse_capture_enter to complete without panic).
// - exit_embedded_terminal restores the prior mode (requires scoped_mouse_capture_exit
//   to complete without panic).
// Both functions currently todo!() panic → Red Gate confirmed.
//
// Terminal-write capture (EnableMouseCapture + `\x1b[?1006h` ordering) would require
// a mock stdout seam or a PTY loopback fixture, which is out of scope for S-041 test
// writing. If such a seam is added in a future story, the ordering assertion should be:
//   1. first byte sequence: crossterm EnableMouseCapture (CSI ? 1000h)
//   2. second byte sequence: `\x1b[?1006h`  (SGR extended mode)
//   and on exit (reversed):
//   1. first byte sequence: `\x1b[?1006l`
//   2. second byte sequence: crossterm DisableMouseCapture (CSI ? 1000l)
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.09.003 Invariant 1 — enter_embedded_terminal enables scoped mouse capture
///
/// Structural assertion: calling `enter_embedded_terminal` completes without panic
/// AND transitions AppMode to EmbeddedTerminal. The scoped_mouse_capture_enter()
/// call inside is the thing that currently panics (todo!() stub) — confirming Red Gate.
///
/// # Limitation
///
/// The `\x1b[?1006h` write to stdout and EnableMouseCapture ordering cannot be
/// asserted without a mock terminal seam. This test confirms the lifecycle call
/// completes and produces the correct mode transition.
#[tokio::test]
async fn test_BC_2_09_003_scoped_mouse_capture_lifecycle_enter_transitions_mode() {
    use monocle_tui::app::enter_embedded_terminal;

    let mut app = App::new(MonocleConfig::default());
    // Wire a real IPC sender so enter_embedded_terminal doesn't return early (no-IPC path).
    let (tx, _rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    // Pre-insert session into pty_dump_received so the enter path skips the
    // AttachSession protocol (faster test; we're testing lifecycle, not IPC).
    app.pty_dump_received
        .insert("session-lifecycle-001".to_string());

    // This call must NOT panic once scoped_mouse_capture_enter() is implemented.
    // Currently it WILL panic (todo!() stub) — that is the Red Gate.
    enter_embedded_terminal(&mut app, "session-lifecycle-001".to_string()).await;

    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { ref session_id, .. } if session_id == "session-lifecycle-001"),
        "enter_embedded_terminal must transition mode to EmbeddedTerminal"
    );
}

/// AC-002 / BC-2.09.003 Invariant 1 — exit_embedded_terminal disables scoped mouse capture
///
/// Structural assertion: calling `exit_embedded_terminal` completes without panic
/// AND restores the prior AppMode. The scoped_mouse_capture_exit() call inside is the
/// thing that currently panics (todo!() stub) — confirming Red Gate.
#[test]
fn test_BC_2_09_003_scoped_mouse_capture_lifecycle_exit_restores_mode() {
    use monocle_tui::app::exit_embedded_terminal;

    let mut app = App::new(MonocleConfig::default());
    // Manually set mode to EmbeddedTerminal (bypassing enter to isolate exit test).
    app.mode = AppMode::EmbeddedTerminal {
        session_id: "session-lifecycle-002".to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // This call must NOT panic once scoped_mouse_capture_exit() is implemented.
    // Currently it WILL panic (todo!() stub) — that is the Red Gate.
    exit_embedded_terminal(&mut app, "session-lifecycle-002");

    // After exit, mode must be restored to Dashboard (prior = FocusSnapshot::Sessions).
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "exit_embedded_terminal must restore AppMode to Dashboard"
    );
}

/// AC-001 / AC-002 / BC-2.09.003 Invariant 1 — lifecycle ordering: enter then exit restores mode
///
/// This test exercises the FULL round-trip:
///   enter_embedded_terminal → AppMode::EmbeddedTerminal
///   exit_embedded_terminal  → AppMode::Dashboard (prior restored)
///
/// The exit sequence order (SGR 1006l BEFORE DisableMouseCapture) is enforced by the
/// production implementation. A structural regression would be visible if the mode
/// is not correctly restored, or if any step panics.
#[tokio::test]
async fn test_BC_2_09_003_scoped_mouse_capture_lifecycle_full_roundtrip() {
    use monocle_tui::app::{enter_embedded_terminal, exit_embedded_terminal};

    let mut app = App::new(MonocleConfig::default());
    let (tx, _rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    app.pty_dump_received
        .insert("session-roundtrip".to_string());

    // Step 1: enter
    enter_embedded_terminal(&mut app, "session-roundtrip".to_string()).await;
    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { ref session_id, .. } if session_id == "session-roundtrip"),
        "after enter_embedded_terminal, mode must be EmbeddedTerminal"
    );

    // Step 2: exit
    exit_embedded_terminal(&mut app, "session-roundtrip");
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "after exit_embedded_terminal, mode must be restored to Dashboard"
    );
}

// ---------------------------------------------------------------------------
// Negative dispatch test — mouse event in non-EmbeddedTerminal mode is ignored
// ---------------------------------------------------------------------------

/// BC-2.09.003 Precondition 1 — Mouse event in Dashboard mode (not EmbeddedTerminal)
/// is silently ignored; no KeyInput sent.
///
/// Mouse capture is NOT active outside EmbeddedTerminal (CC-GLOBAL-MOUSE-CAPTURE
/// invariant), so mouse events arriving in Dashboard mode must be dropped.
#[tokio::test]
async fn test_BC_2_09_003_mouse_event_in_dashboard_mode_no_ipc() {
    let mut app = App::new(MonocleConfig::default());
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    // mode is Dashboard by default

    let layers = build_builtin_binding_layers();
    let mut sessions_state = Default::default();

    let mouse_event = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 5,
        modifiers: KeyModifiers::NONE,
    });

    let result = handle_crossterm_event(&mut app, mouse_event, &layers, &mut sessions_state).await;
    assert!(result.is_ok(), "mouse in Dashboard must not return Err");

    let msgs = drain_channel(&mut rx);
    assert!(
        msgs.is_empty(),
        "mouse event in Dashboard mode must NOT produce any KeyInput; got {} messages",
        msgs.len()
    );
}

// ---------------------------------------------------------------------------
// Regression guard — key events in EmbeddedTerminal still work after mouse stub added
// (ensures the Mouse arm addition did not break the Key arm)
// ---------------------------------------------------------------------------

/// Regression: Key events in EmbeddedTerminal still route to KeyInput after the
/// Event::Mouse arm was added. Exercises the Key arm to confirm no breakage.
///
/// This is a smoke test — the full key forwarding suite is in bc_2_09_wiring_tests.rs.
#[tokio::test]
async fn test_BC_2_09_003_key_forwarding_unaffected_by_mouse_arm() {
    let pane = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    };
    let (mut app, mut rx) = make_app_embedded_with_pane("session-key-smoke", pane);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = Default::default();

    let key_event = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char('z'),
        KeyModifiers::NONE,
        KeyEventKind::Press,
    ));

    handle_crossterm_event(&mut app, key_event, &layers, &mut sessions_state)
        .await
        .expect("key event must not Err");

    let msgs = drain_channel(&mut rx);
    assert_eq!(msgs.len(), 1, "Key 'z' must produce exactly one KeyInput");
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, .. } => {
            assert_eq!(
                bytes.as_slice(),
                b"z",
                "Char('z') must produce bytes [0x7a]"
            );
        }
        other => panic!("expected KeyInput for 'z', got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Regression guards: scoped mouse-capture symmetry invariant
// (BC-2.09.003 Invariant 1 / F-S041-P7-HIGH-001)
//
// These four tests provide positive-coverage assertions for BC-2.09.003 Invariant 1:
// mouse capture is SCOPED to AppMode::EmbeddedTerminal — it MUST be torn down on
// EVERY departure path from EmbeddedTerminal, including the EmbeddedTerminal→Overlay
// transition that previously lacked coverage (the structural gap flagged by
// Adversarial Pass-7 finding F-S041-P7-HIGH-001).
//
// All tests PASS against the fix at f35e08a (App::mouse_capture_active observable
// added; enter_embedded_terminal sets it true; every EmbeddedTerminal departure sets
// it false). Test #2 is THE KEY REGRESSION: it exercises the EmbeddedTerminal→Overlay
// path (on_permission_prompt_queued) and would have FAILED before the fix because
// mouse_capture_active was never cleared on that path.
//
// # Driving strategy
//
// Tests drive transition through REAL handler entry points (not field mutation):
//   - enter_embedded_terminal  — real async fn (run through pre-seeded pty_dump_received
//     so the fast-path is used; avoids the .send().await AttachSession await in tests).
//   - on_permission_prompt_queued — real pub free function (the IPC dispatch path).
//   - on_permission_prompt_resolved — real pub free function.
//   - exit_embedded_terminal  — real pub free function (Esc / normal exit path).
//   - on_transport_event(Disconnected) — real pub free function (disconnect path).
//
// # No version-pin literals (POL-11)
// ---------------------------------------------------------------------------

/// Helper: Build an App already in EmbeddedTerminal with mouse_capture_active set
/// correctly as the real enter_embedded_terminal would do it.
///
/// Uses the pty_dump_received fast path (no .send().await) so the test does not
/// require a running tokio runtime IPC channel and does not depend on AttachSession.
/// mouse_capture_active is set true by enter_embedded_terminal after scoped_mouse_capture_enter().
async fn make_app_embedded_capture_active(
    session_id: &str,
) -> (App, mpsc::Receiver<ClientToServer>) {
    use monocle_tui::enter_embedded_terminal;

    let mut app = App::new(MonocleConfig::default());
    let (tx, rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    // Pre-seed pty_dump_received so enter_embedded_terminal takes the O(1) fast path
    // (no AttachSession send, no dump_in_progress change). This isolates the test to
    // the mode transition and scoped_mouse_capture_enter() call.
    app.pty_dump_received.insert(session_id.to_string());
    enter_embedded_terminal(&mut app, session_id.to_string()).await;
    (app, rx)
}

/// Build a PermissionPromptPayload for the given session.
///
/// Helper keeps construction in one place so session_id / UUIDs are consistent
/// across the three tests that inject a permission prompt.
fn make_permission_prompt(session_id: &str) -> PermissionPromptPayload {
    PermissionPromptPayload {
        prompt_id: Uuid::new_v4(),
        session_id: session_id.to_string(),
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({"command": "echo regression-guard"}),
        old_content: None,
        new_content: None,
    }
}

/// BC-2.09.003 Invariant 1 / AC-001 — enter_embedded_terminal sets mouse_capture_active true.
///
/// Pre-condition: App starts in Dashboard (mouse_capture_active == false).
/// Post-condition: After enter_embedded_terminal, mouse_capture_active == true
/// AND mode == EmbeddedTerminal.
///
/// This is the positive baseline: confirms the observable is armed on entry.
/// Would fail if enter_embedded_terminal forgot to set mouse_capture_active = true.
#[tokio::test]
async fn test_BC_2_09_003_mouse_capture_active_in_embedded_terminal() {
    let session_id = "00000002-0001-4000-8001-000000000001";
    let (app, _rx) = make_app_embedded_capture_active(session_id).await;

    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "post-condition: mode must be EmbeddedTerminal after enter_embedded_terminal"
    );
    assert!(
        app.mouse_capture_active,
        "BC-2.09.003 Invariant 1: mouse_capture_active must be true after \
         enter_embedded_terminal — capture must be active while in EmbeddedTerminal"
    );
}

/// BC-2.09.003 Invariant 1 / F-S041-P7-HIGH-001 — THE KEY REGRESSION.
///
/// Permission prompt arriving over IPC while in EmbeddedTerminal transitions
/// the mode to AppMode::Overlay via on_permission_prompt_queued. This path does NOT
/// go through exit_embedded_terminal. Before the fix at f35e08a, on_permission_prompt_queued
/// did not clear mouse_capture_active, so scoped mouse capture leaked into Overlay mode.
///
/// This test asserts that after on_permission_prompt_queued:
/// - mode == AppMode::Overlay (transition happened)
/// - mouse_capture_active == false (capture was torn down on departure from EmbeddedTerminal)
///
/// Would have FAILED before f35e08a (mouse_capture_active remained true).
#[tokio::test]
async fn test_BC_2_09_003_mouse_capture_torn_down_on_permission_prompt_overlay() {
    let session_id = "00000002-0001-4000-8001-000000000002";
    let (mut app, _rx) = make_app_embedded_capture_active(session_id).await;

    // Pre-condition: capture is active in EmbeddedTerminal.
    assert!(
        app.mouse_capture_active,
        "pre-condition: mouse_capture_active must be true before permission prompt injection"
    );
    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "pre-condition: mode must be EmbeddedTerminal before permission prompt injection"
    );

    // Drive the real IPC path: on_permission_prompt_queued transitions
    // EmbeddedTerminal → AppMode::Overlay without going through exit_embedded_terminal.
    let payload = make_permission_prompt(session_id);
    on_permission_prompt_queued(&mut app, payload);

    // Post-condition 1: mode transitioned to Overlay.
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "on_permission_prompt_queued must transition mode to AppMode::Overlay \
         (EmbeddedTerminal→Overlay path)"
    );
    // Post-condition 2 — THE REGRESSION ASSERTION:
    // mouse_capture_active must be false after EmbeddedTerminal→Overlay transition.
    // Before f35e08a: on_permission_prompt_queued never set mouse_capture_active = false,
    // so this assertion would fail with "expected false, got true".
    assert!(
        !app.mouse_capture_active,
        "F-S041-P7-HIGH-001 / BC-2.09.003 Invariant 1: mouse_capture_active must be false \
         after EmbeddedTerminal→Overlay transition via on_permission_prompt_queued — \
         scoped capture must not leak into Overlay mode"
    );
}

/// BC-2.09.003 Invariant 1 — mouse_capture_active stays false after Overlay→Dashboard.
///
/// Continues from the EmbeddedTerminal→Overlay state reached in test #2:
/// resolves the permission prompt (stack empties) → mode collapses to Dashboard.
/// mouse_capture_active must remain false (not re-enabled on Overlay→Dashboard).
///
/// This guards against a hypothetical re-arming of capture on the return path.
#[tokio::test]
async fn test_BC_2_09_003_mouse_capture_off_after_permission_resolve_to_dashboard() {
    let session_id = "00000002-0001-4000-8001-000000000003";
    let (mut app, _rx) = make_app_embedded_capture_active(session_id).await;

    // Step 1: inject permission prompt (EmbeddedTerminal → Overlay).
    let payload = make_permission_prompt(session_id);
    let prompt_id = payload.prompt_id;
    on_permission_prompt_queued(&mut app, payload);

    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "step 1 pre-condition: mode must be Overlay after on_permission_prompt_queued"
    );
    assert!(
        !app.mouse_capture_active,
        "step 1 pre-condition: mouse_capture_active must be false in Overlay mode"
    );

    // Step 2: resolve the permission prompt (Overlay → Dashboard).
    on_permission_prompt_resolved(&mut app, prompt_id);

    // Post-condition 1: mode collapsed back to Dashboard.
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "on_permission_prompt_resolved with empty stack must collapse to Dashboard"
    );
    // Post-condition 2: capture remains off (not re-enabled on Overlay→Dashboard).
    assert!(
        !app.mouse_capture_active,
        "BC-2.09.003 Invariant 1: mouse_capture_active must remain false after \
         Overlay→Dashboard collapse — capture must NOT be re-enabled on permission resolve"
    );
}

/// BC-2.09.003 Invariant 1 / AC-002 — exit_embedded_terminal (Esc / normal exit) tears down capture.
///
/// After enter_embedded_terminal, calling exit_embedded_terminal (the normal Esc path)
/// must set mouse_capture_active = false. Also verifies the mode is restored to Dashboard.
///
/// Also exercises the transport-disconnect path: on_transport_event(Disconnected) while
/// in EmbeddedTerminal calls exit_embedded_terminal internally, so mouse_capture_active
/// must be false after disconnect.
#[tokio::test]
async fn test_BC_2_09_003_mouse_capture_torn_down_on_normal_exit() {
    use monocle_tui::exit_embedded_terminal;

    let session_id = "00000002-0001-4000-8001-000000000004";
    let (mut app, _rx) = make_app_embedded_capture_active(session_id).await;

    // Pre-condition: capture is active in EmbeddedTerminal.
    assert!(
        app.mouse_capture_active,
        "pre-condition: mouse_capture_active must be true before exit"
    );

    // Normal exit path: Esc → exit_embedded_terminal (the Action::ExitEmbeddedTerminal handler).
    exit_embedded_terminal(&mut app, session_id);

    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "exit_embedded_terminal must restore mode to Dashboard"
    );
    assert!(
        !app.mouse_capture_active,
        "BC-2.09.003 Invariant 1 / AC-002: mouse_capture_active must be false after \
         exit_embedded_terminal — scoped capture must be torn down on normal Esc exit"
    );
}

/// BC-2.09.003 Invariant 1 — transport-disconnect tears down mouse capture.
///
/// on_transport_event(Disconnected) while in EmbeddedTerminal calls exit_embedded_terminal
/// internally, which calls scoped_mouse_capture_exit and sets mouse_capture_active = false.
/// Verifies that the disconnect path (IPC channel drop) also satisfies Invariant 1.
#[tokio::test]
async fn test_BC_2_09_003_mouse_capture_torn_down_on_transport_disconnect() {
    let session_id = "00000002-0001-4000-8001-000000000005";
    let (mut app, _rx) = make_app_embedded_capture_active(session_id).await;

    // Pre-condition: in EmbeddedTerminal with capture active.
    assert!(
        app.mouse_capture_active,
        "pre-condition: mouse_capture_active must be true before disconnect"
    );
    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "pre-condition: mode must be EmbeddedTerminal before disconnect"
    );

    // Drive the real transport-disconnect path.
    // on_transport_event(Disconnected) calls exit_embedded_terminal for EmbeddedTerminal mode,
    // which in turn calls scoped_mouse_capture_exit and sets mouse_capture_active = false.
    on_transport_event(&mut app, TransportEvent::Disconnected);

    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "on_transport_event(Disconnected) must restore mode to Dashboard"
    );
    assert!(
        !app.mouse_capture_active,
        "BC-2.09.003 Invariant 1: mouse_capture_active must be false after transport disconnect \
         while in EmbeddedTerminal — exit_embedded_terminal must tear down scoped capture"
    );
}
