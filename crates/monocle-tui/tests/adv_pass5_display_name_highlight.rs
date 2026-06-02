//! ADV Pass-5 — F-2 MAJOR: BC-2.06.006 PC-4 highlight must cover display_name-only matches.
//!
//! # Finding (F-2 MAJOR)
//!
//! `render_sessions_filter` computes nucleo match indices ONLY against `project_name`
//! (the `atom.indices(haystack_project, …)` call). For a session that matches the query
//! via `display_name` OR-match but NOT via `project_name`, the indices call returns `None`
//! (no match against project_name) and the fallback plain-row path is taken.
//!
//! Result: no highlight is rendered for the display_name-matched characters even though
//! the session WAS included in results (it won via OR-match on display_name). This
//! violates BC-2.06.006 PC-4.
//!
//! # Test scenario
//!
//! Session: `project_name = "xyz-project"`, `display_name = "Claude Code"`.
//! Query: `"cla"`.
//!
//! - `"cla"` does NOT fuzzy-match `"xyz-project"` (no match → score_project = None).
//! - `"cla"` DOES fuzzy-match `"Claude Code"` (c-l-a in "Claude" → score_display = Some(N)).
//! - The session appears in results (OR-match via display_name).
//! - The rendered row should show "Claude Code" with 'c', 'l', 'a' highlighted (BOLD).
//! - CURRENTLY: `atom.indices` is called against `project_name = "xyz-project"` → None.
//!   No highlight is applied. All cells in the displayed text have DEFAULT style.
//!
//! # Fix required
//!
//! The implementer must compute indices against the WINNING field (the one that produced
//! the higher OR-match score) and apply highlights on the text that is DISPLAYED (which
//! is currently the display_name for the icon/harness column or project_name for the
//! project column).
//!
//! Concretely: if the session matched via display_name (score_display > score_project),
//! call `atom.indices(haystack_display, …)` and apply highlights on the displayed
//! display_name text rather than on the project_name text.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]
// expect/unwrap are idiomatic assertion amplification in test code.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_config::MonocleConfig;
use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::tui::state::{AppMode, FocusSnapshot, PanelId};
use monocle_tui::app::App;
use monocle_tui::ui::sessions_panel::{render_sessions_filter, SessionsPanelState};
use ratatui::{
    backend::TestBackend,
    layout::{Position, Rect},
    style::Modifier,
    Terminal,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Build an `App` in `Filtering { Sessions }` mode with one session that has
/// `project_name = "xyz-project"` and `display_name = "Claude Code"`.
///
/// The query `"cla"` fuzzy-matches `"Claude Code"` (display_name) but NOT
/// `"xyz-project"` (project_name). This is the canonical display_name-only-match
/// test vector for F-2.
fn app_with_display_name_only_match() -> App {
    let mut app = App::new(MonocleConfig::default());
    // Use new_with_display_name to supply a display_name distinct from project_name.
    // project_name: "xyz-project" (no "cla" match)
    // display_name: "Claude Code" (matches "cla")
    app.sessions = vec![EnrichedSession::new_with_display_name(
        "sess-dn-001".to_string(),
        "claude-code".to_string(),
        None, // transcript_path
        None, // config_path
        SessionStatus::Active,
        None,                            // last_event_micros
        Some("xyz-project".to_string()), // project_name — "cla" does NOT match
        None,                            // started_at
        0,                               // token_count
        None,                            // cost_usd
        "Claude Code".to_string(),       // display_name — "cla" DOES match
    )];
    app.mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "cla".to_string(),
        prior: FocusSnapshot::Sessions,
    };
    app
}

// ---------------------------------------------------------------------------
// F-2: BC-2.06.006 PC-4 — display_name-only match must render highlight
//
// RED: `render_sessions_filter` calls `atom.indices(haystack_project, …)` only.
//      For a session matched via display_name (project_name has no match), indices()
//      returns None → plain-row fallback → no highlight. All cells: DEFAULT style.
// ---------------------------------------------------------------------------

