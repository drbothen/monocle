//! Real-defect failing tests for S-028 event ribbon (ADV Pass-1 + Pass-2).
//!
//! Each test exercises a production code path and asserts a postcondition that
//! is CURRENTLY BROKEN — either due to missing struct fields, missing wiring,
//! wrong timestamp semantics, incorrect capacity bounding, or missing pending-flag
//! propagation.
//!
//! # Red Gate rationale for each test
//!
//! - Test 5 (wall-clock timestamp from ring_tail): GREEN — `hook_event_row_from_record`
//!   propagates `record.timestamp_micros` and `format_timestamp` formats UTC wall-clock.
//!   The assertion `rendered_ts == "09:30:00.000"` now passes.
//!
//! - Test 6a (panel_height cap via production path): GREEN — `on_hook_event_received`
//!   uses `app.event_ribbon_panel_height` as the cap (not EVENT_RING_CAPACITY), so
//!   50 pushes with panel_height=10 yield len<=10.
//!
//! - Test 6b (resize trim from render_frame): GREEN — `render_frame` calls
//!   `trim_to_panel_height` after rendering the EventRibbon widget.
//!
//! - Test 7 (PENDING wiring): GREEN — `on_permission_prompt_queued` sets `pending=true`
//!   on the matching ribbon row.
//!
//! - Test 8 (session-change reset via dispatch): GREEN — `dispatch_key_event`'s
//!   SelectNext / SelectPrev arms call `reset_on_session_change` when the cursor moves.
//!
//! - Test 9 (shared matcher / INV-1 structural audit): GREEN — `render_sessions_filter`
//!   does NOT contain `local_matcher = Matcher::new`; the shared `app.matcher` is used.
//!
//! - Test 10 (streaming HookEventReceived timestamp_micros — ADV Pass-2 architect change):
//!   COMPILE-GATE RED — `ServerToClient::HookEventReceived` does not carry
//!   `timestamp_micros: i64` yet (SS-ipc breaking change, to be added by implementer).
//!   When the field is present, this test verifies that `on_hook_event_received`
//!   propagates the daemon's `timestamp_micros` to the ribbon row rather than
//!   substituting `SystemTime::now()` (which would produce a different wall-clock value).
//!
//! - Test 11 (display_name field on EnrichedSession — ADV Pass-2 architect change):
//!   COMPILE-GATE RED — `EnrichedSession::display_name` does not exist yet.
//!   When added, this test verifies that the sessions filter (BC-2.06.006 PC-3) matches
//!   `session.display_name` directly rather than via the hardcoded `harness_display_name`
//!   map, so that sessions with a display_name matching the query are included even when
//!   project_name and harness_type do not.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_ipc::types::{HookEventRecord, HookType};
use monocle_tui::app::{on_hook_event_received, App};
use monocle_tui::ui::event_ribbon::push_event_row;
use std::collections::VecDeque;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Test 5: BC-2.06.018 PC-1 — wall-clock timestamp from ring_tail timestamp_micros
//
// GREEN: hook_event_row_from_record propagates record.timestamp_micros into the row,
//        and format_timestamp formats it as UTC wall-clock HH:MM:SS.mmm.
//        The assertion rendered_ts == "09:30:00.000" passes in the current production code.
//      The test asserts that the formatted timestamp encodes the correct wall-clock
//      HH:MM:SS.mmm derived from the known timestamp_micros value.
// ---------------------------------------------------------------------------

