//! Integration tests for S-031 (ADV Pass-1 findings).
//!
//! Exercises the PRODUCTION dispatch_key_event / render_frame / startup paths
//! for the profile picker feature. Every test in this file MUST FAIL against the
//! current (pre-implementation) code — the Red Gate.
//!
//! # What is missing today (causing RED)
//!
//! 1. `build_builtin_binding_layers` has no Ctrl-P → `Action::ProfilePicker` entry.
//! 2. `dispatch_key_event` has no picker-open pre-check: picker-local keys (↓/↑, Enter,
//!    Esc) are not intercepted before `resolve_binding` runs.
//! 3. `render_frame` has no branch for `app.profile_picker.is_some()` — the picker
//!    widget is never called.
//! 4. `App::new` / `run()` startup does not call `detect_ccr(&config)` — `app.ccr_path`
//!    stays `None` even when a CCR could be detected.
//! 5. `render_status_bar` / `render_frame` do not pass `ccr_path` or call `ccr_path_text`.
//! 6. `open_profile_picker` uses `project_profiles.values().find_map(...)` (first-match
//!    over HashMap values, HashMap ordering is non-deterministic) instead of looking up
//!    the sticky entry for the SPECIFIC directory passed as `current_dir`.
//! 7. `commit_profile_selection` uses `.unwrap_or_default()` on the selected profile ID
//!    from an empty list → writes an empty string into `project_profiles`, violating
//!    BC-2.07.005 EC-106 (no write on empty profiles).
//! 8. `commit_profile_selection` has no write-failure seam: it always calls
//!    `MonocleConfig::config_path()` (which succeeds on a real machine) so there is no
//!    way to inject a path failure without a dedicated seam.
//! 9. No assertion that the CWD key used for the project_profiles write matches the key
//!    used for the read-side resolve_profile_for_dir lookup.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default,
    dead_code,
    unused_imports,
    unused_mut
)]

use monocle_config::{HarnessProfile, MonocleConfig};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_tui::app::{
    build_builtin_binding_layers, close_profile_picker, commit_profile_selection,
    commit_profile_selection_with_path, dispatch_key_event, open_profile_picker,
    open_profile_picker_with_dir, render_frame, App,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use ratatui::{backend::TestBackend, Terminal};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_config_with_profiles(ids: &[&str]) -> MonocleConfig {
    let mut config = MonocleConfig::default();
    for id in ids {
        config.harness_profiles.push(HarnessProfile {
            id: id.to_string(),
            display_name: format!("Profile {}", id),
            binary_path: "/usr/local/bin/claude".to_string(),
            config_dir: None,
        });
    }
    config
}

fn make_config_with_project_profiles(
    profile_ids: &[&str],
    project_entries: &[(&str, &str)],
) -> MonocleConfig {
    let mut config = make_config_with_profiles(profile_ids);
    for (dir, pid) in project_entries {
        config
            .project_profiles
            .insert(dir.to_string(), pid.to_string());
    }
    config
}

/// Collect all cell symbols from a `Terminal<TestBackend>` buffer as a single string.
fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

/// Collect a single row from the buffer as a string.
fn row_text(terminal: &Terminal<TestBackend>, row: u16) -> String {
    let buf = terminal.backend().buffer().clone();
    let width = buf.area().width;
    (0..width)
        .map(|x| buf[(x, row)].symbol().to_string())
        .collect()
}

/// Build a `monocle_core::tui::binding::KeyEvent` for Ctrl-P.
fn ctrl_p_key() -> monocle_core::tui::binding::KeyEvent {
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
    KeyEvent {
        code: KeyCode::Char('p'),
        modifiers: KeyModifiers {
            ctrl: true,
            shift: false,
            alt: false,
        },
    }
}

/// Build a bare (no-modifier) key event.
fn bare_key(code: monocle_core::tui::binding::KeyCode) -> monocle_core::tui::binding::KeyEvent {
    use monocle_core::tui::binding::{KeyEvent, KeyModifiers};
    KeyEvent {
        code,
        modifiers: KeyModifiers::default(),
    }
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.07.005 PC-1 (INTEGRATION-1)
//
// dispatch_key_event with Ctrl-P opens the picker in ALL AppModes.
//
// FAILS NOW: `build_builtin_binding_layers` does not register
// `Ctrl-P → Action::ProfilePicker` (no Global entry for that key+modifier combo).
// `dispatch_key_event` therefore resolves None for Ctrl-P and returns Continue
// without calling `open_profile_picker`. `app.profile_picker` remains None.
// ---------------------------------------------------------------------------

/// AC-010 / BC-2.07.005 PC-1 (RED):
/// Ctrl-P dispatched via `dispatch_key_event` opens the picker from Dashboard mode.
/// `app.profile_picker` must be `Some` after the dispatch.
/// The `App.mode` must remain `Dashboard { .. }` (BC-2.07.005 INV-1 — AppMode unchanged).
///
/// FAILS: `build_builtin_binding_layers` does not register Ctrl-P.
#[test]
fn test_BC_2_07_005_dispatch_ctrl_p_from_dashboard_opens_picker() {
    let config = make_config_with_profiles(&["cc", "cm"]);
    let mut app = App::new(config);
    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Precondition: app starts in Dashboard, picker is None.
    assert!(matches!(app.mode, AppMode::Dashboard { .. }));
    assert!(app.profile_picker.is_none());

    let outcome = dispatch_key_event(
        &mut app,
        &ctrl_p_key(),
        &binding_layers,
        &mut sessions_state,
    );

    assert!(
        app.profile_picker.is_some(),
        "BC-2.07.005 PC-1 / AC-010 / INTEGRATION-1: dispatch_key_event with Ctrl-P must open \
         the picker (set app.profile_picker = Some); FAILS because Ctrl-P is not registered in \
         build_builtin_binding_layers"
    );
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "BC-2.07.005 INV-1 / AC-001: AppMode must remain Dashboard after Ctrl-P"
    );
    // The outcome must be Continue (Ctrl-P does not quit).
    assert_eq!(
        outcome,
        monocle_tui::app::KeyOutcome::Continue,
        "Ctrl-P must return Continue (not Quit)"
    );
}

/// AC-010 / BC-2.07.005 INV-1 (RED):
/// Ctrl-P opens the picker from Overlay mode (picker fires in ALL AppModes — no guard).
///
/// FAILS: same root cause — Ctrl-P not registered.
#[test]
fn test_BC_2_07_005_dispatch_ctrl_p_from_overlay_opens_picker_appmode_unchanged() {
    use monocle_core::tui::state::PromptModal;
    use monocle_core::tui::state::ToolPayload;
    use monocle_tui::app::PermissionDecisionKind as _; // ensure ToolPayload visible
    use std::collections::VecDeque;
    use std::time::Instant;
    use uuid::Uuid;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);

    // Manually put app into Overlay mode with a permission prompt.
    let modal = PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "sess-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_payload: ToolPayload::Bash {
            command: "ls".to_string(),
        },
        received_at: Instant::now(),
    };
    app.overlay_stack.push_back(modal);
    app.mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };

    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(
        &mut app,
        &ctrl_p_key(),
        &binding_layers,
        &mut sessions_state,
    );

    assert!(
        app.profile_picker.is_some(),
        "BC-2.07.005 INV-1 / AC-010: Ctrl-P must open picker from Overlay mode; \
         FAILS because Ctrl-P is not registered"
    );
    // AppMode must remain Overlay — picker does not alter AppMode.
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.07.005 INV-1: AppMode must remain Overlay after Ctrl-P; picker does not change mode"
    );
}

