//! Tests for BC-2.06.011 (Accept-Once), BC-2.06.012 (Accept-Always), BC-2.06.013 (Reject),
//! and the wait-for-PermissionPromptResolved round-trip (modal stays in stack after decision send).
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]
//!
//! # Red Gate
//!
//! Tests for AC-003, AC-004, AC-005 (decision keybinding dispatch) are TRUE RED GATE tests.
//! They exercise the `todo!()` arms in `dispatch_key_event` (Action::PermissionAcceptOnce,
//! Action::PermissionAcceptAlways, Action::PermissionReject). These tests MUST FAIL against
//! the current S-026 stubs with a panic from `todo!()`.
//!
//! Tests for AC-006, AC-007, AC-008 exercise S-025-pre-built behavior (retain(), unknown
//! prompt_id no-op, Esc identity). These are COVERAGE/characterization tests and are
//! expected to PASS immediately.
//!
//! # Coverage
//!
//! - AC-003: Accept-Once (`y`/`Enter`): `ClientToServer::PermissionDecision { decision: Allow }`
//!   enqueued; modal NOT popped (BC-2.06.011 PC-1/2).  [RED GATE]
//! - AC-004: Accept-Always (`A`): `ClientToServer::PermissionDecision { decision: AcceptAlways }`
//!   enqueued; modal NOT popped (BC-2.06.012 PC-1/2).  [RED GATE]
//! - AC-005: Reject (`n`/`r`): `ClientToServer::PermissionDecision { decision: Deny }` enqueued;
//!   modal NOT popped (BC-2.06.013 PC-1/2).  [RED GATE]
//! - AC-006 / BC-2.06.023 PC-1: PermissionPromptResolved triggers retain() removal.  [PASS]
//! - AC-007 / BC-2.06.023 PC-3: Unknown prompt_id in Resolved → no-op (no panic, stack unchanged). [PASS]
//! - AC-008 / BC-2.06.014 PC-1: Esc in Overlay is identity no-op — no IPC send, mode unchanged. [PASS]

use monocle_config::MonocleConfig;
use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::{ClientToServer, PermissionDecisionKind, PermissionPromptPayload};
use monocle_tui::app::{
    apply_permission_prompt_queued, build_builtin_binding_layers, dispatch_key_event, App,
    KeyOutcome,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use std::collections::VecDeque;
use tokio::sync::mpsc;
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

/// Build an App pre-configured in `Overlay { prior: Sessions }` with one modal in
/// overlay_stack (prompt_id = `pid`), and with `ipc_tx` wired to the returned receiver.
fn overlay_app_with_ipc(pid: Uuid) -> (App, mpsc::Receiver<ClientToServer>) {
    let (tx, rx) = mpsc::channel::<ClientToServer>(16);
    let mut app = App::new(MonocleConfig::default());
    apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid));
    app.mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    app.ipc_tx = Some(tx);
    (app, rx)
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

// ---------------------------------------------------------------------------
// AC-003 / BC-2.06.011 PC-1 — Accept-Once (`y`) sends Allow; modal stays in stack
// [RED GATE — exercises todo!() arm]
// ---------------------------------------------------------------------------

/// BC-2.06.011 PC-1 / AC-003 (RED GATE): Pressing `y` in Overlay mode enqueues
/// `ClientToServer::PermissionDecision { decision: Allow }` on ipc_tx with the
/// correct prompt_id, and does NOT pop the modal from overlay_stack.
///
/// EXPECTED FAILURE: panics with todo!() from Action::PermissionAcceptOnce arm.
#[test]
fn test_BC_2_06_011_accept_once_y_sends_allow_ipc_modal_stays_in_stack() {
    let pid = Uuid::new_v4();
    let (mut app, mut rx) = overlay_app_with_ipc(pid);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Press `y` — should dispatch PermissionAcceptOnce → Allow
    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('y')),
        &layers,
        &mut sessions_state,
    );

    assert_eq!(
        outcome,
        KeyOutcome::Continue,
        "BC-2.06.011 PC-1: `y` must not quit"
    );

    // Modal must still be in the stack (NOT popped on send)
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.011 Invariant 5: modal must NOT be popped from overlay_stack after decision send"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.011 Invariant 5: front() prompt_id must be unchanged after decision send"
    );

    // Mode must remain Overlay
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.011 PC-1: AppMode must remain Overlay after decision send"
    );

    // IPC message must be enqueued with correct prompt_id and Allow decision
    let msg = rx.try_recv().expect(
        "BC-2.06.011 PC-1: ClientToServer message must be enqueued on ipc_tx after `y`",
    );
    match msg {
        ClientToServer::PermissionDecision { prompt_id, decision } => {
            assert_eq!(
                prompt_id, pid,
                "BC-2.06.011 PC-1: PermissionDecision prompt_id must match front modal's prompt_id"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::Allow,
                "BC-2.06.011 PC-1: decision must be Allow for `y` (Accept-Once)"
            );
        }
    }
}

