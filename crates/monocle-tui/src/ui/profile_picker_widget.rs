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
    widgets::{List, ListState},
};

/// Compute the centered modal `Rect` for the profile picker.
///
/// The modal is `min(terminal_width - 4, 60)` wide and `min(profiles + 4, 20)` tall,
/// centered within `area`. Width and height are each at minimum 4 to avoid zero-size
/// rendering panics.
#[allow(clippy::todo)]
pub fn modal_rect(_profile_count: usize, _area: Rect) -> Rect {
    todo!("S-031: compute centered Rect for profile picker modal")
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
#[allow(clippy::todo)]
pub fn render_profile_picker(
    _state: &ProfilePickerState,
    _config: &MonocleConfig,
    _area: Rect,
    _frame_buf: &mut Buffer,
) {
    todo!("S-031: render profile picker modal (Clear + Block + List)")
}

/// Build the `List` widget for the profile picker.
///
/// Each item is a `ListItem`. The active profile (matched against
/// `active_profile_id` from the config's `project_profiles`) gets a `"* "` prefix;
/// all others get `"  "` (two spaces for alignment).
///
/// When `profiles` is empty, returns a `List` with a single item containing the
/// no-profiles message string.
#[allow(clippy::todo)]
pub fn build_profile_list<'a>(
    _profiles: &'a [String],
    _active_profile_id: Option<&'a str>,
) -> (List<'a>, ListState) {
    todo!("S-031: build List widget for profile picker entries")
}
