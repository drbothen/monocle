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
use crossterm::event::KeyEvent;
use monocle_ipc::types::ClientToServer;
use tokio::sync::mpsc::Sender;

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
#[allow(clippy::todo)]
pub fn setup_keyboard_enhancement() -> Result<()> {
    todo!("S-040: setup PushKeyboardEnhancementFlags + EnableBracketedPaste at TUI startup (BC-2.09.004 INV-1 / BC-2.09.005 INV-1)")
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
#[allow(clippy::todo)]
pub fn teardown_keyboard_enhancement() {
    todo!("S-040: teardown PopKeyboardEnhancementFlags + DisableBracketedPaste at TUI exit (BC-2.09.004 PC-5 / BC-2.09.005 INV-4)")
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
#[allow(clippy::todo)]
pub async fn dispatch_embedded_terminal_key(
    _event: KeyEvent,
    _session_id: &str,
    _ipc_tx: &Sender<ClientToServer>,
) -> bool {
    todo!("S-040: implement EmbeddedTerminal key dispatch arm: Esc intercept + crossterm_key_to_pty + key_event_to_pty_bytes + KeyInput IPC send (BC-2.09.002)")
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
#[allow(clippy::todo)]
pub async fn dispatch_embedded_terminal_paste(
    _text: &str,
    _session_id: &str,
    _ipc_tx: &Sender<ClientToServer>,
) {
    todo!("S-040: implement bracketed paste dispatch: wrap \\x1b[200~<text>\\x1b[201~ + KeyInput IPC send (BC-2.09.005)")
}
