//! AC-010 integration render + dispatch tests for S-028.
//!
//! Drives `App::render_frame` (the production render path) against a
//! `ratatui::backend::TestBackend` and asserts that the rendered terminal buffer
//! contains Event Ribbon content and Sessions filter content respectively.
//!
//! Also drives `dispatch_key_event` for ribbon scroll key sequences.
//!
//! # Red Gate rationale (historical)
//!
//! - Test 1 (ribbon render in Dashboard): originally RED — render_frame did not call
//!   EventRibbon::render. GREEN after S-028 implementation wired the event ribbon area.
//! - Test 2a (Filtering mode layout): originally RED — render_frame did not call
//!   render_sessions_filter in Filtering mode. GREEN after S-028 implementation.
//! - Test 2b (zero-match sentinel): originally RED for the same reason. GREEN after S-028.
//! - Tests 3a-3e (scroll dispatch): originally compile-gate RED because Action::ScrollDown
//!   and Action::ScrollUp did not exist. GREEN after S-028 added and wired both variants.
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
// GREEN: Action::ScrollDown and Action::ScrollUp exist in the Action enum
//        (monocle-core/src/tui/state.rs) and are wired in dispatch_key_event
//        (monocle-tui/src/app.rs). Tests 3a-3e compile and pass.
//
//        'j' in Dashboard { EventRibbon } dispatches via the SelectNext arm, which
//        calls scroll_ribbon_down in EventRibbon focus (the dual-dispatch path).
//        'k' dispatches via SelectPrev which calls scroll_ribbon_up in EventRibbon focus.
//
// BC coverage: BC-2.06.018 PC-5 (j/k/G/gg scroll actions), AC-007, AC-010.
// ---------------------------------------------------------------------------

/// When the ribbon has focus (Dashboard { EventRibbon }) and the user presses `j`
/// or `↓`, `dispatch_key_event` dispatches `Action::ScrollDown`, moving the ribbon
/// scroll offset one row toward older events (down the newest-first list).
///
/// BC-2.06.018 PC-5 / AC-007 / AC-010.
///
/// GREEN: Action::ScrollDown exists and 'j' in Dashboard { EventRibbon } is
/// dispatched via the SelectNext arm which calls scroll_ribbon_down in EventRibbon focus.
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
         scroll from row 0 to row 1 (toward older events)."
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
/// GREEN: Action::ScrollDown exists and '↓' in Dashboard { EventRibbon } dispatches
/// via the SelectNext arm which calls scroll_ribbon_down in EventRibbon focus.
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
         from row 0 to row 1."
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
/// GREEN: 'j' in Dashboard { EventRibbon } dispatches via SelectNext which calls
/// scroll_ribbon_down in EventRibbon focus. Two presses advance the offset by 2.
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
         must advance ribbon scroll offset from row 0 to row 2."
    );
}

/// When the ribbon has focus and the user presses `k` or `↑`, `dispatch_key_event`
/// dispatches `Action::ScrollUp`, moving the ribbon scroll offset one row toward newer
/// events (up the newest-first list). At row 0, `pinned_top` is cleared.
///
/// BC-2.06.018 PC-5 / AC-007.
///
/// GREEN: Action::ScrollUp exists and 'k' in Dashboard { EventRibbon } dispatches
/// via SelectPrev which calls scroll_ribbon_up in EventRibbon focus.
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
         scroll from row 3 to row 2 (toward newer events)."
    );
}

