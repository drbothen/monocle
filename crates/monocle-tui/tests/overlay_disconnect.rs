//! Tests for BC-2.06.016 (disconnect clears overlay; reconnect restores via InitialState).
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]
//!
//! # Red Gate
//!
//! All tests in this file exercise code paths pre-built by S-025:
//! - `on_transport_event(Disconnected)` — clears overlay_stack, transitions to Dashboard,
//!   sets DAEMON_DISCONNECT_STATUS.
//! - `on_initial_state` with overlay — re-populates overlay_stack, enters Overlay mode.
//! - `apply_permission_prompt_queued` idempotency — snapshot-window dedup.
//!
//! These are COVERAGE/characterization tests asserting real behavior. They are expected
//! to PASS immediately because the production code is already in place.
//!
//! # Coverage
//!
//! - AC-011 / BC-2.06.016 PC-1: `TransportEvent::Disconnected` → overlay_stack.clear(),
//!   AppMode → Dashboard { focused: Sessions }, status_message = DAEMON_DISCONNECT_STATUS.
//! - AC-012 / BC-2.06.016 PC-2: `ServerToClient::InitialState { overlay_stack: [P1, P2] }`
//!   re-populates overlay_stack and enters Overlay mode.
//! - BC-2.05.002 Invariant 4 / AC-001 snapshot-window dedup (`test_snapshot_window_prompt_dedup`):
//!   InitialState with [X, Y] then streaming PermissionPromptQueued for Y → VecDeque = {X, Y} exactly once.

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::PermissionPromptPayload;
use monocle_tui::app::{
    apply_permission_prompt_queued, on_initial_state, on_transport_event, App, TransportEvent,
};
use monocle_tui::DAEMON_DISCONNECT_STATUS;
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
// AC-011 / BC-2.06.016 PC-1 — Disconnect clears overlay and transitions to Dashboard
// ---------------------------------------------------------------------------

/// BC-2.06.016 PC-1 / AC-011 (coverage): TransportEvent::Disconnected clears
/// overlay_stack (set to empty VecDeque) and transitions AppMode to
/// Dashboard { focused: Sessions }.
#[test]
fn test_BC_2_06_016_pc1_disconnect_clears_overlay_stack() {
    let mut app = default_app();
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();

    // Setup: non-empty overlay in Overlay mode
    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid1), bash_payload(pid2)],
        0,
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.016 precondition: must be in Overlay mode with prompts"
    );
    assert_eq!(app.overlay_stack.len(), 2, "precondition: 2 prompts in stack");

    // Receive Disconnected
    on_transport_event(&mut app, TransportEvent::Disconnected);

    // overlay_stack must be completely cleared
    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.016 PC-1: overlay_stack must be empty after TransportEvent::Disconnected"
    );

    // AppMode must collapse to Dashboard { focused: Sessions }
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "BC-2.06.016 PC-1 (SOQ-3): AppMode must be Dashboard {{ focused: Sessions }} after disconnect"
    );
}

/// BC-2.06.016 PC-1 / AC-011 (coverage): Disconnect sets status_message to
/// DAEMON_DISCONNECT_STATUS ("[disconnected] reconnecting...").
#[test]
fn test_BC_2_06_016_pc1_disconnect_sets_status_message() {
    let mut app = default_app();

    on_transport_event(&mut app, TransportEvent::Disconnected);

    assert_eq!(
        app.status_message.as_deref(),
        Some(DAEMON_DISCONNECT_STATUS),
        "BC-2.06.016 PC-1: status_message must be DAEMON_DISCONNECT_STATUS after disconnect"
    );
}

/// BC-2.06.016 PC-1 / AC-011 (coverage): Disconnect is idempotent — calling twice
/// does not panic and leaves the app in the same stable state.
#[test]
fn test_BC_2_06_016_pc1_disconnect_is_idempotent() {
    let mut app = default_app();
    let pid = Uuid::new_v4();

    on_initial_state(&mut app, vec![], vec![], vec![bash_payload(pid)], 0);

    // First disconnect
    on_transport_event(&mut app, TransportEvent::Disconnected);
    assert!(app.overlay_stack.is_empty(), "first disconnect: overlay cleared");

    // Second disconnect (no overlay to clear; should not panic)
    on_transport_event(&mut app, TransportEvent::Disconnected);
    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.016 PC-1: second disconnect must leave overlay empty (idempotent)"
    );
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "BC-2.06.016 PC-1: mode must remain Dashboard after second disconnect"
    );
}

