//! Tests for BC-2.06.023 (UUID removal via retain(), duplicate removal, empty-stack collapse).
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]
//!
//! # Red Gate
//!
//! All tests in this file exercise the `retain()` behavior (direct VecDeque mutation),
//! `on_initial_state` (overlay restore), and `transition()` (collapse check). All
//! production code was pre-built by S-025. These are COVERAGE/characterization tests
//! expected to PASS immediately.
//!
//! # Coverage
//!
//! - BC-2.06.023 PC-1: `retain()` removes the matching modal regardless of stack position.
//! - BC-2.06.023 PC-2: `retain()` removes ALL entries matching the prompt_id (duplicate
//!   from reconnect race — though apply_permission_prompt_queued prevents duplicates,
//!   retain() handles them safely).
//! - BC-2.06.023 PC-3: Unknown prompt_id in PermissionPromptResolved → WARN logged, no-op.
//! - BC-2.06.023 PC-4 / AC-015: After retain() empties the stack, AppMode must not remain
//!   Overlay — empty stack triggers collapse to Dashboard { focused: prior }.
//! - Architecture Compliance: retain() is NOT routed through transition() — direct mutation
//!   followed by a post-retain collapse check.
//! - AC-015: After retain() of last item, the IPC handler must transition to Dashboard.

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{Action, AppMode, FocusSnapshot};
use monocle_ipc::types::PermissionPromptPayload;
use monocle_tui::app::{apply_permission_prompt_queued, on_initial_state, App};
use std::collections::VecDeque;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn bash_payload(prompt_id: Uuid) -> PermissionPromptPayload {
    PermissionPromptPayload {
        prompt_id,
        session_id: "sess-001".into(),
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({"command": "ls"}),
        old_content: None,
        new_content: None,
    }
}

fn default_app() -> App {
    App::new(MonocleConfig::default())
}

/// Simulate the full PermissionPromptResolved handler that will be implemented in S-026.
///
/// This is the pattern the S-026 implementer must follow (from BC-2.06.023 and AC-006):
///   1. retain() removes matching entry (by UUID, not position)
///   2. If stack is now empty and mode is Overlay, collapse to Dashboard { focused: prior }
///   3. If stack is non-empty, mode stays Overlay
///
/// Architecture Compliance Rule: retain() is NOT routed through transition().
/// The collapse (if needed) calls transition(Overlay { prior }, PopOverlay).
fn simulate_permission_prompt_resolved(app: &mut App, prompt_id: Uuid) {
    // Step 1: direct retain() mutation (NOT through transition())
    app.overlay_stack.retain(|m| m.prompt_id != prompt_id);

    // Step 2: collapse check
    if app.overlay_stack.is_empty() {
        if let AppMode::Overlay { prior } = app.mode.clone() {
            use monocle_core::tui::state::transition;
            app.mode = transition(AppMode::Overlay { prior }, Action::PopOverlay);
        }
    }
    // Step 3: if stack non-empty, mode stays Overlay — no change needed
}

// ---------------------------------------------------------------------------
// BC-2.06.023 PC-1 — retain() removes by UUID, regardless of position
// ---------------------------------------------------------------------------

/// BC-2.06.023 PC-1 (coverage): retain() removes the front entry. Stack shrinks by 1.
#[test]
fn test_BC_2_06_023_pc1_retain_removes_front_entry() {
    let mut overlay: VecDeque<_> = VecDeque::new();
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();

    apply_permission_prompt_queued(&mut overlay, bash_payload(pid1));
    apply_permission_prompt_queued(&mut overlay, bash_payload(pid2));

    overlay.retain(|m| m.prompt_id != pid1);

    assert_eq!(
        overlay.len(),
        1,
        "BC-2.06.023 PC-1: retain() must remove the front entry"
    );
    assert_eq!(
        overlay.front().unwrap().prompt_id,
        pid2,
        "BC-2.06.023 PC-1: P2 must now be at front after P1 removed"
    );
}

/// BC-2.06.023 PC-1 (coverage): retain() removes the back entry (not at front).
/// This validates position-independence — not just front() behavior.
#[test]
fn test_BC_2_06_023_pc1_retain_removes_back_entry() {
    let mut overlay: VecDeque<_> = VecDeque::new();
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();

    apply_permission_prompt_queued(&mut overlay, bash_payload(pid1));
    apply_permission_prompt_queued(&mut overlay, bash_payload(pid2));

    // Resolve P2 (the back entry, NOT front)
    overlay.retain(|m| m.prompt_id != pid2);

    assert_eq!(
        overlay.len(),
        1,
        "BC-2.06.023 PC-1: retain() must remove back entry (position-independent)"
    );
    assert_eq!(
        overlay.front().unwrap().prompt_id,
        pid1,
        "BC-2.06.023 PC-1: P1 remains at front after P2 (back) removed"
    );
}

