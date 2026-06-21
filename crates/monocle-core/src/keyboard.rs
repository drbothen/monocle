//! Core-owned mirror types and pure keyboard encoding functions for the embedded PTY.
//!
//! **Dependency constraint (F-P2-I06 ruling):** This module MUST NOT depend on `crossterm`,
//! `ratatui`, or any other effectful/TUI crate. All types here are plain Rust enums/structs
//! with no external crate dependencies. Conversion from crossterm types to these types is
//! performed exclusively in `monocle-tui/src/keyboard_conv.rs`.
//!
//! **Functions in this module are pure:** no I/O, no state mutation, deterministic.
//! They are located in `monocle-core` (pure-core crate) per the architecture mapping
//! in S-040 and SS-embedded-pty.md §Core-Owned Mirror Types.

// ---------------------------------------------------------------------------
// Core-owned mirror types
// These mirror crossterm/ratatui fields exactly, but live in monocle-core so
// key_event_to_pty_bytes and related functions remain crossterm/ratatui-free.
// monocle-tui converts at the dispatch boundary (zero-cost field copy of primitives/enums).
// See SS-embedded-pty.md §Core-Owned Mirror Types.
// ---------------------------------------------------------------------------

/// Mirror of `crossterm::event::KeyCode`.
///
/// Variants cover the v1A scope (BC-2.09.002 table). Add variants as BCs expand.
/// `Null` maps unrecognized crossterm keycodes to a no-op result from `key_event_to_pty_bytes`.
///
/// `#[non_exhaustive]` per BC-2.02.003 — v1A scope is explicitly limited; future
/// BCs (e.g., S-041 mouse variants) will extend this enum.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyKeyCode {
    /// A printable Unicode character.
    Char(char),
    /// Enter key.
    Enter,
    /// Backspace key.
    Backspace,
    /// Tab key (forward tab).
    Tab,
    /// Back-tab (Shift+Tab as a distinct keycode on some terminals).
    BackTab,
    /// Escape key.
    Esc,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Function key F(n) where n is 1–12.
    F(u8),
    /// Unrecognized or unsupported key — `key_event_to_pty_bytes` returns `None`.
    Null,
}

/// Mirror of `crossterm::event::KeyModifiers` (bitflags).
///
/// Matches crossterm bit values exactly so `monocle-tui` conversion is a single cast.
/// Only SHIFT, CONTROL, and ALT are used in v1A scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtyKeyModifiers(pub u8);

impl PtyKeyModifiers {
    /// No modifiers active.
    pub const NONE: Self = PtyKeyModifiers(0b0000_0000);
    /// Shift modifier.
    pub const SHIFT: Self = PtyKeyModifiers(0b0000_0001);
    /// Control modifier.
    pub const CONTROL: Self = PtyKeyModifiers(0b0000_0100);
    /// Alt/Meta modifier.
    pub const ALT: Self = PtyKeyModifiers(0b0000_1000);

    /// Returns `true` if all bits in `other` are set in `self`.
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Returns `true` if no modifier bits are set.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for PtyKeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        PtyKeyModifiers(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for PtyKeyModifiers {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        PtyKeyModifiers(self.0 & rhs.0)
    }
}

/// Mirror of `crossterm::event::KeyEventKind`.
///
/// `#[non_exhaustive]` per BC-2.02.003 — crossterm may add new event kinds in
/// future versions; new variants would be mapped to `PtyKeyEventKind::Release` as
/// a safe default until a BC update formalizes handling.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyKeyEventKind {
    /// Key press (initial down event).
    Press,
    /// Key held down, auto-repeating.
    Repeat,
    /// Key release (only emitted with Kitty keyboard enhancement enabled).
    Release,
}

/// Mirror of `crossterm::event::KeyEvent`.
///
/// Passed to `key_event_to_pty_bytes()` after conversion from crossterm types in
/// `monocle-tui/src/keyboard_conv.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyKeyEvent {
    /// The key that was pressed, repeated, or released.
    pub code: PtyKeyCode,
    /// Active modifier keys at the time of the event.
    pub modifiers: PtyKeyModifiers,
    /// Whether this is a press, repeat, or release event.
    pub kind: PtyKeyEventKind,
}

/// Mirror of `crossterm::event::MouseButton`.
///
/// `#[non_exhaustive]` per BC-2.02.003 — future mouse devices may report additional
/// button types beyond left/middle/right.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyMouseButton {
    /// Primary (left) mouse button.
    Left,
    /// Middle (scroll-wheel click) mouse button.
    Middle,
    /// Secondary (right) mouse button.
    Right,
}

/// Mirror of `crossterm::event::MouseEventKind` (v1A Ps table scope).
///
/// `#[non_exhaustive]` per BC-2.02.003 — v1A scope is limited to the 8 variants below;
/// future BCs may extend the SGR encoding table with additional event kinds.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyMouseEventKind {
    /// A mouse button was pressed.
    Down(PtyMouseButton),
    /// A mouse button was released.
    Up(PtyMouseButton),
    /// The mouse was moved while a button was held.
    Drag(PtyMouseButton),
    /// The mouse was moved without any button held (requires any-event tracking mode 1003).
    Moved,
    /// The scroll wheel was rotated up.
    ScrollUp,
    /// The scroll wheel was rotated down.
    ScrollDown,
    /// Horizontal scroll left.
    ScrollLeft,
    /// Horizontal scroll right.
    ScrollRight,
}

