//! Keyboard and paste dispatch stubs for `AppMode::EmbeddedTerminal`.
//!
//! This module contains the stub entry points for the S-040 keyboard forwarding work:
//! - Global Kitty keyboard enhancement + bracketed paste TUI startup/exit.
//! - `EmbeddedTerminal` keyboard dispatch arm (Key events → `KeyInput` IPC).
//! - `EmbeddedTerminal` paste dispatch arm (Paste events → bracketed-wrap → `KeyInput` IPC).
//!
//! All function bodies are `todo!()` — the implementer writes real code here.
//! Tests for these functions MUST fail (Red Gate) until the implementer completes them.
//!
//! # Integration contract
//!
//! `setup_keyboard_enhancement()` is called from `app::run()` at TUI startup (after
//! `enable_raw_mode()`), and `teardown_keyboard_enhancement()` is called on TUI exit.
//!
//! `dispatch_embedded_terminal_key()` and `dispatch_embedded_terminal_paste()` are called
//! from `app::run()`'s main event loop inside the `AppMode::EmbeddedTerminal` arm.
//!
//! # Esc intercept ordering (BC-2.09.002 Invariant 2)
//!
//! `dispatch_embedded_terminal_key()` MUST check for bare Esc BEFORE calling
//! `crossterm_key_to_pty()` or `key_event_to_pty_bytes()`. This ordering is non-negotiable:
//! the Action dispatch layer intercepts `KeyCode::Esc` (no modifiers) as
//! `Action::ExitEmbeddedTerminal` before the key reaches the PTY forwarding path.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use monocle_core::keyboard::key_event_to_pty_bytes;
use monocle_ipc::framing::MAX_MESSAGE_BYTES;
use monocle_ipc::types::ClientToServer;
use tokio::sync::mpsc::Sender;

use crate::keyboard_conv::crossterm_key_to_pty;

/// Install global keyboard enhancement flags and enable bracketed paste at TUI startup.
///
/// Must be called ONCE at TUI startup, after `enable_raw_mode()`, BEFORE entering the
/// main event loop. These flags are global (not gated on `EmbeddedTerminal` entry) per
/// BC-2.09.004 Invariant 1 and BC-2.09.005 Invariant 1.
///
/// # Detection (ADV-BLOCKER-001 / SS-embedded-pty.md §Risk Mitigations v1.13.0)
///
/// Uses `crossterm::terminal::supports_keyboard_enhancement()` to detect Kitty
/// keyboard protocol support. This is the correct crossterm API — an earlier
/// hand-rolled probe approach was retired because it violated the architectural
/// rule against threads reading stdin outside the crossterm event loop.
///
/// EC-234: `Err(_)` from `supports_keyboard_enhancement` → treated as `false`;
/// TRACE log emitted, no panic. Kitty flags are NOT pushed on Err.
///
/// `EnableBracketedPaste` and `DisableBracketedPaste` remain unconditional —
/// they are widely supported and low risk independent of Kitty protocol.
///
/// # Returns
///
/// The `kitty_active: bool`. Callers store this in an `Arc<AtomicBool>` so all
/// TUI exit paths (normal and panic) call `teardown_keyboard_enhancement(kitty_active)`
/// symmetrically.
///
/// # Errors
///
/// Returns `Err` if any stdout `execute!` call (PushKeyboardEnhancementFlags or
/// EnableBracketedPaste) fails.
pub fn setup_keyboard_enhancement() -> Result<bool> {
    use crossterm::event::{
        EnableBracketedPaste, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    };
    use std::io;

    // ADV-BLOCKER-001 / SS-embedded-pty.md §Risk Mitigations v1.13.0:
    // Use crossterm::terminal::supports_keyboard_enhancement() — the canonical detection
    // API. FORBIDDEN: hand-rolled CSI?u probe with a detached stdin reader thread.
    //
    // EC-234: Err → false (kitty_active=false), TRACE log, flags skipped, no panic.
    let kitty = crossterm::terminal::supports_keyboard_enhancement().unwrap_or_else(|e| {
        tracing::trace!(
            error = %e,
            "setup_keyboard_enhancement: supports_keyboard_enhancement() returned Err; \
             treating as false (EC-234) — Kitty flags skipped, bracketed paste still enabled"
        );
        false
    });

    if kitty {
        tracing::trace!(
            "setup_keyboard_enhancement: Kitty protocol detected; installing enhancement flags"
        );
        // The canonical flag set is 3 flags:
        //   DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES
        // REPORT_ASSOCIATED_TEXT is unavailable in crossterm 0.29 and intentionally excluded.
        // REPORT_ALTERNATE_KEYS is intentionally omitted (not required by this protocol surface).
        crossterm::execute!(
            io::stdout(),
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
            ),
        )?;
    } else {
        tracing::trace!(
            "setup_keyboard_enhancement: Kitty not detected; skipping PushKeyboardEnhancementFlags"
        );
    }

    // EnableBracketedPaste unconditionally — widely supported, low risk.
    // BC-2.09.005 Invariant 1: paste support is independent of Kitty protocol.
    crossterm::execute!(io::stdout(), EnableBracketedPaste)?;

    Ok(kitty)
}