/// BC-2.06.006 PC-4 (F-2): when a session matches the query ONLY via `display_name`
/// (not via `project_name`), the matched characters in the DISPLAYED text must be
/// rendered with a distinct Span style (BOLD modifier or a distinct foreground color).
///
/// Test vector: `project_name = "xyz-project"`, `display_name = "Claude Code"`, query = `"cla"`.
/// `"cla"` does not match `"xyz-project"` but does match `"Claude Code"`.
///
/// RED: `render_sessions_filter` computes indices ONLY against `project_name`.
///      For this session, `atom.indices("xyz-project", …)` returns `None` → no highlight.
///      Every cell in the rendered row has DEFAULT style.
///
/// FIX: compute indices against the WINNING match field (display_name in this case)
///      and render the matched characters with `Span::styled(BOLD)`.
#[test]
fn test_BC_2_06_006_PC4_F2_display_name_only_match_renders_highlight() {
    let mut app = app_with_display_name_only_match();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = SessionsPanelState::default();

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 10);
            // Pass query "cla" — the same as in app.mode
            render_sessions_filter(&mut app, "cla", area, frame.buffer_mut(), &mut state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();

    // Input box at row 0. First session row at y=1.
    let row_y = 1u16;

    // Verify the session IS rendered (it must be in results due to display_name match).
    let rendered_row: String = (0..80u16)
        .map(|x| {
            buffer
                .cell(Position::new(x, row_y))
                .map(|c| c.symbol().to_string())
                .unwrap_or_default()
        })
        .collect::<String>()
        .trim_end()
        .to_string();

    assert!(
        !rendered_row.trim().is_empty()
            && (rendered_row.contains("sess-dn-001")
                || rendered_row.contains("Claude")
                || rendered_row.contains("xyz-project")),
        "precondition: session must appear in results for query 'cla' (matched via display_name \
         'Claude Code'). Rendered row at y=1: {:?}. \
         If the session is absent, the OR-match scoring logic itself is broken.",
        rendered_row
    );

    // The rendered row must contain the matched text. Depending on implementation:
    // - If display_name is shown in the icon/harness column, search for "Claude Code".
    // - If project_name is shown in the project column, the highlight must be on
    //   the project text, but since "xyz-project" doesn't match, the implementation
    //   must highlight the display_name text wherever it is rendered.
    //
    // Per BC-2.06.006 PC-4: "matched character positions are rendered with a highlight
    // style via ratatui::text::Span::styled". The test checks that AT LEAST ONE cell
    // in the entire rendered row (y=1) carries BOLD or a distinct foreground color.
    // This is intentionally broad: the highlight may appear on the display_name text
    // in any column position, and we do not prescribe which column.
    let has_bold_cell = (0..80u16).any(|x| {
        buffer
            .cell(Position::new(x, row_y))
            .map(|c| c.style().add_modifier.contains(Modifier::BOLD))
            .unwrap_or(false)
    });

    let has_distinct_fg = (0..80u16).any(|x| {
        buffer
            .cell(Position::new(x, row_y))
            .map(|c| {
                use ratatui::style::Color;
                matches!(
                    c.style().fg,
                    Some(Color::Yellow)
                        | Some(Color::Green)
                        | Some(Color::Cyan)
                        | Some(Color::Magenta)
                        | Some(Color::Blue)
                        | Some(Color::Red)
                        | Some(Color::White)
                        | Some(Color::Gray)
                        | Some(Color::LightRed)
                        | Some(Color::LightGreen)
                        | Some(Color::LightYellow)
                        | Some(Color::LightBlue)
                        | Some(Color::LightMagenta)
                        | Some(Color::LightCyan)
                        | Some(Color::Rgb(_, _, _))
                        | Some(Color::Indexed(_))
                )
            })
            .unwrap_or(false)
    });

    assert!(
        has_bold_cell || has_distinct_fg,
        "BC-2.06.006 PC-4 F-2 RED: session matched via display_name 'Claude Code' (query='cla') \
         must render at least one BOLD or distinctly-colored cell in the matched character region \
         at y=1. \
         CURRENT DEFECT: render_sessions_filter calls atom.indices() ONLY against project_name \
         ('xyz-project'). Since 'xyz-project' does not match 'cla', indices() returns None → \
         plain-row fallback → DEFAULT style on all cells. \
         FIX: when the OR-match winner is display_name, call atom.indices() against \
         display_name and render matched chars with Span::styled(BOLD). \
         Rendered row: {:?}",
        rendered_row
    );
}

// ---------------------------------------------------------------------------
// F-2 regression: project_name-matched sessions still highlight project_name (GREEN).
//
// If a session matches via project_name (regardless of display_name), the
// project_name characters must still be highlighted. This test stays GREEN.
// ---------------------------------------------------------------------------

/// BC-2.06.006 PC-4 regression: when a session matches via `project_name`,
/// the matched project_name characters must still be highlighted.
/// This stays GREEN before and after the F-2 fix.
#[test]
fn test_BC_2_06_006_PC4_F2_regression_project_name_match_still_highlights() {
    // Session: project_name = "monocle" (matches "mono"), display_name = "Claude Code" (no match).
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![EnrichedSession::new_with_display_name(
        "sess-pn-001".to_string(),
        "claude-code".to_string(),
        None, // transcript_path
        None, // config_path
        SessionStatus::Active,
        None,                        // last_event_micros
        Some("monocle".to_string()), // project_name — matches "mono"
        None,                        // started_at
        0,                           // token_count
        None,                        // cost_usd
        "Claude Code".to_string(),   // display_name — "mono" does NOT match this
    )];
    app.mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "mono".to_string(),
        prior: FocusSnapshot::Sessions,
    };

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = SessionsPanelState::default();

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 10);
            render_sessions_filter(&mut app, "mono", area, frame.buffer_mut(), &mut state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();
    let row_y = 1u16;

    // Find "monocle" in the buffer at y=1.
    let monocle_start_x = (0..70u16).find(|&x| {
        let chars: String = (x..x + 7)
            .map(|cx| {
                buffer
                    .cell(Position::new(cx, row_y))
                    .map(|c| c.symbol().to_string())
                    .unwrap_or_default()
            })
            .collect();
        chars == "monocle"
    });

    let start_x = monocle_start_x
        .expect("regression: 'monocle' must appear in rendered row at y=1 for project_name match");

    let has_bold = (start_x..start_x + 7).any(|x| {
        buffer
            .cell(Position::new(x, row_y))
            .map(|c| c.style().add_modifier.contains(Modifier::BOLD))
            .unwrap_or(false)
    });

    assert!(
        has_bold,
        "BC-2.06.006 PC-4 regression: matched project_name 'monocle' (query='mono') must \
         still render with BOLD highlight after the F-2 fix. \
         Found no BOLD cell in the 'monocle' region at x={start_x}..{end_x}, y=1.",
        end_x = start_x + 7
    );
}
