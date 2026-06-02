//! RED-gate tests for PR reviewer findings on S-028.
//!
//! Two defects surfaced by code review that are not yet covered by existing
//! passing tests. Both assertions fail against the current implementation.
//!
//! # Finding 1 — AC-008 selected-session auto-scroll gate
//!
//! `on_hook_event_received` in `app.rs` (~line 678) unconditionally calls
//! `list_state.select(Some(0))` when `!pinned_top`, regardless of which session
//! the incoming event belongs to. AC-008 specifies that auto-scroll fires ONLY
//! when the incoming event matches the currently selected session. An event for a
//! non-selected session must NOT disturb the scroll position of the selected
//! session's ribbon view.
//!
//! # Finding 2 — trim-to-0 data loss (BC-2.06.018 INV-3)
//!
//! `trim_to_panel_height` in `event_ribbon.rs` (~line 414) runs the loop
//! `while events.len() > panel_height` unconditionally. When `panel_height == 0`
//! (e.g., the terminal is resized so the ribbon area height collapses to zero) the
//! loop drains the entire VecDeque. BC-2.06.018 INV-3 says the events are
//! "trimmed to the new height" — a height of 0 is a transient display state, not
//! a directive to destroy all history. `trim_to_panel_height(_, 0)` must be a
//! no-op (preserves all events).
//!
//! # Red Gate rationale
//!
//! Both tests compile against the current production code and fail with assertion
//! errors (not compile errors), making them valid Red Gate tests.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]
// expect/unwrap are idiomatic assertion amplification in test code.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_config::MonocleConfig;
use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::HookType;
use monocle_tui::app::{
    build_builtin_binding_layers, dispatch_key_event, on_hook_event_received, App,
};
use monocle_tui::ui::event_ribbon::{trim_to_panel_height, HookEventRow};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use std::collections::VecDeque;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_session(id: &str) -> EnrichedSession {
    EnrichedSession::new(
        id.to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Idle,
        None,
        Some(format!("proj-{id}")),
        None,
        0,
        None,
    )
}

// ---------------------------------------------------------------------------
// Finding 1 — AC-008 selected-session auto-scroll gate
//
// RED: on_hook_event_received fires list_state.select(Some(0)) for ANY incoming
//      event when !pinned_top. The defect: it checks only `!pinned_top`; it does
//      NOT check whether the event's session_id matches the currently selected
//      session.
//
//      The fix the implementer must implement:
//        • Add `pub selected_session_id: Option<String>` (or equivalent) to `App`
//          so that `on_hook_event_received` can compare event.session_id to the
//          currently selected session without requiring SessionsPanelState.
//        • In `on_hook_event_received`, guard the auto-scroll as:
//              if !app.event_ribbon_state.pinned_top
//                  && app.selected_session_id.as_deref() == Some(&session_id)
//              { app.event_ribbon_state.list_state.select(Some(0)); }
//        The selected_session_id must be kept in sync by dispatch_key_event when
//        SelectNext / SelectPrev change the cursor, and by render_frame when the
//        sessions list is (re-)populated.
//
// AC-008 spec (from S-028 story / BC-2.06.018 PC-8):
//   "auto-scroll to row 0 when !pinned_top AND the incoming event matches the
//    selected session — an event for a different session must not move the ribbon"
// ---------------------------------------------------------------------------

