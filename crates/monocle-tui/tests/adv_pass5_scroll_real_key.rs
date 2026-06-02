//! ADV Pass-5 — F-1 REPLACEMENT: Real j/↓/k/↑ in EventRibbon focus must dispatch
//!              Action::ScrollDown / Action::ScrollUp (BC-2.06.018 AC-010 §2).
//!
//! # Why the Pass-4 test was vacuous
//!
//! The old `adv_pass4_scroll_dispatch.rs` test mapped SYNTHETIC PUA keys
//! (`\u{E001}` / `\u{E002}`) to `Action::ScrollDown` / `Action::ScrollUp` via
//! `build_layers_with_scroll_actions()`, then dispatched those synthetic keys.
//! This proved that the `ScrollDown`/`ScrollUp` arms execute when reachable —
//! but NO real user key (`j`, `↓`, `k`, `↑`) actually produces `Action::ScrollDown`
//! / `Action::ScrollUp`. Real `j`/`↓` still resolve to `Action::SelectNext` (builtin)
//! and real `k`/`↑` to `Action::SelectPrev`.
//!
//! # What this test proves
//!
//! 1. `resolve_binding` is called directly with the real binding layers (no synthetic
//!    override) and the Dashboard/EventRibbon mode.
//! 2. For real `j` / `↓` in that mode, the resolved action MUST be `Action::ScrollDown`.
//! 3. For real `k` / `↑` in that mode, the resolved action MUST be `Action::ScrollUp`.
//!
//! RED: currently `build_builtin_binding_layers()` maps `j → SelectNext` and
//! `k → SelectPrev` globally with no per-context override for EventRibbon focus.
//! `resolve_binding` returns `SelectNext`/`SelectPrev`, not `ScrollDown`/`ScrollUp`.
//!
//! AFTER FIX: the implementer adds per-context bindings for
//! `(j, Dashboard)` → `ScrollDown` and `(k, Dashboard)` → `ScrollUp` (or equivalent
//! mechanism that produces `ScrollDown`/`ScrollUp` when EventRibbon is focused).
//! Then `resolve_binding` returns `ScrollDown`/`ScrollUp` and these tests pass.
//!
//! # Regression guards (GREEN before and after fix)
//!
//! - `j` in Sessions focus must still produce `SelectNext` (sessions cursor move).
//! - `k` in Sessions focus must still produce `SelectPrev` (sessions cursor move).
//!
//! These tests remain GREEN because the fix only overrides the binding in EventRibbon
//! focus, not in Sessions focus.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]
// expect/unwrap are idiomatic assertion amplification in test code.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_config::MonocleConfig;
use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::tui::binding::{resolve_binding, KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{Action, AppMode, FocusSnapshot};
use monocle_ipc::types::HookType;
use monocle_tui::app::{
    build_builtin_binding_layers, dispatch_key_event, on_hook_event_received, App,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn no_mod() -> KeyModifiers {
    KeyModifiers::default()
}

/// Return a human-readable name for an `Action` Option for assertion messages.
/// `Action` does not derive `Debug`, so we match manually.
fn action_display(a: &Option<Action>) -> &'static str {
    match a {
        Some(Action::ScrollDown) => "Some(ScrollDown)",
        Some(Action::ScrollUp) => "Some(ScrollUp)",
        Some(Action::SelectNext) => "Some(SelectNext)",
        Some(Action::SelectPrev) => "Some(SelectPrev)",
        Some(Action::Quit) => "Some(Quit)",
        Some(Action::Noop) => "Some(Noop)",
        Some(_) => "Some(<other>)",
        None => "None",
    }
}

fn make_app_with_events_event_ribbon_focus() -> App {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![
        EnrichedSession::new(
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
        ),
        EnrichedSession::new(
            "sess-002".to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Active,
            None,
            None,
            None,
            0,
            None,
        ),
    ];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
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
    app.event_ribbon_state.list_state.select(Some(0));
    app.event_ribbon_state.pinned_top = false;
    app
}

// ---------------------------------------------------------------------------
// F-1: Real j / ↓ in EventRibbon focus must resolve to Action::ScrollDown
//
// RED: `build_builtin_binding_layers()` maps j → SelectNext (builtin) with no
// per-context override for Dashboard/EventRibbon focus. `resolve_binding` returns
// `SelectNext`, not `ScrollDown`.
//
// AFTER FIX: a per-context binding maps j / ↓ → `Action::ScrollDown` when in
// Dashboard mode, so `resolve_binding` returns `ScrollDown`.
// ---------------------------------------------------------------------------

/// BC-2.06.018 AC-010 §2 (F-1): `j` with real binding layers in `Dashboard { EventRibbon }`
/// focus must resolve to `Action::ScrollDown`.
///
/// RED: `build_builtin_binding_layers()` has no per-context binding for
/// `(j, Dashboard)` → `ScrollDown`. `resolve_binding` returns `SelectNext`.
///
/// FIX: add a per-context binding `(KeyCode::Char('j'), AppModeTag::Dashboard)`
/// → `Action::ScrollDown`.
#[test]
fn test_BC_2_06_018_AC010_F1_real_j_resolves_to_scroll_down_in_event_ribbon_focus() {
    let layers = build_builtin_binding_layers();
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: no_mod(),
    };

    let resolved = resolve_binding(&j_key, &mode, &layers);
    let action = resolved.map(|(a, _)| a);

    assert!(
        matches!(action, Some(Action::ScrollDown)),
        "BC-2.06.018 AC-010 F-1 RED: real 'j' in Dashboard {{EventRibbon}} focus must resolve \
         to Action::ScrollDown via the binding layer. Got {}. \
         CURRENT DEFECT: build_builtin_binding_layers() maps j → SelectNext (builtin) with \
         no per-context override for (j, Dashboard). \
         FIX: add a per-context binding (j, Dashboard) → ScrollDown so real 'j' in \
         EventRibbon focus reaches the ScrollDown arm (not SelectNext).",
        action_display(&action)
    );
}

