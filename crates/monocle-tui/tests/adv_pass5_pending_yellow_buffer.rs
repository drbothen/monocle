//! ADV Pass-5 — F-4 NITPICK: BC-2.06.018 PC-4 PENDING-yellow style assertion at buffer level.
//!
//! # Finding (F-4 NITPICK)
//!
//! Existing tests for BC-2.06.018 PC-4 only check that `row.pending == true` at the data
//! layer. No test verifies at the ratatui buffer level that a `PENDING` row actually renders
//! the "PENDING" text with `Color::Yellow` foreground.
//!
//! This is a regression guard: the production code renders:
//!
//! ```rust
//! Span::styled("PENDING", Style::default().fg(Color::Yellow))
//! ```
//!
//! If a future refactor changes the color or drops the styled span, no test catches it.
//!
//! # Test approach
//!
//! Construct an `App` with one `HookEventRow` where `pending = true`. Render the
//! `EventRibbon` widget into a `TestBackend`. Scan the buffer for cells containing
//! the text "PENDING" and assert that each such cell has `Color::Yellow` foreground.
//!
//! # RED / GREEN status
//!
//! This test MAY pass immediately if the production render is correct. If it passes,
//! it is a regression guard (green gate). If the production render had a bug (wrong
//! color, plain text instead of styled), it fails — exposing the defect.
//!
//! The pass-5 finding explicitly notes: "it may PASS if production is correct; if so,
//! mark it a regression guard, not RED."
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]
// expect/unwrap are idiomatic assertion amplification in test code.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_config::MonocleConfig;
use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::HookType;
use monocle_tui::app::{on_hook_event_received, on_permission_prompt_queued, App};
use monocle_tui::ui::event_ribbon::{EventRibbon, EventRibbonState};
use ratatui::{
    backend::TestBackend,
    layout::{Position, Rect},
    style::Color,
    Terminal,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Build an `App` with one `HookEventRow` that has `pending = true`.
///
/// The pending flag is set via `on_permission_prompt_queued` which scans
/// `event_ribbon_events` for the most recent `PreToolUse` row matching the
/// session_id and sets `pending = true` (BC-2.06.018 PC-4 — production path).
fn app_with_one_pending_event() -> App {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![EnrichedSession::new(
        "sess-pend-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::WaitingOnPermission,
        None,
        Some("test-project".to_string()),
        None,
        0,
        None,
    )];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };

    // Push a PreToolUse event for sess-pend-001 — this is the row that will be
    // marked pending when a PermissionPromptQueued arrives.
    on_hook_event_received(
        &mut app,
        HookType::PreToolUse,
        "sess-pend-001".to_string(),
        "{}".to_string(),
        10u64,
        1_000_000i64, // non-zero timestamp to get a parseable wall-clock value
    );

    // Verify the PreToolUse row was pushed (precondition for pending flag setting).
    assert!(
        !app.event_ribbon_events.is_empty(),
        "precondition: at least one event row must exist before marking pending"
    );

    // Mark the row as pending via the production path (on_permission_prompt_queued).
    let payload = monocle_ipc::types::PermissionPromptPayload {
        prompt_id: Uuid::new_v4(),
        session_id: "sess-pend-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_input: {
            let mut m = serde_json::Map::new();
            m.insert(
                "command".to_string(),
                serde_json::Value::String("ls -la".to_string()),
            );
            serde_json::Value::Object(m)
        },
        old_content: None,
        new_content: None,
    };
    on_permission_prompt_queued(&mut app, payload);

    // Verify the pending flag was set by on_permission_prompt_queued.
    let has_pending = app.event_ribbon_events.iter().any(|r| r.pending);
    assert!(
        has_pending,
        "precondition: on_permission_prompt_queued must set pending=true on the PreToolUse \
         row for sess-pend-001 (BC-2.06.018 PC-4)"
    );

    app
}

// ---------------------------------------------------------------------------
// F-4: BC-2.06.018 PC-4 — PENDING row renders "PENDING" with Color::Yellow
//      at the ratatui buffer level (regression guard).
// ---------------------------------------------------------------------------

