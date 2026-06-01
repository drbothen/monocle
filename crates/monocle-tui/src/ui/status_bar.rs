//! Status bar widget for monocle-tui (S-027).
//!
//! The status bar is always visible as the last row of the terminal. It is NOT
//! dimmed by the overlay background (BC-2.06.019 PC-1 / AC-008).
//!
//! # Layout (AC-008, BC-2.06.019)
//!
//! - Left: current `AppMode` indicator:
//!   - `"[DASHBOARD]"` — `AppMode::Dashboard { .. }`
//!   - `"[FILTERING]"` — `AppMode::Filtering { .. }`
//!   - `"[OVERLAY N]"` — `AppMode::Overlay { .. }` (N = stack depth)
//!   - `"[FULLSCREEN]"` — `AppMode::Fullscreen { .. }`
//! - Right: `"[dropped: N]"` in yellow when `drop_counter > 0`; nothing otherwise.
//!
//! # Relationship to existing status bar
//!
//! S-025 introduced a status bar in `render_frame` (in `app.rs`) that renders a
//! `MONOCLE_STATUS_LABEL` / drop-counter label. S-027 extends the status bar with a
//! mode indicator and replaces the `app.rs` inline render with a call to
//! `render_status_bar` from this module. The `render_frame` function in `app.rs` is
//! updated (by the S-027 implementer) to call `render_status_bar` instead of the
//! inline `Paragraph` render.

use monocle_core::tui::state::AppMode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