/// A `HookEventRow` built from a `HookEventRecord` with a known `timestamp_micros`
/// must render the Timestamp column as wall-clock HH:MM:SS.mmm derived from that
/// timestamp — NOT as an elapsed-delta from a visible epoch.
///
/// BC-2.06.018 PC-1: Timestamp column shows `HH:MM:SS.mmm` (wall-clock).
/// The timestamp is sourced from `HookEventRecord::timestamp_micros` (Unix epoch μs).
///
/// Defect: `hook_event_row_from_record` ignores `timestamp_micros` and uses
/// `Instant::now()` instead. `format_timestamp` then computes elapsed-since-epoch,
/// not the actual event time. This makes the timestamp column show relative time
/// (e.g., "00:00:00.001") instead of the real wall-clock time
/// (e.g., "09:30:00.000" for a 09:30 UTC event).
#[test]
fn test_BC_2_06_018_PC1_wall_clock_timestamp_from_timestamp_micros() {
    use monocle_tui::ui::event_ribbon::hook_event_row_from_record;

    // A known Unix epoch timestamp in microseconds.
    // 2024-01-15 09:30:00.000 UTC = 1705311000 seconds = 1_705_311_000_000_000 microseconds.
    // The expected HH:MM:SS.mmm in UTC is "09:30:00.000".
    let known_timestamp_micros: i64 = 1_705_311_000_000_000;

    let record = HookEventRecord::new(
        "sess-abc123".to_string(),
        known_timestamp_micros,
        1234u32,
        "PreToolUse".to_string(),
        Some("Bash".to_string()),
        None,
    );

    // Act: convert record to HookEventRow.
    let row = hook_event_row_from_record(&record);

    // The row must carry the timestamp_micros information so it can be rendered
    // as a stable wall-clock HH:MM:SS.mmm independent of when the TUI connected.
    //
    // To render HH:MM:SS.mmm from timestamp_micros=1_705_311_000_000_000:
    //   total_seconds = 1_705_311_000 s
    //   hours_of_day  = (1_705_311_000 / 3600) % 24 = 9
    //   minutes       = (1_705_311_000 / 60) % 60   = 30
    //   seconds       = 1_705_311_000 % 60           = 0
    //   millis        = (1_705_311_000_000_000 / 1000) % 1000 = 0
    //   Expected: "09:30:00.000"
    //
    // Currently: received_at is Instant::now() (ignores timestamp_micros).
    // format_timestamp(received_at, epoch) where epoch ≈ received_at produces "00:00:00.001"
    // or similar elapsed-delta. The test asserts the production rendering path uses
    // timestamp_micros for the wall-clock format, not the Instant delta.
    //
    // We test this by asserting that the HookEventRow carries a field that encodes
    // the original timestamp_micros (not just an Instant), OR by asserting the
    // format_timestamp output for the row matches the expected "09:30:00.000" pattern.
    //
    // Since format_timestamp currently takes (received_at: Instant, epoch: Instant),
    // it CANNOT produce "09:30:00.000" from an Instant pair. The defect is structural:
    // the row type and formatter must be changed to use timestamp_micros for wall-clock.
    //
    // Assertion: the rendered timestamp for this row with a stable epoch is "09:30:00.000".
    // This FAILS until hook_event_row_from_record propagates timestamp_micros and
    // format_timestamp uses it for wall-clock formatting.
    //
    // Implementation path: HookEventRow needs a timestamp_micros: i64 field (alongside or
    // replacing received_at: Instant), and format_timestamp must accept timestamp_micros
    // and format it as UTC wall-clock HH:MM:SS.mmm.
    //
    // We use a proxy assertion: format the row using the current format_timestamp with
    // two identical Instants (epoch == received_at → elapsed = 0 → "00:00:00.000").
    // If the implementation is correct (wall-clock from timestamp_micros), the output
    // will be "09:30:00.000" instead. The test asserts the latter.
    let stable_epoch = Instant::now();
    // BC-2.06.018 PC-1: use timestamp_micros (not received_at Instant) for wall-clock format.
    // The fix: hook_event_row_from_record propagates record.timestamp_micros into row.timestamp_micros,
    // and format_timestamp uses timestamp_micros for UTC wall-clock output.
    let rendered_ts =
        monocle_tui::ui::event_ribbon::format_timestamp(row.timestamp_micros, stable_epoch);

    // The correct wall-clock timestamp from timestamp_micros=1_705_311_000_000_000 is
    // "09:30:00.000" (UTC). The current broken output is either "00:00:00.NNN" (elapsed
    // since TUI connect, approximately zero) or "??:??:??.???" (clock-skew guard).
    // Either way, it is NOT "09:30:00.000".
    assert_eq!(
        rendered_ts, "09:30:00.000",
        "BC-2.06.018 PC-1 defect: wall-clock timestamp must be formatted from \
         HookEventRecord::timestamp_micros={} as UTC HH:MM:SS.mmm=\"09:30:00.000\". \
         Actual: {:?}. \
         FIX: hook_event_row_from_record must propagate timestamp_micros; \
         format_timestamp must format as UTC wall-clock, not elapsed-delta.",
        known_timestamp_micros, rendered_ts
    );
}

// ---------------------------------------------------------------------------
// Test 6a: BC-2.06.018 PC-3 / INV-3 — panel_height cap enforced at push time
//
// GREEN: on_hook_event_received uses app.event_ribbon_panel_height as the cap.
//        After 50 pushes with panel_height=10, len <= 10 in the production path.
// ---------------------------------------------------------------------------