/// When `k`/`↑` scrolls back to row 0 (newest), `pinned_top` must be cleared to
/// re-enable auto-scroll (BC-2.06.018 PC-5 / AC-008: scrolling to top resumes auto-scroll).
///
/// GREEN: Action::ScrollUp exists and 'k' dispatches via SelectPrev which calls
/// scroll_ribbon_up. Reaching row 0 clears pinned_top.
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
        "BC-2.06.018 PC-5: 'k' from row 1 must move to row 0 (newest)."
    );
    assert!(
        !app.event_ribbon_state.pinned_top,
        "BC-2.06.018 PC-5 / AC-008: when 'k' moves to row 0 (newest), pinned_top must \
         be cleared (auto-scroll re-enabled)."
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

// ---------------------------------------------------------------------------
// Tests 5a-5b: AC-007 / BC-2.06.018 PC-5 — G/gg jump-to-oldest / jump-to-newest
//
// RED: BEHAVIORAL FAILURE — `G` and `gg` have NO bindings in build_builtin_binding_layers().
//      There is also no pending-key state machine for composing the two-keystroke `gg`
//      sequence. Until the implementer adds:
//        1. A per-context binding `G` → Action::ScrollJumpOldest (or equivalent) for
//           Dashboard { EventRibbon } focus.
//        2. A per-context binding `g` → pending-key prefix, plus a second `g` press that
//           fires Action::ScrollJumpNewest (or equivalent).
//        3. The dispatch_key_event arms that handle the new actions.
//      pressing `G` or `g` resolves to None (no binding match) and the ribbon scroll
//      state is unchanged. The tests below assert the jump semantics and will fail
//      with assertion errors because the offset does not change.
//
// BC coverage: BC-2.06.018 PC-5 (G/gg jump navigation), AC-007.
// ---------------------------------------------------------------------------

/// In Dashboard { focused: EventRibbon }, pressing `G` must jump the ribbon selection
/// to the OLDEST event (last index in event_ribbon_events, bottom of newest-first list)
/// and set `pinned_top = true` (user is manually positioned away from newest).
///
/// BC-2.06.018 PC-5: G jumps to the oldest event row (bottom of the newest-first list).
/// AC-007: G/gg bindings are part of the per-context EventRibbon navigation set.
///
/// RED: behavioral failure — `G` has no binding in build_builtin_binding_layers().
/// Pressing `G` resolves to None; the ribbon selection does not change.
/// The assertion `selected() == Some(last_index)` fails because the selection stays
/// at the initial value (Some(0)), not the last index.
///
/// FIX: the implementer must register `G` as a per-context binding for Dashboard mode
/// with EventRibbon focus, and wire a new Action variant (e.g., ScrollJumpOldest) that
/// sets `list_state.select(Some(event_count - 1))` and `pinned_top = true`.
#[test]
fn test_BC_2_06_018_AC007_G_jumps_to_oldest_event() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    // Arrange: App in Dashboard { EventRibbon } with 5 events, scroll at row 0 (newest).
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
    // Start at row 0 (newest). pinned_top=false.
    app.event_ribbon_state.list_state.select(Some(0));
    app.event_ribbon_state.pinned_top = false;

    let event_count = app.event_ribbon_events.len();
    assert_eq!(event_count, 5, "precondition: 5 events in ribbon");

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Act: press 'G' (uppercase G) → should jump to oldest event (last index = 4).
    let big_g_key = KeyEvent {
        code: KeyCode::Char('G'),
        modifiers: KeyModifiers::default(),
    };
    dispatch_key_event(&mut app, &big_g_key, &layers, &mut sessions_state);

    // Assert: selection must be at the last index (oldest event = bottom of newest-first list).
    // BC-2.06.018 PC-5: G jumps to oldest (bottom = last index in newest-first ordering).
    // RED: 'G' has no binding → dispatch returns None → selection unchanged at Some(0).
    // The assertion fails because Some(0) != Some(4).
    let last_index = event_count.saturating_sub(1);
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(last_index),
        "BC-2.06.018 PC-5 / AC-007: 'G' in Dashboard {{ EventRibbon }} must jump \
         ribbon selection to the oldest event (last index={last_index} in a {event_count}-event \
         newest-first list). RED: 'G' has no binding in build_builtin_binding_layers(); \
         selection did not change. \
         FIX: register a per-context 'G' binding for Dashboard {{ EventRibbon }} focus \
         that jumps to last index and sets pinned_top=true."
    );
    assert!(
        app.event_ribbon_state.pinned_top,
        "BC-2.06.018 PC-5: after 'G' jump to oldest, pinned_top must be true \
         (user is manually positioned away from newest). \
         RED: 'G' has no binding; pinned_top was not set."
    );
}

