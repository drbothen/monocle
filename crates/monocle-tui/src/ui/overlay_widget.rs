//! Permission overlay modal widget for monocle-tui (S-027).
//!
//! Renders the active `PromptModal` from `App::overlay_stack` as a centered
//! modal over a dimmed dashboard background.
//!
//! # Layout (AC-001, BC-2.06.010 PC-1)
//!
//! - Header: `"Permission Request"` with `session_id`, `tool_name`, stack depth
//!   indicator `"(1 of N)"`, and elapsed timer `"Waiting: <Ns>"`.
//! - Body: tool-specific content rendered by one of the `render_*_payload` helpers.
//! - Footer: keyboard hint line `"[y] Accept  [A] Accept Always  [n/r] Reject  [Esc] No-op"`.
//! - Modal width: `min(terminal_width - 4, 100)`.
//!
//! # Dim background (AC-002, BC-2.06.010 PC-2)
//!
//! All cells outside the modal are rendered with `Modifier::DIM`. The status bar
//! is NOT dimmed (BC-2.06.019 PC-1 / AC-008).
//!
//! # Purity boundary (AC-007, BC-2.06.015 INV-1)
//!
//! `similar` is imported here (in `monocle-tui`) ONLY. It MUST NOT be added to
//! `monocle-core`. Any refactor that moves diff logic to `monocle-core` is a
//! purity boundary violation.

use monocle_core::tui::state::{PromptModal, ToolPayload};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};
use similar::{ChangeTag, TextDiff};

/// Render the permission overlay modal centered over the given `area`.
///
/// Called from the render loop when `AppMode::Overlay { .. }` is active.
/// `modal` is the front entry of `App::overlay_stack` (the oldest pending prompt).
/// `stack_depth` is `app.overlay_stack.len()` (used for the `"(1 of N)"` indicator).
///
/// # Dimming
///
/// The caller is responsible for dimming the background area BEFORE calling this
/// function. Use [`render_dimmed_background`] to apply `Modifier::DIM` to the
/// non-status-bar cells.
///
/// # Side effects
///
/// None. This function is a pure renderer: it reads `modal` fields and writes
/// ratatui `Buffer` cells. It does NOT mutate `AppMode` or `overlay_stack`.
pub fn render_overlay_widget(
    modal: &PromptModal,
    stack_depth: usize,
    area: Rect,
    frame: &mut Frame,
) {
    let modal_area = modal_rect(area);

    // Clear the modal region first (so it is not dimmed behind the modal).
    Clear.render(modal_area, frame.buffer_mut());

    // Compute elapsed timer.
    let elapsed_secs = std::time::Instant::now()
        .duration_since(modal.received_at)
        .as_secs();

    // Build header and footer lines.
    let header_line = build_header_line(
        &modal.session_id,
        &modal.tool_name,
        stack_depth,
        elapsed_secs,
    );
    // BC-2.06.024 PC-3 / AC-006: scroll hint location is ALWAYS the body (inside
    // render_generic_payload) and NEVER the footer. The footer carries exactly the
    // decision-key hints; adding a second scroll-hint there would produce a double-hint
    // when overflow is true and a spurious hint when overflow is false.
    let footer_line = build_footer_line();

    // Outer block for the modal (has border, no title — header is rendered as first body line).
    let outer_block = Block::default().borders(Borders::ALL);

    // Compute inner area for content (inside the border).
    let inner_area = outer_block.inner(modal_area);

    // Render the outer block/border (no title — header line is body row 0).
    outer_block.render(modal_area, frame.buffer_mut());

    // Split inner area: 1 header row, body rows, 1 footer row.
    if inner_area.height == 0 {
        return;
    }

    // Header: 2 rows to accommodate the full header text (session + tool + timer)
    // even in narrow terminals (AC-009 / BC-2.06.020 PC-1).
    let header_height = 2u16;
    let footer_height = 1u16;
    let body_height = inner_area
        .height
        .saturating_sub(header_height + footer_height);

    let header_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: header_height,
    };
    let body_area = Rect {
        x: inner_area.x,
        y: inner_area.y + header_height,
        width: inner_area.width,
        height: body_height,
    };
    let footer_area = Rect {
        x: inner_area.x,
        y: inner_area.y + header_height + body_height,
        width: inner_area.width,
        height: footer_height,
    };

    let buf = frame.buffer_mut();

    // Render the header line (row 0 of inner area) with wrapping so long
    // headers (including "Waiting:" timer) are visible even in narrow terminals.
    if inner_area.height >= 1 {
        Paragraph::new(header_line)
            .wrap(Wrap { trim: false })
            .render(header_area, buf);
    }

    // Render payload-specific body content.
    match &modal.tool_payload {
        ToolPayload::Bash { command } => {
            render_bash_payload(command, body_area, buf);
        }
        ToolPayload::Read { path } => {
            render_read_payload(path, body_area, buf);
        }
        ToolPayload::Edit {
            old_content,
            new_content,
            path,
        } => {
            render_edit_payload(old_content, new_content, path, body_area, buf);
        }
        ToolPayload::Generic {
            tool_name,
            tool_input,
        } => {
            render_generic_payload(tool_name, tool_input, body_area, buf);
        }
        _ => {
            // Forward-compatibility: unknown ToolPayload variants render a placeholder.
            let p = Paragraph::new(Line::from(Span::styled(
                "(unknown tool payload)",
                Style::default().fg(Color::DarkGray),
            )));
            p.render(body_area, buf);
        }
    }

    // Render footer.
    Paragraph::new(footer_line).render(footer_area, buf);
}