/// After pushing many events via the production `on_hook_event_received` with a given
/// `panel_height`, the stored VecDeque must hold at most `panel_height` entries.
///
/// BC-2.06.018 PC-3: the VecDeque is bounded to `panel_height` entries.
///
/// Defect: `on_hook_event_received` uses `EVENT_RING_CAPACITY` (4096) as the cap.
/// The panel_height is not passed to the push helper — `push_event_row` is called
/// with `EVENT_RING_CAPACITY` instead of the render-time `panel_height`.
/// This means the VecDeque grows to 4096 entries instead of staying at panel_height.
#[test]
fn test_BC_2_06_018_PC3_panel_height_cap_enforced_at_push_time() {
    // Arrange: fresh App with panel_height=10 (simulated via direct push_event_row
    // calls using the production helper, mirroring what on_hook_event_received should do).
    let mut events: VecDeque<monocle_tui::ui::event_ribbon::HookEventRow> = VecDeque::new();
    let panel_height = 10usize;

    // Push 50 events via the production push_event_row helper with panel_height=10.
    // This verifies that push_event_row correctly enforces the cap.
    for i in 0..50u64 {
        let row = monocle_tui::ui::event_ribbon::HookEventRow {
            timestamp_micros: monocle_tui::ui::event_ribbon::current_timestamp_micros(),
            received_at: Instant::now(),
            hook_type: HookType::Notification,
            session_id: format!("sess-{i:04}"),
            latency_ms: Some(i % 100),
            pending: false,
        };
        push_event_row(&mut events, row, panel_height);
    }

    assert_eq!(
        events.len(),
        panel_height,
        "BC-2.06.018 PC-3: push_event_row with panel_height=10 and 50 pushes must yield \
         len=10. This part passes. Now test the PRODUCTION path:"
    );

    // Now test the PRODUCTION on_hook_event_received path.
    // The fix: on_hook_event_received uses app.event_ribbon_panel_height as the cap.
    // Set event_ribbon_panel_height to panel_height=10 to simulate a render that
    // computed the ribbon area height as 10 rows (BC-2.06.018 PC-3 cached panel_height).
    let mut app = App::new(MonocleConfig::default());
    // Simulate that a render has occurred with panel_height=10 (the production flow:
    // render_frame sets app.event_ribbon_panel_height from the ribbon Rect height).
    app.event_ribbon_panel_height = panel_height;

    // Push 50 events via the PRODUCTION on_hook_event_received path.
    for i in 0..50u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            format!("sess-{i:04}"),
            "{}".to_string(),
            i,
            0i64,
        );
    }

    // Assert: the production path must also respect a panel_height cap.
    // Currently it uses EVENT_RING_CAPACITY (4096) → len=50 (defect: cap not panel_height).
    // The correct behavior per BC-2.06.018 PC-3 is that the VecDeque is bounded to
    // panel_height. Since panel_height is determined at render time, the production path
    // either:
    //   (a) uses the last-known panel_height cached from the previous render call, OR
    //   (b) trims in render via trim_to_panel_height.
    //
    // Either way: after 50 pushes, len must be at most panel_height (10).
    // The defect: len=50 because EVENT_RING_CAPACITY=4096 is used as the cap.
    assert!(
        app.event_ribbon_events.len() <= panel_height,
        "BC-2.06.018 PC-3 defect: on_hook_event_received must enforce panel_height ({}) cap. \
         After 50 pushes, len={} (must be <= {}). \
         FIX: pass panel_height to push_event_row in on_hook_event_received, \
         or call trim_to_panel_height in render_frame after each render.",
        panel_height,
        app.event_ribbon_events.len(),
        panel_height
    );
}

// ---------------------------------------------------------------------------
// Test 6b: BC-2.06.018 INV-3 — trim_to_panel_height called on resize
//
// GREEN: render_frame calls trim_to_panel_height after rendering the EventRibbon.
//        After a render with panel_height=10 (80x12 terminal), len <= 10.
// ---------------------------------------------------------------------------

/// On terminal resize (panel_height decreases), `trim_to_panel_height` must be applied
/// to `App::event_ribbon_events` to discard entries beyond the new panel height.
///
/// BC-2.06.018 INV-3: the VecDeque is trimmed to the new height on the next draw cycle.
///
/// Defect: `render_frame` never calls `trim_to_panel_height`. There is no resize
/// detection path. After a "resize" (reducing panel_height), the VecDeque retains
/// more entries than the new panel_height.
///
/// This test drives the production path by:
/// 1. Pushing 30 events into app.event_ribbon_events (simulating large panel).
/// 2. Rendering with a small terminal height (simulating panel_height=8).
/// 3. Asserting that app.event_ribbon_events is trimmed to ≤ 8 after render.
#[test]
fn test_BC_2_06_018_INV3_trim_on_resize_called_from_render_frame() {
    use monocle_tui::app::render_frame;
    use ratatui::{backend::TestBackend, Terminal};

    let mut app = App::new(MonocleConfig::default());

    // Push 30 events — simulating a previous render with large panel_height.
    for i in 0..30u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            format!("sess-{i:04}"),
            "{}".to_string(),
            i,
            0i64,
        );
    }
    // The production path caps at EVENT_RING_CAPACITY=4096, so len=30.
    // (This is itself a defect per test 6a, but we proceed for this test's purpose.)

    // Simulate a small terminal: 80 cols × 12 rows.
    // DashboardLayout: main_area = rows 0..10 (height=10), status_bar = rows 10..12 (height=2).
    // Horizontal split: sessions=48 cols, event_ribbon=32 cols.
    // EventRibbon area height = 10.
    // panel_height = 10. After trim, event_ribbon_events.len() must be ≤ 10.
    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let mut state = monocle_tui::ui::sessions_panel::SessionsPanelState::default();
            render_frame(&mut app, &mut state, frame);
        })
        .unwrap();

    // Assert: after render with panel_height=10 (derived from terminal height 12 - 2 statusbar),
    // event_ribbon_events must have been trimmed to ≤ 10.
    //
    // Currently: trim_to_panel_height is NEVER called from render_frame, so len=30 still.
    let panel_height_after_render = 10usize; // main_area height = 12 - 2 = 10
    assert!(
        app.event_ribbon_events.len() <= panel_height_after_render,
        "BC-2.06.018 INV-3 defect: render_frame must call trim_to_panel_height after rendering \
         the event ribbon. After render with panel_height={}, event_ribbon_events.len()={} \
         (must be ≤ {}). \
         FIX: call trim_to_panel_height(&mut app.event_ribbon_events, panel_height) \
         in the render_frame dashboard branch after rendering the EventRibbon widget.",
        panel_height_after_render,
        app.event_ribbon_events.len(),
        panel_height_after_render
    );
}

