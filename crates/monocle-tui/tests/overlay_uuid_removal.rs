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
//! - BC-2.06.023 PC-3: Unknown prompt_id in PermissionPromptResolved → silent discard, TRACE-only, no-op.
//! - BC-2.06.023 PC-4 / AC-015: After retain() empties the stack, AppMode must not remain
//!   Overlay — empty stack triggers collapse to Dashboard { focused: prior }.
//! - Architecture Compliance: retain() is NOT routed through transition() — direct mutation
//!   followed by a post-retain collapse check.
//! - AC-015: After retain() of last item, the IPC handler must transition to Dashboard.

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{Action, AppMode, FocusSnapshot};
use monocle_ipc::types::PermissionPromptPayload;
use monocle_tui::app::{
    apply_permission_prompt_queued, on_initial_state, on_permission_prompt_resolved, App,
};
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

/// BC-2.06.023 PC-3 / AC-007 (data-layer): retain() with a non-matching predicate
/// leaves the VecDeque entirely intact. No panic, no side effects.
/// This test exercises the raw data structure; for the App-level production path see
/// `test_BC_2_06_023_pc3_ac007_unknown_prompt_id_no_state_change_production`.
#[test]
fn test_BC_2_06_023_pc3_unknown_prompt_id_noop_stack_unchanged() {
    let pid = Uuid::new_v4();
    let unknown = Uuid::new_v4();
    let mut overlay: VecDeque<_> = VecDeque::new();

    apply_permission_prompt_queued(&mut overlay, bash_payload(pid));

    // Data-layer: retain() directly — validates the predicate semantics.
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

/// BC-2.06.023 PC-3 / AC-007 (data-layer): retain() on empty stack with unknown prompt_id
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

/// BC-2.06.023 PC-3 / AC-007 (production path): `on_permission_prompt_resolved` for
/// an unknown prompt_id is a silent no-op — stack unchanged, AppMode unchanged, NO
/// WARN emitted (TRACE only per BC-2.06.023 PC-3, story v1.11).
///
/// Drives the PRODUCTION `on_permission_prompt_resolved` entrypoint directly
/// (F-S026-ADV2-HIGH-001). Non-vacuity: if the production handler has an inverted
/// predicate (`retain(|m| m.prompt_id == unknown_id)`), the stack would be emptied
/// and mode would collapse — both stack-len and mode asserts would fail.
#[test]
fn test_BC_2_06_023_pc3_ac007_unknown_prompt_id_no_state_change_production() {
    let pid = Uuid::new_v4();
    let unknown = Uuid::new_v4();
    let mut app = default_app();
    apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid));
    app.mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };

    // Production handler for unknown prompt_id — silent discard, TRACE only.
    on_permission_prompt_resolved(&mut app, unknown);

    // Stack unchanged
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.023 PC-3 AC-007: unknown prompt_id → stack unchanged (len=1)"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.023 PC-3 AC-007: existing modal must remain in stack"
    );
    // Mode unchanged — no spurious collapse
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.023 PC-3 AC-007: AppMode must remain Overlay after unknown prompt_id no-op"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.023 PC-4 / AC-015 — Empty-stack collapse after last retain()
// ---------------------------------------------------------------------------

/// BC-2.06.023 PC-4 / AC-015: After `on_permission_prompt_resolved` removes the last
/// modal, AppMode collapses from Overlay to Dashboard { focused: prior }.
///
/// Drives the PRODUCTION `on_permission_prompt_resolved` entrypoint directly
/// (F-S026-ADV2-HIGH-001: no vacuous mirror). Non-vacuity: if the production
/// handler omits the empty-stack collapse, the final assert will fail.
#[test]
fn test_BC_2_06_023_pc4_empty_stack_collapse_to_dashboard_after_last_retain() {
    let mut app = default_app();
    let pid = Uuid::new_v4();

    on_initial_state(&mut app, vec![], vec![], vec![bash_payload(pid)], 0);
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "precondition: Overlay mode with 1 prompt"
    );

    // Production handler — not a test-local mirror.
    on_permission_prompt_resolved(&mut app, pid);

    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.023 PC-4: on_permission_prompt_resolved must empty the stack after last removal"
    );
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "BC-2.06.023 PC-4: AppMode must collapse to Dashboard {{ focused: Sessions }} \
         when overlay_stack empties (on_permission_prompt_resolved collapse path)"
    );
}

/// BC-2.06.023 PC-4 / AC-015: Removing one of two prompts via `on_permission_prompt_resolved`
/// leaves the stack non-empty, so mode remains Overlay.
///
/// Drives the PRODUCTION `on_permission_prompt_resolved` entrypoint directly
/// (F-S026-ADV2-HIGH-001). Non-vacuity: if retain() is inverted in production,
/// the stack will be empty and the mode will collapse — both asserts will fail.
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

    // Production handler — not a test-local mirror.
    on_permission_prompt_resolved(&mut app, pid1);

    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.023 PC-4: after removing P1 via on_permission_prompt_resolved, P2 remains"
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.023 PC-4: mode stays Overlay when stack is non-empty after removal"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid2,
        "BC-2.06.023 PC-4: P2 is now at front"
    );
}

/// BC-2.06.023 PC-4 / AC-015: Resolving all three prompts one-by-one via
/// `on_permission_prompt_resolved` collapses to Dashboard only after the LAST removal.
///
/// Drives the PRODUCTION `on_permission_prompt_resolved` entrypoint directly for all
/// three calls (F-S026-ADV2-HIGH-001). Non-vacuity: if the collapse guard is absent,
/// the final assert on mode=Dashboard will fail on the third call.
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
    on_permission_prompt_resolved(&mut app, pid1);
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.023 PC-4: after removing P1, mode stays Overlay (2 remain)"
    );

    // Remove P2 — 1 remains, still Overlay
    on_permission_prompt_resolved(&mut app, pid2);
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.023 PC-4: after removing P2, mode stays Overlay (1 remains)"
    );

    // Remove P3 — 0 remain, collapses to Dashboard
    on_permission_prompt_resolved(&mut app, pid3);
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