/// BC-2.06.023 PC-1 (coverage): retain() removes a middle entry in a 4-item stack.
#[test]
fn test_BC_2_06_023_pc1_retain_removes_middle_entry_in_four_item_stack() {
    let pids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
    let mut overlay: VecDeque<_> = VecDeque::new();

    for &pid in &pids {
        apply_permission_prompt_queued(&mut overlay, bash_payload(pid));
    }
    assert_eq!(overlay.len(), 4, "precondition: 4 items");

    // Remove pids[2] (index 2, middle)
    overlay.retain(|m| m.prompt_id != pids[2]);

    assert_eq!(
        overlay.len(),
        3,
        "BC-2.06.023 PC-1: retain() must remove entry at index 2 of 4"
    );
    let ids: Vec<Uuid> = overlay.iter().map(|m| m.prompt_id).collect();
    assert_eq!(
        ids,
        vec![pids[0], pids[1], pids[3]],
        "BC-2.06.023 PC-1: after removing index 2, order must be [P0, P1, P3]"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.023 PC-2 — retain() removes ALL entries with matching prompt_id
// ---------------------------------------------------------------------------

/// BC-2.06.023 PC-2 (coverage): retain() removes ALL entries with the matching
/// prompt_id, not just the first. In normal operation, apply_permission_prompt_queued
/// prevents duplicates, but retain() is defined to handle them safely.
#[test]
fn test_BC_2_06_023_pc2_retain_removes_all_duplicate_entries() {
    // Bypass apply_permission_prompt_queued to create an artificial duplicate
    // (simulating a hypothetical race that bypassed the idempotency guard)
    use monocle_core::tui::state::{PromptModal, ToolPayload};
    use std::time::Instant;

    let pid = Uuid::new_v4();
    let other_pid = Uuid::new_v4();
    let mut overlay: VecDeque<PromptModal> = VecDeque::new();

    // Insert duplicate directly (bypassing apply_permission_prompt_queued guard)
    let make_modal = |id: Uuid| PromptModal {
        prompt_id: id,
        session_id: "sess-001".into(),
        tool_name: "Bash".into(),
        tool_payload: ToolPayload::Bash {
            command: "ls".into(),
        },
        received_at: Instant::now(),
    };

    overlay.push_back(make_modal(pid));
    overlay.push_back(make_modal(other_pid));
    overlay.push_back(make_modal(pid)); // artificial duplicate
    assert_eq!(overlay.len(), 3, "precondition: 3 items (1 duplicate)");

    // retain() removes ALL matching entries
    overlay.retain(|m| m.prompt_id != pid);

    assert_eq!(
        overlay.len(),
        1,
        "BC-2.06.023 PC-2: retain() must remove ALL entries with matching prompt_id (both duplicates)"
    );
    assert_eq!(
        overlay.front().unwrap().prompt_id,
        other_pid,
        "BC-2.06.023 PC-2: only the non-matching entry survives"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.023 PC-3 / AC-007 — Unknown prompt_id in Resolved is no-op
// ---------------------------------------------------------------------------

/// BC-2.06.023 PC-3 / AC-007 (coverage): retain() with a non-matching predicate
/// leaves the stack entirely intact. No panic, no side effects.
#[test]
fn test_BC_2_06_023_pc3_unknown_prompt_id_noop_stack_unchanged() {
    let pid = Uuid::new_v4();
    let unknown = Uuid::new_v4();
    let mut overlay: VecDeque<_> = VecDeque::new();

    apply_permission_prompt_queued(&mut overlay, bash_payload(pid));

    // Simulate PermissionPromptResolved for unknown prompt_id
    overlay.retain(|m| m.prompt_id != unknown);

    assert_eq!(
        overlay.len(),
        1,
        "BC-2.06.023 PC-3: unknown prompt_id in Resolved → stack unchanged (len=1)"
    );
    assert_eq!(
        overlay.front().unwrap().prompt_id,
        pid,
        "BC-2.06.023 PC-3: existing prompt_id must not be removed"
    );
}

/// BC-2.06.023 PC-3 / AC-007 (coverage): retain() on empty stack with unknown prompt_id
/// is a safe no-op. (Can happen if TUI received Resolved before Queued.)
#[test]
fn test_BC_2_06_023_pc3_unknown_prompt_id_on_empty_stack_is_safe() {
    use monocle_core::tui::state::PromptModal;
    let unknown = Uuid::new_v4();
    let mut overlay: VecDeque<PromptModal> = VecDeque::new();

    // Empty stack — retain() is safe
    overlay.retain(|m| m.prompt_id != unknown);

    assert!(
        overlay.is_empty(),
        "BC-2.06.023 PC-3: retain() on empty stack must leave it empty (no panic)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.023 PC-4 / AC-015 — Empty-stack collapse after last retain()
// ---------------------------------------------------------------------------

/// BC-2.06.023 PC-4 / AC-015 (coverage): After retain() removes the last modal,
/// the simulated handler collapses AppMode from Overlay to Dashboard { focused: prior }.
///
/// Architecture Compliance Rule: the collapse is NOT routed through transition() directly
/// for the retain() path — it uses the PopOverlay action after the direct mutation.
/// (See simulate_permission_prompt_resolved above for the canonical pattern.)
#[test]
fn test_BC_2_06_023_pc4_empty_stack_collapse_to_dashboard_after_last_retain() {
    let mut app = default_app();
    let pid = Uuid::new_v4();

    on_initial_state(&mut app, vec![], vec![], vec![bash_payload(pid)], 0);
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "precondition: Overlay mode with 1 prompt"
    );

    simulate_permission_prompt_resolved(&mut app, pid);

    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.023 PC-4: retain() must empty the stack after last removal"
    );
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "BC-2.06.023 PC-4: AppMode must collapse to Dashboard {{ focused: Sessions }} \
         when overlay_stack empties after retain()"
    );
}

/// BC-2.06.023 PC-4 / AC-015 (coverage): Removing one of two prompts leaves the
/// stack non-empty, so mode remains Overlay.
#[test]
fn test_BC_2_06_023_pc4_non_empty_stack_after_retain_stays_overlay() {
    let mut app = default_app();
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();

    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid1), bash_payload(pid2)],
        0,
    );

    simulate_permission_prompt_resolved(&mut app, pid1);

    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.023 PC-4: after removing P1, P2 remains"
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.023 PC-4: mode stays Overlay when stack is non-empty after retain()"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid2,
        "BC-2.06.023 PC-4: P2 is now at front"
    );
}

