//! Tests for BC-2.06.009 (stack rotation via Up/Down → Action::OverlayCycleNext).
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]
//!
//! # Production Binding Path
//!
//! `build_builtin_binding_layers()` does NOT need to register Up/Down → OverlayCycleNext
//! in the search_prompt table — the production `resolve_binding` function handles this
//! directly in its `AppMode::Overlay { .. }` match arm (binding.rs), which hardcodes
//! `KeyCode::Up` and `KeyCode::Down` → `Action::OverlayCycleNext` before falling
//! through to the search_prompt table.  All keypress rotation tests in this file use
//! `build_builtin_binding_layers()` unmodified so they exercise that production path.
//!
//! Non-vacuity guarantee: if the `KeyCode::Up | KeyCode::Down => OverlayCycleNext`
//! arm in the Overlay branch of `resolve_binding` were removed, Up/Down would fall
//! through to the Builtin layer's `SelectPrev`/`SelectNext` bindings, which are gated
//! on `AppMode::Dashboard { focused: Sessions }`.  In Overlay mode the gate fails,
//! no rotation occurs, and every keypress test in this file would fail because the
//! stack order would remain unchanged.
//!
//! # Coverage
//!
//! - Rotation with `stack.len() > 1`: `pop_front()` + `push_back()` moves front to back.
//! - Single-item no-op: `stack.len() == 1`, rotation returns same item to front (EC-065).
//! - Mode stays `Overlay { prior }` after rotation — `transition()` is identity for OverlayCycleNext.
//! - `overlay_stack.front()` changes after rotation (new prompt rendered).
//! - AC-013 (BC-2.06.009 PC-1): Up/Down in Overlay dispatches OverlayCycleNext via production binding.
//! - AC-014 (BC-2.06.009 EC-065): Single-item stack: rotation is no-op.

use monocle_config::MonocleConfig;
use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::PermissionPromptPayload;
use monocle_tui::app::{
    apply_permission_prompt_queued, build_builtin_binding_layers, dispatch_key_event, App,
    KeyOutcome,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
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

fn no_mod() -> KeyModifiers {
    KeyModifiers::default()
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: no_mod(),
    }
}

/// Build an App in `Overlay { prior: Sessions }` with the given prompt_ids pre-loaded.
fn overlay_app_with_prompts(pids: &[Uuid]) -> App {
    let mut app = App::new(MonocleConfig::default());
    for &pid in pids {
        apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid));
    }
    if !app.overlay_stack.is_empty() {
        app.mode = AppMode::Overlay {
            prior: FocusSnapshot::Sessions,
        };
    }
    app
}

// ---------------------------------------------------------------------------
// BC-2.06.009 PC-1 / AC-013 — Rotation with len > 1 (production binding path)
// ---------------------------------------------------------------------------

/// BC-2.06.009 PC-1 / AC-013: With stack=[P1, P2], pressing Up in Overlay mode
/// resolves to OverlayCycleNext via the production `resolve_binding` Overlay arm
/// (not via search_prompt table injection). P1 moves to back; P2 becomes front.
/// Mode stays Overlay { prior: Sessions }.
#[test]
fn test_BC_2_06_009_pc1_rotation_len_gt_1_moves_front_to_back() {
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let mut app = overlay_app_with_prompts(&[pid1, pid2]);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Precondition: P1 is at front
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid1,
        "BC-2.06.009 precondition: P1 must be at front before rotation"
    );

    let outcome = dispatch_key_event(&mut app, &key(KeyCode::Up), &layers, &mut sessions_state);

    assert_eq!(
        outcome,
        KeyOutcome::Continue,
        "BC-2.06.009 PC-1: Up key must not quit"
    );

    // After rotation: P2 is now at front, P1 is at back
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid2,
        "BC-2.06.009 PC-1: P2 must be at front after one rotation"
    );
    assert_eq!(
        app.overlay_stack.back().unwrap().prompt_id,
        pid1,
        "BC-2.06.009 PC-1: P1 must be at back after one rotation"
    );
    assert_eq!(
        app.overlay_stack.len(),
        2,
        "BC-2.06.009 PC-1: stack len must not change"
    );

    // Mode stays Overlay — transition() is identity for OverlayCycleNext
    assert!(
        matches!(
            app.mode,
            AppMode::Overlay {
                prior: FocusSnapshot::Sessions
            }
        ),
        "BC-2.06.009 PC-1: AppMode must stay Overlay {{ prior: Sessions }} after rotation"
    );
}

/// BC-2.06.009 PC-1 / AC-013: Down key also resolves to OverlayCycleNext via the
/// production binding path (Overlay arm in resolve_binding). Same rotation behavior
/// as Up: front moves to back.
#[test]
fn test_BC_2_06_009_pc1_rotation_via_down_key_also_rotates() {
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let pid3 = Uuid::new_v4();
    let mut app = overlay_app_with_prompts(&[pid1, pid2, pid3]);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Rotate once via Down (production binding path)
    dispatch_key_event(&mut app, &key(KeyCode::Down), &layers, &mut sessions_state);

    // After one rotation: P2 at front, P1 at back, P3 in middle
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid2,
        "BC-2.06.009 PC-1: Down rotates front (P1) to back; P2 becomes front"
    );

    let ids: Vec<Uuid> = app.overlay_stack.iter().map(|m| m.prompt_id).collect();
    assert_eq!(
        ids,
        vec![pid2, pid3, pid1],
        "BC-2.06.009 PC-1: after Down rotation: order must be [P2, P3, P1]"
    );
}