/// AC-010 / BC-2.07.005 INV-1 (RED):
/// Ctrl-P opens the picker from Filtering mode (no AppMode guard).
///
/// FAILS: same root cause — Ctrl-P not registered.
#[test]
fn test_BC_2_07_005_dispatch_ctrl_p_from_filtering_opens_picker() {
    use monocle_core::tui::state::PanelId;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    app.mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        prior: FocusSnapshot::Sessions,
        query: String::new(),
    };

    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(
        &mut app,
        &ctrl_p_key(),
        &binding_layers,
        &mut sessions_state,
    );

    assert!(
        app.profile_picker.is_some(),
        "BC-2.07.005 INV-1 / AC-010: Ctrl-P must open picker from Filtering mode; \
         FAILS because Ctrl-P is not registered"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.07.005 PC-9 (INTEGRATION-3)
//
// While picker is open, dispatch_key_event routes picker-local keys to their
// handlers BEFORE resolve_binding runs.
//
// FAILS NOW: `dispatch_key_event` has no picker-open pre-check. When picker is open,
// pressing ↓ falls through to `resolve_binding → Action::SelectNext` and mutates
// `sessions_state` rather than calling `picker_select_next`. Esc goes to
// `transition(AppMode, Action::Esc)` instead of `close_profile_picker`. Enter fires
// `Action::EnterFullscreen` (from builtin) instead of `commit_profile_selection`.
// Keys unrelated to the picker (e.g., Tab) should NOT leak to the underlying mode.
// ---------------------------------------------------------------------------

/// AC-010 / BC-2.07.005 PC-9 (RED):
/// While picker is open, pressing ↓ calls `picker_select_next` (increments
/// `selected_index`) and does NOT mutate `sessions_state.list_state`.
///
/// FAILS: no picker pre-check → ↓ resolves to Action::SelectNext → sessions_state mutated.
#[test]
fn test_BC_2_07_005_picker_open_down_arrow_routes_to_picker_not_session_scroll() {
    use monocle_core::tui::binding::KeyCode;

    let config = make_config_with_profiles(&["aaa", "bbb", "ccc"]);
    let mut app = App::new(config);
    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Open picker — now app.profile_picker is Some.
    // NOTE: we call open_profile_picker directly (not via dispatch_key_event) because
    // INTEGRATION-1 is the test for Ctrl-P dispatch; here we focus on PC-9 isolation.
    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());

    let initial_picker_idx = app.profile_picker.as_ref().unwrap().selected_index;
    let initial_session_sel = sessions_state.list_state.selected();

    // Press ↓ while picker is open.
    dispatch_key_event(
        &mut app,
        &bare_key(KeyCode::Down),
        &binding_layers,
        &mut sessions_state,
    );

    // Picker selected_index must increment (picker consumed the event).
    let picker_idx_after = app.profile_picker.as_ref().unwrap().selected_index;
    assert_eq!(
        picker_idx_after,
        (initial_picker_idx + 1) % 3,
        "BC-2.07.005 PC-9 / AC-010 / INTEGRATION-3: ↓ while picker open must increment \
         picker.selected_index; FAILS because no picker pre-check exists"
    );

    // sessions_state must NOT have been touched by the ↓ keypress.
    assert_eq!(
        sessions_state.list_state.selected(),
        initial_session_sel,
        "BC-2.07.005 PC-9 / AC-004: ↓ while picker open must NOT mutate sessions_state; \
         FAILS because ↓ falls through to Action::SelectNext which mutates sessions_state"
    );
}