/// Remove keyboard enhancement flags and disable bracketed paste at TUI exit.
///
/// Must be called on all TUI exit paths (normal exit and panic hook). Called AFTER the
/// main event loop exits but BEFORE `disable_raw_mode()`.
///
/// Conditionally sends:
/// - `crossterm::event::PopKeyboardEnhancementFlags` (only when `kitty_active == true`)
/// - `crossterm::event::DisableBracketedPaste` (always)
///
/// The `kitty_active` flag must match the return value of `setup_keyboard_enhancement()`.
/// If `PushKeyboardEnhancementFlags` was not sent, `PopKeyboardEnhancementFlags` MUST NOT
/// be sent — mismatched push/pop corrupts the terminal's keyboard enhancement stack.
///
/// Errors are logged at WARN level and swallowed — the terminal is being torn down
/// and there is nothing useful to do with a teardown failure.
pub fn teardown_keyboard_enhancement(kitty_active: bool) {
    use crossterm::event::{DisableBracketedPaste, PopKeyboardEnhancementFlags};
    use std::io::stdout;

    if kitty_active {
        // Pop only if we pushed — mismatched push/pop corrupts the enhancement stack.
        if let Err(e) =
            crossterm::execute!(stdout(), PopKeyboardEnhancementFlags, DisableBracketedPaste,)
        {
            tracing::warn!("teardown_keyboard_enhancement: execute! failed: {e}");
        }
    } else {
        // Kitty was not active — only disable bracketed paste (always enabled).
        if let Err(e) = crossterm::execute!(stdout(), DisableBracketedPaste) {
            tracing::warn!("teardown_keyboard_enhancement: DisableBracketedPaste failed: {e}");
        }
    }
}

/// Dispatch a crossterm `KeyEvent` while `AppMode::EmbeddedTerminal` is active.
///
/// This is the SINGLE code path from crossterm `Event::Key` to `KeyInput` send
/// (ADV-HIGH-002 / SS-embedded-pty.md §SSOT dispatch ruling). `handle_crossterm_event`
/// in `app.rs` calls this helper passing `app.kitty_active` — there MUST NOT be an
/// inline duplicate dispatch path in `handle_crossterm_event`.
///
/// Implements the keyboard forwarding path per BC-2.09.002:
/// 1. Intercepts bare `KeyCode::Esc` (no modifiers) → returns `true` as the exit signal
///    (caller calls `exit_embedded_terminal`). BEFORE any conversion — BC-2.09.002 Invariant 2.
/// 2. All other Press/Repeat events → `crossterm_key_to_pty(event)` →
///    `key_event_to_pty_bytes(pty_event, kitty_active)`.
/// 3. If `key_event_to_pty_bytes` returns `Some(bytes)` → sends
///    `ClientToServer::KeyInput { session_id, bytes }` via `ipc_tx`.
/// 4. Release events → `key_event_to_pty_bytes` returns `None` → nothing sent (BC-2.09.002 PC-3).
///
/// Kitty-enhanced keys: handled inside `key_event_to_pty_bytes` via `is_kitty_enhanced_key` /
/// `encode_kitty_key` (BC-2.09.004).
///
/// # Parameters
///
/// - `event`: The raw crossterm `KeyEvent` from the event loop.
/// - `session_id`: UUID string of the currently-embedded session.
/// - `kitty_active`: Whether Kitty keyboard protocol was negotiated at startup.
///   `true` → modifier combos route to Kitty CSI-u encoding via `encode_kitty_key`.
///   `false` → standard VT encoding (non-Kitty fallback table).
///   Set from `app.kitty_active` in production; `false` in standalone unit tests.
/// - `ipc_tx`: The outbound IPC channel sender. `.send().await` is used (backpressure,
///   per F-S039-004 ruling — `try_send()` is forbidden for keyboard bytes).
///
/// # Returns
///
/// `true` if `Action::ExitEmbeddedTerminal` was dispatched (Esc intercepted);
/// `false` for all other outcomes (key forwarded or discarded).
pub async fn dispatch_embedded_terminal_key(
    event: KeyEvent,
    session_id: &str,
    kitty_active: bool,
    ipc_tx: &Sender<ClientToServer>,
) -> bool {
    // BC-2.09.002 Invariant 2: Intercept bare Esc (no modifiers, Press only) as
    // ExitEmbeddedTerminal BEFORE calling key_event_to_pty_bytes(). This ordering is
    // non-negotiable. The `kind == Press` guard is required because Kitty keyboard
    // protocol emits both Press and Release events; a bare-Esc Release MUST NOT trigger
    // exit (PC-3 — Release events are discarded, not intercepted).
    if event.code == KeyCode::Esc
        && event.modifiers == KeyModifiers::NONE
        && event.kind == crossterm::event::KeyEventKind::Press
    {
        // Signal caller to dispatch Action::ExitEmbeddedTerminal.
        // Nothing is sent to the PTY for this Esc.
        return true;
    }

    // Convert crossterm type to core-owned type at the purity boundary.
    let pty_event = crossterm_key_to_pty(event);

    // Translate to PTY bytes. Returns None for Release events and unrecognized keys.
    // `kitty_active` routes modifier combos to Kitty CSI-u encoding when true;
    // otherwise uses standard VT fallback table.
    if let Some(bytes) = key_event_to_pty_bytes(pty_event, kitty_active) {
        // Send with backpressure (.send().await) per F-S039-004 ruling.
        // If the channel is closed (daemon dead), log and continue — the TUI
        // will detect the disconnect via transport events.
        if let Err(e) = ipc_tx
            .send(ClientToServer::KeyInput {
                session_id: session_id.to_owned(),
                bytes,
            })
            .await
        {
            tracing::warn!(
                "dispatch_embedded_terminal_key: IPC send failed (channel closed?): {e}"
            );
        }
    }

    false
}

