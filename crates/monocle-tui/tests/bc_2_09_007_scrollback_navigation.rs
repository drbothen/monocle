//! TDD test suite for S-043: Scrollback Navigation
//!
//! Anchored to BC-2.09.007 (Scrollback — 1000 Rows Default; Configurable;
//! PtyScrollUp/Down Navigate).
//!
//! All handlers are implemented: `handle_pty_scroll_up`, `handle_pty_scroll_down`,
//! and `render_embedded_terminal`. Tests on already-implemented GC and resize-reset
//! paths assert S-043's framing of those invariants.
//!
//! ADV Pass-1 additions (v1.4.0 BCs):
//!   - test_BC_2_09_007_content_anchored_new_output (replaces
//!     test_BC_2_09_007_new_output_does_not_reset_scroll_offset — PC-5)
//!   - test_BC_2_09_007_concurrent_status_bar_badges (PC-4, EC-245)
//!   - test_BC_2_09_007_status_bar_string_rendered (AC-007, HIGH-003)
//!   - test_BC_2_09_007_ctrl_up_dispatch_scrolls_no_ipc (AC-002/AC-006/PC-3, BLOCKER-001)
//!   - test_BC_2_09_007_ctrl_down_dispatch_scrolls_no_ipc (symmetric)
//!
//! Test naming: test_BC_2_09_007_<assertion_name> as required by the TDD contract.
//!
//! BC clause → test mapping:
//!   Postcondition 1 / AC-001 → test_BC_2_09_007_scrollback_rows_default_1000
//!   Postcondition 1 / AC-001 / EC-242 → test_BC_2_09_007_scrollback_rows_capped_10000
//!   Postcondition 1 / AC-001 / EC-243 → test_BC_2_09_007_scrollback_rows_clamped_min_1
//!   Postcondition 2a / AC-002 → test_BC_2_09_007_scrollup_increments_offset
//!   Postcondition 2b / AC-003 / EC-241 → test_BC_2_09_007_scrolldown_decrements_floor_0
//!   Postcondition 2c / AC-004 / EC-240 → test_BC_2_09_007_clamp_at_max
//!   Postcondition 2d / AC-005 / AC-011 → test_BC_2_09_007_focus_switch_preserves_offsets
//!   Postcondition 3 / AC-006 → test_BC_2_09_007_no_ipc_for_scroll
//!   Postcondition 4 / AC-007 → test_BC_2_09_007_status_bar_indicator_when_scrolled
//!   Postcondition 4 / AC-007 (render string) → test_BC_2_09_007_status_bar_string_rendered
//!   Postcondition 4 / AC-007 / PC-4 / EC-245 → test_BC_2_09_007_concurrent_status_bar_badges
//!   Postcondition 5 / AC-008 / AC-014 / EC-244 / PC-5 → test_BC_2_09_007_content_anchored_new_output
//!   Postcondition 3 / AC-002 / AC-006 / PC-3 (dispatch) → test_BC_2_09_007_ctrl_up_dispatch_scrolls_no_ipc
//!   Postcondition 3 / AC-003 / AC-006 / PC-3 (dispatch) → test_BC_2_09_007_ctrl_down_dispatch_scrolls_no_ipc
//!   Invariant 3a / AC-009 → test_BC_2_09_007_resize_resets_scroll_offset_to_zero
//!   Invariant 3b / AC-010 → test_BC_2_09_007_terminated_session_removes_scroll_entry
//!   Invariant 5 / AC-011 → test_BC_2_09_007_no_singular_shared_offset_field
//!   Render / AC-007 → test_BC_2_09_007_render_embedded_terminal_with_scroll_offset

// BC-2.09.007: test names use SCREAMING_SNAKE_CASE to embed the BC ID for traceability.
// This violates the Rust non_snake_case lint but is required by the TDD naming contract.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{
    clamp_scrollback_rows, default_scrollback_rows, AppMode, FocusSnapshot,
};
use monocle_ipc::types::ClientToServer;
use monocle_tui::app::{
    build_builtin_binding_layers, gc_pty_session, handle_crossterm_event, handle_pty_scroll_down,
    handle_pty_scroll_up, on_pty_output, on_resize_detected, render_frame, App,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use ratatui::{backend::TestBackend, Terminal};
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Test helpers (matching bc_2_09_001 patterns exactly)
// ---------------------------------------------------------------------------

/// Build a minimal `App` from a default config for unit tests.
///
/// Starts with no sessions, no parsers, scrollback_rows = 1000.
fn make_app() -> App {
    let config = MonocleConfig::default();
    App::new(config)
}

/// Build an `App` and register a parser for `session_id` so tests that exercise
/// scroll handlers have a non-empty `pty_parsers` map.
///
/// Mirrors the `make_app_with_session` helper in bc_2_09_001.
fn make_app_with_session(session_id: &str) -> App {
    let mut app = make_app();
    let parser = vt100::Parser::new(24, 80, app.scrollback_rows as usize);
    app.pty_parsers.insert(session_id.to_string(), parser);
    app.pty_scroll_offsets.insert(session_id.to_string(), 0);
    app
}

/// Build an `App` in `AppMode::EmbeddedTerminal` focused on `session_id`.
///
/// Registers a parser for `session_id` and sets the mode so that
/// `handle_pty_scroll_up` / `handle_pty_scroll_down` can find the focused session.
fn make_app_in_embedded(session_id: &str) -> App {
    let mut app = make_app_with_session(session_id);
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_string(),
        prior: FocusSnapshot::Sessions,
    };
    app
}

/// Feed `n` lines of content into the parser for `session_id` so that
/// effective scrollback max is non-zero. Each line is the format `"line{i}\r\n"`.
#[allow(clippy::unwrap_used)]
fn feed_lines(app: &mut App, session_id: &str, n: usize) {
    for i in 0..n {
        let bytes = format!("line{}\r\n", i).into_bytes();
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        parser.process(&bytes);
    }
}

/// Read `pty_scroll_offsets[session_id]`; panics if the entry is absent.
fn scroll_offset(app: &App, session_id: &str) -> usize {
    *app.pty_scroll_offsets
        .get(session_id)
        .unwrap_or_else(|| panic!("pty_scroll_offsets has no entry for '{}'", session_id))
}

/// Return the number of scrollback rows currently stored in the parser's history.
///
/// vt100 0.16.2 does not expose `Screen::scrollback_len()` publicly. The internal
/// `Grid::scrollback_len` is the capacity, but the actual rows in history grow as
/// content is fed. We obtain the effective maximum by setting scrollback to a
/// sentinel value far beyond any possible history and reading back the clamped result.
///
/// After calling this function the parser's scrollback offset is reset to 0 (live tail).
fn effective_scrollback_max(parser: &mut vt100::Parser) -> usize {
    parser.screen_mut().set_scrollback(usize::MAX);
    let max = parser.screen().scrollback();
    // Restore live tail.
    parser.screen_mut().set_scrollback(0);
    max
}

/// Collect all text from a ratatui `TestBackend` terminal buffer into a flat string.
///
/// Used by render_frame-based tests to scrape the rendered status bar for badge
/// strings such as `[scrolled back N rows]` and `[dump: N drops]`.
fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

/// Render `app` into a 100×30 TestBackend via `render_frame` and return the terminal.
///
/// Matches the helper pattern in render_frame_integration_s028.rs.
fn render_app_to_terminal(app: &mut App) -> Terminal<TestBackend> {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
    terminal
        .draw(|frame| {
            let mut state = SessionsPanelState::default();
            render_frame(app, &mut state, frame);
        })
        .expect("terminal.draw must succeed");
    terminal
}

