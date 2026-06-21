//! Integration tests for S-040 wiring: `handle_crossterm_event` routes Key and Paste
//! events through the correct dispatch helpers when `AppMode::EmbeddedTerminal` is active.
//!
//! These tests verify the end-to-end wiring path required by F-S040-BLOCKER-001:
//!
//! 1. `Event::Key` in `EmbeddedTerminal` → `dispatch_embedded_terminal_key` → `KeyInput` IPC.
//! 2. `Event::Key(Esc)` in `EmbeddedTerminal` → `exit_embedded_terminal` (mode restored).
//! 3. `Event::Paste` in `EmbeddedTerminal` → bracketed paste → `KeyInput` IPC.
//! 4. `Event::Key` in non-EmbeddedTerminal → existing `dispatch_key_event` binding chain.
//! 5. `Event::Paste` in non-EmbeddedTerminal → silently ignored.
//! 6. `Event::Key(Esc)` ordering: Esc intercepted BEFORE PTY bytes (BC-2.09.002 Invariant 2).
//!
//! # Test design
//!
//! Each test constructs an `App`, sets `app.ipc_tx` to a real bounded mpsc sender,
//! sets `app.mode` directly to `AppMode::EmbeddedTerminal`, and drives
//! `handle_crossterm_event`. The test then drains the IPC receiver and asserts the
//! correct `ClientToServer` messages were sent (or not sent).
//!
//! No real UDS socket is needed — we bypass the network layer and test only the
//! event-routing logic.

#![allow(non_snake_case)]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::ClientToServer;
use monocle_tui::{build_builtin_binding_layers, handle_crossterm_event, App};
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build an `App` in `EmbeddedTerminal` mode with a wired `ipc_tx`.
/// Returns `(app, ipc_rx)` where `ipc_rx` receives `ClientToServer` messages.
fn make_app_embedded(session_id: &str) -> (App, mpsc::Receiver<ClientToServer>) {
    let mut app = App::new(MonocleConfig::default());
    let (tx, rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_owned(),
        prior: FocusSnapshot::Sessions,
    };
    (app, rx)
}

/// Build an `App` in Dashboard mode (default) with a wired `ipc_tx`.
fn make_app_dashboard() -> (App, mpsc::Receiver<ClientToServer>) {
    let mut app = App::new(MonocleConfig::default());
    let (tx, rx) = mpsc::channel::<ClientToServer>(32);
    app.ipc_tx = Some(tx);
    // mode is already Dashboard { focused: Sessions } by default
    (app, rx)
}

/// Build a crossterm `KeyEvent` with the given code, modifiers, and kind.
fn make_key_event(code: KeyCode, modifiers: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
    KeyEvent::new_with_kind(code, modifiers, kind)
}

/// Drain all messages from an mpsc receiver without blocking. Returns collected messages.
fn drain_channel(rx: &mut mpsc::Receiver<ClientToServer>) -> Vec<ClientToServer> {
    let mut out = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        out.push(msg);
    }
    out
}

/// Real builtin `BindingLayers` for tests.
/// In EmbeddedTerminal mode the binding layers are bypassed; in Dashboard mode
/// (quit test) they are used by `dispatch_key_event`.
fn empty_binding_layers() -> monocle_core::tui::binding::BindingLayers {
    build_builtin_binding_layers()
}

// ---------------------------------------------------------------------------
// F-S040-BLOCKER-001: EmbeddedTerminal Key arm routes through dispatch helper
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.09.002 PC-1 — Key event in EmbeddedTerminal sends KeyInput over IPC
///
/// 'a' key → dispatch_embedded_terminal_key → ClientToServer::KeyInput { bytes: [0x61] }
#[tokio::test]
async fn test_BC_2_09_002_embedded_terminal_key_routes_to_ipc() {
    let (mut app, mut rx) = make_app_embedded("session-001");
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let key_event = make_key_event(KeyCode::Char('a'), KeyModifiers::NONE, KeyEventKind::Press);
    let result = handle_crossterm_event(
        &mut app,
        Event::Key(key_event),
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(
        result.is_ok(),
        "handle_crossterm_event returned Err for non-Esc key"
    );

    let msgs = drain_channel(&mut rx);
    assert_eq!(msgs.len(), 1, "exactly one KeyInput should be sent for 'a'");
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            assert_eq!(bytes, &[0x61], "Char('a') → [0x61]");
            assert_eq!(session_id, "session-001");
        }
        other => panic!("expected KeyInput, got {:?}", other),
    }
    // Mode should remain EmbeddedTerminal (Esc was not pressed).
    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "mode must remain EmbeddedTerminal after non-Esc key"
    );
}