/// Dispatch a bracketed paste event while `AppMode::EmbeddedTerminal` is active.
///
/// Implements BC-2.09.005 paste forwarding:
/// - Wraps `text` as `\x1b[200~` + text.as_bytes() + `\x1b[201~`.
/// - Sends the complete bracketed payload as `ClientToServer::KeyInput { session_id, bytes }`.
/// - Large pastes (> 64 KiB) are sent as a single `KeyInput` — no chunking (BC-2.09.005 Invariant 3).
///
/// This function handles `Event::Paste` ONLY — paste events MUST NOT be routed through
/// `key_event_to_pty_bytes()` (BC-2.09.005 Invariant 2; S-040 AC-010).
///
/// # Parameters
///
/// - `text`: The pasted text string from `crossterm::event::Event::Paste(text)`.
/// - `session_id`: UUID string of the currently-embedded session.
/// - `ipc_tx`: The outbound IPC channel sender.
pub async fn dispatch_embedded_terminal_paste(
    text: &str,
    session_id: &str,
    ipc_tx: &Sender<ClientToServer>,
) {
    // BC-2.09.005 EC-245: oversized-paste guard — checks SERIALIZED frame size.
    //
    // write_framed encodes ClientToServer::KeyInput as JSON via serde_json::to_vec.
    // Vec<u8> bytes serialize as a JSON integer array (e.g., [120,27,91,...]), where
    // each 3-digit decimal value expands to ~4 chars.  This produces roughly 3-4x
    // expansion from raw byte count to JSON frame size.
    //
    // The guard MUST compare the actual serialized frame size (exactly what write_framed
    // checks) — not the raw bracketed byte count.  Comparing raw length would pass a
    // paste of ~80_000 bytes (raw 80_012 < 262_144) even though its JSON frame is
    // ~320_100 bytes > 262_144, causing write_framed to reject with MessageTooLarge,
    // which KILLS the IPC writer task and silently drops ALL subsequent keystrokes.
    //
    // Approach: build the exact ClientToServer::KeyInput message that would be sent,
    // serialize it with serde_json::to_vec, and measure the result.  This guarantees
    // parity with write_framed — no false negatives, no false positives.
    //
    // Per BC-2.09.005 Invariant 3, fragmentation is forbidden.  Drop is the correct
    // response when the serialized frame exceeds the ceiling.

    // BC-2.09.005 PC-2/PC-3: wrap paste text in bracketed paste sequences.
    // Full payload in a single KeyInput — no chunking (BC-2.09.005 Invariant 3).
    let bracketed_len = b"\x1b[200~".len() + text.len() + b"\x1b[201~".len();
    let mut bytes = Vec::with_capacity(bracketed_len);
    bytes.extend_from_slice(b"\x1b[200~");
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(b"\x1b[201~");

    // Build the exact message to measure its serialized size before enqueuing.
    let msg = ClientToServer::KeyInput {
        session_id: session_id.to_owned(),
        bytes,
    };

    // Serialize to measure the frame size exactly as write_framed would.
    // serde_json::to_vec failure on a KeyInput is not realistically possible
    // (no non-serializable fields), but guard defensively.
    let serialized = match serde_json::to_vec(&msg) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                "dispatch_embedded_terminal_paste: failed to serialize KeyInput for size check; \
                 dropping paste (BC-2.09.005 EC-245): {e}"
            );
            return;
        }
    };

    if serialized.len() > MAX_MESSAGE_BYTES {
        tracing::warn!(
            paste_bytes = bracketed_len,
            serialized_bytes = serialized.len(),
            ceiling = MAX_MESSAGE_BYTES,
            "dispatch_embedded_terminal_paste: serialized paste frame exceeds IPC ceiling; \
             dropping (BC-2.09.005 EC-245 — fragmentation forbidden, oversized paste discarded)"
        );
        return;
    }

    if let Err(e) = ipc_tx.send(msg).await {
        tracing::warn!("dispatch_embedded_terminal_paste: IPC send failed (channel closed?): {e}");
    }
}
