//! Layout builder for the monocle TUI.
//!
//! Dashboard layout (BC-2.06.005 PC-5):
//! - Sessions panel: left 60% of main area.
//! - Event ribbon: right 40% of main area.
//! - Status bar: always-visible 2-row strip at the bottom.
//!
//! Fullscreen layout (BC-2.06.007 PC-2):
//! - Active panel: 100% of main area.
//! - Status bar: always-visible 2-row strip at the bottom.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// The rendered layout split for the Dashboard mode.
///
/// Fields are the [`Rect`] regions passed to each widget's `render` call.
#[derive(Debug, Clone, Copy)]
pub struct DashboardLayout {
    /// The full terminal area minus the status bar.
    pub main_area: Rect,
    /// Left 60% of `main_area` — Sessions panel.
    pub sessions_area: Rect,
    /// Right 40% of `main_area` — Event ribbon panel.
    pub event_ribbon_area: Rect,
    /// Bottom 2 rows — status bar (breadcrumb + keybinding hints).
    pub status_bar_area: Rect,
}

/// The rendered layout split for Fullscreen mode.
#[derive(Debug, Clone, Copy)]
pub struct FullscreenLayout {
    /// 100% of the main area — the active fullscreen panel.
    pub panel_area: Rect,
    /// Bottom 2 rows — status bar.
    pub status_bar_area: Rect,
}

/// Build the `DashboardLayout` by splitting the terminal frame area.
///
/// Sessions panel occupies 60% and event ribbon 40% of the main area.
/// The bottom 2 rows are reserved for the status bar in all modes.
pub fn build_dashboard_layout(terminal_area: Rect) -> DashboardLayout {
    // Split terminal into main area (rows-2) and status bar (2 rows).
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(terminal_area);

    let main_area = vertical[0];
    let status_bar_area = vertical[1];

    // Split main area 60% sessions / 40% event ribbon.
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_area);

    DashboardLayout {
        main_area,
        sessions_area: horizontal[0],
        event_ribbon_area: horizontal[1],
        status_bar_area,
    }
}

/// Build the `FullscreenLayout` by splitting the terminal frame area.
///
/// The panel occupies 100% of the main area; the status bar occupies the
/// bottom 2 rows.
pub fn build_fullscreen_layout(terminal_area: Rect) -> FullscreenLayout {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(terminal_area);

    FullscreenLayout {
        panel_area: vertical[0],
        status_bar_area: vertical[1],
    }
}
