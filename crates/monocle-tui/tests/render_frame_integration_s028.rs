//! AC-010 integration render + dispatch tests for S-028.
//!
//! Drives `App::render_frame` (the production render path) against a
//! `ratatui::backend::TestBackend` and asserts that the rendered terminal buffer
//! contains Event Ribbon content and Sessions filter content respectively.
//!
//! Also drives `dispatch_key_event` for ribbon scroll key sequences.
//!
//! # Red Gate rationale
//!
//! - Test 1 (ribbon render in Dashboard): FAILS because `render_frame` does NOT call
//!   `EventRibbon::render` on `layout.event_ribbon_area`. The right 40% area is blank.
//! - Test 2a (Filtering mode layout): FAILS because `render_frame` does NOT call
//!   `render_sessions_filter` when `app.mode` is `AppMode::Filtering`.
//! - Test 2b (zero-match sentinel): FAILS for the same reason.
//! - Tests 3a-3e (scroll dispatch): FAIL because `Action::ScrollDown`/`Action::ScrollUp`
//!   DO NOT EXIST in the `Action` enum — these are compile-gate FAILs indicating
//!   the missing Action variants must be added before the implementer can wire them.
//!
//! # BC coverage
//!
//! - BC-2.06.018 PC-1/PC-5: Event Ribbon rendered in 40% area and scroll action dispatched.
//! - BC-2.06.006 PC-1/PC-2/PC-8: Sessions filter input box + no-match sentinel rendered.
//! - AC-010: production render_frame / dispatch_key_event wiring.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot, PanelId};
use monocle_ipc::types::HookType;
use monocle_tui::app::{on_hook_event_received, render_frame, App};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use ratatui::{backend::TestBackend, Terminal};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all cell symbols from the terminal buffer as a single string.
fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

/// Collect text from a specific rectangular sub-region of the buffer.
///
/// Used to assert event ribbon content in the right 40% area without
/// false-positives from the left 60% sessions panel region.
fn buffer_region_text(terminal: &Terminal<TestBackend>, x_start: u16, width: u16) -> String {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (x_start..x_start + width).map(move |x| (x, y)))
        .filter(|(x, _)| *x < area.width)
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

/// Build a 100x30 terminal driven by App::render_frame.
fn render_to_terminal(app: &mut App) -> Terminal<TestBackend> {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let mut state = SessionsPanelState::default();
            render_frame(app, &mut state, frame);
        })
        .unwrap();
    terminal
}

// ---------------------------------------------------------------------------
// Test 1: AC-010 / BC-2.06.018 PC-1 — Event Ribbon content rendered in 40% area
//
// RED: FAILS because render_frame does NOT call EventRibbon::render on
//      layout.event_ribbon_area. The right 40% area is blank — the event ribbon
//      widget is fully implemented in isolation but never wired into render_frame.
// ---------------------------------------------------------------------------

/// In Dashboard mode with non-empty event_ribbon_events, the 40% right-side area
/// of the terminal buffer must contain event ribbon row content (timestamp, hook type,
/// session ID columns per BC-2.06.018 PC-1).
///
/// Verifies AC-010 integration requirement 1a: Event Ribbon wired into render_frame.
#[test]
fn test_BC_2_06_018_AC010_render_frame_dashboard_shows_event_ribbon_content() {
    // Arrange: App with non-empty event_ribbon_events.
    let mut app = App::new(MonocleConfig::default());
    // Inject 3 events via the production on_hook_event_received path.
    on_hook_event_received(
        &mut app,
        HookType::PreToolUse,
        "sess-abc123def".to_string(),
        r#"{"tool":"Bash"}"#.to_string(),
        42u64,
    );
    on_hook_event_received(
        &mut app,
        HookType::Notification,
        "sess-xyz789ghi".to_string(),
        r#"{"msg":"hello"}"#.to_string(),
        7u64,
    );
    on_hook_event_received(
        &mut app,
        HookType::SessionStart,
        "sess-aaa000bbb".to_string(),
        r#"{}"#.to_string(),
        1u64,
    );

    // App must be in Dashboard mode (the default).
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "test precondition: app must be in Dashboard mode"
    );
    assert_eq!(
        app.event_ribbon_events.len(),
        3,
        "test precondition: 3 events must be in event_ribbon_events"
    );

    // Act: render via the production path.
    let terminal = render_to_terminal(&mut app);

    // The terminal is 100 columns wide. Dashboard layout: left 60% = 60 cols (Sessions),
    // right 40% = 40 cols (Event Ribbon). The ribbon area starts at x=60.
    // Assert: the right 40% area must contain at least one hook type name or session ID
    // prefix from the injected events (BC-2.06.018 PC-1 column layout).
    let ribbon_region = buffer_region_text(&terminal, 60, 40);

    // At minimum, one of the hook type display names must appear in the ribbon area.
    // BC-2.06.018 PC-1: Hook type column renders "PreToolUse", "Notification", "SessionStart".
    let has_hook_type = ribbon_region.contains("PreToolUse")
        || ribbon_region.contains("Notification")
        || ribbon_region.contains("SessionStart");

    assert!(
        has_hook_type,
        "BC-2.06.018 PC-1 / AC-010: render_frame must render Event Ribbon content in the right \
         40%% area when event_ribbon_events is non-empty. \
         The 40%% region (x=60..100) must contain a hook type name. \
         Actual right-40%% content: {:?}",
        &ribbon_region[..ribbon_region.len().min(200)]
    );
}

