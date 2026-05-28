#![allow(non_snake_case)]
#![allow(clippy::expect_used)]
#![allow(clippy::type_complexity)]
//! S-024: Failing tests for the TUI state machine — `transition()`, `AppMode`, `FocusSnapshot`.
//!
//! Every test in this file corresponds to a clause from BC-2.06.001, BC-2.06.002, or
//! BC-2.06.003 §transition(). Tests are named `test_BC_S_SS_NNN_xxx` for full traceability.
//!
//! All calls into `transition()`, `FocusSnapshot::cycle()`, and `FocusSnapshot::to_panel_id()`
//! use `std::panic::catch_unwind` + `AssertUnwindSafe` because the stubs contain `todo!()`.
//! The catch-unwind wrapper converts a `todo!()` panic into an explicit test failure with a
//! clear message, satisfying the Red Gate requirement: tests must FAIL on assertions, not
//! silently pass.
#![allow(clippy::panic)]

use monocle_core::tui::state::{Action, AppMode, FocusSnapshot, PanelId, PromptModal, ToolPayload};
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Construct a minimal `PromptModal` for tests that need to push/pop overlays.
/// `PromptModal` contains `received_at: Instant` which is `!UnwindSafe`, so
/// callers must wrap the containing value in `AssertUnwindSafe` before passing
/// to `catch_unwind`.
fn make_test_modal() -> PromptModal {
    PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "test-session-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_payload: ToolPayload::Bash {
            command: "echo hello".to_string(),
        },
        received_at: Instant::now(),
    }
}

/// Construct a second distinct `PromptModal` for multi-item stack tests.
fn make_test_modal_2() -> PromptModal {
    PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "test-session-002".to_string(),
        tool_name: "Write".to_string(),
        tool_payload: ToolPayload::Edit {
            old_content: "old".to_string(),
            new_content: "new".to_string(),
            path: PathBuf::from("/tmp/test.rs"),
        },
        received_at: Instant::now(),
    }
}

/// Call `transition()` inside `catch_unwind`, converting a `todo!()` panic
/// into an explicit assertion failure. Returns the resulting `AppMode` if the
/// function returns, or panics with a descriptive message if `todo!()` fires.
macro_rules! call_transition {
    ($mode:expr, $action:expr) => {{
        let mode = $mode;
        let action = $action;
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            monocle_core::tui::state::transition(mode, action)
        }));
        result.unwrap_or_else(|_| {
            panic!(
                "transition() panicked (likely todo!()) — Red Gate: implementation not yet written"
            )
        })
    }};
}

// ===========================================================================
// AC-001 / BC-2.06.001 PC-1: AppMode enum variants exist and can be constructed
// ===========================================================================

/// BC-2.06.001 PC-1: Dashboard variant can be constructed.
#[test]
fn test_BC_2_06_001_dashboard_variant_constructs() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    // Force evaluation — the variant must exist to construct it, but we verify
    // it's actually the Dashboard variant by matching. This test passes at the
    // type level; it should pass once types are defined. Red Gate for transition
    // behavior is exercised in subsequent tests.
    match mode {
        AppMode::Dashboard { focused: _ } => {}
        _ => panic!("expected Dashboard variant"),
    }
}

/// BC-2.06.001 PC-1: Filtering variant can be constructed.
#[test]
fn test_BC_2_06_001_filtering_variant_constructs() {
    let mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: String::new(),
        prior: FocusSnapshot::Sessions,
    };
    match mode {
        AppMode::Filtering { .. } => {}
        _ => panic!("expected Filtering variant"),
    }
}

/// BC-2.06.001 PC-1: Overlay variant can be constructed.
///
/// F-S025-ADV2-HIGH-003: AppMode::Overlay no longer stores the modal stack.
/// The stack lives in App::overlay_stack. This test verifies the variant exists.
#[test]
fn test_BC_2_06_001_overlay_variant_constructs() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    match mode {
        AppMode::Overlay { .. } => {}
        _ => panic!("expected Overlay variant"),
    }
}

/// BC-2.06.001 PC-1: Fullscreen variant can be constructed.
#[test]
fn test_BC_2_06_001_fullscreen_variant_constructs() {
    let mode = AppMode::Fullscreen {
        panel: PanelId::EventRibbon,
        prior: FocusSnapshot::EventRibbon,
    };
    match mode {
        AppMode::Fullscreen { .. } => {}
        _ => panic!("expected Fullscreen variant"),
    }
}

// ===========================================================================
// AC-013 / BC-2.06.001 INV-1: AppMode is NOT #[non_exhaustive]
// Compile-time test: an exhaustive match with no `_` arm must compile.
// ===========================================================================