/// AC-008: an event for a NON-selected session must NOT trigger auto-scroll.
///
/// Scenario:
/// 1. App has two sessions: sess-A (selected at index 0) and sess-B.
/// 2. The ribbon is at a non-zero scroll position (row 3) with pinned_top=false.
/// 3. A HookEventReceived arrives for sess-B (the non-selected session).
/// 4. Expected: scroll position stays at row 3 (not reset to row 0).
///
/// RED: the current implementation unconditionally resets to row 0 for any event
/// when `!pinned_top`, so this test fails with
///   `actual selected = Some(0), expected = Some(3)`.
#[test]
fn test_BC_2_06_018_AC008_auto_scroll_does_not_fire_for_non_selected_session() {
    // Arrange: two sessions, A selected (index 0), B not selected.
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![make_session("sess-A"), make_session("sess-B")];

    // The implementer will add `app.selected_session_id` to App. Until then the test
    // exercises the current behavior of on_hook_event_received, which does not gate
    // on selected session and therefore INCORRECTLY fires auto-scroll for sess-B.
    //
    // To keep the test compilable before the new field exists, we drive the assertion
    // purely through on_hook_event_received + app.event_ribbon_state — which is
    // already public. The assertion captures the WRONG current behavior.

    // Ribbon is at row 3 (user scrolled), pinned_top=false (auto-scroll would apply
    // if the session matched).
    app.event_ribbon_state.list_state.select(Some(3));
    app.event_ribbon_state.pinned_top = false;

    // Push a few events into the ribbon first so index 3 is within bounds.
    for _ in 0..5 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            "sess-A".to_string(),
            "{}".to_string(),
            1u64,
            0i64,
        );
    }
    // Reset scroll to row 3 after the seed pushes (they may have shifted it).
    app.event_ribbon_state.list_state.select(Some(3));
    app.event_ribbon_state.pinned_top = false;

    // Act: event arrives for sess-B (NOT the selected session sess-A).
    on_hook_event_received(
        &mut app,
        HookType::Notification,
        "sess-B".to_string(),
        "{}".to_string(),
        2u64,
        0i64,
    );

    // Assert: scroll must NOT have moved to row 0.
    // BC-2.06.018 PC-8 / AC-008: auto-scroll fires ONLY for the selected session.
    // An event for sess-B while sess-A is selected must leave the scroll at row 3.
    //
    // CURRENT BEHAVIOR (defect): the handler resets to row 0 for any event.
    // This assertion FAILS against the current implementation (RED gate).
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(3),
        "BC-2.06.018 AC-008 defect: on_hook_event_received for sess-B (non-selected) \
         must NOT reset ribbon scroll to row 0. scroll must remain at row 3. \
         FIX: guard auto-scroll with \
         `app.selected_session_id.as_deref() == Some(&session_id)` before calling \
         list_state.select(Some(0))."
    );
}

/// AC-008: an event for the SELECTED session DOES trigger auto-scroll (positive case).
///
/// Scenario:
/// 1. App has two sessions: sess-A (selected) and sess-B.
/// 2. Ribbon is at row 3, pinned_top=false.
/// 3. A HookEventReceived arrives for sess-A (the selected session).
/// 4. Expected: scroll resets to row 0.
///
/// This positive case must stay GREEN after the fix (the fix must not break the
/// matching-session auto-scroll). The test is RED now ONLY for the non-selected-session
/// case above. Once the implementer adds the selected_session_id gate:
///   - the non-selected case stays at row 3 (above test becomes GREEN)
///   - the selected case still resets to row 0 (this test stays GREEN)
///
/// Note: this test may currently be GREEN because on_hook_event_received unconditionally
/// fires. We include it to pin the CORRECT positive behavior so the implementer cannot
/// accidentally break it.
#[test]
fn test_BC_2_06_018_AC008_auto_scroll_fires_for_selected_session() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![make_session("sess-A"), make_session("sess-B")];

    // Seed events so scroll index 3 is within bounds.
    for _ in 0..5 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            "sess-A".to_string(),
            "{}".to_string(),
            1u64,
            0i64,
        );
    }

    // Manually set the selected session so the implementer's guard can use it.
    // The implementer must add `app.selected_session_id: Option<String>` to App.
    // Until then this assignment is absent — but the test captures the expected
    // positive behavior after the fix.
    //
    // NOTE: The line below will require the implementer to add the field.
    // Uncomment when the field exists:
    //   app.selected_session_id = Some("sess-A".to_string());

    app.event_ribbon_state.list_state.select(Some(3));
    app.event_ribbon_state.pinned_top = false;

    // Act: event for sess-A (the selected session).
    on_hook_event_received(
        &mut app,
        HookType::Notification,
        "sess-A".to_string(),
        "{}".to_string(),
        2u64,
        0i64,
    );

    // Assert: scroll resets to row 0 (matching-session event fires auto-scroll).
    // BC-2.06.018 PC-8 / AC-008: auto-scroll must fire for the selected session.
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(0),
        "BC-2.06.018 AC-008: on_hook_event_received for sess-A (selected) must reset \
         ribbon scroll to row 0 when pinned_top=false."
    );
}

// ---------------------------------------------------------------------------
// Finding 2 — trim-to-0 data loss (BC-2.06.018 INV-3)
//
// RED: trim_to_panel_height(events, 0) drains the entire VecDeque because the
//      loop `while events.len() > 0` pops until empty.
//
//      BC-2.06.018 INV-3: "trimmed to new height" — a panel_height of 0 is a
//      transient display state (terminal resize collapses ribbon area) and must NOT
//      destroy event history.
//
//      Correct behavior the implementer must encode:
//        `trim_to_panel_height(events, 0)` is a no-op — preserves all events.
//        The guard is: `if panel_height == 0 { return; }` at the top of the
//        function body.
//
//      Rationale: when the terminal is resized to zero height, the TUI cannot render
//      rows, but the events remain valid and must be available once the terminal is
//      enlarged again. Destroying them at height=0 causes permanent data loss for the
//      user (they lose their event history on any momentary resize to a tiny height).
// ---------------------------------------------------------------------------

