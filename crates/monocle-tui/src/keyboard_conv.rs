//! Crossterm-to-PtyKey and ratatui-to-PtyRect type conversions — the ONLY place in
//! the workspace where crossterm and ratatui types touch the `monocle-core` purity boundary.
//!
//! These conversions are infallible field-by-field copies. No logic; no I/O.
//! Called at the `EmbeddedTerminal` event dispatch site in `event_loop.rs` before
//! calling into `monocle_core::keyboard` functions.
//!
//! **Forbidden:** Adding any `crossterm` or `ratatui` type to `monocle-core/Cargo.toml`
//! is FORBIDDEN (SS-tui.md §Scope, F-P2-I06 ruling). This file is the single conversion
//! seam; all type adaptation happens here and nowhere else.
//!
//! S-041 extends this file (from S-040's base) with `crossterm_mouse_to_pty()` and
//! `ratatui_rect_to_pty()`. These are the only additional crossterm/ratatui types that
//! may appear in this file — confined to the S-041 mouse/rect conversions below.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use monocle_core::keyboard::{
    PtyKeyCode, PtyKeyEvent, PtyKeyEventKind, PtyKeyModifiers, PtyMouseButton, PtyMouseEvent,
    PtyMouseEventKind, PtyRect,
};

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

// ---------------------------------------------------------------------------
// S-041: Mouse and Rect conversions (BC-2.09.003)
// These are the ONLY additional crossterm/ratatui types permitted in this file
// per the F-P2-I06 confined-seam ruling.
// ---------------------------------------------------------------------------

/// Convert a `crossterm::event::MouseEvent` to a `monocle_core::keyboard::PtyMouseEvent`.
///
/// Infallible conversion: every `crossterm::event::MouseEventKind` variant maps to a
/// corresponding `PtyMouseEventKind` variant. Called at the `EmbeddedTerminal` mouse
/// dispatch arm in `event_loop.rs` before calling `mouse_event_to_pty_bytes()`.
///
/// This function is the purity boundary crossing for mouse events: crossterm types
/// remain in `monocle-tui`; `monocle-core` functions see only `PtyMouseEvent`.
pub fn crossterm_mouse_to_pty(e: crossterm::event::MouseEvent) -> PtyMouseEvent {
    PtyMouseEvent {
        kind: crossterm_mouse_kind_to_pty(e.kind),
        column: e.column,
        row: e.row,
        modifiers: crossterm_mouse_mods_to_pty(e.modifiers),
    }
}

/// Convert a `ratatui::layout::Rect` to a `monocle_core::keyboard::PtyRect`.
///
/// Infallible field-by-field copy. Called at the `EmbeddedTerminal` mouse dispatch
/// arm alongside `crossterm_mouse_to_pty()` to supply the pane area for coordinate
/// translation in `mouse_event_to_pty_bytes()`.
///
/// `ratatui::layout::Rect` and `PtyRect` have identical field types (`u16`), so
/// this is a zero-cost structural copy at the purity seam.
pub fn ratatui_rect_to_pty(r: ratatui::layout::Rect) -> PtyRect {
    PtyRect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

// Private helpers for crossterm_mouse_to_pty — kept private to enforce the single-seam
// invariant (crossterm mouse types confined to this file).
fn crossterm_mouse_button_to_pty(b: crossterm::event::MouseButton) -> PtyMouseButton {
    match b {
        crossterm::event::MouseButton::Left => PtyMouseButton::Left,
        crossterm::event::MouseButton::Middle => PtyMouseButton::Middle,
        crossterm::event::MouseButton::Right => PtyMouseButton::Right,
    }
}

fn crossterm_mouse_kind_to_pty(k: crossterm::event::MouseEventKind) -> PtyMouseEventKind {
    match k {
        crossterm::event::MouseEventKind::Down(btn) => {
            PtyMouseEventKind::Down(crossterm_mouse_button_to_pty(btn))
        }
        crossterm::event::MouseEventKind::Up(btn) => {
            PtyMouseEventKind::Up(crossterm_mouse_button_to_pty(btn))
        }
        crossterm::event::MouseEventKind::Drag(btn) => {
            PtyMouseEventKind::Drag(crossterm_mouse_button_to_pty(btn))
        }
        crossterm::event::MouseEventKind::Moved => PtyMouseEventKind::Moved,
        crossterm::event::MouseEventKind::ScrollUp => PtyMouseEventKind::ScrollUp,
        crossterm::event::MouseEventKind::ScrollDown => PtyMouseEventKind::ScrollDown,
        crossterm::event::MouseEventKind::ScrollLeft => PtyMouseEventKind::ScrollLeft,
        crossterm::event::MouseEventKind::ScrollRight => PtyMouseEventKind::ScrollRight,
    }
}

/// Convert crossterm `KeyModifiers` (from a `MouseEvent`) to `PtyKeyModifiers`.
///
/// Mouse events carry the same `KeyModifiers` type as keyboard events in crossterm.
/// The bit remapping is identical to `crossterm_mods_to_pty` for keyboard events.
fn crossterm_mouse_mods_to_pty(m: crossterm::event::KeyModifiers) -> PtyKeyModifiers {
    use crossterm::event::KeyModifiers;
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