/// BC-2.06.001 INV-1 / AC-013: Exhaustive match over AppMode compiles without `_`.
///
/// If AppMode were `#[non_exhaustive]`, this function would fail to compile,
/// which would fail the entire test binary. The absence of a `_` arm is the assertion.
#[test]
fn test_BC_2_06_001_appmode_exhaustive_match_compiles_without_wildcard() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    // Exhaustive match — no `_` arm. This MUST compile.
    // If AppMode were #[non_exhaustive], the compiler would reject this.
    let _variant_name = match mode {
        AppMode::Dashboard { .. } => "Dashboard",
        AppMode::Filtering { .. } => "Filtering",
        AppMode::Overlay { .. } => "Overlay",
        AppMode::Fullscreen { .. } => "Fullscreen",
    };
    // This assertion will fail once transition() is implemented but this particular
    // match compiles — so this test verifies the compile-time property, not runtime.
    // The test itself passes, but it documents the INV-1 requirement.
}

// ===========================================================================
// AC-002 / BC-2.06.002 Pre-1: FocusSnapshot derives Clone, PartialEq, Eq, Debug
// ===========================================================================

/// BC-2.06.002 Pre-2: FocusSnapshot derives Clone.
#[test]
fn test_BC_2_06_002_focus_snapshot_clone() {
    let a = FocusSnapshot::Sessions;
    let b = a.clone();
    assert_eq!(a, b, "FocusSnapshot must derive Clone and PartialEq");
}

/// BC-2.06.002 Pre-2: FocusSnapshot derives PartialEq and Eq.
#[test]
fn test_BC_2_06_002_focus_snapshot_eq() {
    assert_eq!(FocusSnapshot::Sessions, FocusSnapshot::Sessions);
    assert_ne!(FocusSnapshot::Sessions, FocusSnapshot::EventRibbon);
}

/// BC-2.06.002 Pre-2: FocusSnapshot derives Debug (format must not panic).
#[test]
fn test_BC_2_06_002_focus_snapshot_debug() {
    let s = format!("{:?}", FocusSnapshot::Sessions);
    assert!(!s.is_empty(), "Debug output must be non-empty");
    let e = format!("{:?}", FocusSnapshot::EventRibbon);
    assert!(!e.is_empty(), "Debug output must be non-empty");
}

// ===========================================================================
// AC-002 / BC-2.06.002: FocusSnapshot::cycle() — round-robin order
// ===========================================================================

/// BC-2.06.002 Pre-3 / AC-002: cycle() advances Sessions → EventRibbon.
#[test]
fn test_BC_2_06_002_focus_snapshot_cycle_sessions_to_event_ribbon() {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| FocusSnapshot::Sessions.cycle()));
    let next = result.unwrap_or_else(|_| {
        panic!("FocusSnapshot::cycle() panicked (likely todo!()) — Red Gate: not yet implemented")
    });
    assert_eq!(
        next,
        FocusSnapshot::EventRibbon,
        "Sessions.cycle() must return EventRibbon (round-robin)"
    );
}

/// BC-2.06.002 Pre-3 / EC-069: cycle() on EventRibbon wraps back to Sessions.
#[test]
fn test_BC_2_06_002_focus_snapshot_cycle_event_ribbon_wraps_to_sessions() {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| FocusSnapshot::EventRibbon.cycle()));
    let next = result.unwrap_or_else(|_| {
        panic!("FocusSnapshot::cycle() panicked (likely todo!()) — Red Gate: not yet implemented")
    });
    assert_eq!(
        next,
        FocusSnapshot::Sessions,
        "EventRibbon.cycle() must wrap back to Sessions (round-robin, Phase 1 has 2 panels)"
    );
}

/// BC-2.06.002 EC-069: Two cycle() calls return to the original focus (full round-trip).
#[test]
fn test_BC_2_06_002_focus_snapshot_cycle_full_round_trip() {
    let original = FocusSnapshot::Sessions;

    let step1 = std::panic::catch_unwind(AssertUnwindSafe(|| original.clone().cycle()))
        .unwrap_or_else(|_| panic!("cycle() panicked on first call"));
    let step2 = std::panic::catch_unwind(AssertUnwindSafe(|| step1.cycle()))
        .unwrap_or_else(|_| panic!("cycle() panicked on second call"));

    assert_eq!(
        step2, original,
        "Two cycle() calls must return to the original FocusSnapshot"
    );
}

// ===========================================================================
// AC-002 / BC-2.06.002: FocusSnapshot::to_panel_id() mapping
// ===========================================================================