/// Apply `Modifier::DIM` to all cells in `area` to visually de-emphasize the
/// content behind the overlay modal (AC-002, BC-2.06.010 PC-2).
///
/// The status bar row is NOT included in `area` — the caller must exclude it
/// before calling this function so the status bar remains full-brightness
/// (BC-2.06.019 PC-1).
pub fn render_dimmed_background(area: Rect, buf: &mut Buffer) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let current_style = buf[(x, y)].style();
            let new_style = current_style.add_modifier(Modifier::DIM);
            buf[(x, y)].set_style(new_style);
        }
    }
}

// ---------------------------------------------------------------------------
// Payload-specific body renderers (AC-003 through AC-006, BC-2.06.024)
// ---------------------------------------------------------------------------

/// Render a `ToolPayload::Bash { command }` body inside `area`.
///
/// Renders a bordered `Block` titled `"Command"` containing the label line
/// `command: <command>` per BC-2.06.024 PC-1. If `command` is empty, renders
/// `command: (empty)` as a safe fallback (BC-2.06.024 PC-1.4). Long command
/// values wrap using `Wrap { trim: false }` so the full command is visible
/// without silent truncation (BC-2.06.024 PC-1.5 / INV-4).
///
/// # Parameters
///
/// - `command`: the shell command string from `ToolPayload::Bash`.
/// - `area`: the body area inside the modal (header and footer already excluded).
/// - `buf`: the ratatui `Buffer` to render into.
pub fn render_bash_payload(command: &str, area: Rect, buf: &mut Buffer) {
    let block = Block::default().borders(Borders::ALL).title("Command");
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // BC-2.06.024 PC-1: render `command: <command>` label line.
    // BC-2.06.024 PC-1.4: if command is empty, render `command: (empty)`.
    let label_value = if command.is_empty() {
        "(empty)".to_string()
    } else {
        command.to_string()
    };
    let label_line = format!("command: {label_value}");

    Paragraph::new(Line::from(label_line))
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

/// Render a `ToolPayload::Read { path }` body inside `area`.
///
/// Renders a bordered `Block` titled `"File"` containing the label line
/// `path: <path>` per BC-2.06.024 PC-2. If `path` is empty, renders
/// `path: (empty)` as a safe fallback (BC-2.06.024 PC-2.4). Long paths
/// wrap using `Wrap { trim: false }` (BC-2.06.024 INV-4).
///
/// # Parameters
///
/// - `path`: the filesystem path from `ToolPayload::Read`.
/// - `area`: the body area inside the modal (header and footer already excluded).
/// - `buf`: the ratatui `Buffer` to render into.
pub fn render_read_payload(path: &std::path::Path, area: Rect, buf: &mut Buffer) {
    let block = Block::default().borders(Borders::ALL).title("File");
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // BC-2.06.024 PC-2: render `path: <path>` label line.
    // BC-2.06.024 PC-2.4: if path is empty, render `path: (empty)`.
    let path_str = path.display().to_string();
    let label_value = if path_str.is_empty() {
        "(empty)".to_string()
    } else {
        path_str
    };
    let content = format!("path: {label_value}");

    Paragraph::new(Line::from(content))
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

/// Render a `ToolPayload::Edit { old_content, new_content, path }` body as a
/// unified diff inside `area` (AC-005, BC-2.06.010 / BC-2.06.015).
///
/// Diff is computed via `similar::TextDiff::from_lines(old_content, new_content)`.
/// Lines are colored:
/// - `ChangeTag::Delete` → red (`Color::Red`), prefix `-`
/// - `ChangeTag::Insert` → green (`Color::Green`), prefix `+`
/// - `ChangeTag::Equal`  → default color, prefix ` `
///
/// The diff area height is capped to `(area.height - 8)` rows (BC-2.06.010 PC-5).
/// The modal header for Edit payloads shows `"Edit: <path>"`.
///
/// # Purity boundary
///
/// `similar` is used ONLY in this function. It MUST NOT be called from `monocle-core`.
///
/// # Parameters
///
/// - `old_content`: the original file content.
/// - `new_content`: the proposed file content after the edit.
/// - `path`: the file path being edited (shown in the header).
/// - `area`: the body area inside the modal.
/// - `buf`: the ratatui `Buffer` to render into.
pub fn render_edit_payload(
    old_content: &str,
    new_content: &str,
    path: &std::path::Path,
    area: Rect,
    buf: &mut Buffer,
) {
    // Compute the height cap: area.height - 8 rows (BC-2.06.010 PC-5).
    // Minimum 0 so saturating_sub is used.
    let height_cap = (area.height as usize).saturating_sub(8);

    // Title block showing "Edit: <path>".
    let path_str = path.display().to_string();
    let title = format!("Edit: {path_str}");
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Compute unified diff via `similar` (AC-005, BC-2.06.015).
    let diff = TextDiff::from_lines(old_content, new_content);

    // Collect diff lines up to the height cap.
    let mut diff_lines: Vec<Line<'static>> = Vec::new();

    for change in diff.iter_all_changes() {
        if height_cap > 0 && diff_lines.len() >= height_cap {
            break;
        }
        let (prefix, color) = match change.tag() {
            ChangeTag::Delete => ("-", Some(Color::Red)),
            ChangeTag::Insert => ("+", Some(Color::Green)),
            ChangeTag::Equal => (" ", None),
        };

        let value = change.value().trim_end_matches('\n').to_string();
        let style = match color {
            Some(c) => Style::default().fg(c),
            None => Style::default(),
        };

        let line = Line::from(vec![
            Span::styled(prefix.to_string(), style),
            Span::styled(value, style),
        ]);
        diff_lines.push(line);
    }

    let text = ratatui::text::Text::from(diff_lines);
    Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

/// Render a `ToolPayload::Generic { tool_name, tool_input }` body inside `area`.
///
/// Renders two label lines in a `Block` titled `"Tool Input"` (AC-006 /
/// BC-2.06.024 PC-3):
/// - `tool: <tool_name>` — identifies the requesting tool.
/// - `input: <json_excerpt>` — compact JSON (via `serde_json::to_string`),
///   truncated at 256 bytes/chars to prevent unbounded rendering.
///
/// On JSON serialization failure (BC-2.06.024 EC-007) renders
/// `input: (unrepresentable)` and emits `tracing::warn!`.
///
/// If content exceeds `(area.height - 6)` lines, a `"↑↓ to scroll"` hint is
/// rendered as the last line of the body (not in the footer — to ensure exactly
/// one hint on overflow and zero hints when no overflow per AC-006).
///
/// # Parameters
///
/// - `tool_name`: the tool name string (used in the `tool:` label line and block title).
/// - `tool_input`: the raw JSON tool input value.
/// - `area`: the body area inside the modal (header and footer already excluded).
/// - `buf`: the ratatui `Buffer` to render into.
pub fn render_generic_payload(
    tool_name: &str,
    tool_input: &serde_json::Value,
    area: Rect,
    buf: &mut Buffer,
) {
    let block = Block::default().borders(Borders::ALL).title("Tool Input");
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Produce JSON excerpt, truncated at 256 chars (BC-2.06.024 PC-3).
    // BC-2.06.024 EC-007: on serialization failure, render `input: (unrepresentable)`
    // and log at WARN level. `serde_json::Value` serialization should not fail in
    // practice, but the fallback is required by the BC.
    let full_json = match serde_json::to_string(tool_input) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                tool_name = %tool_name,
                error = %e,
                "render_generic_payload: serde_json::to_string failed for tool_input \
                 (BC-2.06.024 EC-007) — rendering (unrepresentable) fallback"
            );
            let fallback_line = Line::from("input: (unrepresentable)");
            Paragraph::new(ratatui::text::Text::from(vec![
                Line::from(format!("tool: {tool_name}")),
                fallback_line,
            ]))
            .wrap(Wrap { trim: false })
            .render(inner, buf);
            return;
        }
    };
    let excerpt = if full_json.len() > 256 {
        // Truncate at a UTF-8 character boundary at or before 256 bytes.
        let truncate_at = full_json
            .char_indices()
            .take_while(|(i, _)| *i < 256)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}…", &full_json[..truncate_at])
    } else {
        full_json
    };

    // Determine if scroll hint is needed (AC-006 / BC-2.06.024 PC-3).
    // Show scroll hint when JSON content (tool: + input: lines) exceeds the
    // "modal height minus 6" threshold per AC-006.
    // When area.height <= 6, the threshold is 0: any non-empty content overflows.
    let area_height = area.height as usize;
    let json_line_count = excerpt.lines().count();
    // threshold = area.height - 6 (floored at 0 via saturating_sub)
    let threshold = area_height.saturating_sub(6);
    // Content overflows when: json_line_count > threshold.
    // When threshold == 0 (area.height <= 6), show scroll hint if any JSON content exists.
    let show_scroll = json_line_count > threshold;

    if show_scroll && inner.height >= 2 {
        // Reserve the last inner row for the scroll hint; render content above it.
        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height - 1,
        };
        let hint_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };

        let content_lines: Vec<Line<'static>> = vec![
            Line::from(format!("tool: {tool_name}")),
            Line::from(format!("input: {excerpt}")),
        ];
        let text = ratatui::text::Text::from(content_lines);
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .render(content_area, buf);

        Paragraph::new(Line::from(Span::styled(
            "↑↓ to scroll",
            Style::default().fg(Color::DarkGray),
        )))
        .render(hint_area, buf);
    } else {
        let content_lines: Vec<Line<'static>> = vec![
            Line::from(format!("tool: {tool_name}")),
            Line::from(format!("input: {excerpt}")),
        ];
        let text = ratatui::text::Text::from(content_lines);
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .render(inner, buf);
    }
}