/// Build a crossterm `KeyEvent` for Ctrl+Up (Press).
fn ctrl_up_event() -> crossterm::event::Event {
    crossterm::event::Event::Key(crossterm::event::KeyEvent::new_with_kind(
        crossterm::event::KeyCode::Up,
        crossterm::event::KeyModifiers::CONTROL,
        crossterm::event::KeyEventKind::Press,
    ))
}

/// Build a crossterm `KeyEvent` for Ctrl+Down (Press).
fn ctrl_down_event() -> crossterm::event::Event {
    crossterm::event::Event::Key(crossterm::event::KeyEvent::new_with_kind(
        crossterm::event::KeyCode::Down,
        crossterm::event::KeyModifiers::CONTROL,
        crossterm::event::KeyEventKind::Press,
    ))
}

/// Drain all messages from an IPC receiver channel without blocking.
fn drain_ipc(rx: &mut mpsc::Receiver<ClientToServer>) -> Vec<ClientToServer> {
    let mut out = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        out.push(msg);
    }
    out
}

// ---------------------------------------------------------------------------
// AC-001: parser initialized with scrollback_rows default 1000 (framed by S-043)
// BC-2.09.007 Postcondition 1 / Invariant 1 (absent config → 1000)
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_scrollback_rows_default_1000
///
/// Exercises BC-2.09.007 Postcondition 1 and Invariant 1 from the S-043 perspective:
///   - Absent `pty_scrollback_rows` in config → `App::scrollback_rows == 1000`.
///   - The `default_scrollback_rows()` pure helper returns 1000.
///   - A `vt100::Parser` created with `scrollback_rows = 1000` accepts scrollback.
///
/// S-039 owns the config-load; S-043 asserts the field is present and correct.
///
/// S-039 owns the config-load; S-043 asserts the field is present and correctly defaulted.
#[test]
fn test_BC_2_09_007_scrollback_rows_default_1000() {
    // Assert the canonical default helper.
    assert_eq!(
        default_scrollback_rows(),
        1000,
        "BC-2.09.007 Invariant 1: default_scrollback_rows() must return 1000 \
         (absent pty_scrollback_rows config key)"
    );

    // Assert App::new with None config wires scrollback_rows = 1000.
    let app = App::new(MonocleConfig {
        pty_scrollback_rows: None,
        ..MonocleConfig::default()
    });
    assert_eq!(
        app.scrollback_rows, 1000,
        "BC-2.09.007 Postcondition 1: App::scrollback_rows must be 1000 when \
         pty_scrollback_rows is absent from config"
    );

    // Assert a parser created with 1000 scrollback rows can accept scrollback.
    let mut parser = vt100::Parser::new(24, 80, 1000);
    // Feed enough lines to create scrollback history.
    for i in 0..50u8 {
        let line = format!("scrollback-line-{}\r\n", i);
        parser.process(line.as_bytes());
    }
    // effective_scrollback_max() > 0 proves the parser was initialized with a real scrollback
    // buffer. vt100 0.16.2 does not expose Screen::scrollback_len() publicly; we probe by
    // setting scrollback to a large sentinel and reading back the clamped result.
    let max = effective_scrollback_max(&mut parser);
    assert!(
        max > 0,
        "BC-2.09.007 Postcondition 1: vt100::Parser initialized with 1000-row scrollback \
         must have effective scrollback max > 0 after feeding content — got {}",
        max
    );
}

/// test_BC_2_09_007_scrollback_rows_capped_10000
///
/// Exercises BC-2.09.007 Invariant 2 and Edge Case EC-242:
///   Config `pty_scrollback_rows: Some(15000)` → `App::scrollback_rows == 10000` (clamped).
///
/// Also asserts the `clamp_scrollback_rows` pure helper at the upper boundary.
#[test]
fn test_BC_2_09_007_scrollback_rows_capped_10000() {
    // Assert clamp_scrollback_rows upper boundary.
    assert_eq!(
        clamp_scrollback_rows(15000),
        10000,
        "BC-2.09.007 Invariant 2 / EC-242: clamp_scrollback_rows(15000) must return 10000"
    );

    // Assert App::new wires the clamped value.
    let app = App::new(MonocleConfig {
        pty_scrollback_rows: Some(15000),
        ..MonocleConfig::default()
    });
    assert_eq!(
        app.scrollback_rows, 10000,
        "BC-2.09.007 Invariant 2 / EC-242: App::scrollback_rows must be 10000 when \
         config value 15000 is clamped to the maximum"
    );
}

/// test_BC_2_09_007_scrollback_rows_clamped_min_1
///
/// Exercises BC-2.09.007 Invariant 1 and Edge Case EC-243:
///   Config `pty_scrollback_rows: Some(0)` → `App::scrollback_rows == 1` (clamped to minimum).
///
/// A value of 0 means "no scrollback"; the spec requires clamping to 1, not defaulting
/// to 1000 (that would be wrong — the key IS present, just out-of-range).
#[test]
fn test_BC_2_09_007_scrollback_rows_clamped_min_1() {
    // Assert clamp_scrollback_rows lower boundary.
    assert_eq!(
        clamp_scrollback_rows(0),
        1,
        "BC-2.09.007 Invariant 1 / EC-243: clamp_scrollback_rows(0) must return 1 \
         (present key at 0 is clamped to minimum, not defaulted to 1000)"
    );

    // Assert App::new wires the clamped minimum.
    let app = App::new(MonocleConfig {
        pty_scrollback_rows: Some(0),
        ..MonocleConfig::default()
    });
    assert_eq!(
        app.scrollback_rows, 1,
        "BC-2.09.007 Invariant 1 / EC-243: App::scrollback_rows must be 1 when \
         pty_scrollback_rows=0 (clamped to minimum, not defaulted to 1000)"
    );
}

// ---------------------------------------------------------------------------
// AC-002: PtyScrollUp increments pty_scroll_offsets[focused_session_id]
// BC-2.09.007 Postcondition 2a
//
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_scrollup_increments_offset
///
/// Exercises BC-2.09.007 Postcondition 2a (AC-002):
///   PtyScrollUp × 10 increments pty_scroll_offsets[focused_session_id] to 10.
///   pty_scroll_offsets for a second session (not focused) is unaffected.
///
/// Canonical test vector from BC-2.09.007:
///   1100 lines of output; PtyScrollUp × 10 (session "s1" focused)
///   → pty_scroll_offsets["s1"] = 10; other sessions unchanged.
#[test]
fn test_BC_2_09_007_scrollup_increments_offset() {
    let s1 = "s1-scroll-up-001";
    let s2 = "s2-scroll-up-001";
    let mut app = make_app_in_embedded(s1);

    // Register a second session with its own offset (must remain unchanged).
    let parser2 = vt100::Parser::new(24, 80, app.scrollback_rows as usize);
    app.pty_parsers.insert(s2.to_string(), parser2);
    app.pty_scroll_offsets.insert(s2.to_string(), 0);

    // Feed enough content to s1 to ensure effective scrollback max >= 10.
    feed_lines(&mut app, s1, 50);

    // Verify precondition: s1 has at least 10 rows of scrollback history.
    let s1_max = {
        let parser = app.pty_parsers.get_mut(s1).unwrap();
        effective_scrollback_max(parser)
    };
    assert!(
        s1_max >= 10,
        "BC-2.09.007 AC-002 precondition: s1 must have at least 10 rows of scrollback \
         history to scroll up 10 steps; got {} rows",
        s1_max
    );

    // Pre-condition: both offsets start at 0.
    assert_eq!(
        scroll_offset(&app, s1),
        0,
        "BC-2.09.007 AC-002 precondition: s1 offset must be 0 before scroll-up"
    );
    assert_eq!(
        scroll_offset(&app, s2),
        0,
        "BC-2.09.007 AC-002 precondition: s2 offset must be 0 before scroll-up"
    );

    // Act: PtyScrollUp × 10.
    for _ in 0..10 {
        handle_pty_scroll_up(&mut app);
    }

    // Assert 1 (AC-002): s1 offset incremented to 10.
    assert_eq!(
        scroll_offset(&app, s1),
        10,
        "BC-2.09.007 Postcondition 2a / AC-002: pty_scroll_offsets[s1] must be 10 \
         after PtyScrollUp × 10"
    );

    // Assert 2 (AC-002 / Invariant 5): s2 offset unaffected.
    assert_eq!(
        scroll_offset(&app, s2),
        0,
        "BC-2.09.007 Postcondition 2a / AC-002: pty_scroll_offsets[s2] must remain 0 \
         — PtyScrollUp only affects the focused session's offset"
    );
}