/// BC-2.06.002 Pre-4 / AC-002: Sessions maps to PanelId::Sessions.
#[test]
fn test_BC_2_06_002_to_panel_id_sessions() {
    let result =
        std::panic::catch_unwind(AssertUnwindSafe(|| FocusSnapshot::Sessions.to_panel_id()));
    let panel_id = result.unwrap_or_else(|_| {
        panic!("to_panel_id() panicked (likely todo!()) — Red Gate: not yet implemented")
    });
    assert_eq!(
        panel_id,
        PanelId::Sessions,
        "Sessions.to_panel_id() must return PanelId::Sessions"
    );
}

/// BC-2.06.002 Pre-4 / AC-002: EventRibbon maps to PanelId::EventRibbon.
#[test]
fn test_BC_2_06_002_to_panel_id_event_ribbon() {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        FocusSnapshot::EventRibbon.to_panel_id()
    }));
    let panel_id = result.unwrap_or_else(|_| {
        panic!("to_panel_id() panicked (likely todo!()) — Red Gate: not yet implemented")
    });
    assert_eq!(
        panel_id,
        PanelId::EventRibbon,
        "EventRibbon.to_panel_id() must return PanelId::EventRibbon"
    );
}

// ===========================================================================
// AC-003 / BC-2.06.001 PC-3: PromptModal and ToolPayload variants construct
// ===========================================================================

/// BC-2.06.001 PC-3 / AC-003: PromptModal can be constructed with all required fields.
#[test]
fn test_BC_2_06_001_prompt_modal_constructs_with_all_fields() {
    let modal = PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "session-abc".to_string(),
        tool_name: "Read".to_string(),
        tool_payload: ToolPayload::Read {
            path: PathBuf::from("/etc/hosts"),
        },
        received_at: Instant::now(),
    };
    assert_eq!(modal.session_id, "session-abc");
    assert_eq!(modal.tool_name, "Read");
}

/// BC-2.06.001 PC-3 / AC-003: ToolPayload::Edit variant constructs correctly.
#[test]
fn test_BC_2_06_001_tool_payload_edit_constructs() {
    let payload = ToolPayload::Edit {
        old_content: "before".to_string(),
        new_content: "after".to_string(),
        path: PathBuf::from("/tmp/edit_target.rs"),
    };
    match payload {
        ToolPayload::Edit {
            old_content,
            new_content,
            path,
        } => {
            assert_eq!(old_content, "before");
            assert_eq!(new_content, "after");
            assert_eq!(path, PathBuf::from("/tmp/edit_target.rs"));
        }
        _ => panic!("expected Edit variant"),
    }
}

/// BC-2.06.001 PC-3 / AC-003: ToolPayload::Bash variant constructs correctly.
#[test]
fn test_BC_2_06_001_tool_payload_bash_constructs() {
    let payload = ToolPayload::Bash {
        command: "cargo test".to_string(),
    };
    match payload {
        ToolPayload::Bash { command } => assert_eq!(command, "cargo test"),
        _ => panic!("expected Bash variant"),
    }
}

/// BC-2.06.001 PC-3 / AC-003: ToolPayload::Read variant constructs correctly.
#[test]
fn test_BC_2_06_001_tool_payload_read_constructs() {
    let payload = ToolPayload::Read {
        path: PathBuf::from("/etc/passwd"),
    };
    match payload {
        ToolPayload::Read { path } => assert_eq!(path, PathBuf::from("/etc/passwd")),
        _ => panic!("expected Read variant"),
    }
}

/// BC-2.06.001 PC-3 / AC-003: ToolPayload::Generic variant constructs correctly.
#[test]
fn test_BC_2_06_001_tool_payload_generic_constructs() {
    let payload = ToolPayload::Generic {
        tool_name: "FutureTool".to_string(),
        tool_input: serde_json::json!({"key": "value"}),
    };
    match payload {
        ToolPayload::Generic {
            tool_name,
            tool_input,
        } => {
            assert_eq!(tool_name, "FutureTool");
            assert_eq!(tool_input["key"], "value");
        }
        _ => panic!("expected Generic variant"),
    }
}

// ===========================================================================
// AC-005 / BC-2.06.001 PC-3 / EC-060: Empty-stack collapse invariant
// ===========================================================================

/// BC-2.06.001 PC-3 / EC-060 / AC-005: PopOverlay always collapses to Dashboard (prior restored).
///
/// F-S025-ADV2-HIGH-003: transition() no longer has access to the overlay stack.
/// PopOverlay ALWAYS returns Dashboard { focused: prior }. The App-level handler is
/// responsible for re-entering Overlay if App::overlay_stack is still non-empty after
/// the pop. This preserves the empty-stack collapse invariant (AC-005) while keeping
/// transition() pure.
#[test]
fn test_BC_2_06_001_ac005_pop_overlay_always_collapses_to_dashboard() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::PopOverlay)
    }));
    let next = result.unwrap_or_else(|_| {
        panic!("transition() panicked — Red Gate: implementation not yet written")
    });
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::Sessions,
                "PopOverlay must restore prior FocusSnapshot (App-level decides re-entry)"
            );
        }
        _ => panic!("PopOverlay must always return Dashboard from transition() (F-S025-ADV2-HIGH-003)"),
    }
}

