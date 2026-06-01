//! Status bar widget for monocle-tui (S-027).
//!
//! The status bar occupies the LAST TWO ROWS of the terminal and is always
//! visible. It is NOT dimmed by the overlay background (BC-2.06.019 PC-1 / AC-008).
//!
//! # Layout — TWO-ROW design (AC-008, BC-2.06.019/020/021)
//!
//! Row 0 (upper — breadcrumb row):
//! - Left: `AppMode` indicator (e.g. `"[DASHBOARD]"`, `"[OVERLAY N]"`).
//! - Middle: breadcrumb path derived from current focus (e.g.
//!   `"Dashboard > Sessions"`, `"Dashboard > Overlay [1 prompt]"`).
//! - Right (optional): `"drops: N"` in yellow when `drop_counter > 0`.
//!   `drops: N` is NEVER suppressed by a `status_message` (BC-2.06.019 PC-7).
//!
//! Row 1 (lower — hint row):
//! - When `status_message` is `Some(msg)`: the message is rendered in yellow,
//!   temporarily superseding the keybinding hint. When `status_message` is
//!   `None`, the canonical keybinding hint for the current mode is shown (e.g.
//!   `"Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit"`).
//!   Canonical per-mode strings are defined in BC-2.06.021 PC-1. Must fit within
//!   79 display columns (BC-2.06.021 INV-3).
//!
//! # Coexistence guarantee (BC-2.06.019 PC-7)
//!
//! The mutual-exclusion pattern `if status_message { msg } else { drop_counter }`
//! is FORBIDDEN. Both signals occupy different rows: `drops: N` is permanently on
//! the upper row when non-zero; `status_message` is always on the lower row.
//!
//! # Relationship to earlier status bar (S-025)
//!
//! S-025 introduced a 1-row `Paragraph` rendering `MONOCLE_STATUS_LABEL` /
//! `format_drop_counter` in `render_frame`. S-027 replaces that inline render
//! with a call to `render_status_bar` from this module, upgrading to the
//! two-row breadcrumb + hint layout and the canonical `drops: N` text format
//! (BC-2.06.019 PC-2).

use monocle_core::tui::state::AppMode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