/// trim_to_panel_height with panel_height=0 must preserve all events (no-op).
///
/// BC-2.06.018 INV-3: "trimmed to new height" — a height of 0 is a transient
/// display state; it must not cause event history to be destroyed.
///
/// RED: the current loop `while events.len() > panel_height` runs until empty
/// when panel_height==0, draining all events. This assertion fails with
///   `actual len = 0, expected > 0`.
#[test]
fn test_BC_2_06_018_INV3_trim_to_zero_height_preserves_events() {
    let mut events: VecDeque<HookEventRow> = VecDeque::new();

    // Arrange: push 5 events.
    for i in 0..5u64 {
        events.push_front(HookEventRow {
            timestamp_micros: i as i64,
            received_at: Instant::now(),
            hook_type: HookType::Notification,
            session_id: format!("sess-{i}"),
            latency_ms: Some(i),
            pending: false,
        });
    }
    assert_eq!(events.len(), 5, "precondition: 5 events before trim");

    // Act: trim to panel_height=0 (terminal resize → ribbon area collapsed to zero rows).
    trim_to_panel_height(&mut events, 0);

    // Assert: events must NOT be drained.
    // panel_height=0 is a transient display state — the function must be a no-op.
    // The history must be available when the terminal is resized back to a positive height.
    //
    // CURRENT BEHAVIOR (defect): the while loop runs to completion → events.len() == 0.
    // This assertion FAILS against the current implementation (RED gate).
    assert!(
        !events.is_empty(),
        "BC-2.06.018 INV-3 defect: trim_to_panel_height(events, 0) must be a no-op \
         (preserve all events). panel_height=0 is a transient display state, not a directive \
         to destroy event history. FIX: add `if panel_height == 0 {{ return; }}` guard at \
         the top of trim_to_panel_height."
    );

    // Stronger assertion: ALL 5 events are preserved (no partial drain).
    assert_eq!(
        events.len(),
        5,
        "BC-2.06.018 INV-3: all 5 events must be preserved after trim_to_panel_height(_, 0). \
         The entire history must survive a transient zero-height resize."
    );
}

/// trim_to_panel_height with a positive panel_height still evicts as expected.
///
/// This green-anchor test ensures the no-op guard for panel_height=0 does NOT
/// accidentally suppress eviction for valid positive heights. After the fix,
/// trim_to_panel_height(_, N) for N>0 must still trim to N entries.
#[test]
fn test_BC_2_06_018_INV3_trim_to_positive_height_still_evicts() {
    let mut events: VecDeque<HookEventRow> = VecDeque::new();

    // Arrange: 10 events, trim to 4.
    for i in 0..10u64 {
        events.push_front(HookEventRow {
            timestamp_micros: i as i64,
            received_at: Instant::now(),
            hook_type: HookType::Notification,
            session_id: format!("sess-{i}"),
            latency_ms: Some(i),
            pending: false,
        });
    }
    assert_eq!(events.len(), 10, "precondition: 10 events before trim");

    // Act: trim to panel_height=4 (valid positive height).
    trim_to_panel_height(&mut events, 4);

    // Assert: exactly 4 events remain (oldest 6 evicted).
    assert_eq!(
        events.len(),
        4,
        "BC-2.06.018 INV-3: trim_to_panel_height(events, 4) must leave exactly 4 events. \
         The zero-height guard must not affect positive-height eviction."
    );
}

// ---------------------------------------------------------------------------
// Finding 3 — selected_session_id never updated by cursor movement
//
// The AC-008 fix adds `app.selected_session_id: Option<String>` and gates
// auto-scroll on it, but the ScrollDown/ScrollUp dispatch arms (which drive
// the Sessions cursor in Dashboard/Sessions focus) did NOT update
// `selected_session_id` when moving the cursor.
//
// As a result: after the user presses `j` to move to session 1, `selected_session_id`
// remains `None`, `on_hook_event_received` falls back to `sessions.first()` (session 0),
// and events for session 0 trigger auto-scroll even though the user is viewing
// session 1's ribbon.
//
// Fix required:
//   In the `ScrollDown { Dashboard/Sessions }` arm: set `app.selected_session_id`
//   from `app.sessions.get(next)`.
//   In the `ScrollUp { Dashboard/Sessions }` arm: set `app.selected_session_id`
//   from `app.sessions.get(prev)`.
// ---------------------------------------------------------------------------