// ---------------------------------------------------------------------------
// AC-003 / EC-241: PtyScrollDown at offset=0 is a no-op (floor 0)
// BC-2.09.007 Postcondition 2b
//
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_scrolldown_decrements_floor_0
///
/// Exercises BC-2.09.007 Postcondition 2b (AC-003) and Edge Case EC-241:
///   PtyScrollDown when offset == 0 is a complete no-op.
///   Offset stays at 0; no error; no panic; no IPC.
///
/// Canonical test vector:
///   PtyScrollDown when pty_scroll_offsets[focused_session_id] = 0
///   → offset stays 0; no error.
#[test]
fn test_BC_2_09_007_scrolldown_decrements_floor_0() {
    let session_id = "s1-scrolldown-floor";
    let mut app = make_app_in_embedded(session_id);

    // Pre-condition: offset is 0 (live tail).
    assert_eq!(
        scroll_offset(&app, session_id),
        0,
        "BC-2.09.007 EC-241 precondition: offset must start at 0"
    );

    // Act: PtyScrollDown while at floor.
    handle_pty_scroll_down(&mut app);

    // Assert 1 (AC-003 / EC-241): offset remains 0 — no underflow, no panic.
    assert_eq!(
        scroll_offset(&app, session_id),
        0,
        "BC-2.09.007 Postcondition 2b / AC-003 / EC-241: pty_scroll_offsets must remain 0 \
         after PtyScrollDown when already at live tail (floor 0)"
    );

    // Assert 2 (AC-013): a second PtyScrollDown is also a no-op.
    handle_pty_scroll_down(&mut app);
    assert_eq!(
        scroll_offset(&app, session_id),
        0,
        "BC-2.09.007 AC-013: repeated PtyScrollDown at floor must not underflow — stays 0"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / AC-012 / EC-240: scroll past scrollback_len() is clamped
// BC-2.09.007 Postcondition 2c
//
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_clamp_at_max
///
/// Exercises BC-2.09.007 Postcondition 2c (AC-004), AC-012, and Edge Case EC-240:
///   PtyScrollUp past scrollback_len() is clamped to the available scrollback rows.
///   No error. No panic.
///
/// After scrolling past the maximum, one more PtyScrollUp must leave the offset
/// unchanged at the maximum available value.
#[test]
fn test_BC_2_09_007_clamp_at_max() {
    let session_id = "s1-clamp-max";
    let mut app = make_app_in_embedded(session_id);

    // Feed enough lines to create a bounded scrollback history.
    feed_lines(&mut app, session_id, 100);

    // Determine the effective max scrollback rows available in the parser's history.
    // vt100 0.16.2 does not expose Screen::scrollback_len() publicly; we probe by
    // setting scrollback to a large sentinel and reading back the clamped result.
    let max_available = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };

    // Guard: if max_available == 0, this test cannot exercise clamping.
    assert!(
        max_available > 0,
        "BC-2.09.007 EC-240 precondition: effective scrollback max must be > 0 after feeding \
         content; got {} — parser must be initialized with non-zero scrollback_rows",
        max_available
    );

    // Act: scroll up far more than the available scrollback rows.
    // This MUST clamp, not overflow or wrap.
    for _ in 0..(max_available + 50) {
        handle_pty_scroll_up(&mut app);
    }

    // Assert (AC-004 / EC-240): offset is clamped at max_available.
    let final_offset = scroll_offset(&app, session_id);
    assert_eq!(
        final_offset, max_available,
        "BC-2.09.007 Postcondition 2c / AC-004 / EC-240: pty_scroll_offsets must be \
         clamped at effective scrollback max ({}) after scrolling past the top; got {}",
        max_available, final_offset
    );

    // Assert (AC-012): one more PtyScrollUp from the maximum leaves offset unchanged.
    handle_pty_scroll_up(&mut app);
    let after_extra = scroll_offset(&app, session_id);
    assert_eq!(
        after_extra, max_available,
        "BC-2.09.007 AC-012: offset must remain at max ({}) after PtyScrollUp when \
         already at the oldest scrollback row; got {}",
        max_available, after_extra
    );
}

// ---------------------------------------------------------------------------
// AC-005 / AC-011: focus switch preserves per-session offsets
// BC-2.09.007 Postcondition 2d / Invariant 5
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_focus_switch_preserves_offsets
///
/// Exercises BC-2.09.007 Postcondition 2d (AC-005) and Invariant 5 (AC-011):
///   Each session's scroll offset is stored independently.
///   Switching focus from s1 (offset=10) to s2 (offset=0):
///   - s1's offset remains 10 in pty_scroll_offsets["s1"].
///   - s2's offset is 0 in pty_scroll_offsets["s2"].
///   - The app mode now reflects s2 as the focused session.
///
/// Canonical test vector:
///   Focus switch from "s1" (offset=10) to "s2" (offset=0)
///   → pty_scroll_offsets["s1"] = 10 preserved; pty_scroll_offsets["s2"] = 0;
///     render uses pty_scroll_offsets["s2"] for new focused session.
#[test]
fn test_BC_2_09_007_focus_switch_preserves_offsets() {
    let s1 = "s1-focus-switch";
    let s2 = "s2-focus-switch";

    // Start focused on s1.
    let mut app = make_app_in_embedded(s1);

    // Register s2 with its own parser and offset.
    let parser2 = vt100::Parser::new(24, 80, app.scrollback_rows as usize);
    app.pty_parsers.insert(s2.to_string(), parser2);
    app.pty_scroll_offsets.insert(s2.to_string(), 0);

    // Feed content to s1 so scrollback_len() > 10.
    feed_lines(&mut app, s1, 50);

    // Scroll s1 up by 10.
    for _ in 0..10 {
        handle_pty_scroll_up(&mut app);
    }

    // Verify s1 is at 10 before the focus switch.
    assert_eq!(
        scroll_offset(&app, s1),
        10,
        "BC-2.09.007 AC-005 precondition: s1 offset must be 10 before focus switch"
    );

    // Simulate focus switch: transition app mode to EmbeddedTerminal for s2.
    // (The implementer wires this through the action dispatch; here we simulate
    // the mode transition directly as the test fixture for AC-005.)
    app.mode = AppMode::EmbeddedTerminal {
        session_id: s2.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Assert 1 (AC-005 / Invariant 5): s1 offset preserved at 10.
    assert_eq!(
        scroll_offset(&app, s1),
        10,
        "BC-2.09.007 Postcondition 2d / AC-005: pty_scroll_offsets[s1] must remain 10 \
         after focus switch — focus switch must NOT reset the leaving session's offset"
    );

    // Assert 2 (AC-005 / Invariant 5): s2 offset is 0.
    assert_eq!(
        scroll_offset(&app, s2),
        0,
        "BC-2.09.007 Postcondition 2d / AC-005: pty_scroll_offsets[s2] must be 0 \
         — focus switch must NOT reset the incoming session's offset (AC-011)"
    );

    // Assert 3 (AC-011): no singular shared offset — each session has its own entry.
    // pty_scroll_offsets must have SEPARATE entries for s1 and s2.
    assert!(
        app.pty_scroll_offsets.contains_key(s1),
        "BC-2.09.007 Invariant 5 / AC-011: pty_scroll_offsets must have an entry for s1 \
         (per-session HashMap, not a shared field)"
    );
    assert!(
        app.pty_scroll_offsets.contains_key(s2),
        "BC-2.09.007 Invariant 5 / AC-011: pty_scroll_offsets must have an entry for s2 \
         (per-session HashMap, not a shared field)"
    );
    assert_ne!(
        scroll_offset(&app, s1),
        scroll_offset(&app, s2),
        "BC-2.09.007 Invariant 5 / AC-011: s1 and s2 offsets must be independent values \
         (s1=10, s2=0)"
    );

    // Assert 4 (AC-005): the app mode correctly reflects s2 as the focused session.
    assert!(
        matches!(
            &app.mode,
            AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == s2
        ),
        "BC-2.09.007 AC-005: AppMode must reflect s2 as the focused session after switch"
    );
}

// ---------------------------------------------------------------------------
// AC-006: No IPC message sent for PtyScrollUp / PtyScrollDown
// BC-2.09.007 Postcondition 3
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_no_ipc_for_scroll
///
/// Exercises BC-2.09.007 Postcondition 3 (AC-006):
///   Neither PtyScrollUp nor PtyScrollDown sends any IPC message.
///   Specifically: no ClientToServer::ResizePane and no ClientToServer::KeyInput
///   appear in the IPC channel after calling the scroll handlers.
///
/// Scrollback navigation is a TUI-local viewport operation only.
#[tokio::test]
async fn test_BC_2_09_007_no_ipc_for_scroll() {
    // UUID-format session ID required: handle_pty_scroll_up/down use pty_parsers which
    // is keyed by session_id. The mode carries session_id as a String key.
    let session_id = "00000007-0000-4000-8007-000000000002";
    let mut app = make_app_in_embedded(session_id);

    // Wire a real bounded mpsc channel so we can inspect IPC sends.
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    // Feed content so scroll has something to operate on.
    feed_lines(&mut app, session_id, 50);

    // Act: PtyScrollUp × 5.
    for _ in 0..5 {
        handle_pty_scroll_up(&mut app);
    }

    // Act: PtyScrollDown × 3.
    for _ in 0..3 {
        handle_pty_scroll_down(&mut app);
    }

    // Assert (AC-006 / Postcondition 3): NO IPC message sent.
    // try_recv() must return Empty — scrollback is TUI-local.
    let result = cmd_rx.try_recv();
    assert!(
        matches!(result, Err(tokio::sync::mpsc::error::TryRecvError::Empty)),
        "BC-2.09.007 Postcondition 3 / AC-006: PtyScrollUp/Down must NOT send any IPC \
         message — scrollback is a TUI-local viewport operation. Got: {:?}",
        result.ok()
    );
}

// ---------------------------------------------------------------------------
// AC-007: Status bar indicator when scrolled back
// BC-2.09.007 Postcondition 4
//
// render_embedded_terminal returns usize (the effective scroll offset) for the
// caller to build the "[scrolled back N rows]" indicator.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_status_bar_indicator_when_scrolled
///
/// Exercises BC-2.09.007 Postcondition 4 (AC-007):
///   When pty_scroll_offsets[focused_session_id] > 0, render_embedded_terminal
///   returns an effective offset > 0, which the status bar uses to show
///   "[scrolled back N rows]". When offset == 0 (live tail), effective offset is 0.
///
/// This test drives render_embedded_terminal directly with two scroll offsets
/// (non-zero and zero) and asserts the returned effective offset accordingly.
#[test]
fn test_BC_2_09_007_status_bar_indicator_when_scrolled() {
    use monocle_tui::ui::embedded_terminal::render_embedded_terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    // Arrange: headless ratatui backend (80 cols × 24 rows).
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");

    // Create a parser with enough content for a non-zero scroll offset to be meaningful.
    let mut parser = vt100::Parser::new(24, 80, 1000);
    for i in 0..50u8 {
        let line = format!("content-line-{}\r\n", i);
        parser.process(line.as_bytes());
    }

    let max_history = effective_scrollback_max(&mut parser);
    assert!(
        max_history > 0,
        "BC-2.09.007 AC-007 precondition: effective scrollback max must be > 0 — parser must \
         be initialized with non-zero scrollback capacity. Got {}",
        max_history
    );

    // Test Case A: scroll_offset > 0 → effective_offset > 0 (indicator shown).
    let non_zero_offset = 5.min(max_history);
    let mut effective_offset_nonzero = 0usize;
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            effective_offset_nonzero =
                render_embedded_terminal(frame, area, &mut parser, non_zero_offset);
        })
        .expect("terminal.draw must succeed");

    assert!(
        effective_offset_nonzero > 0,
        "BC-2.09.007 Postcondition 4 / AC-007: render_embedded_terminal must return \
         effective_offset > 0 when scroll_offset={} is passed (indicator should be shown); \
         got effective_offset={}",
        non_zero_offset,
        effective_offset_nonzero
    );

    // Test Case B: scroll_offset = 0 → effective_offset = 0 (indicator absent).
    let mut effective_offset_zero = 1usize; // Pre-set to non-zero to detect failure.
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            effective_offset_zero = render_embedded_terminal(frame, area, &mut parser, 0);
        })
        .expect("terminal.draw must succeed");

    assert_eq!(
        effective_offset_zero, 0,
        "BC-2.09.007 Postcondition 4 / AC-007: render_embedded_terminal must return \
         effective_offset=0 when scroll_offset=0 (live tail; indicator must be absent)"
    );
}

