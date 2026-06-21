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
// dispatch_embedded_terminal_key with false) would be caught by these tests.
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

    assert!(
        result.is_ok(),
        "handle_crossterm_event must not error for Ctrl+Shift+Enter"
    );

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

// ---------------------------------------------------------------------------
// PASS-4 ADV-HIGH-002: oversized-paste guard (BC-2.09.005 EC-245)
//
// dispatch_embedded_terminal_paste MUST check whether the bracketed IPC payload
// would exceed MAX_MESSAGE_BYTES (262144) before enqueueing. If it would, the paste
// is DROPPED (WARN log, no send) and the IPC writer task remains alive.
//
// Rationale (BC-2.09.005 EC-245): the no-fragmentation rule precludes splitting a
// single paste across multiple KeyInput messages. An over-ceiling paste without this
// guard would enqueue a frame that write_framed rejects with MessageTooLarge, which
// terminates the IPC writer task and silently drops ALL subsequent keystrokes.
//
// dispatch_embedded_terminal_paste implements this size guard; these tests verify
// that oversized payloads are dropped and the channel remains open.
// ---------------------------------------------------------------------------

/// ADV-HIGH-002 / BC-2.09.005 EC-245 — oversized paste is dropped, writer task survives
///
/// Constructs a paste text of 262200 bytes so that
///   `\x1b[200~` (6) + 262200 + `\x1b[201~` (6) = 262212 bytes of raw payload.
/// The KeyInput JSON envelope adds at minimum:
///   `{"KeyInput":{"session_id":"s","bytes":[` + `, digits, ]}}` ~ 50+ bytes.
/// The total serialized JSON therefore exceeds MAX_MESSAGE_BYTES (262144 bytes).
///
/// The guard in dispatch_embedded_terminal_paste MUST detect this before sending.
/// After the guard fires: the mpsc channel receives ZERO messages, the sender is
/// NOT closed, and subsequent small pastes still succeed (writer task alive).
///
/// Source: BC-2.09.005 EC-245; ADV-HIGH-002.
#[tokio::test]
async fn test_BC_2_09_005_oversized_paste_guard() {
    use monocle_ipc::types::ClientToServer;
    use monocle_tui::event_loop::dispatch_embedded_terminal_paste;
    use tokio::sync::mpsc;

    // A paste text of 262200 bytes:
    //   bracketed frame: 6 + 262200 + 6 = 262212 raw bytes
    //   JSON bytes-array encoding: each byte becomes up to 4 chars ("255,"),
    //   so the array alone is >>262144 bytes — clearly over the 262144 ceiling.
    let oversized_text = "x".repeat(262_200);
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);

    dispatch_embedded_terminal_paste(&oversized_text, "session-oversized", &tx).await;

    // MUST receive ZERO messages — the guard dropped the oversized paste.
    let msgs: Vec<_> = {
        let mut v = Vec::new();
        while let Ok(m) = rx.try_recv() {
            v.push(m);
        }
        v
    };
    assert!(
        msgs.is_empty(),
        "oversized paste must be dropped by guard (0 KeyInput messages expected); \
         got {}. The size guard in dispatch_embedded_terminal_paste must drop oversized frames \
         before enqueueing. Source: BC-2.09.005 EC-245; ADV-HIGH-002",
        msgs.len()
    );

    // The sender MUST still be alive (writer task not killed).
    // Verify: a small follow-up paste succeeds (channel still open).
    dispatch_embedded_terminal_paste("hello", "session-oversized", &tx).await;
    let follow_up: Vec<_> = {
        let mut v = Vec::new();
        while let Ok(m) = rx.try_recv() {
            v.push(m);
        }
        v
    };
    assert_eq!(
        follow_up.len(),
        1,
        "IPC channel must remain open after oversized paste drop; \
         follow-up 'hello' paste must produce exactly 1 KeyInput"
    );
    match &follow_up[0] {
        ClientToServer::KeyInput { bytes, .. } => {
            assert_eq!(
                bytes.as_slice(),
                b"\x1b[200~hello\x1b[201~",
                "follow-up paste after oversized drop must produce correct bracketed sequence \
                 (regression guard: EC-232/AC-009 happy path not broken)"
            );
        }
        other => panic!("expected KeyInput for follow-up paste, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// PASS-5 BLOCKER-001: JSON-expansion paste guard (BC-2.09.005 EC-245)
//
// The production guard in dispatch_embedded_terminal_paste must account for
// JSON serialization expansion when comparing against MAX_MESSAGE_BYTES.
// write_framed serializes ClientToServer::KeyInput to JSON, encoding the bytes
// Vec<u8> as a JSON integer array.  Each byte value 0-255 serializes as its
// decimal string representation plus a comma separator:
//
//   byte 120 ('x') -> "120," = 4 chars    (3-digit value)
//   byte  27 (\x1b) -> "27,"  = 3 chars
//
// This produces an approximately 3-4x expansion from raw to serialized frame size.
//
// The failure mode without a serialized-size guard:
//   A paste of 80_000 'x' chars has:
//     - raw bracketed length:  6 + 80_000 + 6 = 80_012 bytes  (< 262_144 ceiling)
//     - JSON frame length:     ~320_100 bytes                  (> 262_144 ceiling)
//
// A guard that checks only the raw byte count would pass the 80_000-char paste.
// write_framed would then reject it with MessageTooLarge, killing the IPC writer
// task and silently dropping ALL subsequent keystrokes for the session.
//
// BC-2.09.005 EC-245 requires the guard to check the FRAMED/SERIALIZED size, not
// the raw bracketed byte count.  The production implementation satisfies this.
// ---------------------------------------------------------------------------

/// BLOCKER-001 (pass-5) / BC-2.09.005 EC-245 — JSON-expansion paste slips raw guard, kills writer
///
/// Constructs a paste whose RAW bracketed length is UNDER the 262_144-byte ceiling but
/// whose SERIALIZED JSON frame is OVER it.  Specifically:
///
///   text = "x".repeat(80_000)   (each 'x' byte = 120)
///
///   raw bracketed length:  b"\x1b[200~".len() + 80_000 + b"\x1b[201~".len()
///                        = 6 + 80_000 + 6 = 80_012 bytes
///                        80_012 < 262_144  =>  would pass a naive raw-byte guard
///
///   JSON frame length:     each 3-digit byte value ("120,") = 4 chars/byte
///                          total bytes-array ~= 80_012 * 4 = 320_048 chars
///                          full frame ~= 320_100 bytes
///                          320_100 > 262_144  =>  write_framed rejects with MessageTooLarge
///
/// The IPC writer task calls write_framed after dequeueing.  MessageTooLarge terminates
/// that task, silently dropping ALL subsequent keystrokes.
///
/// The CORRECT behavior (BC-2.09.005 EC-245): the guard in dispatch_embedded_terminal_paste
/// must detect this before sending (by computing or approximating the serialized frame
/// size) and drop the paste, leaving the channel/writer alive.
///
/// After the drop, a small follow-up paste ("hello") must succeed (channel still open,
/// writer task not killed) — the writer-survival assertion.
///
/// # Regression Guard
///
/// This test was the BLOCKER-001 Red Gate (pass-5). The production implementation
/// now checks the serialized JSON frame size before enqueueing, so this test passes.
/// It remains as a regression guard: a future change that reverts to a raw-byte-only
/// guard would cause the 80_000-char paste to be enqueued (1 message received,
/// expected 0) and this test would catch it immediately.
///
/// Source: BC-2.09.005 EC-245; S-040 BLOCKER-001 (pass-5).
#[tokio::test]
async fn test_BC_2_09_005_paste_json_expansion_guard() {
    use monocle_ipc::types::ClientToServer;
    use monocle_tui::event_loop::dispatch_embedded_terminal_paste;
    use tokio::sync::mpsc;

    // 80_000 'x' chars:
    //   raw bracketed = 80_012 bytes  (< 262_144: passes a naive raw-byte guard)
    //   JSON frame    ~ 320_100 bytes (> 262_144: write_framed would reject without serialized-size guard)
    let expansion_text = "x".repeat(80_000);

    // Use a generous channel capacity — the test must prove the paste is NOT enqueued,
    // not that the channel is full.
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(32);

    dispatch_embedded_terminal_paste(&expansion_text, "session-expansion", &tx).await;

    // Drain everything the function sent.
    let msgs: Vec<_> = {
        let mut v = Vec::new();
        while let Ok(m) = rx.try_recv() {
            v.push(m);
        }
        v
    };

    // MUST receive ZERO messages — the serialized-size guard drops the oversized paste.
    //
    // If this assertion fails (1 message received), the guard has regressed to a
    // raw-byte-only check: 80_012 < 262_144 passes, but the JSON frame is ~320_100
    // bytes, which exceeds the 262_144 ceiling and would cause write_framed to
    // reject with MessageTooLarge, killing the IPC writer task.
    assert!(
        msgs.is_empty(),
        "JSON-expansion paste guard regression (BC-2.09.005 EC-245 / BLOCKER-001): \
         received {} KeyInput message(s); expected 0. \
         The serialized-size guard must drop this paste: raw bracketed length \
         80_012 < 262_144, but JSON frame ~320_100 bytes > 262_144 ceiling. \
         write_framed would reject with MessageTooLarge, killing the IPC writer task \
         and silently dropping ALL subsequent keystrokes. \
         Guard must check serialized frame size, not raw byte count.",
        msgs.len()
    );

    // Writer-survival assertion: even after the guard fires, the channel MUST remain
    // open.  A small follow-up paste must produce exactly one KeyInput.
    dispatch_embedded_terminal_paste("hello", "session-expansion", &tx).await;
    let follow_up: Vec<_> = {
        let mut v = Vec::new();
        while let Ok(m) = rx.try_recv() {
            v.push(m);
        }
        v
    };
    assert_eq!(
        follow_up.len(),
        1,
        "Writer-survival failure: IPC channel must remain open after JSON-expansion guard fires; \
         follow-up 'hello' paste must produce exactly 1 KeyInput (BC-2.09.005 EC-245)."
    );
    match &follow_up[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            assert_eq!(
                bytes.as_slice(),
                b"\x1b[200~hello\x1b[201~",
                "follow-up bracketed paste mismatch after JSON-expansion guard (BC-2.09.005 AC-009)"
            );
            assert_eq!(session_id, "session-expansion");
        }
        other => panic!(
            "expected ClientToServer::KeyInput for follow-up paste, got {:?}",
            other
        ),
    }
}

/// ADV-HIGH-002 regression guard / BC-2.09.005 AC-009 — normal small paste still works
///
/// Verifies that after the oversized-paste guard is implemented, a small paste
/// (e.g., "hello") still produces exactly one KeyInput with correct bracketed framing.
/// This is the happy-path regression guard confirming the guard only fires at the ceiling.
///
/// Source: BC-2.09.005 EC-232; AC-009; ADV-HIGH-002 regression guard.
#[tokio::test]
async fn test_BC_2_09_005_oversized_paste_guard_does_not_affect_small_paste() {
    use monocle_ipc::types::ClientToServer;
    use monocle_tui::event_loop::dispatch_embedded_terminal_paste;
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel::<ClientToServer>(4);
    dispatch_embedded_terminal_paste("hello", "session-small", &tx).await;

    let msgs: Vec<_> = {
        let mut v = Vec::new();
        while let Ok(m) = rx.try_recv() {
            v.push(m);
        }
        v
    };
    assert_eq!(
        msgs.len(),
        1,
        "small paste must produce exactly 1 KeyInput (not affected by oversized guard)"
    );
    match &msgs[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            assert_eq!(bytes.as_slice(), b"\x1b[200~hello\x1b[201~");
            assert_eq!(session_id, "session-small");
        }
        other => panic!("expected KeyInput, got {:?}", other),
    }
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