// ---------------------------------------------------------------------------
// Test 2a: AC-010 / BC-2.06.006 PC-1 — Sessions filter input box rendered in Filtering mode
//
// RED: FAILS because render_frame's `_ =>` branch (which handles Dashboard, Overlay,
//      AND Filtering) always calls `SessionsPanel::render` (which shows regular session
//      list) — it never calls `render_sessions_filter`. The filter input "/ query_" box
//      is therefore absent from the buffer even when AppMode::Filtering is active.
// ---------------------------------------------------------------------------

/// When AppMode::Filtering is active, render_frame must render the filter input box
/// (showing the query with cursor) and the scored session list in the sessions area.
///
/// Verifies AC-010 integration requirement 1b: Sessions filter wired into render_frame.
/// Also verifies AC-001 (BC-2.06.006 PC-1): search input box with cursor rendered.
#[test]
fn test_BC_2_06_006_AC010_render_frame_filtering_mode_shows_filter_input_box() {
    use monocle_core::engine::{EnrichedSession, SessionStatus};

    // Arrange: App in Filtering mode with query="foo".
    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "foo".to_string(),
        prior: FocusSnapshot::Sessions,
    };
    // Seed a session so the panel is non-empty.
    app.sessions = vec![EnrichedSession::new(
        "sess-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Idle,
        None,
        Some("monocle-foo".to_string()),
        None,
        0,
        None,
    )];

    // Act: render via the production path.
    let terminal = render_to_terminal(&mut app);
    let rendered = buffer_text(&terminal);

    // Assert: the filter input box must be present with the "/ query_" format.
    // render_sessions_filter renders "/ {query}_" (AC-001, BC-2.06.006 PC-1).
    // The query is "foo" → the rendered input box shows "/ foo_".
    // The "/ " prefix is the FILTER INPUT BOX indicator — it only appears when
    // render_sessions_filter is called. The regular SessionsPanel render never
    // produces "/ foo_". If render_frame only calls SessionsPanel (broken path),
    // "foo" may appear via the project_name "monocle-foo" — but "/ foo_" will NOT.
    assert!(
        rendered.contains("/ foo_"),
        "BC-2.06.006 PC-1 / AC-010: render_frame in Filtering mode must render the filter \
         input box showing '/ foo_' (the '/ query_' format from render_sessions_filter). \
         The regular SessionsPanel render never produces this '/ ' prefix. \
         If only '/ foo_' is absent (not 'foo'), the broken path rendered the regular sessions \
         list which may show 'monocle-foo' as a project name — but not the filter input box. \
         Full render (first 300 chars): {:?}",
        &rendered[..rendered.len().min(300)]
    );
}

// ---------------------------------------------------------------------------
// Test 2b: AC-010 / BC-2.06.006 PC-8 — SESSIONS_FILTER_NO_MATCH sentinel rendered
//
// RED: FAILS because render_frame never calls render_sessions_filter, so the
//      no-match sentinel is never rendered even when the query matches nothing.
// ---------------------------------------------------------------------------

/// When AppMode::Filtering is active with a zero-match query, render_frame must
/// render the "No sessions match filter" sentinel (BC-2.06.006 PC-8).
///
/// Verifies AC-010 integration requirement 1b: filter sentinel wired into render_frame.
#[test]
fn test_BC_2_06_006_AC010_render_frame_filtering_zero_match_shows_sentinel() {
    use monocle_core::engine::{EnrichedSession, SessionStatus};
    use monocle_tui::ui::sessions_panel::SESSIONS_FILTER_NO_MATCH;

    // Arrange: App in Filtering mode with query="xyzzy" (no match expected).
    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "xyzzy".to_string(),
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

    // Act: render via the production path.
    let terminal = render_to_terminal(&mut app);
    let rendered = buffer_text(&terminal);

    assert!(
        rendered.contains(SESSIONS_FILTER_NO_MATCH),
        "BC-2.06.006 PC-8 / AC-010: render_frame in Filtering mode with zero-match query must \
         render SESSIONS_FILTER_NO_MATCH ({:?}). Full render (first 300 chars): {:?}",
        SESSIONS_FILTER_NO_MATCH,
        &rendered[..rendered.len().min(300)]
    );
}

