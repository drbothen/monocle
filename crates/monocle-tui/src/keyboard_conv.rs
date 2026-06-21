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
//! in that story's scope. The function stubs for mouse and rect are intentionally
//! absent from this file to keep S-040 stub scope minimal.

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
#[allow(clippy::todo)]
pub fn crossterm_key_to_pty(_e: KeyEvent) -> PtyKeyEvent {
    todo!("S-040: implement crossterm_key_to_pty field-by-field conversion")
}

// ---------------------------------------------------------------------------
// Private conversion helpers
// These helpers are called from crossterm_key_to_pty() once the implementer
// fills in the real logic. dead_code is suppressed here because the stubs are
// structurally present for the implementer to call from the public function above.
// ---------------------------------------------------------------------------

#[allow(clippy::todo, dead_code)]
fn crossterm_keycode_to_pty(_c: KeyCode) -> PtyKeyCode {
    todo!("S-040: implement crossterm_keycode_to_pty match")
}

#[allow(clippy::todo, dead_code)]
fn crossterm_mods_to_pty(_m: KeyModifiers) -> PtyKeyModifiers {
    todo!("S-040: implement crossterm_mods_to_pty bitflag mapping")
}

#[allow(clippy::todo, dead_code)]
fn crossterm_kind_to_pty(_k: KeyEventKind) -> PtyKeyEventKind {
    todo!("S-040: implement crossterm_kind_to_pty match")
}