/// AC-010 / BC-2.07.005 PC-9 (RED):
/// While picker is open, pressing Esc calls `close_profile_picker` (sets picker to None)
/// and does NOT change AppMode.
///
/// FAILS: no picker pre-check → Esc resolves to Action::Esc → `transition(mode, Esc)`
/// which is an identity in Dashboard mode but does NOT close the picker.
#[test]
fn test_BC_2_07_005_picker_open_esc_routes_to_close_picker() {
    use monocle_core::tui::binding::KeyCode;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());

    dispatch_key_event(
        &mut app,
        &bare_key(KeyCode::Esc),
        &binding_layers,
        &mut sessions_state,
    );

    assert!(
        app.profile_picker.is_none(),
        "BC-2.07.005 PC-8 / AC-010 / INTEGRATION-3: Esc while picker open must set \
         app.profile_picker = None; FAILS because no picker pre-check exists"
    );
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "BC-2.07.005 PC-8: AppMode must remain Dashboard after Esc (picker-close, not mode-change)"
    );
}

/// AC-010 / BC-2.07.005 PC-5 (RED):
/// While picker is open, pressing Enter commits the selection (calls
/// `commit_profile_selection`), sets `app.profile_picker = None`, and does NOT
/// trigger `Action::EnterFullscreen`.
///
/// FAILS: no picker pre-check → Enter resolves to Action::EnterFullscreen from builtin.
#[test]
fn test_BC_2_07_005_picker_open_enter_commits_and_closes_picker() {
    use monocle_core::tui::binding::KeyCode;

    let mut config = make_config_with_profiles(&["cc"]);
    // Give it a writable CWD path for the commit.
    config
        .project_profiles
        .insert("/test/dir".to_string(), "cc".to_string());
    let mut app = App::new(config);
    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());
    // App must remain in Dashboard (not Fullscreen) after pressing Enter while picker is open.
    let mode_before = app.mode.clone();

    dispatch_key_event(
        &mut app,
        &bare_key(KeyCode::Enter),
        &binding_layers,
        &mut sessions_state,
    );

    assert!(
        app.profile_picker.is_none(),
        "BC-2.07.005 PC-5 / AC-010 / INTEGRATION-3: Enter while picker open must commit and \
         close picker (app.profile_picker = None); FAILS because no picker pre-check exists"
    );
    // Mode must NOT have changed to Fullscreen.
    assert!(
        !matches!(app.mode, AppMode::Fullscreen { .. }),
        "BC-2.07.005 PC-7: AppMode must NOT change to Fullscreen after Enter while picker open; \
         Enter was consumed by the picker, not by the builtin EnterFullscreen binding"
    );
    let _ = mode_before; // silence warning
}

