//! TDD test suite for BC-2.09.003: Mouse Events Forwarded to PTY in SGR Encoding.
//!
//! Tests in this file exercise `mouse_event_to_pty_bytes()` — a PURE function in
//! `monocle-core/src/keyboard.rs`. All tests use only core-owned `Pty*` types
//! (NO crossterm or ratatui imports — F-P2-I06 purity boundary).
//!
//! # Coverage mapping
//!
//! | Test name | AC / BC clause |
//! |-----------|----------------|
//! | test_BC_2_09_003_mouse_events_sgr_encoded_left_press     | AC-004, PC-2, EC-220 |
//! | test_BC_2_09_003_mouse_events_sgr_encoded_left_release   | AC-004, PC-2 (terminator m) |
//! | test_BC_2_09_003_mouse_events_sgr_scroll_up              | AC-011, EC-222, PC-2 ScrollUp=64 |
//! | test_BC_2_09_003_drag_encoding                           | AC-004, PC-2 Drag(Left)=32 |
//! | test_BC_2_09_003_out_of_pane_returns_none                | AC-006, EC-221, PC-5 |
//! | test_BC_2_09_003_1_indexed_origin                        | AC-009, EC-220 |
//! | test_BC_2_09_003_1_indexed_nonzero_pane_origin           | AC-009, EC-220 (non-origin pane) |
//! | test_BC_2_09_003_modifier_bits_ctrl                      | AC-004, PC-2 Ctrl|=16 |
//! | test_BC_2_09_003_modifier_bits_shift                     | AC-004, PC-2 Shift|=4 |
//! | test_BC_2_09_003_modifier_bits_alt                       | AC-004, PC-2 Alt|=8 |
//! | test_BC_2_09_003_scroll_down_encoding                    | AC-011, EC-222, PC-2 ScrollDown=65 |
//! | test_BC_2_09_003_middle_button_press                     | AC-004, PC-2 Down(Middle)=1 |
//! | test_BC_2_09_003_right_button_press                      | AC-004, PC-2 Down(Right)=2 |
//! | test_BC_2_09_003_terminator_m_for_release_only           | AC-004, PC-2 terminator correctness |
//! | test_BC_2_09_003_terminator_M_for_non_release_variants   | AC-004, PC-2 terminator correctness |
//! | test_BC_2_09_003_drag_middle_encoding                    | AC-004, PC-2 Drag(Middle)=33 |
//! | test_BC_2_09_003_drag_right_encoding                     | AC-004, PC-2 Drag(Right)=34 |
//! | test_BC_2_09_003_middle_button_release                   | AC-004, PC-2 Up(Middle)=1, terminator m |
//! | test_BC_2_09_003_right_button_release                    | AC-004, PC-2 Up(Right)=2, terminator m |
//!
//! # Red Gate
//!
//! All tests fail against the current stub because `mouse_event_to_pty_bytes` is
//! `todo!()` — panicking on the first call.
//!
//! # No version-pin literals
//!
//! This file contains NO dependency version strings per POL-11.

#![allow(non_snake_case)]