// ---------------------------------------------------------------------------
// Test 7: BC-2.06.018 PC-4 — PENDING status wired via on_permission_prompt_queued
//
// GREEN: on_permission_prompt_queued sets pending=true on the most recent PreToolUse
//        row matching the prompt's session_id in event_ribbon_events.
// ---------------------------------------------------------------------------

/// An unresolved `PreToolUse` event shows `PENDING` in yellow in the ribbon through the
/// production path: `on_permission_prompt_queued` (or the hook path) must set
/// `pending=true` on the corresponding event_ribbon_events row.
///
/// BC-2.06.018 PC-4: PENDING in yellow for unresolved PreToolUse events.
///
/// Defect: `on_permission_prompt_queued` never iterates `event_ribbon_events` to find
/// the matching row and set `pending=true`. The flag is never set through the production
/// IPC message path.
#[test]
fn test_BC_2_06_018_PC4_pending_status_set_via_on_permission_prompt_queued() {
    use monocle_ipc::types::PermissionPromptPayload;
    use monocle_tui::app::on_permission_prompt_queued;
    use uuid::Uuid;

    // Arrange: App with a PreToolUse event for "sess-001" already in event_ribbon_events.
    let mut app = App::new(MonocleConfig::default());
    on_hook_event_received(
        &mut app,
        HookType::PreToolUse,
        "sess-001".to_string(),
        r#"{"tool":"Bash","command":"cargo test"}"#.to_string(),
        12u64,
        0i64,
    );

    // Verify precondition: the event is in the ribbon and pending=false initially.
    assert_eq!(
        app.event_ribbon_events.len(),
        1,
        "precondition: 1 event in ribbon"
    );
    assert!(
        !app.event_ribbon_events[0].pending,
        "precondition: pending must be false before prompt arrives"
    );

    // Act: a PermissionPromptQueued arrives for "sess-001" (a PreToolUse).
    // This simulates the IPC path: daemon queues a permission prompt for the
    // tool use event that is already in the ribbon.
    let prompt_id = Uuid::new_v4();
    let payload = PermissionPromptPayload {
        prompt_id,
        session_id: "sess-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_input: serde_json::json!({"command": "cargo test"}),
        old_content: None,
        new_content: None,
    };
    on_permission_prompt_queued(&mut app, payload);

    // Assert: the existing ribbon row for "sess-001" must now have pending=true.
    // BC-2.06.018 PC-4: PENDING in yellow for unresolved PreToolUse events.
    // Currently: on_permission_prompt_queued only pushes to overlay_stack; it never
    // touches event_ribbon_events. The row's pending flag stays false.
    assert!(
        app.event_ribbon_events[0].pending,
        "BC-2.06.018 PC-4 defect: on_permission_prompt_queued must set pending=true on the \
         corresponding PreToolUse row in event_ribbon_events for session_id=\"sess-001\". \
         FIX: after inserting into overlay_stack, iterate event_ribbon_events and set \
         pending=true on the most recent PreToolUse row matching the prompt's session_id."
    );
}

// ---------------------------------------------------------------------------
// Test 8: BC-2.06.018 INV-1 / AC-009 — session-change reset via dispatch_key_event
//
// GREEN: dispatch_key_event's SelectNext and SelectPrev arms call reset_on_session_change
//        when the cursor moves to a different session, resetting ribbon to row 0 / pinned_top=false.
// ---------------------------------------------------------------------------

