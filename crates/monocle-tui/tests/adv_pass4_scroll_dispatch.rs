//! ADV Pass-4 — C1: EventRibbon-focused j/↓/k/↑ must dispatch Action::ScrollDown/ScrollUp
//!                   variants (BC-2.06.018 AC-010 §2).
//!
//! # Finding (C1 BLOCKER)
//!
//! `Action::ScrollDown` and `Action::ScrollUp` are DEAD variants in `dispatch_key_event`.
//! The current binding table maps j/↓ → `Action::SelectNext` and k/↑ → `Action::SelectPrev`.
//! The `dispatch_key_event` match arms for `SelectNext`/`SelectPrev` contain per-focus
//! disambiguation that calls `scroll_ribbon_down`/`scroll_ribbon_up` in EventRibbon focus.
//!
//! AC-010 §2 requires `Action::ScrollDown`/`ScrollUp` to be the DISPATCHED variants.
//! The implementer will:
//!   1. Add `Action::ScrollDown` and `Action::ScrollUp` to the binding table for
//!      j/↓ and k/↑ respectively, OR add a per-context EventRibbon binding overriding
//!      the current global SelectNext/SelectPrev.
//!   2. Add `Action::ScrollDown` / `Action::ScrollUp` match arms in `dispatch_key_event`
//!      that call `scroll_ribbon_down`/`scroll_ribbon_up` unconditionally (because these
//!      actions only fire in EventRibbon context by design).
//!
//! # How to prove the ScrollDown/ScrollUp variant is DEAD
//!
//! `scroll_ribbon_down`/`scroll_ribbon_up` are currently called from the `SelectNext`/
//! `SelectPrev` arms — not from a dedicated `ScrollDown`/`ScrollUp` arm.  The
//! `(action, _)` catch-all arm (for all other actions) just calls `transition()`, which
//! for `ScrollDown`/`ScrollUp` returns identity (EC-061: unmatched (mode, action) → identity).
//! So if `Action::ScrollDown` were dispatched, the ribbon would NOT scroll — it would
//! be a no-op.
//!
//! The RED test: invoke `dispatch_key_event` with a mock binding that maps a dedicated
//! test key to `Action::ScrollDown` / `Action::ScrollUp`, then assert the ribbon
//! DID scroll.  Currently it will NOT scroll (ScrollDown falls through to the identity
//! catch-all arm which does not call scroll_ribbon_down).
//!
//! This isolates the dead-variant defect: if `Action::ScrollDown` had a wired arm,
//! the ribbon would scroll; since it doesn't, it doesn't.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]
// expect/unwrap are idiomatic assertion amplification in test code.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_config::MonocleConfig;
use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::tui::binding::{BindingLayers, KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{Action, AppMode, FocusSnapshot};
use monocle_tui::app::{dispatch_key_event, on_hook_event_received, App};
use monocle_ipc::types::HookType;
use monocle_tui::ui::sessions_panel::SessionsPanelState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_app_with_events_event_ribbon_focus() -> App {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![
        EnrichedSession::new(
            "sess-001".to_string(),
            "claude-code".to_string(),
            None, None, SessionStatus::Active, None, None, None, 0, None,
        ),
        EnrichedSession::new(
            "sess-002".to_string(),
            "claude-code".to_string(),
            None, None, SessionStatus::Active, None, None, None, 0, None,
        ),
    ];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    for i in 0..10u64 {
        on_hook_event_received(
            &mut app, HookType::Notification, "sess-001".to_string(),
            "{}".to_string(), i, 0i64,
        );
    }
    // Ribbon at row 0 (newest), not pinned.
    app.event_ribbon_state.list_state.select(Some(0));
    app.event_ribbon_state.pinned_top = false;
    app
}

fn no_mod() -> KeyModifiers { KeyModifiers::default() }

/// Build a `BindingLayers` that maps a UNIQUE test key (`Ctrl+PageDown`) to
/// `Action::ScrollDown` and another (`Ctrl+PageUp`) to `Action::ScrollUp`.
///
/// Using an unmapped key ensures the test key goes only through the `ScrollDown`/
/// `ScrollUp` arm — not through any other binding.  The actual key code used is
/// `KeyCode::Unknown` with a synthetic per-context entry to isolate the Action.
///
/// Since `BindingLayers::builtin` / `global` / `per_context` accept any `KeyEvent`,
/// we insert a global binding for a char that no real binding table uses
/// (specifically the private-use characters).
///
/// In practice: we use `KeyCode::Char('\u{E001}')` (PUA) → `Action::ScrollDown` and
/// `KeyCode::Char('\u{E002}')` → `Action::ScrollUp` as our synthetic test keys.
fn build_layers_with_scroll_actions() -> BindingLayers {
    let mut layers = monocle_tui::app::build_builtin_binding_layers();
    // Override j/↓ → ScrollDown in Dashboard EventRibbon context.
    // This simulates the FIX: per-context binding for EventRibbon that maps
    // j/↓ → ScrollDown (displacing the existing global SelectNext binding).
    // We DON'T do that here to avoid changing the existing bindings; instead
    // we add a dedicated per-context mapping for a synthetic test key.
    //
    // Actually, the cleanest approach for the RED test: add a GLOBAL binding
    // for a synthetic key that maps to Action::ScrollDown / Action::ScrollUp.
    // Then dispatch that key in EventRibbon focus and assert the ribbon scrolls.
    // Currently the ScrollDown/ScrollUp arms don't exist in dispatch_key_event —
    // so dispatching those Actions does nothing (ribbon stays at 0).
    layers.global.insert(
        KeyEvent {
            code: KeyCode::Char('\u{E001}'), // PUA synthetic key → ScrollDown
            modifiers: no_mod(),
        },
        Action::ScrollDown,
    );
    layers.global.insert(
        KeyEvent {
            code: KeyCode::Char('\u{E002}'), // PUA synthetic key → ScrollUp
            modifiers: no_mod(),
        },
        Action::ScrollUp,
    );
    layers
}

// ---------------------------------------------------------------------------
// C1-DEAD: Action::ScrollDown/ScrollUp variant is a no-op (dead) in
// dispatch_key_event — ribbon does NOT scroll when these actions are dispatched.
//
// RED: the ribbon scroll stays at 0 when Action::ScrollDown is dispatched via
//      the binding layer, because dispatch_key_event has NO arm for ScrollDown.
//      It falls through to the generic `(action, _)` arm → transition() → identity.
//      After the fix: a dedicated ScrollDown/ScrollUp arm calls scroll_ribbon_down/up.
// ---------------------------------------------------------------------------

/// BC-2.06.018 AC-010 §2 (C1 — DEAD VARIANT): dispatching `Action::ScrollDown` in
/// `Dashboard { EventRibbon }` focus via the binding layer must call `scroll_ribbon_down`
/// and advance the ribbon scroll offset.
///
/// RED: `dispatch_key_event` has no `Action::ScrollDown` arm. The action falls through
/// to the `(action, _)` catch-all → `transition(mode, ScrollDown)` → identity (EC-061).
/// `scroll_ribbon_down` is never called. The ribbon scroll offset remains at 0.
///
/// After the fix: the implementer adds an `Action::ScrollDown` arm that calls
/// `scroll_ribbon_down(&mut app.event_ribbon_state, &app.event_ribbon_events)`.
#[test]
fn test_BC_2_06_018_AC010_C1_scroll_down_variant_is_dead_in_dispatch() {
    let mut app = make_app_with_events_event_ribbon_focus();
    let layers = build_layers_with_scroll_actions();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    // Precondition: ribbon at row 0 (newest).
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(0),
        "precondition: ribbon must start at row 0"
    );
    assert!(
        !app.event_ribbon_state.pinned_top,
        "precondition: ribbon must not be pinned_top"
    );

    // Dispatch the synthetic key bound to Action::ScrollDown.
    let scroll_down_key = KeyEvent {
        code: KeyCode::Char('\u{E001}'),
        modifiers: no_mod(),
    };
    dispatch_key_event(&mut app, &scroll_down_key, &layers, &mut sessions_state);

    // Assert: ribbon scroll offset must have moved from 0 to 1 (scroll_ribbon_down fired).
    // RED: currently no ScrollDown arm exists → ribbon stays at 0.
    // AFTER FIX: ribbon moves to 1 and pinned_top = true.
    let ribbon_selected = app.event_ribbon_state.list_state.selected();
    assert!(
        ribbon_selected.map(|i| i > 0).unwrap_or(false),
        "BC-2.06.018 AC-010 C1 RED (dead variant): dispatching Action::ScrollDown must \
         scroll the ribbon down (select index > 0). Got {:?}. \
         CURRENT DEFECT: dispatch_key_event has no Action::ScrollDown arm — the action \
         falls through to the transition() catch-all which returns identity (no scroll). \
         FIX: add an Action::ScrollDown arm that calls scroll_ribbon_down().",
        ribbon_selected
    );

    assert!(
        app.event_ribbon_state.pinned_top,
        "BC-2.06.018 AC-010 C1 RED: after ScrollDown, pinned_top must be true \
         (scroll_ribbon_down sets pinned_top when index > 0). \
         Currently no ScrollDown arm → pinned_top stays false."
    );
}