/// BC-2.06.001 PC-3 / EC-060 / AC-005: PopOverlay preserves prior FocusSnapshot regardless of
/// which prior was set.
///
/// F-S025-ADV2-HIGH-003: transition() always returns Dashboard { prior }; App-level handler
/// re-enters Overlay if overlay_stack non-empty. This test verifies prior is correctly preserved.
#[test]
fn test_BC_2_06_001_ac005_pop_overlay_preserves_prior() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::EventRibbon,
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::PopOverlay)
    }));
    let next = result.unwrap_or_else(|_| {
        panic!("transition() panicked — Red Gate: implementation not yet written")
    });
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::EventRibbon,
                "prior must be preserved through PopOverlay (EventRibbon case)"
            );
        }
        _ => panic!("PopOverlay must return Dashboard"),
    }
}

// ===========================================================================
// AC-006 / BC-2.06.003 PC-3: Filtering entry and exit
// ===========================================================================

/// BC-2.06.003 PC-3 / AC-006: StartFilter from Dashboard enters Filtering with empty query.
#[test]
fn test_BC_2_06_003_ac006_start_filter_enters_filtering_with_empty_query() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let next = call_transition!(
        mode,
        Action::StartFilter {
            panel: PanelId::Sessions
        }
    );
    match next {
        AppMode::Filtering {
            panel,
            query,
            prior,
        } => {
            assert_eq!(
                panel,
                PanelId::Sessions,
                "panel must match the StartFilter target"
            );
            assert!(query.is_empty(), "query must be empty on StartFilter");
            assert_eq!(
                prior,
                FocusSnapshot::Sessions,
                "prior must capture the Dashboard focus at StartFilter"
            );
        }
        _ => panic!("StartFilter from Dashboard must return Filtering"),
    }
}

/// BC-2.06.003 PC-3 / AC-006: CommitFilter from Filtering returns Dashboard with restored focus.
#[test]
fn test_BC_2_06_003_ac006_commit_filter_returns_dashboard_with_prior() {
    let mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "my-query".to_string(),
        prior: FocusSnapshot::EventRibbon,
    };
    let next = call_transition!(mode, Action::CommitFilter);
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::EventRibbon,
                "CommitFilter must restore the prior FocusSnapshot, not a hardcoded value"
            );
        }
        _ => panic!("CommitFilter must return Dashboard"),
    }
}

/// BC-2.06.003 PC-3 / AC-006: CancelFilter from Filtering returns Dashboard with restored focus.
#[test]
fn test_BC_2_06_003_ac006_cancel_filter_returns_dashboard_with_prior() {
    let mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "abandoned-query".to_string(),
        prior: FocusSnapshot::Sessions,
    };
    let next = call_transition!(mode, Action::CancelFilter);
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::Sessions,
                "CancelFilter must restore the prior FocusSnapshot"
            );
        }
        _ => panic!("CancelFilter must return Dashboard"),
    }
}

/// BC-2.06.002 PC-4: Filtering exit via Escape restores prior FocusSnapshot (using CancelFilter action).
/// Note: The BC specifies the escape action routes to CancelFilter semantics.
#[test]
fn test_BC_2_06_002_filtering_close_restores_prior_focus() {
    let prior = FocusSnapshot::EventRibbon;
    let mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "foo".to_string(),
        prior: prior.clone(),
    };
    // CancelFilter is the cancel-path out of Filtering per AC-006
    let next = call_transition!(mode, Action::CancelFilter);
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused, prior,
                "Filtering close must restore prior FocusSnapshot (BC-2.06.002 PC-4)"
            );
        }
        _ => panic!("CancelFilter must return Dashboard"),
    }
}

// ===========================================================================
// AC-007 / BC-2.06.003 PC-4: Fullscreen entry and exit
// ===========================================================================

/// BC-2.06.003 PC-4 / AC-007: EnterFullscreen from Dashboard captures focus.
#[test]
fn test_BC_2_06_003_ac007_enter_fullscreen_captures_focus() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let next = call_transition!(
        mode,
        Action::EnterFullscreen {
            panel: PanelId::Sessions
        }
    );
    match next {
        AppMode::Fullscreen { panel, prior } => {
            assert_eq!(
                panel,
                PanelId::Sessions,
                "panel must match EnterFullscreen target"
            );
            assert_eq!(
                prior,
                FocusSnapshot::Sessions,
                "prior must capture Dashboard focus at EnterFullscreen"
            );
        }
        _ => panic!("EnterFullscreen from Dashboard must return Fullscreen"),
    }
}