/// BC-2.09.002 Invariant 2 / AC-004 — bare Esc in EmbeddedTerminal exits mode, sends nothing
///
/// Esc (no modifiers) → dispatch_embedded_terminal_key returns true → exit_embedded_terminal
/// called → mode restored to Dashboard. No KeyInput sent.
#[tokio::test]
async fn test_BC_2_09_002_esc_exits_embedded_terminal_no_key_input_sent() {
    let (mut app, mut rx) = make_app_embedded("session-esc");
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let esc_event = make_key_event(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Press);
    let result = handle_crossterm_event(
        &mut app,
        Event::Key(esc_event),
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(
        result.is_ok(),
        "Esc must not propagate Err (not a quit event)"
    );

    // Zero bytes must be sent to the IPC channel.
    let msgs = drain_channel(&mut rx);
    assert!(
        msgs.is_empty(),
        "Esc must NOT forward bytes to PTY — {} KeyInput(s) sent unexpectedly",
        msgs.len()
    );

    // Mode must be restored to Dashboard (exit_embedded_terminal was called).
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "Esc in EmbeddedTerminal must restore Dashboard mode"
    );
}

/// BC-2.09.002 Invariant 2 ordering — Esc interception precedes PTY bytes
///
/// This test proves the Esc-intercept fires BEFORE key_event_to_pty_bytes by verifying
/// no bytes are forwarded even though Esc has a non-None byte representation (\x1b)
/// in key_event_to_pty_bytes. The dispatch helper must short-circuit before conversion.
#[tokio::test]
async fn test_BC_2_09_002_esc_intercept_precedes_pty_bytes() {
    // This is the same as the above test but framed explicitly as an ordering proof.
    // If the implementation called key_event_to_pty_bytes BEFORE the Esc check,
    // it would forward \x1b and return false — this test would catch both failure modes.
    let (mut app, mut rx) = make_app_embedded("session-ordering");
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let esc_event = make_key_event(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Press);
    let result = handle_crossterm_event(
        &mut app,
        Event::Key(esc_event),
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(result.is_ok());
    let msgs = drain_channel(&mut rx);
    assert!(
        msgs.is_empty(),
        "Esc-intercept ordering violation: {} byte(s) forwarded before intercept check",
        msgs.len()
    );
    assert!(matches!(app.mode, AppMode::Dashboard { .. }));
}

// ---------------------------------------------------------------------------
// F-S040-BLOCKER-001: EmbeddedTerminal Paste arm routes through dispatch helper
// ---------------------------------------------------------------------------

/// AC-009 / AC-010 / BC-2.09.005 PC-2 — Paste in EmbeddedTerminal wraps in bracket sequences
///
/// Event::Paste("hello") → dispatch_embedded_terminal_paste →
/// KeyInput { bytes: b"\x1b[200~hello\x1b[201~" }
#[tokio::test]
async fn test_BC_2_09_005_paste_in_embedded_terminal_sends_bracketed_key_input() {
    let (mut app, mut rx) = make_app_embedded("session-paste");
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let result = handle_crossterm_event(
        &mut app,
        Event::Paste("hello".to_owned()),
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(result.is_ok());

    let msgs = drain_channel(&mut rx);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one KeyInput should be sent for paste"
    );
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            let expected = b"\x1b[200~hello\x1b[201~";
            assert_eq!(
                bytes.as_slice(),
                expected,
                "bracketed paste wrapping mismatch"
            );
            assert_eq!(session_id, "session-paste");
        }
        other => panic!("expected KeyInput, got {:?}", other),
    }
}