/// BC-2.06.018 AC-010 §2 (C1 — DEAD VARIANT): dispatching `Action::ScrollUp` in
/// `Dashboard { EventRibbon }` focus via the binding layer must call `scroll_ribbon_up`
/// and return the ribbon to row 0 (clearing pinned_top).
///
/// RED: `dispatch_key_event` has no `Action::ScrollUp` arm. Falls through to catch-all
/// → `transition()` → identity. `scroll_ribbon_up` is never called. The ribbon stays
/// at the scrolled-down position instead of returning to row 0.
///
/// After the fix: an `Action::ScrollUp` arm calls
/// `scroll_ribbon_up(&mut app.event_ribbon_state, &app.event_ribbon_events)`.
#[test]
fn test_BC_2_06_018_AC010_C1_scroll_up_variant_is_dead_in_dispatch() {
    let mut app = make_app_with_events_event_ribbon_focus();
    let layers = build_layers_with_scroll_actions();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    // Set ribbon to row 5 (scrolled down), pinned.
    app.event_ribbon_state.list_state.select(Some(5));
    app.event_ribbon_state.pinned_top = true;

    // Precondition: ribbon at row 5, pinned_top = true.
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(5),
        "precondition: ribbon must be at row 5"
    );
    assert!(
        app.event_ribbon_state.pinned_top,
        "precondition: pinned_top must be true"
    );

    // Dispatch the synthetic key bound to Action::ScrollUp — scroll from 5 back toward 0.
    // We dispatch it 5 times to scroll from 5 → 0.
    let scroll_up_key = KeyEvent {
        code: KeyCode::Char('\u{E002}'),
        modifiers: no_mod(),
    };
    for _ in 0..5 {
        dispatch_key_event(&mut app, &scroll_up_key, &layers, &mut sessions_state);
    }

    // Assert: ribbon must be back at row 0 and pinned_top must be cleared.
    // RED: currently no ScrollUp arm → ribbon stays at 5, pinned_top stays true.
    let ribbon_selected = app.event_ribbon_state.list_state.selected();
    assert_eq!(
        ribbon_selected,
        Some(0),
        "BC-2.06.018 AC-010 C1 RED (dead variant): dispatching Action::ScrollUp (×5) must \
         scroll the ribbon back to row 0 from row 5. Got {:?}. \
         CURRENT DEFECT: no ScrollUp arm in dispatch_key_event → ribbon stays at 5. \
         FIX: add an Action::ScrollUp arm that calls scroll_ribbon_up().",
        ribbon_selected
    );

    assert!(
        !app.event_ribbon_state.pinned_top,
        "BC-2.06.018 AC-010 C1 RED: after ScrollUp to row 0, pinned_top must be false \
         (scroll_ribbon_up clears pinned_top when index reaches 0). \
         Currently no ScrollUp arm → pinned_top stays true."
    );
}