/// BC-2.06.003 PC-4 / AC-007: ExitFullscreen returns Dashboard with restored prior focus.
#[test]
fn test_BC_2_06_003_ac007_exit_fullscreen_restores_prior_focus() {
    let prior = FocusSnapshot::EventRibbon;
    let mode = AppMode::Fullscreen {
        panel: PanelId::EventRibbon,
        prior: prior.clone(),
    };
    let next = call_transition!(mode, Action::ExitFullscreen);
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused, prior,
                "ExitFullscreen must restore prior FocusSnapshot, not panel-derived focus"
            );
        }
        _ => panic!("ExitFullscreen must return Dashboard"),
    }
}

/// BC-2.06.002 PC-1: Fullscreen panel is NOT used to derive focused on exit.
/// This verifies that even when `panel != prior`, the `prior` is used.
#[test]
fn test_BC_2_06_002_fullscreen_exit_uses_prior_not_panel() {
    // EventRibbon panel, but Sessions was the prior focus
    let mode = AppMode::Fullscreen {
        panel: PanelId::EventRibbon,
        prior: FocusSnapshot::Sessions,
    };
    let next = call_transition!(mode, Action::ExitFullscreen);
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::Sessions,
                "ExitFullscreen must use prior (Sessions), not panel (EventRibbon)"
            );
        }
        _ => panic!("ExitFullscreen must return Dashboard"),
    }
}

// ===========================================================================
// AC-008 / BC-2.06.003 PC-5: Esc in Overlay is identity (no-op)
// ===========================================================================

/// BC-2.06.003 PC-5 / AC-008: Esc in Overlay returns same Overlay unchanged.
///
/// F-S025-ADV2-HIGH-003: AppMode::Overlay no longer stores the stack. The test
/// verifies that Esc is identity: mode stays Overlay and prior is preserved.
#[test]
fn test_BC_2_06_003_ac008_esc_in_overlay_is_identity() {
    let prior = FocusSnapshot::Sessions;
    let mode = AppMode::Overlay {
        prior: prior.clone(),
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::Esc)
    }));
    let next =
        result.unwrap_or_else(|_| panic!("transition() panicked — Red Gate: not yet implemented"));
    match next {
        AppMode::Overlay { prior: returned_prior } => {
            assert_eq!(
                returned_prior, prior,
                "Esc in Overlay must NOT change prior (identity transition)"
            );
        }
        AppMode::Dashboard { .. } => {
            panic!(
                "Esc in Overlay must NOT collapse to Dashboard (AC-008: Esc is no-op in Overlay)"
            )
        }
        _ => panic!("Esc in Overlay must return Overlay unchanged"),
    }
}

// ===========================================================================
// AC-009 / BC-2.06.003 PC-6: Overlay push/pop
// ===========================================================================

/// BC-2.06.003 PC-6 / AC-009: PushOverlay from Dashboard creates Overlay capturing prior focus.
///
/// F-S025-ADV2-HIGH-003: transition() only returns the mode change (Dashboard → Overlay).
/// The modal is pushed to App::overlay_stack by the App-level handler BEFORE calling
/// transition(). The transition() itself ignores the modal argument.
#[test]
fn test_BC_2_06_003_ac009_push_overlay_from_dashboard_creates_overlay() {
    let modal = make_test_modal();
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::PushOverlay { modal })
    }));
    let next =
        result.unwrap_or_else(|_| panic!("transition() panicked — Red Gate: not yet implemented"));
    match next {
        AppMode::Overlay { prior } => {
            assert_eq!(
                prior,
                FocusSnapshot::Sessions,
                "PushOverlay must capture the Dashboard focus as prior"
            );
        }
        _ => panic!("PushOverlay from Dashboard must return Overlay"),
    }
}

/// BC-2.06.003 PC-6 / AC-009: PushOverlay from existing Overlay is identity (mode stays Overlay,
/// prior preserved).
///
/// F-S025-ADV2-HIGH-003: App-level handler pushes to App::overlay_stack; transition() returns
/// Overlay with prior preserved. Stack item count is not observable from transition() alone.
#[test]
fn test_BC_2_06_003_ac009_push_overlay_from_overlay_stays_overlay() {
    let modal2 = make_test_modal_2();
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::EventRibbon,
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::PushOverlay { modal: modal2 })
    }));
    let next =
        result.unwrap_or_else(|_| panic!("transition() panicked — Red Gate: not yet implemented"));
    match next {
        AppMode::Overlay { prior } => {
            assert_eq!(
                prior,
                FocusSnapshot::EventRibbon,
                "PushOverlay from Overlay must preserve prior"
            );
        }
        _ => panic!("PushOverlay from Overlay must return Overlay with prior preserved"),
    }
}