/// In Dashboard { focused: EventRibbon }, pressing `gg` (two 'g' presses: first 'g'
/// enters a pending-key state, second 'g' fires the jump) must jump the ribbon selection
/// to the NEWEST event (row 0, top of newest-first list) and clear `pinned_top = false`
/// (auto-scroll resumes).
///
/// BC-2.06.018 PC-5: gg jumps to the newest event row (row 0 = top of newest-first list).
/// AC-007: G/gg bindings are part of the per-context EventRibbon navigation set.
///
/// RED: behavioral failure — `g` has no binding in build_builtin_binding_layers() and
/// there is no pending-key state machine for the `gg` two-keystroke sequence.
/// Two presses of `g` each resolve to None; the ribbon selection does not change.
/// The assertion `selected() == Some(0)` would pass trivially if we start at row 0,
/// so this test starts at a non-zero row to make the failure observable.
///
/// FIX: the implementer must:
///   1. Track a `pending_key: Option<char>` (or similar) in App or dispatch state.
///   2. On the first `g` press in Dashboard { EventRibbon } focus: set pending_key = Some('g').
///   3. On the second `g` press while pending_key == Some('g'): fire the jump to row 0,
///      clear pinned_top, and clear pending_key.
///   4. On any other key while pending_key == Some('g'): clear pending_key and handle
///      the key normally.
#[test]
fn test_BC_2_06_018_AC007_gg_jumps_to_newest_event() {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    use monocle_tui::app::{build_builtin_binding_layers, dispatch_key_event};
    use monocle_tui::ui::sessions_panel::SessionsPanelState;

    // Arrange: App in Dashboard { EventRibbon } with 5 events, scroll at row 4 (oldest).
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
    // Start at bottom (row 4 = oldest). pinned_top=true (user has scrolled to oldest).
    app.event_ribbon_state.list_state.select(Some(4));
    app.event_ribbon_state.pinned_top = true;

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let g_key = KeyEvent {
        code: KeyCode::Char('g'),
        modifiers: KeyModifiers::default(),
    };

    // Act: press 'g' twice → pending-key 'g' then fire jump-to-newest.
    // First 'g': should enter pending-key state (no visible selection change yet).
    dispatch_key_event(&mut app, &g_key, &layers, &mut sessions_state);
    // Second 'g': should fire the jump to row 0 (newest).
    dispatch_key_event(&mut app, &g_key, &layers, &mut sessions_state);

    // Assert: selection must be at row 0 (newest event = top of newest-first list).
    // BC-2.06.018 PC-5: gg jumps to newest (row 0).
    // RED: 'g' has no binding and no pending-key state machine exists;
    // both dispatches return None and the selection stays at Some(4).
    // The assertion fails because Some(4) != Some(0).
    assert_eq!(
        app.event_ribbon_state.list_state.selected(),
        Some(0),
        "BC-2.06.018 PC-5 / AC-007: 'gg' (two 'g' presses) in Dashboard {{ EventRibbon }} \
         must jump ribbon selection to the newest event (row 0 = top of newest-first list). \
         RED: 'g' has no binding and no pending-key state machine exists; \
         both 'g' presses resolve to None and selection stayed at Some(4). \
         FIX: implement pending-key state for 'g' in EventRibbon focus: \
         first 'g' enters prefix state, second 'g' fires jump to row 0 + clears pinned_top."
    );
    assert!(
        !app.event_ribbon_state.pinned_top,
        "BC-2.06.018 PC-5: after 'gg' jump to newest (row 0), pinned_top must be false \
         (auto-scroll resumes). RED: 'g' has no binding; pinned_top unchanged at true."
    );
}

// ---------------------------------------------------------------------------
// Test 6: F-W7G3-MED-001 — Event Ribbon shows the CORRECT session's events
//         in Filtering mode when the scored list is reordered relative to app.sessions.
//
// RED under the old code: sessions_state.list_state.selected() indexed into app.sessions
// (insertion order), but in Filtering mode that index refers to the scored list. A
// filter query that puts session B at position 0 and session A at position 1 caused
// the ribbon to show A's events (app.sessions[0]) while B was highlighted.
//
// GREEN under the fix: render_sessions_filter returns the session_id of the highlighted
// row in the scored list; render_frame passes it directly to EventRibbon as selected_sid.
//
// BC coverage: BC-2.06.018 PC-1 (ribbon shows events for the selected session),
//              BC-2.05.004 INV-3 (client-side filtering by session_id).
// ---------------------------------------------------------------------------