use monocle_core::keyboard::{
    mouse_event_to_pty_bytes, PtyKeyModifiers, PtyMouseButton, PtyMouseEvent, PtyMouseEventKind,
    PtyRect,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Construct a `PtyMouseEvent` with no modifiers.
fn mouse_event(kind: PtyMouseEventKind, column: u16, row: u16) -> PtyMouseEvent {
    PtyMouseEvent {
        kind,
        column,
        row,
        modifiers: PtyKeyModifiers::NONE,
    }
}

/// Construct a `PtyMouseEvent` with specified modifiers.
fn mouse_event_with_mods(
    kind: PtyMouseEventKind,
    column: u16,
    row: u16,
    modifiers: PtyKeyModifiers,
) -> PtyMouseEvent {
    PtyMouseEvent {
        kind,
        column,
        row,
        modifiers,
    }
}

/// Construct a `PtyRect` at origin with the given dimensions.
fn origin_pane(width: u16, height: u16) -> PtyRect {
    PtyRect {
        x: 0,
        y: 0,
        width,
        height,
    }
}

/// Construct a `PtyRect` at non-zero position.
fn pane_at(x: u16, y: u16, width: u16, height: u16) -> PtyRect {
    PtyRect {
        x,
        y,
        width,
        height,
    }
}

// ---------------------------------------------------------------------------
// BC-2.09.003 PC-2 — SGR encoding: happy path canonical test vectors
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.09.003 PC-2 — Left button press at (col=5, row=3), pane at origin
///
/// Canonical test vector from BC-2.09.003 §Canonical Test Vectors:
///   Down(Left) at crossterm (column=5, row=3), pane_area {x:0, y:0, width:80, height:24}
///   → Px = 5 - 0 + 1 = 6, Py = 3 - 0 + 1 = 4
///   → `\x1b[<0;6;4M`  (base_Ps=0 for Left, terminator=M for press)
#[test]
fn test_BC_2_09_003_mouse_events_sgr_encoded_left_press() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 5, 3);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<0;6;4M".to_vec()),
        "Down(Left) at (5,3) in 80x24 pane@origin must encode as \\x1b[<0;6;4M (Px=6, Py=4)"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Left button release at (col=5, row=3), pane at origin
///
/// Canonical test vector from BC-2.09.003 §Canonical Test Vectors:
///   Up(Left) at crossterm (column=5, row=3), pane_area {x:0, y:0, width:80, height:24}
///   → Px = 6, Py = 4
///   → `\x1b[<0;6;4m`  (terminator LOWERCASE m for release events)
#[test]
fn test_BC_2_09_003_mouse_events_sgr_encoded_left_release() {
    let event = mouse_event(PtyMouseEventKind::Up(PtyMouseButton::Left), 5, 3);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<0;6;4m".to_vec()),
        "Up(Left) at (5,3) must use lowercase terminator 'm' — got {:?}",
        result
    );
}

/// AC-011 / EC-222 / BC-2.09.003 PC-2 — ScrollUp at (col=20, row=10), pane at origin
///
/// Canonical test vector from BC-2.09.003 §Canonical Test Vectors:
///   ScrollUp at crossterm (column=20, row=10), pane_area {x:0, y:0, width:80, height:24}
///   → Px = 20 + 1 = 21, Py = 10 + 1 = 11
///   → `\x1b[<64;21;11M`  (base_Ps=64 for ScrollUp)
#[test]
fn test_BC_2_09_003_mouse_events_sgr_scroll_up() {
    let event = mouse_event(PtyMouseEventKind::ScrollUp, 20, 10);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<64;21;11M".to_vec()),
        "ScrollUp at (20,10) in 80x24 pane@origin must encode as \\x1b[<64;21;11M"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Drag(Left) at (col=10, row=5), pane at origin
///
///   Drag(Left) → base_Ps = 32 (button_base=0 + 32 motion bit)
///   Px = 10 + 1 = 11, Py = 5 + 1 = 6
///   → `\x1b[<32;11;6M`
#[test]
fn test_BC_2_09_003_drag_encoding() {
    let event = mouse_event(PtyMouseEventKind::Drag(PtyMouseButton::Left), 10, 5);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<32;11;6M".to_vec()),
        "Drag(Left) at (10,5) must encode as \\x1b[<32;11;6M (Ps=32)"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / EC-221 — Out-of-pane events return None
// ---------------------------------------------------------------------------

/// AC-006 / EC-221 / BC-2.09.003 PC-5 — Event outside pane returns None; NOT forwarded
///
///   column=200, row=200 is outside a 80x24 pane at origin → None returned
///   No spurious PTY mouse event is sent (BC-2.09.003 PC-5).
#[test]
fn test_BC_2_09_003_out_of_pane_returns_none() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 200, 200);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result, None,
        "column=200, row=200 outside 80x24 pane must return None (EC-221)"
    );
}