/// BC-2.06.003 PC-6 / AC-009: PopOverlay from Overlay returns Dashboard with prior restored.
///
/// F-S025-ADV2-HIGH-003: transition(Overlay{prior}, PopOverlay) always returns
/// Dashboard{prior}. App-level handler: pops App::overlay_stack first; if stack is
/// still non-empty, re-enters Overlay. The stack-item-count collapse invariant is
/// enforced at the App level, not in transition().
#[test]
fn test_BC_2_06_003_ac009_pop_overlay_returns_dashboard_with_prior() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::PopOverlay)
    }));
    let next =
        result.unwrap_or_else(|_| panic!("transition() panicked — Red Gate: not yet implemented"));
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::Sessions,
                "PopOverlay must return Dashboard restoring prior (Sessions)"
            );
        }
        _ => panic!("PopOverlay must return Dashboard from transition()"),
    }
}

// ===========================================================================
// AC-014 / BC-2.06.001 INV-2: Purity boundary — monocle-core Cargo.toml
// ===========================================================================

/// BC-2.06.001 INV-2 / AC-014: monocle-core/Cargo.toml must NOT contain forbidden I/O deps.
///
/// Reads the actual Cargo.toml on disk and searches for forbidden dependency names.
/// Fails if any of: similar, nucleo, ratatui, crossterm appear as [dependencies] entries.
#[test]
fn test_BC_2_06_001_ac014_monocle_core_cargo_toml_has_no_forbidden_io_deps() {
    // Locate Cargo.toml relative to the monocle-core package root.
    // In integration tests, the cwd is the workspace root; we use CARGO_MANIFEST_DIR.
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo test");
    let cargo_toml_path = std::path::Path::new(&manifest_dir).join("Cargo.toml");
    let contents =
        std::fs::read_to_string(&cargo_toml_path).expect("Failed to read monocle-core/Cargo.toml");

    let forbidden = ["similar", "nucleo", "ratatui", "crossterm"];
    for dep in &forbidden {
        // Search in the [dependencies] section only — must not appear as a key
        assert!(
            !contents.contains(dep),
            "monocle-core/Cargo.toml must NOT contain '{dep}' (forbidden I/O dependency per AC-014 / BC-2.06.001 INV-2)",
        );
    }
}

// ===========================================================================
// AC-015 / BC-2.06.003 INV-1: transition() totality — no panic for any input
// ===========================================================================

/// BC-2.06.003 INV-1 / AC-015: transition() must not panic for a representative set of
/// (AppMode, Action) pairs. After implementation, all these should return valid AppModes.
/// Before implementation (Red Gate), all will panic with todo!(); the test fails on each.
#[test]
fn test_BC_2_06_003_ac015_transition_totality_dashboard_identity_actions() {
    // For each identity-or-noop action on Dashboard, transition() must return *an* AppMode
    // without panicking. We test a representative set.
    let noop_actions_on_dashboard: Vec<(&str, Box<dyn Fn() -> AppMode>)> = vec![
        (
            "MoveFocus",
            Box::new(|| {
                monocle_core::tui::state::transition(
                    AppMode::Dashboard {
                        focused: FocusSnapshot::Sessions,
                    },
                    Action::MoveFocus,
                )
            }),
        ),
        (
            "Noop",
            Box::new(|| {
                monocle_core::tui::state::transition(
                    AppMode::Dashboard {
                        focused: FocusSnapshot::Sessions,
                    },
                    Action::Noop,
                )
            }),
        ),
        (
            "Esc on Dashboard (identity)",
            Box::new(|| {
                monocle_core::tui::state::transition(
                    AppMode::Dashboard {
                        focused: FocusSnapshot::Sessions,
                    },
                    Action::Esc,
                )
            }),
        ),
        (
            "PermissionAcceptOnce on Dashboard (identity: EC-061)",
            Box::new(|| {
                monocle_core::tui::state::transition(
                    AppMode::Dashboard {
                        focused: FocusSnapshot::Sessions,
                    },
                    Action::PermissionAcceptOnce,
                )
            }),
        ),
    ];

    for (label, action_fn) in noop_actions_on_dashboard {
        let result = std::panic::catch_unwind(AssertUnwindSafe(action_fn));
        let _mode = result.unwrap_or_else(|_| {
            panic!(
                "transition() panicked for action '{label}' on Dashboard — Red Gate: not yet implemented"
            )
        });
        // If it returns, we accept any AppMode — totality just requires no panic.
    }
}