// ---------------------------------------------------------------------------
// AC-008 / AC-014 / EC-244 / PC-5: content-anchored offset on new PTY output
// BC-2.09.007 Postcondition 5
//
// Replaces test_BC_2_09_007_new_output_does_not_reset_scroll_offset (ADV Pass-1).
//
// The prior test only asserted offset != 0 after new output (numeric-preserve
// semantics). The v1.4.0 BC requires CONTENT-ANCHORED semantics: the offset
// must INCREASE by the number of new scrollback rows produced by the process()
// call so that the viewport stays pinned to the same content rows.
//
// NOTE: on_pty_output validates that session_id is a well-formed UUID (SEC-004).
// Tests that exercise on_pty_output MUST use UUID-format session IDs.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_content_anchored_new_output
///
/// Exercises BC-2.09.007 Postcondition 5 / PC-5 (AC-008, AC-014, EC-244):
///   New PtyOutput while scrolled back (offset=10) MUST increment the offset
///   by K (the number of new scrollback rows produced), yielding offset = 10+K
///   (clamped to scrollback_max). Numeric-preserve (offset stays 10) is incorrect
///   per v1.4.0 semantics.
///
/// Canonical test vector (BC-2.09.007):
///   Scrolled to offset=10; 5 new rows of output arrive
///   → pty_scroll_offsets = min(15, scrollback_max).
#[tokio::test]
async fn test_BC_2_09_007_content_anchored_new_output() {
    // UUID-format session ID required: on_pty_output validates via Uuid::parse_str.
    let session_id = "00000007-0000-4000-8007-000000000001";
    let mut app = make_app_in_embedded(session_id);

    // Feed initial content so the parser has a scrollback history well above 10.
    // 50 lines into a 24-row screen: 26 rows go into scrollback history.
    feed_lines(&mut app, session_id, 50);

    // Read scrollback_max BEFORE new output arrives.
    let scrollback_max_before = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };
    assert!(
        scrollback_max_before >= 10,
        "BC-2.09.007 PC-5 precondition: scrollback history must be >= 10 rows after 50 lines; \
         got {}",
        scrollback_max_before
    );

    // Set scroll offset to 10 (simulating "already scrolled back").
    // Direct mutation bypasses handle_pty_scroll_up (which tests the scroll handler;
    // this test isolates the on_pty_output content-anchored update path).
    app.pty_scroll_offsets.insert(session_id.to_string(), 10);
    assert_eq!(
        scroll_offset(&app, session_id),
        10,
        "BC-2.09.007 PC-5 precondition: offset must be 10 before new output"
    );

    // Construct K=5 new lines. Each line is exactly one output row in the 80-col parser.
    // The new rows will push 5 new lines into scrollback history (parser is 24 rows tall;
    // each new line beyond the visible area scrolls one row into history).
    let new_line_count = 5usize;
    let new_bytes: Vec<u8> = (0..new_line_count)
        .flat_map(|i| format!("new-content-row-{}\r\n", i).into_bytes())
        .collect();

    // Measure scrollback_max before vs after to compute the actual K delta.
    // This avoids hardcoding assumptions about how many rows land in scrollback
    // (depends on current screen fullness). The BC specifies: new_offset = 10 + delta.
    let scrollback_max_before_process = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };

    // Act: new PtyOutput arrives while scrolled back.
    on_pty_output(&mut app, session_id.to_string(), new_bytes);

    // Measure scrollback_max after process() to compute the actual new-row delta.
    let scrollback_max_after_process = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };

    // K = new scrollback rows added by the process() call.
    // The BC algorithm: new_offset = min(old_offset + K, new_scrollback_max).
    let k = scrollback_max_after_process.saturating_sub(scrollback_max_before_process);
    let expected_offset = (10 + k).min(scrollback_max_after_process);

    // Assert 1 (PC-5 / AC-008 / EC-244): offset must be content-anchored (10 + K), NOT 10.
    let actual_offset = scroll_offset(&app, session_id);
    assert_eq!(
        actual_offset,
        expected_offset,
        "BC-2.09.007 Postcondition 5 / PC-5 / AC-008 / EC-244: offset must be content-anchored \
         (old_offset + K = 10 + {} = {}, clamped to {}); got {}. \
         Numeric-preserve (offset stays 10) is WRONG — the viewport drifts toward newer content.",
        k,
        10 + k,
        scrollback_max_after_process,
        actual_offset
    );

    // Assert 2 (PC-5): the offset is NOT the raw numeric-preserve value (10) when K > 0.
    // If K == 0, the new bytes produced no new scrollback rows (e.g., the output did not
    // scroll the screen). In that edge case numeric-preserve and content-anchored are
    // equivalent (10 + 0 = 10), so we only enforce the distinction when K > 0.
    if k > 0 {
        assert_ne!(
            actual_offset, 10,
            "BC-2.09.007 PC-5: offset must NOT stay at the numeric-preserve value 10 \
             when K={} new scrollback rows arrived — must be content-anchored at {}",
            k, expected_offset
        );
    }

    // Assert 3 (AC-014): parser was updated with new bytes (content is accessible).
    let parser = app
        .pty_parsers
        .get_mut(session_id)
        .expect("BC-2.09.007 AC-014: parser must still exist after PtyOutput");
    parser.screen_mut().set_scrollback(0);
    let live_contents = parser.screen().contents();
    parser.screen_mut().set_scrollback(usize::MAX);
    let all_contents = parser.screen().contents();
    parser.screen_mut().set_scrollback(0);

    assert!(
        live_contents.contains("new-content-row-") || all_contents.contains("new-content-row-"),
        "BC-2.09.007 AC-014: parser must be updated with new bytes even while scrolled back; \
         'new-content-row-' must appear in live or scrollback content. Live: {:?}",
        live_contents.trim_end()
    );

    // Assert 4 (AC-014 indicator): offset is > 0 → scrollback indicator would still show.
    assert!(
        scroll_offset(&app, session_id) > 0,
        "BC-2.09.007 AC-014: pty_scroll_offsets must remain > 0 after content-anchored update \
         — status bar indicator must still show"
    );
}