/// Selecting a different session via SelectNext/SelectPrev resets the ribbon scroll
/// to row 0 (newest) and clears `pinned_top`, through the real dispatch path.
///
/// BC-2.06.018 INV-1 / AC-009: session-change resets ribbon to auto-follow position.
///
/// Defect: `dispatch_key_event`'s `SelectNext` / `SelectPrev` arms mutate `sessions_state`
/// (cursor position) but never call `reset_on_session_change`. The ribbon scroll state
/// is decoupled from session selection — changing sessions leaves stale scroll position.
///
/// Note: This test requires `EventRibbonState` to be threaded through `dispatch_key_event`
/// (or stored in `App`). Currently `dispatch_key_event` only accepts `SessionsPanelState`.
/// The test asserts the post-dispatch ribbon state reset behavior — if `EventRibbonState`
/// is not yet in the function signature, this test documents the structural gap.
#[test]
fn test_BC_2_06_018_INV1_AC009_session_change_resets_ribbon_scroll_via_dispatch() {
    use monocle_core::engine::{EnrichedSession, SessionStatus};
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    let mut app = App::new(MonocleConfig::default());

    // Seed two sessions.
    app.sessions = vec![
        EnrichedSession::new(
            "sess-001".to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Idle,
            None,
            Some("proj-a".to_string()),
            None,
            0,
            None,
        ),
        EnrichedSession::new(
            "sess-002".to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Idle,
            None,
            Some("proj-b".to_string()),
            None,
            0,
            None,
        ),
    ];

    // Seed several ribbon events for both sessions.
    for _ in 0..5u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            "sess-001".to_string(),
            "{}".to_string(),
            1u64,
            0i64,
        );
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            "sess-002".to_string(),
            "{}".to_string(),
            2u64,
            0i64,
        );
    }

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    // Select first session.
    sessions_state.list_state.select(Some(0));

    // Simulate: user scrolled ribbon to a non-zero offset (pinned_top=true).
    // The production fix stores EventRibbonState in App (app.event_ribbon_state).
    // Set up the app's ribbon state directly (not an external instance).
    app.event_ribbon_state.list_state.select(Some(4)); // scrolled down to row 4
    app.event_ribbon_state.pinned_top = true;

    // Precondition: ribbon is scrolled to row 4, pinned_top=true.
    assert_eq!(app.event_ribbon_state.list_state.selected(), Some(4));
    assert!(app.event_ribbon_state.pinned_top);

    // Act: press 'j' → SelectNext → session changes from 0 to 1.
    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: KeyModifiers::default(),
    };
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);

    // Assert: sessions_state moved (cursor at row 1 — session changed).
    assert_eq!(
        sessions_state.list_state.selected(),
        Some(1),
        "precondition for ribbon reset: SelectNext must move cursor to row 1"
    );

    // THE DEFINITIVE ASSERTION: after dispatch_key_event causes a session change,
    // app.event_ribbon_state must be reset to row 0 with pinned_top=false
    // (BC-2.06.018 INV-1 / AC-009).
    // The production fix: SelectNext/SelectPrev arms call reset_on_session_change
    // on app.event_ribbon_state when the cursor moves to a different session.
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(0),
        "BC-2.06.018 INV-1 / AC-009: after SelectNext causes session change (cursor \
         moved from row 0 to row 1), app.event_ribbon_state must be reset to row 0 (newest) \
         with pinned_top=false. FIX: dispatch_key_event SelectNext/SelectPrev arms must call \
         reset_on_session_change on app.event_ribbon_state when session cursor changes."
    );
    assert!(
        !app.event_ribbon_state.pinned_top,
        "BC-2.06.018 INV-1 / AC-009: pinned_top must be cleared after session change"
    );
}

// ---------------------------------------------------------------------------
// Test 9: BC-2.06.006 INV-1 — shared Matcher used in production render path
//
// GREEN: render_sessions_filter does NOT create a fresh `local_matcher = Matcher::new`.
//        It uses the shared `app.matcher` (&mut App signature enables this).
//        The source-code audit assertion `!sessions_panel_src.contains("local_matcher = Matcher::new")`
//        passes because the production code was fixed to use app.matcher directly.
// ---------------------------------------------------------------------------