/// BC-2.06.001 EC-061 / AC-015: Unmatched (Dashboard, PermissionAcceptOnce) returns identity.
#[test]
fn test_BC_2_06_001_ec061_unmatched_action_returns_identity() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let next = call_transition!(mode, Action::PermissionAcceptOnce);
    // EC-061: identity transition — must return original mode unchanged
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::Sessions,
                "Unmatched (Dashboard, PermissionAcceptOnce) must return identity Dashboard"
            );
        }
        _ => panic!("Unmatched action on Dashboard must return identity (EC-061)"),
    }
}

// ===========================================================================
// BC-2.06.002 EC-065: OverlayCycleNext does not change prior
// ===========================================================================

/// BC-2.06.002 EC-065 / BC-2.06.002 PC-3: OverlayCycleNext preserves prior FocusSnapshot.
///
/// F-S025-ADV2-HIGH-003: transition() returns Overlay with prior unchanged; App-level
/// handler rotates App::overlay_stack. Stack-item count is not observable from transition().
#[test]
fn test_BC_2_06_002_ec065_overlay_cycle_next_preserves_prior() {
    let prior = FocusSnapshot::Sessions;
    let mode = AppMode::Overlay {
        prior: prior.clone(),
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::OverlayCycleNext)
    }));
    let next =
        result.unwrap_or_else(|_| panic!("transition() panicked — Red Gate: not yet implemented"));
    match next {
        AppMode::Overlay { prior: returned_prior } => {
            assert_eq!(
                returned_prior, prior,
                "OverlayCycleNext must NOT change prior (BC-2.06.002 PC-3)"
            );
        }
        _ => panic!("OverlayCycleNext must return Overlay"),
    }
}

// ===========================================================================
// BC-2.06.002 EC-065: Overlay close after OverlayCycleNext still uses original prior
// ===========================================================================

/// BC-2.06.002 EC-065: After OverlayCycleNext then PopOverlay, prior is preserved through both.
///
/// F-S025-ADV2-HIGH-003: OverlayCycleNext is identity from mode perspective.
/// PopOverlay always returns Dashboard with original prior.
#[test]
fn test_BC_2_06_002_ec065_overlay_close_after_cycle_uses_original_prior() {
    let prior = FocusSnapshot::Sessions;

    // Cycle: mode stays Overlay, prior preserved.
    let mode1 = AppMode::Overlay {
        prior: prior.clone(),
    };
    let after_cycle = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode1, Action::OverlayCycleNext)
    }))
    .unwrap_or_else(|_| panic!("transition(OverlayCycleNext) panicked — Red Gate"));

    // Pop: always Dashboard with original prior.
    let after_pop = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(after_cycle, Action::PopOverlay)
    }))
    .unwrap_or_else(|_| panic!("transition(PopOverlay) panicked — Red Gate"));

    match after_pop {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused, prior,
                "After OverlayCycleNext then PopOverlay, prior must be the original prior"
            );
        }
        _ => panic!("PopOverlay after OverlayCycleNext must return Dashboard"),
    }
}

// ===========================================================================
// BC-2.06.002 EC-066: Fullscreen from EventRibbon restores EventRibbon on exit
// ===========================================================================

/// BC-2.06.002 EC-066: Fullscreen entered from EventRibbon restores EventRibbon on Escape.
#[test]
fn test_BC_2_06_002_ec066_fullscreen_from_event_ribbon_restores_event_ribbon() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let fullscreen = call_transition!(
        mode,
        Action::EnterFullscreen {
            panel: PanelId::EventRibbon
        }
    );
    let restored = call_transition!(fullscreen, Action::ExitFullscreen);
    match restored {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::EventRibbon,
                "Fullscreen from EventRibbon must restore EventRibbon (EC-066)"
            );
        }
        _ => panic!("ExitFullscreen must return Dashboard"),
    }
}

// ===========================================================================
// BC-2.06.002 EC-068: Esc in Dashboard is identity
// ===========================================================================

/// BC-2.06.002 EC-068: Esc in Dashboard mode returns identity (same focused).
#[test]
fn test_BC_2_06_002_ec068_esc_in_dashboard_is_identity() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let next = call_transition!(mode, Action::Esc);
    match next {
        AppMode::Dashboard { focused } => {
            assert_eq!(
                focused,
                FocusSnapshot::Sessions,
                "Esc in Dashboard must be identity transition (EC-068)"
            );
        }
        _ => panic!("Esc in Dashboard must return Dashboard unchanged (EC-068)"),
    }
}

// ===========================================================================
// BC-2.06.001 PC-2: transition() is deterministic (same input → same output)
// ===========================================================================