// ---------------------------------------------------------------------------
// AC-009 / Invariant 3a: ResizePane for session resets scroll offset to 0
// BC-2.09.007 Invariant 3a
//
// The ResizePane reset is owned by S-042. This test asserts that S-042
// implemented it correctly, and catches any regression introduced by S-043.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_resize_resets_scroll_offset_to_zero
///
/// Exercises BC-2.09.007 Invariant 3a (AC-009):
///   When a resize event fires for session_id, pty_scroll_offsets[session_id] is
///   reset to 0 (live tail). Rationale: resize reflows content; old offset is
///   meaningless against the new layout.
///
/// The scroll offset reset is performed by `on_resize_detected` (S-042 delivery),
/// which is the TUI-local handler that runs when the pane area changes. This test
/// asserts the S-042 implementation satisfies the BC-2.09.007 Invariant 3a contract
/// as required by AC-009.
#[test]
fn test_BC_2_09_007_resize_resets_scroll_offset_to_zero() {
    let session_id = "s1-resize-reset";
    let mut app = make_app_with_session(session_id);

    // Directly set offset to non-zero to simulate a scrolled-back state.
    app.pty_scroll_offsets.insert(session_id.to_string(), 7);
    assert_eq!(
        scroll_offset(&app, session_id),
        7,
        "BC-2.09.007 AC-009 precondition: offset must be 7 before resize"
    );

    // Put app in EmbeddedTerminal mode for the session.
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Act: drive a resize detection event through on_resize_detected.
    // SS-embedded-pty.md §Scrollback offset invariants: resize reflows content;
    // the offset reset to 0 happens in on_resize_detected (S-042 delivery).
    on_resize_detected(&mut app, session_id, 30, 100);

    // Assert (Invariant 3a / AC-009): offset reset to 0.
    assert_eq!(
        scroll_offset(&app, session_id),
        0,
        "BC-2.09.007 Invariant 3a / AC-009: pty_scroll_offsets[session_id] must be 0 \
         after on_resize_detected — resize reflows content; old offset is meaningless"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / Invariant 3b: Terminated session removes scroll offset entry
// BC-2.09.007 Invariant 3b
//
// gc_pty_session is already implemented (S-039). This test asserts the removal
// semantics from the S-043 perspective: the entry must be REMOVED, not reset to 0.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_terminated_session_removes_scroll_entry
///
/// Exercises BC-2.09.007 Invariant 3b (AC-010):
///   When a session transitions to Terminated and is GC'd,
///   pty_scroll_offsets.remove(session_id) is called.
///   The entry is REMOVED (not reset to 0).
///
/// This test drives gc_pty_session directly (the S-039 production function).
#[test]
fn test_BC_2_09_007_terminated_session_removes_scroll_entry() {
    let session_id = "s1-gc-removes-offset";
    let mut app = make_app_with_session(session_id);

    // Set offset to non-zero to distinguish "removed" from "reset to 0".
    app.pty_scroll_offsets.insert(session_id.to_string(), 15);
    assert_eq!(
        scroll_offset(&app, session_id),
        15,
        "BC-2.09.007 AC-010 precondition: offset must be 15 before GC"
    );

    // Pre-condition: entry exists.
    assert!(
        app.pty_scroll_offsets.contains_key(session_id),
        "BC-2.09.007 AC-010 precondition: pty_scroll_offsets must have entry for session"
    );

    // Act: GC the session (Terminated path).
    gc_pty_session(&mut app, session_id);

    // Assert 1 (Invariant 3b / AC-010): entry REMOVED (not reset to 0).
    assert!(
        !app.pty_scroll_offsets.contains_key(session_id),
        "BC-2.09.007 Invariant 3b / AC-010: pty_scroll_offsets.remove(session_id) must be called \
         on GC — the entry must be REMOVED, not reset to 0"
    );

    // Assert 2: parser also removed (cross-check with GC completeness).
    assert!(
        !app.pty_parsers.contains_key(session_id),
        "BC-2.09.007 AC-010 (GC completeness): pty_parsers must also be removed on GC"
    );
}

// ---------------------------------------------------------------------------
// AC-011 / Invariant 5: pty_scroll_offsets is a HashMap, NOT a shared field
// BC-2.09.007 Invariant 5
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_no_singular_shared_offset_field
///
/// Exercises BC-2.09.007 Invariant 5 (AC-011):
///   pty_scroll_offsets is canonically HashMap<String, usize> keyed by session_id.
///   There is NO shared singular pty_scroll_offset: usize field.
///   This is the I7 fix: a shared offset caused focus-switch to show the wrong
///   session's scrollback position.
///
/// This test verifies that:
///   1. pty_scroll_offsets is a HashMap (multiple entries coexist independently).
///   2. Two sessions can have different offsets simultaneously.
///   3. Modifying one session's offset does not affect the other's entry.
#[test]
fn test_BC_2_09_007_no_singular_shared_offset_field() {
    let s1 = "s1-hashmap-test";
    let s2 = "s2-hashmap-test";
    let s3 = "s3-hashmap-test";
    let mut app = make_app();

    // Create parsers for three sessions.
    for sid in &[s1, s2, s3] {
        let parser = vt100::Parser::new(24, 80, app.scrollback_rows as usize);
        app.pty_parsers.insert(sid.to_string(), parser);
        app.pty_scroll_offsets.insert(sid.to_string(), 0);
    }

    // Set distinct offsets for each session.
    app.pty_scroll_offsets.insert(s1.to_string(), 3);
    app.pty_scroll_offsets.insert(s2.to_string(), 7);
    app.pty_scroll_offsets.insert(s3.to_string(), 0);

    // Assert 1 (Invariant 5): all three sessions have their own entries.
    assert_eq!(
        scroll_offset(&app, s1),
        3,
        "BC-2.09.007 Invariant 5: s1 offset must be 3 (per-session HashMap)"
    );
    assert_eq!(
        scroll_offset(&app, s2),
        7,
        "BC-2.09.007 Invariant 5: s2 offset must be 7 (per-session HashMap)"
    );
    assert_eq!(
        scroll_offset(&app, s3),
        0,
        "BC-2.09.007 Invariant 5: s3 offset must be 0 (per-session HashMap)"
    );

    // Assert 2 (Invariant 5): modifying s2 does not affect s1 or s3.
    app.pty_scroll_offsets.insert(s2.to_string(), 99);
    assert_eq!(
        scroll_offset(&app, s1),
        3,
        "BC-2.09.007 Invariant 5: s1 offset must remain 3 after s2 is modified"
    );
    assert_eq!(
        scroll_offset(&app, s3),
        0,
        "BC-2.09.007 Invariant 5: s3 offset must remain 0 after s2 is modified"
    );
    assert_eq!(
        scroll_offset(&app, s2),
        99,
        "BC-2.09.007 Invariant 5: s2 offset must be 99 after update"
    );

    // Assert 3 (Invariant 5): three distinct entries exist (not a shared field).
    assert_eq!(
        app.pty_scroll_offsets.len(),
        3,
        "BC-2.09.007 Invariant 5: pty_scroll_offsets must have exactly 3 entries \
         (one per session — this is the I7 fix confirming the HashMap is used)"
    );
}

// ---------------------------------------------------------------------------
// Render wiring: render_embedded_terminal called with scroll_offset from App
// BC-2.09.007 Postcondition 1 + Postcondition 4 (render path)
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_render_embedded_terminal_with_scroll_offset
///
/// Exercises the S-043 extension to render_embedded_terminal:
///   render_embedded_terminal(frame, area, &mut parser, scroll_offset) applies
///   `parser.screen_mut().set_scrollback(scroll_offset)` before rendering.
///
///   The vt100 API: set_scrollback(N) where N=0 is live tail, N>0 scrolls up.
///   After set_scrollback(N), parser.screen().scrollback() returns the clamped
///   effective value. The function must return the effective scroll offset so the
///   caller can build the "[scrolled back N rows]" status bar indicator.
///
/// This test verifies the render function sets scrollback on the parser screen
/// and returns the effective (vt100-clamped) offset.
#[test]
fn test_BC_2_09_007_render_embedded_terminal_with_scroll_offset() {
    use monocle_tui::ui::embedded_terminal::render_embedded_terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    // Arrange: headless terminal.
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");

    // Parser with content to verify scrollback renders distinct rows.
    let mut parser = vt100::Parser::new(24, 80, 1000);
    for i in 0..50u8 {
        let line = format!("render-line-{}\r\n", i);
        parser.process(line.as_bytes());
    }

    let max_scrollback = effective_scrollback_max(&mut parser);
    assert!(
        max_scrollback > 0,
        "BC-2.09.007 render precondition: effective scrollback max must be > 0 after feeding \
         content — parser must be initialized with non-zero scrollback capacity"
    );

    // Case A: scroll_offset = 0 (live tail) → effective offset = 0.
    let mut effective_at_zero = 999usize;
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            effective_at_zero = render_embedded_terminal(frame, area, &mut parser, 0);
        })
        .expect("terminal.draw must succeed at scroll_offset=0");

    assert_eq!(
        effective_at_zero, 0,
        "BC-2.09.007 render: effective scroll offset must be 0 when scroll_offset=0 \
         (live tail — set_scrollback(0) returns scrollback()=0)"
    );

    // Case B: scroll_offset = 5 → effective offset = 5 (within range).
    let request_offset = 5.min(max_scrollback);
    let mut effective_at_five = 999usize;
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            effective_at_five = render_embedded_terminal(frame, area, &mut parser, request_offset);
        })
        .expect("terminal.draw must succeed at scroll_offset=5");

    assert_eq!(
        effective_at_five, request_offset,
        "BC-2.09.007 render: effective scroll offset must be {} when scroll_offset={} \
         (within scrollback range — set_scrollback clamps and screen().scrollback() reflects it)",
        request_offset, request_offset
    );

    // Case C: scroll_offset beyond max → effective offset clamped to max_scrollback.
    let oversized_offset = max_scrollback + 500;
    let mut effective_clamped = 0usize;
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            effective_clamped =
                render_embedded_terminal(frame, area, &mut parser, oversized_offset);
        })
        .expect("terminal.draw must succeed at oversized scroll_offset");

    assert_eq!(
        effective_clamped, max_scrollback,
        "BC-2.09.007 render / EC-240: effective scroll offset must be clamped to \
         scrollback_len()={} when scroll_offset={} exceeds the buffer",
        max_scrollback, oversized_offset
    );
}