/// AC-004 / BC-2.07.005 PC-9 (RED):
/// While picker is open, pressing Tab (a non-picker key) must NOT leak to the underlying
/// mode's handler. Tab normally fires Action::MoveFocus — while picker is open, the
/// picker consumes ALL keys, so focus must NOT change.
///
/// FAILS: no picker pre-check → Tab resolves to Action::MoveFocus and changes app.mode.
#[test]
fn test_BC_2_07_005_picker_open_tab_key_isolated_does_not_move_focus() {
    use monocle_core::tui::binding::KeyCode;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    let binding_layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    open_profile_picker(&mut app);
    let mode_before = app.mode.clone();

    // Tab normally fires Action::MoveFocus in any mode.
    dispatch_key_event(
        &mut app,
        &bare_key(monocle_core::tui::binding::KeyCode::Tab),
        &binding_layers,
        &mut sessions_state,
    );

    // While picker is open ALL keys are consumed by the picker.
    // Tab has no picker-local mapping → it is silently consumed (ignored).
    // The underlying mode (Dashboard focus etc.) must NOT change.
    let mode_after = app.mode.clone();
    assert!(
        matches!((&mode_before, &mode_after),
            (AppMode::Dashboard { focused: f1 }, AppMode::Dashboard { focused: f2 }) if f1 == f2),
        "BC-2.07.005 PC-9 / AC-004: Tab while picker open must not mutate AppMode focus; \
         FAILS because no picker pre-check — Tab leaks to Action::MoveFocus"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.07.005 PC-2 (INTEGRATION-2)
//
// render_frame renders the picker modal when app.profile_picker is Some.
//
// FAILS NOW: `render_frame` in `app.rs` has no branch for
// `app.profile_picker.is_some()`. `render_profile_picker` is never called.
// The picker title "Profile Picker" does NOT appear in the terminal buffer.
// ---------------------------------------------------------------------------

/// AC-010 / BC-2.07.005 PC-2 (RED):
/// When `app.profile_picker` is `Some`, `render_frame` must render the picker modal.
/// The title `"Profile Picker"` must appear in the terminal buffer.
///
/// FAILS: `render_frame` has no `profile_picker` branch.
#[test]
fn test_BC_2_07_005_render_frame_renders_picker_modal_when_picker_is_some() {
    let config = make_config_with_profiles(&["cc", "cm"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());

    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| render_frame(&mut app, &mut sessions_state, frame))
        .expect("render_frame must not panic");

    let all_text = buffer_text(&terminal);

    assert!(
        all_text.contains("Profile Picker"),
        "BC-2.07.005 PC-2 / AC-010 / INTEGRATION-2: render_frame must render the picker modal \
         title 'Profile Picker' when app.profile_picker is Some; \
         FAILS because render_frame has no profile_picker branch"
    );
}

/// AC-002 / BC-2.07.005 PC-3 (RED):
/// When picker is open with profiles, profile IDs must appear in the rendered buffer.
///
/// FAILS: same root cause — render_frame never calls render_profile_picker.
#[test]
fn test_BC_2_07_005_render_frame_picker_modal_shows_profile_list() {
    let config = make_config_with_profiles(&["alpha", "beta"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| render_frame(&mut app, &mut sessions_state, frame))
        .expect("render_frame must not panic");

    let all_text = buffer_text(&terminal);

    assert!(
        all_text.contains("alpha"),
        "BC-2.07.005 PC-2: profile 'alpha' must appear in picker modal; \
         FAILS because render_frame never calls render_profile_picker"
    );
    assert!(
        all_text.contains("beta"),
        "BC-2.07.005 PC-2: profile 'beta' must appear in picker modal; \
         FAILS because render_frame never calls render_profile_picker"
    );
}

/// AC-002 / BC-2.07.005 PC-3 (RED):
/// When picker is open with empty profiles, the no-profiles message must appear in the
/// rendered buffer with the EXACT canonical literal from BC-2.07.005 PC-3.
///
/// Canonical literal (BC-2.07.005 PC-3): "No profiles configured — add profiles to config.json"
/// (em-dash between "configured" and "add").
///
/// FAILS (Pass-6 ADV): production `NO_PROFILES_MSG` uses a period and different word order
/// ("No profiles configured. Edit config.json to add profiles.") — diverges from the BC spec
/// literal. Also fails if render_frame never calls render_profile_picker.
#[test]
fn test_BC_2_07_005_render_frame_picker_modal_shows_no_profiles_message() {
    use monocle_tui::ui::profile_picker_widget::NO_PROFILES_MSG;

    // The exact canonical literal mandated by BC-2.07.005 PC-3 / AC-002.
    // Em-dash (U+2014): "No profiles configured — add profiles to config.json"
    // Must NOT be weakened to a substring — must catch string drift.
    const SPEC_LITERAL: &str = "No profiles configured \u{2014} add profiles to config.json";

    let config = MonocleConfig::default();
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| render_frame(&mut app, &mut sessions_state, frame))
        .expect("render_frame must not panic");

    let all_text = buffer_text(&terminal);

    // Assert the EXACT canonical spec literal appears in the rendered buffer.
    // A substring-only check (e.g. "No profiles configured") is insufficient:
    // it cannot detect drift in the em-dash, word order, or trailing text.
    // This assertion intentionally FAILS until NO_PROFILES_MSG is aligned to
    // the BC-2.07.005 PC-3 canonical literal.
    assert!(
        all_text.contains(SPEC_LITERAL),
        "BC-2.07.005 PC-3 / AC-002: picker modal must render the EXACT spec literal \
         {:?} when harness_profiles is empty; \
         actual NO_PROFILES_MSG = {:?}; \
         rendered buffer did not contain the exact em-dash form. \
         Implementer: align NO_PROFILES_MSG to the BC-2.07.005 PC-3 canonical literal.",
        SPEC_LITERAL,
        NO_PROFILES_MSG
    );
}

// ---------------------------------------------------------------------------
// AC-010 / AC-007 / BC-2.07.005 PC-3 (INTEGRATION-4 + 5)
//
// App startup: ccr_path initialized, CCR path in status bar.
//
// FAILS NOW: `App::new` does not call `detect_ccr`. `app.ccr_path` is always
// `None` after construction even when detect_ccr would return Some. Additionally
// `render_status_bar` accepts no `ccr_path` parameter — `ccr_path_text` is never
// called from the render path.
// ---------------------------------------------------------------------------

/// AC-010 / AC-007 / BC-2.07.005 PC-3 (RED):
/// After App construction, `app.ccr_path` must be set by calling `detect_ccr(&config)`.
/// It must NOT be permanently None due to `App::new` failing to call detect_ccr.
///
/// This test is structurally robust against the "accidentally equal" tautology trap:
/// we set `config.ccr_path = Some("/bin/sh")` — a binary that EXISTS on all POSIX
/// systems. `detect_ccr` must return `Some(PathBuf::from("/bin/sh"))` when it respects
/// the `config.ccr_path` override (existence validated by detect_ccr per BC-2.07.006).
/// `App::new` must call `detect_ccr` and store `Some(PathBuf::from("/bin/sh"))` in
/// `app.ccr_path`. The current `App::new` hardcodes `ccr_path: None` — assertion fails.
///
/// FAILS: `App::new` never calls `detect_ccr`, so `app.ccr_path` is always `None`.
#[test]
fn test_BC_2_07_005_startup_ccr_path_initialized_from_detect_ccr() {
    // /bin/sh is present on all POSIX systems. detect_ccr with config.ccr_path = Some("/bin/sh")
    // must return Some(PathBuf::from("/bin/sh")).
    let mut config = MonocleConfig::default();
    config.ccr_path = Some("/bin/sh".to_string());

    // Verify the precondition: detect_ccr with this config returns Some.
    let ccr_from_detect = monocle_config::detect_ccr(&config);
    assert!(
        ccr_from_detect.is_some(),
        "Test precondition: detect_ccr with config.ccr_path = '/bin/sh' must return Some \
         (the binary exists); if this precondition fails the test environment is non-standard"
    );

    // App::new must call detect_ccr and store the result.
    // Current implementation hardcodes ccr_path: None — assertion fails.
    let app = App::new(config);

    assert_eq!(
        app.ccr_path, ccr_from_detect,
        "AC-010 / AC-007 / BC-2.07.005 PC-3: App::new must initialize app.ccr_path via \
         detect_ccr(&config). Expected Some('/bin/sh'), got {:?}. \
         FAILS because App::new hardcodes ccr_path: None without calling detect_ccr.",
        app.ccr_path
    );
}

/// AC-007 / BC-2.07.005 PC-3 (RED):
/// `render_frame` must display `"CCR: <path>"` or `"CCR: none"` somewhere in the
/// rendered terminal buffer (in the status bar area).
///
/// FAILS: `render_status_bar` does not accept a `ccr_path` parameter and does not
/// call `ccr_path_text`. The rendered buffer never contains "CCR:".
#[test]
fn test_BC_2_07_005_render_frame_status_bar_shows_ccr_path_when_some() {
    let mut config = MonocleConfig::default();
    // Set an explicit CCR path so detect_ccr can return it (if the implementer adds startup init).
    config.ccr_path = Some("/fake/ccr".to_string());
    let mut app = App::new(config);
    // Manually set ccr_path to Some so render_frame can read it, even if startup init is absent.
    app.ccr_path = Some(std::path::PathBuf::from("/fake/ccr"));

    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| render_frame(&mut app, &mut sessions_state, frame))
        .expect("render_frame must not panic");

    let all_text = buffer_text(&terminal);

    assert!(
        all_text.contains("CCR:"),
        "AC-007 / BC-2.07.005 PC-3 / AC-010 / INTEGRATION-5: render_frame must display \
         'CCR: <path>' in the status bar when app.ccr_path is Some; \
         FAILS because render_status_bar does not accept ccr_path and never calls ccr_path_text"
    );
}

/// AC-007 / BC-2.07.005 PC-3 (RED):
/// When `app.ccr_path` is `None`, the status bar must render `"CCR: none"`.
///
/// FAILS: same root cause — render_status_bar does not call ccr_path_text.
#[test]
fn test_BC_2_07_005_render_frame_status_bar_shows_ccr_none_when_absent() {
    let config = MonocleConfig::default();
    let mut app = App::new(config);
    // ccr_path is already None (default).
    assert!(app.ccr_path.is_none());

    let mut sessions_state = SessionsPanelState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| render_frame(&mut app, &mut sessions_state, frame))
        .expect("render_frame must not panic");

    let all_text = buffer_text(&terminal);

    assert!(
        all_text.contains("CCR: none"),
        "AC-007 / BC-2.07.005 PC-3 / INTEGRATION-5: render_frame must display 'CCR: none' in \
         the status bar when app.ccr_path is None; \
         FAILS because render_status_bar never calls ccr_path_text"
    );
}