/// BC-2.06.001 PC-2 / BC-2.06.003 INV-1 PC-5: transition() is deterministic.
/// Same (mode, action) pair called twice must produce equivalent outputs.
#[test]
fn test_BC_2_06_001_pc2_transition_is_deterministic() {
    // We test with CancelFilter which has a clearly specified output
    let make_mode = || AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "test".to_string(),
        prior: FocusSnapshot::Sessions,
    };

    let result1 = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(make_mode(), Action::CancelFilter)
    }));
    let result2 = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(make_mode(), Action::CancelFilter)
    }));

    let out1 = result1.unwrap_or_else(|_| panic!("First call panicked — Red Gate"));
    let out2 = result2.unwrap_or_else(|_| panic!("Second call panicked — Red Gate"));

    // Both must be Dashboard with Sessions focus (deterministic)
    match (out1, out2) {
        (AppMode::Dashboard { focused: f1 }, AppMode::Dashboard { focused: f2 }) => {
            assert_eq!(
                f1, f2,
                "transition() must be deterministic: same input → same output"
            );
        }
        _ => panic!("Both calls must return identical Dashboard — determinism violated"),
    }
}

// ===========================================================================
// BC-2.06.003 PC-6 / AC-009: PushOverlay from Filtering enters Overlay
// ===========================================================================

/// BC-2.06.003 PC-6 / AC-009: PushOverlay from Filtering mode creates Overlay (captures prior focus).
///
/// F-S025-ADV2-HIGH-003: transition() returns Overlay with prior from Filtering.
/// App-level handler has already pushed the modal to App::overlay_stack.
#[test]
fn test_BC_2_06_003_ac009_push_overlay_from_filtering_creates_overlay() {
    let modal = make_test_modal();
    let mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: "partial".to_string(),
        prior: FocusSnapshot::Sessions,
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::state::transition(mode, Action::PushOverlay { modal })
    }));
    let next =
        result.unwrap_or_else(|_| panic!("transition() panicked — Red Gate: not yet implemented"));
    match next {
        AppMode::Overlay { prior } => {
            assert_eq!(
                prior,
                FocusSnapshot::Sessions,
                "PushOverlay from Filtering must capture prior (Sessions)"
            );
        }
        _ => panic!("PushOverlay from Filtering must return Overlay"),
    }
}

// ===========================================================================
// BC-2.06.005 PC-2 / AC-006 / S-025: Action::MoveFocus cycles focus in Dashboard
// ===========================================================================
//
// The MoveFocus arm was missing from transition() at S-025 story start (cross-story
// gap). These tests document and guard the canonical two-panel tab order:
//   Sessions → EventRibbon → Sessions  (FocusSnapshot::cycle())
// per SS-tui.md §FocusSnapshot::cycle and BC-2.06.005 AC-006.

/// BC-2.06.005 PC-2 / AC-006: Dashboard { Sessions } + MoveFocus → Dashboard { EventRibbon }.
///
/// This is the canonical "Tab" binding in Dashboard mode — cycles focus to the
/// next panel in the two-panel round-robin order (Sessions → EventRibbon).
#[test]
fn test_BC_2_06_005_ac006_move_focus_sessions_to_event_ribbon() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let next = monocle_core::tui::state::transition(mode, Action::MoveFocus);
    match next {
        AppMode::Dashboard {
            focused: FocusSnapshot::EventRibbon,
        } => { /* expected */ }
        other => panic!(
            "MoveFocus from Sessions must yield Dashboard {{EventRibbon}}, got discriminant {:?}",
            core::mem::discriminant(&other)
        ),
    }
}

/// BC-2.06.005 PC-2 / AC-006: Dashboard { EventRibbon } + MoveFocus → Dashboard { Sessions }.
///
/// Completes the two-panel round-robin: EventRibbon → Sessions.
#[test]
fn test_BC_2_06_005_ac006_move_focus_event_ribbon_to_sessions() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let next = monocle_core::tui::state::transition(mode, Action::MoveFocus);
    match next {
        AppMode::Dashboard {
            focused: FocusSnapshot::Sessions,
        } => { /* expected */ }
        other => panic!(
            "MoveFocus from EventRibbon must yield Dashboard {{Sessions}}, got discriminant {:?}",
            core::mem::discriminant(&other)
        ),
    }
}

/// BC-2.06.005 PC-2 / AC-006: MoveFocus is idempotent over two applications —
/// after two Tab presses, focus returns to the original panel.
#[test]
fn test_BC_2_06_005_ac006_move_focus_round_trip_two_tabs() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let after_one = monocle_core::tui::state::transition(mode, Action::MoveFocus);
    let after_two = monocle_core::tui::state::transition(after_one, Action::MoveFocus);
    match after_two {
        AppMode::Dashboard {
            focused: FocusSnapshot::Sessions,
        } => { /* expected: round-trip restores Sessions */ }
        other => panic!(
            "Two MoveFocus applications must return to original focus; got discriminant {:?}",
            core::mem::discriminant(&other)
        ),
    }
}