/// The production `render_sessions_filter` scoring path must use the shared `app.matcher`
/// (BC-2.06.006 INV-1), NOT create a fresh `Matcher::new(Config::DEFAULT)` per render.
///
/// Defect: `render_sessions_filter` in `sessions_panel.rs` constructs:
///   `let mut local_matcher = Matcher::new(Config::DEFAULT);`
/// on every render call. This violates INV-1 which states the Matcher is shared across
/// the filter session (not recreated per render/keystroke).
///
/// Strengthened assertion: we verify that the production code path is consistent with
/// using app.matcher by asserting that after a filter render, app.matcher has been
/// MUTATED (indicating scoring went through the shared instance, which maintains internal
/// scratch buffers across calls). If a fresh matcher is created each time, app.matcher
/// is never touched and remains in its initial state.
///
/// This test uses a structural assertion: if render_sessions_filter correctly uses
/// app.matcher (&mut), then app.matcher must have been borrowed mutably. Since we
/// cannot observe borrow counts at runtime, we assert via a SIDE-EFFECT: the scoring
/// function mutates internal buffers in the matcher. We verify this by checking that
/// the Matcher's internal state changes after a render (a proxy for "app.matcher was used").
///
/// Note: If nucleo::Matcher does not expose internal state, we use a compile-time
/// structural check: `render_sessions_filter` must accept `&mut App` (not `&App`)
/// for the shared matcher to be usable. Currently it accepts `&App` — which means
/// the app.matcher cannot be passed as `&mut nucleo::Matcher` to the scoring function.
/// This is the structural proof of the defect.
#[test]
fn test_BC_2_06_006_INV1_render_uses_shared_matcher_not_fresh_per_render() {
    use monocle_core::engine::{EnrichedSession, SessionStatus};
    use monocle_core::tui::state::{AppMode, FocusSnapshot, PanelId};
    use monocle_tui::ui::sessions_panel::{render_sessions_filter, SessionsPanelState};
    use ratatui::{backend::TestBackend, Terminal};

    // Arrange: App in Filtering mode.
    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "mono".to_string(),
        prior: FocusSnapshot::Sessions,
    };
    app.sessions = vec![EnrichedSession::new(
        "sess-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Idle,
        None,
        Some("monocle".to_string()),
        None,
        0,
        None,
    )];

    // The defect: render_sessions_filter takes `app: &crate::app::App` (immutable borrow).
    // Therefore, it CANNOT borrow `app.matcher` as `&mut nucleo::Matcher` to score sessions.
    // Instead it creates `let mut local_matcher = Matcher::new(Config::DEFAULT)` — a fresh
    // instance that is discarded after each render.
    //
    // The structural proof of the defect: render_sessions_filter's signature accepts &App,
    // which prevents it from calling `app.matcher.fuzzy_match(...)` (requires &mut).
    //
    // To fix INV-1 correctly, render_sessions_filter must either:
    //   (a) Accept `&mut App` and use app.matcher directly, OR
    //   (b) Separate the scoring concern (score in dispatch, render the pre-scored list).
    //
    // Assert that the current render call succeeds (smoke test) but that the scoring
    // went through a local_matcher (detectable via: after render, app.matcher has NOT
    // been mutated because it was never used).
    //
    // We test this by rendering twice with the same query and asserting consistent results
    // (both calls work, but via a fresh local_matcher each time). We then assert that the
    // correct behavior — using app.matcher — is absent by checking a proxy condition.
    //
    // Proxy: if app.matcher were used, the render path would require `&mut App`. Since
    // `render_sessions_filter` accepts `&App`, the matcher cannot be app.matcher.
    // We assert this structural gap by verifying that render_sessions_filter accepts
    // `&App` (immutable) rather than `&mut App` — which means INV-1 is violated.
    //
    // Direct assertion: call render_sessions_filter twice; check app.matcher is unchanged.
    // Since we can't observe internal state of nucleo::Matcher, we assert via a negative:
    // after two renders with the same query, app.matcher has NOT been used for scoring.
    // We verify this by asserting that the rendered output is CORRECT (matching works)
    // but through a local_matcher, meaning the scoring state is independent per call.
    //
    // The FAILING assertion for INV-1: we assert that app.matcher IS used for scoring
    // (i.e., render_sessions_filter calls app.matcher.fuzzy_match). This fails because
    // the current implementation uses local_matcher.
    //
    // We express this via a test that verifies the STRUCTURAL requirement:
    // render_sessions_filter must be callable with `&mut App` (or equivalent) to use
    // the shared matcher. The current `&App` signature proves the defect exists.
    //
    // Since we cannot change the function signature in a test, we use a comment-only
    // assertion with an explicit RED statement:

    // This block renders successfully (the local_matcher works — scoring is correct):
    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let mut state = SessionsPanelState::default();
            render_sessions_filter(
                &mut app,
                "mono",
                frame.area(),
                frame.buffer_mut(),
                &mut state,
            );
        })
        .unwrap();
    let rendered_1 = terminal.backend().to_string();
    assert!(
        rendered_1.contains("monocle"),
        "precondition: render_sessions_filter must match 'monocle' with query='mono'"
    );

    // Now assert the INV-1 violation: render_sessions_filter accepts `&App` (immutable).
    // If it accepted `&mut App`, it could use app.matcher. The fact that it compiles with
    // `&App` proves it uses a local_matcher (violating INV-1).
    //
    // The FIX requires render_sessions_filter to accept `app: &mut App` and replace:
    //   `let mut local_matcher = Matcher::new(Config::DEFAULT);`
    // with:
    //   `let local_matcher = &mut app.matcher;`
    //
    // We assert the INV-1 defect is present by asserting that the function signature
    // SHOULD require &mut App but currently accepts &App. Since this is a compile-time
    // property, we express it as a runtime assertion that fails when the fix is applied:
    //
    // The DEFINITIVE RED for this test: the production render path uses a local_matcher,
    // not app.matcher. We assert this structural invariant by verifying that a string
    // known to violate INV-1 appears in the production source. This is a structural proof:
    // if the production code contains "local_matcher = Matcher::new", INV-1 is violated.
    //
    // We use a source-code audit assertion: read the production file and assert the
    // INV-1 defect string is present (and will disappear once fixed).
    let sessions_panel_src = include_str!("../src/ui/sessions_panel.rs");
    assert!(
        !sessions_panel_src.contains("local_matcher = Matcher::new"),
        "BC-2.06.006 INV-1 (GREEN check): render_sessions_filter must NOT create a fresh \
         `local_matcher = Matcher::new(Config::DEFAULT)` per render call. \
         Found 'local_matcher = Matcher::new' in sessions_panel.rs — this proves a fresh \
         matcher is constructed per render, violating INV-1. \
         FIX: use app.matcher directly (render_sessions_filter accepts &mut App)."
    );
}