// ---------------------------------------------------------------------------
// REAL DEFECT: MAJOR-1 / BC-2.07.005 PC-4
//
// Per-directory pre-selection must use resolve_profile_for_dir(config, current_dir)
// — NOT a first-match across all project_profiles.values().
//
// FAILS NOW: `open_profile_picker` scans `project_profiles.values()` without
// consulting `current_dir`. HashMap value iteration order is non-deterministic.
// With two entries `/a→profileX` and `/b→profileY`, opening the picker FOR /b
// may pre-select profileX (wrong) depending on HashMap iteration order.
// The CORRECT behavior requires passing `current_dir` to `open_profile_picker`.
// ---------------------------------------------------------------------------

/// MAJOR-1 / BC-2.07.005 PC-4 / BC-2.07.004 INV-1 (RED):
/// Per-directory preselection requires open_profile_picker to accept current_dir.
///
/// This test asserts the BEHAVIORAL CONSEQUENCE of the defect:
/// With two project_profiles entries for DIFFERENT directories with DIFFERENT profiles,
/// opening the picker twice (for different directories) must yield DIFFERENT selected_index
/// values. With the current first-match implementation, BOTH calls return the same index
/// (whichever HashMap happens to return first) — identical regardless of intended directory.
///
/// Config:
///   "/dir-a" → "aaa" (sorted index 0)
///   "/dir-b" → "zzz" (sorted index 2)
///
/// The test opens once, records the index, closes, opens again. The CURRENT broken
/// implementation returns the same first-match index both times (same HashMap order).
/// The CORRECT implementation needs current_dir to return index 0 for /dir-a and
/// index 2 for /dir-b.
///
/// The assertion: after fixing MAJOR-1, calling open for "/dir-a" must return 0
/// and for "/dir-b" must return 2. Since the current API cannot differentiate, the
/// test asserts that index 0 is returned for "/dir-a" — which FAILS when first-match
/// consistently returns index 2 ("zzz") due to HashMap ordering on this machine.
///
/// NOTE TO IMPLEMENTER: Rename to `open_profile_picker_for_dir(app, current_dir)`.
#[test]
fn test_BC_2_07_005_per_directory_preselection_uses_current_dir_not_first_match() {
    // "/dir-a" → "aaa" (sorted index 0). "/dir-b" → "zzz" (sorted index 2).
    // With current first-match, both open calls return the same index (non-deterministic).
    // The correct behavior: /dir-a → index 0, /dir-b → index 2.
    let config = make_config_with_project_profiles(
        &["aaa", "mmm", "zzz"],
        &[
            ("/dir-a", "aaa"), // sorted index 0
            ("/dir-b", "zzz"), // sorted index 2
        ],
    );

    // Open FOR "/dir-a" — must yield index 0 ("aaa").
    let mut app = App::new(config.clone());
    open_profile_picker_with_dir(&mut app, "/dir-a");
    let idx_for_dir_a = app.profile_picker.as_ref().unwrap().selected_index;
    close_profile_picker(&mut app);

    // Open FOR "/dir-b" — must yield index 2 ("zzz").
    let mut app2 = App::new(config.clone());
    open_profile_picker_with_dir(&mut app2, "/dir-b");
    let idx_for_dir_b = app2.profile_picker.as_ref().unwrap().selected_index;

    // The per-directory implementation returns DIFFERENT indices for different dirs.
    assert_ne!(
        idx_for_dir_a, idx_for_dir_b,
        "MAJOR-1 / BC-2.07.005 PC-4: opening picker for '/dir-a' (should be index 0 = 'aaa') \
         and '/dir-b' (should be index 2 = 'zzz') must yield DIFFERENT selected_index values.",
    );

    // Precise per-directory assertions.
    assert_eq!(idx_for_dir_a, 0, "/dir-a must pre-select 'aaa' (index 0)");
    assert_eq!(idx_for_dir_b, 2, "/dir-b must pre-select 'zzz' (index 2)");
}