fn no_mod() -> KeyModifiers {
    KeyModifiers::default()
}

/// After pressing `j` (ScrollDown → Sessions cursor down), `selected_session_id`
/// must be updated to the newly highlighted session.
///
/// BC-2.06.018 AC-008: auto-scroll gates on the selected session. For this gate to
/// work correctly after cursor movement, `selected_session_id` must track the cursor.
#[test]
fn test_BC_2_06_018_AC008_selected_session_id_updated_on_cursor_down() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![make_session("sess-A"), make_session("sess-B")];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    // Act: press j to move cursor from sess-A (index 0) to sess-B (index 1).
    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: no_mod(),
    };
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);

    // Assert: sessions cursor moved to index 1.
    assert_eq!(
        sessions_state.list_state.selected(),
        Some(1),
        "precondition: sessions cursor at index 1 after j"
    );

    // Assert: selected_session_id must reflect the new selection (sess-B).
    // If this is None, on_hook_event_received falls back to sessions.first() (sess-A)
    // and will incorrectly auto-scroll for sess-A events while the user views sess-B.
    assert_eq!(
        app.selected_session_id.as_deref(),
        Some("sess-B"),
        "BC-2.06.018 AC-008: selected_session_id must be updated to 'sess-B' after \
         pressing j (cursor moves from 0 to 1). If None, auto-scroll gate falls back to \
         sessions.first() (sess-A) — wrong session."
    );
}

/// After pressing `k` (ScrollUp → Sessions cursor up), `selected_session_id`
/// must be updated to the newly highlighted session.
#[test]
fn test_BC_2_06_018_AC008_selected_session_id_updated_on_cursor_up() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![make_session("sess-A"), make_session("sess-B")];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    // Start at index 1 (sess-B selected).
    sessions_state.list_state.select(Some(1));

    // Act: press k to move cursor up from sess-B (index 1) to sess-A (index 0).
    let k_key = KeyEvent {
        code: KeyCode::Char('k'),
        modifiers: no_mod(),
    };
    dispatch_key_event(&mut app, &k_key, &layers, &mut sessions_state);

    // Assert: sessions cursor moved to index 0.
    assert_eq!(
        sessions_state.list_state.selected(),
        Some(0),
        "precondition: sessions cursor at index 0 after k"
    );

    // Assert: selected_session_id must reflect the new selection (sess-A).
    assert_eq!(
        app.selected_session_id.as_deref(),
        Some("sess-A"),
        "BC-2.06.018 AC-008: selected_session_id must be updated to 'sess-A' after \
         pressing k (cursor moves from 1 to 0)."
    );
}

/// End-to-end: after navigating to sess-B, an event for sess-A must NOT trigger auto-scroll.
///
/// This closes the gap: verifies that the cursor-sync + auto-scroll gate work together
/// in the realistic scenario (user presses j → navigates to sess-B → event for sess-A
/// arrives → ribbon scroll must NOT be disturbed).
#[test]
fn test_BC_2_06_018_AC008_auto_scroll_correct_after_cursor_move() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![make_session("sess-A"), make_session("sess-B")];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    // Seed the ribbon with some events so scroll index 3 is within bounds.
    for _ in 0..5 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            "sess-A".to_string(),
            "{}".to_string(),
            1u64,
            0i64,
        );
    }

    // Navigate: press j to select sess-B (index 1).
    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: no_mod(),
    };
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);
    // selected_session_id should now be "sess-B".
    assert_eq!(
        app.selected_session_id.as_deref(),
        Some("sess-B"),
        "precondition: cursor moved to sess-B via j"
    );

    // Manually set ribbon scroll to row 3 (user scrolled up in the ribbon).
    app.event_ribbon_state.list_state.select(Some(3));
    app.event_ribbon_state.pinned_top = false;

    // Act: event arrives for sess-A (not the selected session sess-B).
    on_hook_event_received(
        &mut app,
        HookType::Notification,
        "sess-A".to_string(),
        "{}".to_string(),
        2u64,
        0i64,
    );

    // Assert: ribbon scroll must NOT have moved to row 0.
    // sess-A event must not disturb the ribbon when sess-B is selected.
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(3),
        "BC-2.06.018 AC-008: after navigating to sess-B via j, an event for sess-A must \
         NOT reset ribbon scroll to row 0. scroll must remain at row 3."
    );
}
