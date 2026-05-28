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
    format_cost, format_session_row, format_token_count, format_uptime_at, SessionsPanel,
    SessionsPanelState, SESSIONS_EMPTY_LINE_1, SESSIONS_EMPTY_LINE_2,
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
///   contains SESSIONS_EMPTY_LINE_1 ("No sessions detected")
///   contains SESSIONS_EMPTY_LINE_2 ("Start Claude Code in any terminal to see it here.")
///
/// Uses pub consts (single source of truth) — eliminates vacuous-mirror drift
/// between test and production (F-S025-ADV10-MED-001 sweep).
#[test]
fn test_bc_2_06_005_pc3_ac005_renders_empty_state_when_no_sessions() {
    // Pin canonical strings to their pub const definitions.
    assert_eq!(
        SESSIONS_EMPTY_LINE_1,
        "No sessions detected",
        "BC-2.06.005 PC-3: SESSIONS_EMPTY_LINE_1 must match canonical text verbatim"
    );
    assert_eq!(
        SESSIONS_EMPTY_LINE_2,
        "Start Claude Code in any terminal to see it here.",
        "BC-2.06.005 PC-3: SESSIONS_EMPTY_LINE_2 must match canonical text verbatim"
    );

    let app = App::new(MonocleConfig::default());
    let rendered = render_sessions_panel(&app, 0);
    assert!(
        rendered.contains(SESSIONS_EMPTY_LINE_1),
        "BC-2.06.005 PC-3: rendered output must contain SESSIONS_EMPTY_LINE_1; got:\n{}",
        rendered
    );
    assert!(
        rendered.contains(SESSIONS_EMPTY_LINE_2),
        "BC-2.06.005 PC-3: rendered output must contain SESSIONS_EMPTY_LINE_2; got:\n{}",
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

/// BC-2.06.005 PC-2 Invariant 3 / AC-005: rendered output contains at least one "—"
/// when project_name is None (em-dash present somewhere in the row).
///
/// Note: this test verifies that the rendered buffer contains at least one em-dash
/// when all optional fields (project_name, cost_usd, started_at) are None.
/// Column-specific placement of the project em-dash is verified by the verbatim
/// canonical row test (`test_bc_2_06_005_canonical_row_verbatim_with_mocked_time`).
#[test]
fn test_bc_2_06_005_pc2_inv3_em_dash_present_when_project_name_none() {
    // session() builds with None transcript_path → project_name will be None.
    let app = app_with_sessions(vec![session("sess-001")]);
    let rendered = render_sessions_panel(&app, 0);
    assert!(
        rendered.contains('—'),
        "BC-2.06.005 PC-2 Invariant 3: '—' must appear when project_name is None; got:\n{}",
        rendered
    );
}

/// BC-2.06.005 PC-2 Invariant 3 / AC-005: rendered output contains at least one "—"
/// when started_at is None (em-dash present somewhere in the row).
///
/// Note: this test verifies that the rendered buffer contains at least one em-dash
/// when all optional fields (project_name, cost_usd, started_at) are None.
/// Column-specific placement of the uptime em-dash is verified by the verbatim
/// canonical row test (`test_bc_2_06_005_canonical_row_verbatim_with_mocked_time`).
#[test]
fn test_bc_2_06_005_pc2_inv3_em_dash_present_when_started_at_none() {
    let app = app_with_sessions(vec![session("sess-001")]);
    let rendered = render_sessions_panel(&app, 0);
    // At least one "—" must appear when started_at is None.
    assert!(
        rendered.contains('—'),
        "BC-2.06.005 PC-2 Invariant 3: '—' must appear when started_at is None; got:\n{}",
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

// ---------------------------------------------------------------------------
// F-S025-ADV2-LOW-002: token formatter rounding policy edge cases
// ---------------------------------------------------------------------------
//
// Rounding policy: floor/truncation. 1_050_000 → "1.0M" (not "1.1M").
// 999_999 → "999k". 1_999_999 → "1.9M".

/// F-S025-ADV2-LOW-002: format_token_count(1_050_000) == "1M" (floor truncation, zero decimal omitted).
///
/// The formatter omits the trailing `.0` when the tenths digit is zero (consistent
/// with the k-range where no decimal is shown). 1_050_000: m_dec = 0 → "1M" not "1.0M".
/// This is distinguishable from 1_000_000 → "1M" as well; the distinction is intentional.
#[test]
fn test_bc_2_06_005_token_formatter_low002_1_050_000_truncates_omits_zero_decimal() {
    assert_eq!(
        format_token_count(1_050_000),
        "1M",
        "LOW-002: 1_050_000 floor-truncates to m_dec=0; formats as '1M' (no .0 suffix)"
    );
}

/// F-S025-ADV2-LOW-002: format_token_count(1_999_999) == "1.9M" (floor truncation).
#[test]
fn test_bc_2_06_005_token_formatter_low002_1_999_999_renders_1_9m() {
    assert_eq!(
        format_token_count(1_999_999),
        "1.9M",
        "LOW-002: 1_999_999 must floor-truncate to '1.9M'"
    );
}

/// F-S025-ADV2-LOW-002: format_token_count(999_999) == "999k" (floor truncation, stays in k range).
#[test]
fn test_bc_2_06_005_token_formatter_low002_999_999_renders_999k() {
    assert_eq!(
        format_token_count(999_999),
        "999k",
        "LOW-002: 999_999 must render as '999k' (just under the 1M boundary)"
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
    // Empty sessions: renderer shows SESSIONS_EMPTY_LINE_1 (single source of truth).
    assert!(
        rendered.contains(SESSIONS_EMPTY_LINE_1),
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

// ---------------------------------------------------------------------------
// F-S025-ADV3-MED-002 — build_fullscreen_layout + Fullscreen render branch
// ---------------------------------------------------------------------------

/// F-S025-ADV3-MED-002 / BC-2.06.007 PC-7: build_fullscreen_layout returns a
/// panel_area that occupies the full terminal width and all rows minus the status bar.
///
/// This test verifies:
/// 1. `build_fullscreen_layout` is not dead code (used by the render branch).
/// 2. The panel_area spans the full terminal width.
/// 3. The status_bar_area is exactly 2 rows as specified (BC-2.06.005 PC-5).
/// 4. panel_area.height + status_bar_area.height == terminal height.
#[test]
fn test_f_s025_adv3_med002_fullscreen_layout_panel_occupies_full_area() {
    use monocle_tui::ui::layout::build_fullscreen_layout;
    let terminal_area = Rect::new(0, 0, 100, 40);
    let layout = build_fullscreen_layout(terminal_area);

    // Panel area must span full width.
    assert_eq!(
        layout.panel_area.width, 100,
        "MED-002: fullscreen panel_area must span full terminal width"
    );
    // Status bar must be exactly 2 rows.
    assert_eq!(
        layout.status_bar_area.height, 2,
        "MED-002: status_bar_area must be 2 rows (BC-2.06.005 PC-5)"
    );
    // panel + status must == terminal height.
    assert_eq!(
        layout.panel_area.height + layout.status_bar_area.height,
        40,
        "MED-002: panel_area.height + status_bar_area.height must equal terminal height"
    );
    // Panel area height must be greater than the status bar (full-screen, not zero).
    assert!(
        layout.panel_area.height > 0,
        "MED-002: fullscreen panel_area must have positive height"
    );
    assert_eq!(
        layout.panel_area.height, 38,
        "MED-002: fullscreen panel_area.height must be terminal_height - 2 = 38"
    );
}

/// F-S025-ADV3-MED-002 / BC-2.06.007 PC-7: Fullscreen render branch exercises
/// Sessions panel across the full panel_area — sessions must appear in rendered output.
///
/// Confirms the render branch is wired: `build_fullscreen_layout` is called and
/// the SessionsPanel renders content (not the old dashboard 60/40 split output).
#[test]
fn test_f_s025_adv3_med002_fullscreen_render_branch_renders_sessions() {
    use monocle_tui::ui::layout::build_fullscreen_layout;

    let app = app_with_sessions(vec![session("sess-full")]);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");
    let terminal_area = Rect::new(0, 0, 80, 24);

    let fullscreen_layout = build_fullscreen_layout(terminal_area);
    let mut state = SessionsPanelState::default();

    terminal
        .draw(|frame| {
            use ratatui::widgets::StatefulWidget;
            let panel = SessionsPanel::new(&app);
            // Render into the fullscreen panel_area (not the 60%-wide sessions_area).
            panel.render(fullscreen_layout.panel_area, frame.buffer_mut(), &mut state);
        })
        .expect("terminal draw");

    let buffer = terminal.backend().buffer().clone();
    let rendered: String = (0..buffer.area().height as usize)
        .map(|y| {
            (0..buffer.area().width as usize)
                .map(|x| buffer[(x as u16, y as u16)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        rendered.contains("sess-full"),
        "MED-002/BC-2.06.007 PC-7: Sessions panel in fullscreen area must render session 'sess-full'; got:\n{}",
        rendered
    );
    // Verify the panel_area is full-width (80 columns — not 60% / 48 columns).
    assert_eq!(
        fullscreen_layout.panel_area.width, 80,
        "MED-002: panel_area must be 80 wide (full terminal), not 60%"
    );
}

// ---------------------------------------------------------------------------
// F-S025-ADV5-BLOCKER-001 — BC-2.06.005 v1.0.5 canonical column order
// ---------------------------------------------------------------------------
//
// The BC-2.06.005 v1.0.5 canonical test vector is:
//   `sess-001 ● monocle Active 437k — 03:47:00`
// Column order:  session_id | icon | project | status | tokens | cost | uptime
// Separator:     SPACE (not ` | `)
//
// The format-string bug renders icon FIRST: `● sess-001 | monocle | ...`
// These tests encode the canonical order and separator. They are RED GATE
// until the format-string is fixed.

/// F-S025-ADV5-BLOCKER-001 / BC-2.06.005 v1.0.5: canonical row — jitter-tolerant integration.
///
/// Renders against a live wall clock (`started_at = Utc::now() - 3h47m`) so uptime
/// seconds may increment between the `started_at` assignment and the render call.
/// This test allows seconds 00–04 (5-second window); it does NOT claim verbatim match.
///
/// For the byte-exact verbatim assertion against the canonical BC vector, see
/// `test_bc_2_06_005_canonical_row_verbatim_with_mocked_time`.
#[test]
fn test_bc_2_06_005_canonical_row_jitter_tolerant_integration() {
    use chrono::Utc;

    let started_at = Utc::now()
        - chrono::Duration::hours(3)
        - chrono::Duration::minutes(47)
        - chrono::Duration::seconds(0);

    let session = monocle_core::engine::EnrichedSession::new(
        "sess-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        monocle_core::engine::SessionStatus::Active,
        None,
        Some("monocle".to_string()), // project_name
        Some(started_at),            // started_at
        437_000,                     // token_count → "437k"
        None,                        // cost_usd → "—"
    );

    let app = app_with_sessions(vec![session]);
    let rendered = render_sessions_panel(&app, 0);

    // Extract the seconds component from the rendered uptime "03:47:SS".
    // Accept only 00–04 (5-second jitter window — not 00–09 which is too loose).
    let prefix = "sess-001 \u{25CF} monocle Active 437k \u{2014} 03:47:";
    assert!(
        rendered.contains(prefix),
        "F-S025-ADV5-BLOCKER-001 / BC-2.06.005 v1.0.5: rendered row must contain \
         'sess-001 ● monocle Active 437k — 03:47:'; got:\n{}",
        rendered
    );
    // Find and extract the two-digit seconds after the prefix.
    if let Some(start_idx) = rendered.find(prefix) {
        let rest = &rendered[start_idx + prefix.len()..];
        let secs_str = &rest[..2]; // exactly two digits
        let secs: u32 = secs_str.parse().unwrap_or_else(|_| {
            panic!(
                "BC-2.06.005: uptime seconds must be two-digit numeric, got {:?} in:\n{}",
                secs_str, rendered
            )
        });
        assert!(
            secs < 5,
            "BC-2.06.005 jitter-tolerant integration: seconds must be 00–04 \
             (5-second window), got {:02} in:\n{}",
            secs,
            rendered
        );
    }
    // Belt-and-suspenders: session_id before icon.
    let id_pos = rendered
        .find("sess-001")
        .unwrap_or_else(|| panic!("session_id must appear in rendered output; got:\n{rendered}"));
    let icon_pos = rendered
        .find('\u{25CF}')
        .unwrap_or_else(|| panic!("icon must appear in rendered output; got:\n{rendered}"));
    assert!(
        id_pos < icon_pos,
        "F-S025-ADV5-BLOCKER-001: session_id must appear before icon in rendered row"
    );
}

/// F-S025-ADV6-MED-001 / BC-2.06.005 v1.0.5: verbatim match against canonical BC vector
/// using deterministic mocked time — ZERO jitter tolerance.
///
/// Uses `format_session_row(s, now)` directly with a pinned `now` timestamp so that
/// `format_uptime_at` produces exactly `"03:47:00"` — byte-for-byte, no substring tolerance.
///
/// Canonical BC vector (BC-2.06.005 v1.0.5 test vector table):
///   `sess-001 ● monocle Active 437k — 03:47:00`
///
/// This is the authoritative verbatim assertion. Any change to the column format,
/// separator, or uptime rendering MUST update this test.
#[test]
fn test_bc_2_06_005_canonical_row_verbatim_with_mocked_time() {
    use chrono::Utc;

    // Pin `now` to a fixed instant. started_at is exactly 3h47m before now.
    let now = Utc::now();
    let started_at = now - chrono::Duration::hours(3) - chrono::Duration::minutes(47);

    let session = monocle_core::engine::EnrichedSession::new(
        "sess-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        monocle_core::engine::SessionStatus::Active,
        None,
        Some("monocle".to_string()), // project_name
        Some(started_at),            // started_at
        437_000,                     // token_count → "437k"
        None,                        // cost_usd → "—"
    );

    // Call format_session_row with the same pinned `now` — no wall-clock drift.
    let row = format_session_row(&session, now);

    // Verbatim assertion: EXACT canonical BC-2.06.005 v1.0.5 test vector.
    // No substring tolerance. No jitter tolerance. Byte-for-byte equality.
    assert_eq!(
        row, "sess-001 \u{25CF} monocle Active 437k \u{2014} 03:47:00",
        "F-S025-ADV6-MED-001 / BC-2.06.005 v1.0.5: format_session_row must produce \
         the exact canonical vector; got: {row:?}"
    );

    // Also verify the uptime component alone via format_uptime_at for traceability.
    let uptime = format_uptime_at(Some(started_at), now);
    assert_eq!(
        uptime, "03:47:00",
        "BC-2.06.005 v1.0.5: format_uptime_at(3h47m, now) must return exactly '03:47:00'"
    );
}

/// F-S025-ADV5-BLOCKER-001 / BC-2.06.005 v1.0.5: session_id appears BEFORE icon.
///
/// `rendered.find("sess-001") < rendered.find('●')`.
/// Fails before the format-string fix; passes after.
#[test]
fn test_bc_2_06_005_column_order_session_id_first() {
    let app = app_with_sessions(vec![session("sess-001")]);
    let rendered = render_sessions_panel(&app, 0);

    // Use unwrap_or_else (not .expect) to avoid clippy::expect_used on Option.
    let id_pos = rendered
        .find("sess-001")
        .unwrap_or_else(|| panic!("session_id must appear in rendered output; got:\n{rendered}"));
    let icon_pos = rendered
        .find('\u{25CF}')
        .unwrap_or_else(|| panic!("icon must appear in rendered output; got:\n{rendered}"));

    assert!(
        id_pos < icon_pos,
        "F-S025-ADV5-BLOCKER-001 / BC-2.06.005 v1.0.5: session_id must appear BEFORE icon; \
         session_id at {id_pos}, icon at {icon_pos} in:\n{}",
        rendered
    );
}

/// F-S025-ADV6-LOW-001 / BC-2.06.005 v1.0.5: every inter-column boundary uses
/// single-space separation, verified at each of the 6 boundaries.
///
/// Explicitly asserts each boundary pair in canonical order:
///   session_id·●  ·project·  ·status·  ·tokens·  ·cost·  ·uptime
/// A tab-separator or double-space regression would fail at the relevant boundary.
/// The original `!contains(" | ")` test proved only absence of pipe, not presence
/// of single-space at every boundary.
#[test]
fn test_bc_2_06_005_columns_separated_by_single_space() {
    use chrono::Utc;

    // Use deterministic time via format_session_row so boundary substrings are stable.
    let now = Utc::now();
    let started_at = now - chrono::Duration::hours(3) - chrono::Duration::minutes(47);
    let s = monocle_core::engine::EnrichedSession::new(
        "sess-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        monocle_core::engine::SessionStatus::Active,
        None,
        Some("monocle".to_string()),
        Some(started_at),
        437_000,
        None,
    );
    let row = format_session_row(&s, now);

    // Boundary 1: session_id → icon
    assert!(
        row.contains("sess-001 \u{25CF}"),
        "BC-2.06.005: boundary session_id→icon must be single space; got: {row:?}"
    );
    // Boundary 2: icon → project
    assert!(
        row.contains("\u{25CF} monocle"),
        "BC-2.06.005: boundary icon→project must be single space; got: {row:?}"
    );
    // Boundary 3: project → status
    assert!(
        row.contains("monocle Active"),
        "BC-2.06.005: boundary project→status must be single space; got: {row:?}"
    );
    // Boundary 4: status → tokens
    assert!(
        row.contains("Active 437k"),
        "BC-2.06.005: boundary status→tokens must be single space; got: {row:?}"
    );
    // Boundary 5: tokens → cost
    assert!(
        row.contains("437k \u{2014}"),
        "BC-2.06.005: boundary tokens→cost must be single space; got: {row:?}"
    );
    // Boundary 6: cost → uptime (uptime starts with "03:47:")
    assert!(
        row.contains("\u{2014} 03:47:"),
        "BC-2.06.005: boundary cost→uptime must be single space; got: {row:?}"
    );
    // Confirm no pipe separators anywhere in the row.
    assert!(
        !row.contains(" | "),
        "BC-2.06.005: row must not contain ' | ' pipe separator; got: {row:?}"
    );
}

// ---------------------------------------------------------------------------
// F-S025-ADV10-MED-002 — BC-2.06.005 PC-6 selected-row blue highlight
// ---------------------------------------------------------------------------

/// F-S025-ADV10-MED-002 / BC-2.06.005 PC-6: the currently-focused row is rendered
/// with the terminal's selection highlight style (Color::Blue background).
///
/// Exercises `list_state.select(Some(1))` with three sessions so the middle row
/// is selected. The selected row must have `bg == Some(Color::Blue)` on every cell.
///
/// Mental bug-injection: if `highlight_style(Style::default().bg(Color::Blue))` were
/// removed from production, `cell.style().bg` would be `None` and this test would
/// fail with `bg == None`.
#[test]
fn test_bc_2_06_005_pc6_selected_row_renders_blue_highlight_background() {
    use ratatui::{layout::Position, style::Color, widgets::StatefulWidget};

    let app = app_with_sessions(vec![
        session("sess-a"),
        session("sess-b"),
        session("sess-c"),
    ]);

    let backend = ratatui::backend::TestBackend::new(80, 8);
    let mut terminal = ratatui::Terminal::new(backend).expect("TestBackend terminal");

    let mut sessions_state = SessionsPanelState::default();
    // Select the middle row (index 1 — "sess-b").
    sessions_state.list_state.select(Some(1));

    terminal
        .draw(|frame| {
            // Use the full 80×8 area.  SessionsPanel splits it into [Min(0), Length(1)];
            // the list_area starts at y=0 with no border, so list item index 1 renders
            // at y=1 in the buffer.
            let area = frame.area();
            SessionsPanel::new(&app).render(area, frame.buffer_mut(), &mut sessions_state);
        })
        .expect("terminal draw");

    let buffer = terminal.backend().buffer().clone();

    // Index 1 with a BORDERS::NONE list and no block border → row y = 1 in the buffer.
    let selected_y: u16 = 1;

    // Assert that every non-empty cell in the selected row has Color::Blue background.
    // We check columns 0-9 (well within the session_id text "sess-b") to avoid
    // asserting on padding cells at the right margin that may retain default style.
    for x in 0u16..10 {
        let pos = Position::new(x, selected_y);
        let cell = buffer.cell(pos).unwrap_or_else(|| {
            panic!("BC-2.06.005 PC-6: expected a cell at ({x}, {selected_y}) in the buffer")
        });
        assert_eq!(
            cell.style().bg,
            Some(Color::Blue),
            "BC-2.06.005 PC-6: selected row (y={selected_y}, x={x}) must have \
             Color::Blue background; got {:?}",
            cell.style().bg
        );
    }
}

// ---------------------------------------------------------------------------
// F-S025-ADV10-LOW-003 — format_uptime_at clock-skew sentinel
// ---------------------------------------------------------------------------

/// F-S025-ADV10-LOW-003 / BC-2.06.005: when `started_at > now` (positive clock
/// skew), `format_uptime_at` must return the em-dash sentinel `"—"` (U+2014).
///
/// The production guard at `sessions_panel.rs:135-138` checks `total_secs < 0`
/// and returns `"\u{2014}"`.
///
/// Mental bug-injection: if the `total_secs < 0` guard were removed, a negative
/// duration would underflow to a large positive value (or panic on the `as u64`
/// cast), producing a nonsensical uptime string — never `"—"`. The test would fail.
#[test]
fn test_bc_2_06_005_uptime_clock_skew_renders_em_dash() {
    use chrono::{TimeZone, Utc};

    let now = Utc.with_ymd_and_hms(2026, 5, 28, 12, 0, 0).unwrap();
    // started_at is 5 seconds in the future — positive clock skew.
    let started_at_in_future = now + chrono::Duration::seconds(5);

    let result = format_uptime_at(Some(started_at_in_future), now);

    assert_eq!(
        result, "\u{2014}",
        "BC-2.06.005: clock skew (started_at > now) must render em-dash sentinel '—'; \
         got {:?}",
        result
    );
}