/// MAJOR-1 / BC-2.07.005 PC-4 (RED) — symmetric direction:
/// Opening picker FOR "/target" must pre-select "zzz" (sorted index 2 — LAST).
/// A DECOY entry at "/decoy" maps to "aaa" (sorted index 0 — FIRST).
///
/// Profiles sorted: ["aaa", "mmm", "zzz"] → aaa=0, mmm=1, zzz=2.
/// Entries: "/decoy"→"aaa" (index 0, decoy), "/target"→"zzz" (index 2, target).
///
/// Correct: index 2. First-match returns "aaa" (index 0) → assertion fails.
/// Test is deterministically RED.
#[test]
fn test_BC_2_07_005_per_directory_preselection_for_dir_a_selects_profile_x() {
    let config = make_config_with_project_profiles(
        &["aaa", "mmm", "zzz"],
        &[
            ("/decoy", "aaa"),  // decoy — sorted index 0
            ("/target", "zzz"), // target — sorted index 2
        ],
    );
    let mut app = App::new(config);

    // Open the picker FOR directory "/target" — must pre-select "zzz" (index 2).
    // After MAJOR-1 fix: open_profile_picker_with_dir uses resolve_profile_for_dir
    // keyed by current_dir, so "/target" → "zzz" (index 2). The /decoy entry is ignored.
    open_profile_picker_with_dir(&mut app, "/target");

    let state = app.profile_picker.as_ref().expect("picker must be Some");

    assert_eq!(
        state.selected_index, 2,
        "MAJOR-1 / BC-2.07.005 PC-4: opening picker for '/target' must pre-select 'zzz' \
         (index 2 after sort ['aaa','mmm','zzz']). \
         The /decoy entry must NOT influence the pre-selection."
    );
}

// ---------------------------------------------------------------------------
// REAL DEFECT: BC-2.07.005 EC-106
//
// Empty-list commit: with empty harness_profiles, pressing Enter on the empty
// picker must NOT write config and must NOT persist an empty-string project_profiles
// entry. Esc dismisses cleanly.
//
// FAILS NOW: `commit_profile_selection` uses
//   `.get(state.selected_index).cloned().unwrap_or_default()`
// where `unwrap_or_default()` on Option<String> returns "". Then it writes
//   config.project_profiles.insert(current_dir, "".to_string())
// and calls write_config. This is a data-corruption defect: the empty string
// doesn't match any profile ID, so the next resolve_profile_for_dir call returns
// None (EC-105) — but the config file now has a spurious `"": ""` or `"dir": ""`
// entry.
// ---------------------------------------------------------------------------

/// BC-2.07.005 EC-106 (RED):
/// With empty `harness_profiles`, pressing Enter (commit) on the picker must be
/// a no-op: it must NOT write `project_profiles[current_dir] = ""` and must NOT
/// call write_config.
///
/// Observable: after commit on empty picker, `project_profiles` must NOT contain
/// an entry for `current_dir`.
///
/// FAILS: `commit_profile_selection` writes `project_profiles[current_dir] = ""`
/// via `unwrap_or_default()`.
#[test]
fn test_BC_2_07_005_ec106_empty_profiles_commit_does_not_write_project_profiles_entry() {
    let config = MonocleConfig::default(); // empty harness_profiles
    let mut app = App::new(config);
    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());
    // Picker has empty profiles.
    assert!(app.profile_picker.as_ref().unwrap().profiles.is_empty());

    // Commit on empty picker.
    commit_profile_selection(&mut app, "/home/user/project");

    // After commit on empty picker:
    // 1. project_profiles MUST NOT have an entry for current_dir.
    let entry = app.config.project_profiles.get("/home/user/project");
    assert!(
        entry.is_none(),
        "BC-2.07.005 EC-106: commit on empty picker must NOT write project_profiles entry; \
         found entry {:?}. FAILS because unwrap_or_default() writes empty string.",
        entry
    );
}

