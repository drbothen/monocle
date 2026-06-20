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
use tui_term::widget::PseudoTerminal;

/// Render the embedded PTY terminal widget for `parser` into `frame` at `area`.
///
/// Creates a `tui_term::widget::PseudoTerminal::new(parser.screen())` and renders it
/// into the provided `Rect`. Called from `render_frame` when `AppMode::EmbeddedTerminal`
/// is active (BC-2.09.001 AC-003 / postcondition 3).
///
/// # Arguments
///
/// * `frame` — ratatui `Frame` for the current draw call.
/// * `area` — the `Rect` inside which the PTY widget is rendered.
/// * `parser` — the `vt100::Parser` whose screen state is rendered.
pub fn render_embedded_terminal(frame: &mut Frame<'_>, area: Rect, parser: &vt100::Parser) {
    // BC-2.09.001 AC-003 / Postcondition 3: create PseudoTerminal widget from
    // the parser's current screen and render it into the provided area.
    let widget = PseudoTerminal::new(parser.screen());
    frame.render_widget(&widget, area);
}
