//! Embedded PTY terminal widget for monocle-tui (S-039, BC-2.09.001).
//!
//! Renders the PTY output captured by `vt100::Parser` into a ratatui `Frame` using
//! the `tui_term::widget::PseudoTerminal` widget.
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
// PseudoTerminal is used by the implementation of render_embedded_terminal.
// The import is retained here as a forward reference for the implementer.
#[allow(unused_imports)]
use tui_term::widget::PseudoTerminal;

/// Render the embedded PTY terminal widget for `parser` into `frame` at `area`.
///
/// Applies `scroll_offset` rows of scrollback to the parser screen before rendering.
/// The call sequence per the tui-term scrollback API is:
///
/// 1. `parser.screen_mut().set_scrollback(scroll_offset)` — drives which rows the
///    screen reports; `0` is live tail, `N` is N rows back into scrollback history.
///    vt100 clamps the value automatically to the actual scrollback buffer size.
/// 2. `PseudoTerminal::new(parser.screen())` — hands the now-scrolled immutable
///    screen reference to the widget.
/// 3. `frame.render_widget(&widget, area)` — renders the scrolled view.
///
/// Called from `render_frame` when `AppMode::EmbeddedTerminal` is active and the
/// scroll offset is read from `App::pty_scroll_offsets[session_id]`
/// (BC-2.09.007 Postconditions 2a/2b, AC-007).
///
/// The `effective_offset` return value reflects the actual clamped offset applied
/// by vt100 (`parser.screen().scrollback()`). The status bar uses this value to
/// display the `[scrolled back N rows]` indicator when offset > 0 (AC-007).
/// When the offset is 0, the indicator is absent.
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
/// caller to render the scrolled-back status bar indicator.
#[allow(clippy::todo)]
pub fn render_embedded_terminal(
    _frame: &mut Frame<'_>,
    _area: Rect,
    _parser: &mut vt100::Parser,
    _scroll_offset: usize,
) -> usize {
    todo!()
}