/// BC-2.06.016 PC-1 / AC-011 (coverage): Disconnect from Dashboard (no active overlay)
/// is safe — overlay_stack was already empty, mode stays Dashboard.
#[test]
fn test_BC_2_06_016_pc1_disconnect_from_dashboard_no_overlay_is_safe() {
    let mut app = default_app();
    // No overlay set up; starting from Dashboard with empty overlay
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "precondition: starting in Dashboard mode"
    );
    assert!(app.overlay_stack.is_empty(), "precondition: no overlay");

    on_transport_event(&mut app, TransportEvent::Disconnected);

    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.016 PC-1: disconnect from Dashboard must not corrupt overlay_stack"
    );
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "BC-2.06.016 PC-1: mode must be Dashboard {{ Sessions }} after disconnect from Dashboard"
    );
}

// ---------------------------------------------------------------------------
// AC-012 / BC-2.06.016 PC-2 — Reconnect restores overlay from InitialState
// ---------------------------------------------------------------------------

/// BC-2.06.016 PC-2 / AC-012 (coverage): After disconnect, receiving InitialState
/// with overlay=[P1, P2] re-populates overlay_stack and enters Overlay mode.
#[test]
fn test_BC_2_06_016_pc2_reconnect_restores_overlay_from_initial_state() {
    let mut app = default_app();
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();

    // First connection: enter overlay
    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid1)],
        0,
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "precondition: in Overlay mode after first connection"
    );

    // Disconnect: clears overlay
    on_transport_event(&mut app, TransportEvent::Disconnected);
    assert!(app.overlay_stack.is_empty(), "disconnect clears overlay");
    assert!(matches!(app.mode, AppMode::Dashboard { .. }), "disconnect → Dashboard");

    // Reconnect: InitialState re-populates with [P1, P2] (P1 still pending, P2 is new)
    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid1), bash_payload(pid2)],
        0,
    );

    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.016 PC-2: non-empty overlay in InitialState on reconnect → Overlay mode"
    );
    assert_eq!(
        app.overlay_stack.len(),
        2,
        "BC-2.06.016 PC-2: both P1 and P2 must be in overlay_stack after reconnect"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid1,
        "BC-2.06.016 PC-2: FIFO preserved — P1 (oldest) at front after reconnect restore"
    );
}

/// BC-2.06.016 PC-2 / AC-012 (coverage): After disconnect, if InitialState arrives with
/// empty overlay (daemon resolved all prompts during disconnect), AppMode stays Dashboard.
#[test]
fn test_BC_2_06_016_pc2_reconnect_with_empty_overlay_stays_dashboard() {
    let mut app = default_app();
    let pid = Uuid::new_v4();

    on_initial_state(&mut app, vec![], vec![], vec![bash_payload(pid)], 0);
    on_transport_event(&mut app, TransportEvent::Disconnected);

    // Reconnect with empty overlay (daemon resolved the prompt)
    on_initial_state(&mut app, vec![], vec![], vec![], 0);

    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "BC-2.06.016 PC-2: empty overlay in InitialState on reconnect → stays Dashboard"
    );
    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.016 PC-2: overlay_stack must be empty after reconnect with no pending prompts"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.002 Invariant 4 / AC-001 — Snapshot-window dedup (`test_snapshot_window_prompt_dedup`)
//
// Canonical test per architect-decisions-pass-6.md §Implementer Directive step 3:
// (1) Daemon running with queued prompt_id=X in pending_decisions.
// (2) New TUI connects; concurrent prompt_id=Y arrives during snapshot window.
// (3) TUI receives InitialState.overlay_stack containing X and Y (snapshot included Y).
// (4) TUI also receives streaming PermissionPromptQueued { payload: Y } from mpsc.
// (5) Assert: TUI VecDeque<PromptModal> contains X and Y exactly once each (Y not doubled).
// ---------------------------------------------------------------------------