/// BC-2.06.023 PC-4 / AC-015 (coverage): Resolving all three prompts one-by-one
/// collapses to Dashboard only after the LAST removal.
#[test]
fn test_BC_2_06_023_pc4_sequential_removals_collapse_only_on_last() {
    let mut app = default_app();
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let pid3 = Uuid::new_v4();

    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid1), bash_payload(pid2), bash_payload(pid3)],
        0,
    );

    // Remove P1 — 2 remain, still Overlay
    simulate_permission_prompt_resolved(&mut app, pid1);
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.023 PC-4: after removing P1, mode stays Overlay (2 remain)"
    );

    // Remove P2 — 1 remains, still Overlay
    simulate_permission_prompt_resolved(&mut app, pid2);
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.023 PC-4: after removing P2, mode stays Overlay (1 remains)"
    );

    // Remove P3 — 0 remain, collapses to Dashboard
    simulate_permission_prompt_resolved(&mut app, pid3);
    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.023 PC-4: after removing P3, stack must be empty"
    );
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "BC-2.06.023 PC-4: after removing last prompt, AppMode collapses to Dashboard"
    );
}

// ---------------------------------------------------------------------------
// Architecture Compliance Rule — retain() is NOT routed through transition()
// ---------------------------------------------------------------------------

/// Architecture Compliance Rule (BC-2.06.023 / §Forbidden Dependencies):
/// The retain() removal path is a direct VecDeque mutation. transition() is called
/// AFTER the mutation only for the collapse check (PopOverlay). This test verifies
/// that transition() itself does NOT perform any stack mutation — it is stack-agnostic.
#[test]
fn test_BC_2_06_023_architecture_compliance_transition_does_not_touch_stack() {
    use monocle_core::tui::state::transition;

    let prior = FocusSnapshot::Sessions;
    let mode = AppMode::Overlay {
        prior: prior.clone(),
    };

    // transition(Overlay, PopOverlay) returns Dashboard — it does NOT touch any stack.
    // This is the pure function call the collapse handler uses.
    let result = transition(mode, Action::PopOverlay);

    assert!(
        matches!(
            result,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "Architecture Compliance: transition(Overlay, PopOverlay) → Dashboard {{ prior }}; \
         stack mutation is App-level, NOT inside transition()"
    );
}

/// Architecture Compliance Rule: transition(Overlay, PopOverlay) preserves the
/// `prior` FocusSnapshot — it restores to the panel that had focus before the overlay.
#[test]
fn test_BC_2_06_023_transition_pop_overlay_preserves_prior_focus() {
    use monocle_core::tui::state::transition;

    // prior = EventRibbon (user had EventRibbon focused before overlay)
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::EventRibbon,
    };
    let result = transition(mode, Action::PopOverlay);

    assert!(
        matches!(
            result,
            AppMode::Dashboard {
                focused: FocusSnapshot::EventRibbon
            }
        ),
        "BC-2.06.023 PC-4: collapse must restore prior FocusSnapshot — \
         EventRibbon prior preserved in Dashboard after PopOverlay"
    );
}
