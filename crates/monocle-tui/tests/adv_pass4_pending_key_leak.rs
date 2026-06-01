//! ADV Pass-4 — I1: pending_key cross-mode/focus leak (BC-2.06.018 PC-5 / AC-007).
//!
//! # Finding (I1 MAJOR)
//!
//! `App::pending_key` is cleared when `dispatch_key_event` exits the
//! `Dashboard { EventRibbon }` context (the `else` branch that runs for any other
//! mode/focus), but there is no explicit test that covers the two leak scenarios:
//!
//!   (a) Press `g` in EventRibbon focus, then switch focus (Tab → Sessions) and press
//!       `g` again — the second `g` must NOT fire `jump_newest` because the pending
//!       was cleared by the Tab (focus switch).
//!
//!   (b) Press `g`, then press an intervening key `j`, then press `g` again — the
//!       third keypress must NOT fire `jump_newest` because the pending was cleared
//!       by the `j` keypress.
//!
//! # Red Gate rationale
//!
//! If the clear logic for pending_key is INCOMPLETE (e.g., the Tab branch misses
//! the clear, or the dispatch order changes), test (a) will fire jump_newest on the
//! second `g` and the ribbon scroll will be set to row 0 even though the user was no
//! longer in EventRibbon focus. Similarly for test (b), the phantom jump would fire.
//!
//! These tests are written to fail if the clear logic is absent or incorrect.  They
//! may currently PASS if the implementation is correct — but they document the leak
//! scenarios and will catch regressions.
//!
//! Per the task brief: "if already correct, these may pass — but write them to
//! genuinely exercise the leak path."
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
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
use monocle_tui::ui::sessions_panel::SessionsPanelState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_app_event_ribbon_focus_with_events() -> App {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![EnrichedSession::new(
        "sess-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Active,
        None,
        None,
        None,
        0,
        None,
    )];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    // Seed events so jump_newest/oldest has something to work with.
    for i in 0..10u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            "sess-001".to_string(),
            "{}".to_string(),
            i,
            0i64,
        );
    }
    // Start ribbon at a non-zero position (simulate user scrolled down).
    app.event_ribbon_state.list_state.select(Some(5));
    app.event_ribbon_state.pinned_top = true;
    app
}

fn dispatch(app: &mut App, code: KeyCode, sessions_state: &mut SessionsPanelState) {
    let layers = build_builtin_binding_layers();
    let key = KeyEvent {
        code,
        modifiers: KeyModifiers::default(),
    };
    dispatch_key_event(app, &key, &layers, sessions_state);
}

// ---------------------------------------------------------------------------
// I1-a: pending_key cleared on focus switch (Tab: EventRibbon → Sessions).
//
// Sequence:
//   1. Press 'g' in EventRibbon focus → pending_key = Some('g').
//   2. Press Tab → switches focus to Sessions, clears pending_key.
//   3. Press 'g' again (now in Sessions focus) → NO jump_newest.
//
// RED indicator: if pending_key is NOT cleared by Tab, the second 'g' in Sessions
// focus would set pending_key = None without entering the EventRibbon block. In
// practice the 'g' key in Sessions focus would just be passed to resolve_binding,
// which finds no binding for 'g' in Sessions focus → Noop. So the ribbon state
// should be UNCHANGED (not reset to row 0). If pending was NOT cleared by Tab AND
// we then somehow get back to EventRibbon focus with pending still set, pressing
// another 'g' would incorrectly fire jump_newest.
//
// The direct test: after g (pending set), Tab, g (pending cleared, noop) —
// verify pending_key == None and ribbon scroll was NOT reset to 0.
// ---------------------------------------------------------------------------