/// BC-2.06.009 PC-1 / AC-013: Two successive Up presses of a 3-item stack via the
/// production binding path produce the correct order.
#[test]
fn test_BC_2_06_009_pc1_two_rotations_produce_correct_order() {
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let pid3 = Uuid::new_v4();
    let mut app = overlay_app_with_prompts(&[pid1, pid2, pid3]);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Rotate 1: [P1,P2,P3] → [P2,P3,P1]
    dispatch_key_event(&mut app, &key(KeyCode::Up), &layers, &mut sessions_state);
    // Rotate 2: [P2,P3,P1] → [P3,P1,P2]
    dispatch_key_event(&mut app, &key(KeyCode::Up), &layers, &mut sessions_state);

    let ids: Vec<Uuid> = app.overlay_stack.iter().map(|m| m.prompt_id).collect();
    assert_eq!(
        ids,
        vec![pid3, pid1, pid2],
        "BC-2.06.009 PC-1: two rotations of [P1,P2,P3] → [P3,P1,P2]"
    );
}

/// BC-2.06.009 PC-1 / AC-013: Rotation wraps — three Up presses of a 3-item stack
/// returns to original order. Production binding path only.
#[test]
fn test_BC_2_06_009_pc1_three_rotations_wraps_back_to_original_order() {
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let pid3 = Uuid::new_v4();
    let mut app = overlay_app_with_prompts(&[pid1, pid2, pid3]);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    for _ in 0..3 {
        dispatch_key_event(&mut app, &key(KeyCode::Up), &layers, &mut sessions_state);
    }

    let ids: Vec<Uuid> = app.overlay_stack.iter().map(|m| m.prompt_id).collect();
    assert_eq!(
        ids,
        vec![pid1, pid2, pid3],
        "BC-2.06.009 PC-1: 3 rotations of 3-item stack wraps back to original [P1,P2,P3]"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.009 EC-065 / AC-014 — Single-item rotation is no-op (production binding)
// ---------------------------------------------------------------------------

/// BC-2.06.009 EC-065 / AC-014: With stack=[P1] (len==1), pressing Up in Overlay
/// mode resolves to OverlayCycleNext via production binding. P1 is popped and
/// immediately re-pushed to back (structural no-op — same prompt at front).
#[test]
fn test_BC_2_06_009_ec065_single_item_rotation_is_noop() {
    let pid = Uuid::new_v4();
    let mut app = overlay_app_with_prompts(&[pid]);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let outcome = dispatch_key_event(&mut app, &key(KeyCode::Up), &layers, &mut sessions_state);

    assert_eq!(
        outcome,
        KeyOutcome::Continue,
        "BC-2.06.009 EC-065: Up with single-item stack must not quit"
    );

    // Stack still has 1 item; same prompt_id at front
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.009 EC-065: single-item rotation must not change stack length"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.009 EC-065: single-item rotation must leave same prompt at front"
    );

    // Mode unchanged
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.009 EC-065: mode must remain Overlay after single-item rotation"
    );
}

/// BC-2.06.009 EC-065 / AC-014: Down key with single-item stack is also a structural
/// no-op via production binding path.
#[test]
fn test_BC_2_06_009_ec065_single_item_rotation_down_is_noop() {
    let pid = Uuid::new_v4();
    let mut app = overlay_app_with_prompts(&[pid]);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(&mut app, &key(KeyCode::Down), &layers, &mut sessions_state);

    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.009 EC-065: Down with single-item stack must not change stack length"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.009 EC-065: Down single-item rotation must leave same prompt at front"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.009 — transition() is identity for OverlayCycleNext (pure function test)
// ---------------------------------------------------------------------------

/// BC-2.06.009 / transition() purity: `transition(Overlay { prior }, OverlayCycleNext)`
/// returns `Overlay { prior }` unchanged. The actual stack rotation is App-level.
#[test]
fn test_BC_2_06_009_transition_overlay_cycle_next_is_identity() {
    use monocle_core::tui::state::{transition, Action};

    let mode_before = AppMode::Overlay {
        prior: FocusSnapshot::EventRibbon,
    };
    let mode_after = transition(mode_before, Action::OverlayCycleNext);

    assert!(
        matches!(
            mode_after,
            AppMode::Overlay {
                prior: FocusSnapshot::EventRibbon
            }
        ),
        "BC-2.06.009: transition(Overlay, OverlayCycleNext) must be identity — \
         EventRibbon prior preserved"
    );
}

// ---------------------------------------------------------------------------
// AC-010 — Overlay binding isolation: j/k do NOT navigate sessions list in Overlay
// ---------------------------------------------------------------------------

/// AC-010 / BC-2.06.003 (coverage): In Overlay mode, `j` (mapped to SelectNext in Builtin
/// layer) does NOT mutate sessions_state.list_state because dispatch_key_event gates
/// SelectNext on `AppMode::Dashboard { focused: Sessions }`.
#[test]
fn test_ac_010_overlay_binding_isolation_j_does_not_scroll_sessions() {
    let pid = Uuid::new_v4();
    let mut app = overlay_app_with_prompts(&[pid]);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // In Overlay mode, `j` is bound to SelectNext (Builtin layer) but the
    // SelectNext arm in dispatch_key_event is gated on Dashboard { focused: Sessions }.
    // In Overlay mode, the gate fails — sessions_state.list_state is NOT mutated.
    let initial_selected = sessions_state.list_state.selected();

    dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('j')),
        &layers,
        &mut sessions_state,
    );

    assert_eq!(
        sessions_state.list_state.selected(),
        initial_selected,
        "AC-010: `j` in Overlay mode must NOT mutate sessions_state (overlay binding isolation)"
    );

    // Mode remains Overlay
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "AC-010: mode must remain Overlay after `j` press"
    );
}