/// BC-2.06.018 PC-4 (F-4): a ribbon row with `pending = true` must render the
/// text "PENDING" with `Color::Yellow` foreground at the ratatui buffer cell level.
///
/// This is a buffer-level regression guard. The production code renders:
///   `Span::styled("PENDING", Style::default().fg(Color::Yellow))`
///
/// If this assertion currently passes, it confirms the production render is correct
/// and this test becomes a regression guard. If it fails, the production render
/// has a defect (wrong color, dropped styled span, etc.).
///
/// GREEN if production is correct (regression guard).
/// RED if the styled span is absent or uses a different color.
#[test]
fn test_BC_2_06_018_PC4_F4_pending_row_renders_yellow_in_buffer() {
    let app = app_with_one_pending_event();

    let backend = TestBackend::new(80, 5);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut ribbon_state = EventRibbonState::default();
    ribbon_state.list_state.select(Some(0));

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 5);
            let widget = EventRibbon::new(&app, Some("sess-pend-001"));
            ratatui::widgets::StatefulWidget::render(
                widget,
                area,
                frame.buffer_mut(),
                &mut ribbon_state,
            );
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();

    // The ribbon renders one row for sess-pend-001 (the PreToolUse event).
    // With `pending = true`, the row must contain "PENDING" somewhere.
    // The first rendered row is at y=0 (no header — EventRibbon renders items directly).
    let rendered_row: String = (0..80u16)
        .map(|x| {
            buffer
                .cell(Position::new(x, 0u16))
                .map(|c| c.symbol().to_string())
                .unwrap_or_default()
        })
        .collect::<String>()
        .trim_end()
        .to_string();

    assert!(
        rendered_row.contains("PENDING"),
        "precondition: PENDING row must render 'PENDING' text in row 0. \
         Got: {:?}. \
         If 'PENDING' is absent, the pending-flag render path (event_ribbon.rs PC-4) \
         is not reached.",
        rendered_row
    );

    // Find x-offset of 'P' in "PENDING" in the buffer at y=0.
    let pending_start_x = (0..74u16).find(|&x| {
        let chars: String = (x..x + 7)
            .map(|cx| {
                buffer
                    .cell(Position::new(cx, 0u16))
                    .map(|c| c.symbol().to_string())
                    .unwrap_or_default()
            })
            .collect();
        chars == "PENDING"
    });

    let start_x = pending_start_x.unwrap_or_else(|| {
        panic!(
            "BC-2.06.018 PC-4 F-4: could not locate 'PENDING' substring in buffer row 0 \
             even though the rendered text contains it. Buffer row: {:?}",
            rendered_row
        )
    });

    // Assert: every cell in the "PENDING" span (start_x..start_x+7) must have
    // Color::Yellow foreground (BC-2.06.018 PC-4: PENDING rendered with yellow style).
    for x in start_x..start_x + 7 {
        let cell_fg = buffer
            .cell(Position::new(x, 0u16))
            .map(|c| c.style().fg)
            .unwrap_or(None);

        assert_eq!(
            cell_fg,
            Some(Color::Yellow),
            "BC-2.06.018 PC-4 F-4 REGRESSION GUARD: 'PENDING' text at buffer position \
             (x={x}, y=0) must have Color::Yellow foreground. Got {:?}. \
             The production event_ribbon.rs render path uses \
             Span::styled(\"PENDING\", Style::default().fg(Color::Yellow)) — \
             if this assertion fails, that span was changed or removed.",
            cell_fg
        );
    }
}

// ---------------------------------------------------------------------------
// F-4 complement: a non-pending row must NOT render "PENDING" (regression guard).
// ---------------------------------------------------------------------------

/// BC-2.06.018 PC-4 regression: a ribbon row with `pending = false` must NOT
/// render "PENDING" text. This stays GREEN both before and after any fix.
#[test]
fn test_BC_2_06_018_PC4_F4_non_pending_row_does_not_render_pending_text() {
    use monocle_ipc::types::HookType;
    use monocle_tui::app::on_hook_event_received;

    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![EnrichedSession::new(
        "sess-nonpend".to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Active,
        None,
        Some("test-project".to_string()),
        None,
        0,
        None,
    )];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };

    // Push a non-pending Notification event.
    on_hook_event_received(
        &mut app,
        HookType::Notification,
        "sess-nonpend".to_string(),
        "{}".to_string(),
        5u64,
        1_000_000i64,
    );

    // Verify pending = false.
    assert!(
        app.event_ribbon_events.iter().all(|r| !r.pending),
        "precondition: non-pending row must have pending=false"
    );

    let backend = TestBackend::new(80, 5);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut ribbon_state = EventRibbonState::default();
    ribbon_state.list_state.select(Some(0));

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 5);
            let widget = EventRibbon::new(&app, Some("sess-nonpend"));
            ratatui::widgets::StatefulWidget::render(
                widget,
                area,
                frame.buffer_mut(),
                &mut ribbon_state,
            );
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();

    let rendered_row: String = (0..80u16)
        .map(|x| {
            buffer
                .cell(Position::new(x, 0u16))
                .map(|c| c.symbol().to_string())
                .unwrap_or_default()
        })
        .collect::<String>()
        .trim_end()
        .to_string();

    assert!(
        !rendered_row.contains("PENDING"),
        "BC-2.06.018 PC-4 regression: non-pending row must NOT contain 'PENDING' text. \
         Got: {:?}",
        rendered_row
    );
}