/// AC-006 — Out-of-pane column-only violation returns None
///
///   col=80 is one past the right edge of an 80-wide pane → None
#[test]
fn test_BC_2_09_003_out_of_pane_column_boundary_returns_none() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 80, 0);
    // pane: x=0, y=0, width=80 → valid columns are 0..79
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result, None,
        "col=80 is one past the right edge of an 80-wide pane — must return None"
    );
}

/// AC-006 — Out-of-pane row-only violation returns None
///
///   row=24 is one past the bottom edge of a 24-row pane → None
#[test]
fn test_BC_2_09_003_out_of_pane_row_boundary_returns_none() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 0, 24);
    // pane: x=0, y=0, height=24 → valid rows are 0..23
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result, None,
        "row=24 is one past the bottom edge of a 24-row pane — must return None"
    );
}

// ---------------------------------------------------------------------------
// AC-009 / EC-220 — 1-indexed origin
// ---------------------------------------------------------------------------

/// AC-009 / EC-220 / BC-2.09.003 PC-2 — Click at pane origin → Px=1, Py=1 (1-indexed)
///
///   event at (column=pane.x, row=pane.y) with pane at origin (x=0, y=0):
///   Px = 0 - 0 + 1 = 1, Py = 0 - 0 + 1 = 1
///   → `\x1b[<0;1;1M`
///
///   This catches the 1-indexed formula: 0-indexed crossterm coordinate → 1-indexed SGR.
#[test]
fn test_BC_2_09_003_1_indexed_origin() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 0, 0);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<0;1;1M".to_vec()),
        "Click at pane origin (0,0) must produce Px=1,Py=1 (1-indexed): \\x1b[<0;1;1M"
    );
}

/// AC-009 / EC-220 — Click at non-zero pane origin produces Px=1, Py=1
///
///   Stronger test: pane at (x=3, y=2), event at (column=3, row=2)
///   Px = 3 - 3 + 1 = 1, Py = 2 - 2 + 1 = 1
///   This verifies the pane-relative offset: even for a non-origin pane, the corner
///   of the pane always maps to Px=1, Py=1.
#[test]
fn test_BC_2_09_003_1_indexed_nonzero_pane_origin() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 3, 2);
    let pane_area = pane_at(3, 2, 80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<0;1;1M".to_vec()),
        "Click at non-zero pane corner (3,2) in pane@(3,2) must produce Px=1,Py=1"
    );
}