// ---------------------------------------------------------------------------
// AC-007 / HIGH-003: status bar rendered buffer literally contains "[scrolled back N rows]"
// BC-2.09.007 Postcondition 4
//
// test_BC_2_09_007_status_bar_indicator_when_scrolled asserts the usize RETURN VALUE
// of render_embedded_terminal. This test also scrapes the rendered status bar string
// via render_frame (the full production render path) to confirm the badge string appears.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_status_bar_string_rendered
///
/// Exercises BC-2.09.007 Postcondition 4 / AC-007 (ADV Pass-1 HIGH-003):
///   When pty_scroll_offsets[focused] > 0, the rendered terminal buffer (via
///   render_frame + TestBackend) literally contains `[scrolled back N rows]`
///   with the correct N.
///
///   When pty_scroll_offsets[focused] == 0 (live tail), the string is ABSENT
///   from the rendered buffer. This also catches spurious rendering of the
///   indicator at live tail.
#[test]
fn test_BC_2_09_007_status_bar_string_rendered() {
    // Arrange: App in EmbeddedTerminal mode with a scrolled-back session.
    let session_id = "s1-status-bar-string";
    let mut app = make_app_in_embedded(session_id);

    // Feed enough content to create scrollback history so set_scrollback(5) is non-zero.
    feed_lines(&mut app, session_id, 50);

    // Verify precondition: effective scrollback max >= 5.
    let max_sb = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };
    assert!(
        max_sb >= 5,
        "BC-2.09.007 status-bar string precondition: effective scrollback max must be >= 5; \
         got {}",
        max_sb
    );

    // --- Case A: scrolled back 5 rows → indicator must be present ---
    let test_offset: usize = 5;
    app.pty_scroll_offsets
        .insert(session_id.to_string(), test_offset);

    let terminal_a = render_app_to_terminal(&mut app);
    let text_a = buffer_text(&terminal_a);

    let expected_badge = format!("[scrolled back {} rows]", test_offset);
    assert!(
        text_a.contains(&expected_badge),
        "BC-2.09.007 Postcondition 4 / AC-007: rendered status bar must contain '{}' \
         when pty_scroll_offsets[focused] = {}; rendered buffer (first 300 chars): {:?}",
        expected_badge,
        test_offset,
        &text_a[..text_a.len().min(300)]
    );

    // --- Case B: live tail (offset=0) → indicator must be ABSENT ---
    app.pty_scroll_offsets.insert(session_id.to_string(), 0);

    let terminal_b = render_app_to_terminal(&mut app);
    let text_b = buffer_text(&terminal_b);

    // The literal badge string must not appear when at live tail.
    assert!(
        !text_b.contains("[scrolled back"),
        "BC-2.09.007 Postcondition 4 / AC-007: '[scrolled back' must NOT appear in \
         the rendered status bar when pty_scroll_offsets[focused] == 0 (live tail); \
         found in buffer: {:?}",
        text_b.lines().find(|l| l.contains("[scrolled back"))
    );
}

