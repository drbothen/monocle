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
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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
/// - `timestamp_micros`: Unix epoch microseconds used for Timestamp column (`HH:MM:SS.mmm`).
///   For `ring_tail` entries this comes from `HookEventRecord::timestamp_micros` (daemon time).
///   For streaming `HookEventReceived` messages this is the TUI's wall-clock capture at receipt.
/// - `received_at`: monotonic `Instant` for ordering only (newest-first — BC-2.06.018 PC-2).
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
    /// Unix epoch microseconds for the Timestamp column (BC-2.06.018 PC-1).
    ///
    /// For `ring_tail` entries: sourced from `HookEventRecord::timestamp_micros` (the daemon's
    /// wall-clock at hook receipt). For streaming `HookEventReceived` messages: sourced from
    /// `HookEventReceived::timestamp_micros` (the daemon's wall-clock capture — SS-ipc v1.10.0 /
    /// BC-2.05.004 PC-2). NOT `SystemTime::now()` at TUI receive time.
    ///
    /// Used exclusively for Timestamp column rendering as UTC wall-clock `HH:MM:SS.mmm`
    /// via `format_timestamp`. Stable across scroll/window changes (not an elapsed delta).
    pub timestamp_micros: i64,
    /// Monotonic instant of receipt — used ONLY for ordering (BC-2.06.018 PC-2 newest-first).
    ///
    /// NOT used for Timestamp column rendering (use `timestamp_micros` for that).
    /// Retained so that VecDeque ordering can be verified in tests via `>=` comparisons.
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

