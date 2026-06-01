//! ADV Pass-4 — I2: BC-2.06.006 PC-4 matched-character highlighting (S-028).
//!
//! # Finding (I2 MAJOR)
//!
//! `render_sessions_filter` currently renders matched sessions as plain `ListItem`s
//! (via `format_session_row` → a single plain string). No per-character highlight
//! is applied to matched character positions. Nucleo match indices are never computed.
//!
//! BC-2.06.006 PC-4 requires that matched characters are rendered with a distinct
//! `Span::styled` style (e.g. bold or a highlight color) so that the user can
//! visually identify which characters in the row matched the query.
//!
//! # Red Gate rationale
//!
//! The test renders a session whose `project_name` (or display field) contains
//! characters that match the query, then inspects the `TestBackend` buffer to
//! verify that at least one cell in the matched region has a non-default style
//! (bold or a distinct foreground color).
//!
//! Currently `render_sessions_filter` produces a plain `ListItem::new(row_string)`
//! — every cell has the default style.  The test assertion that matched cells carry
//! a distinct style will FAIL until the implementer wires nucleo match indices and
//! applies `Span::styled` on the matched character positions.
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
// Helpers
// ---------------------------------------------------------------------------

/// Build an `App` in `Filtering { Sessions }` mode with one session whose
/// `project_name` is known to fuzzy-match the query `"mono"`.
fn app_with_session_for_highlight(project_name: &str, query: &str) -> App {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![EnrichedSession::new(
        "sess-hl-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Active,
        None,
        Some(project_name.to_string()),
        None,
        0,
        None,
    )];
    app.mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: query.to_string(),
        prior: FocusSnapshot::Sessions,
    };
    app
}

// ---------------------------------------------------------------------------
// I2: BC-2.06.006 PC-4 — matched characters rendered with distinct Span style.
//
// RED: currently render_sessions_filter renders plain ListItem (no per-char highlight).
//      Every cell in the session row has the DEFAULT style (no bold, no distinct fg).
//      The assertion that at least one cell in the query-matched region has a non-default
//      style (bold Modifier or a distinct fg color) FAILS.
// ---------------------------------------------------------------------------

