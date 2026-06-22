//! Embedded PTY terminal widget for monocle-tui (S-039 + S-043, BC-2.09.001 + BC-2.09.007).
//!
//! Renders the PTY output captured by `vt100::Parser` into a ratatui `Frame` using
//! the `tui_term::widget::PseudoTerminal` widget. Supports per-session scrollback
//! navigation via the tui-term 0.3.4 scrollback API.
//!
//! # Module Purity
//!
//! This module is EFFECTFUL shell (per SS-embedded-pty.md §Module Purity table):
//! `ratatui::terminal.draw()` performs terminal I/O. The function must live in
//! `monocle-tui`, NOT in `monocle-core`.
//!
//! # Architecture constraint
//!
//! `monocle-tui` MUST NOT depend on `monocle-runtime` internals or `portable-pty`.
//! The TUI receives serialized PTY bytes via IPC and renders through `vt100::Parser`;
//! it never owns a PTY device.

use ratatui::layout::Rect;
use ratatui::Frame;
use tui_term::widget::PseudoTerminal;

/// Render the embedded PTY terminal widget for `parser` into `frame` at `area`.
///
/// Applies `scroll_offset` rows of scrollback to the parser screen before rendering,
/// using the canonical tui-term 0.3.4 scrollback call sequence:
///
/// 1. `parser.screen_mut().set_scrollback(scroll_offset)` — drives which rows the
///    screen reports; `0` is live tail, `N` is N rows back into scrollback history.
///    vt100 clamps the value automatically to the actual scrollback buffer size.
/// 2. `PseudoTerminal::new(parser.screen())` — hands the now-scrolled immutable
///    screen reference to the widget.
/// 3. `frame.render_widget(widget, area)` — renders the scrolled view.
/// 4. Read back `parser.screen().scrollback()` to obtain the vt100-clamped effective
///    offset for the `[scrolled back N rows]` status bar indicator (AC-007).
///
/// Called from `render_frame` when `AppMode::EmbeddedTerminal` is active and the
/// scroll offset is read from `App::pty_scroll_offsets[session_id]`.
///
/// # Arguments
///
/// * `frame` — ratatui `Frame` for the current draw call.
/// * `area` — the `Rect` inside which the PTY widget is rendered.
/// * `parser` — mutable reference to the `vt100::Parser` whose screen state is rendered.
///   Mutable because `set_scrollback` requires `&mut Screen`.
/// * `scroll_offset` — number of rows scrolled back from live tail; 0 means live tail.
///
/// # Returns
///
/// The effective (vt100-clamped) scroll offset after `set_scrollback`. Used by the
/// caller to display the `[scrolled back N rows]` indicator when the value is > 0.
/// Returns 0 when the view is at the live tail.
pub fn render_embedded_terminal(
    frame: &mut Frame<'_>,
    area: Rect,
    parser: &mut vt100::Parser,
    scroll_offset: usize,
) -> usize {
    // Step 1: drive the scrollback offset on the Screen.
    // set_scrollback(0) = live tail; set_scrollback(N) = N rows up into history.
    // vt100 clamps N to the actual scrollback buffer size automatically.
    parser.screen_mut().set_scrollback(scroll_offset);

    // Step 2: build the PseudoTerminal widget with the now-scrolled screen.
    // PseudoTerminal in tui-term 0.3.4 takes an immutable screen reference; the
    // scroll state has already been applied via set_scrollback above.
    let widget = PseudoTerminal::new(parser.screen());

    // Step 3: render into the provided area.
    frame.render_widget(widget, area);

    // Step 4: read back the vt100-clamped effective offset for the status bar.
    // screen().scrollback() returns 0 when at live tail, N when scrolled N rows up.
    parser.screen().scrollback()
}