/// In Filtering mode with a query that reorders sessions (session B scores higher
/// than session A), the Event Ribbon must show events for the highlighted (filtered-top)
/// session B — NOT for app.sessions[0] (session A in insertion order).
///
/// This test would FAIL under the old buggy code: `app.sessions.get(list_state_index)`
/// uses the filtered-list index to index the unfiltered sessions, returning the wrong session.
/// It PASSES under the fix: render_sessions_filter returns the session_id of the
/// highlighted row in the scored list and render_frame passes it to EventRibbon directly.
#[test]
fn test_F_W7G3_MED_001_event_ribbon_shows_highlighted_session_events_in_filter_mode() {
    use monocle_core::engine::{EnrichedSession, SessionStatus};

    // Arrange: two sessions.
    // - sess-ALPHA: project "alpha-project" (inserted first → app.sessions[0])
    // - sess-ZETA: project "zeta-xyz" (inserted second → app.sessions[1])
    // The query "xyz" scores "zeta-xyz" much higher than "alpha-project", so the
    // scored list has sess-ZETA at index 0 (top) and sess-ALPHA at index 1 (or absent
    // if its score is 0). The user highlights the top row (index 0 in the scored list).
    //
    // Under the old buggy code: list_state.selected() == Some(0) → app.sessions.get(0)
    // returns sess-ALPHA → ribbon shows ALPHA events. WRONG.
    // Under the fix: render_sessions_filter returns "sess-ZETA" → ribbon shows ZETA events.

    let mut app = App::new(monocle_config::MonocleConfig::default());
    app.sessions = vec![
        EnrichedSession::new(
            "sess-ALPHA".to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Idle,
            None,
            Some("alpha-project".to_string()),
            None,
            0,
            None,
        ),
        EnrichedSession::new(
            "sess-ZETA".to_string(),
            "claude-code".to_string(),
            None,
            None,
            SessionStatus::Idle,
            None,
            Some("zeta-xyz".to_string()),
            None,
            0,
            None,
        ),
    ];

    // Seed distinct events: only sess-ZETA has a "SessionStart" event; sess-ALPHA
    // only has a "Notification" event. The ribbon content uniquely identifies the session.
    on_hook_event_received(
        &mut app,
        HookType::Notification,
        "sess-ALPHA".to_string(),
        r#"{"msg":"alpha-event"}"#.to_string(),
        1u64,
        0i64,
    );
    on_hook_event_received(
        &mut app,
        HookType::SessionStart,
        "sess-ZETA".to_string(),
        r#"{}"#.to_string(),
        2u64,
        0i64,
    );

    // Enter Filtering mode with query "xyz" — "zeta-xyz" matches; "alpha-project" does not
    // (or scores zero). The scored list will have sess-ZETA at position 0.
    app.mode = monocle_core::tui::state::AppMode::Filtering {
        panel: monocle_core::tui::state::PanelId::Sessions,
        query: "xyz".to_string(),
        prior: monocle_core::tui::state::FocusSnapshot::Sessions,
    };

    // Render via the production path. The terminal is 100x30.
    // We use a custom render helper that captures the sessions_state after render
    // so we can verify sessions_state.list_state picks up the right selection.
    // The key assertion is on the ribbon region content (right 40% = x=60..100).
    let backend = ratatui::backend::TestBackend::new(100, 30);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            // sessions_state with no pre-set selection — render_sessions_filter will
            // populate list_state after scoring. With one matching session (ZETA at 0),
            // ratatui leaves list_state with no auto-selection; we set it to Some(0)
            // before the second render to simulate the user highlighting the top result.
            let mut state = SessionsPanelState::default();
            state.list_state.select(Some(0)); // user highlights row 0 of the filter list
            render_frame(&mut app, &mut state, frame);
        })
        .unwrap();

    // The right 40% area (x=60..100) is the Event Ribbon.
    // Under the fix: selected_sid = "sess-ZETA" → ribbon shows "SessionStart" (ZETA's event).
    // Under the old bug: selected_sid = "sess-ALPHA" (app.sessions[0]) →
    //   ribbon shows "Notification" (ALPHA's event), NOT "SessionStart".
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    let ribbon_text: String = (0..area.height)
        .flat_map(|y| (60u16..100u16).map(move |x| (x, y)))
        .filter(|(x, _)| *x < area.width)
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        ribbon_text.contains("SessionStart"),
        "F-W7G3-MED-001: In Filtering mode with query 'xyz', sess-ZETA (project 'zeta-xyz') \
         is highlighted at filter-list position 0. The Event Ribbon must show sess-ZETA's \
         'SessionStart' event — NOT sess-ALPHA's 'Notification' event. \
         Old bug: app.sessions.get(list_state_index=0) returns sess-ALPHA (insertion order), \
         so the ribbon shows ALPHA's events while ZETA is highlighted. \
         Fix: render_sessions_filter returns 'sess-ZETA' directly (from scored[0]); \
         render_frame passes it to EventRibbon as selected_sid. \
         Actual right-40%% content (first 300 chars): {:?}",
        &ribbon_text[..ribbon_text.len().min(300)]
    );

    assert!(
        !ribbon_text.contains("Notification"),
        "F-W7G3-MED-001: The ribbon must NOT show sess-ALPHA's 'Notification' event \
         when sess-ZETA (zeta-xyz) is highlighted in the filter list. \
         Finding 'Notification' in the ribbon region means the old index-space bug is present. \
         Actual right-40%% content (first 300 chars): {:?}",
        &ribbon_text[..ribbon_text.len().min(300)]
    );
}