/// BC-2.06.006 PC-4 (I2): when filtering sessions, characters in the matched session
/// row that correspond to nucleo match positions must be rendered with a distinct
/// `Span::styled` style (e.g. bold, or a highlight foreground color).
///
/// This verifies the VISUAL requirement: the user sees which characters matched the
/// query — not just which sessions are included in the results.
///
/// RED: `render_sessions_filter` renders plain `ListItem::new(row_string)` — all cells
/// have the default style (no bold, no distinct fg). The assertion fails because no
/// cell in the matched region carries the expected distinct style.
///
/// FIX: implement per-character highlight by:
///   1. Calling `atom.indices(haystack, &mut app.matcher, &mut indices_buf)` to obtain
///      the matched character positions in the project_name / display_name string.
///   2. Building a `Line` from multiple `Span`s: non-matched chars → `Span::raw`,
///      matched chars → `Span::styled(char, Style::default().add_modifier(Modifier::BOLD))`.
///   3. Passing the `Line` to `ListItem::new(line)` instead of a plain string.
#[test]
fn test_BC_2_06_006_PC4_I2_matched_chars_rendered_with_distinct_style() {
    // Session: project_name = "monocle", query = "mono".
    // Nucleo fuzzy match of "mono" against "monocle" will produce match indices
    // covering the characters 'm', 'o', 'n', 'o' at positions 0, 1, 2, 3 in
    // "monocle". These characters must render with a distinct style (e.g. BOLD).
    let mut app = app_with_session_for_highlight("monocle", "mono");

    // Render in an 80×10 TestBackend.
    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = SessionsPanelState::default();

    terminal
        .draw(|frame| {
            // Area: full terminal minus the first row for the input box.
            // render_sessions_filter splits area into input row + list area.
            let area = Rect::new(0, 0, 80, 10);
            render_sessions_filter(&mut app, "mono", area, frame.buffer_mut(), &mut state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();

    // The input box occupies row 0 (render_sessions_filter layout: 1 row for input).
    // The first session row is at y=1 (list_area starts at y=1).
    // The session row contains "sess-hl-001 ● monocle Active ..." in column order.
    // "monocle" appears after the session_id and icon — find its x-offset.

    // First, verify the session IS rendered (the row contains the session content).
    let row_y = 1u16; // first item in the list area
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
        rendered_row.contains("monocle") || rendered_row.contains("sess-hl-001"),
        "precondition: session row must be rendered at y=1; got: {:?}",
        rendered_row
    );

    // Now find the x-offset of "monocle" in the rendered row (the project_name column).
    // The exact offset depends on session_id length + icon + space separators.
    // We search for the 'm' of "monocle" in the buffer at y=1.
    //
    // Session row format: "sess-hl-001 ● monocle Active 0 — —"
    // session_id = "sess-hl-001" (11 chars), then " ● " (3 chars = space + ● + space)
    // = offset 14. "monocle" starts at x=14.
    // However, the buffer includes the ratatui List padding (no extra padding for BORDERS::NONE).
    // We search the buffer dynamically for 'm' followed by "onocle".
    let monocle_start_x = (0..70u16).find(|&x| {
        // Check if position (x, 1) starts "monocle".
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

    let start_x = monocle_start_x.unwrap_or_else(|| {
        panic!(
            "BC-2.06.006 PC-4: could not find 'monocle' in rendered row at y=1. \
             Row content: {:?}. \
             FIX: render_sessions_filter must render matched sessions so project_name \
             appears in the list item.",
            rendered_row
        )
    });

    // Assert: at least one cell in the "monocle" substring (positions start_x..start_x+7)
    // must have a DISTINCT style — specifically Modifier::BOLD (or a non-default fg).
    // BC-2.06.006 PC-4 requires matched characters to be rendered with Span::styled.
    //
    // The query "mono" fuzzy-matches characters at positions 0..4 in "monocle".
    // After the fix, cells at start_x, start_x+1, start_x+2, start_x+3 (m, o, n, o)
    // must have a non-default style modifier.
    //
    // Currently: all cells have the DEFAULT style (no modifier). The assertion FAILS.
    let has_bold_cell = (start_x..start_x + 7).any(|x| {
        buffer
            .cell(Position::new(x, row_y))
            .map(|c| c.style().add_modifier.contains(Modifier::BOLD))
            .unwrap_or(false)
    });

    // Also check for a distinct foreground color (some implementations use color instead of bold).
    let has_distinct_fg = (start_x..start_x + 7).any(|x| {
        buffer
            .cell(Position::new(x, row_y))
            .map(|c| {
                // Any non-None/non-Reset foreground color counts as distinct.
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
        "BC-2.06.006 PC-4 RED (I2): matched characters in 'monocle' (query='mono') \
         must be rendered with a distinct Span style (BOLD modifier or a distinct \
         foreground color) at x={start_x}..{end_x} in the rendered buffer. \
         Found no bold cell or distinct fg in that range. \
         FIX: use nucleo atom.indices() to compute match positions in project_name, \
         then build a Line with Span::styled for matched chars and Span::raw for \
         non-matched chars. Pass the styled Line to ListItem::new(line).",
        end_x = start_x + 7
    );
}

// ---------------------------------------------------------------------------
// I2-b: with an empty query, rows render with DEFAULT style (no highlight).
//
// This is a regression guard: empty query must NOT apply any match highlighting.
// Stays GREEN both before and after the fix.
// ---------------------------------------------------------------------------

/// BC-2.06.006 PC-4 regression: with an EMPTY query, all session row cells must
/// have DEFAULT style (no BOLD, no distinct fg). Matched-char highlighting must
/// only fire when the query is non-empty and nucleo produces match indices.
///
/// This test stays GREEN both before and after the PC-4 fix.
#[test]
fn test_BC_2_06_006_PC4_I2_regression_empty_query_no_highlight() {
    // Session: project_name = "monocle", query = "" (empty).
    let mut app = app_with_session_for_highlight("monocle", "");

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = SessionsPanelState::default();

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 10);
            render_sessions_filter(&mut app, "", area, frame.buffer_mut(), &mut state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();

    // With empty query, all sessions are shown in default style (no highlights).
    // No cell in the session row (y=1) should have BOLD modifier from highlight logic.
    // (The blue selection highlight may apply to the selected row, but not BOLD for match.)
    let row_y = 1u16;
    let has_bold_from_highlight = (0..70u16).any(|x| {
        buffer
            .cell(Position::new(x, row_y))
            .map(|c| c.style().add_modifier.contains(Modifier::BOLD))
            .unwrap_or(false)
    });

    assert!(
        !has_bold_from_highlight,
        "BC-2.06.006 PC-4 regression: empty query must NOT apply BOLD highlight to session \
         row cells. Found BOLD at y={row_y} with empty query."
    );
}