/// Render the always-visible two-row status bar into `area`.
///
/// Called from `render_frame` in `app.rs` for ALL modes. The caller must ensure
/// `area` is the last `Constraint::Length(2)` rows of the terminal layout
/// (from `build_dashboard_layout` / `build_fullscreen_layout`).
///
/// When `area.height == 1` the function degrades gracefully: it renders only the
/// upper breadcrumb/indicator row (no hint line). When `area.height >= 2` it renders
/// the full two-row layout: upper breadcrumb row + lower hint row.
///
/// # Arguments
///
/// - `mode`: the current `AppMode` (determines indicator text and hint line).
/// - `drop_counter`: cumulative event drop count from `App::drop_counter`.
///   Shown as `"drops: N"` in yellow on the upper row when `> 0`
///   (BC-2.06.019 PC-2 canonical format — NOT `"[dropped: N]"`).
/// - `overlay_stack_depth`: `app.overlay_stack.len()` — used for `"[OVERLAY N]"` and
///   the breadcrumb `"[N prompts]"` count.
/// - `status_message`: optional transport notification (e.g., `"[disconnected] reconnecting..."`).
///   When `Some(msg)`, it is rendered in yellow on the LOWER (hint) row, temporarily
///   superseding the keybinding hint. It NEVER displaces `drops: N` from the upper row
///   (BC-2.06.019 PC-7 coexistence guarantee — the mutual-exclusion pattern is forbidden).
/// - `ccr_path`: the resolved CCR binary path from `App::ccr_path` (S-031, AC-007 /
///   BC-2.07.005 PC-3). Rendered on the lower row as `"CCR: <path>"` or `"CCR: none"`.
///   When `status_message` is `Some`, the status message takes precedence and the CCR
///   text is appended (space-separated) on the same lower row.
/// - `area`: the `Rect` to render into (two rows per AC-008).
/// - `buf`: the ratatui `Buffer` to write into.
pub fn render_status_bar(
    mode: &AppMode,
    drop_counter: u64,
    overlay_stack_depth: usize,
    status_message: Option<&str>,
    ccr_path: Option<&std::path::Path>,
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
        // Single-row degraded layout: render mode indicator + drops:N + CCR (BC-2.06.019 PC-7).
        // When area.height == 1, there is no room for the hint row.  The drop counter
        // takes priority over status_message in this degraded case because drops:N signals
        // an ongoing data-loss condition (BC-2.06.019 PC-7 rationale: permanent operational
        // indicator vs transient notification).  The status_message is silent here — this is
        // the 1-row degraded path only; the normal 2-row path is always preferred.
        let right_span: Option<Span<'static>> = drop_counter_span(drop_counter).or_else(|| {
            status_message
                .map(|msg| Span::styled(msg.to_string(), Style::default().fg(Color::Yellow)))
        });
        let ccr_text = ccr_path_text(ccr_path);

        let line = match right_span {
            Some(right) => Line::from(vec![
                Span::styled(indicator, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                right,
                Span::raw("  "),
                Span::styled(ccr_text, Style::default().fg(Color::DarkGray)),
            ]),
            None => Line::from(vec![
                Span::styled(indicator, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(ccr_text, Style::default().fg(Color::DarkGray)),
            ]),
        };

        Paragraph::new(line).render(area, buf);
    } else {
        // Multi-row: split into mode/breadcrumb row (row 0) and hint/status row (row 1+).
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        // Row 0 (upper — breadcrumb row): indicator + breadcrumb + drops:N + CCR path.
        // drops:N is ALWAYS shown here when drop_counter > 0 — never suppressed by
        // status_message (BC-2.06.019 PC-7 coexistence guarantee).
        // S-031 (AC-007 / BC-2.07.005 PC-3): CCR path is appended to the upper row,
        // separated from the breadcrumb/drops area. This keeps the lower (hint) row
        // stable (BC-2.06.021 PC-1 canonical strings unchanged).
        let drops_span: Option<Span<'static>> = drop_counter_span(drop_counter);
        let ccr_text = ccr_path_text(ccr_path);
        let ccr_span = Span::styled(ccr_text, Style::default().fg(Color::DarkGray));
        let top_spans: Vec<Span<'static>> = match drops_span {
            Some(drops) => vec![
                Span::styled(indicator, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(breadcrumb, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                drops,
                Span::raw("  "),
                ccr_span,
            ],
            None => vec![
                Span::styled(indicator, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(breadcrumb, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                ccr_span,
            ],
        };
        Paragraph::new(Line::from(top_spans)).render(rows[0], buf);

        // Row 1 (lower — hint/status row): status_message when Some, keybinding hint otherwise.
        // BC-2.06.019 PC-7: status_message (from disconnect, [t] stub, etc.) renders HERE,
        // on the lower row, temporarily superseding the keybinding hint. When None, the
        // canonical keybinding hint renders per BC-2.06.021.
        // NOTE: CCR path is on the UPPER row (row 0) so that the hint row strings remain
        // the canonical BC-2.06.021 PC-1 values (required by BC-2.06.021 INV-3 / exact tests).
        let lower_line: Line<'static> = if let Some(msg) = status_message {
            Line::from(Span::styled(
                msg.to_string(),
                Style::default().fg(Color::Yellow),
            ))
        } else {
            Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray)))
        };
        Paragraph::new(lower_line).render(rows[1], buf);
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

/// Build the CCR path footer string for the status bar (S-031, AC-007 / BC-2.07.005 PC-3).
///
/// Returns `"CCR: <path>"` when `ccr_path` is `Some`, `"CCR: none"` when `None`.
/// This string is appended to the lower (hint) row of the status bar when the
/// profile picker successfully resolves a CCR path after a profile switch or at startup.
///
/// The caller (`render_status_bar`) decides whether to render this alongside the
/// keybinding hint or separately — the exact layout integration is implemented in S-031.
pub fn ccr_path_text(ccr_path: Option<&std::path::Path>) -> String {
    match ccr_path {
        Some(path) => format!("CCR: {}", path.display()),
        None => "CCR: none".to_string(),
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
