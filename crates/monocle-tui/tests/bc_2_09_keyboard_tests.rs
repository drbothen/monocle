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
//! # Coverage
//!
//! These tests verify the fully-implemented dispatch functions in event_loop.rs
//! (dispatch_embedded_terminal_key, dispatch_embedded_terminal_paste).
//! All tests pass against the current production implementation.

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

    let exited = dispatch_embedded_terminal_key(event, "session-abc", false, &tx).await;

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

    let exited = dispatch_embedded_terminal_key(event, "session-abc", false, &tx).await;

    assert!(
        !exited,
        "dispatch_embedded_terminal_key must return false for non-Esc key"
    );

    // Key 'a' must be forwarded as [0x61]
    let sent = drain_channel(&mut rx);
    assert_eq!(sent.len(), 1, "exactly one KeyInput should be sent for 'a'");
    match &sent[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            assert_eq!(bytes, &[0x61], "Char('a') should produce bytes [0x61]");
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

    let exited = dispatch_embedded_terminal_key(event, "session-abc", false, &tx).await;

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
// ADV-MED-001 — Esc Release must NOT trigger ExitEmbeddedTerminal
// ---------------------------------------------------------------------------

/// BC-2.09.002 Invariant 2 / PC-3 — bare Esc KeyEventKind::Release must not exit
///
/// When Kitty keyboard protocol is active (kitty_active=true), crossterm emits
/// both Press and Release events for every key. The ExitEmbeddedTerminal intercept
/// MUST fire ONLY on KeyEventKind::Press. A bare-Esc Release event (code Esc,
/// modifiers NONE, kind Release) MUST be discarded: dispatch returns false and
/// zero bytes are sent to the IPC channel.
///
/// This was the ADV-MED-001 Red Gate test. The guard at event_loop.rs correctly
/// checks `code == Esc && modifiers == NONE && kind == Press`, so a Release event
/// returns false (correctly not signalling exit). This test is now a regression
/// guard confirming the kind==Press check remains in place.
///
/// Contrast with test_BC_2_09_002_esc_not_forwarded_directly, which uses
/// KeyEventKind::Press and expects true (exit signalled).
#[tokio::test]
async fn test_BC_2_09_002_esc_release_does_not_exit() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);
    // Release events only occur when Kitty keyboard enhancement is active.
    let event = make_key_event(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Release,
    );

    let exited = dispatch_embedded_terminal_key(event, "session-esc-release", true, &tx).await;

    // Release MUST NOT signal exit — ADV-MED-001 / BC-2.09.002 Invariant 2 / PC-3
    assert!(
        !exited,
        "dispatch_embedded_terminal_key must return false for bare Esc Release \
         (BC-2.09.002 INV-2 / PC-3 — Release must be discarded, not intercepted as exit)"
    );

    // Release events are discarded; zero bytes must be sent to the IPC channel.
    let sent = drain_channel(&mut rx);
    assert!(
        sent.is_empty(),
        "Esc Release must send 0 bytes to PTY; got {} message(s) (ADV-MED-001)",
        sent.len()
    );
}

// ---------------------------------------------------------------------------
// ADV-OBS-1 — Esc Repeat must NOT exit; must forward \x1b to PTY
// ---------------------------------------------------------------------------