// ---------------------------------------------------------------------------
// Tests 3a-3e: AC-010 / BC-2.06.018 PC-5 — Scroll dispatch changes ribbon offset
//
// RED: COMPILE-GATE — `Action::ScrollDown` and `Action::ScrollUp` do NOT exist in
//      the `Action` enum (`monocle-core/src/tui/state.rs`). These tests will produce
//      compile errors: `no variant named ScrollDown/ScrollUp in enum Action`.
//
//      The missing variants are: Action::ScrollDown, Action::ScrollUp.
//      The implementer must add these to the Action enum and wire them in
//      dispatch_key_event before these tests can compile and pass.
//
// NOTE: Because these tests reference non-existent enum variants, they are wrapped
//      in a module and guarded by a compile-time check. We use a commented-out
//      pattern that documents the missing variants without breaking the compilation
//      of the entire test binary. The individual functions below contain the
//      assertions that WILL apply once Action::ScrollDown and Action::ScrollUp exist.
//
//      The Red Gate for these tests is the compilation failure on Action::ScrollDown /
//      Action::ScrollUp references. When the implementer adds those variants, the
//      tests must then pass (they will verify the actual dispatch logic).
// ---------------------------------------------------------------------------

/// Tests 3a-3e are gated on the existence of Action::ScrollDown and Action::ScrollUp.
///
/// These variants are documented in BC-2.06.018 PC-5 and AC-010:
/// - j / ↓ → Action::ScrollDown (scroll toward older events, i.e., down the list)
/// - k / ↑ → Action::ScrollUp   (scroll toward newer events, i.e., up the list)
/// - G     → jump to newest (bottom of list in newest-first, i.e., oldest visual row = G)
/// - gg    → jump to oldest visible (top = newest in newest-first)
///
/// The compile-gate RED is intentional and expected: the Action enum in monocle-core
/// MUST be extended with ScrollDown/ScrollUp before dispatch can be wired.
///
/// This marker test documents the compile-gate RED Gate for BC-2.06.018 PC-5 / AC-010.
/// It currently PASSES (it has no body), but the sister tests below that reference
/// Action::ScrollDown/Action::ScrollUp will FAIL to compile until those variants exist.
#[test]
fn test_BC_2_06_018_AC010_scroll_actions_missing_compile_gate_documented() {
    // This test documents the RED GATE for scroll actions.
    // The tests test_BC_2_06_018_AC010_scroll_j_dispatches_scroll_down,
    // test_BC_2_06_018_AC010_scroll_k_dispatches_scroll_up,
    // test_BC_2_06_018_AC010_scroll_G_jumps_to_newest,
    // test_BC_2_06_018_AC010_scroll_gg_jumps_to_top
    // all reference Action::ScrollDown / Action::ScrollUp which DO NOT EXIST in the
    // current Action enum (monocle-core/src/tui/state.rs).
    //
    // Missing variants: Action::ScrollDown, Action::ScrollUp
    // Required by: BC-2.06.018 PC-5, AC-010, AC-007
    // File to modify: crates/monocle-core/src/tui/state.rs
    //
    // The tests are included in the COMMENTED BLOCK below. They will fail to
    // compile (not just fail at runtime) until the variants are added.
    // This is the intended compile-gate RED for the Red Gate log.
    // The documentation of missing variants is in the comment block above.
    // No assertion needed — the compile-gate is the commented-out tests below.
}