/// Mirror of `crossterm::event::MouseEvent` fields used by `mouse_event_to_pty_bytes`.
///
/// `monocle-core` does NOT depend on crossterm. `monocle-tui` converts
/// `crossterm::event::MouseEvent` → `PtyMouseEvent` at the event dispatch site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyMouseEvent {
    /// The kind of mouse event (button, scroll, motion).
    pub kind: PtyMouseEventKind,
    /// Terminal column of the event (0-indexed).
    pub column: u16,
    /// Terminal row of the event (0-indexed).
    pub row: u16,
    /// Active modifier keys at the time of the mouse event.
    pub modifiers: PtyKeyModifiers,
}

/// Minimal pane area rectangle — mirrors `ratatui::layout::Rect` fields.
///
/// `monocle-core` does NOT depend on ratatui. `monocle-tui` converts
/// `ratatui::layout::Rect` → `PtyRect` at the event dispatch site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtyRect {
    /// Left edge column (0-indexed).
    pub x: u16,
    /// Top edge row (0-indexed).
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

// ---------------------------------------------------------------------------
// Pure keyboard encoding functions
// ---------------------------------------------------------------------------

/// Translate a `PtyKeyEvent` to the terminal byte sequence for PTY stdin.
///
/// Returns `None` for events that MUST NOT be forwarded:
/// - `PtyKeyEventKind::Release` events (BC-2.09.002 PC-3).
/// - Pure modifier-only key events (BC-2.09.002 PC-4).
/// - Unrecognized keycodes (`PtyKeyCode::Null`).
///
/// The caller (Action dispatch in `monocle-tui/src/event_loop.rs`) MUST intercept
/// `PtyKeyCode::Esc` (no modifiers) as `Action::ExitEmbeddedTerminal` BEFORE calling
/// this function (BC-2.09.002 Invariant 2).
///
/// # Purity
///
/// This function is pure: no I/O, no state mutation, deterministic (BC-2.09.002 Invariant 1).
///
/// # Parameters
///
/// - `event`: A `PtyKeyEvent` (core-owned type). `monocle-tui` converts
///   `crossterm::event::KeyEvent` → `PtyKeyEvent` via `keyboard_conv::crossterm_key_to_pty()`
///   before calling this function. See SS-embedded-pty.md §Dependency Boundary (F-P2-I06).
#[allow(clippy::todo)]
pub fn key_event_to_pty_bytes(_event: PtyKeyEvent) -> Option<Vec<u8>> {
    todo!("S-040: implement key_event_to_pty_bytes per BC-2.09.002 PC-2 translation table")
}

/// Returns `true` if `(code, mods)` should be encoded as a Kitty CSI u sequence
/// rather than a standard VT byte sequence.
///
/// Called by the TUI dispatch arm before `key_event_to_pty_bytes` to determine whether
/// Kitty-enhanced encoding applies. Returns `false` on terminals that do not support
/// Kitty keyboard protocol (the `PushKeyboardEnhancementFlags` command silently no-ops
/// on unsupported terminals, so enhanced `KeyEvent` variants are never generated —
/// `is_kitty_enhanced_key` will naturally return `false` for all observed events).
///
/// # Purity
///
/// This function is pure: no I/O, no state mutation (BC-2.09.004 Invariant 3).
#[allow(clippy::todo)]
pub fn is_kitty_enhanced_key(_code: &PtyKeyCode, _mods: PtyKeyModifiers) -> bool {
    todo!("S-040: implement is_kitty_enhanced_key per BC-2.09.004")
}

/// Encode a Kitty keyboard protocol key event as a CSI u byte sequence.
///
/// Produces: `ESC [ <unicode_codepoint> ; <modifier_value> u`
///
/// where:
/// - `<unicode_codepoint>` is the decimal Unicode codepoint of the key.
/// - `<modifier_value>` = `1 + sum(active bits: Shift=1, Alt=2, Ctrl=4)`.
///
/// Example: `Ctrl+Shift+Enter` → `\x1b[13;6u`
/// (Enter codepoint = 13; modifier = 1 + shift(1) + ctrl(4) = 6)
///
/// Called only when `is_kitty_enhanced_key(code, mods)` returns `true`.
///
/// # Purity
///
/// This function is pure: no I/O, no state mutation (BC-2.09.004 Invariant 3).
#[allow(clippy::todo)]
pub fn encode_kitty_key(
    _code: &PtyKeyCode,
    _mods: PtyKeyModifiers,
    _kind: PtyKeyEventKind,
) -> Vec<u8> {
    todo!("S-040: implement encode_kitty_key CSI u sequence per BC-2.09.004")
}

/// Return the PTY byte sequence for function key F(n), n ∈ 1..=12.
///
/// Authoritative mapping per BC-2.09.002 PC-2 table:
/// - F1 → `\x1bOP`, F2 → `\x1bOQ`, F3 → `\x1bOR`, F4 → `\x1bOS`
/// - F5 → `\x1b[15~`, F6 → `\x1b[17~`, F7 → `\x1b[18~`, F8 → `\x1b[19~`
/// - F9 → `\x1b[20~`, F10 → `\x1b[21~`, F11 → `\x1b[23~`, F12 → `\x1b[24~`
///
/// Called by `key_event_to_pty_bytes` for `PtyKeyCode::F(n)` variants.
///
/// # Purity
///
/// This function is pure: no I/O, no state mutation.
#[allow(clippy::todo)]
pub fn fn_key_bytes(_n: u8) -> Vec<u8> {
    todo!("S-040: implement fn_key_bytes per BC-2.09.002 PC-2 F-key table")
}