/// BC-2.07.005 EC-106 (RED) — Esc on empty picker must close cleanly without write.
#[test]
fn test_BC_2_07_005_ec106_empty_profiles_esc_closes_without_write() {
    let config = MonocleConfig::default();
    let mut app = App::new(config);
    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());

    close_profile_picker(&mut app);

    assert!(
        app.profile_picker.is_none(),
        "BC-2.07.005 EC-106: Esc on empty picker must close (set profile_picker = None)"
    );
    assert!(
        app.config.project_profiles.is_empty(),
        "BC-2.07.005 EC-106: Esc must not write project_profiles (project_profiles must remain empty)"
    );
}

// ---------------------------------------------------------------------------
// REAL DEFECT: BC-2.07.005 PC-5c / AC-006
//
// Write-failure path: when write_config fails, commit_profile_selection must
// STILL apply the in-memory selection AND set app.status_message = Some("Config save failed: ...").
// The persisted file must be unchanged.
//
// FAILS NOW: The current `commit_profile_selection` calls
//   MonocleConfig::config_path().and_then(|path| write_config(&app.config, &path))
// There is no seam to inject a failing path. Any test must force a write failure
// by injecting an unwritable path — but `commit_profile_selection` hardcodes the
// config_path() call.
//
// NOTE TO IMPLEMENTER: Add a `config_path_override: Option<&Path>` parameter to
// `commit_profile_selection` (or a separate `commit_profile_selection_with_path`).
// This enables tests to inject a failing path without using env vars or mocks.
// ---------------------------------------------------------------------------

/// BC-2.07.005 PC-5c / AC-006 (RED):
/// When write_config fails (config path under non-existent parent directory), the
/// in-memory profile selection IS applied and `app.status_message` is set to
/// `"Config save failed: ..."`. The picker closes. The caller's config file is
/// unchanged (write failed atomically).
///
/// FAILS: `commit_profile_selection` has no path seam. The hardcoded
/// `MonocleConfig::config_path()` succeeds on a developer's machine (HOME exists),
/// so the failure path is untestable without a seam. Even if config_path fails (HOME
/// not set), the test can't inject the specific error. Add a seam.
///
/// This test calls `commit_profile_selection` with the understanding that on most
/// machines write_config WILL succeed (writing to real config path). The test therefore
/// asserts the INTENT and will FAIL only once the seam exists and the path is
/// forced to fail. Before the seam exists, it will fail at compile-time (wrong arity)
/// or at runtime with write success (missing status_message).
///
/// Implementation note to the implementer: expose
///   `pub fn commit_profile_selection_with_path(app: &mut App, current_dir: &str, config_path: &Path)`
/// and have `commit_profile_selection` call it with `MonocleConfig::config_path()`.
/// This test will then call `commit_profile_selection_with_path` with a path under
/// `/nonexistent/parent/config.json` to force the write failure.
#[test]
fn test_BC_2_07_005_pc5c_write_failure_applies_in_memory_and_sets_status_message() {
    use std::path::Path;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());

    // Force write failure by injecting a path under a non-existent parent directory.
    // commit_profile_selection_with_path is the write-failure seam (AC-006 / BC-2.07.005 PC-5c).
    let bad_path = Path::new("/nonexistent_parent_dir_s031/monocle/config.json");
    commit_profile_selection_with_path(&mut app, "/home/user/project", bad_path);

    // After forced write failure:
    // 1. In-memory profile must have been applied (project_profiles entry set).
    let in_memory = app
        .config
        .project_profiles
        .get("/home/user/project")
        .cloned();
    assert!(
        in_memory.is_some(),
        "BC-2.07.005 PC-5c: in-memory profile must be set even on write failure"
    );

    // 2. app.status_message must contain "Config save failed:".
    // With the seam + bad_path: write fails → status_message is Some("Config save failed: ...").
    assert!(
        app.status_message
            .as_deref()
            .map(|m| m.starts_with("Config save failed:"))
            .unwrap_or(false),
        "BC-2.07.005 PC-5c / AC-006: on write_config failure, app.status_message must be \
         Some(\"Config save failed: <error>\"); got: {:?}",
        app.status_message
    );

    // 3. Picker must close.
    assert!(
        app.profile_picker.is_none(),
        "BC-2.07.005 PC-5c: picker must close after commit (even on write failure)"
    );
}

// ---------------------------------------------------------------------------
// REAL DEFECT: BLOCKER-2 / BC-2.07.004 INV-1 / BC-2.07.005 INV-5
//
// CWD source / normalization consistency: the key used for project_profiles write
// (commit_profile_selection) must match the key used for read-side lookup
// (resolve_profile_for_dir). Inconsistent normalization is a correctness defect.
//
// FAILS NOW: `commit_profile_selection` uses the `current_dir: &str` argument
// verbatim for the write key, BUT `open_profile_picker` computes pre-selection
// using first-match over `project_profiles.values()` (not keyed by current_dir).
// After the MAJOR-1 fix, pre-selection will use resolve_profile_for_dir(config, current_dir).
// This test asserts the ROUND-TRIP property: commit then resolve must agree on the key.
// ---------------------------------------------------------------------------

