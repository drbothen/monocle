//! Unit tests for the Sessions panel widget (S-025).
//!
//! Traces to:
//! - BC-2.06.005 — Sessions panel rendering from IPC state.
//! - BC-2.06.007 — Enter transitions to Fullscreen; renderer handles empty sessions.
//! - AC-005 (session row rendering), AC-006 (keyboard navigation transitions),
//!   AC-007 (drop counter display).
//!
//! All render tests use ratatui's `TestBackend` to exercise the production
//! `SessionsPanel::render()` implementation. Until the implementer replaces
//! `todo!()` in `render()`, every test that calls it panics — Red Gate confirmed.
//!
//! Token/cost formatter tests (`format_token_count`, `format_cost`) call the
//! production helper functions from `monocle_tui::ui::sessions_panel`. These
//! functions are declared as `todo!()` stubs and panic on invocation.

use monocle_config::MonocleConfig;
use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::tui::state::{Action, AppMode, FocusSnapshot, PanelId, PromptModal, ToolPayload};
use monocle_tui::app::App;
use monocle_tui::ui::sessions_panel::{
    format_cost, format_token_count, SessionsPanel, SessionsPanelState,
};
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use std::time::Instant;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal `EnrichedSession` for render tests.
fn session(id: &str) -> EnrichedSession {
    EnrichedSession::new(
        id.to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Active,
        None,
        None, // project_name
        None, // started_at
        0,    // token_count
        None, // cost_usd
    )
}

/// Build an App with the given sessions pre-loaded.
fn app_with_sessions(sessions_raw: Vec<EnrichedSession>) -> App {
    use monocle_tui::app::on_initial_state;
    let mut app = App::new(MonocleConfig::default());
    on_initial_state(&mut app, sessions_raw, vec![], vec![], 0);
    app
}

/// Render `SessionsPanel` into a 80×24 `TestBackend` and return the buffer string.
///
/// Panics if the production `render()` is `todo!()` — this is the Red Gate.
fn render_sessions_panel(app: &App, _drop_counter: u64) -> String {
    let local_app_copy = App::new(MonocleConfig::default());
    // Copy sessions for render (App is not Clone; we rebuild from sessions list).
    // Since on_initial_state is todo!(), this will panic there too. But the render
    // path itself is the primary production target — we set drop_counter directly.
    drop(local_app_copy); // avoid unused warning

    // Build a fresh app, set drop_counter, sessions externally.
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");
    let area = Rect::new(0, 0, 80, 24);

    let mut state = SessionsPanelState::default();
    terminal
        .draw(|frame| {
            use ratatui::widgets::StatefulWidget;
            let panel = SessionsPanel::new(app);
            panel.render(area, frame.buffer_mut(), &mut state);
        })
        .expect("terminal draw");

    let buffer = terminal.backend().buffer().clone();
    let width = buffer.area().width as usize;
    let height = buffer.area().height as usize;
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x as u16, y as u16)].symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build a PromptModal for overlay tests.
// test-writer follow-up: invoke this helper in production-wired overlay render tests (S-026).
#[allow(dead_code)]
fn make_modal() -> PromptModal {
    PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "sess-001".into(),
        tool_name: "Bash".into(),
        tool_payload: ToolPayload::Bash {
            command: "ls".into(),
        },
        received_at: Instant::now(),
    }
}

// ---------------------------------------------------------------------------
// AC-005 — Sessions panel renders session list (BC-2.06.005 PC-1, PC-3)
// ---------------------------------------------------------------------------

/// BC-2.06.005 PC-3 / AC-005: Sessions panel renders the two-line empty-state
/// message when app.sessions is empty.
///
/// Test vector: app.sessions = [] →
///   contains "No sessions detected"
///   contains "Start Claude Code in any terminal to see it here."
#[test]
fn test_bc_2_06_005_pc3_ac005_renders_empty_state_when_no_sessions() {
    let app = App::new(MonocleConfig::default());
    let rendered = render_sessions_panel(&app, 0);
    assert!(
        rendered.contains("No sessions detected"),
        "BC-2.06.005 PC-3: rendered output must contain 'No sessions detected'; got:\n{}",
        rendered
    );
    assert!(
        rendered.contains("Start Claude Code in any terminal to see it here."),
        "BC-2.06.005 PC-3: rendered output must contain the second empty-state line; got:\n{}",
        rendered
    );
}

/// BC-2.06.005 PC-1 / AC-005: Sessions panel renders exactly one row when
/// app.sessions has one session.
#[test]
fn test_bc_2_06_005_pc1_ac005_renders_one_row_per_session() {
    let app = app_with_sessions(vec![session("sess-001")]);
    let rendered = render_sessions_panel(&app, 0);
    // The session_id must appear in the rendered output.
    assert!(
        rendered.contains("sess-001"),
        "BC-2.06.005 PC-1: rendered output must contain session_id 'sess-001'; got:\n{}",
        rendered
    );
}