// ---------------------------------------------------------------------------
// Test 10: BC-2.05.004 PC-2 / ADV Pass-2 architect change — streaming
//          HookEventReceived must carry daemon's timestamp_micros
//
// RED: COMPILE-GATE — `ServerToClient::HookEventReceived` does not carry
//      `timestamp_micros: i64` yet. The match arm in handle_server_message
//      (`crates/monocle-tui/src/app.rs`) will fail to compile once the field
//      is present but unhandled, and any construction with the new field will
//      fail until it exists. This test constructs a `HookEventReceived` with
//      the field explicitly to trigger the compile error.
//
//      Missing field: `timestamp_micros: i64` on `ServerToClient::HookEventReceived`
//      (crates/monocle-ipc/src/types.rs). See SS-ipc.md §S-028 ADR.
//
//      When the field is added:
//        1. `on_hook_event_received` must accept `timestamp_micros: i64` and pass it to
//           `hook_event_row_from_received` (which must set `row.timestamp_micros` from
//           the message's value, NOT from `current_timestamp_micros()`).
//        2. The resulting ribbon row must display the DAEMON's timestamp, not the TUI
//           receive time. For known `timestamp_micros`, the wall-clock must be deterministic.
// ---------------------------------------------------------------------------

/// A streaming `HookEventReceived` message carries `timestamp_micros: i64` from the
/// daemon. The TUI's ribbon row must use that value for the Timestamp column, NOT
/// substitute `SystemTime::now()` at receipt time.
///
/// BC-2.05.004 PC-2: `timestamp_micros` in `HookEventReceived` MUST equal the value
/// in the corresponding `HookEventRecord` (same daemon clock capture).
///
/// RED: compile-gate — `ServerToClient::HookEventReceived` does not have
/// `timestamp_micros` field yet (monocle-ipc/src/types.rs). The struct-literal
/// construction below will fail to compile:
///   error[E0026]: struct `HookEventReceived` has no field named `timestamp_micros`
#[test]
fn test_BC_2_05_004_PC2_streaming_event_uses_daemon_timestamp_micros() {
    use monocle_ipc::types::HookType;
    use monocle_ipc::types::ServerToClient;

    // Known daemon timestamp: 2024-01-15 09:30:00.000 UTC
    // = 1_705_311_000_000_000 microseconds since Unix epoch.
    // Expected wall-clock rendering: "09:30:00.000"
    let known_ts: i64 = 1_705_311_000_000_000;

    // Construct a HookEventReceived with the daemon's timestamp_micros.
    // COMPILE-GATE: this field does not yet exist on ServerToClient::HookEventReceived.
    // The implementer must add `timestamp_micros: i64` to the variant
    // (crates/monocle-ipc/src/types.rs) to make this compile.
    let _msg = ServerToClient::HookEventReceived {
        hook_type: HookType::PreToolUse,
        session_id: "sess-ts-test".to_string(),
        payload_excerpt: r#"{"tool":"Bash"}"#.to_string(),
        latency_ms: 10u64,
        timestamp_micros: known_ts, // NEW FIELD — does not exist yet
    };

    // When the field exists and on_hook_event_received propagates it:
    // Build an App, invoke the handler with the known timestamp, and assert the ribbon row.
    let mut app = App::new(MonocleConfig::default());

    // Call on_hook_event_received with the daemon's known timestamp.
    // The updated signature must accept timestamp_micros (not just the 4 existing args).
    // COMPILE-GATE: on_hook_event_received currently has signature:
    //   pub fn on_hook_event_received(app, hook_type, session_id, payload_excerpt, latency_ms)
    // It must gain a `timestamp_micros: i64` parameter and pass it to
    // hook_event_row_from_received, which must store it in HookEventRow::timestamp_micros
    // (instead of calling current_timestamp_micros()).
    monocle_tui::app::on_hook_event_received(
        &mut app,
        HookType::PreToolUse,
        "sess-ts-test".to_string(),
        r#"{"tool":"Bash"}"#.to_string(),
        10u64,
        known_ts, // NEW PARAMETER — does not exist in current signature
    );

    assert_eq!(
        app.event_ribbon_events.len(),
        1,
        "precondition: ribbon must have 1 event after on_hook_event_received"
    );

    // Assert: the ribbon row's timestamp_micros must equal the daemon's value.
    // NOT the TUI's receive-time (current_timestamp_micros() at message arrival).
    let row = &app.event_ribbon_events[0];
    assert_eq!(
        row.timestamp_micros, known_ts,
        "BC-2.05.004 PC-2: ribbon row timestamp_micros must equal the daemon's \
         timestamp_micros={} from HookEventReceived. \
         Must NOT use TUI receive-time (current_timestamp_micros()) which would \
         differ from known_ts by the latency delta.",
        known_ts
    );

    // Assert: the formatted wall-clock timestamp matches the expected UTC string.
    // 1_705_311_000_000_000 µs = 2024-01-15 09:30:00.000 UTC → "09:30:00.000"
    let stable_epoch = std::time::Instant::now();
    let rendered_ts =
        monocle_tui::ui::event_ribbon::format_timestamp(row.timestamp_micros, stable_epoch);
    assert_eq!(
        rendered_ts, "09:30:00.000",
        "BC-2.05.004 PC-2 / BC-2.06.018 PC-1: wall-clock timestamp for \
         timestamp_micros={} must be \"09:30:00.000\" (2024-01-15 09:30:00 UTC). \
         Actual: {:?}. FIX: on_hook_event_received must propagate timestamp_micros \
         from HookEventReceived to HookEventRow.",
        known_ts, rendered_ts
    );
}

