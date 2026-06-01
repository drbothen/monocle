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
// GREEN: render_frame calls EventRibbon::render on layout.event_ribbon_area.
//        The right 40% area contains hook type names from the event ribbon.
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
        0i64,
    );
    on_hook_event_received(
        &mut app,
        HookType::Notification,
        "sess-xyz789ghi".to_string(),
        r#"{"msg":"hello"}"#.to_string(),
        7u64,
        0i64,
    );
    on_hook_event_received(
        &mut app,
        HookType::SessionStart,
        "sess-aaa000bbb".to_string(),
        r#"{}"#.to_string(),
        1u64,
        0i64,
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
// GREEN: render_frame calls render_sessions_filter when AppMode::Filtering is active.
//        The filter input box "/ foo_" is present in the rendered buffer.
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
// GREEN: render_frame calls render_sessions_filter in Filtering mode; when the query
//        has zero matches, SESSIONS_FILTER_NO_MATCH is rendered.
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
//      the `Action` enum (`monocle-core/src/tui/state.rs`). The tests below reference
//      these variants directly and will produce compile errors until the implementer
//      adds them:
//        error[E0599]: no variant named `ScrollDown` found for enum `Action`
//        error[E0599]: no variant named `ScrollUp` found for enum `Action`
//
//      The missing variants are: Action::ScrollDown, Action::ScrollUp.
//      The implementer must add these to the Action enum (crates/monocle-core/src/tui/state.rs)
//      and wire them in dispatch_key_event (crates/monocle-tui/src/app.rs) before these
//      tests can compile and pass.
//
// BC coverage: BC-2.06.018 PC-5 (j/k/G/gg scroll actions), AC-007, AC-010.
// ---------------------------------------------------------------------------

/// When the ribbon has focus (Dashboard { EventRibbon }) and the user presses `j`
/// or `↓`, `dispatch_key_event` dispatches `Action::ScrollDown`, moving the ribbon
/// scroll offset one row toward older events (down the newest-first list).
///
/// BC-2.06.018 PC-5 / AC-007 / AC-010.
///
/// RED: compile-gate — `Action::ScrollDown` does not exist in the Action enum
/// (`monocle-core/src/tui/state.rs`). The test will fail to compile until the
/// implementer adds the variant and wires it in dispatch_key_event.
#[test]
fn test_BC_2_06_018_AC010_scroll_j_dispatches_scroll_down() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    // Arrange: Dashboard focused on EventRibbon, scroll at row 0, 5 events loaded.
    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    for i in 0..5u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            format!("sess-{i:03}"),
            "{}".to_string(),
            i,
            0i64,
        );
    }
    app.event_ribbon_state.list_state.select(Some(0));
    app.event_ribbon_state.pinned_top = false;

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act: press 'j' → Action::ScrollDown (scroll one row toward older events).
    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: KeyModifiers::default(),
    };
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);

    // Assert: scroll offset moved from 0 to 1 (one row toward older events).
    // BC-2.06.018 PC-5: j/↓ scrolls toward older events (higher index in newest-first list).
    // pinned_top must be set to true (user manually scrolled away from newest).
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(1),
        "BC-2.06.018 PC-5 / AC-007: 'j' in Dashboard {{ EventRibbon }} must move ribbon \
         scroll from row 0 to row 1 (toward older events). \
         COMPILE-GATE: Action::ScrollDown missing in monocle-core/src/tui/state.rs."
    );
    assert!(
        app.event_ribbon_state.pinned_top,
        "BC-2.06.018 PC-5: after 'j' scroll, pinned_top must be true (user scrolled away from newest)"
    );
}

/// When the ribbon has focus and the user presses `↓`, `dispatch_key_event` dispatches
/// `Action::ScrollDown` (identical semantics to `j`).
///
/// BC-2.06.018 PC-5 / AC-007.
///
/// RED: compile-gate — `Action::ScrollDown` does not exist.
#[test]
fn test_BC_2_06_018_AC010_scroll_down_arrow_dispatches_scroll_down() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    for i in 0..5u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            format!("sess-{i:03}"),
            "{}".to_string(),
            i,
            0i64,
        );
    }
    app.event_ribbon_state.list_state.select(Some(0));
    app.event_ribbon_state.pinned_top = false;

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act: press ↓ → Action::ScrollDown.
    let down_key = KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::default(),
    };
    dispatch_key_event(&mut app, &down_key, &layers, &mut sessions_state);

    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(1),
        "BC-2.06.018 PC-5: ↓ in Dashboard {{ EventRibbon }} must move ribbon scroll \
         from row 0 to row 1. COMPILE-GATE: Action::ScrollDown missing."
    );
}