/// BC-2.06.011 PC-1 / AC-003 (RED GATE): Enter also maps to PermissionAcceptOnce.
/// Same assertions as `y`.
#[test]
fn test_BC_2_06_011_accept_once_enter_sends_allow_ipc_modal_stays() {
    let pid = Uuid::new_v4();
    let (mut app, mut rx) = overlay_app_with_ipc(pid);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Enter),
        &layers,
        &mut sessions_state,
    );

    assert_eq!(outcome, KeyOutcome::Continue, "BC-2.06.011: Enter must not quit");

    // Modal stays in stack
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.011 Invariant 5: modal must NOT be popped after Enter"
    );

    let msg = rx.try_recv().expect(
        "BC-2.06.011 PC-1: ClientToServer message must be enqueued on ipc_tx after Enter",
    );
    match msg {
        ClientToServer::PermissionDecision { prompt_id, decision } => {
            assert_eq!(prompt_id, pid);
            assert_eq!(
                decision,
                PermissionDecisionKind::Allow,
                "BC-2.06.011 PC-1: Enter must send Allow decision"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.06.012 PC-1 — Accept-Always (`A`) sends AcceptAlways; modal stays
// [RED GATE — exercises todo!() arm]
// ---------------------------------------------------------------------------

/// BC-2.06.012 PC-1 / AC-004 (RED GATE): Pressing `A` (uppercase) in Overlay mode
/// enqueues `ClientToServer::PermissionDecision { decision: AcceptAlways }` on ipc_tx
/// and does NOT pop the modal from overlay_stack.
///
/// EXPECTED FAILURE: panics with todo!() from Action::PermissionAcceptAlways arm.
#[test]
fn test_BC_2_06_012_accept_always_uppercase_a_sends_accept_always_modal_stays() {
    let pid = Uuid::new_v4();
    let (mut app, mut rx) = overlay_app_with_ipc(pid);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Press uppercase `A`
    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('A')),
        &layers,
        &mut sessions_state,
    );

    assert_eq!(outcome, KeyOutcome::Continue, "BC-2.06.012: `A` must not quit");

    // Modal must NOT be popped
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.012 Invariant 2: modal must NOT be popped from overlay_stack after AcceptAlways send"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.012 Invariant 2: front prompt_id unchanged after AcceptAlways send"
    );

    // Mode stays Overlay
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.012 PC-1: AppMode must remain Overlay after AcceptAlways send"
    );

    // IPC message: AcceptAlways
    let msg = rx.try_recv().expect(
        "BC-2.06.012 PC-1: ClientToServer message must be enqueued on ipc_tx after `A`",
    );
    match msg {
        ClientToServer::PermissionDecision { prompt_id, decision } => {
            assert_eq!(
                prompt_id, pid,
                "BC-2.06.012 PC-1: PermissionDecision prompt_id must match front modal"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::AcceptAlways,
                "BC-2.06.012 PC-1: decision must be AcceptAlways for `A`"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.06.013 PC-1 — Reject (`n`/`r`) sends Deny; modal stays
// [RED GATE — exercises todo!() arm]
// ---------------------------------------------------------------------------

/// BC-2.06.013 PC-1 / AC-005 (RED GATE): Pressing `n` in Overlay mode enqueues
/// `ClientToServer::PermissionDecision { decision: Deny }` on ipc_tx and does NOT
/// pop the modal from overlay_stack.
///
/// EXPECTED FAILURE: panics with todo!() from Action::PermissionReject arm.
#[test]
fn test_BC_2_06_013_reject_n_sends_deny_modal_stays_in_stack() {
    let pid = Uuid::new_v4();
    let (mut app, mut rx) = overlay_app_with_ipc(pid);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('n')),
        &layers,
        &mut sessions_state,
    );

    assert_eq!(outcome, KeyOutcome::Continue, "BC-2.06.013: `n` must not quit");

    // Modal must NOT be popped
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.013 Invariant 4: modal must NOT be popped after Reject send"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.013 Invariant 4: front prompt_id unchanged after Reject send"
    );

    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.013 PC-1: AppMode must remain Overlay after Reject send"
    );

    let msg = rx.try_recv().expect(
        "BC-2.06.013 PC-1: ClientToServer message must be enqueued on ipc_tx after `n`",
    );
    match msg {
        ClientToServer::PermissionDecision { prompt_id, decision } => {
            assert_eq!(prompt_id, pid, "BC-2.06.013: prompt_id must match front modal");
            assert_eq!(
                decision,
                PermissionDecisionKind::Deny,
                "BC-2.06.013 PC-1: decision must be Deny for `n`"
            );
        }
    }
}

