//! Profile picker modal widget for monocle-tui (S-031).
//!
//! Renders the `ProfilePickerState` from `App::profile_picker` as a centered
//! modal overlay using `ratatui::widgets::Clear` + `Block` + `List`.
//!
//! # Layout (AC-002, BC-2.07.004 PC-2 / BC-2.07.005 PC-2)
//!
//! - A centered floating modal with a titled border (`"Profile Picker"`).
//! - Each row shows a profile name; the currently active profile is marked
//!   with a `"* "` prefix.
//! - When no profiles exist, renders: `"No profiles configured. Edit config.json
//!   to add profiles."` (AC-002 / BC-2.07.005 PC-3).
//!
//! # Keyboard isolation (AC-004, BC-2.07.005 PC-9 / BC-2.07.004 INV-4)
//!
//! While the picker is open, ALL key events are consumed by the picker handler
//! in `app.rs`. This widget is purely presentational — keyboard handling is in
//! the `dispatch_key_event` / picker pre-check path.
//!
//! # AppMode contract (AC-008, BC-2.07.004 INV-1 / BC-2.07.005 INV-4)
//!
//! The picker MUST NOT use `AppMode::Overlay`. It is modeled as
//! `Option<ProfilePickerState>` in `App` and can coexist over any `AppMode`.

use crate::app::ProfilePickerState;
use monocle_config::MonocleConfig;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Widget},
};

/// No-profiles message rendered when `harness_profiles` is empty (AC-002 / BC-2.07.005 PC-3).
pub const NO_PROFILES_MSG: &str = "No profiles configured. Edit config.json to add profiles.";

/// Compute the centered modal `Rect` for the profile picker.
///
/// The modal is `min(terminal_width - 4, 60)` wide and `min(profiles + 4, 20)` tall,
/// centered within `area`. Width and height are each at minimum 4 to avoid zero-size
/// rendering panics.
pub fn modal_rect(profile_count: usize, area: Rect) -> Rect {
    // Width: min(terminal_width - 4, 60), minimum 4.
    let modal_width = area.width.saturating_sub(4).clamp(4, 60);

    // Height: profiles + 2 (borders) + 1 (title) + 1 (padding) = profiles + 4,
    // capped at 20, minimum 4. At least 1 row for the empty message.
    let content_rows = (profile_count.max(1) as u16).saturating_add(4);
    let modal_height = content_rows.clamp(4, 20);

    // Center horizontally and vertically within the area.
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;

    Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    }
}

/// Render the profile picker modal over `area`.
///
/// Clears the modal rectangle first (via `Clear`) so the picker floats above
/// whatever is rendered beneath it (BC-2.07.005 PC-1 — picker appears over any
/// `AppMode`). Then renders a `Block` border with title `"Profile Picker"`,
/// and a `List` of profile names with the active profile marked `"* "`.
///
/// When `config.harness_profiles` is empty, renders the no-profiles message
/// (BC-2.07.005 PC-3 / AC-002 / BC-2.07.004 PC-2).
///
/// # Arguments
///
/// - `state`: the current `ProfilePickerState` (selection index + profile name snapshot).
/// - `config`: the loaded `MonocleConfig` — used to determine the `active_profile`
///   for marking the active row with `"* "`.
/// - `area`: the full terminal area to center the modal within.
/// - `frame_buf`: the ratatui `Buffer` to render into.
pub fn render_profile_picker(
    state: &ProfilePickerState,
    config: &MonocleConfig,
    area: Rect,
    frame_buf: &mut Buffer,
) {
    let modal_area = modal_rect(state.profiles.len(), area);

    // Clear the modal region so the picker floats above the underlying render.
    Clear.render(modal_area, frame_buf);

    // Determine the active profile: look through project_profiles values for the
    // first one that matches a profile in harness_profiles (first-match semantics,
    // same as open_profile_picker — AC-002 / BC-2.07.004 PC-2).
    let active_profile_id: Option<&str> = config
        .project_profiles
        .values()
        .find(|profile_id| config.harness_profiles.iter().any(|p| &&p.id == profile_id))
        .map(|s| s.as_str());

    // Build the list widget.
    let (list, mut list_state) = build_profile_list(&state.profiles, active_profile_id);

    // Set the selected item in ListState.
    if !state.profiles.is_empty() {
        list_state.select(Some(state.selected_index));
    }

    // Render block (border + title) first to get the inner area.
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            "Profile Picker",
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(Color::White));

    // Compute the inner area (subtract borders).
    let inner_area = block.inner(modal_area);

    // Render the block border.
    block.render(modal_area, frame_buf);

    // Render the list inside the block inner area.
    ratatui::widgets::StatefulWidget::render(list, inner_area, frame_buf, &mut list_state);
}

/// Build the `List` widget for the profile picker.
///
/// Each item is a `ListItem`. The active profile (matched against
/// `active_profile_id` from the config's `project_profiles`) gets a `"* "` prefix;
/// all others get `"  "` (two spaces for alignment).
///
/// When `profiles` is empty, returns a `List` with a single item containing the
/// no-profiles message string.
pub fn build_profile_list<'a>(
    profiles: &'a [String],
    active_profile_id: Option<&'a str>,
) -> (List<'a>, ListState) {
    let items: Vec<ListItem<'a>> = if profiles.is_empty() {
        // AC-002 / BC-2.07.005 PC-3: empty profiles message.
        vec![ListItem::new(Line::from(Span::styled(
            NO_PROFILES_MSG,
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        profiles
            .iter()
            .map(|id| {
                // Mark active profile with "* " prefix; all others get "  " for alignment.
                let prefix = if active_profile_id == Some(id.as_str()) {
                    "* "
                } else {
                    "  "
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Yellow)),
                    Span::raw(id.as_str()),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    (list, ListState::default())
}