// ---------------------------------------------------------------------------
// Internal layout helpers (not pub — render_overlay_widget uses these)
// ---------------------------------------------------------------------------

/// Compute the modal `Rect` centered within `terminal_area`.
///
/// Width: `min(terminal_area.width.saturating_sub(4), 100)` (AC-001).
/// Height: `min(terminal_area.height.saturating_sub(4), 30)` — implementation-defined cap.
fn modal_rect(terminal_area: Rect) -> Rect {
    let modal_width = terminal_area.width.saturating_sub(4).min(100);
    let modal_height = terminal_area.height.saturating_sub(4).min(30);

    // Center the modal horizontally and vertically.
    let x = terminal_area.x + (terminal_area.width.saturating_sub(modal_width)) / 2;
    let y = terminal_area.y + (terminal_area.height.saturating_sub(modal_height)) / 2;

    Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    }
}

/// Build the header `Line` for the overlay modal.
///
/// Format: `"Permission Request | session: <session_id> | tool: <tool_name> | (1 of N) | Waiting: <Ns>"`
///
/// `elapsed_secs` is `Instant::now().duration_since(modal.received_at).as_secs()`.
fn build_header_line(
    session_id: &str,
    tool_name: &str,
    stack_depth: usize,
    elapsed_secs: u64,
) -> Line<'static> {
    let text = format!(
        "Permission Request | session: {} | tool: {} | (1 of {}) | Waiting: {}s",
        session_id, tool_name, stack_depth, elapsed_secs
    );
    Line::from(Span::styled(text, Style::default().fg(Color::White)))
}

/// Build the footer `Line` with keyboard hints.
///
/// Standard format: `"[y] Accept  [A] Accept Always  [n/r] Reject  [Esc] No-op"`.
///
/// Scroll hints (`"↑↓ to scroll"`) are NOT appended here — they are rendered by
/// `render_generic_payload` in the body area when content overflows (AC-006 /
/// BC-2.06.024 PC-3). Keeping the scroll hint exclusively in the body ensures:
/// - Non-overflowing Generic → zero scroll hints (no spurious footer hint).
/// - Overflowing Generic → exactly one scroll hint (body only, not double).
fn build_footer_line() -> Line<'static> {
    let text = "[y] Accept  [A] Accept Always  [n/r] Reject  [Esc] No-op";
    Line::from(Span::styled(text, Style::default().fg(Color::DarkGray)))
}
