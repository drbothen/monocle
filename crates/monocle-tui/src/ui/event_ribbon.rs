//! Event Ribbon panel widget (BC-2.06.018, S-028).
//!
//! # Overview
//!
//! The Event Ribbon renders a scrollable, newest-first log of hook events. Events are
//! stored in `App::event_ribbon_events: VecDeque<HookEventRow>` (all sessions) and
//! filtered client-side by `session_id` for display (BC-2.05.004 INV-3: no IPC-layer
//! filtering; filtering is purely client-side in the TUI).
//!
//! # Column layout (BC-2.06.018 PC-1)
//!
//! | Column    | Source                       | Width   |
//! |-----------|------------------------------|---------|
//! | Timestamp | `HookEventRow::received_at`  | 12 chars |
//! | Hook type | `HookType` display name      | 16 chars |
//! | Session   | `session_id` first 8 chars   | 10 chars |
//! | Latency   | `latency_ms` as `NNNms`      | 8 chars  |
//! | Status    | `PENDING` or blank           | 8 chars  |
//!
//! # Scroll state (BC-2.06.018 PC-5)
//!
//! `EventRibbonState` holds a `ratatui::widgets::ListState` for scroll offset tracking
//! and a `pinned_top: bool` flag that controls auto-scroll behaviour (BC-2.06.018 PC-8
//! / AC-008): when `pinned_top` is `false` (user is at the bottom), new events
//! cause the ribbon to auto-scroll to the newest event; when `true` (user has scrolled
//! up), the scroll offset is preserved and new events accumulate silently.
//!
//! # VecDeque cap (BC-2.06.018 PC-3)
//!
//! The `App::event_ribbon_events` VecDeque is bounded to `panel_height` entries,
//! determined at render time. Oldest entries (back) are popped when full. This is a
//! fixed-size sliding window, not an infinite log.

use monocle_ipc::types::HookType;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListState, StatefulWidget},
};
use std::collections::VecDeque;
use std::time::Instant;

use crate::app::App;

// ---------------------------------------------------------------------------
// HookEventRow — TUI-side render type (BC-2.06.018 PC-1)
// ---------------------------------------------------------------------------

/// A single event row in the Event Ribbon panel (BC-2.06.018 PC-1).
///
/// This is the TUI-side render type produced from `ServerToClient::HookEventReceived`
/// messages (BC-2.05.004) and from `InitialState::ring_tail` records (BC-2.05.002).
/// It holds only the fields required for display in the Event Ribbon columns.
///
/// Fields follow BC-2.06.018 PC-1 column definitions:
/// - `received_at`: used for Timestamp column (`HH:MM:SS.mmm`)
/// - `hook_type`: used for Hook type column
/// - `session_id`: full session ID (display truncates to 8 chars — BC-2.06.018 INV-4)
/// - `latency_ms`: used for Latency column (`NNNms` or `—`)
/// - `pending`: used for Status column (`PENDING` in yellow when `true`)
///
/// # Sourcing
///
/// Rows are created by two code paths per BC-2.05.002 + BC-2.05.004:
/// 1. `InitialState::ring_tail` (on connect, BC-2.05.002): each `HookEventRecord`
///    is converted to `HookEventRow` by `hook_event_row_from_record()`.
/// 2. `HookEventReceived` streaming messages (BC-2.05.004): each message is
///    converted to `HookEventRow` by `hook_event_row_from_received()`.
///
/// Both paths push to `App::event_ribbon_events`. Filtering by `session_id` for
/// display is purely client-side (BC-2.05.004 INV-3).
#[derive(Debug, Clone)]
pub struct HookEventRow {
    /// Wall-clock instant at which the event was received by the TUI.
    ///
    /// For `ring_tail` entries, this is the TUI's receive time (at connect), not
    /// the original daemon hook-receive time. For `HookEventReceived` messages,
    /// this is the TUI's message-arrival time.
    /// Used for Timestamp column rendering (`HH:MM:SS.mmm`).
    pub received_at: Instant,
    /// Hook type discriminant (e.g., `HookType::PreToolUse`).
    ///
    /// Used for Hook type column rendering. `#[non_exhaustive]` on `HookType`
    /// requires a catch-all arm in rendering code (BC-2.05.004 PC-5).
    pub hook_type: HookType,
    /// Full session identifier. Display truncates to first 8 characters.
    ///
    /// The full ID is stored for future detail views (BC-2.06.018 INV-4).
    /// The session_id from `HookEventRecord.session_id` or `HookEventReceived.session_id`.
    pub session_id: String,
    /// Wall-clock milliseconds from HTTP POST receipt to HTTP ACK (BC-2.05.004 PC-1).
    ///
    /// `None` when latency was not measured (e.g., entries from `ring_tail` where
    /// latency data was not retained in `HookEventRecord`). BC-2.06.018 EC-118:
    /// renders as `—` (em-dash) rather than `0ms` when `None`.
    pub latency_ms: Option<u64>,
    /// Whether this row corresponds to an unresolved `PreToolUse` permission prompt.
    ///
    /// `true` → renders `PENDING` in yellow in the Status column (BC-2.06.018 PC-4).
    /// Reverts to `false` when `ClientToServer::PermissionDecision` is sent.
    pub pending: bool,
}

