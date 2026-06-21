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
/// future versions. Because the enum is `#[non_exhaustive]`, the compiler requires
/// an exhaustive match with a wildcard arm (`_`), making any unhandled future variant
/// a compile-time error rather than a silent runtime behaviour change. Callers that
/// pattern-match on this type MUST handle the wildcard arm explicitly.
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
/// - `kitty_active`: `true` if the terminal negotiated Kitty keyboard protocol at startup
///   (set by the `CSI ? u` query in `event_loop::setup_keyboard_enhancement`). When `true`,
///   modifier combos not otherwise matched route to `encode_kitty_key` via the Kitty
///   catch-all arm. When `false`, the VT-fallback table is used.
pub fn key_event_to_pty_bytes(event: PtyKeyEvent, kitty_active: bool) -> Option<Vec<u8>> {
    // BC-2.09.002 PC-3: Release events are discarded — not forwarded to PTY.
    if event.kind == PtyKeyEventKind::Release {
        return None;
    }

    let mods = event.modifiers;

    match event.code {
        // -----------------------------------------------------------------------
        // Arm 1: Unmodified specific keys — matched unconditionally (mods.is_empty()
        // guards prevent false matches on modifier combos of the same keycode).
        // These are the authoritative VT byte sequences per BC-2.09.002 PC-2 table.
        // They are placed FIRST so that named keys always encode correctly on any
        // terminal — the Kitty catch-all arm below only fires for unmatched combos.
        // -----------------------------------------------------------------------

        // Printable characters with no modifier → UTF-8 bytes of the character.
        PtyKeyCode::Char(c) if mods.is_empty() => Some(c.to_string().into_bytes()),

        // -----------------------------------------------------------------------
        // Arm 2: Ctrl+printable → control bytes \x00–\x1f (arm 1 didn't match
        // because mods is non-empty here). Handles Ctrl+@ → \x00 (NUL) and
        // Ctrl+[ → \x1b (Esc) as special cases of the formula c - '@'.
        // -----------------------------------------------------------------------
        PtyKeyCode::Char(c)
            if mods.contains(PtyKeyModifiers::CONTROL) && !mods.contains(PtyKeyModifiers::ALT) =>
        {
            let ctrl_byte = (c.to_ascii_uppercase() as u8).wrapping_sub(b'@');
            if ctrl_byte <= 31 {
                Some(vec![ctrl_byte])
            } else {
                None
            }
        }

        // Special keys (BC-2.09.002 PC-2 table) — matched unconditionally.
        // Enter/Backspace/Tab/Esc have no modifier variants in the BC-2.09.002 VT table;
        // modifier combos (e.g. Ctrl+Enter) fall through to the Kitty arm below.
        PtyKeyCode::Enter if mods.is_empty() => Some(b"\r".to_vec()),
        PtyKeyCode::Backspace if mods.is_empty() => Some(b"\x7f".to_vec()),
        PtyKeyCode::Tab if mods.is_empty() => Some(b"\t".to_vec()),
        // Esc reached here means the dispatch layer forwarded it (not the first Esc).
        // Per BC-2.09.002 Invariant 2: the first bare Esc is intercepted BEFORE this
        // function; reaching here means the dispatch layer has already handled the exit.
        // In practice key_event_to_pty_bytes should NOT be called with Esc (the dispatch
        // layer consumes it), but if it is, forward as \x1b per the BC table.
        PtyKeyCode::Esc if mods.is_empty() => Some(b"\x1b".to_vec()),

        // Arrow keys (BC-2.09.002 PC-2 table) — unmodified only.
        // Modified arrows are handled by VT-fallback arms below (or Kitty arm if active).
        PtyKeyCode::Up if mods.is_empty() => Some(b"\x1b[A".to_vec()),
        PtyKeyCode::Down if mods.is_empty() => Some(b"\x1b[B".to_vec()),
        PtyKeyCode::Right if mods.is_empty() => Some(b"\x1b[C".to_vec()),
        PtyKeyCode::Left if mods.is_empty() => Some(b"\x1b[D".to_vec()),

        // Navigation keys (BC-2.09.002 PC-2 table).
        PtyKeyCode::Home if mods.is_empty() => Some(b"\x1b[H".to_vec()),
        PtyKeyCode::End if mods.is_empty() => Some(b"\x1b[F".to_vec()),
        PtyKeyCode::PageUp if mods.is_empty() => Some(b"\x1b[5~".to_vec()),
        PtyKeyCode::PageDown if mods.is_empty() => Some(b"\x1b[6~".to_vec()),
        PtyKeyCode::Insert if mods.is_empty() => Some(b"\x1b[2~".to_vec()),
        PtyKeyCode::Delete if mods.is_empty() => Some(b"\x1b[3~".to_vec()),

        // Function keys F1–F12 (BC-2.09.002 PC-2 table) — unmodified.
        PtyKeyCode::F(n) if mods.is_empty() => Some(fn_key_bytes(n)),

        // -----------------------------------------------------------------------
        // Arm 3: Kitty catch-all — fires for modifier-carrying combos when
        // kitty_active=true. `is_kitty_enhanced_key` returns true when
        // kitty_active=true AND mods is non-empty AND code is not Null.
        // This arm is placed AFTER all named VT arms so that unmodified keys and
        // Ctrl+printable never reach it. On non-Kitty terminals (kitty_active=false),
        // is_kitty_enhanced_key returns false and this arm is skipped entirely.
        // SS-embedded-pty.md §Translation function S2-002 / HIGH-001 ruling.
        // -----------------------------------------------------------------------
        ref code if is_kitty_enhanced_key(code, mods, kitty_active) => {
            Some(encode_kitty_key(code, mods, event.kind))
        }

        // -----------------------------------------------------------------------
        // Arm 4: Alt+printable → ESC prefix (standard xterm Alt encoding).
        // Only reached on non-Kitty terminals (arm 3 fires first when kitty_active=true).
        // -----------------------------------------------------------------------
        PtyKeyCode::Char(c) if mods.contains(PtyKeyModifiers::ALT) => {
            let mut bytes = vec![b'\x1b'];
            bytes.extend_from_slice(c.to_string().as_bytes());
            Some(bytes)
        }

        // -----------------------------------------------------------------------
        // Arm 5: Shift+Tab (BackTab keycode).
        // Only reached on non-Kitty terminals (arm 3 fires first when kitty_active=true).
        // -----------------------------------------------------------------------
        PtyKeyCode::BackTab => Some(b"\x1b[Z".to_vec()),

        // -----------------------------------------------------------------------
        // Arm 6: VT-fallback modified arrows for non-Kitty terminals.
        // Standard xterm modifier encoding: CSI 1;<mod+1><arrow>.
        // Shift=2, Ctrl=5, Shift+Ctrl=6, Alt=3, etc.
        // On Kitty terminals (kitty_active=true), arm 3 fires first — intentional.
        // -----------------------------------------------------------------------
        PtyKeyCode::Up if mods.contains(PtyKeyModifiers::CONTROL) => Some(b"\x1b[1;5A".to_vec()),
        PtyKeyCode::Down if mods.contains(PtyKeyModifiers::CONTROL) => Some(b"\x1b[1;5B".to_vec()),
        PtyKeyCode::Right if mods.contains(PtyKeyModifiers::CONTROL) => Some(b"\x1b[1;5C".to_vec()),
        PtyKeyCode::Left if mods.contains(PtyKeyModifiers::CONTROL) => Some(b"\x1b[1;5D".to_vec()),
        PtyKeyCode::Up if mods.contains(PtyKeyModifiers::SHIFT) => Some(b"\x1b[1;2A".to_vec()),
        PtyKeyCode::Down if mods.contains(PtyKeyModifiers::SHIFT) => Some(b"\x1b[1;2B".to_vec()),
        PtyKeyCode::Right if mods.contains(PtyKeyModifiers::SHIFT) => Some(b"\x1b[1;2C".to_vec()),
        PtyKeyCode::Left if mods.contains(PtyKeyModifiers::SHIFT) => Some(b"\x1b[1;2D".to_vec()),

        // -----------------------------------------------------------------------
        // Arm 7: EC-217 TRACE+None — unrecognized modifier combo on non-Kitty terminal.
        // Modifier combos with no VT encoding and no Kitty path emit a TRACE log and
        // return None. This is the best-effort boundary per BC-2.09.002 PC-1: NOT a
        // silent drop — the TRACE makes it observable. On Kitty terminals this arm is
        // unreachable for modifier combos (arm 3 catches all non-empty mods).
        // -----------------------------------------------------------------------
        _ if !mods.is_empty() => {
            tracing::trace!(
                code = ?event.code,
                mods = ?mods,
                "key_event_to_pty_bytes: no VT encoding for modifier combo on non-Kitty terminal; dropping"
            );
            None
        }

        // -----------------------------------------------------------------------
        // Arm 8: unrecognized key with no modifiers (PtyKeyCode::Null and anything
        // else not matched above — e.g. Esc with modifiers falls here via arm 7).
        // -----------------------------------------------------------------------
        _ => None,
    }
}