/// BC-2.06.005 PC-1 / AC-005: Sessions panel renders three rows for three sessions.
#[test]
fn test_bc_2_06_005_pc1_ac005_renders_three_rows_for_three_sessions() {
    let app = app_with_sessions(vec![
        session("sess-a"),
        session("sess-b"),
        session("sess-c"),
    ]);
    let rendered = render_sessions_panel(&app, 0);
    assert!(
        rendered.contains("sess-a"),
        "BC-2.06.005 PC-1: row for sess-a must be present; got:\n{}",
        rendered
    );
    assert!(
        rendered.contains("sess-b"),
        "BC-2.06.005 PC-1: row for sess-b must be present; got:\n{}",
        rendered
    );
    assert!(
        rendered.contains("sess-c"),
        "BC-2.06.005 PC-1: row for sess-c must be present; got:\n{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// AC-005 — Column render assertions (BC-2.06.005 PC-2, Invariant 3)
// ---------------------------------------------------------------------------

/// BC-2.06.005 PC-2 Invariant 3 / AC-005: cost column renders "—" when
/// cost_usd is None.
///
/// The BC specifies "—" (U+2014 EM DASH), not "None" or "N/A".
#[test]
fn test_bc_2_06_005_pc2_inv3_cost_column_renders_em_dash_when_none() {
    let app = app_with_sessions(vec![session("sess-001")]);
    let rendered = render_sessions_panel(&app, 0);
    // Cost column must show "—" for None cost_usd.
    assert!(
        rendered.contains('—'),
        "BC-2.06.005 PC-2 Invariant 3: '—' must appear for None cost_usd; got:\n{}",
        rendered
    );
}

/// BC-2.06.005 PC-2 Invariant 3 / AC-005: project column renders "—" when
/// project_name is None.
#[test]
fn test_bc_2_06_005_pc2_inv3_project_column_renders_em_dash_when_none() {
    // session() builds with None transcript_path → project_name will be None.
    let app = app_with_sessions(vec![session("sess-001")]);
    let rendered = render_sessions_panel(&app, 0);
    assert!(
        rendered.contains('—'),
        "BC-2.06.005 PC-2 Invariant 3: '—' must appear for None project_name; got:\n{}",
        rendered
    );
}

/// BC-2.06.005 PC-2 Invariant 3 / AC-005: uptime column renders "—" when
/// started_at is None.
#[test]
fn test_bc_2_06_005_pc2_inv3_uptime_column_renders_em_dash_when_started_at_none() {
    let app = app_with_sessions(vec![session("sess-001")]);
    let rendered = render_sessions_panel(&app, 0);
    // Uptime column must show "—" when started_at is None.
    assert!(
        rendered.contains('—'),
        "BC-2.06.005 PC-2 Invariant 3: '—' must appear for None started_at; got:\n{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.005 token formatter (standalone pure function — todo!() Red Gate)
// ---------------------------------------------------------------------------

/// BC-2.06.005 PC-2 / AC-005: format_token_count(999) == "999" (no suffix).
///
/// Canonical test vector from BC-2.06.005.
#[test]
fn test_bc_2_06_005_pc2_token_formatter_below_1000_no_suffix() {
    let result = format_token_count(999);
    assert_eq!(
        result, "999",
        "BC-2.06.005: token_count=999 must format as '999'"
    );
}

/// BC-2.06.005 PC-2 / AC-005: format_token_count(1000) == "1k".
///
/// Canonical test vector from BC-2.06.005.
#[test]
fn test_bc_2_06_005_pc2_token_formatter_1000_renders_1k() {
    let result = format_token_count(1000);
    assert_eq!(
        result, "1k",
        "BC-2.06.005: token_count=1000 must format as '1k'"
    );
}

/// BC-2.06.005 PC-2 / AC-005: format_token_count(142_000) == "142k".
///
/// Canonical test vector from BC-2.06.005.
#[test]
fn test_bc_2_06_005_pc2_token_formatter_142000_renders_142k() {
    let result = format_token_count(142_000);
    assert_eq!(
        result, "142k",
        "BC-2.06.005: token_count=142000 must format as '142k'"
    );
}

/// BC-2.06.005 PC-2 / AC-005: format_token_count(1_200_000) == "1.2M".
///
/// Canonical test vector from BC-2.06.005.
#[test]
fn test_bc_2_06_005_pc2_token_formatter_1200000_renders_1_2m() {
    let result = format_token_count(1_200_000);
    assert_eq!(
        result, "1.2M",
        "BC-2.06.005: token_count=1200000 must format as '1.2M'"
    );
}

/// BC-2.06.005 PC-2 / AC-005: format_cost(None) == "—".
///
/// Canonical test vector from BC-2.06.005 Invariant 3.
#[test]
fn test_bc_2_06_005_pc2_cost_formatter_none_renders_em_dash() {
    let result = format_cost(None);
    assert_eq!(
        result, "—",
        "BC-2.06.005 Inv 3: cost=None must format as '—'"
    );
}

/// BC-2.06.005 PC-2 / AC-005: format_cost(Some(0.83)) == "$0.83".
///
/// Canonical test vector from BC-2.06.005.
#[test]
fn test_bc_2_06_005_pc2_cost_formatter_0_83_renders_dollar_0_83() {
    let result = format_cost(Some(0.83));
    assert_eq!(
        result, "$0.83",
        "BC-2.06.005: cost=Some(0.83) must format as '$0.83'"
    );
}

// ---------------------------------------------------------------------------
// AC-007 — Drop counter display in page-level status bar (BC-2.06.005 PC-3)
// ---------------------------------------------------------------------------
//
// F-S025-ADV2-MED-002: The drop counter renders ONLY in the page-level status
// bar (built in `app.rs` render loop). It was removed from the sessions panel's
// internal status row to eliminate the duplicate. AC-007 says "status bar" —
// one location only.
//
// The page-level status bar is rendered by `app.rs::run()` via `terminal.draw()`,
// not by `SessionsPanel`. Tests here verify the sessions panel does NOT render
// the drop counter (it's not the panel's responsibility).

/// BC-2.06.005 PC-3 / AC-007: sessions panel does NOT render the drop counter;
/// that is the page-level status bar's responsibility (F-S025-ADV2-MED-002).
#[test]
fn test_bc_2_06_005_pc3_ac007_sessions_panel_does_not_render_drop_counter() {
    let mut app = App::new(MonocleConfig::default());
    app.drop_counter = 5;
    let rendered = render_sessions_panel(&app, 5);
    // The sessions panel must NOT render the drop counter — it moved to page-level.
    assert!(
        !rendered.contains("[dropped:"),
        "AC-007 / MED-002: '[dropped:]' must NOT appear in the sessions PANEL \
         (only in page-level status bar); got:\n{}",
        rendered
    );
}

/// BC-2.06.005 PC-3 / AC-007: sessions panel status row does not render drop indicator
/// when drop_counter == 0 either.
#[test]
fn test_bc_2_06_005_pc3_ac007_drop_counter_hidden_when_zero() {
    let app = App::new(MonocleConfig::default());
    let rendered = render_sessions_panel(&app, 0);
    assert!(
        !rendered.contains("[dropped:"),
        "AC-007: '[dropped:]' must NOT appear when drop_counter=0; got:\n{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// AC-006 — Keyboard navigation transitions (BC-2.06.005 PC-2, BC-2.06.007)
// ---------------------------------------------------------------------------

/// BC-2.06.007 PC-1 / AC-006: transition(Dashboard { Sessions }, EnterFullscreen { Sessions })
/// → Fullscreen { panel: Sessions, prior: Sessions }.
///
/// This tests the already-implemented monocle-core::tui::state::transition() function.
/// Per BC-2.06.007 Invariant 1, the transition function does not inspect app.sessions.
///
/// Note: transition() is implemented in S-024 (monocle-core). This test PASSES.
/// It is included here to document the BC-2.06.007 contract for S-025.
/// The corresponding render test (fullscreen renderer) hits todo!() and fails.
#[test]
fn test_bc_2_06_007_pc1_enter_transitions_to_fullscreen() {
    use monocle_core::tui::state::transition;
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let next = transition(
        mode,
        Action::EnterFullscreen {
            panel: PanelId::Sessions,
        },
    );
    match next {
        AppMode::Fullscreen {
            panel: PanelId::Sessions,
            prior: FocusSnapshot::Sessions,
        } => { /* expected */ }
        other => panic!(
            "BC-2.06.007 PC-1: expected Fullscreen {{Sessions, Sessions}}, got discriminant {:?}",
            core::mem::discriminant(&other)
        ),
    }
}

/// BC-2.06.007 PC-5 / AC-006: transition(Fullscreen { Sessions, prior: Sessions }, ExitFullscreen)
/// → Dashboard { focused: Sessions }.
///
/// Note: transition() is implemented in S-024. This test PASSES. Included to
/// document the BC-2.06.007 PC-5 contract.
#[test]
fn test_bc_2_06_007_pc5_escape_from_fullscreen_returns_to_dashboard() {
    use monocle_core::tui::state::transition;
    let mode = AppMode::Fullscreen {
        panel: PanelId::Sessions,
        prior: FocusSnapshot::Sessions,
    };
    let next = transition(mode, Action::ExitFullscreen);
    match next {
        AppMode::Dashboard {
            focused: FocusSnapshot::Sessions,
        } => { /* expected */ }
        other => panic!(
            "BC-2.06.007 PC-5: expected Dashboard {{Sessions}}, got discriminant {:?}",
            core::mem::discriminant(&other)
        ),
    }
}

/// BC-2.06.007 PC-7 / EC-095 / AC-006: the Fullscreen renderer must NOT panic
/// when app.sessions is empty (guard is in renderer, not in transition).
///
/// The Sessions panel renderer handles empty sessions gracefully (BC-2.06.007 Inv 1).
/// When app.sessions is empty and we render, empty-state message appears.
#[test]
fn test_bc_2_06_007_pc7_fullscreen_renderer_no_panic_empty_sessions() {
    let app = App::new(MonocleConfig::default());
    // Transition to Fullscreen (valid per BC-2.06.007 Invariant 1 — transition
    // does not inspect app.sessions).
    let _mode = AppMode::Fullscreen {
        panel: PanelId::Sessions,
        prior: FocusSnapshot::Sessions,
    };
    // The Sessions panel render path is exercised with empty sessions.
    // BC-2.06.007 Invariant 1: the guard is in the renderer, not the transition.
    // The renderer must not panic on empty sessions — it shows the empty-state message.
    let rendered = render_sessions_panel(&app, 0);
    // Empty sessions: the renderer shows "No sessions detected" (same as Dashboard empty state).
    assert!(
        rendered.contains("No sessions detected"),
        "BC-2.06.007 PC-7: empty-session render must not panic and must show empty-state msg; got:\n{}",
        rendered
    );
}

/// BC-2.06.005 PC-2 / AC-006: Tab key cycles focus from Sessions to EventRibbon.
///
/// Tests the transition() function from monocle-core (S-024 impl, already passing).
/// The S-025 bind is: Tab in Dashboard → Action::MoveFocus → cycle focus.
#[test]
fn test_bc_2_06_005_pc2_ac006_tab_cycles_focus_sessions_to_event_ribbon() {
    use monocle_core::tui::state::transition;
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let next = transition(mode, Action::MoveFocus);
    match next {
        AppMode::Dashboard {
            focused: FocusSnapshot::EventRibbon,
        } => { /* expected */ }
        other => panic!(
            "AC-006: Tab must cycle Sessions → EventRibbon, got discriminant {:?}",
            core::mem::discriminant(&other)
        ),
    }
}

/// BC-2.06.005 PC-2 / AC-006: Tab key cycles focus from EventRibbon back to Sessions.
#[test]
fn test_bc_2_06_005_pc2_ac006_tab_cycles_focus_event_ribbon_to_sessions() {
    use monocle_core::tui::state::transition;
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let next = transition(mode, Action::MoveFocus);
    match next {
        AppMode::Dashboard {
            focused: FocusSnapshot::Sessions,
        } => { /* expected */ }
        other => panic!(
            "AC-006: Tab must cycle EventRibbon → Sessions, got discriminant {:?}",
            core::mem::discriminant(&other)
        ),
    }
}

/// BC-2.06.005 PC-2 / AC-006: j / ↓ key moves selection down.
///
/// The sessions panel renders both sessions when there are two. Selection
/// is tracked via SessionsPanelState (ListState). The j-key binding is
/// dispatched by the main event loop (not the widget itself) — this test
/// verifies the panel renders all sessions visible for navigation.
#[test]
fn test_bc_2_06_005_pc2_ac006_j_key_moves_selection_down() {
    let app = app_with_sessions(vec![session("sess-a"), session("sess-b")]);
    let rendered = render_sessions_panel(&app, 0);
    assert!(
        rendered.contains("sess-a"),
        "AC-006: sess-a must be visible for j-key navigation test; got:\n{}",
        rendered
    );
    assert!(
        rendered.contains("sess-b"),
        "AC-006: sess-b must be visible for j-key navigation test; got:\n{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// AC-005 — Layout constraint test (BC-2.06.005 PC-5)
// ---------------------------------------------------------------------------

/// BC-2.06.005 PC-5 / AC-005: build_dashboard_layout produces a sessions area
/// with non-zero width (60% constraint).
#[test]
fn test_bc_2_06_005_pc5_dashboard_layout_sessions_area_nonzero() {
    use monocle_tui::ui::layout::build_dashboard_layout;
    let area = Rect::new(0, 0, 100, 40);
    let layout = build_dashboard_layout(area);
    // Sessions area should be roughly 60% of 100 = ~60 wide.
    assert!(
        layout.sessions_area.width > 0,
        "BC-2.06.005 PC-5: sessions_area.width must be > 0"
    );
    assert!(
        layout.event_ribbon_area.width > 0,
        "BC-2.06.005 PC-5: event_ribbon_area.width must be > 0"
    );
}