/// BC-2.09.005 Invariant 3 — large paste (500 bytes) sent as single KeyInput, no fragmentation
#[tokio::test]
async fn test_BC_2_09_005_large_paste_single_key_input_no_fragmentation_via_handle() {
    let (mut app, mut rx) = make_app_embedded("session-largepaste");
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let text: String = "x".repeat(500);
    let result =
        handle_crossterm_event(&mut app, Event::Paste(text), &layers, &mut sessions_state).await;

    assert!(result.is_ok());

    let msgs = drain_channel(&mut rx);
    assert_eq!(
        msgs.len(),
        1,
        "500-byte paste must produce exactly 1 KeyInput"
    );
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, .. } => {
            assert_eq!(bytes.len(), 6 + 500 + 6, "bracketed paste length mismatch");
            assert!(
                bytes.starts_with(b"\x1b[200~"),
                "missing bracketed paste header"
            );
            assert!(
                bytes.ends_with(b"\x1b[201~"),
                "missing bracketed paste trailer"
            );
        }
        other => panic!("expected KeyInput, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// F-S040-BLOCKER-001: non-EmbeddedTerminal paths are not broken
// ---------------------------------------------------------------------------

/// Verify Paste in non-EmbeddedTerminal mode is silently ignored (no IPC send)
#[tokio::test]
async fn test_paste_ignored_outside_embedded_terminal() {
    let (mut app, mut rx) = make_app_dashboard();
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let result = handle_crossterm_event(
        &mut app,
        Event::Paste("should be ignored".to_owned()),
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(result.is_ok());
    let msgs = drain_channel(&mut rx);
    assert!(
        msgs.is_empty(),
        "Paste in Dashboard mode must send no IPC messages; got {}",
        msgs.len()
    );
}

// ---------------------------------------------------------------------------
// HIGH-001: handle_crossterm_event reads app.kitty_active (SSOT regression guard)
//
// These tests verify that handle_crossterm_event correctly threads app.kitty_active
// into key_event_to_pty_bytes. A regression that hardcodes kitty_active=false in the
// production path (e.g., by ignoring app.kitty_active and always calling
// dispatch_embedded_terminal_key with false) must fail these tests.
//
// Source: S-040 pass-3 directive HIGH-001; SS-embedded-pty.md §Translation function
// S2-002 / ADV-BLOCKER-001; BC-2.09.004 PC-1.
// ---------------------------------------------------------------------------

/// HIGH-001 / BC-2.09.004 PC-1 — Ctrl+Shift+Enter with kitty_active=true sends \x1b[13;6u
///
/// When app.kitty_active=true and the event is Ctrl+Shift+Enter, handle_crossterm_event
/// must route through the Kitty CSI-u path and send \x1b[13;6u as the KeyInput bytes.
///
/// Enter codepoint = 13; modifier = 1 + shift(1) + ctrl(4) = 6.
///
/// If the production code ignores app.kitty_active (hardcodes false), no KeyInput is
/// sent for Ctrl+Shift+Enter (Enter has mods.is_empty() guard; Ctrl+Enter has no
/// Ctrl+printable arm; the TRACE+None arm fires → None → no send). This test catches
/// that regression.
///
/// Source: S-040 pass-3 directive HIGH-001; BC-2.09.004 PC-1.
#[tokio::test]
async fn test_BC_2_09_004_handle_crossterm_event_kitty_active_true_ctrl_shift_enter() {
    let (mut app, mut rx) = make_app_embedded("session-kitty");
    // Set kitty_active=true — this is the production path for Kitty-capable terminals.
    app.kitty_active = true;
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let key_event = make_key_event(
        KeyCode::Enter,
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        KeyEventKind::Press,
    );
    let result = handle_crossterm_event(
        &mut app,
        Event::Key(key_event),
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(result.is_ok(), "handle_crossterm_event must not error for Ctrl+Shift+Enter");

    let msgs = drain_channel(&mut rx);
    assert_eq!(
        msgs.len(),
        1,
        "Ctrl+Shift+Enter with kitty_active=true must send exactly one KeyInput \
         (Kitty CSI-u path); got {} messages. \
         If 0: production code is ignoring app.kitty_active (HIGH-001 SSOT regression).",
        msgs.len()
    );
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            // Enter codepoint=13; modifier=1+shift(1)+ctrl(4)=6 → \x1b[13;6u
            assert_eq!(
                bytes.as_slice(),
                b"\x1b[13;6u",
                "Ctrl+Shift+Enter with kitty_active=true must produce \\x1b[13;6u (BC-2.09.004 PC-1)"
            );
            assert_eq!(session_id, "session-kitty");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
    // Mode must remain EmbeddedTerminal (Enter is not Esc).
    assert!(
        matches!(app.mode, AppMode::EmbeddedTerminal { .. }),
        "EmbeddedTerminal mode must be preserved after Ctrl+Shift+Enter"
    );
}

/// HIGH-001 / BC-2.09.002 EC-228 — Ctrl+Shift+Enter with kitty_active=false sends nothing
///
/// When app.kitty_active=false (non-Kitty terminal), Ctrl+Shift+Enter has no VT encoding:
/// - Enter arm has `mods.is_empty()` guard → skipped (mods are CONTROL|SHIFT).
/// - Ctrl+printable arm requires PtyKeyCode::Char — PtyKeyCode::Enter does not match.
/// - Kitty catch-all: is_kitty_enhanced_key returns false (kitty_active=false).
/// - TRACE+None arm fires → None → no KeyInput sent.
///
/// This test proves kitty_active=false is the non-Kitty fallback boundary (EC-228),
/// and is the counterpart to the kitty_active=true test above (HIGH-001 SSOT pair).
///
/// Source: S-040 pass-3 directive HIGH-001; BC-2.09.002 EC-228.
#[tokio::test]
async fn test_BC_2_09_002_handle_crossterm_event_kitty_active_false_ctrl_shift_enter_no_send() {
    let (mut app, mut rx) = make_app_embedded("session-non-kitty");
    // kitty_active remains false (default) — non-Kitty terminal.
    // app.kitty_active is false by default from App::new().
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let key_event = make_key_event(
        KeyCode::Enter,
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        KeyEventKind::Press,
    );
    let result = handle_crossterm_event(
        &mut app,
        Event::Key(key_event),
        &layers,
        &mut sessions_state,
    )
    .await;

    assert!(
        result.is_ok(),
        "handle_crossterm_event must not error for Ctrl+Shift+Enter on non-Kitty terminal"
    );

    let msgs = drain_channel(&mut rx);
    // No VT encoding exists for Ctrl+Shift+Enter on non-Kitty → TRACE+None → 0 bytes sent.
    assert!(
        msgs.is_empty(),
        "Ctrl+Shift+Enter with kitty_active=false must send 0 KeyInput messages \
         (TRACE+None path; EC-228); got {}. \
         If non-zero: production code is ignoring kitty_active=false guard.",
        msgs.len()
    );
}

/// Verify 'q' in Dashboard mode still produces Err (quit signal) via binding chain
#[tokio::test]
async fn test_quit_key_in_dashboard_returns_err() {
    let (mut app, _rx) = make_app_dashboard();
    let layers = empty_binding_layers();
    let mut sessions_state = Default::default();

    let q_event = make_key_event(KeyCode::Char('q'), KeyModifiers::NONE, KeyEventKind::Press);
    let result =
        handle_crossterm_event(&mut app, Event::Key(q_event), &layers, &mut sessions_state).await;

    // 'q' in Dashboard resolves to Action::Quit → KeyOutcome::Quit → Err(())
    assert!(
        result.is_err(),
        "handle_crossterm_event must return Err for quit key in Dashboard"
    );
}