/// Returns `true` if `(code, mods)` should be encoded as a Kitty CSI u sequence
/// rather than a standard VT byte sequence.
///
/// # Design (BC-2.09.004 PC-1 / SS-embedded-pty.md §Translation function S2-002)
///
/// crossterm-0.29 has NO Kitty-specific `KeyCode` variants — every key arrives as the
/// same `KeyCode::Enter` / `KeyCode::Up` / `KeyCode::Char(c)` etc. regardless of
/// whether Kitty protocol was negotiated. A pure function over `(code, mods)` therefore
/// CANNOT know whether the terminal negotiated Kitty protocol; the `kitty_active: bool`
/// parameter carries that information explicitly (set at TUI startup from the `CSI ? u`
/// query result stored in `App::kitty_active`).
///
/// **Return value:**
/// - `false` immediately when `!kitty_active` (early-return guard — non-Kitty terminal).
/// - `false` immediately when `mods.is_empty()` (unmodified keys are already handled
///   by the named VT arms in `key_event_to_pty_bytes`).
/// - `false` for `PtyKeyCode::Null` (unrecognized key — no CSI u encoding).
/// - `true` for all other `(code, mods)` combinations when `kitty_active = true`.
///   This includes modifier variants of Enter, Tab, Backspace, Esc, arrows, navigation
///   keys, and Fn keys that are not covered by the named VT arms.
///
/// # Purity
///
/// This function is pure: no I/O, no state mutation (BC-2.09.004 Invariant 3).
pub fn is_kitty_enhanced_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kitty_active: bool) -> bool {
    // BC-2.09.004 PC-4 / EC-216: early-return guards.
    // When kitty_active=false (non-Kitty terminal), ALWAYS return false.
    // When mods.is_empty(), unmodified keys are already covered by named VT arms
    // in key_event_to_pty_bytes — no CSI u encoding needed.
    if !kitty_active || mods.is_empty() {
        return false;
    }
    // Null key has no CSI u encoding (no codepoint to encode).
    !matches!(code, PtyKeyCode::Null)
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
pub fn encode_kitty_key(
    code: &PtyKeyCode,
    mods: PtyKeyModifiers,
    _kind: PtyKeyEventKind,
) -> Vec<u8> {
    // Unicode codepoint for the key.
    let codepoint: u32 = pty_key_codepoint(code);

    // Modifier value = 1 + sum of active modifier bits.
    // Shift=1, Alt=2, Ctrl=4 (BC-2.09.004 PC-2 canonical bitfield).
    let mut mod_bits: u32 = 0;
    if mods.contains(PtyKeyModifiers::SHIFT) {
        mod_bits += 1;
    }
    if mods.contains(PtyKeyModifiers::ALT) {
        mod_bits += 2;
    }
    if mods.contains(PtyKeyModifiers::CONTROL) {
        mod_bits += 4;
    }
    let modifier_value = 1 + mod_bits;

    // CSI u sequence: ESC [ <codepoint> ; <modifier_value> u
    format!("\x1b[{};{}u", codepoint, modifier_value).into_bytes()
}

/// Return the Unicode codepoint for a `PtyKeyCode` (used in Kitty CSI u encoding).
fn pty_key_codepoint(code: &PtyKeyCode) -> u32 {
    match code {
        PtyKeyCode::Char(c) => *c as u32,
        PtyKeyCode::Enter => 13,      // CR
        PtyKeyCode::Tab => 9,         // HT
        PtyKeyCode::BackTab => 9,     // same codepoint as Tab; Shift bit distinguishes
        PtyKeyCode::Backspace => 127, // DEL
        PtyKeyCode::Esc => 27,
        PtyKeyCode::Up => 65,       // 'A'
        PtyKeyCode::Down => 66,     // 'B'
        PtyKeyCode::Right => 67,    // 'C'
        PtyKeyCode::Left => 68,     // 'D'
        PtyKeyCode::Home => 72,     // 'H'
        PtyKeyCode::End => 70,      // 'F'
        PtyKeyCode::PageUp => 53,   // '5' (tilde sequences use numeric codes)
        PtyKeyCode::PageDown => 54, // '6'
        PtyKeyCode::Insert => 50,   // '2'
        PtyKeyCode::Delete => 51,   // '3'
        // Function keys: standard VT codepoints for CSI u.
        // These are the standard decimal codepoints used by Kitty for F-keys.
        PtyKeyCode::F(n) => match n {
            1 => 57344,
            2 => 57345,
            3 => 57346,
            4 => 57347,
            5 => 57348,
            6 => 57349,
            7 => 57350,
            8 => 57351,
            9 => 57352,
            10 => 57353,
            11 => 57354,
            12 => 57355,
            _ => 0,
        },
        PtyKeyCode::Null => 0,
    }
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
pub fn fn_key_bytes(n: u8) -> Vec<u8> {
    match n {
        // F1–F4: SS3 sequences (Application Cursor Key format).
        1 => b"\x1bOP".to_vec(),
        2 => b"\x1bOQ".to_vec(),
        3 => b"\x1bOR".to_vec(),
        4 => b"\x1bOS".to_vec(),
        // F5–F12: VT tilde sequences.
        5 => b"\x1b[15~".to_vec(),
        6 => b"\x1b[17~".to_vec(),
        7 => b"\x1b[18~".to_vec(),
        8 => b"\x1b[19~".to_vec(),
        9 => b"\x1b[20~".to_vec(),
        10 => b"\x1b[21~".to_vec(),
        11 => b"\x1b[23~".to_vec(),
        12 => b"\x1b[24~".to_vec(),
        // Unknown F-key: return empty (caller should not reach here in v1A).
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests — pure monocle-core tests; NO crossterm types allowed here.
// All test names follow test_BC_S_SS_NNN_xxx() pattern (TDD naming convention).
// These tests MUST FAIL until the implementer updates the production signatures.
//
// RED GATE: All calls to key_event_to_pty_bytes and is_kitty_enhanced_key use
// the NEW 2-arg / 3-arg signatures. The production code still has the OLD
// 1-arg / 2-arg signatures. This causes a compile error in monocle-core's
// test target — that compile error IS the Red Gate for the new-signature tests.
// The implementer must update the production signatures to make these compile.
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
    /// kitty_active=false (non-Kitty terminal; plain printable chars are VT-invariant).
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_printable() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x61]));
    }

    /// BC-2.09.002 PC-2 — Ctrl+C → [0x03] (ETX)
    ///
    /// Canonical test vector from BC-2.09.002 §Canonical Test Vectors.
    /// kitty_active=false (Ctrl+letter is a named VT arm; kitty_active is irrelevant).
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_ctrl() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('c'),
            modifiers: PtyKeyModifiers::CONTROL,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x03]));
    }

    /// BC-2.09.002 PC-2 — all four arrow keys produce correct VT sequences
    ///
    /// Arrow Up   → ESC [ A  (\x1b\x5b\x41)
    /// Arrow Down → ESC [ B  (\x1b\x5b\x42)
    /// Arrow Right→ ESC [ C  (\x1b\x5b\x43)
    /// Arrow Left → ESC [ D  (\x1b\x5b\x44)
    ///
    /// kitty_active=false; unmodified arrow keys are VT-invariant.
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
            assert_eq!(key_event_to_pty_bytes(event, false), Some(expected));
        }
    }

    /// BC-2.09.002 PC-2 — F1–F4 → SS3 sequences; F5 → tilde sequence
    ///
    /// F1 → \x1bOP, F2 → \x1bOQ, F3 → \x1bOR, F4 → \x1bOS
    /// F5 → \x1b[15~
    ///
    /// kitty_active=false; function key sequences are VT-invariant.
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
                key_event_to_pty_bytes(event, false),
                Some(expected),
                "F{n} key bytes mismatch"
            );
        }
    }

    /// BC-2.09.002 PC-3 — Release events return None; nothing forwarded to PTY
    ///
    /// Canonical test vector from BC-2.09.002 §Canonical Test Vectors.
    /// kitty_active=false; Release-discard is unconditional.
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_release_discarded() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Release,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), None);
    }

    /// BC-2.09.002 Invariant 3 — Ctrl+D → [0x04] (ASCII EOT); session termination signal
    ///
    /// Canonical test vector from BC-2.09.002 §Canonical Test Vectors.
    /// kitty_active=false; Ctrl+letter is a named VT arm.
    #[test]
    fn test_BC_2_09_002_keyboard_forwarding_ctrl_d_eot() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('d'),
            modifiers: PtyKeyModifiers::CONTROL,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x04]));
    }

    /// BC-2.09.002 PC-2 — full Ctrl+[A-Z] translation table
    ///
    /// Ctrl+A → \x01 through Ctrl+Z → \x1a (control characters 1–26).
    /// kitty_active=false; Ctrl+letter VT arm is kitty_active-independent.
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
                key_event_to_pty_bytes(event, false),
                Some(vec![expected_byte]),
                "Ctrl+{} should produce \\x{:02x}",
                ch,
                expected_byte
            );
        }
    }

    /// BC-2.09.002 PC-2 — Enter → \r (carriage return)
    ///
    /// kitty_active=false; Enter → \r is VT-invariant (named arm matches before Kitty).
    #[test]
    fn test_BC_2_09_002_enter_maps_to_carriage_return() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Enter,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x0d]));
    }

    /// BC-2.09.002 PC-2 — Backspace → \x7f (DEL)
    ///
    /// kitty_active=false; Backspace → DEL is VT-invariant.
    #[test]
    fn test_BC_2_09_002_backspace_maps_to_del() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Backspace,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x7f]));
    }

    /// BC-2.09.002 PC-2 — Tab → \t
    ///
    /// kitty_active=false; Tab → HT is VT-invariant.
    #[test]
    fn test_BC_2_09_002_tab_maps_to_horizontal_tab() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Tab,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x09]));
    }

    /// BC-2.09.002 PC-2 — navigation keys (Home/End/PgUp/PgDn/Ins/Del)
    ///
    /// Sequence table from BC-2.09.002 PC-2.
    /// kitty_active=false; navigation keys are VT-invariant (named arms match first).
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
            assert_eq!(key_event_to_pty_bytes(event, false), Some(expected));
        }
    }

    /// BC-2.09.002 PC-2 — Alt+char → ESC-prefix (ESC + UTF-8 bytes of char)
    ///
    /// Alt+a → \x1b a (\x1b\x61)
    /// kitty_active=false: Alt+printable ESC-prefix arm fires (Kitty arm is skipped).
    #[test]
    fn test_BC_2_09_002_alt_char_produces_esc_prefix() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::ALT,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x1b, 0x61]));
    }

    /// BC-2.09.002 PC-2 — Shift+Tab (BackTab) → \x1b[Z
    ///
    /// kitty_active=false; BackTab VT arm fires (Kitty arm is skipped).
    #[test]
    fn test_BC_2_09_002_shift_tab_maps_to_csi_z() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::BackTab,
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(
            key_event_to_pty_bytes(event, false),
            Some(vec![0x1b, b'[', b'Z'])
        );
    }

    /// BC-2.09.002 PC-2 — F6–F12 tilde sequences (full table)
    ///
    /// F6→\x1b[17~, F7→\x1b[18~, F8→\x1b[19~, F9→\x1b[20~,
    /// F10→\x1b[21~, F11→\x1b[23~, F12→\x1b[24~
    ///
    /// kitty_active=false; function key sequences are VT-invariant.
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
                key_event_to_pty_bytes(event, false),
                Some(expected),
                "F{n} key bytes mismatch"
            );
        }
    }

    /// BC-2.09.002 PC-3/PC-4 — Release events for Release kind return None
    ///
    /// Additional coverage: Release on any key (not just 'a') returns None.
    /// kitty_active=false; Release-discard is unconditional.
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
            assert_eq!(key_event_to_pty_bytes(event, false), None);
        }
    }

    /// BC-2.09.002 PC-1/PC-2 — Repeat kind is forwarded (same as Press)
    ///
    /// key_event_to_pty_bytes treats Press and Repeat identically.
    /// kitty_active=false; Repeat-forwarding is unconditional.
    #[test]
    fn test_BC_2_09_002_repeat_kind_forwarded_same_as_press() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('a'),
            modifiers: PtyKeyModifiers::NONE,
            kind: PtyKeyEventKind::Repeat,
        };
        assert_eq!(key_event_to_pty_bytes(event, false), Some(vec![0x61]));
    }

    // -----------------------------------------------------------------------
    // BC-2.09.002 EC-217 — TRACE+None boundary: modifier combo with no VT
    // encoding on non-Kitty terminal returns None (observable drop, not silent).
    // -----------------------------------------------------------------------

    /// BC-2.09.002 EC-217 — Alt+Up on non-Kitty terminal → None
    ///
    /// Alt+Up has no VT encoding in the standard table. On a non-Kitty terminal
    /// (kitty_active=false), the `_ if !mods.is_empty()` TRACE+None arm fires.
    /// The drop is NOT silent — a TRACE log is emitted (observable at TRACE level).
    /// Per BC-2.09.002 PC-1: no keyboard class is silently dropped; TRACE+None
    /// satisfies the "no silent drop" invariant.
    ///
    /// Source: BC-2.09.002 EC-217; SS-embedded-pty.md §Translation function HIGH-001.
    #[test]
    fn test_BC_2_09_002_modifier_combo_no_vt_trace_none() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Up,
            modifiers: PtyKeyModifiers::ALT,
            kind: PtyKeyEventKind::Press,
        };
        // Alt+Up: no VT encoding arm; TRACE log emitted; None returned.
        // This test verifies the observable-drop boundary per EC-217.
        assert_eq!(
            key_event_to_pty_bytes(event, false),
            None,
            "Alt+Up on non-Kitty terminal must return None (EC-217 TRACE+None boundary)"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.09.002 PC-2 — Coverage-gap: Ctrl+@ and Ctrl+[ (special Ctrl rows)
    // -----------------------------------------------------------------------

    /// BC-2.09.002 PC-2 — Ctrl+@ → [0x00] (NUL)
    ///
    /// Ctrl+@ is the standard xterm encoding for NUL.
    /// '@' ASCII is 0x40; 0x40 - 0x40 = 0x00.
    /// Source: BC-2.09.002 PC-2 key table row "Ctrl+@ | \x00 (NUL)".
    #[test]
    fn test_BC_2_09_002_ctrl_at_nul() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('@'),
            modifiers: PtyKeyModifiers::CONTROL,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(
            key_event_to_pty_bytes(event, false),
            Some(vec![0x00]),
            "Ctrl+@ must produce NUL (BC-2.09.002 PC-2)"
        );
    }

    /// BC-2.09.002 PC-2 — Ctrl+[ → [0x1b] (ESC byte)
    ///
    /// Ctrl+[ is the standard xterm encoding for ESC as a byte.
    /// '[' ASCII is 0x5b; 0x5b - 0x40 = 0x1b.
    ///
    /// Note: this is the PURE FUNCTION result. The Esc-EXIT interception
    /// (BC-2.09.002 Invariant 2) lives in the dispatch layer (monocle-tui),
    /// not in key_event_to_pty_bytes. Ctrl+[ is a distinct keycode path
    /// from bare Esc; the dispatch layer intercepts bare Esc (PtyKeyCode::Esc),
    /// not Ctrl+[.
    ///
    /// Source: BC-2.09.002 PC-2 key table row "Ctrl+[ | \x1b (Esc)".
    #[test]
    fn test_BC_2_09_002_ctrl_bracket_esc() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('['),
            modifiers: PtyKeyModifiers::CONTROL,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(
            key_event_to_pty_bytes(event, false),
            Some(vec![0x1b]),
            "Ctrl+[ must produce ESC byte 0x1b (BC-2.09.002 PC-2)"
        );
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
    // BC-2.09.004: Kitty Keyboard Protocol — kitty_active bool parameter
    // -----------------------------------------------------------------------

    /// BC-2.09.004 PC-1 — is_kitty_enhanced_key returns true when kitty_active=true
    /// and modifier combo is non-empty.
    ///
    /// kitty_active=true: Up+CONTROL is a modifier combo; returns true.
    /// This is the Kitty-active path that routes to CSI-u encoding.
    ///
    /// Source: BC-2.09.004 PC-1; SS-embedded-pty.md §Translation function.
    #[test]
    fn test_BC_2_09_004_kitty_active_true_modifier_combo() {
        assert!(
            is_kitty_enhanced_key(&PtyKeyCode::Up, PtyKeyModifiers::CONTROL, true),
            "is_kitty_enhanced_key must return true for Up+CONTROL when kitty_active=true (BC-2.09.004 PC-1)"
        );
    }

    /// BC-2.09.004 PC-4 / EC-216 — is_kitty_enhanced_key returns false when kitty_active=false
    ///
    /// kitty_active=false: early-return guard fires unconditionally.
    /// Even a modifier-carrying combo (Up+CONTROL) returns false on non-Kitty terminals.
    /// The standard VT table is used as fallback.
    ///
    /// Source: BC-2.09.004 PC-4; EC-216; SS-embedded-pty.md §Translation function.
    #[test]
    fn test_BC_2_09_004_kitty_active_false_returns_false() {
        assert!(
            !is_kitty_enhanced_key(&PtyKeyCode::Up, PtyKeyModifiers::CONTROL, false),
            "is_kitty_enhanced_key must return false when kitty_active=false (BC-2.09.004 PC-4)"
        );
    }

    /// BC-2.09.004 PC-2 — Ctrl+Shift+Enter → \x1b[13;6u via key_event_to_pty_bytes
    ///
    /// End-to-end integration: when kitty_active=true and the event is Ctrl+Shift+Enter,
    /// key_event_to_pty_bytes reaches the Kitty catch-all arm (is_kitty_enhanced_key
    /// returns true), calls encode_kitty_key, and produces the CSI-u sequence.
    ///
    /// This test confirms the kitty_active catch-all arm is live (not dead code).
    ///
    /// Expected bytes: ESC [ 1 3 ; 6 u
    /// Derivation: Enter codepoint=13; modifier = 1 + shift(1) + ctrl(4) = 6.
    /// Source: BC-2.09.004 §Canonical Test Vectors.
    #[test]
    fn test_BC_2_09_004_kitty_active_true_csi_u_via_key_event_to_pty_bytes() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Enter,
            modifiers: PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT,
            kind: PtyKeyEventKind::Press,
        };
        // kitty_active=true: Kitty catch-all arm fires; encode_kitty_key produces CSI-u.
        assert_eq!(
            key_event_to_pty_bytes(event, true),
            Some(b"\x1b[13;6u".to_vec()),
            "Ctrl+Shift+Enter with kitty_active=true must produce \\x1b[13;6u (BC-2.09.004)"
        );
    }

    /// BC-2.09.004 PC-2 — Ctrl+Shift+Enter → \x1b[13;6u (direct encode_kitty_key call)
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
        // kitty_active=false: early-return guard fires; returns false regardless of mods.
        assert!(!is_kitty_enhanced_key(
            &PtyKeyCode::Enter,
            PtyKeyModifiers::NONE,
            false
        ));
        // Even with modifiers, kitty_active=false short-circuits; returns false.
        assert!(!is_kitty_enhanced_key(
            &PtyKeyCode::Enter,
            PtyKeyModifiers::CONTROL,
            false
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

    // -----------------------------------------------------------------------
    // PASS-3 DIRECTIVES — all tests below are new Red Gate additions
    // per S-040 pass-3 design directives (SS-embedded-pty.md §Trace v1.13.0).
    // Each test MUST FAIL until the implementer applies the corresponding fix.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // O-1: Real Kitty functional-key codepoints
    //
    // Current production pty_key_codepoint() uses WRONG values for nav/arrow keys
    // (e.g., Up=65='A', Home=72='H'). The correct Kitty functional-key codepoints per
    // https://sw.kovidgoyal.net/kitty/keyboard-protocol/#functional-key-definitions are:
    //   Up=57352, Down=57353, Left=57351, Right=57354
    //   Home=57360, End=57361, PageUp=57362, PageDown=57363
    //   Insert=57348, Delete=57349
    //   F1=57364..F12=57375
    // These tests will FAIL (assertion error) until pty_key_codepoint() is corrected.
    // -----------------------------------------------------------------------

    /// O-1 / BC-2.09.004 — Ctrl+Up → \x1b[57352;5u (Kitty functional-key codepoint)
    ///
    /// Up codepoint = 57352 (Kitty spec); modifier = 1 + ctrl(4) = 5.
    /// Current production has Up=65 ('A') → \x1b[65;5u — WRONG (data corruption).
    ///
    /// Source: SS-embedded-pty.md §Trace v1.13.0 O-1; BC-2.09.004 EC-225.
    #[test]
    fn test_BC_2_09_004_kitty_ctrl_up() {
        let mods = PtyKeyModifiers::CONTROL;
        // Direct encode_kitty_key call.
        let result = encode_kitty_key(&PtyKeyCode::Up, mods, PtyKeyEventKind::Press);
        // Expected: ESC [ 5 7 3 5 2 ; 5 u
        assert_eq!(
            result,
            b"\x1b[57352;5u".to_vec(),
            "Ctrl+Up Kitty codepoint must be 57352 (not 65='A'); \
             source: SS-embedded-pty.md §Trace v1.13.0 O-1"
        );

        // End-to-end via key_event_to_pty_bytes (kitty_active=true → Kitty catch-all arm).
        let event = PtyKeyEvent {
            code: PtyKeyCode::Up,
            modifiers: PtyKeyModifiers::CONTROL,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(
            key_event_to_pty_bytes(event, true),
            Some(b"\x1b[57352;5u".to_vec()),
            "key_event_to_pty_bytes(Ctrl+Up, kitty_active=true) must produce \\x1b[57352;5u"
        );
    }

    /// O-1 / BC-2.09.004 — Shift+Home → \x1b[57360;2u (Kitty functional-key codepoint)
    ///
    /// Home codepoint = 57360 (Kitty spec); modifier = 1 + shift(1) = 2.
    /// Current production has Home=72 ('H') → \x1b[72;2u — WRONG.
    ///
    /// Source: SS-embedded-pty.md §Trace v1.13.0 O-1.
    #[test]
    fn test_BC_2_09_004_kitty_home_shift_csi_u() {
        let mods = PtyKeyModifiers::SHIFT;
        let result = encode_kitty_key(&PtyKeyCode::Home, mods, PtyKeyEventKind::Press);
        // Expected: ESC [ 5 7 3 6 0 ; 2 u
        assert_eq!(
            result,
            b"\x1b[57360;2u".to_vec(),
            "Shift+Home Kitty codepoint must be 57360 (not 72='H'); \
             source: SS-embedded-pty.md §Trace v1.13.0 O-1"
        );
    }

    // -----------------------------------------------------------------------
    // MED-001: Tab+SHIFT arm (non-Kitty path)
    //
    // When a terminal reports Shift+Tab as KeyCode::Tab + SHIFT modifier (rather than
    // KeyCode::BackTab), key_event_to_pty_bytes MUST still produce \x1b[Z on non-Kitty
    // terminals. The arm `PtyKeyCode::Tab if mods.contains(PtyKeyModifiers::SHIFT)` is
    // REQUIRED. Without it, Tab+SHIFT falls to arm 7 (TRACE+None) on non-Kitty terminals.
    //
    // Source: SS-embedded-pty.md §Trace v1.13.0 ADV-MED-001.
    // -----------------------------------------------------------------------

    /// MED-001 / BC-2.09.002 — Tab+SHIFT → \x1b[Z on non-Kitty terminal
    ///
    /// Some terminals report Shift+Tab as PtyKeyCode::Tab + PtyKeyModifiers::SHIFT
    /// rather than PtyKeyCode::BackTab. Both must produce \x1b[Z (CSI Z).
    /// kitty_active=false: Tab+SHIFT arm must fire BEFORE the TRACE+None arm.
    ///
    /// Source: SS-embedded-pty.md §Trace v1.13.0 ADV-MED-001; BC-2.09.002 AC-001.
    #[test]
    fn test_BC_2_09_002_tab_shift_csi_z() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Tab,
            modifiers: PtyKeyModifiers::SHIFT,
            kind: PtyKeyEventKind::Press,
        };
        assert_eq!(
            key_event_to_pty_bytes(event, false),
            Some(b"\x1b[Z".to_vec()),
            "Tab+SHIFT on non-Kitty terminal must produce \\x1b[Z (MED-001); \
             source: SS-embedded-pty.md §Trace v1.13.0"
        );
    }

    // -----------------------------------------------------------------------
    // MED-002: Ctrl+Shift+letter → control byte (canonical Ctrl-arm guard)
    //
    // The canonical Ctrl guard is `mods.contains(CONTROL) && !mods.contains(ALT)`.
    // Ctrl+Shift+letter must produce the same control byte as Ctrl+letter alone
    // (standard terminal convention: Shift does not affect the control-byte mapping).
    // This arm fires on non-Kitty terminals (kitty_active=false) for Ctrl+Shift+letter.
    //
    // The OLD guard `mods == CONTROL` (equality) excluded Ctrl+Shift+letter, dropping
    // it to TRACE+None — unexpected and incorrect. The `contains` form is canonical.
    //
    // Source: SS-embedded-pty.md §Trace v1.13.0 ADV-MED-002.
    // -----------------------------------------------------------------------

    /// MED-002 / BC-2.09.002 — Ctrl+Shift+C → [0x03] (control byte; Shift ignored)
    ///
    /// On a non-Kitty terminal, Ctrl+Shift+letter produces the same control byte
    /// as Ctrl+letter alone. The canonical guard `mods.contains(CONTROL) && !mods.contains(ALT)`
    /// matches Ctrl+Shift combos (SHIFT bit does not affect control-byte generation).
    ///
    /// Derivation: 'C' → uppercase 'C' (0x43) → 0x43 - 0x40 = 0x03 (ETX).
    ///
    /// Source: SS-embedded-pty.md §Trace v1.13.0 ADV-MED-002; BC-2.09.002 AC-001.
    #[test]
    fn test_BC_2_09_002_ctrl_shift_letter_control_byte() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Char('c'),
            modifiers: PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT,
            kind: PtyKeyEventKind::Press,
        };
        // kitty_active=false: Ctrl-arm guard `contains(CONTROL) && !contains(ALT)` fires.
        // SHIFT bit does not affect the control-byte result.
        assert_eq!(
            key_event_to_pty_bytes(event, false),
            Some(vec![0x03]),
            "Ctrl+Shift+C on non-Kitty terminal must produce [0x03] (ETX); \
             source: SS-embedded-pty.md §Trace v1.13.0 ADV-MED-002"
        );
    }

    // -----------------------------------------------------------------------
    // MED-003: PtyKeyModifiers::contains all-bits semantics
    //
    // The current implementation uses `self.0 & other.0 != 0` (any-bit test).
    // The CORRECT semantics for `contains` is all-bits: `self.0 & other.0 == other.0`.
    // The any-bit form causes false positives: CONTROL.contains(CONTROL | ALT) returns
    // true because `CONTROL.0 & (CONTROL | ALT).0 != 0` — but ALT bit is NOT set in
    // CONTROL, so `contains(CONTROL|ALT)` should return false.
    //
    // Source: SS-embedded-pty.md §Core-Owned Mirror Types.
    // -----------------------------------------------------------------------

    /// MED-003 — PtyKeyModifiers::contains must use all-bits semantics
    ///
    /// `contains(other)` returns true IFF ALL bits of `other` are set in `self`.
    ///
    /// Current production: `self.0 & other.0 != 0` (any-bit) — WRONG.
    /// Correct:            `self.0 & other.0 == other.0` (all-bits).
    ///
    /// The second and third assertions below FAIL on the current implementation
    /// because the any-bit form returns true when only a partial bit overlap exists.
    ///
    /// Source: SS-embedded-pty.md §Core-Owned Mirror Types; standard bitflags convention.
    #[test]
    fn test_pty_modifiers_contains_all_bits() {
        // (CONTROL | SHIFT).contains(CONTROL) — CONTROL bit is fully set → true (both impls agree).
        assert!(
            (PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT).contains(PtyKeyModifiers::CONTROL),
            "(CONTROL|SHIFT).contains(CONTROL) must be true — CONTROL bit is fully set"
        );

        // (CONTROL | SHIFT).contains(CONTROL | ALT) — ALT bit is NOT set → false.
        // any-bit impl returns true (CONTROL overlaps) — WRONG. all-bits returns false.
        assert!(
            !(PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT)
                .contains(PtyKeyModifiers::CONTROL | PtyKeyModifiers::ALT),
            "(CONTROL|SHIFT).contains(CONTROL|ALT) must be false — ALT bit is not set; \
             source: SS-embedded-pty.md §Core-Owned Mirror Types (all-bits semantics)"
        );

        // CONTROL.contains(CONTROL | SHIFT) — SHIFT bit is NOT set → false.
        // any-bit impl returns true (CONTROL overlaps) — WRONG. all-bits returns false.
        assert!(
            !PtyKeyModifiers::CONTROL
                .contains(PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT),
            "CONTROL.contains(CONTROL|SHIFT) must be false — SHIFT bit is not set in CONTROL; \
             source: SS-embedded-pty.md §Core-Owned Mirror Types (all-bits semantics)"
        );
    }

    // -----------------------------------------------------------------------
    // EC-218: Esc+modifier — NOT intercepted as ExitEmbeddedTerminal
    //
    // Bare Esc (mods.is_empty()) is intercepted by the dispatch layer.
    // Esc+ANY-modifier is NOT intercepted as ExitEmbeddedTerminal; it reaches
    // key_event_to_pty_bytes. Routing depends on kitty_active:
    //   kitty_active=false: no VT encoding for Esc+ALT → TRACE+None → None.
    //   kitty_active=true: is_kitty_enhanced_key returns true → encode_kitty_key
    //                      → \x1b[27;3u (Esc codepoint=27; modifier=1+alt(2)=3).
    //
    // Source: BC-2.09.002 EC-218; S-040 AC-015; SS-embedded-pty.md §Esc key handling.
    // -----------------------------------------------------------------------

    /// EC-218 / BC-2.09.002 AC-015 — Esc+ALT on non-Kitty terminal → None (TRACE+None)
    ///
    /// Esc with ANY non-empty modifier set is NOT the bare-Esc ExitEmbeddedTerminal
    /// intercept (that intercept is only for mods.is_empty()). On non-Kitty terminal,
    /// the `_ if !mods.is_empty()` TRACE+None arm fires → None.
    ///
    /// Source: BC-2.09.002 EC-218; S-040 AC-015.
    #[test]
    fn test_BC_2_09_002_esc_with_modifier_non_kitty_returns_none() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Esc,
            modifiers: PtyKeyModifiers::ALT,
            kind: PtyKeyEventKind::Press,
        };
        // kitty_active=false: no VT encoding for Esc+ALT; TRACE+None arm fires.
        assert_eq!(
            key_event_to_pty_bytes(event, false),
            None,
            "Esc+ALT on non-Kitty terminal must return None (TRACE+None; EC-218)"
        );
    }

    /// EC-218 / BC-2.09.002 AC-015 — Esc+ALT on Kitty terminal → \x1b[27;3u (CSI-u)
    ///
    /// On Kitty terminal (kitty_active=true), Esc+ALT is NOT intercepted as exit;
    /// is_kitty_enhanced_key returns true (mods non-empty, code != Null, kitty_active=true)
    /// → encode_kitty_key → \x1b[27;3u.
    ///
    /// Derivation: Esc codepoint = 27; modifier = 1 + alt(2) = 3.
    ///
    /// Source: BC-2.09.002 EC-218; S-040 AC-015.
    #[test]
    fn test_BC_2_09_002_esc_with_modifier_kitty_produces_csi_u() {
        let event = PtyKeyEvent {
            code: PtyKeyCode::Esc,
            modifiers: PtyKeyModifiers::ALT,
            kind: PtyKeyEventKind::Press,
        };
        // kitty_active=true: Kitty arm fires; Esc codepoint=27; ALT modifier bit.
        // modifier_value = 1 + alt(2) = 3.
        assert_eq!(
            key_event_to_pty_bytes(event, true),
            Some(b"\x1b[27;3u".to_vec()),
            "Esc+ALT on Kitty terminal must produce \\x1b[27;3u (EC-218)"
        );
    }
}