/// Starting at row 0 (newest), two consecutive `j` presses must advance the
/// ribbon scroll to row 2. This confirms that each 'j' increments the offset
/// and that the action is handled distinctly from the Sessions `SelectNext`.
///
/// This test serves as a prerequisite for the clamp test: it verifies that 'j'
/// moves the offset before we test that it stops at the boundary.
///
/// BC-2.06.018 PC-5 (j scrolls toward older events), AC-007.
///
/// RED: assertion failure — 'j' currently dispatches Action::SelectNext which
/// is a no-op in Dashboard {{ EventRibbon }} focus. The offset stays at Some(0)
/// instead of advancing to Some(2) after two presses.
#[test]
fn test_BC_2_06_018_AC010_scroll_j_twice_advances_two_rows() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    // 5 events so we can scroll twice without hitting the bottom.
    app.event_ribbon_panel_height = 5;
    for i in 0..5u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            format!("sess-{i:03}"),
            "{}".to_string(),
            i,
            0i64,
        );
    }
    // Start at top (row 0 = newest).
    app.event_ribbon_state.list_state.select(Some(0));
    app.event_ribbon_state.pinned_top = false;

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let j_key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: KeyModifiers::default(),
    };

    // Press 'j' twice.
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);
    dispatch_key_event(&mut app, &j_key, &layers, &mut sessions_state);

    // Assert: scroll offset must be Some(2) (moved 2 rows toward older events).
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(2),
        "BC-2.06.018 PC-5 / AC-007: two 'j' presses in Dashboard {{ EventRibbon }} \
         must advance ribbon scroll offset from row 0 to row 2. \
         Current: offset did not move (ScrollDown not wired; 'j' dispatches SelectNext \
         which is no-op in EventRibbon focus). \
         FIX: add Action::ScrollDown to the binding for 'j' in EventRibbon focus."
    );
}

/// When the ribbon has focus and the user presses `k` or `↑`, `dispatch_key_event`
/// dispatches `Action::ScrollUp`, moving the ribbon scroll offset one row toward newer
/// events (up the newest-first list). At row 0, `pinned_top` is cleared.
///
/// BC-2.06.018 PC-5 / AC-007.
///
/// RED: compile-gate — `Action::ScrollUp` does not exist in the Action enum.
#[test]
fn test_BC_2_06_018_AC010_scroll_k_dispatches_scroll_up() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    for i in 0..5u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            format!("sess-{i:03}"),
            "{}".to_string(),
            i,
            0i64,
        );
    }
    // Start at row 3 (user has scrolled down, pinned).
    app.event_ribbon_state.list_state.select(Some(3));
    app.event_ribbon_state.pinned_top = true;

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act: press 'k' → Action::ScrollUp.
    let k_key = KeyEvent {
        code: KeyCode::Char('k'),
        modifiers: KeyModifiers::default(),
    };
    dispatch_key_event(&mut app, &k_key, &layers, &mut sessions_state);

    // Assert: scroll moved from 3 to 2 (toward newer events).
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(2),
        "BC-2.06.018 PC-5 / AC-007: 'k' in Dashboard {{ EventRibbon }} must move ribbon \
         scroll from row 3 to row 2 (toward newer events). \
         COMPILE-GATE: Action::ScrollUp missing in monocle-core/src/tui/state.rs."
    );
}

/// When `k`/`↑` scrolls back to row 0 (newest), `pinned_top` must be cleared to
/// re-enable auto-scroll (BC-2.06.018 PC-5 / AC-008: scrolling to top resumes auto-scroll).
///
/// RED: compile-gate — `Action::ScrollUp` does not exist.
#[test]
fn test_BC_2_06_018_AC010_scroll_k_at_row0_clears_pinned_top() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    let mut app = App::new(MonocleConfig::default());
    app.mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    for i in 0..3u64 {
        on_hook_event_received(
            &mut app,
            HookType::Notification,
            format!("sess-{i:03}"),
            "{}".to_string(),
            i,
            0i64,
        );
    }
    // Start at row 1 (one below newest), pinned_top=true.
    app.event_ribbon_state.list_state.select(Some(1));
    app.event_ribbon_state.pinned_top = true;

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act: press 'k' from row 1 → moves to row 0.
    let k_key = KeyEvent {
        code: KeyCode::Char('k'),
        modifiers: KeyModifiers::default(),
    };
    dispatch_key_event(&mut app, &k_key, &layers, &mut sessions_state);

    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(0),
        "BC-2.06.018 PC-5: 'k' from row 1 must move to row 0 (newest). \
         COMPILE-GATE: Action::ScrollUp missing."
    );
    assert!(
        !app.event_ribbon_state.pinned_top,
        "BC-2.06.018 PC-5 / AC-008: when 'k' moves to row 0 (newest), pinned_top must \
         be cleared (auto-scroll re-enabled). COMPILE-GATE: Action::ScrollUp missing."
    );
}

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