// ---------------------------------------------------------------------------
// AC-007 / PC-4 / EC-245: scrollback indicator and dump-drop badge coexist
// BC-2.09.007 Postcondition 4 (concurrent-badge mandate from v1.4.0)
//
// render_frame concatenates both badges when both conditions hold (PC-4).
// The scrollback indicator (persistent viewport state) is NEVER suppressed
// by any transient diagnostic badge.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_concurrent_status_bar_badges
///
/// Exercises BC-2.09.007 Postcondition 4 / PC-4 / EC-245:
///   When the focused session is scrolled back (offset > 0) AND a dump-drop
///   condition is active, BOTH `[scrolled back N rows]` AND `[dump: N drops]`
///   must appear in the rendered status bar simultaneously. Neither suppresses
///   the other.
#[test]
fn test_BC_2_09_007_concurrent_status_bar_badges() {
    // Arrange: App in EmbeddedTerminal with scrolled-back session AND dump-drop active.
    let session_id = "s1-concurrent-badge";
    let mut app = make_app_in_embedded(session_id);

    // Feed content to create scrollback history.
    feed_lines(&mut app, session_id, 50);

    let max_sb = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };
    assert!(
        max_sb >= 5,
        "BC-2.09.007 PC-4 precondition: scrollback max must be >= 5; got {}",
        max_sb
    );

    // Set scrolled-back offset.
    let scroll_back: usize = 5;
    app.pty_scroll_offsets
        .insert(session_id.to_string(), scroll_back);

    // Activate dump-drop condition: dump_in_progress = true AND pending_pty_drop_count = 3.
    app.dump_in_progress.insert(session_id.to_string(), true);
    app.pending_pty_drop_count.insert(session_id.to_string(), 3);

    // Act: render via render_frame.
    let terminal = render_app_to_terminal(&mut app);
    let text = buffer_text(&terminal);

    // Assert 1 (PC-4 / EC-245): scrollback indicator present.
    let scrollback_badge = format!("[scrolled back {} rows]", scroll_back);
    assert!(
        text.contains(&scrollback_badge),
        "BC-2.09.007 PC-4 / EC-245: rendered status bar must contain '{}' when scrolled back \
         even with dump-drop active; rendered buffer (first 400 chars): {:?}",
        scrollback_badge,
        &text[..text.len().min(400)]
    );

    // Assert 2 (PC-4 / EC-245): dump-drop badge present.
    let dump_badge = "[dump: 3 drops]";
    assert!(
        text.contains(dump_badge),
        "BC-2.09.007 PC-4 / EC-245: rendered status bar must contain '{}' when dump-drop \
         is active even with scrollback indicator also active; rendered buffer (first 400 chars): {:?}",
        dump_badge,
        &text[..text.len().min(400)]
    );

    // Assert 3 (PC-4): BOTH appear on the same render — neither suppresses the other.
    // (The two individual assertions above together prove concurrent rendering.)
}

// ---------------------------------------------------------------------------
// ADV Pass-2 F-S043-P2-MED-001: status_message coexists with scrollback indicator
// BC-2.09.007 Postcondition 4 / PC-4 (concurrent diagnostic badge mandate)
//
// Defect: in the EmbeddedTerminal render branch, the status bar is composed as:
//   pty_status_owned = Some(scrollback_indicator_string)  [when offset > 0]
//   pty_status_msg = pty_status_owned.as_deref().or(app.status_message.as_deref())
//
// The .or() silently drops app.status_message whenever pty_status_owned is Some,
// which is the case when the scrollback indicator is active (offset > 0). This
// suppresses transient diagnostic messages set by on_dump_window_timeout (and similar
// handlers) whenever the user is scrolled back — a silent correctness failure.
//
// BC-2.09.007 PC-4 mandates that ALL concurrent diagnostic badges coexist in the
// status bar. The scrollback indicator being present does NOT excuse dropping
// app.status_message; both must appear simultaneously.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_status_message_coexists_with_scrollback
///
/// Exercises BC-2.09.007 Postcondition 4 / PC-4 (ADV Pass-2 F-S043-P2-MED-001):
///   When the focused session is scrolled back (offset > 0) AND app.status_message
///   is set to a transient diagnostic warning (e.g., the timeout notification set
///   by on_dump_window_timeout), BOTH the `[scrolled back N rows]` indicator AND the
///   status_message warn text must appear in the rendered status bar simultaneously.
///
///   The `.or()` pattern in the current implementation drops app.status_message
///   whenever pty_status_owned is Some (i.e., whenever the scrollback indicator is
///   active). This test FAILS against that current code, confirming RED gate.
///
///   The scenario: on_dump_window_timeout sets app.status_message to
///   "[warn] scrollback dump timed out for <session_id>", then clears dump_in_progress.
///   If the user is already scrolled back (offset > 0), the next render will have
///   pty_status_owned = Some("[scrolled back N rows]") and the .or() will prevent
///   the warn message from ever reaching the status bar.
#[test]
fn test_BC_2_09_007_status_message_coexists_with_scrollback() {
    // Arrange: App in EmbeddedTerminal with scrolled-back session.
    let session_id = "s1-status-msg-coexist";
    let mut app = make_app_in_embedded(session_id);

    // Feed enough content to create a valid scrollback history.
    feed_lines(&mut app, session_id, 50);

    let max_sb = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };
    assert!(
        max_sb >= 5,
        "BC-2.09.007 PC-4 precondition: scrollback max must be >= 5; got {}",
        max_sb
    );

    // Set scrolled-back offset (offset > 0 causes scrollback_indicator to be Some).
    let scroll_back: usize = 5;
    app.pty_scroll_offsets
        .insert(session_id.to_string(), scroll_back);

    // Simulate the state left by on_dump_window_timeout:
    //   - dump_in_progress is REMOVED (step 1 of on_dump_window_timeout)
    //   - app.status_message is set to the canonical warn string (step 7)
    // dump_in_progress is NOT set → dump_drop_status will be None.
    // Only the scrollback indicator (pty_status_owned) is Some, so the .or() wins
    // and drops the status_message under the current (buggy) implementation.
    let warn_msg = format!("[warn] scrollback dump timed out for {}", session_id);
    app.status_message = Some(warn_msg.clone());

    // Act: render via render_frame.
    let terminal = render_app_to_terminal(&mut app);
    let text = buffer_text(&terminal);

    // Assert 1 (PC-4): scrollback indicator must be present in the rendered status bar.
    let scrollback_badge = format!("[scrolled back {} rows]", scroll_back);
    assert!(
        text.contains(&scrollback_badge),
        "BC-2.09.007 PC-4 / F-S043-P2-MED-001: rendered status bar must contain '{}' \
         when scrolled back; rendered buffer (first 400 chars): {:?}",
        scrollback_badge,
        &text[..text.len().min(400)]
    );

    // Assert 2 (PC-4 / F-S043-P2-MED-001): app.status_message warn text must ALSO be
    // present. The .or() pattern drops it when pty_status_owned is Some — this assert
    // is the RED gate assertion. It will FAIL against the current implementation.
    assert!(
        text.contains(&warn_msg),
        "BC-2.09.007 PC-4 / F-S043-P2-MED-001: rendered status bar must contain the \
         app.status_message warn text '{}' even when scrollback indicator is also active; \
         the .or() pattern silently drops status_message when pty_status_owned is Some — \
         both must coexist per PC-4 concurrent-badge mandate. \
         Rendered buffer (first 400 chars): {:?}",
        warn_msg,
        &text[..text.len().min(400)]
    );
}

