// Stub module: all function bodies are todo!(). Parameters are intentionally
// unused and private helpers are dead until the S-027 implementer fills them in.
#![allow(unused_variables, unused_imports, dead_code)]

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

use monocle_core::tui::state::PromptModal;
use ratatui::{buffer::Buffer, layout::Rect, text::Line, Frame};

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
    todo!(
        "S-027: render overlay modal for {:?} stack_depth={} area={:?}",
        modal.tool_name,
        stack_depth,
        area
    )
}

/// Apply `Modifier::DIM` to all cells in `area` to visually de-emphasize the
/// content behind the overlay modal (AC-002, BC-2.06.010 PC-2).
///
/// The status bar row is NOT included in `area` — the caller must exclude it
/// before calling this function so the status bar remains full-brightness
/// (BC-2.06.019 PC-1).
pub fn render_dimmed_background(area: Rect, buf: &mut Buffer) {
    todo!(
        "S-027: apply Modifier::DIM to background area={:?}",
        area
    )
}

// ---------------------------------------------------------------------------
// Payload-specific body renderers (AC-003 through AC-006, BC-2.06.024)
// ---------------------------------------------------------------------------

/// Render a `ToolPayload::Bash { command }` body inside `area`.
///
/// Renders `command` in a bordered `Block` titled `"Command"`. If `command`
/// exceeds `available_height` rows, it is truncated with `"... (truncated)"`.
///
/// # Parameters
///
/// - `command`: the shell command string from `ToolPayload::Bash`.
/// - `area`: the body area inside the modal (header and footer already excluded).
/// - `buf`: the ratatui `Buffer` to render into.
pub fn render_bash_payload(command: &str, area: Rect, buf: &mut Buffer) {
    todo!(
        "S-027: render Bash payload command={:?} area={:?}",
        command,
        area
    )
}

/// Render a `ToolPayload::Read { path }` body inside `area`.
///
/// Renders `path` in a single-line `Block` titled `"File"`.
///
/// # Parameters
///
/// - `path`: the filesystem path from `ToolPayload::Read`.
/// - `area`: the body area inside the modal.
/// - `buf`: the ratatui `Buffer` to render into.
pub fn render_read_payload(path: &std::path::Path, area: Rect, buf: &mut Buffer) {
    todo!(
        "S-027: render Read payload path={:?} area={:?}",
        path,
        area
    )
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
    todo!(
        "S-027: render Edit payload diff path={:?} area={:?}",
        path,
        area
    )
}

/// Render a `ToolPayload::Generic { tool_name, tool_input }` body inside `area`.
///
/// Renders `serde_json::to_string_pretty(tool_input)` in a `Block` titled
/// `"Tool Input"`. If the JSON exceeds `(area.height - 6)` rows, scroll hints
/// `"↑↓ to scroll"` are shown in the footer alongside the decision keys.
///
/// # Parameters
///
/// - `tool_name`: the tool name string (for the block title).
/// - `tool_input`: the raw JSON tool input value.
/// - `area`: the body area inside the modal.
/// - `buf`: the ratatui `Buffer` to render into.
pub fn render_generic_payload(
    tool_name: &str,
    tool_input: &serde_json::Value,
    area: Rect,
    buf: &mut Buffer,
) {
    todo!(
        "S-027: render Generic payload tool_name={:?} area={:?}",
        tool_name,
        area
    )
}

// ---------------------------------------------------------------------------
// Internal layout helpers (not pub — render_overlay_widget uses these)
// ---------------------------------------------------------------------------

/// Compute the modal `Rect` centered within `terminal_area`.
///
/// Width: `min(terminal_area.width.saturating_sub(4), 100)` (AC-001).
/// Height: `min(terminal_area.height.saturating_sub(4), 30)` — implementation-defined cap.
fn modal_rect(terminal_area: Rect) -> Rect {
    todo!(
        "S-027: compute modal_rect for terminal_area={:?}",
        terminal_area
    )
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
    todo!(
        "S-027: build header line session_id={:?} tool={:?} depth={} elapsed={}s",
        session_id,
        tool_name,
        stack_depth,
        elapsed_secs
    )
}

/// Build the footer `Line` with keyboard hints.
///
/// Standard format: `"[y] Accept  [A] Accept Always  [n/r] Reject  [Esc] No-op"`.
/// If `show_scroll_hint` is true, appends `"  ↑↓ to scroll"` (for Generic payloads
/// exceeding the available height — AC-006).
fn build_footer_line(show_scroll_hint: bool) -> Line<'static> {
    todo!(
        "S-027: build footer line show_scroll_hint={}",
        show_scroll_hint
    )
}