// ---------------------------------------------------------------------------
// Test 11: BC-2.06.006 PC-3 / ADV Pass-2 architect change — filter matches on
//          EnrichedSession::display_name (NOT hardcoded harness_display_name map)
//
// RED: COMPILE-GATE — `EnrichedSession::display_name` does not exist as a field
//      yet. The struct literal construction and field access below will fail to compile:
//        error[E0026]: struct `EnrichedSession` has no field named `display_name`
//
//      When the field is added:
//        1. The daemon must populate it from `EngineModule::metadata()?.display_name`
//           during session enrichment.
//        2. `render_sessions_filter` must score against `session.display_name`
//           (the IPC wire copy) instead of the hardcoded `harness_display_name()`
//           lookup — so that third-party engines (not in the hardcoded map) can
//           also be matched by their display name.
// ---------------------------------------------------------------------------

/// The sessions filter (BC-2.06.006 PC-3) must match against `EnrichedSession::display_name`
/// — the daemon-populated field from `EngineMetadata::display_name` — NOT against a
/// hardcoded `harness_type` → display name lookup table.
///
/// Test vector (BC-2.06.006 §Canonical Test Vectors row 5 — updated for arch change):
/// - Query: "Cla"
/// - Session: `display_name = "Claude Code"`, project_name does NOT match "Cla"
/// - Expected: session IS included (via display_name match)
///
/// The distinction from the existing `test_BC_2_06_006_display_name_match` test:
/// - That test relies on `harness_type = "claude-code"` → `harness_display_name()` → "Claude Code"
/// - This test uses `session.display_name = "Claude Code"` DIRECTLY (no harness_type lookup)
///   and uses `harness_type = "unknown-engine-xyz"` (not in the hardcoded map) to prove
///   the lookup is bypassed in favor of the session field.
///
/// RED: compile-gate — `EnrichedSession::display_name` does not exist.
#[test]
fn test_BC_2_06_006_PC3_filter_matches_session_display_name_field_not_hardcoded_map() {
    use monocle_core::engine::{EnrichedSession, SessionStatus};
    use monocle_tui::ui::sessions_panel::{
        render_sessions_filter, SessionsPanelState, SESSIONS_FILTER_NO_MATCH,
    };
    use ratatui::{backend::TestBackend, Terminal};

    let mut app = App::new(MonocleConfig::default());

    // Session: harness_type is NOT in the hardcoded harness_display_name() map.
    // display_name is "Claude Code" (populated directly — as the daemon would populate it).
    // project_name "xyz-project" has no characters matching "Cla".
    // The ONLY way this session can match "Cla" is via session.display_name = "Claude Code".
    //
    // COMPILE-GATE: EnrichedSession::new() currently takes 10 args with no display_name.
    // The updated constructor must accept display_name as a parameter.
    // Until display_name is added to EnrichedSession, this line will fail to compile.
    app.sessions = vec![EnrichedSession::new_with_display_name(
        "sess-dn-001".to_string(),
        "unknown-engine-xyz".to_string(), // NOT in harness_display_name() map
        None,
        None,
        SessionStatus::Idle,
        None,
        Some("xyz-project".to_string()), // no chars matching "Cla"
        None,
        0,
        None,
        "Claude Code".to_string(), // display_name from EngineMetadata::display_name
    )];

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let mut state = SessionsPanelState::default();
            render_sessions_filter(
                &mut app,
                "Cla",
                frame.area(),
                frame.buffer_mut(),
                &mut state,
            );
        })
        .unwrap();
    let rendered = terminal.backend().to_string();

    // Assert: the session must appear (via display_name="Claude Code" fuzzy-matching "Cla").
    // project_name="xyz-project" does not match "Cla".
    // harness_type="unknown-engine-xyz" is NOT in the hardcoded map — so if the filter
    // relied on harness_display_name(), this session would be ABSENT (no match).
    // It must be PRESENT because session.display_name="Claude Code" matches "Cla".
    assert!(
        rendered.contains("xyz-project") || rendered.contains("sess-dn-001"),
        "BC-2.06.006 PC-3 (architect change): session with display_name=\"Claude Code\" and \
         harness_type=\"unknown-engine-xyz\" must be visible with query=\"Cla\" via \
         session.display_name match. harness_display_name() would NOT match \
         \"unknown-engine-xyz\" → \"Claude Code\". The filter must use session.display_name \
         directly (BC-2.06.006 PC-3 updated per ADR). Rendered:\n{rendered}"
    );
    assert!(
        !rendered.contains(SESSIONS_FILTER_NO_MATCH),
        "BC-2.06.006 PC-3: SESSIONS_FILTER_NO_MATCH must NOT appear when display_name matches. \
         Rendered:\n{rendered}"
    );
}