/// Capture the current wall-clock time as Unix epoch microseconds.
///
/// Used by `hook_event_row_from_record` for ring_tail entries that are pre-existing
/// when the TUI connects (the daemon has already captured their timestamp in
/// `HookEventRecord::timestamp_micros`). For streaming `HookEventReceived` messages,
/// `hook_event_row_from_received` uses the daemon-supplied `timestamp_micros` field
/// directly (SS-ipc v1.10.0 / BC-2.05.004 PC-2) rather than calling this function.
/// Falls back to 0 on `SystemTime` errors (should not occur on any supported platform).
pub fn current_timestamp_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
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
        // BC-2.06.018 PC-1: use the daemon's wall-clock timestamp (not TUI receive time).
        // ring_tail records carry timestamp_micros from the daemon's hook receipt time.
        timestamp_micros: record.timestamp_micros,
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
/// `timestamp_micros` is sourced from the `HookEventReceived::timestamp_micros` field —
/// the daemon's wall-clock capture at hook POST receipt (BC-2.05.004 PC-2 / SS-ipc v1.10.0).
/// The TUI MUST use this value, NOT `current_timestamp_micros()` at TUI receive time.
/// `pending` is initially `false`; set to `true` by the overlay-push logic in
/// `on_permission_prompt_queued` when the session_id + hook_type matches a new
/// `PreToolUse` prompt (see BC-2.06.018 PC-4 — overlay management is in S-026/S-027;
/// the `pending` flag here is the TUI-side display state).
pub fn hook_event_row_from_received(
    hook_type: HookType,
    session_id: String,
    latency_ms: u64,
    timestamp_micros: i64,
) -> HookEventRow {
    // BC-2.05.004 / SS-ipc v1.10.0: use the daemon's timestamp_micros (not TUI receive time).
    // This value comes from HookEventReceived::timestamp_micros, which the daemon captures ONCE
    // at hook POST receipt and also writes to HookEventRecord::timestamp_micros in the ring
    // (BC-2.05.004 PC-2 equality: ring and IPC carry the same daemon-clock value).
    HookEventRow {
        timestamp_micros,
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
            Widget::render(Paragraph::new(Line::from(EVENT_RIBBON_EMPTY)), area, buf);
            return;
        }

        // Epoch is unused in wall-clock mode but kept for format_timestamp signature compat.
        let epoch = Instant::now();

        let items: Vec<ListItem> = visible
            .iter()
            .map(|row| {
                // BC-2.06.018 PC-1: use timestamp_micros for stable wall-clock HH:MM:SS.mmm.
                let ts = format_timestamp(row.timestamp_micros, epoch);
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
pub fn push_event_row(events: &mut VecDeque<HookEventRow>, row: HookEventRow, panel_height: usize) {
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

/// Format a Unix epoch microsecond timestamp as UTC wall-clock `HH:MM:SS.mmm`
/// for the Timestamp column (BC-2.06.018 PC-1).
///
/// `timestamp_micros` is a Unix epoch timestamp in microseconds (as stored in
/// `HookEventRow::timestamp_micros`). The result is the UTC time of day:
/// `HH:MM:SS.mmm` where `HH` is hours since midnight UTC, `MM` minutes, `SS`
/// seconds, and `mmm` milliseconds.
///
/// The second parameter `_epoch` is accepted for backward compatibility with
/// call sites that previously passed an `Instant` epoch; it is ignored because
/// the wall-clock derivation uses only `timestamp_micros` (BC-2.06.018 PC-1:
/// stable wall-clock — not an elapsed delta from a visible epoch).
///
/// Returns `"??:??:??.???"` on negative timestamp (clock skew guard).
pub fn format_timestamp(timestamp_micros: i64, _epoch: Instant) -> String {
    // BC-2.06.018 PC-1: format as UTC wall-clock HH:MM:SS.mmm from Unix epoch microseconds.
    // Guard against negative timestamps (clock skew, test sentinel values).
    if timestamp_micros < 0 {
        return "??:??:??.???".to_string();
    }
    let total_micros = timestamp_micros as u64;
    let total_secs = total_micros / 1_000_000;
    let millis = (total_micros % 1_000_000) / 1_000;
    let hours = (total_secs / 3600) % 24;
    let minutes = (total_secs / 60) % 60;
    let seconds = total_secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

// ---------------------------------------------------------------------------
// Scroll helpers (BC-2.06.018 PC-5 / AC-007)
// ---------------------------------------------------------------------------

/// Scroll the Event Ribbon one row toward older events (down the newest-first list).
///
/// Increments `state.list_state.selected()` by 1, clamped at `events.len() - 1`
/// to avoid out-of-bounds access (BC-2.06.018 EC-116: no panic at oldest row).
///
/// Sets `state.pinned_top = true` when the new index is > 0 (user scrolled away
/// from the newest event). At index 0 → 0 (empty or already clamped) `pinned_top`
/// is not changed.
///
/// `events` is passed as a reference to determine the clamp boundary — it is NOT
/// mutated. The caller holds the full VecDeque (not a slice) so the length is the
/// authoritative event count.
pub fn scroll_ribbon_down(
    state: &mut EventRibbonState,
    events: &std::collections::VecDeque<HookEventRow>,
) {
    let event_count = events.len();
    if event_count == 0 {
        return;
    }
    let current = state.list_state.selected().unwrap_or(0);
    let next = (current + 1).min(event_count.saturating_sub(1));
    state.list_state.select(Some(next));
    // BC-2.06.018 PC-5: set pinned_top=true when user scrolls away from newest (index 0).
    if next > 0 {
        state.pinned_top = true;
    }
}

/// Scroll the Event Ribbon one row toward newer events (up the newest-first list).
///
/// Decrements `state.list_state.selected()` by 1, clamped at 0 (the newest event).
/// When the new index reaches 0, `state.pinned_top` is cleared to `false`
/// (BC-2.06.018 AC-008: auto-scroll resumes when user returns to the newest event).
pub fn scroll_ribbon_up(
    state: &mut EventRibbonState,
    _events: &std::collections::VecDeque<HookEventRow>,
) {
    let current = state.list_state.selected().unwrap_or(0);
    let prev = current.saturating_sub(1);
    state.list_state.select(Some(prev));
    // BC-2.06.018 AC-008 / PC-5: clear pinned_top when user returns to row 0 (newest).
    if prev == 0 {
        state.pinned_top = false;
    }
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

// ---------------------------------------------------------------------------
// Jump helpers (BC-2.06.018 PC-5 / AC-007: G → oldest, gg → newest)
// ---------------------------------------------------------------------------

/// Jump the Event Ribbon to the OLDEST event (last index in newest-first ordering).
///
/// `G` in vi-style navigation jumps to the bottom of the list, which is the oldest
/// event in the newest-first `VecDeque` (BC-2.06.018 PC-5: oldest = back = last index).
/// Sets `pinned_top = true` — the user is explicitly at the oldest event, away from
/// the newest (auto-scroll suppressed until the user navigates back to row 0).
///
/// No-op when the event list is empty.
pub fn jump_oldest(
    state: &mut EventRibbonState,
    events: &std::collections::VecDeque<HookEventRow>,
) {
    let event_count = events.len();
    if event_count == 0 {
        return;
    }
    let last_index = event_count.saturating_sub(1);
    state.list_state.select(Some(last_index));
    // BC-2.06.018 PC-5: pinned_top=true when user navigates away from newest (row 0).
    state.pinned_top = true;
}

/// Jump the Event Ribbon to the NEWEST event (row 0 = top of newest-first list).
///
/// `gg` in vi-style navigation jumps to the top of the list, which is the newest
/// event in the newest-first `VecDeque` (BC-2.06.018 PC-5: newest = front = row 0).
/// Clears `pinned_top = false` — the user has returned to the newest event, so
/// auto-scroll resumes (BC-2.06.018 AC-008).
///
/// Always selects row 0; no-op check on event count is not required (an empty ribbon
/// is a valid state — row 0 on an empty list is handled by ratatui ListState).
pub fn jump_newest(state: &mut EventRibbonState) {
    state.list_state.select(Some(0));
    // BC-2.06.018 AC-008 / PC-5: clear pinned_top when user returns to row 0 (newest).
    state.pinned_top = false;
}