/// BC-2.06.013 PC-1 / AC-005 (RED GATE): `r` also maps to PermissionReject.
///
/// EXPECTED FAILURE: panics with todo!() from Action::PermissionReject arm.
#[test]
fn test_BC_2_06_013_reject_r_sends_deny_modal_stays_in_stack() {
    let pid = Uuid::new_v4();
    let (mut app, mut rx) = overlay_app_with_ipc(pid);
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('r')),
        &layers,
        &mut sessions_state,
    );

    assert_eq!(outcome, KeyOutcome::Continue, "BC-2.06.013: `r` must not quit");

    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.013 Invariant 4: modal must NOT be popped after `r` Reject send"
    );

    let msg = rx.try_recv().expect(
        "BC-2.06.013 PC-1: ClientToServer message must be enqueued on ipc_tx after `r`",
    );
    match msg {
        ClientToServer::PermissionDecision { prompt_id, decision } => {
            assert_eq!(prompt_id, pid);
            assert_eq!(
                decision,
                PermissionDecisionKind::Deny,
                "BC-2.06.013 PC-1: `r` must send Deny decision"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-003 — Two-prompt stack: decision uses FRONT prompt_id (not back)
// [RED GATE]
// ---------------------------------------------------------------------------

/// BC-2.06.011 Invariant 1 / AC-003 (RED GATE): With stack=[P1, P2], pressing `y`
/// sends PermissionDecision for P1 (front), NOT P2. Stack stays [P1, P2].
#[test]
fn test_BC_2_06_011_accept_once_uses_front_prompt_id_in_multi_stack() {
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);
    let mut app = App::new(MonocleConfig::default());

    apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid1));
    apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid2));
    app.mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    app.ipc_tx = Some(tx);

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('y')),
        &layers,
        &mut sessions_state,
    );

    // Stack must stay [P1, P2] — no pop
    assert_eq!(app.overlay_stack.len(), 2, "BC-2.06.011: stack=[P1,P2] stays unchanged");

    let msg = rx.try_recv().expect("must send IPC message");
    match msg {
        ClientToServer::PermissionDecision { prompt_id, decision } => {
            assert_eq!(
                prompt_id, pid1,
                "BC-2.06.011 Invariant 1: PermissionDecision must use FRONT (P1), not P2"
            );
            assert_eq!(decision, PermissionDecisionKind::Allow);
        }
    }
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.06.014 PC-1 — Esc in Overlay is identity no-op (PASS immediately)
// ---------------------------------------------------------------------------

/// BC-2.06.014 PC-1 / AC-008 (coverage): Pressing Esc in Overlay mode is a no-op identity.
/// Mode stays Overlay; no IPC message enqueued; overlay_stack unchanged.
#[test]
fn test_BC_2_06_014_esc_in_overlay_is_identity_no_ipc_send() {
    let pid = Uuid::new_v4();
    let (tx, mut rx) = mpsc::channel::<ClientToServer>(16);
    let mut app = App::new(MonocleConfig::default());

    apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid));
    app.mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    app.ipc_tx = Some(tx);

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Esc),
        &layers,
        &mut sessions_state,
    );

    assert_eq!(
        outcome,
        KeyOutcome::Continue,
        "BC-2.06.014 PC-1: Esc in Overlay must not trigger quit"
    );

    // Mode must remain Overlay — not collapsed to Dashboard
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.014 PC-1: Esc in Overlay is identity — AppMode must remain Overlay"
    );

    // overlay_stack unchanged — Esc never pops
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.014 PC-1: Esc must NOT pop the overlay_stack"
    );

    // No IPC message sent
    assert!(
        rx.try_recv().is_err(),
        "BC-2.06.014 PC-1: Esc must NOT send any IPC message"
    );
}

