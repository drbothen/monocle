//! AC-012 integration render test for S-027.
//!
//! Drives `App::render_frame` (the production render path) against a
//! `ratatui::backend::TestBackend` and asserts that the rendered terminal buffer
//! contains the correct status bar and overlay content.
//!
//! # Why this test matters (BLOCKER-1)
//!
//! `render_status_bar` is fully implemented and unit-tested in isolation via
//! `monocle-tui/tests/overlay_render.rs`. But `render_frame` in `app.rs` still
//! calls the LEGACY 1-row `Paragraph::new(status_line)` renderer — it does NOT
//! call `render_status_bar`. Unit tests on `render_status_bar` alone would pass
//! even if the production path is broken. This test exercises the PRODUCTION PATH
//! so the BLOCKER cannot be hidden by unit-test-only coverage.
//!
//! # Canonical BC coverage
//!
//! - BC-2.06.019 PC-1: mode indicator present in rendered buffer.
//! - BC-2.06.020 PC-1: breadcrumb `Dashboard > Sessions` on second-to-last row.
//! - BC-2.06.021 PC-1: hint line `Tab: cycle  Enter: fullscreen  /: filter ...` on last row.
//! - BC-2.06.010 PC-1: overlay modal header `Permission Request` when AppMode::Overlay.
//! - BC-2.06.021 PC-1: Overlay hint line present on last row when AppMode::Overlay.
//!
//! # Red Gate
//!
//! These tests MUST FAIL against the current implementation because `render_frame`
//! renders the legacy 1-row `Paragraph` (not `render_status_bar`), so the breadcrumb
//! and hint line do NOT appear in the terminal buffer. The tests will pass only after
//! the implementer wires `render_status_bar` into `render_frame` (AC-012).
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot, PromptModal, ToolPayload};
use monocle_tui::app::{render_frame, App};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use ratatui::{backend::TestBackend, Terminal};
use std::time::Instant;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal `App` in Dashboard mode.
fn dashboard_app() -> App {
    App::new(MonocleConfig::default())
}

/// Build a minimal `App` in Overlay mode with one Bash modal in the stack.
fn overlay_app() -> App {
    let mut app = App::new(MonocleConfig::default());
    app.overlay_stack.push_back(PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "sess-integration-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_payload: ToolPayload::Bash {
            command: "cargo test --workspace".to_string(),
        },
        received_at: Instant::now(),
    });
    app.mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    app
}

/// Collect all cell symbols from a `Terminal<TestBackend>` buffer as a single string.
fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

/// Collect a single row from the buffer as a string.
fn row_text(terminal: &Terminal<TestBackend>, row: u16) -> String {
    let buf = terminal.backend().buffer().clone();
    let width = buf.area().width;
    (0..width)
        .map(|x| buf[(x, row)].symbol().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// AC-012 / BC-2.06.020 PC-1 — Dashboard mode: breadcrumb present in render_frame output
// ---------------------------------------------------------------------------

/// AC-012 / BC-2.06.020 PC-1 (RED GATE): `render_frame` in Dashboard mode must render the
/// breadcrumb string `"Dashboard > Sessions"` in the second-to-last terminal row.
///
/// The BLOCKER: the current `render_frame` renders the legacy 1-row `Paragraph` (the
/// `MONOCLE_STATUS_LABEL` string) via `Widget::render(Paragraph::new(status_line), ...)`.
/// It does NOT call `render_status_bar`. Therefore the breadcrumb `"Dashboard > Sessions"`
/// does NOT appear in the terminal buffer. This test FAILS against the broken code.
#[test]
fn test_BC_2_06_020_render_frame_dashboard_mode_breadcrumb_appears_in_buffer() {
    let app = dashboard_app();
    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_frame(&app, &mut sessions_state, frame);
        })
        .expect("render_frame must not panic");

    let all_text = buffer_text(&terminal);

    // BC-2.06.020 PC-1: breadcrumb must appear somewhere in the rendered buffer.
    assert!(
        all_text.contains("Dashboard > Sessions"),
        "AC-012 / BC-2.06.020 PC-1: render_frame must render breadcrumb 'Dashboard > Sessions' \
         in the terminal buffer; the broken impl renders the legacy 1-row Paragraph which does \
         NOT contain the breadcrumb"
    );
}