/// AC-009 — Pane-relative offset verified with non-origin pane and interior coordinate
///
///   Pane at (x=10, y=5), event at (column=15, row=8):
///   Px = 15 - 10 + 1 = 6, Py = 8 - 5 + 1 = 4
///   → `\x1b[<0;6;4M`
#[test]
fn test_BC_2_09_003_pane_relative_offset_nonzero_pane() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 15, 8);
    let pane_area = pane_at(10, 5, 80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<0;6;4M".to_vec()),
        "Event at (15,8) in pane@(10,5) must produce Px=6, Py=4: \\x1b[<0;6;4M"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.09.003 PC-2 — Modifier bits additive: Shift|=4, Alt|=8, Ctrl|=16
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.09.003 PC-2 — Ctrl modifier: Down(Left)+CONTROL at pane origin
///
///   base_Ps = 0 (Left), modifier_bits = CONTROL|=16
///   Ps_final = 0 | 16 = 16
///   Px = 1, Py = 1 (pane origin)
///   → `\x1b[<16;1;1M`
#[test]
fn test_BC_2_09_003_modifier_bits_ctrl() {
    let event = mouse_event_with_mods(
        PtyMouseEventKind::Down(PtyMouseButton::Left),
        0,
        0,
        PtyKeyModifiers::CONTROL,
    );
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<16;1;1M".to_vec()),
        "Down(Left)+CONTROL at pane origin must produce Ps_final=16: \\x1b[<16;1;1M"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Shift modifier: Down(Left)+SHIFT at pane origin
///
///   base_Ps = 0 (Left), modifier_bits = SHIFT|=4
///   Ps_final = 0 | 4 = 4
///   → `\x1b[<4;1;1M`
#[test]
fn test_BC_2_09_003_modifier_bits_shift() {
    let event = mouse_event_with_mods(
        PtyMouseEventKind::Down(PtyMouseButton::Left),
        0,
        0,
        PtyKeyModifiers::SHIFT,
    );
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<4;1;1M".to_vec()),
        "Down(Left)+SHIFT at pane origin must produce Ps_final=4: \\x1b[<4;1;1M"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Alt modifier: Down(Left)+ALT at pane origin
///
///   base_Ps = 0 (Left), modifier_bits = ALT|=8
///   Ps_final = 0 | 8 = 8
///   → `\x1b[<8;1;1M`
#[test]
fn test_BC_2_09_003_modifier_bits_alt() {
    let event = mouse_event_with_mods(
        PtyMouseEventKind::Down(PtyMouseButton::Left),
        0,
        0,
        PtyKeyModifiers::ALT,
    );
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<8;1;1M".to_vec()),
        "Down(Left)+ALT at pane origin must produce Ps_final=8: \\x1b[<8;1;1M"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Combined Ctrl+Shift: Down(Left)+CTRL+SHIFT at pane origin
///
///   Ps_final = 0 | 16 | 4 = 20
///   → `\x1b[<20;1;1M`
#[test]
fn test_BC_2_09_003_modifier_bits_ctrl_shift_combined() {
    let event = mouse_event_with_mods(
        PtyMouseEventKind::Down(PtyMouseButton::Left),
        0,
        0,
        PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT,
    );
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<20;1;1M".to_vec()),
        "Down(Left)+CTRL+SHIFT at pane origin must produce Ps_final=20"
    );
}

// ---------------------------------------------------------------------------
// AC-011 / EC-222 — Scroll down encoding
// ---------------------------------------------------------------------------

/// AC-011 / EC-222 / BC-2.09.003 PC-2 — ScrollDown at (col=5, row=3), pane at origin
///
///   base_Ps = 65 (ScrollDown)
///   Px = 5 + 1 = 6, Py = 3 + 1 = 4
///   → `\x1b[<65;6;4M`
#[test]
fn test_BC_2_09_003_scroll_down_encoding() {
    let event = mouse_event(PtyMouseEventKind::ScrollDown, 5, 3);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<65;6;4M".to_vec()),
        "ScrollDown at (5,3) must encode as \\x1b[<65;6;4M (base_Ps=65)"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — Middle and Right button coverage
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.09.003 PC-2 — Down(Middle) at pane origin → base_Ps=1
///
///   Down(Middle) → base_Ps = 1
///   Px = 1, Py = 1
///   → `\x1b[<1;1;1M`
#[test]
fn test_BC_2_09_003_middle_button_press() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Middle), 0, 0);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<1;1;1M".to_vec()),
        "Down(Middle) at pane origin must encode as \\x1b[<1;1;1M (base_Ps=1)"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Down(Right) at pane origin → base_Ps=2
///
///   Down(Right) → base_Ps = 2
///   → `\x1b[<2;1;1M`
#[test]
fn test_BC_2_09_003_right_button_press() {
    let event = mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Right), 0, 0);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<2;1;1M".to_vec()),
        "Down(Right) at pane origin must encode as \\x1b[<2;1;1M (base_Ps=2)"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Up(Middle) uses base_Ps=1 and terminator 'm'
#[test]
fn test_BC_2_09_003_middle_button_release() {
    let event = mouse_event(PtyMouseEventKind::Up(PtyMouseButton::Middle), 0, 0);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<1;1;1m".to_vec()),
        "Up(Middle) at pane origin must encode as \\x1b[<1;1;1m (base_Ps=1, terminator m)"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Up(Right) uses base_Ps=2 and terminator 'm'
#[test]
fn test_BC_2_09_003_right_button_release() {
    let event = mouse_event(PtyMouseEventKind::Up(PtyMouseButton::Right), 0, 0);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<2;1;1m".to_vec()),
        "Up(Right) at pane origin must encode as \\x1b[<2;1;1m (base_Ps=2, terminator m)"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — Drag variants (Middle, Right)
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.09.003 PC-2 — Drag(Middle) at pane origin → base_Ps=33 (1+32)
///
///   Drag(Middle) → base_Ps = 1 + 32 = 33
///   → `\x1b[<33;1;1M`
#[test]
fn test_BC_2_09_003_drag_middle_encoding() {
    let event = mouse_event(PtyMouseEventKind::Drag(PtyMouseButton::Middle), 0, 0);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<33;1;1M".to_vec()),
        "Drag(Middle) at pane origin must encode as \\x1b[<33;1;1M (base_Ps=33)"
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Drag(Right) at pane origin → base_Ps=34 (2+32)
///
///   Drag(Right) → base_Ps = 2 + 32 = 34
///   → `\x1b[<34;1;1M`
#[test]
fn test_BC_2_09_003_drag_right_encoding() {
    let event = mouse_event(PtyMouseEventKind::Drag(PtyMouseButton::Right), 0, 0);
    let pane_area = origin_pane(80, 24);
    let result = mouse_event_to_pty_bytes(event, pane_area);
    assert_eq!(
        result,
        Some(b"\x1b[<34;1;1M".to_vec()),
        "Drag(Right) at pane origin must encode as \\x1b[<34;1;1M (base_Ps=34)"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — Terminator correctness: 'm' for Up variants, 'M' for all others
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.09.003 PC-2 — Terminator correctness: Up(Left) uses 'm', Down uses 'M'
///
/// This test verifies the critical terminator distinction:
/// - All Up (release) events → lowercase 'm' terminator
/// - All Down/Drag/Scroll events → uppercase 'M' terminator
///
/// Uses byte slice comparison to make the terminator unambiguous in the assertion.
#[test]
fn test_BC_2_09_003_terminator_m_for_release_only() {
    let release_event = mouse_event(PtyMouseEventKind::Up(PtyMouseButton::Left), 0, 0);
    let pane_area = origin_pane(80, 24);
    let bytes = mouse_event_to_pty_bytes(release_event, pane_area)
        .expect("Up(Left) at pane origin must return Some");
    // Last byte must be 'm' (0x6d), not 'M' (0x4d).
    let last = *bytes.last().expect("SGR sequence must not be empty");
    assert_eq!(
        last, b'm',
        "Up(Left) release terminator must be 'm' (0x6d), got 0x{:02x}",
        last
    );
}

/// AC-004 / BC-2.09.003 PC-2 — Press, Drag, and Scroll all use uppercase 'M' terminator
#[test]
fn test_BC_2_09_003_terminator_M_for_non_release_variants() {
    let pane_area = origin_pane(80, 24);

    let cases = [
        (
            "Down(Left)",
            mouse_event(PtyMouseEventKind::Down(PtyMouseButton::Left), 0, 0),
        ),
        (
            "Drag(Left)",
            mouse_event(PtyMouseEventKind::Drag(PtyMouseButton::Left), 0, 0),
        ),
        ("ScrollUp", mouse_event(PtyMouseEventKind::ScrollUp, 0, 0)),
        (
            "ScrollDown",
            mouse_event(PtyMouseEventKind::ScrollDown, 0, 0),
        ),
    ];

    for (label, event) in cases {
        let bytes = mouse_event_to_pty_bytes(event, pane_area)
            .unwrap_or_else(|| panic!("{} at pane origin must return Some", label));
        let last = *bytes.last().expect("SGR sequence must not be empty");
        assert_eq!(
            last, b'M',
            "{} terminator must be 'M' (0x4d), got 0x{:02x}",
            label, last
        );
    }
}