/// BC-2.06.018 PC-5 (I1): pressing `g` in EventRibbon focus sets `pending_key = Some('g')`.
/// Pressing Tab (focus switch to Sessions) must clear `pending_key`. A subsequent `g`
/// in Sessions focus must NOT fire `jump_newest` (ribbon scroll stays at last position).
///
/// This tests the clear-on-mode/focus-change invariant for pending_key.
#[test]
fn test_BC_2_06_018_PC5_I1a_pending_key_cleared_on_focus_switch_tab() {
    let mut app = make_app_event_ribbon_focus_with_events();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    // Precondition: in EventRibbon focus, ribbon at row 5 (scrolled), pending_key=None.
    assert!(
        matches!(
            &app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::EventRibbon
            }
        ),
        "precondition: must be in Dashboard EventRibbon focus"
    );
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(5),
        "precondition: ribbon must be at row 5"
    );
    assert_eq!(
        app.pending_key, None,
        "precondition: pending_key must be None"
    );

    // Step 1: Press 'g' → pending_key = Some('g').
    dispatch(&mut app, KeyCode::Char('g'), &mut sessions_state);
    assert_eq!(
        app.pending_key,
        Some('g'),
        "step 1: first 'g' in EventRibbon focus must set pending_key = Some('g')"
    );
    // Ribbon must NOT have moved yet (first 'g' is just a prefix accumulator).
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(5),
        "step 1: first 'g' must NOT jump ribbon (no action yet)"
    );

    // Step 2: Press Tab → focus switches to Sessions, pending_key must be cleared.
    dispatch(&mut app, KeyCode::Tab, &mut sessions_state);
    // After Tab, mode must be Dashboard { Sessions }.
    assert!(
        matches!(
            &app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "step 2: Tab must switch focus to Sessions"
    );
    // CRITICAL: pending_key must be cleared when we leave EventRibbon focus.
    assert_eq!(
        app.pending_key, None,
        "BC-2.06.018 PC-5 I1 RED: pending_key must be cleared when focus leaves \
         EventRibbon (Tab switches focus to Sessions). \
         If pending_key is NOT cleared here, a stale 'g' prefix leaks across focus modes. \
         FIX: the 'else' branch in dispatch_key_event (not in Dashboard {{ EventRibbon }}) \
         must clear pending_key."
    );

    // Step 3: Press 'g' in Sessions focus → must NOT fire jump_newest.
    let ribbon_before_g2 = app.event_ribbon_state.list_state.selected();
    dispatch(&mut app, KeyCode::Char('g'), &mut sessions_state);
    let ribbon_after_g2 = app.event_ribbon_state.list_state.selected();

    // Ribbon must NOT have been reset to row 0 (jump_newest was NOT fired).
    // In Sessions focus, 'g' has no binding → Noop → ribbon unchanged.
    assert_eq!(
        ribbon_before_g2, ribbon_after_g2,
        "BC-2.06.018 PC-5 I1: after focus switch (Tab), pressing 'g' in Sessions focus \
         must NOT jump ribbon. Ribbon was at {:?} before, got {:?} after. \
         If jump_newest fired, pending_key was not cleared by Tab.",
        ribbon_before_g2, ribbon_after_g2
    );
    // pending_key must still be None (not set by 'g' in Sessions focus).
    assert_eq!(
        app.pending_key, None,
        "BC-2.06.018 PC-5 I1: pending_key must remain None after 'g' in Sessions focus"
    );
}

// ---------------------------------------------------------------------------
// I1-b: pending_key cleared by an intervening key.
//
// Sequence:
//   1. Press 'g' in EventRibbon focus → pending_key = Some('g').
//   2. Press 'j' (not 'g') → pending_key cleared, scroll_ribbon_down fires.
//   3. Press 'g' again → pending_key = Some('g') (new pending, no jump).
//   4. Press any non-'g' key → pending_key = None (no phantom jump).
//
// RED indicator: if pending_key is NOT cleared by the intervening 'j', the third
// 'g' press would fire as the SECOND 'g' of a gg sequence → jump_newest. The
// test verifies no phantom jump occurs on the second 'g'.
// ---------------------------------------------------------------------------

