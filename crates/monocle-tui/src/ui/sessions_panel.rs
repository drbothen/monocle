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

use chrono::{DateTime, Utc};
use monocle_core::engine::{EnrichedSession, SessionStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::app::App;

// ---------------------------------------------------------------------------
// Empty-state canonical strings (BC-2.06.005 PC-3)
// ---------------------------------------------------------------------------

/// Line 1 of the sessions panel empty-state message (BC-2.06.005 PC-3).
///
/// Single source of truth shared by production render and integration tests.
/// Any change must be accompanied by a BC-2.06.005 version bump.
pub const SESSIONS_EMPTY_LINE_1: &str = "No sessions detected";

/// Line 2 of the sessions panel empty-state message (BC-2.06.005 PC-3).
///
/// Single source of truth shared by production render and integration tests.
/// Any change must be accompanied by a BC-2.06.005 version bump.
pub const SESSIONS_EMPTY_LINE_2: &str = "Start Claude Code in any terminal to see it here.";

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
/// **Rounding policy:** integer truncation (floor) for both the `k` and `M` ranges.
/// `1_050_000` → `"1M"` (m_dec == 0, decimal elided; not `"1.1M"`). `999_999` → `"999k"` (not `"1000k"`).
/// This matches the `k`-range where `1_999` → `"1k"` (not `"2k"`). Consistent
/// floor-truncation avoids surprising "up-rounding" on column widths and is easier
/// to reason about for debugging (F-S025-ADV2-LOW-002).
/// When the tenths digit is zero the `.0` suffix is elided: `1_000_000` → `"1M"`,
/// `1_050_000` → `"1M"`, `1_100_000` → `"1.1M"` (F-S025-ADV10-LOW-001).
///
/// Invariant 2 (BC-2.06.005): the formatter is deterministic — the same `u64`
/// always produces the same string.
pub fn format_token_count(count: u64) -> String {
    if count < 1_000 {
        count.to_string()
    } else if count < 1_000_000 {
        let k = count / 1_000;
        format!("{k}k")
    } else if count >= 1_000_000_000 {
        // F-S025-ADV9-LOW-002: cap at "999M+" to prevent unbounded column width.
        // EC-086 pattern extended to token counts: same cap discipline as uptime 999:59:59.
        "999M+".to_string()
    } else {
        // >= 1_000_000 and < 1_000_000_000: render as N.NM (one decimal place per BC-2.06.005)
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
/// Returns `"—"` when `cost_usd` is `Some(NaN)`, `Some(±inf)`, or `Some(negative)` —
/// all of these are invalid/corrupt producer data and must not produce `"$NaN"`,
/// `"$inf"`, `"$-inf"`, `"$-0.00"`, or `"$-N.NN"` in the TUI column. Cost is never
/// semantically negative; negative values indicate upstream corruption.
/// Returns `"$N.NN"` (two decimal places) for finite, non-negative values.
///
/// Invariant 3 (BC-2.06.005): `None` always renders as `"—"`, never as
/// `"None"`, `"N/A"`, `"-"`, or any other sentinel.
/// F-S025-ADV9-MED-001: NaN and ±infinity are treated as `"—"`.
/// F-S025-ADV10-LOW-002: negative values (including -0.0) are treated as `"—"`.
pub fn format_cost(cost_usd: Option<f64>) -> String {
    match cost_usd {
        None => "\u{2014}".to_string(), // U+2014 EM DASH
        // NaN, ±infinity, and any negative value (including -0.0) are invalid.
        // cost < 0.0 does not catch -0.0 in IEEE 754 (-0.0 == 0.0), so we also
        // check is_sign_negative() to catch the -0.0 edge case explicitly.
        Some(cost)
            if cost.is_nan() || cost.is_infinite() || cost < 0.0 || cost.is_sign_negative() =>
        {
            "\u{2014}".to_string()
        }
        Some(cost) => format!("${cost:.2}"),
    }
}

/// Format uptime given an explicit `now` instant — the canonical entry point.
///
/// Accepts `now: DateTime<Utc>` so callers can supply a deterministic value for
/// testing. Production callers (e.g., `format_session_row`) inject a per-frame
/// `Utc::now()` captured once at render time. Tests inject a fixed instant for
/// deterministic assertions.
///
/// Returns `"—"` (U+2014 EM DASH) when `started_at` is `None` (BC-2.06.005 Invariant 3).
/// Returns `"—"` when the elapsed duration is negative (clock skew guard).
///
/// Duration display follows BC-2.06.005 PC-2 canonical format `HH:MM:SS`:
/// - Normal: two-digit hours, two-digit minutes, two-digit seconds (e.g., `"03:47:00"`).
/// - ≥ 100 hours (EC-086): natural extension without zero-padding hours (e.g., `"100:00:00"`).
/// - ≥ 1000 hours: capped at `"999:59:59"` to avoid unbounded column width.
/// - Negative elapsed (clock skew): `"—"` sentinel.
///
/// # Test vectors (BC-2.06.005 line 127)
///
/// - `started_at = now - 3h47m` → `"03:47:00"`
/// - `started_at = now - 100h` → `"100:00:00"` (EC-086)
/// - `started_at = None` → `"—"`
pub fn format_uptime_at(
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    now: DateTime<Utc>,
) -> String {
    match started_at {
        None => "\u{2014}".to_string(), // U+2014 EM DASH
        Some(start) => {
            let elapsed = now.signed_duration_since(start);
            let total_secs = elapsed.num_seconds();
            if total_secs < 0 {
                // Clock skew: started_at is in the future — render sentinel.
                return "\u{2014}".to_string();
            }
            let total_secs = total_secs as u64;
            let hours = total_secs / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;
            // Cap at 999:59:59 to prevent unbounded column width.
            if hours >= 1000 {
                return "999:59:59".to_string();
            }
            if hours >= 100 {
                // EC-086: ≥ 100 hours — no zero-padding on hours column.
                format!("{hours}:{minutes:02}:{seconds:02}")
            } else {
                // Normal case: two-digit zero-padded hours.
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            }
        }
    }
}

#[cfg(test)]
mod uptime_tests {
    use super::format_uptime_at;
    use chrono::Utc;

    /// BC-2.06.005 PC-2 canonical test vector: 3h47m → "03:47:00".
    #[test]
    fn test_bc_2_06_005_uptime_canonical_3h47m() {
        let start = Utc::now() - chrono::Duration::seconds(3 * 3600 + 47 * 60);
        let result = format_uptime_at(Some(start), Utc::now());
        assert_eq!(result, "03:47:00", "3h47m must format as 03:47:00");
    }

    /// EC-086: ≥ 100 hours extension format.
    #[test]
    fn test_bc_2_06_005_uptime_ec086_100h() {
        let start = Utc::now() - chrono::Duration::seconds(100 * 3600);
        let result = format_uptime_at(Some(start), Utc::now());
        assert_eq!(
            result, "100:00:00",
            "100h must format as 100:00:00 (EC-086)"
        );
    }

    /// None → "—" sentinel.
    #[test]
    fn test_bc_2_06_005_uptime_none_renders_em_dash() {
        let result = format_uptime_at(None, Utc::now());
        assert_eq!(result, "—", "None must render as em dash");
    }

    /// 1h2m3s → "01:02:03".
    #[test]
    fn test_bc_2_06_005_uptime_1h2m3s() {
        let start = Utc::now() - chrono::Duration::seconds(3600 + 2 * 60 + 3);
        let result = format_uptime_at(Some(start), Utc::now());
        assert_eq!(result, "01:02:03");
    }

    /// 59 seconds → "00:00:59".
    #[test]
    fn test_bc_2_06_005_uptime_59s() {
        let start = Utc::now() - chrono::Duration::seconds(59);
        let result = format_uptime_at(Some(start), Utc::now());
        assert_eq!(result, "00:00:59");
    }

    /// 0 seconds → "00:00:00".
    #[test]
    fn test_bc_2_06_005_uptime_0s() {
        let start = Utc::now();
        let result = format_uptime_at(Some(start), Utc::now());
        // Allow "00:00:00" or "00:00:01" due to sub-second timing jitter.
        assert!(
            result == "00:00:00" || result == "00:00:01",
            "0s must format as 00:00:00 (or 00:00:01 for timing jitter), got {result}"
        );
    }
}

#[cfg(test)]
mod format_cost_tests {
    use super::format_cost;

    /// F-S025-ADV9-MED-001: NaN must render as em dash, not "$NaN".
    #[test]
    fn test_format_cost_nan_renders_em_dash() {
        assert_eq!(format_cost(Some(f64::NAN)), "\u{2014}");
    }

    /// F-S025-ADV9-MED-001: positive infinity must render as em dash, not "$inf".
    #[test]
    fn test_format_cost_positive_infinity_renders_em_dash() {
        assert_eq!(format_cost(Some(f64::INFINITY)), "\u{2014}");
    }

    /// F-S025-ADV9-MED-001: negative infinity must render as em dash, not "$-inf".
    #[test]
    fn test_format_cost_negative_infinity_renders_em_dash() {
        assert_eq!(format_cost(Some(f64::NEG_INFINITY)), "\u{2014}");
    }
}

#[cfg(test)]
mod format_token_count_cap_tests {
    use super::format_token_count;

    /// F-S025-ADV9-LOW-002: 1_000_000_000 (1B) must cap to "999M+".
    #[test]
    fn test_format_token_count_billion_caps_at_999m_plus() {
        assert_eq!(format_token_count(1_000_000_000), "999M+");
    }

    /// F-S025-ADV9-LOW-002: u64::MAX must cap to "999M+".
    #[test]
    fn test_format_token_count_u64_max_caps_at_999m_plus() {
        assert_eq!(format_token_count(u64::MAX), "999M+");
    }
}

#[cfg(test)]
mod format_session_row_tests {
    use super::format_session_row;
    use chrono::Utc;
    use monocle_core::engine::{EnrichedSession, SessionStatus};

    fn make_session_with_id(id: &str) -> EnrichedSession {
        EnrichedSession::new(
            id.to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Idle,
            None,
            Some("monocle".to_string()),
            None,
            0,
            None,
        )
    }

    fn fixed_now() -> chrono::DateTime<Utc> {
        // 2026-01-01T00:00:00Z — stable reference instant for deterministic tests.
        // SAFETY: 1_767_225_600 is a valid Unix timestamp (2026-01-01T00:00:00Z).
        // chrono::DateTime::from_timestamp is infallible for values in [0, i64::MAX]
        // that fit within the chrono representable range; this value does.
        #[allow(clippy::expect_used)]
        chrono::DateTime::from_timestamp(1_767_225_600, 0)
            .expect("1_767_225_600 is a valid Unix timestamp (2026-01-01T00:00:00Z)")
    }

    /// F-S025-ADV9-LOW-001: empty session_id must render as "?" prefix, not " ● ".
    #[test]
    fn test_format_session_row_empty_session_id_renders_question_mark() {
        let session = make_session_with_id("");
        let row = format_session_row(&session, fixed_now());
        assert!(
            row.starts_with("? ● "),
            "expected row to start with '? ● '; got: {row}"
        );
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
/// Uses `session.project_name` directly (BC-2.06.005 PC-2, HIGH-001 fix).
/// Returns `"—"` (U+2014) when `project_name` is `None`.
fn session_project(session: &EnrichedSession) -> String {
    session
        .project_name
        .as_deref()
        .map(str::to_owned)
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

/// Format a single session row as the canonical BC-2.06.005 v1.0.5 column string.
///
/// Accepts an explicit `now` timestamp so callers can supply a deterministic
/// value for testing (clock injection). Production callers pass `Utc::now()`.
///
/// Column order: `session_id icon project status tokens cost uptime`
/// Separator: single SPACE.
/// Canonical test vector: `"sess-001 ● monocle Active 437k — 03:47:00"`.
///
/// Extracted from the `StatefulWidget::render` loop body so that the exact
/// row format can be verified via unit test with deterministic time (F-S025-ADV6-MED-001).
pub fn format_session_row(s: &EnrichedSession, now: DateTime<Utc>) -> String {
    let icon = harness_icon(&s.harness_type);
    let project = session_project(s);
    let status = format_status(&s.status);
    let tokens = format_token_count(s.token_count);
    let cost = format_cost(s.cost_usd);
    let uptime = format_uptime_at(s.started_at, now);
    // F-S025-ADV9-LOW-001: daemon does not enforce non-empty session_id at the
    // EnrichedSession::new() level, so we guard here defensively. An empty id
    // would produce a leading-space artefact (" ● project …"); substitute "?".
    let id = if s.session_id.is_empty() {
        "?"
    } else {
        &s.session_id
    };
    format!(
        "{id} {icon} {} {} {} {} {}",
        project, status, tokens, cost, uptime
    )
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
            // Uses pub consts — single source of truth with integration tests.
            let empty = Paragraph::new(vec![
                Line::from(SESSIONS_EMPTY_LINE_1),
                Line::from(SESSIONS_EMPTY_LINE_2),
            ])
            .block(Block::default().borders(Borders::NONE));
            Widget::render(empty, list_area, buf);
        } else {
            // Capture `now` once per render call so all rows in a single frame
            // share a consistent timestamp (avoids sub-second drift across rows).
            let now = Utc::now();
            let items: Vec<ListItem> = self
                .app
                .sessions
                .iter()
                .map(|s| {
                    // format_session_row encapsulates BC-2.06.005 v1.0.5 canonical
                    // column order + separator. Pass `now` for deterministic testing.
                    let row = format_session_row(s, now);
                    ListItem::new(row)
                })
                .collect();

            let list = List::new(items)
                .highlight_style(Style::default().bg(Color::Blue))
                .block(Block::default().borders(Borders::NONE));

            StatefulWidget::render(list, list_area, buf, &mut state.list_state);
        }

        // Drop counter is rendered ONLY in the page-level status bar (app.rs render loop),
        // not here. AC-007 says "status bar" — the page-level bar is the single location.
        // F-S025-ADV2-MED-002: removed duplicate drop-counter from panel-internal status row.
        let _ = status_bar_area; // area reserved for future panel-level indicators (e.g., filter hint)
    }
}
