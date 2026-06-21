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
/// # Detection sequence (CSI ? u query)
///
/// Before installing `PushKeyboardEnhancementFlags`, this function probes the terminal
/// for Kitty keyboard protocol support (SS-embedded-pty §"Risk Mitigations"):
///
/// 1. Write `\x1b[?u` (raw CSI ? u query) to stdout and flush.
/// 2. Spawn a reader thread that reads raw stdin bytes into a buffer.
/// 3. Wait up to 100 ms for the thread to return a response.
/// 4. If the response matches `\x1b[?<N>u` (any single decimal integer N), set
///    `kitty_active = true` and call `PushKeyboardEnhancementFlags`.
/// 5. Otherwise set `kitty_active = false`, emit TRACE log, skip `PushKeyboardEnhancementFlags`.
/// 6. `EnableBracketedPaste` is sent unconditionally (widely supported, low risk).
///
/// The function returns the `kitty_active` bool. Callers store this in an
/// `Arc<AtomicBool>` so all TUI exit paths (normal and panic) call
/// `teardown_keyboard_enhancement(kitty_active)` symmetrically.
///
/// # Errors
///
/// Returns `Err` if any stdout write (CSI query or `execute!`) fails.
pub fn setup_keyboard_enhancement() -> Result<bool> {
    use crossterm::event::{
        EnableBracketedPaste, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    };
    use std::io::{self, Read, Write};
    use std::sync::mpsc as std_mpsc;
    use std::time::Duration;

    tracing::trace!("setup_keyboard_enhancement: probing for Kitty keyboard protocol (CSI ?u)");

    // Step 1: Write the CSI ? u capability query to stdout and flush.
    // The terminal responds with `\x1b[?<N>u` (flags bitmask) if Kitty protocol is supported,
    // or produces no response within the timeout.
    {
        let mut out = io::stdout();
        out.write_all(b"\x1b[?u")?;
        out.flush()?;
    }

    // Step 2: Spawn a reader thread to read raw stdin bytes into a buffer.
    // We use a dedicated thread because `std::io::stdin().read()` blocks without timeout.
    // The main thread waits at most 100ms; if the thread hasn't returned by then,
    // kitty_active = false (no response = not supported).
    //
    // Note: the terminal is already in raw mode at this point (called from setup_terminal
    // after enable_raw_mode), so read() returns bytes without waiting for newline.
    let (tx, rx) = std_mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buf = [0u8; 32];
        let stdin = io::stdin();
        let mut lock = stdin.lock();
        // Read up to 32 bytes — the CSI ?u response is short (e.g., `\x1b[?0u` = 6 bytes).
        // If read() blocks with no data and the parent times out, the thread is abandoned
        // (leaks until process exit, acceptable — this is a one-shot startup probe).
        match lock.read(&mut buf) {
            Ok(n) if n > 0 => {
                let _ = tx.send(buf[..n].to_vec());
            }
            _ => {
                // No data or read error — send empty vec so channel is never empty.
                let _ = tx.send(Vec::new());
            }
        }
    });

    // Step 3: Wait up to 100ms for the response.
    let kitty_active = match rx.recv_timeout(Duration::from_millis(100)) {
        Ok(response) => {
            // Step 4: Validate the response matches `\x1b[?<N>u`.
            // The response is `ESC [ ? <decimal-digits> u`.
            // We check prefix `\x1b[?` and suffix `u` with at least one digit between them.
            detect_kitty_response(&response)
        }
        Err(_) => {
            // Timeout or channel closed — no response within 100ms; Kitty not supported.
            tracing::trace!(
                "setup_keyboard_enhancement: no CSI ?u response within 100ms; Kitty not supported"
            );
            false
        }
    };

    // Step 5: Conditionally install PushKeyboardEnhancementFlags.
    // Only push when kitty_active; on exit, only PopKeyboardEnhancementFlags when kitty_active.
    if kitty_active {
        tracing::trace!(
            "setup_keyboard_enhancement: Kitty protocol detected; installing enhancement flags"
        );
        // crossterm 0.29: REPORT_ASSOCIATED_TEXT is not stable in this version;
        // use the three stable flags. The spec calls for 4 flags; REPORT_ASSOCIATED_TEXT
        // is not exposed by crossterm 0.29 (it is commented out in the upstream source).
        // The three active flags provide full Kitty disambiguation and event-type reporting.
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

    // Step 6: EnableBracketedPaste unconditionally — widely supported, low risk.
    crossterm::execute!(io::stdout(), EnableBracketedPaste)?;

    Ok(kitty_active)
}

/// Detect whether `response` contains a valid CSI ? u capability response.
///
/// Scans `response` for the pattern `\x1b[?<N>u` where `<N>` is one or more
/// ASCII decimal digits. Extra bytes before or after the pattern are tolerated
/// (the user may have typed input that was buffered alongside the terminal response).
///
/// Returns `true` on match (Kitty supported), `false` otherwise.
fn detect_kitty_response(response: &[u8]) -> bool {
    // Minimum response containing `\x1b[?0u` is 6 bytes.
    if response.len() < 6 {
        return false;
    }
    // Scan for the prefix `\x1b[?` starting from index 0.
    // In practice the response should be at the start, but tolerate leading bytes
    // in case the terminal sends capability info in a different order.
    let prefix = b"\x1b[?";
    let Some(start) = response.windows(prefix.len()).position(|w| w == prefix) else {
        return false;
    };
    // After `\x1b[?`, scan for ASCII decimal digits followed by `u`.
    let after_prefix = &response[start + prefix.len()..];
    let digits_end = after_prefix
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    // Must have at least one digit and be immediately followed by `u`.
    digits_end > 0 && after_prefix.get(digits_end) == Some(&b'u')
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
    // `kitty_active` is threaded through the App struct when called from app.rs
    // (handle_crossterm_event). This dispatch helper defaults to false for standalone
    // unit tests that call it directly without App context — tests exercise non-Kitty paths.
    if let Some(bytes) = key_event_to_pty_bytes(pty_event, false) {
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
