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

// ---------------------------------------------------------------------------
// Tests — pure monocle-core tests; NO crossterm types allowed here.
// All test names follow test_BC_S_SS_NNN_xxx() pattern (TDD naming convention).
// These tests MUST FAIL until the implementer fills in the todo!() stubs.
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // BC-2.09.002: Full-Fidelity Keyboard Forwarding
    // -----------------------------------------------------------------------

    /// BC-2.09.002 PC-1/PC-2 — printable ASCII character 'a' → [0x61]
    ///
    /// Canonical test vector from BC-2.09.002 §Canonical Test Vectors.
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_printable() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x61]));
    }

    /// BC-2.09.002 PC-2 — Ctrl+C → [0x03] (ETX)
    ///
    /// Canonical test vector from BC-2.09.002 §Canonical Test Vectors.
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_ctrl() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('c'),
            modifiers: PtyKeyModifiers::CONTROL,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x03]));
    }

    /// BC-2.09.002 PC-2 — all four arrow keys produce correct VT sequences
    ///
    /// Arrow Up   → ESC [ A  (\x1b\x5b\x41)
    /// Arrow Down → ESC [ B  (\x1b\x5b\x42)
    /// Arrow Right→ ESC [ C  (\x1b\x5b\x43)
    /// Arrow Left → ESC [ D  (\x1b\x5b\x44)
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_arrows() {
        let cases = [
            (PtyKeyCode::Up, vec![0x1b, 0x5b, 0x41]),
            (PtyKeyCode::Down, vec![0x1b, 0x5b, 0x42]),
            (PtyKeyCode::Right, vec![0x1b, 0x5b, 0x43]),
            (PtyKeyCode::Left, vec![0x1b, 0x5b, 0x44]),
        ];
        for (code, expected) in cases {
            let event = PtyKeyEvent {
                code,
                modifiers: PtyKeyModifiers::NONE,
                kind: PtyKeyEventKind::Press,
            };
            assert_eq!(key_event_to_pty_bytes(event), Some(expected));
        }
    }

    /// BC-2.09.002 PC-2 — F1–F4 → SS3 sequences; F5 → tilde sequence
    ///
    /// F1 → \x1bOP, F2 → \x1bOQ, F3 → \x1bOR, F4 → \x1bOS
    /// F5 → \x1b[15~
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_fn_keys() {
        let cases = [
            (1u8, vec![0x1b, b'O', b'P']),
            (2u8, vec![0x1b, b'O', b'Q']),
            (3u8, vec![0x1b, b'O', b'R']),
            (4u8, vec![0x1b, b'O', b'S']),
            (5u8, vec![0x1b, b'[', b'1', b'5', b'~']),
        ];
        for (n, expected) in cases {
            let event = PtyKeyEvent {
                code: PtyKeyCode::F(n),
                modifiers: PtyKeyModifiers::NONE,
                kind: PtyKeyEventKind::Press,
            };
            assert_eq!(
                key_event_to_pty_bytes(event),
                Some(expected),
                "F{n} key bytes mismatch"
            );
        }
    }

    /// BC-2.09.002 PC-3 — Release events return None; nothing forwarded to PTY
    ///
    /// Canonical test vector from BC-2.09.002 §Canonical Test Vectors.
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_release_discarded() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Release,
        };
        assert_eq!(key_event_to_pty_bytes(event), None);
    }

    /// BC-2.09.002 Invariant 3 — Ctrl+D → [0x04] (ASCII EOT); session termination signal
    ///
    /// Canonical test vector from BC-2.09.002 §Canonical Test Vectors.
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_ctrl_d_eot() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('d'),
            modifiers: PtyKeyModifiers::CONTROL,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x04]));
    }

    /// BC-2.09.002 PC-2 — full Ctrl+[A-Z] translation table
    ///
    /// Ctrl+A → \x01 through Ctrl+Z → \x1a (control characters 1–26).
    #[test]
    fn test_BC_2_09_002_ctrl_az_full_translation_table() {
        for (i, ch) in ('a'..='z').enumerate() {
            let expected_byte = (i + 1) as u8;
            let event = PtyKeyEvent {
                code: PtyKeyCode::Char(ch),
                modifiers: PtyKeyModifiers::CONTROL,
                kind: PtyKeyEventKind::Press,
            };
            assert_eq!(
                key_event_to_pty_bytes(event),
                Some(vec![expected_byte]),
                "Ctrl+{} should produce \\x{:02x}",
                ch,
                expected_byte
            );
        }
    }

    /// BC-2.09.002 PC-2 — Enter → \r (carriage return)
    #[test]
    fn test_BC_2_09_002_enter_maps_to_carriage_return() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Enter,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x0d]));
    }

    /// BC-2.09.002 PC-2 — Backspace → \x7f (DEL)
    #[test]
    fn test_BC_2_09_002_backspace_maps_to_del() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Backspace,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x7f]));
    }

    /// BC-2.09.002 PC-2 — Tab → \t
    #[test]
    fn test_BC_2_09_002_tab_maps_to_horizontal_tab() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Tab,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x09]));
    }

    /// BC-2.09.002 PC-2 — navigation keys (Home/End/PgUp/PgDn/Ins/Del)
    ///
    /// Sequence table from BC-2.09.002 PC-2.
    #[test]
    fn test_BC_2_09_002_navigation_keys_full_table() {
        let cases = [
            (PtyKeyCode::Home, vec![0x1b, b'[', b'H']),
            (PtyKeyCode::End, vec![0x1b, b'[', b'F']),
            (PtyKeyCode::PageUp, vec![0x1b, b'[', b'5', b'~']),
            (PtyKeyCode::PageDown, vec![0x1b, b'[', b'6', b'~']),
            (PtyKeyCode::Insert, vec![0x1b, b'[', b'2', b'~']),
            (PtyKeyCode::Delete, vec![0x1b, b'[', b'3', b'~']),
        ];
        for (code, expected) in cases {
            let event = PtyKeyEvent {
                code,
                modifiers: PtyKeyModifiers::NONE,
                kind: PtyKeyEventKind::Press,
            };
            assert_eq!(key_event_to_pty_bytes(event), Some(expected));
        }
    }

    /// BC-2.09.002 PC-2 — Alt+char → ESC-prefix (ESC + UTF-8 bytes of char)
    ///
    /// Alt+a → \x1b a (\x1b\x61)
    #[test]
    fn test_BC_2_09_002_alt_char_produces_esc_prefix() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::ALT,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x1b, 0x61]));
    }

    /// BC-2.09.002 PC-2 — Shift+Tab (BackTab) → \x1b[Z
    #[test]
    fn test_BC_2_09_002_shift_tab_maps_to_csi_z() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::BackTab,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(
            key_event_to_pty_bytes(event),
            Some(vec![0x1b, b'[', b'Z'])
        );
    }

    /// BC-2.09.002 PC-2 — F6–F12 tilde sequences (full table)
    ///
    /// F6→\x1b[17~, F7→\x1b[18~, F8→\x1b[19~, F9→\x1b[20~,
    /// F10→\x1b[21~, F11→\x1b[23~, F12→\x1b[24~
    #[test]
    fn test_BC_2_09_002_fn_keys_f6_through_f12() {
        let cases = [
            (6u8, b"17"),
            (7u8, b"18"),
            (8u8, b"19"),
            (9u8, b"20"),
            (10u8, b"21"),
            (11u8, b"23"),
            (12u8, b"24"),
        ];
        for (n, num) in cases {
            let expected = {
                let mut v = vec![0x1b, b'['];
                v.extend_from_slice(num.as_ref());
                v.push(b'~');
                v
            };
            let event = PtyKeyEvent {
                code: PtyKeyCode::F(n),
                modifiers: PtyKeyModifiers::NONE,
                kind: PtyKeyEventKind::Press,
            };
            assert_eq!(
                key_event_to_pty_bytes(event),
                Some(expected),
                "F{n} key bytes mismatch"
            );
        }
    }

    /// BC-2.09.002 PC-3/PC-4 — Release events for Release kind return None
    ///
    /// Additional coverage: Release on any key (not just 'a') returns None.
    #[test]
    fn test_BC_2_09_002_release_events_all_keys_return_none() {
        let keys = [
            PtyKeyCode::Enter,
            PtyKeyCode::Up,
            PtyKeyCode::F(1),
            PtyKeyCode::Char('z'),
        ];
        for code in keys {
            let event = PtyKeyEvent {
                code,
                modifiers: PtyKeyModifiers::NONE,
                kind: PtyKeyEventKind::Release,
            };
            assert_eq!(key_event_to_pty_bytes(event), None);
        }
    }

    /// BC-2.09.002 PC-1/PC-2 — Repeat kind is forwarded (same as Press)
    ///
    /// key_event_to_pty_bytes treats Press and Repeat identically.
    #[test]
    fn test_BC_2_09_002_repeat_kind_forwarded_same_as_press() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Repeat,
        };
        assert_eq!(key_event_to_pty_bytes(event), Some(vec![0x61]));
    }

    // -----------------------------------------------------------------------
    // BC-2.09.002 — fn_key_bytes helper
    // -----------------------------------------------------------------------

    /// BC-2.09.002 PC-2 — fn_key_bytes() helper returns correct SS3 sequences for F1–F4
    #[test]
    fn test_BC_2_09_002_fn_key_bytes_ss3_sequences() {
        assert_eq!(fn_key_bytes(1), vec![0x1b, b'O', b'P']);
        assert_eq!(fn_key_bytes(2), vec![0x1b, b'O', b'Q']);
        assert_eq!(fn_key_bytes(3), vec![0x1b, b'O', b'R']);
        assert_eq!(fn_key_bytes(4), vec![0x1b, b'O', b'S']);
    }

    /// BC-2.09.002 PC-2 — fn_key_bytes() helper returns correct tilde sequences for F5–F12
    #[test]
    fn test_BC_2_09_002_fn_key_bytes_tilde_sequences() {
        let cases = [
            (5u8, "15"),
            (6u8, "17"),
            (7u8, "18"),
            (8u8, "19"),
            (9u8, "20"),
            (10u8, "21"),
            (11u8, "23"),
            (12u8, "24"),
        ];
        for (n, num_str) in cases {
            let mut expected = vec![0x1b, b'['];
            expected.extend_from_slice(num_str.as_bytes());
            expected.push(b'~');
            assert_eq!(fn_key_bytes(n), expected, "fn_key_bytes({n}) mismatch");
        }
    }

    // -----------------------------------------------------------------------
    // BC-2.09.004: Kitty Keyboard Protocol
    // -----------------------------------------------------------------------

    /// BC-2.09.004 PC-2 — Ctrl+Shift+Enter → \x1b[13;6u
    ///
    /// Canonical test vector from BC-2.09.004 §Canonical Test Vectors.
    /// Enter codepoint = 13; modifier = 1 + shift(1) + ctrl(4) = 6.
    #[test]
    fn test_BC_2_09_004_kitty_ctrl_shift_enter() {
        let mods = PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT;
        let result = encode_kitty_key(&PtyKeyCode::Enter, mods, PtyKeyEventKind::Press);
        // Expected: ESC [ 1 3 ; 6 u
        assert_eq!(result, b"\x1b[13;6u".to_vec());
    }

    /// BC-2.09.004 PC-4 / EC-228 — non-Kitty terminal: is_kitty_enhanced_key returns false
    ///
    /// On a non-Kitty terminal, PushKeyboardEnhancementFlags silently no-ops.
    /// Standard key events are generated (not enhanced); is_kitty_enhanced_key() returns false.
    /// Standard VT sequences from BC-2.09.002 table are used. No panic.
    #[test]
    fn test_BC_2_09_004_kitty_unsupported_fallback() {
        // On non-Kitty terminals, plain keys don't trigger Kitty encoding.
        assert!(!is_kitty_enhanced_key(&PtyKeyCode::Enter, PtyKeyModifiers::NONE));
        // Even with modifiers, a non-enhanced event (standard terminal) returns false.
        // The standard table handles Enter → \r regardless.
        assert!(!is_kitty_enhanced_key(
            &PtyKeyCode::Enter,
            PtyKeyModifiers::CONTROL
        ));
    }

    /// BC-2.09.004 EC-226 — Shift+Tab (Kitty) → \x1b[9;2u
    ///
    /// Tab codepoint = 9; modifier = 1 + shift(1) = 2.
    #[test]
    fn test_BC_2_09_004_kitty_shift_tab_csi_u() {
        let mods = PtyKeyModifiers::SHIFT;
        let result = encode_kitty_key(&PtyKeyCode::Tab, mods, PtyKeyEventKind::Press);
        // Expected: ESC [ 9 ; 2 u
        assert_eq!(result, b"\x1b[9;2u".to_vec());
    }

    /// BC-2.09.004 Invariant 3 — encode_kitty_key is pure (deterministic)
    ///
    /// Same input always produces same output.
    #[test]
    fn test_BC_2_09_004_kitty_encode_is_pure() {
        let mods = PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT;
        let first = encode_kitty_key(&PtyKeyCode::Enter, mods, PtyKeyEventKind::Press);
        let second = encode_kitty_key(&PtyKeyCode::Enter, mods, PtyKeyEventKind::Press);
        assert_eq!(first, second);
    }

    /// BC-2.09.004 PC-2 — modifier_value formula: 1 + sum(shift=1, alt=2, ctrl=4)
    ///
    /// Alt+Enter → ESC [ 13 ; 3 u  (modifier = 1 + alt(2) = 3)
    #[test]
    fn test_BC_2_09_004_kitty_alt_enter_modifier_value() {
        let mods = PtyKeyModifiers::ALT;
        let result = encode_kitty_key(&PtyKeyCode::Enter, mods, PtyKeyEventKind::Press);
        // Expected: ESC [ 1 3 ; 3 u
        assert_eq!(result, b"\x1b[13;3u".to_vec());
    }

    /// BC-2.09.004 PC-2 — no modifier → modifier_value = 1
    ///
    /// Plain Enter in Kitty context → ESC [ 13 ; 1 u
    #[test]
    fn test_BC_2_09_004_kitty_no_modifier_value_is_one() {
        let result = encode_kitty_key(
            &PtyKeyCode::Enter,
            PtyKeyModifiers::NONE,
            PtyKeyEventKind::Press,
        );
        // Expected: ESC [ 1 3 ; 1 u
        assert_eq!(result, b"\x1b[13;1u".to_vec());
    }
}