/// Render the always-visible status bar into `area`.
///
/// Called from `render_frame` in `app.rs` for ALL modes. The caller must ensure
/// `area` is the last `Constraint::Length(1)` row of the terminal layout
/// (or the `status_bar_area` from `build_dashboard_layout` / `build_fullscreen_layout`).
///
/// # Arguments
///
/// - `mode`: the current `AppMode` (determines the left-side indicator text).
/// - `drop_counter`: cumulative event drop count from `App::drop_counter`.
/// - `overlay_stack_depth`: `app.overlay_stack.len()` — used for `"[OVERLAY N]"` label.
/// - `status_message`: optional notification message (e.g., `"[disconnected] reconnecting..."`).
///   When `Some(msg)`, the message takes priority over the drop counter display
///   (same precedence logic as the existing `render_frame` implementation).
/// - `area`: the `Rect` to render into (single row).
/// - `buf`: the ratatui `Buffer` to write into.
pub fn render_status_bar(
    mode: &AppMode,
    drop_counter: u64,
    overlay_stack_depth: usize,
    status_message: Option<&str>,
    area: Rect,
    buf: &mut Buffer,
) {
    // Row 0: mode indicator + optional status/drop message on the same line (1-row area).
    // Row 1 (if area.height >= 2): breadcrumb + hint line.
    if area.height == 0 {
        return;
    }

    // Build the mode indicator text for the left side.
    let indicator = mode_indicator_text(mode, overlay_stack_depth);

    // Build the breadcrumb string.
    let breadcrumb = breadcrumb_text(mode, overlay_stack_depth);

    // Build the hint line for the current mode.
    let hint = hint_line_text(mode);

    if area.height == 1 {
        // Single-row: render mode indicator + drop counter / status message.
        let right_span: Option<Span<'static>> = if let Some(msg) = status_message {
            Some(Span::styled(
                msg.to_string(),
                Style::default().fg(Color::Yellow),
            ))
        } else {
            drop_counter_span(drop_counter)
        };

        let line = match right_span {
            Some(right) => Line::from(vec![
                Span::styled(indicator, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                right,
            ]),
            None => Line::from(Span::styled(indicator, Style::default().fg(Color::Cyan))),
        };

        Paragraph::new(line).render(area, buf);
    } else {
        // Multi-row: split into mode/breadcrumb row (row 0) and hint row (row 1+).
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        // Row 0: indicator + breadcrumb + optional drop/status.
        let right_span: Option<Span<'static>> = if let Some(msg) = status_message {
            Some(Span::styled(
                msg.to_string(),
                Style::default().fg(Color::Yellow),
            ))
        } else {
            drop_counter_span(drop_counter)
        };

        let top_spans: Vec<Span<'static>> = match right_span {
            Some(right) => vec![
                Span::styled(indicator, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(breadcrumb, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                right,
            ],
            None => vec![
                Span::styled(indicator, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(breadcrumb, Style::default().fg(Color::DarkGray)),
            ],
        };
        Paragraph::new(Line::from(top_spans)).render(rows[0], buf);

        // Row 1: hint line.
        Paragraph::new(Line::from(Span::styled(
            hint,
            Style::default().fg(Color::DarkGray),
        )))
        .render(rows[1], buf);
    }
}

/// Build the mode indicator string for the left side of the status bar.
///
/// Returns one of:
/// - `"[DASHBOARD]"` for `AppMode::Dashboard { .. }`
/// - `"[FILTERING]"` for `AppMode::Filtering { .. }`
/// - `"[OVERLAY N]"` for `AppMode::Overlay { .. }` where N = `overlay_stack_depth`
/// - `"[FULLSCREEN]"` for `AppMode::Fullscreen { .. }`
///
/// `overlay_stack_depth` is passed separately because `AppMode::Overlay` no longer
/// stores the stack (F-S025-ADV2-HIGH-003); the depth is read from `App::overlay_stack`.
pub fn mode_indicator_text(mode: &AppMode, overlay_stack_depth: usize) -> String {
    match mode {
        AppMode::Dashboard { .. } => "[DASHBOARD]".to_string(),
        AppMode::Filtering { .. } => "[FILTERING]".to_string(),
        AppMode::Overlay { .. } => format!("[OVERLAY {}]", overlay_stack_depth),
        AppMode::Fullscreen { .. } => "[FULLSCREEN]".to_string(),
    }
}

/// Build the breadcrumb string for the status bar (BC-2.06.020 PC-1/PC-2).
///
/// Returns a human-readable path describing the current location derived from
/// the current `AppMode` focus per BC-2.06.020 PC-1 derivation table:
/// - `Dashboard { focused: Sessions }` → `"Dashboard > Sessions"`
/// - `Dashboard { focused: EventRibbon }` → `"Dashboard > Events"`
/// - `Overlay { prior }` with `overlay_stack_depth == 1` → `"Dashboard > Overlay [1 prompt]"`
/// - `Overlay { prior }` with `overlay_stack_depth == N (N > 1)` → `"Dashboard > Overlay [N prompts]"`
/// - `Fullscreen { panel: Sessions, .. }` → `"Dashboard > Sessions > Fullscreen"`
/// - `Fullscreen { panel: EventRibbon, .. }` → `"Dashboard > Events > Fullscreen"`
/// - `Filtering { panel: Sessions, .. }` → `"Dashboard > Sessions > Filter"`
/// - `Filtering { panel: EventRibbon, .. }` → `"Dashboard > Events > Filter"`
pub fn breadcrumb_text(mode: &AppMode, overlay_stack_depth: usize) -> String {
    use monocle_core::tui::state::{FocusSnapshot, PanelId};
    match mode {
        AppMode::Dashboard { focused } => match focused {
            FocusSnapshot::Sessions => "Dashboard > Sessions".to_string(),
            FocusSnapshot::EventRibbon => "Dashboard > Events".to_string(),
            _ => "Dashboard".to_string(),
        },
        AppMode::Filtering { panel, .. } => match panel {
            PanelId::Sessions => "Dashboard > Sessions > Filter".to_string(),
            PanelId::EventRibbon => "Dashboard > Events > Filter".to_string(),
            _ => "Dashboard > Filter".to_string(),
        },
        AppMode::Overlay { .. } => {
            if overlay_stack_depth == 1 {
                "Dashboard > Overlay [1 prompt]".to_string()
            } else {
                format!("Dashboard > Overlay [{} prompts]", overlay_stack_depth)
            }
        }
        AppMode::Fullscreen { panel, .. } => match panel {
            PanelId::Sessions => "Dashboard > Sessions > Fullscreen".to_string(),
            PanelId::EventRibbon => "Dashboard > Events > Fullscreen".to_string(),
            _ => "Dashboard > Fullscreen".to_string(),
        },
    }
}

/// Build the hint line text for the current mode (BC-2.06.021 PC-1/PC-5/PC-6).
///
/// All hint strings MUST fit within 79 characters (BC-2.06.021 INV-3).
/// Canonical strings from BC-2.06.021 v1.0.6 PC-1 table.
pub fn hint_line_text(mode: &AppMode) -> String {
    match mode {
        AppMode::Dashboard { .. } => {
            // BC-2.06.021 PC-1 canonical: Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit
            "Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit".to_string()
        }
        AppMode::Filtering { .. } => {
            // BC-2.06.021 PC-1 canonical: (type to filter)  Esc: cancel
            "(type to filter)  Esc: cancel".to_string()
        }
        AppMode::Overlay { .. } => {
            // BC-2.06.021 PC-1 v1.0.6 canonical: y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace
            "y: accept  A: accept-always  n/r: reject  \u{2191}\u{2193}: cycle  Esc: hide  t: trace"
                .to_string()
        }
        AppMode::Fullscreen { .. } => {
            // BC-2.06.021 PC-1 canonical: Esc: back  /: filter  q: quit
            "Esc: back  /: filter  q: quit".to_string()
        }
    }
}

/// Build the drop counter indicator for the right side of the status bar.
///
/// Returns `None` when `drop_counter == 0` (no text shown — BC-2.06.019 PC-1).
/// Returns `Some("drops: N")` styled in yellow when `drop_counter > 0`
/// (BC-2.06.019 PC-2 canonical format: `drops: N`, no brackets, no "dropped").
///
/// The returned `Span` is used by `render_status_bar` to right-align the indicator.
pub fn drop_counter_span(drop_counter: u64) -> Option<Span<'static>> {
    if drop_counter == 0 {
        None
    } else {
        Some(Span::styled(
            format!("drops: {drop_counter}"),
            Style::default().fg(Color::Yellow),
        ))
    }
}