// ---------------------------------------------------------------------------
// AC-002 / AC-006 / PC-3 / BLOCKER-001: Ctrl+Up in EmbeddedTerminal increments
// pty_scroll_offsets and sends NO IPC — via handle_crossterm_event dispatch path.
// BC-2.09.007 Postcondition 2a + Postcondition 3
//
// handle_crossterm_event intercepts scroll bindings (Ctrl+Up → PtyScrollUp,
// Ctrl+Down → PtyScrollDown) BEFORE dispatch_embedded_terminal_key, so scroll
// actions are TUI-local and no IPC is sent.
// ---------------------------------------------------------------------------

/// test_BC_2_09_007_ctrl_up_dispatch_scrolls_no_ipc
///
/// Exercises BC-2.09.007 Postcondition 2a (AC-002) and Postcondition 3 (AC-006 / PC-3)
/// via the production handle_crossterm_event dispatch path (BLOCKER-001):
///   - Feeding a real crossterm Ctrl+Up KeyEvent through handle_crossterm_event
///     in AppMode::EmbeddedTerminal must increment pty_scroll_offsets[focused] by 1.
///   - No `ClientToServer::KeyInput` and no `ClientToServer::ResizePane` may be sent.
#[tokio::test]
async fn test_BC_2_09_007_ctrl_up_dispatch_scrolls_no_ipc() {
    // UUID-format session ID: handle_crossterm_event calls dispatch_embedded_terminal_key
    // which sends ClientToServer::KeyInput using the session_id string directly.
    let session_id = "00000007-0000-4000-8007-000000000003";
    let mut app = make_app_in_embedded(session_id);

    // Wire a real bounded IPC channel so we can inspect outbound sends.
    let (ipc_tx, mut ipc_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(ipc_tx);

    // Feed content so there is scrollback history to scroll into.
    feed_lines(&mut app, session_id, 50);

    // Verify precondition: scrollback max >= 1.
    let max_sb = {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        effective_scrollback_max(parser)
    };
    assert!(
        max_sb >= 1,
        "BC-2.09.007 BLOCKER-001 precondition: scrollback max must be >= 1; got {}",
        max_sb
    );

    // Pre-condition: offset starts at 0.
    assert_eq!(
        scroll_offset(&app, session_id),
        0,
        "BC-2.09.007 BLOCKER-001 precondition: offset must be 0 before Ctrl+Up"
    );

    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act: feed a real crossterm Ctrl+Up key event through the production dispatch path.
    let _ = handle_crossterm_event(
        &mut app,
        ctrl_up_event(),
        &binding_layers,
        &mut sessions_state,
    )
    .await;

    // Assert 1 (AC-002 / Postcondition 2a): offset INCREMENTED by 1.
    let after_offset = scroll_offset(&app, session_id);
    assert_eq!(
        after_offset, 1,
        "BC-2.09.007 AC-002 / Postcondition 2a: pty_scroll_offsets[focused] must be 1 \
         after Ctrl+Up via handle_crossterm_event; got {}. \
         BLOCKER-001: EmbeddedTerminal dispatch intercepts Ctrl+Up → PtyScrollUp \
         BEFORE forwarding to dispatch_embedded_terminal_key.",
        after_offset
    );

    // Assert 2 (AC-006 / Postcondition 3 / PC-3): NO IPC sent.
    let sent = drain_ipc(&mut ipc_rx);
    assert!(
        sent.is_empty(),
        "BC-2.09.007 Postcondition 3 / AC-006: Ctrl+Up must NOT send any IPC message; \
         got {} message(s): {:?}",
        sent.len(),
        sent.iter()
            .map(|m| format!("{:?}", m))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// test_BC_2_09_007_ctrl_down_dispatch_scrolls_no_ipc
///
/// Symmetric test for Ctrl+Down. Exercises BC-2.09.007 Postcondition 2b (AC-003)
/// and Postcondition 3 (AC-006 / PC-3) via handle_crossterm_event:
///   - Start with offset=5; Ctrl+Down decrements to 4.
///   - No IPC sent.
#[tokio::test]
async fn test_BC_2_09_007_ctrl_down_dispatch_scrolls_no_ipc() {
    let session_id = "00000007-0000-4000-8007-000000000004";
    let mut app = make_app_in_embedded(session_id);

    let (ipc_tx, mut ipc_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(ipc_tx);

    feed_lines(&mut app, session_id, 50);

    // Pre-set offset to 5 so Ctrl+Down has room to decrement.
    app.pty_scroll_offsets.insert(session_id.to_string(), 5);
    assert_eq!(
        scroll_offset(&app, session_id),
        5,
        "BC-2.09.007 BLOCKER-001 precondition (ctrl-down): offset must be 5 before Ctrl+Down"
    );

    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act: feed a real crossterm Ctrl+Down key event.
    let _ = handle_crossterm_event(
        &mut app,
        ctrl_down_event(),
        &binding_layers,
        &mut sessions_state,
    )
    .await;

    // Assert 1 (AC-003 / Postcondition 2b): offset DECREMENTED to 4.
    let after_offset = scroll_offset(&app, session_id);
    assert_eq!(
        after_offset, 4,
        "BC-2.09.007 AC-003 / Postcondition 2b: pty_scroll_offsets[focused] must be 4 \
         after Ctrl+Down (from 5) via handle_crossterm_event; got {}. \
         BLOCKER-001: EmbeddedTerminal dispatch routes Ctrl+Down → PtyScrollDown.",
        after_offset
    );

    // Assert 2 (AC-006 / Postcondition 3): NO IPC sent.
    let sent = drain_ipc(&mut ipc_rx);
    assert!(
        sent.is_empty(),
        "BC-2.09.007 Postcondition 3 / AC-006: Ctrl+Down must NOT send any IPC message; \
         got {} message(s)",
        sent.len()
    );
}