/// Convert a `HookEventRecord` (from `InitialState::ring_tail`) to a `HookEventRow`.
///
/// Called during the `on_initial_state` handler to pre-populate `App::event_ribbon_events`
/// from the daemon's ring snapshot (BC-2.05.002 PC-2).
///
/// `latency_ms` is always `None` for `ring_tail` entries because `HookEventRecord`
/// does not carry latency data (the latency is an IPC-layer field on `HookEventReceived`,
/// not stored in the JSONL ring). `pending` is `false` (ring_tail entries are historical;
/// overlay state is managed separately via `InitialState::overlay_stack`).
///
/// The `hook_type` field in `HookEventRecord` is a `String` discriminant (e.g.,
/// `"PreToolUse"`). This function parses it into `HookType` using `HookType`'s
/// serde-compatible string representation; on parse failure (unknown future hook type
/// or malformed string), falls back to `HookType::PreToolUse` with a TRACE log
/// (BC-2.05.004 PC-5 catch-all forward-compat requirement).
pub fn hook_event_row_from_record(record: &monocle_ipc::types::HookEventRecord) -> HookEventRow {
    todo!(
        "S-028 implement: parse record.hook_type string -> HookType, build HookEventRow \
         (BC-2.05.002 PC-2 ring_tail pre-population)"
    )
}

/// Convert a `ServerToClient::HookEventReceived` payload to a `HookEventRow`.
///
/// Called in `on_hook_event_received` (the `HookEventReceived` IPC handler in `app.rs`)
/// to append live events to `App::event_ribbon_events` (BC-2.05.004).
///
/// `latency_ms` is `Some(latency_ms)` from the IPC message field.
/// `pending` is initially `false`; set to `true` by the overlay-push logic in
/// `on_permission_prompt_queued` when the session_id + hook_type matches a new
/// `PreToolUse` prompt (see BC-2.06.018 PC-4 — overlay management is in S-026/S-027;
/// the `pending` flag here is the TUI-side display state).
pub fn hook_event_row_from_received(
    hook_type: HookType,
    session_id: String,
    latency_ms: u64,
) -> HookEventRow {
    todo!(
        "S-028 implement: construct HookEventRow from HookEventReceived fields \
         (BC-2.05.004 streaming path)"
    )
}

// ---------------------------------------------------------------------------
// EventRibbonState — scroll tracking (BC-2.06.018 PC-5)
// ---------------------------------------------------------------------------

/// Mutable scroll state for the `EventRibbon` panel widget.
///
/// Wraps `ratatui::widgets::ListState` (scroll offset and selection) plus
/// `pinned_top: bool` for the auto-scroll / pin-at-top logic (BC-2.06.018 PC-8 / AC-008).
///
/// # Auto-scroll semantics (BC-2.06.018 AC-008)
///
/// - `pinned_top = false` (initial state): when a new `HookEventReceived` event arrives
///   and matches the selected session, the ribbon auto-scrolls to row 0 (newest event
///   at top per BC-2.06.018 PC-2 newest-first ordering).
/// - `pinned_top = true`: the user has manually scrolled up (away from newest); new
///   events accumulate in `App::event_ribbon_events` but the scroll offset is not moved.
///   Scrolling back to the top (row 0) sets `pinned_top = false` again.
#[derive(Debug, Default)]
pub struct EventRibbonState {
    /// Underlying ratatui list scroll/selection state.
    pub list_state: ListState,
    /// True when the user has scrolled away from the newest event (row 0).
    ///
    /// Controls whether new events trigger auto-scroll. Set to `true` when
    /// the user presses `j`/`↓` (scroll toward older events). Reset to `false`
    /// when the user reaches row 0 (scrolls back to the top / newest event).
    pub pinned_top: bool,
}

// ---------------------------------------------------------------------------
// EventRibbon widget (BC-2.06.018 PC-1..PC-6)
// ---------------------------------------------------------------------------

/// The Event Ribbon panel widget (BC-2.06.018).
///
/// Renders the scrollable list of `HookEventRow` entries for the currently selected
/// session. Rows are sourced from a client-side filtered view of `App::event_ribbon_events`
/// (filtered by `session_id` — BC-2.05.004 INV-3).
///
/// Implements `ratatui::widgets::StatefulWidget` with `State = EventRibbonState`.
pub struct EventRibbon<'a> {
    /// Reference to application state for reading `event_ribbon_events`.
    pub app: &'a App,
    /// The `session_id` of the currently selected session in the Sessions panel.
    ///
    /// Events are filtered to this `session_id` client-side (BC-2.05.004 INV-3;
    /// BC-2.06.018 INV-1: no IPC request is issued on selection change).
    /// When `None`, the ribbon renders an empty state ("No session selected").
    pub selected_session_id: Option<&'a str>,
}

