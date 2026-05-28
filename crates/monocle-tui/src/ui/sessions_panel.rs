//! Sessions panel widget (BC-2.06.005, BC-2.06.007).
//!
//! `SessionsPanel` is a `ratatui::widgets::StatefulWidget` that renders one
//! row per `EnrichedSession` in `App::sessions`. State is tracked via
//! `ratatui::widgets::ListState` for selection highlighting.
//!
//! # Column layout (BC-2.06.005 PC-2)
//!
//! | Column  | Source field                         |
//! |---------|--------------------------------------|
//! | Icon    | `harness_type` → `char`              |
//! | Project | `project_name` (`None` → `"—"`)       |
//! | Status  | `SessionStatus` display string        |
//! | Tokens  | `token_count` human-readable          |
//! | Cost    | `cost_usd` (`None` → `"—"`)           |
//! | Uptime  | `now - started_at` (`None` → `"—"`)  |
//!
//! # Empty state (BC-2.06.005 PC-3)
//!
//! When `app.sessions` is empty the panel renders:
//! ```text
//! No sessions detected
//! Start Claude Code in any terminal to see it here.
//! ```
//!
//! # Drop counter (BC-2.06.005 PC-3 / AC-007)
//!
//! The status bar at the bottom of the panel shows `"[dropped: N]"` in yellow
//! when `app.drop_counter > 0`; nothing when it is zero.

use monocle_core::engine::{EnrichedSession, SessionStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::app::App;

// ---------------------------------------------------------------------------
// Token and cost formatters (BC-2.06.005 PC-2, Invariant 2 + 3)
// ---------------------------------------------------------------------------

/// Format a raw token count into a human-readable string (BC-2.06.005 PC-2).
///
/// | Range          | Format  | Example         |
/// |----------------|---------|-----------------|
/// | < 1,000        | raw     | `"999"`         |
/// | 1,000..999,999 | Nk      | `"142k"`, `"1k"` |
/// | >= 1,000,000   | N.NM    | `"1.2M"`        |
///
/// Invariant 2 (BC-2.06.005): the formatter is deterministic — the same `u64`
/// always produces the same string.
pub fn format_token_count(count: u64) -> String {
    if count < 1_000 {
        count.to_string()
    } else if count < 1_000_000 {
        let k = count / 1_000;
        format!("{k}k")
    } else {
        // >= 1_000_000: render as N.NM (one decimal place per BC-2.06.005)
        let m_int = count / 1_000_000;
        let m_dec = (count % 1_000_000) / 100_000; // tenths of millions
        if m_dec == 0 {
            format!("{m_int}M")
        } else {
            format!("{m_int}.{m_dec}M")
        }
    }
}

/// Format an optional cost as a USD string (BC-2.06.005 PC-2, Invariant 3).
///
/// Returns `"—"` (U+2014 EM DASH) when `cost_usd` is `None`.
/// Returns `"$N.NN"` (two decimal places) when `cost_usd` is `Some(f)`.
///
/// Invariant 3 (BC-2.06.005): `None` always renders as `"—"`, never as
/// `"None"`, `"N/A"`, `"-"`, or any other sentinel.
pub fn format_cost(cost_usd: Option<f64>) -> String {
    match cost_usd {
        None => "\u{2014}".to_string(), // U+2014 EM DASH
        Some(cost) => format!("${cost:.2}"),
    }
}

/// Render state for `SessionsPanel` — tracks the currently selected row index.
///
/// Wraps `ratatui::widgets::ListState` so that the implementer can call
/// `list_state.select(Some(idx))` to set the highlighted row.
#[derive(Debug, Default)]
pub struct SessionsPanelState {
    /// Underlying ratatui list selection state.
    pub list_state: ListState,
}

/// Sessions panel widget (BC-2.06.005).
///
/// Borrows `App` for rendering; does not mutate state.
/// Implements `ratatui::widgets::StatefulWidget` with `State = SessionsPanelState`.
pub struct SessionsPanel<'a> {
    /// Reference to application state for rendering session rows.
    pub app: &'a App,
}

impl<'a> SessionsPanel<'a> {
    /// Construct a `SessionsPanel` from a reference to the application state.
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

/// Map a harness type string to the display icon character.
///
/// `"claude-code"` → `'●'` (U+25CF BLACK CIRCLE) per BC-2.06.005 PC-2.
/// Future harness types should be added here without hardcoding Claude-specific
/// logic elsewhere (EC-088).
fn harness_icon(harness_type: &str) -> char {
    match harness_type {
        "claude-code" => '●', // U+25CF BLACK CIRCLE
        _ => '○',             // U+25CB WHITE CIRCLE — generic harness
    }
}

/// Derive the project name from an `EnrichedSession`.
///
/// Uses the directory component of `transcript_path` as the project name.
/// Returns `"—"` (U+2014) when `transcript_path` is `None` (BC-2.06.005 PC-2).
fn session_project(session: &EnrichedSession) -> String {
    session
        .transcript_path
        .as_ref()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "\u{2014}".to_string()) // U+2014 EM DASH
}

/// Format a `SessionStatus` for display in the status column.
fn format_status(status: &SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "Active",
        SessionStatus::Idle => "Idle",
        SessionStatus::WaitingOnPermission => "WaitingOnPermission",
        SessionStatus::Stopping => "Stopping",
        SessionStatus::Stopped => "Stopped",
        // #[non_exhaustive] guard — future variants get a sensible fallback.
        _ => "Unknown",
    }
}

impl StatefulWidget for SessionsPanel<'_> {
    type State = SessionsPanelState;

    /// Render the sessions panel into `buf` within `area`.
    ///
    /// Renders session rows or the empty-state message (BC-2.06.005 PC-3).
    /// Drop counter is shown in the bottom status line when > 0 (AC-007).
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Split area: main list area and status bar (1 row).
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let list_area = chunks[0];
        let status_bar_area = chunks[1];

        // --- Render the session list or empty-state message ---
        if self.app.sessions.is_empty() {
            // BC-2.06.005 PC-3: two-line empty state message.
            let empty = Paragraph::new(vec![
                Line::from("No sessions detected"),
                Line::from("Start Claude Code in any terminal to see it here."),
            ])
            .block(Block::default().borders(Borders::NONE));
            Widget::render(empty, list_area, buf);
        } else {
            let items: Vec<ListItem> = self
                .app
                .sessions
                .iter()
                .map(|s| {
                    let icon = harness_icon(&s.harness_type);
                    let project = session_project(s);
                    let status = format_status(&s.status);
                    // token_count / cost / uptime not yet on EnrichedSession Phase 1 —
                    // render "—" per BC-2.06.005 Invariant 3 until fields are added.
                    let tokens = "\u{2014}";
                    let cost = "\u{2014}";
                    let uptime = "\u{2014}";
                    let row = format!(
                        "{icon} {} | {} | {} | {} | {} | {}",
                        s.session_id, project, status, tokens, cost, uptime
                    );
                    ListItem::new(row)
                })
                .collect();

            let list = List::new(items)
                .highlight_style(Style::default().bg(Color::Blue))
                .block(Block::default().borders(Borders::NONE));

            StatefulWidget::render(list, list_area, buf, &mut state.list_state);
        }

        // --- Render drop counter status bar (AC-007, BC-2.06.005 PC-3) ---
        let status_text = if self.app.drop_counter > 0 {
            Line::from(vec![Span::styled(
                format!("[dropped: {}]", self.app.drop_counter),
                Style::default().fg(Color::Yellow),
            )])
        } else {
            Line::from("")
        };

        Widget::render(Paragraph::new(status_text), status_bar_area, buf);
    }
}