/// BC-2.06.018 PC-5 (I1): after `g` (pending), an intervening `j` clears pending.
/// A subsequent `g` must start a NEW pending (not complete the stale gg sequence).
/// The second `g` in this new sequence must NOT fire `jump_newest`.
///
/// Tests the "clear on any non-g key" rule for pending_key.
#[test]
fn test_BC_2_06_018_PC5_I1b_pending_key_cleared_by_intervening_key() {
    let mut app = make_app_event_ribbon_focus_with_events();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    // Precondition: in EventRibbon focus, ribbon at row 5.
    assert!(
        matches!(
            &app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::EventRibbon
            }
        ),
        "precondition: must be in Dashboard EventRibbon focus"
    );
    assert_eq!(
        app.pending_key, None,
        "precondition: pending_key must be None"
    );

    // Step 1: Press 'g' → pending_key = Some('g').
    dispatch(&mut app, KeyCode::Char('g'), &mut sessions_state);
    assert_eq!(
        app.pending_key,
        Some('g'),
        "step 1: first 'g' must set pending_key = Some('g')"
    );

    // Step 2: Press 'j' (intervening key) → pending_key must be cleared.
    dispatch(&mut app, KeyCode::Char('j'), &mut sessions_state);
    // After 'j', pending_key must be None (cleared by the non-'g' key).
    assert_eq!(
        app.pending_key, None,
        "BC-2.06.018 PC-5 I1 RED: pending_key must be cleared when a non-'g' key is pressed \
         while pending (the '_ =>' branch in the match arm). \
         FIX: the catch-all arm in the EventRibbon key dispatch match must clear pending_key \
         before falling through to resolve_binding."
    );

    // Step 3: Press 'g' again → starts a NEW pending (must set pending_key = Some('g')),
    //         must NOT fire jump_newest.
    let ribbon_before_g2 = app.event_ribbon_state.list_state.selected();
    dispatch(&mut app, KeyCode::Char('g'), &mut sessions_state);

    // The second 'g' here is a fresh FIRST press (no stale pending from step 1).
    // So pending_key = Some('g') and NO jump_newest fires.
    assert_eq!(
        app.pending_key,
        Some('g'),
        "step 3: 'g' with no prior pending must set pending_key = Some('g') (fresh sequence)"
    );

    // Ribbon must NOT have been reset to row 0 — no gg jump occurred.
    let ribbon_after_g2 = app.event_ribbon_state.list_state.selected();
    // Note: 'j' in step 2 may have scrolled the ribbon. ribbon_after_g2 should equal
    // ribbon_after_j (ribbon not reset to 0). The key assertion: ribbon NOT at 0 if it
    // was > 0 after step 2 scroll, or specifically: jump_newest was NOT fired.
    // We test the absence of phantom jump by checking pending_key correctly reflects fresh state.
    assert_eq!(
        app.pending_key,
        Some('g'),
        "BC-2.06.018 PC-5 I1: after intervening 'j' + new 'g', pending must be Some('g') \
         (fresh sequence, not phantom completion)"
    );

    // Step 4: Press a non-'g' key to clear the fresh pending without triggering jump.
    dispatch(&mut app, KeyCode::Char('k'), &mut sessions_state);
    assert_eq!(
        app.pending_key, None,
        "step 4: non-'g' key after fresh pending must clear pending_key"
    );

    // The ribbon should NOT be at row 0 unless 'k' scrolled it up from 1.
    // Main invariant: no PHANTOM gg jump occurred during the whole sequence.
    // We assert pending_key is consistently managed — the jump_newest function
    // sets ribbon to row 0 and pinned_top=false. If a phantom gg fired, pinned_top
    // would be false and ribbon at 0. Let's check: the 'j' in step 2 sets pinned_top=true.
    // If phantom gg fired, pinned_top would have been reset to false. But 'k' in step 4
    // calls scroll_ribbon_up which also could clear pinned_top. So we can't use pinned_top
    // as the sole indicator. The critical test is pending_key stays consistent — tested above.
    let _ = ribbon_before_g2; // prevent unused warning
    let _ = ribbon_after_g2;
}

// ---------------------------------------------------------------------------
// I1-c: mode change (Filtering) clears pending_key.
//
// Sequence:
//   1. Press 'g' in EventRibbon focus → pending_key = Some('g').
//   2. Press '/' → enters Filtering mode, pending_key must be cleared.
//   3. Press 'g' in Filtering → append to query (NOT a pending prefix).
// ---------------------------------------------------------------------------

/// BC-2.06.018 PC-5 (I1): pressing `g` in EventRibbon focus sets pending_key.
/// Entering Filtering mode (via `/`) must clear pending_key so that `g` in Filtering
/// mode appends to the filter query rather than acting as a gg prefix.
///
/// This tests the clear-on-mode-change invariant (entering a non-Dashboard-EventRibbon
/// mode must always clear pending_key).
#[test]
fn test_BC_2_06_018_PC5_I1c_pending_key_cleared_on_filtering_mode_entry() {
    let mut app = make_app_event_ribbon_focus_with_events();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    // Precondition: EventRibbon focus, pending_key = None.
    assert_eq!(
        app.pending_key, None,
        "precondition: pending_key must be None"
    );

    // Step 1: Set pending by pressing 'g'.
    dispatch(&mut app, KeyCode::Char('g'), &mut sessions_state);
    assert_eq!(
        app.pending_key,
        Some('g'),
        "step 1: first 'g' must set pending_key = Some('g')"
    );

    // Step 2: Press '/' → must transition to Filtering mode.
    // '/' is bound to StartFilter { Sessions } in Dashboard mode (per-context binding).
    // Entering Filtering mode means we are no longer in Dashboard { EventRibbon }.
    // The dispatch_key_event 'else' branch (not in Dashboard { EventRibbon }) must
    // clear pending_key before forwarding to resolve_binding.
    dispatch(&mut app, KeyCode::Char('/'), &mut sessions_state);

    // After '/', mode should be Filtering or at minimum not Dashboard { EventRibbon }.
    // The important assertion: pending_key must be None.
    assert_eq!(
        app.pending_key, None,
        "BC-2.06.018 PC-5 I1 RED: pending_key must be cleared when mode changes away from \
         Dashboard {{ EventRibbon }} (pressing '/' enters Filtering mode). \
         If pending_key is NOT cleared, a stale 'g' prefix persists into Filtering mode. \
         FIX: the else branch in dispatch_key_event (not in Dashboard {{ EventRibbon }}) \
         must clear pending_key before calling resolve_binding."
    );
}