/// BC-2.06.018 AC-010 §2 (F-1): `↓` with real binding layers in `Dashboard { EventRibbon }`
/// focus must resolve to `Action::ScrollDown`.
///
/// RED: `build_builtin_binding_layers()` maps `↓ → SelectNext` (builtin) with no override.
/// `resolve_binding` returns `SelectNext`.
///
/// FIX: add a per-context binding `(KeyCode::Down, AppModeTag::Dashboard)` → `Action::ScrollDown`.
#[test]
fn test_BC_2_06_018_AC010_F1_real_down_arrow_resolves_to_scroll_down_in_event_ribbon_focus() {
    let layers = build_builtin_binding_layers();
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let down_key = KeyEvent {
        code: KeyCode::Down,
        modifiers: no_mod(),
    };

    let resolved = resolve_binding(&down_key, &mode, &layers);
    let action = resolved.map(|(a, _)| a);

    assert!(
        matches!(action, Some(Action::ScrollDown)),
        "BC-2.06.018 AC-010 F-1 RED: real '↓' in Dashboard {{EventRibbon}} focus must resolve \
         to Action::ScrollDown via the binding layer. Got {}. \
         CURRENT DEFECT: build_builtin_binding_layers() maps ↓ → SelectNext (builtin) with \
         no per-context override. \
         FIX: add a per-context binding (KeyCode::Down, AppModeTag::Dashboard) → ScrollDown.",
        action_display(&action)
    );
}

/// BC-2.06.018 AC-010 §2 (F-1): `k` with real binding layers in `Dashboard { EventRibbon }`
/// focus must resolve to `Action::ScrollUp`.
///
/// RED: `build_builtin_binding_layers()` maps `k → SelectPrev` (builtin) with no override.
/// `resolve_binding` returns `SelectPrev`.
///
/// FIX: add a per-context binding `(KeyCode::Char('k'), AppModeTag::Dashboard)` →
/// `Action::ScrollUp`.
#[test]
fn test_BC_2_06_018_AC010_F1_real_k_resolves_to_scroll_up_in_event_ribbon_focus() {
    let layers = build_builtin_binding_layers();
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let k_key = KeyEvent {
        code: KeyCode::Char('k'),
        modifiers: no_mod(),
    };

    let resolved = resolve_binding(&k_key, &mode, &layers);
    let action = resolved.map(|(a, _)| a);

    assert!(
        matches!(action, Some(Action::ScrollUp)),
        "BC-2.06.018 AC-010 F-1 RED: real 'k' in Dashboard {{EventRibbon}} focus must resolve \
         to Action::ScrollUp via the binding layer. Got {}. \
         CURRENT DEFECT: build_builtin_binding_layers() maps k → SelectPrev (builtin) with \
         no per-context override. \
         FIX: add a per-context binding (KeyCode::Char('k'), AppModeTag::Dashboard) → ScrollUp.",
        action_display(&action)
    );
}