impl<'a> EventRibbon<'a> {
    /// Construct an `EventRibbon` widget.
    ///
    /// `selected_session_id` should be `None` when no session is currently selected
    /// in the Sessions panel.
    pub fn new(app: &'a App, selected_session_id: Option<&'a str>) -> Self {
        Self {
            app,
            selected_session_id,
        }
    }
}

impl StatefulWidget for EventRibbon<'_> {
    type State = EventRibbonState;

    /// Render the Event Ribbon panel into `buf` within `area`.
    ///
    /// Implementation is `todo!()` — the stub provides the correct signature and type
    /// contract for the test-writer to compile tests against (S-028 stub discipline).
    fn render(self, _area: Rect, _buf: &mut Buffer, _state: &mut Self::State) {
        todo!(
            "S-028 implement: render EventRibbon panel with column layout, newest-first ordering, \
             PENDING status, scroll state (BC-2.06.018 PC-1..PC-6)"
        )
    }
}

// ---------------------------------------------------------------------------
// Rolling-window helpers (BC-2.06.018 PC-3)
// ---------------------------------------------------------------------------

/// Push a new `HookEventRow` onto the front of the rolling window `VecDeque`.
///
/// If the `VecDeque` is already at `panel_height` capacity, the oldest entry (back)
/// is popped before the new entry is pushed to the front (BC-2.06.018 PC-3:
/// fixed-size sliding window). `panel_height` is the count of visible rows available
/// at render time; it must be determined by the caller from the allocated `Rect`.
///
/// # Capacity semantics
///
/// The cap is dynamic: on terminal resize, `panel_height` changes and the `VecDeque`
/// is trimmed on the next render cycle (BC-2.06.018 INV-3). This function only
/// enforces the cap at insert time — callers should also call `trim_to_panel_height`
/// after a resize event.
pub fn push_event_row(
    events: &mut VecDeque<HookEventRow>,
    row: HookEventRow,
    panel_height: usize,
) {
    todo!(
        "S-028 implement: prepend row to VecDeque; pop back if len >= panel_height \
         (BC-2.06.018 PC-3 rolling window)"
    )
}

/// Trim a `VecDeque<HookEventRow>` to at most `panel_height` entries.
///
/// Called after a terminal resize event to enforce the dynamic `panel_height` cap
/// (BC-2.06.018 INV-3). Removes oldest entries (from the back) until `len <= panel_height`.
pub fn trim_to_panel_height(events: &mut VecDeque<HookEventRow>, panel_height: usize) {
    todo!(
        "S-028 implement: pop_back until events.len() <= panel_height \
         (BC-2.06.018 INV-3 dynamic panel_height cap)"
    )
}

// ---------------------------------------------------------------------------
// Timestamp formatter (BC-2.06.018 PC-1 Timestamp column)
// ---------------------------------------------------------------------------

/// Format an `Instant` as `HH:MM:SS.mmm` for the Timestamp column (BC-2.06.018 PC-1).
///
/// Accepts an explicit `epoch: Instant` (the `Instant` at TUI startup) so callers
/// can supply a stable reference for deterministic tests. The elapsed duration
/// `received_at - epoch` is formatted as wall-clock time.
///
/// Returns `"??:??:??.???"` if elapsed would overflow or underflow (clock skew guard).
pub fn format_timestamp(received_at: Instant, epoch: Instant) -> String {
    todo!(
        "S-028 implement: format Instant as HH:MM:SS.mmm relative to epoch \
         (BC-2.06.018 PC-1 Timestamp column)"
    )
}

// ---------------------------------------------------------------------------
// Session-change reset helper (BC-2.06.018 INV-1 / AC-009)
// ---------------------------------------------------------------------------

/// Reset the ribbon scroll state when the selected session changes (BC-2.06.018 INV-1).
///
/// On session-change:
/// 1. Re-filter `App::event_ribbon_events` by `new_session_id` for display.
/// 2. Reset `state.list_state` selection to row 0 (newest event at top).
/// 3. Clear `state.pinned_top = false` (auto-scroll resumes after session change).
///
/// No IPC request is issued (BC-2.06.018 INV-1: all events held client-side;
/// filtering is purely in-memory).
pub fn reset_on_session_change(state: &mut EventRibbonState, _new_session_id: &str) {
    todo!(
        "S-028 implement: reset scroll state to top; clear pinned_top \
         (BC-2.06.018 INV-1 / AC-009 session-change reset)"
    )
}
