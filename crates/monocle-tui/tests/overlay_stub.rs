//! Failing tests for BC-2.06.015: Permission Overlay `[t]` trace-to-source stub.
//!
//! # RED GATE
//!
//! `Action::PermissionTraceToSource` does not yet exist in `monocle-core/src/tui/state.rs`.
//! This entire file will FAIL TO COMPILE until the implementer adds:
//!   1. `Action::PermissionTraceToSource` variant to the `Action` enum (BC-2.06.015 PC-7)
//!   2. `t → Action::PermissionTraceToSource` in the Builtin Overlay arm of `binding.rs`
//!      (BC-2.06.015 INV-1)
//!   3. A match arm for `Action::PermissionTraceToSource` in `app.rs` `dispatch_key_event`
//!      that sets `app.status_message` (BC-2.06.015 PC-1)
//!
//! The compile error naming the missing `PermissionTraceToSource` variant is the
//! canonical Red Gate signal for this story's TDD cycle.
//!
//! # Coverage
//!
//! - test_BC_2_06_015_handler_sets_status_message_exact_canonical_text
//!   (BC-2.06.015 PC-1 — pressing [t] sets app.status_message to the exact canonical text)
//! - test_BC_2_06_015_handler_sets_status_message_mode_stays_overlay
//!   (BC-2.06.015 PC-2 — AppMode remains Overlay { prior } after [t] press)
//! - test_BC_2_06_015_handler_no_ipc_sent_on_trace_key
//!   (BC-2.06.015 PC-3 — no IPC message sent to daemon on [t] press)
//! - test_BC_2_06_015_ec099_t_in_dashboard_status_message_unchanged
//!   (BC-2.06.015 EC-099 — `t` in Dashboard is unbound; status_message unchanged)
//! - test_BC_2_06_015_production_binding_t_in_overlay_resolves_trace_to_source
//!   (BC-2.06.015 INV-1 — `build_builtin_binding_layers()` wires `t` → PermissionTraceToSource
//!   in the Overlay SearchPrompt arm)
//! - test_BC_2_06_015_ec098_repeated_t_press_idempotent_status_message
//!   (BC-2.06.015 EC-098 — pressing [t] multiple times: each press writes the same string,
//!   no state accumulation)

#![allow(non_snake_case)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use monocle_config::MonocleConfig;
use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::types::{ClientToServer, PermissionPromptPayload};
use monocle_tui::app::{
    apply_permission_prompt_queued, build_builtin_binding_layers, dispatch_key_event, App,
    KeyOutcome,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use tokio::sync::mpsc;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// The EXACT canonical placeholder string mandated by BC-2.06.015 PC-1.
/// Asserted verbatim — not derived; not truncated; must match the production arm.
const TRACE_PLACEHOLDER: &str = "[t] Trace to source — Phase 2 feature (Static plane)";

fn no_mod() -> KeyModifiers {
    KeyModifiers::default()
}

fn key_t() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char('t'),
        modifiers: no_mod(),
    }
}

fn bash_payload(prompt_id: Uuid) -> PermissionPromptPayload {
    PermissionPromptPayload {
        prompt_id,
        session_id: "sess-trace-001".into(),
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({"command": "ls /tmp"}),
        old_content: None,
        new_content: None,
    }
}

/// Build an App in `Overlay { prior: Sessions }` with one modal on the overlay_stack
/// and `ipc_tx` wired to the returned `Receiver`. Enables the no-IPC assertion tests.
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

// ---------------------------------------------------------------------------
// BC-2.06.015 PC-1 — [t] sets app.status_message to EXACT canonical text
// ---------------------------------------------------------------------------

