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
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
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
    // BC-2.05.002 PC-2: convert ring_tail HookEventRecord → HookEventRow.
    // Parse hook_type string via serde JSON round-trip (canonical discriminant format).
    // On parse failure (unknown future hook type), fall back to PreToolUse with TRACE log
    // (BC-2.05.004 PC-5 catch-all forward-compat requirement).
    let hook_type = serde_json::from_value::<HookType>(serde_json::Value::String(
        record.hook_type.clone(),
    ))
    .unwrap_or_else(|_| {
        tracing::trace!(
            hook_type = %record.hook_type,
            "hook_event_row_from_record: unknown hook_type string, falling back to PreToolUse \
             (BC-2.05.004 PC-5 forward-compat catch-all)"
        );
        HookType::PreToolUse
    });

    HookEventRow {
        received_at: Instant::now(),
        hook_type,
        session_id: record.session_id.clone(),
        // ring_tail entries carry no latency (stored as None — BC-2.06.018 EC-118).
        latency_ms: None,
        // Historical entries are not pending (pending is overlay-driven, not historical).
        pending: false,
    }
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
    // BC-2.05.004: convert HookEventReceived IPC fields → HookEventRow for the ribbon.
    HookEventRow {
        received_at: Instant::now(),
        hook_type,
        session_id,
        latency_ms: Some(latency_ms),
        pending: false,
    }
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

/// Empty state placeholder text for the Event Ribbon (BC-2.06.018 EC-114).
pub const EVENT_RIBBON_EMPTY: &str = "No events yet";

/// Column widths for the Event Ribbon (BC-2.06.018 PC-1).
const COL_TIMESTAMP: usize = 12;
const COL_HOOK_TYPE: usize = 16;
const COL_SESSION: usize = 10;
const COL_LATENCY: usize = 8;
const COL_STATUS: usize = 8;

/// Format the hook type display name (BC-2.06.018 PC-1 Hook type column).
///
/// BC-2.05.004 PC-5: unknown future variants render as "unknown" (catch-all).
fn format_hook_type(ht: HookType) -> &'static str {
    match ht {
        HookType::PreToolUse => "PreToolUse",
        HookType::Notification => "Notification",
        HookType::SessionStart => "SessionStart",
        HookType::UserPromptSubmit => "UserPromptSubmit",
        HookType::Stop => "Stop",
        // BC-2.05.004 PC-5: non_exhaustive catch-all — render safe fallback.
        _ => "unknown",
    }
}

impl StatefulWidget for EventRibbon<'_> {
    type State = EventRibbonState;

    /// Render the Event Ribbon panel into `buf` within `area`.
    ///
    /// BC-2.06.018 PC-1: column layout (Timestamp | HookType | Session | Latency | Status).
    /// BC-2.06.018 PC-2: newest-first ordering (index 0 = newest).
    /// BC-2.06.018 PC-3: VecDeque bounded to panel_height (enforced via trim_to_panel_height).
    /// BC-2.06.018 PC-4: PENDING in yellow for unresolved PreToolUse rows.
    /// BC-2.06.018 EC-114: empty state placeholder "No events yet".
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let panel_height = area.height as usize;

        // Collect events filtered by selected session_id (BC-2.05.004 INV-3: client-side only).
        let filtered: Vec<&HookEventRow> = match self.selected_session_id {
            Some(sid) => self
                .app
                .event_ribbon_events
                .iter()
                .filter(|r| r.session_id == sid)
                .collect(),
            None => self.app.event_ribbon_events.iter().collect(),
        };

        // Trim to panel_height for display (BC-2.06.018 PC-3 dynamic cap).
        let visible: Vec<&HookEventRow> = filtered.into_iter().take(panel_height).collect();

        if visible.is_empty() {
            // BC-2.06.018 EC-114: empty state.
            Widget::render(
                Paragraph::new(Line::from(EVENT_RIBBON_EMPTY)),
                area,
                buf,
            );
            return;
        }

        // Use the earliest received_at as epoch for timestamp formatting.
        let epoch = visible.last().map(|r| r.received_at).unwrap_or_else(Instant::now);

        let items: Vec<ListItem> = visible
            .iter()
            .map(|row| {
                let ts = format_timestamp(row.received_at, epoch);
                let ht = format_hook_type(row.hook_type);
                // Session ID: first 8 chars (BC-2.06.018 INV-4).
                let sid = if row.session_id.len() > 8 {
                    &row.session_id[..8]
                } else {
                    &row.session_id
                };
                let lat = match row.latency_ms {
                    Some(ms) => format!("{ms}ms"),
                    None => "\u{2014}".to_string(), // em-dash (BC-2.06.018 EC-118)
                };

                if row.pending {
                    // BC-2.06.018 PC-4: PENDING in yellow for unresolved PreToolUse.
                    let spans = vec![
                        Span::raw(format!("{ts:<width$}", width = COL_TIMESTAMP + 1)),
                        Span::raw(format!("{ht:<width$}", width = COL_HOOK_TYPE + 1)),
                        Span::raw(format!("{sid:<width$}", width = COL_SESSION + 1)),
                        Span::raw(format!("{lat:<width$}", width = COL_LATENCY + 1)),
                        Span::styled("PENDING", Style::default().fg(Color::Yellow)),
                    ];
                    ListItem::new(Line::from(spans))
                } else {
                    let text = format!(
                        "{ts:<ts_w$} {ht:<ht_w$} {sid:<sid_w$} {lat:<lat_w$} {:<st_w$}",
                        "",
                        ts_w = COL_TIMESTAMP,
                        ht_w = COL_HOOK_TYPE,
                        sid_w = COL_SESSION,
                        lat_w = COL_LATENCY,
                        st_w = COL_STATUS,
                    );
                    ListItem::new(text)
                }
            })
            .collect();

        let list = List::new(items);
        StatefulWidget::render(list, area, buf, &mut state.list_state);
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
    // BC-2.06.018 PC-3: prepend (newest at front); evict oldest (back) if at cap.
    if panel_height > 0 && events.len() >= panel_height {
        events.pop_back();
    }
    events.push_front(row);
}

/// Trim a `VecDeque<HookEventRow>` to at most `panel_height` entries.
///
/// Called after a terminal resize event to enforce the dynamic `panel_height` cap
/// (BC-2.06.018 INV-3). Removes oldest entries (from the back) until `len <= panel_height`.
pub fn trim_to_panel_height(events: &mut VecDeque<HookEventRow>, panel_height: usize) {
    // BC-2.06.018 INV-3: trim oldest (back) until len <= panel_height.
    while events.len() > panel_height {
        events.pop_back();
    }
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
    // BC-2.06.018 PC-1: format as HH:MM:SS.mmm relative to epoch.
    // Guard against clock skew (received_at before epoch).
    let elapsed = match received_at.checked_duration_since(epoch) {
        Some(d) => d,
        None => return "??:??:??.???".to_string(),
    };
    let total_ms = elapsed.as_millis();
    let hours = (total_ms / 3_600_000) as u64;
    let minutes = ((total_ms % 3_600_000) / 60_000) as u64;
    let seconds = ((total_ms % 60_000) / 1_000) as u64;
    let millis = (total_ms % 1_000) as u64;
    if hours > 99 {
        // Overflow guard: cap display at 99:59:59.999.
        return "??:??:??.???".to_string();
    }
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
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
    // BC-2.06.018 INV-1 / AC-009: reset scroll to top (row 0) and clear pinned_top.
    // No IPC request issued — all events held client-side, filter is render-time.
    state.list_state.select(Some(0));
    state.pinned_top = false;
}