/// BLOCKER-2 / BC-2.07.004 INV-1 / BC-2.07.005 INV-5 (RED):
/// Assert the normalization round-trip AND per-directory pre-selection together.
///
/// This test is structured so that:
/// - The target directory "/my/project" maps to "zzz" (alphabetically LAST → index 2).
/// - A decoy directory "/decoy" maps to "aaa" (alphabetically FIRST → index 0).
///
/// Profiles sorted: ["aaa", "mmm", "zzz"] → "aaa"=0, "mmm"=1, "zzz"=2.
///
/// If `open_profile_picker` uses first-match over HashMap values(), the decoy entry
/// "aaa" (index 0) is likely to come first (HashMap non-deterministic, but "aaa" is
/// alphabetically first and many HashMap implementations happen to stabilize on it).
/// The correct per-dir lookup returns index 2 ("zzz") for "/my/project".
///
/// Steps:
/// 1. resolve_profile_for_dir(config, "/my/project") → Some("zzz"). (read-side correct)
/// 2. resolve_profile_for_dir(config, "/my/project/") → None. (trailing slash = miss)
/// 3. open_profile_picker for "/my/project" must pre-select index 2 ("zzz").
///    FAILS: first-match returns index 0 ("aaa" from "/decoy") because it does not
///    filter by the target directory.
///
/// FAILS: `open_profile_picker` has no `current_dir` parameter.
#[test]
fn test_BC_2_07_004_invariant_write_read_normalization_verbatim_consistent() {
    use monocle_config::resolve_profile_for_dir;

    // "/my/project" → "zzz" (sorted index 2)
    // "/decoy"      → "aaa" (sorted index 0) — the decoy that first-match returns wrong
    let config = make_config_with_project_profiles(
        &["aaa", "mmm", "zzz"],
        &[("/my/project", "zzz"), ("/decoy", "aaa")],
    );
    let mut app = App::new(config);

    // Step 1: read-side resolve must return the correct profile for the verbatim path.
    let found = resolve_profile_for_dir(&app.config, "/my/project");
    assert!(
        found.is_some(),
        "BLOCKER-2 / BC-2.07.004 INV-1: resolve_profile_for_dir('/my/project') must return Some"
    );
    assert_eq!(
        found.unwrap().id,
        "zzz",
        "resolve_profile_for_dir must return 'zzz' for '/my/project'"
    );

    // Step 2: trailing slash is a miss (verbatim normalization — BC-2.07.004 INV-1).
    let found_slash = resolve_profile_for_dir(&app.config, "/my/project/");
    assert!(
        found_slash.is_none(),
        "BC-2.07.004 INV-1: trailing slash mismatch must return None"
    );

    // Step 3: open picker for "/my/project" — must pre-select "zzz" (index 2).
    // After MAJOR-1 fix: open_profile_picker_with_dir uses resolve_profile_for_dir(config, "/my/project")
    // which returns the profile "zzz" → position 2 in sorted list ["aaa","mmm","zzz"].
    // The /decoy entry ("/decoy" → "aaa") is irrelevant because we look up by specific dir.
    open_profile_picker_with_dir(&mut app, "/my/project");

    let state = app.profile_picker.as_ref().expect("picker must be Some");
    assert_eq!(
        state.selected_index, 2,
        "BLOCKER-2 / BC-2.07.005 INV-5: opening picker for '/my/project' must pre-select \
         'zzz' (index 2 after sort ['aaa','mmm','zzz']). \
         The /decoy entry must NOT influence pre-selection."
    );
}

// ---------------------------------------------------------------------------
// STRENGTHEN: test_BC_2_07_005_active_profile_highlighted_as_default_selection
//
// The existing test in profile_switch.rs uses a single project_profiles entry:
//   { "/home/user/project" → "bbb" }
// with three profiles ["aaa", "bbb", "ccc"]. This masks the MAJOR-1 first-match
// defect because with a single entry there is no ambiguity in values().find_map().
//
// Replacement/augmentation: two project_profiles entries at DIFFERENT directories
// → two-entry test exercises MAJOR-1.
// ---------------------------------------------------------------------------

/// BC-2.07.005 PC-4 STRENGTHENED (RED):
/// With TWO project_profiles entries:
/// - "/decoy"  → "aaa" (sorted index 0 — alphabetically first; returned by first-match)
/// - "/target" → "zzz" (sorted index 2 — alphabetically last)
///
/// Opening picker "for /target" must pre-select index 2 ("zzz").
/// First-match returns "aaa" (index 0) from /decoy → assertion fails deterministically.
///
/// REPLACES the single-entry tautological test from profile_switch.rs which masked MAJOR-1.
/// FAILS: `open_profile_picker` uses first-match (no current_dir).
#[test]
fn test_BC_2_07_005_active_profile_highlighted_two_entry_per_dir_not_first_match() {
    // Profiles sorted: ["aaa", "bbb", "zzz"] → aaa=0, bbb=1, zzz=2.
    // Decoy: "/decoy" → "aaa" (index 0, alphabetically first — first-match returns this).
    // Target: "/target" → "zzz" (index 2, the correct result for /target).
    let config = make_config_with_project_profiles(
        &["aaa", "bbb", "zzz"],
        &[
            ("/decoy", "aaa"),  // decoy — index 0, first in sort
            ("/target", "zzz"), // target — index 2, correct for this dir
        ],
    );

    // Opening for "/target" must pre-select "zzz" (index 2).
    // After MAJOR-1 fix: open_profile_picker_with_dir respects current_dir.
    let mut app = App::new(config.clone());
    open_profile_picker_with_dir(&mut app, "/target");
    let state = app.profile_picker.as_ref().unwrap();
    assert_eq!(
        state.selected_index, 2,
        "BC-2.07.005 PC-4 (STRENGTHENED): opening picker for '/target' must pre-select 'zzz' \
         (index 2). The /decoy entry (/decoy → 'aaa') must NOT influence pre-selection."
    );
}
