//! TDD test suite for S-040: Full-Fidelity Keyboard Forwarding.
//!
//! Tests in this file exercise the monocle-tui dispatch layer:
//! - BC-2.09.002: Esc intercept (not forwarded to PTY on first press)
//! - BC-2.09.005: Bracketed paste wrapping (Event::Paste → \x1b[200~...\x1b[201~)
//!
//! Pure keyboard translation tests (key_event_to_pty_bytes, encode_kitty_key, etc.)
//! live in monocle-core/src/keyboard.rs #[cfg(test)] module — those functions have
//! no crossterm dependency and are tested against PtyKey* types only.
//!
//! # Red Gate
//!
//! All tests in this file MUST FAIL until the implementer fills in the todo!() stubs
//! in event_loop.rs (dispatch_embedded_terminal_key, dispatch_embedded_terminal_paste).
//!
//! Tests compile against the current stub code and fail at runtime via panic from todo!().

#![allow(non_snake_case)]

use monocle_ipc::types::ClientToServer;
use monocle_tui::event_loop::{dispatch_embedded_terminal_key, dispatch_embedded_terminal_paste};
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a crossterm KeyEvent with the given code, modifiers, and kind.
fn make_key_event(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
    kind: crossterm::event::KeyEventKind,
) -> crossterm::event::KeyEvent {
    crossterm::event::KeyEvent::new_with_kind(code, modifiers, kind)
}

/// Drain all messages from an mpsc receiver without blocking. Returns collected messages.
fn drain_channel(rx: &mut mpsc::Receiver<ClientToServer>) -> Vec<ClientToServer> {
    let mut out = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        out.push(msg);
    }
    out
}

// ---------------------------------------------------------------------------
// BC-2.09.002 Invariant 2 — Esc is intercepted as ExitEmbeddedTerminal, NOT forwarded
// ---------------------------------------------------------------------------

/// BC-2.09.002 Invariant 2 / EC-210 — bare Esc returns true (exit signal), zero bytes sent
///
/// The Action dispatch layer MUST intercept KeyCode::Esc (no modifiers) as
/// Action::ExitEmbeddedTerminal BEFORE key_event_to_pty_bytes() is called.
/// dispatch_embedded_terminal_key() returns `true` and sends nothing to the IPC channel.
#[tokio::test]
async fn test_BC_2_09_002_esc_not_forwarded_directly() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);
    let event = make_key_event(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Press,
    );

    let exited = dispatch_embedded_terminal_key(event, "session-abc", &tx).await;

    // Must return true (Esc was intercepted as ExitEmbeddedTerminal)
    assert!(
        exited,
        "dispatch_embedded_terminal_key must return true for bare Esc (BC-2.09.002 INV-2)"
    );

    // Zero bytes must be sent to the IPC channel
    let sent = drain_channel(&mut rx);
    assert!(
        sent.is_empty(),
        "Esc must NOT be forwarded to PTY — {} message(s) sent unexpectedly",
        sent.len()
    );
}

/// BC-2.09.002 Invariant 2 — non-Esc key returns false (not an exit event)
///
/// For any non-Esc key, dispatch returns false (did not exit).
#[tokio::test]
async fn test_BC_2_09_002_non_esc_key_does_not_exit() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);
    let event = make_key_event(
        crossterm::event::KeyCode::Char('a'),
        crossterm::event::KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Press,
    );

    let exited = dispatch_embedded_terminal_key(event, "session-abc", &tx).await;

    assert!(
        !exited,
        "dispatch_embedded_terminal_key must return false for non-Esc key"
    );

    // Key 'a' must be forwarded as [0x61]
    let sent = drain_channel(&mut rx);
    assert_eq!(sent.len(), 1, "exactly one KeyInput should be sent for 'a'");
    match &sent[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            assert_eq!(
                bytes,
                &[0x61],
                "Char('a') should produce bytes [0x61]"
            );
            assert_eq!(session_id, "session-abc");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}

/// BC-2.09.002 PC-3 / AC-013 — Release events send nothing to PTY
///
/// dispatch_embedded_terminal_key with Release kind returns false and sends
/// zero bytes to the IPC channel (key_event_to_pty_bytes returns None for Release).
#[tokio::test]
async fn test_BC_2_09_002_release_events_not_forwarded() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);
    let event = make_key_event(
        crossterm::event::KeyCode::Char('a'),
        crossterm::event::KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Release,
    );

    let exited = dispatch_embedded_terminal_key(event, "session-abc", &tx).await;

    assert!(
        !exited,
        "Release event must not trigger ExitEmbeddedTerminal"
    );
    let sent = drain_channel(&mut rx);
    assert!(
        sent.is_empty(),
        "Release events must send 0 bytes; got {} message(s)",
        sent.len()
    );
}