/// BC-2.05.002 Invariant 4 / AC-001 / §Implementer Directive: The snapshot-window dedup
/// scenario. InitialState includes [X, Y], then a streaming PermissionPromptQueued for Y
/// arrives (race condition: Y was buffered in the mpsc channel during the snapshot window).
/// Y must appear exactly once in overlay_stack.
///
/// This is the canonical integration test specified in architect-decisions-pass-6.md.
#[test]
fn test_snapshot_window_prompt_dedup() {
    let mut app = default_app();
    let pid_x = Uuid::new_v4(); // pre-existing prompt
    let pid_y = Uuid::new_v4(); // arrived concurrently during snapshot window

    // Step 3: TUI receives InitialState with X and Y
    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid_x), bash_payload(pid_y)],
        0,
    );

    assert_eq!(
        app.overlay_stack.len(),
        2,
        "snapshot dedup precondition: InitialState with [X, Y] → len=2"
    );

    // Step 4: Streaming PermissionPromptQueued for Y arrives from mpsc
    // (at-least-once delivery: Y was queued in the mpsc before snapshot was taken)
    apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid_y));

    // Step 5: Assert Y appears exactly once (not doubled)
    assert_eq!(
        app.overlay_stack.len(),
        2,
        "BC-2.05.002 Invariant 4 (snapshot-window dedup): Y must NOT be doubled; \
         overlay_stack must still have exactly 2 entries [X, Y]"
    );

    let ids: Vec<Uuid> = app.overlay_stack.iter().map(|m| m.prompt_id).collect();
    let x_count = ids.iter().filter(|&&id| id == pid_x).count();
    let y_count = ids.iter().filter(|&&id| id == pid_y).count();

    assert_eq!(
        x_count,
        1,
        "snapshot dedup: X must appear exactly once in overlay_stack"
    );
    assert_eq!(
        y_count,
        1,
        "BC-2.05.002 Invariant 4 (snapshot-window dedup): Y must appear exactly once, \
         NOT twice even though it arrived in both InitialState and streaming PermissionPromptQueued"
    );
}

/// BC-2.05.002 Invariant 4: Snapshot-window dedup also applies to X (pre-existing prompt).
/// If streaming PermissionPromptQueued for X arrives (e.g., delayed reconnect echo),
/// X must still appear exactly once.
#[test]
fn test_BC_2_05_002_invariant_4_streaming_dedup_of_initial_state_prompt() {
    let mut app = default_app();
    let pid_x = Uuid::new_v4();
    let pid_y = Uuid::new_v4();

    // InitialState with [X, Y]
    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid_x), bash_payload(pid_y)],
        0,
    );

    // Streaming duplicate for X (as if X was re-delivered)
    apply_permission_prompt_queued(&mut app.overlay_stack, bash_payload(pid_x));

    assert_eq!(
        app.overlay_stack.len(),
        2,
        "BC-2.05.002 Invariant 4: streaming dup of X (from InitialState) must be discarded; \
         len stays 2"
    );
}

/// BC-2.06.016 PC-2: Overlay is restored in FIFO order from InitialState.
/// The daemon preserves insertion order in its overlay_stack; the TUI must preserve it too.
#[test]
fn test_BC_2_06_016_pc2_overlay_restored_in_fifo_order() {
    let mut app = default_app();
    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();
    let pid3 = Uuid::new_v4();

    // Reconnect with 3 prompts
    on_initial_state(
        &mut app,
        vec![],
        vec![],
        vec![bash_payload(pid1), bash_payload(pid2), bash_payload(pid3)],
        0,
    );

    let ids: Vec<Uuid> = app.overlay_stack.iter().map(|m| m.prompt_id).collect();
    assert_eq!(
        ids,
        vec![pid1, pid2, pid3],
        "BC-2.06.016 PC-2 / BC-2.06.008 PC-2: restored overlay must preserve FIFO order"
    );
}