/// AC-012 / BC-2.06.021 PC-1 (RED GATE): `render_frame` in Dashboard mode must render the
/// hint line `"Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit"` in the
/// LAST terminal row.
///
/// The BLOCKER: the current `render_frame` renders the legacy Paragraph which shows
/// `"monocle"` (MONOCLE_STATUS_LABEL) — not the hint line. This test FAILS.
#[test]
fn test_BC_2_06_021_render_frame_dashboard_mode_hint_line_appears_on_last_row() {
    let app = dashboard_app();
    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_frame(&app, &mut sessions_state, frame);
        })
        .expect("render_frame must not panic");

    // Last row = row 23 (for a 24-row terminal).
    let last_row = row_text(&terminal, 23);
    let last_row_trimmed = last_row.trim_end();

    assert_eq!(
        last_row_trimmed, "Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit",
        "AC-012 / BC-2.06.021 PC-1: the last terminal row (hint row) must contain the \
         EXACT Dashboard hint line; the broken impl shows 'monocle' (MONOCLE_STATUS_LABEL) \
         instead of the hint line"
    );
}

/// AC-012 / BC-2.06.010 PC-1 (RED GATE): `render_frame` in Overlay mode must render the
/// modal header `"Permission Request"` in the terminal buffer.
///
/// This verifies that the overlay widget is wired into `render_frame` and the
/// `render_overlay_widget` call path works end-to-end through the production renderer.
#[test]
fn test_BC_2_06_010_render_frame_overlay_mode_modal_header_appears_in_buffer() {
    let app = overlay_app();
    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_frame(&app, &mut sessions_state, frame);
        })
        .expect("render_frame must not panic");

    let all_text = buffer_text(&terminal);

    assert!(
        all_text.contains("Permission Request"),
        "AC-012 / BC-2.06.010 PC-1: render_frame must render 'Permission Request' modal header \
         when AppMode::Overlay is active and overlay_stack is non-empty"
    );
}

/// AC-012 / BC-2.06.021 PC-1 (RED GATE): `render_frame` in Overlay mode must render the
/// Overlay hint line `"y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace"`
/// on the LAST terminal row.
///
/// The status bar hint line is always visible — even in Overlay mode — and must reflect
/// the current AppMode hint per BC-2.06.021 PC-1. The BLOCKER: the broken impl renders
/// the legacy Paragraph (drops the hint line entirely).
#[test]
fn test_BC_2_06_021_render_frame_overlay_mode_hint_line_appears_on_last_row() {
    let app = overlay_app();
    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_frame(&app, &mut sessions_state, frame);
        })
        .expect("render_frame must not panic");

    // Last row = row 23 (for a 24-row terminal).
    let last_row = row_text(&terminal, 23);
    let last_row_trimmed = last_row.trim_end();

    assert_eq!(
        last_row_trimmed,
        "y: accept  A: accept-always  n/r: reject  \u{2191}\u{2193}: cycle  Esc: hide  t: trace",
        "AC-012 / BC-2.06.021 PC-1: the last terminal row must contain the Overlay hint line \
         when AppMode::Overlay is active; the broken impl shows 'monocle' (MONOCLE_STATUS_LABEL) \
         instead"
    );
}

/// AC-012 (RED GATE): `render_frame` in Overlay mode must render the breadcrumb
/// `"Dashboard > Overlay [1 prompt]"` on the second-to-last row.
///
/// BC-2.06.020 PC-1: AppMode::Overlay with overlay_stack.len() == 1 →
/// breadcrumb `"Dashboard > Overlay [1 prompt]"`.
#[test]
fn test_BC_2_06_020_render_frame_overlay_mode_breadcrumb_appears_on_breadcrumb_row() {
    let app = overlay_app();
    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_frame(&app, &mut sessions_state, frame);
        })
        .expect("render_frame must not panic");

    // Second-to-last row = row 22 (for a 24-row terminal, breadcrumb row).
    let breadcrumb_row = row_text(&terminal, 22);

    assert!(
        breadcrumb_row.contains("Dashboard > Overlay"),
        "AC-012 / BC-2.06.020 PC-1: breadcrumb row (second-to-last) must contain \
         'Dashboard > Overlay' when AppMode::Overlay is active; \
         got: {:?}",
        breadcrumb_row.trim()
    );
    assert!(
        breadcrumb_row.contains("1 prompt"),
        "AC-012 / BC-2.06.020 PC-1: breadcrumb must show '[1 prompt]' (singular) for \
         overlay_stack.len() == 1; got: {:?}",
        breadcrumb_row.trim()
    );
}
