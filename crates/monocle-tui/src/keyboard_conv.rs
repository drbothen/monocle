//! Crossterm-to-PtyKey type conversions — the ONLY place in the workspace where
//! crossterm and ratatui types touch the `monocle-core` purity boundary.
//!
//! These conversions are infallible field-by-field copies. No logic; no I/O.
//! Called at the `EmbeddedTerminal` event dispatch site in `event_loop.rs` before
//! calling into `monocle_core::keyboard` functions.
//!
//! **Forbidden:** Adding any `crossterm` or `ratatui` type to `monocle-core/Cargo.toml`
//! is FORBIDDEN (SS-tui.md §Scope, F-P2-I06 ruling). This file is the single conversion
//! seam; all type adaptation happens here and nowhere else.
//!
//! S-041 extends this file with `crossterm_mouse_to_pty()` and `ratatui_rect_to_pty()`
//! in that story's scope. Those functions are intentionally absent here — mouse and
//! rect conversion is out of S-040 scope.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use monocle_core::keyboard::{PtyKeyCode, PtyKeyEvent, PtyKeyEventKind, PtyKeyModifiers};

/// Convert a `crossterm::event::KeyEvent` to a `monocle_core::keyboard::PtyKeyEvent`.
///
/// This is an infallible conversion: every `crossterm::event::KeyCode` variant maps
/// to either a recognized `PtyKeyCode` or `PtyKeyCode::Null` (for keycodes outside
/// the v1A scope — `key_event_to_pty_bytes` will return `None` for `Null` keycodes).
///
/// Called by the `EmbeddedTerminal` keyboard dispatch arm in `event_loop.rs` as the
/// type boundary crossing: crossterm types remain in `monocle-tui`; `monocle-core`
/// functions see only `PtyKeyEvent`.
pub fn crossterm_key_to_pty(e: KeyEvent) -> PtyKeyEvent {
    PtyKeyEvent {
        code: crossterm_keycode_to_pty(e.code),
        modifiers: crossterm_mods_to_pty(e.modifiers),
        kind: crossterm_kind_to_pty(e.kind),
    }
}

// ---------------------------------------------------------------------------
// Private conversion helpers
// ---------------------------------------------------------------------------

fn crossterm_keycode_to_pty(c: KeyCode) -> PtyKeyCode {
    match c {
        KeyCode::Char(ch) => PtyKeyCode::Char(ch),
        KeyCode::Enter => PtyKeyCode::Enter,
        KeyCode::Backspace => PtyKeyCode::Backspace,
        KeyCode::Tab => PtyKeyCode::Tab,
        KeyCode::BackTab => PtyKeyCode::BackTab,
        KeyCode::Esc => PtyKeyCode::Esc,
        KeyCode::Delete => PtyKeyCode::Delete,
        KeyCode::Insert => PtyKeyCode::Insert,
        KeyCode::Up => PtyKeyCode::Up,
        KeyCode::Down => PtyKeyCode::Down,
        KeyCode::Left => PtyKeyCode::Left,
        KeyCode::Right => PtyKeyCode::Right,
        KeyCode::Home => PtyKeyCode::Home,
        KeyCode::End => PtyKeyCode::End,
        KeyCode::PageUp => PtyKeyCode::PageUp,
        KeyCode::PageDown => PtyKeyCode::PageDown,
        KeyCode::F(n) => PtyKeyCode::F(n),
        // All unrecognized keycodes map to Null; key_event_to_pty_bytes returns None for Null.
        _ => PtyKeyCode::Null,
    }
}

fn crossterm_mods_to_pty(m: KeyModifiers) -> PtyKeyModifiers {
    let mut bits = 0u8;
    if m.contains(KeyModifiers::SHIFT) {
        bits |= PtyKeyModifiers::SHIFT.0;
    }
    if m.contains(KeyModifiers::CONTROL) {
        bits |= PtyKeyModifiers::CONTROL.0;
    }
    if m.contains(KeyModifiers::ALT) {
        bits |= PtyKeyModifiers::ALT.0;
    }
    PtyKeyModifiers(bits)
}

fn crossterm_kind_to_pty(k: KeyEventKind) -> PtyKeyEventKind {
    match k {
        KeyEventKind::Press => PtyKeyEventKind::Press,
        KeyEventKind::Repeat => PtyKeyEventKind::Repeat,
        KeyEventKind::Release => PtyKeyEventKind::Release,
    }
}
