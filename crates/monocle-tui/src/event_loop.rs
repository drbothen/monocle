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
use monocle_ipc::types::ClientToServer;
use tokio::sync::mpsc::Sender;

use crate::keyboard_conv::crossterm_key_to_pty;

/// Install global keyboard enhancement flags and enable bracketed paste at TUI startup.
///
/// Must be called ONCE at TUI startup, after `enable_raw_mode()`, BEFORE entering the
/// main event loop. These flags are global (not gated on `EmbeddedTerminal` entry) per
/// BC-2.09.004 Invariant 1 and BC-2.09.005 Invariant 1.
///
/// Installs:
/// - `crossterm::event::PushKeyboardEnhancementFlags` with all four flags:
///   `DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES |
///    REPORT_EVENT_TYPES | REPORT_ASSOCIATED_TEXT`
/// - `crossterm::event::EnableBracketedPaste`
///
/// If the terminal does not support Kitty keyboard protocol, `PushKeyboardEnhancementFlags`
/// silently no-ops. A TRACE log is emitted if support is undetectable. No panic; no failure.
///
/// # Errors
///
/// Returns `Err` if the crossterm `execute!` call fails (I/O error writing to stdout).
pub fn setup_keyboard_enhancement() -> Result<()> {
    use crossterm::event::{
        EnableBracketedPaste, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    };
    use std::io::stdout;

    tracing::trace!("setting up global keyboard enhancement flags and bracketed paste");

    // crossterm 0.29: REPORT_ASSOCIATED_TEXT is not stable in this version;
    // use the three stable flags. The spec calls for 4 flags; REPORT_ASSOCIATED_TEXT
    // is not exposed by crossterm 0.29 (it is commented out in the upstream source).
    // The three active flags provide full Kitty disambiguation and event-type reporting.
    crossterm::execute!(
        stdout(),
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
        ),
        EnableBracketedPaste,
    )?;
    Ok(())
}

/// Remove keyboard enhancement flags and disable bracketed paste at TUI exit.
///
/// Must be called on all TUI exit paths (normal exit and panic hook). Called AFTER the
/// main event loop exits but BEFORE `disable_raw_mode()`.
///
/// Sends:
/// - `crossterm::event::PopKeyboardEnhancementFlags`
/// - `crossterm::event::DisableBracketedPaste`
///
/// Errors are logged at WARN level and swallowed — the terminal is being torn down
/// and there is nothing useful to do with a teardown failure.
pub fn teardown_keyboard_enhancement() {
    use crossterm::event::{DisableBracketedPaste, PopKeyboardEnhancementFlags};
    use std::io::stdout;

    if let Err(e) =
        crossterm::execute!(stdout(), PopKeyboardEnhancementFlags, DisableBracketedPaste,)
    {
        tracing::warn!("teardown_keyboard_enhancement: execute! failed: {e}");
    }
}

/// Dispatch a crossterm `KeyEvent` while `AppMode::EmbeddedTerminal` is active.
///
/// Implements the keyboard forwarding path per BC-2.09.002:
/// 1. Intercepts bare `KeyCode::Esc` (no modifiers) → sends `Action::ExitEmbeddedTerminal`
///    (BEFORE any conversion — BC-2.09.002 Invariant 2).
/// 2. All other Press/Repeat events → `crossterm_key_to_pty(event)` → `key_event_to_pty_bytes(pty_event)`.
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
    ipc_tx: &Sender<ClientToServer>,
) -> bool {
    // BC-2.09.002 Invariant 2: Intercept bare Esc (no modifiers) as ExitEmbeddedTerminal
    // BEFORE calling key_event_to_pty_bytes(). This ordering is non-negotiable.
    if event.code == KeyCode::Esc && event.modifiers == KeyModifiers::NONE {
        // Signal caller to dispatch Action::ExitEmbeddedTerminal.
        // Nothing is sent to the PTY for this Esc.
        return true;
    }

    // Convert crossterm type to core-owned type at the purity boundary.
    let pty_event = crossterm_key_to_pty(event);

    // Translate to PTY bytes. Returns None for Release events and unrecognized keys.
    if let Some(bytes) = key_event_to_pty_bytes(pty_event) {
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
/// `key_event_to_pty_bytes()` (BC-2.09.005 Invariant 2; BC-2.09.010 AC-010).
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
    // BC-2.09.005 PC-2/PC-3: wrap paste text in bracketed paste sequences.
    // Full payload in a single KeyInput — no chunking (BC-2.09.005 Invariant 3).
    let mut bytes = Vec::with_capacity(b"\x1b[200~".len() + text.len() + b"\x1b[201~".len());
    bytes.extend_from_slice(b"\x1b[200~");
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(b"\x1b[201~");

    if let Err(e) = ipc_tx
        .send(ClientToServer::KeyInput {
            session_id: session_id.to_owned(),
            bytes,
        })
        .await
    {
        tracing::warn!("dispatch_embedded_terminal_paste: IPC send failed (channel closed?): {e}");
    }
}