/// BC-2.06.018 AC-010 §2 (F-1): `↑` with real binding layers in `Dashboard { EventRibbon }`
/// focus must resolve to `Action::ScrollUp`.
///
/// RED: `build_builtin_binding_layers()` maps `↑ → SelectPrev` (builtin) with no override.
/// `resolve_binding` returns `SelectPrev`.
///
/// FIX: add a per-context binding `(KeyCode::Up, AppModeTag::Dashboard)` →
/// `Action::ScrollUp`.
#[test]
fn test_BC_2_06_018_AC010_F1_real_up_arrow_resolves_to_scroll_up_in_event_ribbon_focus() {
    let layers = build_builtin_binding_layers();
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let up_key = KeyEvent {
        code: KeyCode::Up,
        modifiers: no_mod(),
    };

    let resolved = resolve_binding(&up_key, &mode, &layers);
    let action = resolved.map(|(a, _)| a);

    assert!(
        matches!(action, Some(Action::ScrollUp)),
        "BC-2.06.018 AC-010 F-1 RED: real '↑' in Dashboard {{EventRibbon}} focus must resolve \
         to Action::ScrollUp via the binding layer. Got {}. \
         CURRENT DEFECT: build_builtin_binding_layers() maps ↑ → SelectPrev (builtin) with \
         no per-context override. \
         FIX: add a per-context binding (KeyCode::Up, AppModeTag::Dashboard) → ScrollUp.",
        action_display(&action)
    );
}

// ---------------------------------------------------------------------------
// F-1 dispatch-level integration: real j in EventRibbon focus must scroll ribbon
// and must NOT move the sessions cursor.
// ---------------------------------------------------------------------------

/// BC-2.06.018 AC-010 §2 (F-1 dispatch): real `j` dispatched in `Dashboard { EventRibbon }`
/// focus must scroll the ribbon without touching the sessions cursor.
///
/// This is the integration complement to the four `resolve_binding` tests above.
/// It passes regardless of whether j resolves to ScrollDown or SelectNext (both scroll
/// the ribbon in EventRibbon focus via their respective dispatch arms). The authoritative
/// RED assertions are the `resolve_binding` tests; this test guards the observable end-to-end
/// behavior.
#[test]
fn test_BC_2_06_018_AC010_F1_dispatch_real_j_scrolls_ribbon_not_sessions_cursor() {
    let mut app = make_app_with_events_event_ribbon_focus();
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(0),
        "precondition: ribbon at row 0"
    );

    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: no_mod(),
    };
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);

    // Sessions cursor must NOT have moved (j is for ribbon in EventRibbon focus).
    assert_eq!(
        sessions_state.list_state.selected(),
        Some(0),
        "BC-2.06.018 AC-010 F-1: 'j' in EventRibbon focus must NOT move the sessions cursor. \
         Sessions cursor must stay at 0."
    );

    // Ribbon must have scrolled to row > 0.
    let ribbon_selected = app.event_ribbon_state.list_state.selected();
    assert!(
        ribbon_selected.map(|i| i > 0).unwrap_or(false),
        "BC-2.06.018 AC-010 F-1: 'j' in EventRibbon focus must scroll ribbon to row > 0. \
         Got {:?}.",
        ribbon_selected
    );
}

// ---------------------------------------------------------------------------
// Regression guards: j/k in Sessions focus must still move sessions cursor (GREEN).
//
// After the fix (per-context binding for Dashboard/j → ScrollDown), the Sessions
// focus path must still use SelectNext/SelectPrev for cursor movement.
// ---------------------------------------------------------------------------

/// Regression: `j` in `Dashboard { Sessions }` focus must still move sessions cursor DOWN.
#[test]
fn test_BC_2_06_018_AC010_F1_regression_j_in_sessions_focus_moves_cursor_down() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![
        EnrichedSession::new(
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
        ),
        EnrichedSession::new(
            "sess-002".to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Active,
            None,
            None,
            None,
            0,
            None,
        ),
    ];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(0));

    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: no_mod(),
    };
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);

    assert_eq!(
        sessions_state.list_state.selected(),
        Some(1),
        "regression: 'j' in Sessions focus must move sessions cursor from 0 to 1"
    );
}

/// Regression: `k` in `Dashboard { Sessions }` focus must still move sessions cursor UP.
#[test]
fn test_BC_2_06_018_AC010_F1_regression_k_in_sessions_focus_moves_cursor_up() {
    let mut app = App::new(MonocleConfig::default());
    app.sessions = vec![
        EnrichedSession::new(
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
        ),
        EnrichedSession::new(
            "sess-002".to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Active,
            None,
            None,
            None,
            0,
            None,
        ),
    ];
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    sessions_state.list_state.select(Some(1));

    let k_key = KeyEvent {
        code: KeyCode::Char('k'),
        modifiers: no_mod(),
    };
    dispatch_key_event(&mut app, &k_key, &layers, &mut sessions_state);

    assert_eq!(
        sessions_state.list_state.selected(),
        Some(0),
        "regression: 'k' in Sessions focus must move sessions cursor from 1 to 0"
    );
}