/// BC-2.09.002 Invariant 2 — bare Esc KeyEventKind::Repeat forwards \x1b, does not exit
///
/// Invariant 2 defines three behaviors keyed on KeyEventKind for bare Esc (no modifiers):
///   Press  → Action::ExitEmbeddedTerminal (intercepted, returns true, zero bytes sent)
///   Release → discarded (returns false, zero bytes sent — PC-3)
///   Repeat → NOT intercepted; falls through dispatch_embedded_terminal_key to
///            key_event_to_pty_bytes which produces \x1b (the raw Esc byte forwarded to PTY)
///
/// Repeat events only occur when Kitty keyboard enhancement is active (kitty_active=true).
/// The ExitEmbeddedTerminal guard in dispatch_embedded_terminal_key requires kind==Press;
/// a Repeat event does NOT satisfy that guard and MUST fall through to the PTY forwarding path.
///
/// This is ADV-OBS-1 coverage — closes the third-branch gap. The code is already correct;
/// this test is a regression guard so a future broadening of the guard to `kind != Release`
/// (which would wrongly intercept Repeat) is caught immediately.
///
/// Contrast with:
///   test_BC_2_09_002_esc_not_forwarded_directly  — Press returns true, zero bytes
///   test_BC_2_09_002_esc_release_does_not_exit   — Release returns false, zero bytes
#[tokio::test]
async fn test_BC_2_09_002_esc_repeat_forwards_esc_not_exit() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);
    // Repeat events only occur when Kitty keyboard enhancement is active.
    let event = make_key_event(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Repeat,
    );

    let exited = dispatch_embedded_terminal_key(event, "session-esc-repeat", true, &tx).await;

    // Repeat MUST NOT signal exit — ADV-OBS-1 / BC-2.09.002 Invariant 2 Repeat clause
    assert!(
        !exited,
        "dispatch_embedded_terminal_key must return false for bare Esc Repeat \
         (BC-2.09.002 Invariant 2 — Repeat is NOT intercepted as ExitEmbeddedTerminal)"
    );

    // Esc Repeat must produce exactly one KeyInput carrying the raw \x1b byte.
    let sent = drain_channel(&mut rx);
    assert_eq!(
        sent.len(),
        1,
        "Esc Repeat must forward exactly one KeyInput to PTY; got {} message(s) (ADV-OBS-1)",
        sent.len()
    );
    match &sent[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            assert_eq!(
                bytes.as_slice(),
                b"\x1b",
                "Esc Repeat must produce raw \\x1b byte (BC-2.09.002 Invariant 2 Repeat clause)"
            );
            assert_eq!(session_id, "session-esc-repeat");
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

    dispatch_embedded_terminal_key(event, "my-session-id-001", false, &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(sent.len(), 1);
    match &sent[0] {
        ClientToServer::KeyInput { session_id, .. } => {
            assert_eq!(session_id, "my-session-id-001");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// BC-2.09.005: Paste edge cases — EC-230 and EC-231
// These tests exercise the dispatch helper directly against the fully-implemented
// dispatch_embedded_terminal_paste function. They pass against the current production code.
// ---------------------------------------------------------------------------

/// BC-2.09.005 EC-230 — paste text containing ESC characters is forwarded verbatim
///
/// When the pasted text contains raw ESC bytes (e.g. ANSI color codes like
/// "\x1b[31mred\x1b[0m"), the ESC bytes are included verbatim inside the bracket
/// sequences. No sanitization occurs. The PTY receives them as data.
///
/// Expected: \x1b[200~\x1b[31mred\x1b[0m\x1b[201~
/// (outer bracket + raw ESC-containing text + outer bracket — all contiguous)
///
/// Source: BC-2.09.005 EC-230.
#[tokio::test]
async fn test_BC_2_09_005_paste_with_esc_verbatim() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(4);

    // Text contains raw ESC bytes — ANSI color sequence.
    let ansi_text = "\x1b[31mred\x1b[0m";
    dispatch_embedded_terminal_paste(ansi_text, "session-esc-paste", &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(
        sent.len(),
        1,
        "paste with ESC chars must send exactly one KeyInput"
    );
    match &sent[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            // The expected bytes are:
            //   \x1b[200~  — bracketed paste open sequence (6 bytes)
            //   \x1b[31mred\x1b[0m  — verbatim text including ESC bytes
            //   \x1b[201~  — bracketed paste close sequence (6 bytes)
            // No sanitization of inner ESC bytes.
            let mut expected = b"\x1b[200~".to_vec();
            expected.extend_from_slice(ansi_text.as_bytes());
            expected.extend_from_slice(b"\x1b[201~");

            assert_eq!(
                bytes.as_slice(),
                expected.as_slice(),
                "paste with ESC chars must be forwarded verbatim inside brackets (EC-230)"
            );
            assert_eq!(session_id, "session-esc-paste");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}

/// BC-2.09.005 EC-231 — paste text containing embedded \x1b[200~ sequence forwarded verbatim
///
/// When the pasted text itself contains the literal string "\x1b[200~" (attacker
/// input or nested bracket sequence), it is forwarded verbatim inside the outer
/// bracket sequences. monocle does NOT sanitize or strip embedded bracket sequences.
/// The PTY receives the outer brackets plus the verbatim inner text.
///
/// Expected: \x1b[200~<inner text with embedded \x1b[200~>\x1b[201~
///
/// Source: BC-2.09.005 EC-231.
#[tokio::test]
async fn test_BC_2_09_005_paste_embedded_bracket_verbatim() {
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(4);

    // The paste text itself contains the opening bracket sequence.
    let inner_text = "before\x1b[200~after";
    dispatch_embedded_terminal_paste(inner_text, "session-bracket-embed", &tx).await;

    let sent = drain_channel(&mut rx);
    assert_eq!(
        sent.len(),
        1,
        "paste with embedded bracket must send exactly one KeyInput"
    );
    match &sent[0] {
        ClientToServer::KeyInput { bytes, session_id } => {
            // Expected bytes:
            //   \x1b[200~  — outer bracket open (6 bytes)
            //   before\x1b[200~after  — verbatim inner text (not stripped/sanitized)
            //   \x1b[201~  — outer bracket close (6 bytes)
            let mut expected = b"\x1b[200~".to_vec();
            expected.extend_from_slice(inner_text.as_bytes());
            expected.extend_from_slice(b"\x1b[201~");

            assert_eq!(
                bytes.as_slice(),
                expected.as_slice(),
                "paste with embedded bracket sequence must be forwarded verbatim (EC-231)"
            );
            assert_eq!(session_id, "session-bracket-embed");
        }
        other => panic!("expected ClientToServer::KeyInput, got {:?}", other),
    }
}