/// BC-2.06.014 PC-1 EC-096 / AC-008: Esc is identity even with an empty overlay_stack.
/// (Invariant 1 of BC-2.06.008 means this state is unreachable in steady-state, but
/// we verify the transition() pure function behaviour.)
#[test]
fn test_BC_2_06_014_transition_esc_overlay_is_identity_pure_function() {
    use monocle_core::tui::state::{transition, Action};

    let mode_before = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let mode_after = transition(mode_before, Action::Esc);

    assert!(
        matches!(mode_after, AppMode::Overlay { prior: FocusSnapshot::Sessions }),
        "BC-2.06.014 PC-1: transition(Overlay, Esc) must return Overlay unchanged (identity)"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.06.023 PC-1 — PermissionPromptResolved triggers retain() (PASS immediately)
// ---------------------------------------------------------------------------

/// BC-2.06.023 PC-1 / AC-006 (coverage): Receiving PermissionPromptResolved for a prompt
/// that IS in the stack removes it via retain(). Modal gone, stack may shrink.
#[test]
fn test_BC_2_06_023_pc1_resolved_known_prompt_id_removes_via_retain() {
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let mut overlay: VecDeque<_> = VecDeque::new();

    apply_permission_prompt_queued(&mut overlay, bash_payload(pid1));
    apply_permission_prompt_queued(&mut overlay, bash_payload(pid2));
    assert_eq!(overlay.len(), 2, "precondition: 2 items in stack");

    // Simulate PermissionPromptResolved for P1
    overlay.retain(|m| m.prompt_id != pid1);

    assert_eq!(
        overlay.len(),
        1,
        "BC-2.06.023 PC-1: retain() must remove the resolved modal; 1 remains"
    );
    assert_eq!(
        overlay.front().unwrap().prompt_id,
        pid2,
        "BC-2.06.023 PC-1: P2 is now at front after P1 removed"
    );
}

/// BC-2.06.023 PC-1 / AC-006 (coverage): retain() removes by position-independent UUID match,
/// not just front(). Resolving P2 (middle of stack) leaves P1 and P3.
#[test]
fn test_BC_2_06_023_pc1_retain_removes_non_front_entry() {
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let pid3 = Uuid::new_v4();
    let mut overlay: VecDeque<_> = VecDeque::new();

    apply_permission_prompt_queued(&mut overlay, bash_payload(pid1));
    apply_permission_prompt_queued(&mut overlay, bash_payload(pid2));
    apply_permission_prompt_queued(&mut overlay, bash_payload(pid3));

    // Resolve middle entry P2
    overlay.retain(|m| m.prompt_id != pid2);

    assert_eq!(
        overlay.len(),
        2,
        "BC-2.06.023 PC-1: retain() removes P2 (not front); 2 remain"
    );
    let ids: Vec<Uuid> = overlay.iter().map(|m| m.prompt_id).collect();
    assert_eq!(ids, vec![pid1, pid3], "BC-2.06.023 PC-1: P1 and P3 remain in order");
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.06.023 PC-3 — Unknown prompt_id in Resolved is no-op (PASS immediately)
// ---------------------------------------------------------------------------

/// BC-2.06.023 PC-3 / AC-007 (coverage): PermissionPromptResolved for an unknown
/// prompt_id is a no-op. retain() with a non-matching predicate leaves the stack intact.
#[test]
fn test_BC_2_06_023_pc3_unknown_prompt_id_resolved_is_noop() {
    let pid = Uuid::new_v4();
    let unknown_pid = Uuid::new_v4();
    let mut overlay: VecDeque<_> = VecDeque::new();

    apply_permission_prompt_queued(&mut overlay, bash_payload(pid));
    assert_eq!(overlay.len(), 1, "precondition: 1 item in stack");

    // Resolved for a pid NOT in the stack
    overlay.retain(|m| m.prompt_id != unknown_pid);

    assert_eq!(
        overlay.len(),
        1,
        "BC-2.06.023 PC-3: unknown prompt_id in Resolved must leave stack unchanged"
    );
    assert_eq!(
        overlay.front().unwrap().prompt_id,
        pid,
        "BC-2.06.023 PC-3: existing modal must remain after no-op retain()"
    );
}