// ---------------------------------------------------------------------------
// Regression guard: j/k in Sessions focus still moves sessions cursor (GREEN).
//
// This must stay GREEN before and after the C1 fix — the fix must not break
// existing Session-focus cursor behavior.
// ---------------------------------------------------------------------------

/// Regression: `j` in `Dashboard { Sessions }` focus must still move the sessions
/// cursor DOWN. This behavior must survive the C1 fix (rewiring j → ScrollDown in
/// EventRibbon focus must not break j → SelectNext in Sessions focus).
#[test]
fn test_BC_2_06_018_AC010_C1_regression_j_in_sessions_focus_still_moves_cursor() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![
        EnrichedSession::new("sess-001".to_string(), "claude-code".to_string(),
            None, None, SessionStatus::Active, None, None, None, 0, None),
        EnrichedSession::new("sess-002".to_string(), "claude-code".to_string(),
            None, None, SessionStatus::Active, None, None, None, 0, None),
    ];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let layers = monocle_tui::app::build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    let j_key = KeyEvent { code: KeyCode::Char('j'), modifiers: no_mod() };
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);

    assert_eq!(
        sessions_state.list_state.selected(),
        Some(1),
        "regression: j in Sessions focus must still move sessions cursor from 0 to 1"
    );
}

/// Regression: `k` in `Dashboard { Sessions }` focus must still move sessions cursor UP.
#[test]
fn test_BC_2_06_018_AC010_C1_regression_k_in_sessions_focus_still_moves_cursor_up() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![
        EnrichedSession::new("sess-001".to_string(), "claude-code".to_string(),
            None, None, SessionStatus::Active, None, None, None, 0, None),
        EnrichedSession::new("sess-002".to_string(), "claude-code".to_string(),
            None, None, SessionStatus::Active, None, None, None, 0, None),
    ];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let layers = monocle_tui::app::build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(1)); // start at row 1

    let k_key = KeyEvent { code: KeyCode::Char('k'), modifiers: no_mod() };
    dispatch_key_event(&mut app, &k_key, &layers, &mut sessions_state);

    assert_eq!(
        sessions_state.list_state.selected(),
        Some(0),
        "regression: k in Sessions focus must still move sessions cursor from 1 to 0"
    );
}