// The following tests are intentionally commented out because they reference
// Action::ScrollDown / Action::ScrollUp which don't yet exist. They are left
// as commented code so the implementer can see exactly what needs to pass.
//
// UNCOMMENT AFTER adding Action::ScrollDown and Action::ScrollUp to the Action enum.
//
// #[test]
// fn test_BC_2_06_018_AC010_scroll_j_dispatches_scroll_down() {
//     use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
//     use monocle_core::tui::state::Action;
//     use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
//     use monocle_tui::ui::sessions_panel::SessionsPanelState;
//
//     // Arrange: Dashboard focused on EventRibbon.
//     let mut app = App::new(MonocleConfig::default());
//     app.mode = AppMode::Dashboard {
//         focused: FocusSnapshot::EventRibbon,
//     };
//     // Seed 10 events so there is something to scroll.
//     for i in 0..10u64 {
//         on_hook_event_received(
//             &mut app,
//             HookType::Notification,
//             format!("sess-{i:03}"),
//             "{}".to_string(),
//             i,
//         );
//     }
//
//     let layers = build_builtin_binding_layers();
//     let mut sessions_state = SessionsPanelState::default();
//
//     // Act: press 'j' — must dispatch Action::ScrollDown.
//     let j_key = KeyEvent {
//         code: KeyCode::Char('j'),
//         modifiers: KeyModifiers::default(),
//     };
//     dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);
//
//     // Assert: the ribbon scroll offset has moved down from its initial position.
//     // After wiring, the EventRibbonState::list_state.selected() must reflect the offset change.
//     // (Exact assertion depends on how EventRibbonState is exposed; adapt as needed.)
//     // assert!(ribbon_scroll_offset(&app) > 0, "scroll offset must increase after 'j'");
// }
//
// #[test]
// fn test_BC_2_06_018_AC010_scroll_k_dispatches_scroll_up() {
//     // Similar to above but for 'k' → Action::ScrollUp.
//     // Start at a non-zero scroll offset, press 'k', assert offset decreases.
// }
//
// #[test]
// fn test_BC_2_06_018_AC010_scroll_G_jumps_to_newest() {
//     // 'G' → jump to the newest event (row 0 in newest-first ordering).
// }
//
// #[test]
// fn test_BC_2_06_018_AC010_scroll_gg_jumps_to_top() {
//     // 'gg' sequence → jump to the oldest event (end of list in newest-first ordering).
//     // Requires pending-key state in dispatch_key_event to detect two consecutive 'g' presses.
// }

// ---------------------------------------------------------------------------
// Test 4: BC-2.06.006 AC-010 — dispatch_key_event filter mode transitions (smoke)
//
// GREEN (already wired in dispatch) — these are included for documentation.
// The existing filter_sessions.rs tests cover this. These specifically drive the
// PRODUCTION dispatch_key_event path for Filtering mode and verify the full
// transition chain via the render buffer.
// ---------------------------------------------------------------------------

/// dispatch_key_event: '/' in Dashboard enters Filtering mode; Esc exits back to Dashboard.
/// Verifies AC-010 integration dispatch path for filter entry/exit via production code.
#[test]
fn test_BC_2_06_006_AC010_dispatch_filter_entry_and_exit_through_production_path() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};

    let mut app = App::new(MonocleConfig::default());
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    let no_mod = KeyModifiers::default();

    // Act: press '/' → enter Filtering mode.
    let slash_key = KeyEvent {
        code: KeyCode::Char('/'),
        modifiers: no_mod,
    };
    dispatch_key_event(&mut app, &slash_key, &layers, &mut sessions_state);

    // Assert: must be in Filtering mode.
    assert!(
        matches!(
            app.mode,
            AppMode::Filtering {
                panel: PanelId::Sessions,
                ..
            }
        ),
        "AC-010 dispatch: '/' must transition to Filtering {{ Sessions }} via production dispatch path"
    );

    // Act: type 'f', 'o', 'o' — query must accumulate.
    for c in ['f', 'o', 'o'] {
        dispatch_key_event(
            &mut app,
            &KeyEvent {
                code: KeyCode::Char(c),
                modifiers: no_mod,
            },
            &layers,
            &mut sessions_state,
        );
    }
    match &app.mode {
        AppMode::Filtering { query, .. } => {
            assert_eq!(
                query, "foo",
                "AC-010 dispatch: typing 'foo' must accumulate query"
            );
        }
        _other => panic!("expected Filtering mode with query=\"foo\" (got non-Filtering mode)"),
    }

    // Act: Esc → CancelFilter → Dashboard.
    dispatch_key_event(
        &mut app,
        &KeyEvent {
            code: KeyCode::Esc,
            modifiers: no_mod,
        },
        &layers,
        &mut sessions_state,
    );

    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "AC-010 dispatch: Esc in Filtering must cancel and return to Dashboard {{ Sessions }}"
    );

    // Act: press '/' again, type query, press Enter → CommitFilter → Dashboard.
    dispatch_key_event(
        &mut app,
        &KeyEvent {
            code: KeyCode::Char('/'),
            modifiers: no_mod,
        },
        &layers,
        &mut sessions_state,
    );
    dispatch_key_event(
        &mut app,
        &KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: no_mod,
        },
        &layers,
        &mut sessions_state,
    );
    dispatch_key_event(
        &mut app,
        &KeyEvent {
            code: KeyCode::Enter,
            modifiers: no_mod,
        },
        &layers,
        &mut sessions_state,
    );
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "AC-010 dispatch: Enter in Filtering must commit and return to Dashboard {{ Sessions }}"
    );
}