/// BC-2.06.015 PC-1 (RED GATE): Pressing `t` in `AppMode::Overlay` sets
/// `app.status_message` to the exact canonical placeholder string.
///
/// The placeholder text is press-gated: it appears only after `[t]` is pressed,
/// not on every render tick of the overlay (BC-2.06.015 PC-1, mechanism rationale).
///
/// This test FAILS TO COMPILE until `Action::PermissionTraceToSource` is added
/// to the Action enum. At runtime it will fail on the `status_message` assertion
/// until the `dispatch_key_event` match arm is implemented.
///
/// CANONICAL TEXT (BC-2.06.015 PC-1):
///   `"[t] Trace to source — Phase 2 feature (Static plane)"`
///
/// Assert the EXACT string — any whitespace or punctuation difference is a contract
/// violation. The placeholder is user-visible and must be stable.
///
/// Traces to: BC-2.06.015 PC-1.
#[test]
fn test_BC_2_06_015_handler_sets_status_message_exact_canonical_text() {
    let pid = Uuid::new_v4();
    let (mut app, _rx) = overlay_app_with_ipc(pid);

    // Precondition: status_message starts as None (App::new contract).
    assert!(
        app.status_message.is_none(),
        "precondition: app.status_message must be None before [t] press"
    );

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    let outcome = dispatch_key_event(&mut app, &key_t(), &layers, &mut sessions_state);

    // Key outcome: must not quit.
    assert_eq!(
        outcome,
        KeyOutcome::Continue,
        "BC-2.06.015 PC-1: [t] press must return Continue (not Quit)"
    );

    // The exact canonical placeholder text — not a prefix, not a truncation.
    assert_eq!(
        app.status_message.as_deref(),
        Some(TRACE_PLACEHOLDER),
        "BC-2.06.015 PC-1: app.status_message after [t] press must equal the EXACT \
         canonical text: {:?}; got: {:?}",
        TRACE_PLACEHOLDER,
        app.status_message.as_deref()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.015 PC-2 — AppMode remains Overlay { prior } after [t] press
// ---------------------------------------------------------------------------

/// BC-2.06.015 PC-2 (RED GATE): Pressing `t` in Overlay mode is an identity
/// AppMode transition — `AppMode` remains `Overlay { prior }` unchanged and
/// `App.overlay_stack` is not modified.
///
/// Traces to: BC-2.06.015 PC-2 (no AppMode transition), PC-4 (no navigation).
#[test]
fn test_BC_2_06_015_handler_sets_status_message_mode_stays_overlay() {
    let pid = Uuid::new_v4();
    let (mut app, _rx) = overlay_app_with_ipc(pid);

    // Record the prior BEFORE pressing `t`.
    let prior_before = FocusSnapshot::Sessions; // app built with Sessions prior

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(&mut app, &key_t(), &layers, &mut sessions_state);

    // AppMode must still be Overlay — no collapse to Dashboard.
    match &app.mode {
        AppMode::Overlay {
            prior: current_prior,
        } => {
            assert_eq!(
                *current_prior, prior_before,
                "BC-2.06.015 PC-2: prior must be unchanged after [t] press; \
                 expected {:?}, got {:?}",
                prior_before, current_prior
            );
        }
        AppMode::Dashboard { .. } => {
            panic!(
                "BC-2.06.015 PC-2: [t] press must NOT collapse AppMode to Dashboard — \
                 it is an identity transition"
            );
        }
        _ => {
            panic!(
                "BC-2.06.015 PC-2: [t] press must leave AppMode as Overlay; got a different mode"
            );
        }
    }

    // overlay_stack must be unmodified — same length, same front prompt_id.
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.015 PC-2: overlay_stack must NOT be modified by [t] press; \
         expected len=1"
    );
    assert_eq!(
        app.overlay_stack.front().map(|m| m.prompt_id),
        Some(pid),
        "BC-2.06.015 PC-2: overlay_stack front prompt_id must be unchanged after [t] press"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.015 PC-3 — No IPC message sent on [t] press
// ---------------------------------------------------------------------------

/// BC-2.06.015 PC-3 (RED GATE): Pressing `t` in Overlay mode sends NO IPC message
/// to the daemon. The mutation is a local `App.status_message` write only.
///
/// Uses the wired `ipc_tx` channel and asserts `try_recv()` returns `Err`
/// (empty channel) — the same pattern used by BC-2.06.014 Esc no-IPC test.
///
/// Traces to: BC-2.06.015 PC-3 (no IPC message sent).
#[test]
fn test_BC_2_06_015_handler_no_ipc_sent_on_trace_key() {
    let pid = Uuid::new_v4();
    let (mut app, mut rx) = overlay_app_with_ipc(pid);

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(&mut app, &key_t(), &layers, &mut sessions_state);

    // The IPC channel must be empty — no PermissionDecision or other message sent.
    assert!(
        rx.try_recv().is_err(),
        "BC-2.06.015 PC-3: [t] press must NOT send any IPC message to the daemon; \
         the channel must remain empty (status_message is a local mutation only)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.015 EC-099 — `t` in Dashboard is unbound; status_message unchanged
// ---------------------------------------------------------------------------

/// BC-2.06.015 EC-099 (RED GATE — compile gate, runtime pass): Pressing `t` in
/// `AppMode::Dashboard` does not resolve to `PermissionTraceToSource`.
///
/// `t` is Overlay-only. In Dashboard mode the key either resolves to `None`
/// (no binding) or to some other Dashboard action. `status_message` must
/// remain unchanged after the press.
///
/// This test compiles only when `Action::PermissionTraceToSource` exists (same
/// compile-gate as the other tests). At runtime, with the production binding layers,
/// `t` in Dashboard returns `None` → `status_message` unchanged.
///
/// Traces to: BC-2.06.015 EC-099 (Dashboard: identity; `t` unbound).
#[test]
fn test_BC_2_06_015_ec099_t_in_dashboard_status_message_unchanged() {
    let mut app = App::new(MonocleConfig::default());
    // App::new starts in Dashboard { focused: Sessions }.
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "precondition: App::new must start in Dashboard {{ Sessions }}"
    );

    // Precondition: status_message is None.
    assert!(
        app.status_message.is_none(),
        "precondition: status_message must be None before `t` press in Dashboard"
    );

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(&mut app, &key_t(), &layers, &mut sessions_state);

    // status_message must remain None — `t` in Dashboard is unbound.
    assert!(
        app.status_message.is_none(),
        "BC-2.06.015 EC-099: `t` in Dashboard must NOT set status_message; \
         Dashboard has no binding for PermissionTraceToSource; \
         got: {:?}",
        app.status_message.as_deref()
    );

    // AppMode must remain Dashboard — no unintended transition.
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "BC-2.06.015 EC-099: AppMode must remain Dashboard after `t` press in Dashboard mode"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.015 INV-1 — production binding layer wires `t` → PermissionTraceToSource
// in the Overlay arm
// ---------------------------------------------------------------------------

/// BC-2.06.015 INV-1 (RED GATE): `build_builtin_binding_layers()` must register
/// `t → Action::PermissionTraceToSource` in the Overlay binding arm (SearchPrompt layer).
///
/// This tests the PRODUCTION binding path via `dispatch_key_event` with a wired
/// `build_builtin_binding_layers()` call — not an empty-layers surrogate.
///
/// The assertion is that `app.status_message` is set to `TRACE_PLACEHOLDER` after
/// pressing `t` in Overlay mode using the production layers (same layers used by `run()`).
/// If `build_builtin_binding_layers()` does not register `t`, the action resolves to
/// `Noop` and `status_message` remains `None`.
///
/// This test fails AT RUNTIME (not just at compile time) until both:
///   1. `Action::PermissionTraceToSource` exists (compile gate), AND
///   2. `build_builtin_binding_layers()` registers `t → PermissionTraceToSource` in
///      the Overlay arm (INV-1: Builtin binding).
///
/// Traces to: BC-2.06.015 INV-1 (Builtin binding, non-overridable in Phase 1).
#[test]
fn test_BC_2_06_015_production_binding_t_in_overlay_resolves_trace_to_source() {
    let pid = Uuid::new_v4();
    let (mut app, _rx) = overlay_app_with_ipc(pid);

    // Use the production binding layers — same as run().
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    dispatch_key_event(&mut app, &key_t(), &layers, &mut sessions_state);

    // The production binding must have resolved `t` → PermissionTraceToSource,
    // which sets status_message. If `t` is not registered in the Overlay arm,
    // the action defaults to Noop and status_message remains None.
    assert_eq!(
        app.status_message.as_deref(),
        Some(TRACE_PLACEHOLDER),
        "BC-2.06.015 INV-1: pressing `t` in Overlay with production layers must set \
         app.status_message to the canonical placeholder; \
         got: {:?} — is `t` registered in build_builtin_binding_layers() Overlay arm?",
        app.status_message.as_deref()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.015 EC-098 — repeated [t] presses are idempotent (no state accumulation)
// ---------------------------------------------------------------------------

/// BC-2.06.015 EC-098 (RED GATE): Pressing `t` multiple times in sequence always
/// results in the same `status_message` content — no state accumulation, no error.
///
/// The last-write-wins semantics of `App.status_message` (BC-2.06.015 INV-4) mean
/// each press simply overwrites with the same string. The overlay_stack must also
/// remain unchanged (same length, same front prompt_id) across all presses.
///
/// Traces to: BC-2.06.015 EC-098 (repeated [t] presses), INV-4 (no trace_pressed
/// field; status_message last-write-wins).
#[test]
fn test_BC_2_06_015_ec098_repeated_t_press_idempotent_status_message() {
    let pid = Uuid::new_v4();
    let (mut app, mut rx) = overlay_app_with_ipc(pid);

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Press `t` three times in sequence (BC-2.06.015 EC-098 test vector).
    for press_count in 1..=3 {
        let outcome = dispatch_key_event(&mut app, &key_t(), &layers, &mut sessions_state);

        assert_eq!(
            outcome,
            KeyOutcome::Continue,
            "BC-2.06.015 EC-098: press #{press_count} must return Continue"
        );

        // status_message must be the exact canonical text on every press.
        assert_eq!(
            app.status_message.as_deref(),
            Some(TRACE_PLACEHOLDER),
            "BC-2.06.015 EC-098: press #{press_count} must set status_message to canonical text; \
             got: {:?}",
            app.status_message.as_deref()
        );

        // overlay_stack must remain unchanged through all presses.
        assert_eq!(
            app.overlay_stack.len(),
            1,
            "BC-2.06.015 EC-098: overlay_stack.len() must remain 1 after press #{press_count}"
        );
        assert_eq!(
            app.overlay_stack.front().map(|m| m.prompt_id),
            Some(pid),
            "BC-2.06.015 EC-098: overlay_stack front prompt_id must be unchanged after press #{press_count}"
        );

        // No IPC message on any press (PC-3 repeated).
        assert!(
            rx.try_recv().is_err(),
            "BC-2.06.015 EC-098 + PC-3: press #{press_count} must NOT send any IPC message"
        );
    }

    // AppMode must still be Overlay after 3 presses — no accidental transition.
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.015 EC-098: AppMode must remain Overlay after 3 [t] presses"
    );
}