// ---------------------------------------------------------------------------
// BC-2.09.005: Bracketed Paste
// ---------------------------------------------------------------------------

/// BC-2.09.005 PC-2/PC-3 — "hello world" wrapped in bracket sequences
///
/// Canonical test vector from BC-2.09.005 §Canonical Test Vectors:
/// Event::Paste("hello world") → \x1b[200~hello world\x1b[201~
#[tokio::test]
async fn test_BC_2_09_005_bracketed_paste_wrapped() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);

    dispatch_embedded_terminal_paste("hello world", "session-xyz", &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(
        sent.len(),
        1,
        "exactly one KeyInput should be sent for a paste event"
    );
    match &sent[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            let expected = b"\x1b[200~hello world\x1b[201~";
            assert_eq!(
                bytes.as_slice(),
                expected,
                "paste must be wrapped in bracket sequences"
            );
            assert_eq!(session_id, "session-xyz");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}

/// BC-2.09.005 EC-232 — empty paste → \x1b[200~\x1b[201~ (bracket sequences with no content)
///
/// Canonical test vector from BC-2.09.005 §Canonical Test Vectors.
#[tokio::test]
async fn test_BC_2_09_005_bracketed_paste_empty() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);

    dispatch_embedded_terminal_paste("", "session-xyz", &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(
        sent.len(),
        1,
        "empty paste must still send one KeyInput with bracket sequences"
    );
    match &sent[0] {
        ClientToServer::KeyInput { bytes, .. } => {
            let expected = b"\x1b[200~\x1b[201~";
            assert_eq!(
                bytes.as_slice(),
                expected,
                "empty paste must produce \\x1b[200~\\x1b[201~ with no content between brackets"
            );
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}

/// BC-2.09.005 / AC-009 / AC-012 — large paste (500-byte) sent as single KeyInput, no fragmentation
///
/// A paste of 500 bytes is forwarded as a single KeyInput message (no chunking).
/// The complete bracketed payload is \x1b[200~<500 bytes>\x1b[201~.
/// Total payload size = 6 (header) + 500 + 6 (trailer) = 512 bytes.
#[tokio::test]
async fn test_BC_2_09_005_large_paste_single_key_input_no_fragmentation() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(4);

    // 500-byte paste payload
    let text: String = "x".repeat(500);
    dispatch_embedded_terminal_paste(&text, "session-large", &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(
        sent.len(),
        1,
        "500-byte paste must produce exactly 1 KeyInput (no fragmentation)"
    );
    match &sent[0] {
        ClientToServer::KeyInput { bytes, .. } => {
            let header = b"\x1b[200~";
            let trailer = b"\x1b[201~";
            let expected_len = header.len() + 500 + trailer.len();
            assert_eq!(
                bytes.len(),
                expected_len,
                "large paste payload length mismatch"
            );
            assert!(bytes.starts_with(header), "missing bracketed paste header");
            assert!(bytes.ends_with(trailer), "missing bracketed paste trailer");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}

/// BC-2.09.005 §Canonical Test Vectors — newlines preserved verbatim in paste
///
/// "line1\nline2" → \x1b[200~line1\nline2\x1b[201~ (newline not escaped)
#[tokio::test]
async fn test_BC_2_09_005_paste_newlines_preserved_verbatim() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(4);

    dispatch_embedded_terminal_paste("line1\nline2", "session-nl", &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(sent.len(), 1);
    match &sent[0] {
        ClientToServer::KeyInput { bytes, .. } => {
            let expected = b"\x1b[200~line1\nline2\x1b[201~";
            assert_eq!(bytes.as_slice(), expected);
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// BC-2.09.002 / BC-2.09.004 — session_id is threaded correctly
// ---------------------------------------------------------------------------

/// BC-2.09.002 PC-1 — session_id from parameter is included in KeyInput IPC message
///
/// The KeyInput message must carry the session_id from the dispatch call parameter.
#[tokio::test]
async fn test_BC_2_09_002_key_input_carries_correct_session_id() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(4);
    let event = make_key_event(
        crossterm::event::KeyCode::Char('x'),
        crossterm::event::KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Press,
    );

    dispatch_embedded_terminal_key(event, "my-session-id-001", &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(sent.len(), 1);
    match &sent[0] {
        ClientToServer::KeyInput { session_id, .. } => {
            assert_eq!(session_id, "my-session-id-001");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}
