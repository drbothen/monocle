//! Pending-decision registry for permission prompts (S-022, BC-2.05.005).
//!
//! Tracks in-flight `PreToolUse` permission prompts that are awaiting a TUI client
//! decision. Each entry holds a [`monocle_ipc::types::PermissionPromptPayload`] (for
//! inclusion in `InitialState.overlay_stack` when new TUI clients connect) and a
//! `oneshot::Sender<PermissionDecisionKind>` (to route the user's decision back to the
//! HTTP handler holding the response open).
//!
//! # At-Most-One Resolution Invariant (BC-2.05.005 invariant 2)
//!
//! The `oneshot::channel` per prompt enforces at-most-one resolution. The first
//! `PermissionDecision` to arrive resolves the channel; subsequent decisions for the same
//! `prompt_id` find no registry entry and are silently discarded.
//!
//! # Lock Ordering (BC-2.05.005 / S-022 architecture compliance)
//!
//! When acquiring both the pending-decision registry lock and the fan-out subscriber list
//! lock, ALWAYS acquire the pending-decision registry lock FIRST to prevent deadlock.
//! (Do NOT hold the subscriber list lock while acquiring the registry lock.)
// Mutex::lock() returns Err only on mutex poison (a previous holder panicked while holding
// the lock). Panicking on poison is the correct response — the registry state is undefined
// and continuing would risk data corruption. We allow expect_used here for this pattern only.
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use monocle_ipc::types::{PermissionDecisionKind, PermissionPromptPayload, PromptPayloadInputs};

/// A single pending-decision entry in the registry.
///
/// Each in-flight permission prompt maps to one `PendingEntry`. The `payload` field
/// is cloned into `InitialState.overlay_stack` when a new TUI client connects
/// (BC-2.05.002 PC-2 / AC-015 EC-003).
///
/// The `sender` is consumed exactly once by [`PendingDecisionRegistry::resolve_prompt`].
/// After resolution, the entry is removed from the registry.
struct PendingEntry {
    /// Full payload for this prompt (stable for the lifetime of the entry).
    pub payload: PermissionPromptPayload,
    /// One-shot sender resolved by the first `PermissionDecision` to arrive.
    pub sender: tokio::sync::oneshot::Sender<PermissionDecisionKind>,
}

/// Registry of in-flight permission prompts awaiting TUI client decisions.
///
/// Protected by an outer `Arc<Mutex<...>>` so the registry can be shared across
/// axum handler tasks, the per-client IPC task, and the timeout handler without
/// requiring async locking (the critical sections are very short).
///
/// # Lock discipline
///
/// All public methods acquire the inner `Mutex` only for the minimum duration needed
/// to perform the map operation and release. No async operations are performed while
/// the lock is held.
pub struct PendingDecisionRegistry {
    /// Inner map from `prompt_id` to `PendingEntry`.
    ///
    /// Protected by `Mutex` (sync, not tokio) because all critical sections are
    /// pure data-structure operations (insert / remove / clone), never async awaits.
    inner: Mutex<HashMap<Uuid, PendingEntry>>,
}

impl std::fmt::Debug for PendingDecisionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Only expose the count of pending entries — do not expose payloads or senders.
        let count = self.inner.lock().map(|g| g.len()).unwrap_or(0);
        f.debug_struct("PendingDecisionRegistry")
            .field("pending_count", &count)
            .finish()
    }
}

impl PendingDecisionRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        PendingDecisionRegistry {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new in-flight permission prompt and return its assigned `prompt_id`
    /// together with the complete `PermissionPromptPayload` (F-ADV2-MED-001).
    ///
    /// # Contract (BC-2.05.005 postcondition PC-1)
    ///
    /// - Generates a fresh `Uuid::new_v4()` as the `prompt_id`.
    /// - Constructs a [`PermissionPromptPayload`] from `inputs` with the assigned `prompt_id`.
    /// - Inserts `(payload, sender)` into the registry keyed by `prompt_id`.
    /// - Returns `(prompt_id, payload)` so the caller can broadcast
    ///   `PermissionPromptQueued { payload }` and record the `prompt_id` for cleanup.
    /// - The `prompt_id` is stable: it will appear identically in `PermissionPromptResolved`.
    ///
    /// # API contract clarification (F-ADV2-MED-001)
    ///
    /// Callers pass [`PromptPayloadInputs`] (no `prompt_id` field). The registry assigns the
    /// UUID and returns the complete payload. This eliminates the previous silent-overwrite
    /// pattern where a caller-supplied placeholder was discarded.
    ///
    /// # Parameters
    ///
    /// - `inputs`: The caller-controlled fields for the prompt (session_id, tool_name, etc.).
    ///   No `prompt_id` — the registry owns that.
    /// - `sender`: The `oneshot::Sender<PermissionDecisionKind>` that the HTTP handler is
    ///   awaiting on its receiver end.
    ///
    /// # Returns
    ///
    /// `(prompt_id, payload)` — the caller broadcasts `payload` to TUI clients and stores
    /// `prompt_id` for the timeout cleanup path.
    pub fn register_prompt(
        &self,
        inputs: PromptPayloadInputs,
        sender: tokio::sync::oneshot::Sender<PermissionDecisionKind>,
    ) -> (Uuid, PermissionPromptPayload) {
        let prompt_id = Uuid::new_v4();
        // Build the complete payload with the registry-assigned prompt_id.
        // This ensures snapshot_payloads() returns the correct prompt_id for late-connecting TUI
        // clients (BC-2.05.002 AC-002 — overlay_stack uses the real UUID, not Uuid::nil()).
        let payload = PermissionPromptPayload {
            prompt_id,
            session_id: inputs.session_id,
            tool_name: inputs.tool_name,
            tool_input: inputs.tool_input,
            old_content: inputs.old_content,
            new_content: inputs.new_content,
        };
        let entry = PendingEntry {
            payload: payload.clone(),
            sender,
        };
        let mut map = self
            .inner
            .lock()
            .expect("PendingDecisionRegistry mutex poisoned");
        map.insert(prompt_id, entry);
        (prompt_id, payload)
    }

    /// Resolve a pending permission prompt with the given user decision.
    ///
    /// # Contract (BC-2.05.005 postcondition PC-3 / invariant 2)
    ///
    /// - If `prompt_id` is found: removes the entry from the registry; consumes the
    ///   `oneshot::Sender` by calling `sender.send(decision)`. Ignores the send result
    ///   (the receiver may have been dropped by a prior timeout, which is acceptable —
    ///   the at-most-one invariant is satisfied by the entry removal).
    ///   Returns `Some(payload)` so the caller can broadcast `PermissionPromptResolved`
    ///   and obtain the original payload for logging.
    /// - If `prompt_id` is NOT found (second resolution attempt or already timed out):
    ///   the call is a no-op. Returns `None`. The caller silently discards the message
    ///   (BC-2.05.005 postcondition PC-3 — second decision silently discarded).
    ///
    /// # Returns
    ///
    /// `Some(PermissionPromptPayload)` on first resolution; `None` on subsequent
    /// resolution attempts for the same `prompt_id`.
    pub fn resolve_prompt(
        &self,
        prompt_id: Uuid,
        decision: PermissionDecisionKind,
    ) -> Option<PermissionPromptPayload> {
        let entry = {
            let mut map = self
                .inner
                .lock()
                .expect("PendingDecisionRegistry mutex poisoned");
            map.remove(&prompt_id)
        };
        entry.map(|e| {
            // Ignore send result: receiver may have been dropped (e.g. timeout already
            // resolved the HTTP response). The at-most-one invariant is satisfied by the
            // registry removal above.
            let _ = e.sender.send(decision);
            e.payload
        })
    }

    /// Remove a timed-out prompt from the registry.
    ///
    /// # Contract (BC-2.05.005 postcondition PC-4 — timeout path)
    ///
    /// Called by the `PreToolUse` timeout handler (300ms) after the daemon has resolved
    /// the HTTP response with fail-open semantics and wants to:
    /// 1. Remove the stale registry entry (so late `PermissionDecision` messages are discarded).
    /// 2. Return the `PermissionPromptPayload` to the caller so it can broadcast
    ///    `PermissionPromptResolved { prompt_id }` to all connected TUI clients
    ///    (BC-2.05.005 postcondition PC-4, second bullet — timeout MUST send Resolved).
    ///
    /// If `prompt_id` is not found (already resolved by a TUI client before timeout fired),
    /// this is a no-op and returns `None`.
    ///
    /// # Returns
    ///
    /// `Some(PermissionPromptPayload)` if the entry was present; `None` if already removed.
    pub fn remove_timed_out_prompt(&self, prompt_id: Uuid) -> Option<PermissionPromptPayload> {
        let mut map = self
            .inner
            .lock()
            .expect("PendingDecisionRegistry mutex poisoned");
        map.remove(&prompt_id).map(|e| e.payload)
    }

    /// Return a snapshot of all currently-pending prompt payloads.
    ///
    /// Used by `snapshot_initial_state` to populate `InitialState.overlay_stack`
    /// (BC-2.05.002 PC-2 — `overlay_stack: Vec<PermissionPromptPayload>`).
    ///
    /// The snapshot is a clone of all payloads at the moment of the call. Prompts
    /// resolved between snapshot time and `InitialState` delivery will generate a
    /// subsequent `PermissionPromptResolved` message (AC-006 no-gap-window guarantee).
    pub fn snapshot_payloads(&self) -> Vec<PermissionPromptPayload> {
        let map = self
            .inner
            .lock()
            .expect("PendingDecisionRegistry mutex poisoned");
        map.values().map(|e| e.payload.clone()).collect()
    }
}

impl Default for PendingDecisionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared `PendingDecisionRegistry` handle (Arc-wrapped for multi-owner sharing).
///
/// Used as the type of `DaemonState.pending_decisions` so all axum handler tasks,
/// the per-client IPC task, and the timeout handler share a single registry instance
/// without cloning the struct itself.
pub type SharedPendingDecisionRegistry = Arc<PendingDecisionRegistry>;

#[cfg(test)]
// Allow BC-based test naming: test_BC_S_SS_NNN_xxx — uppercase BC identifier is intentional.
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn make_inputs(tag: &str) -> PromptPayloadInputs {
        PromptPayloadInputs {
            session_id: format!("session-{tag}"),
            tool_name: "Edit".to_string(),
            tool_input: serde_json::json!({"tag": tag}),
            old_content: None,
            new_content: None,
        }
    }

    /// test_BC_2_05_005_register_prompt_returns_uuid_and_stores_entry:
    /// register_prompt assigns a fresh UUID, inserts the entry, and
    /// snapshot_payloads returns a payload whose prompt_id matches the returned UUID.
    /// Traces to BC-2.05.005 postcondition PC-1.
    #[test]
    fn test_BC_2_05_005_register_prompt_returns_uuid_and_stores_entry() {
        let registry = PendingDecisionRegistry::new();
        let (tx, _rx) = tokio::sync::oneshot::channel::<PermissionDecisionKind>();

        let (prompt_id, returned_payload) = registry.register_prompt(make_inputs("reg"), tx);

        // The returned payload must carry the same prompt_id the registry assigned.
        assert_eq!(
            returned_payload.prompt_id, prompt_id,
            "returned payload prompt_id must match the assigned UUID"
        );

        // snapshot_payloads must include exactly one entry matching prompt_id.
        let payloads = registry.snapshot_payloads();
        assert_eq!(
            payloads.len(),
            1,
            "registry must contain exactly one entry after register"
        );
        assert_eq!(
            payloads[0].prompt_id, prompt_id,
            "snapshot payload prompt_id must match"
        );
    }

    /// test_BC_2_05_005_resolve_prompt_returns_some_and_removes_entry:
    /// The first resolve_prompt call for a registered id returns Some(payload) and
    /// removes the entry. A second call for the same id returns None (silently discarded).
    /// Traces to BC-2.05.005 postcondition PC-3 / invariant 2.
    #[test]
    fn test_BC_2_05_005_resolve_prompt_returns_some_and_removes_entry() {
        let registry = PendingDecisionRegistry::new();
        let (tx, _rx) = tokio::sync::oneshot::channel::<PermissionDecisionKind>();

        let (prompt_id, _payload) = registry.register_prompt(make_inputs("resolve"), tx);

        // First resolution: must return Some and remove the entry.
        let result = registry.resolve_prompt(prompt_id, PermissionDecisionKind::Allow);
        assert!(
            result.is_some(),
            "first resolve_prompt must return Some(payload)"
        );
        assert_eq!(
            result
                .expect("resolve_prompt returned None after is_some check")
                .prompt_id,
            prompt_id,
            "resolved payload prompt_id must match"
        );

        // Entry must be gone from the snapshot.
        let payloads = registry.snapshot_payloads();
        assert!(
            payloads.is_empty(),
            "registry must be empty after first resolution"
        );

        // Second resolution attempt: must return None (at-most-one invariant).
        let second = registry.resolve_prompt(prompt_id, PermissionDecisionKind::Deny);
        assert!(
            second.is_none(),
            "second resolve_prompt for same prompt_id must return None"
        );
    }

    /// test_BC_2_05_005_remove_timed_out_prompt_removes_entry:
    /// remove_timed_out_prompt returns Some(payload) and removes the entry from the
    /// registry. snapshot_payloads is empty after the call.
    /// Traces to BC-2.05.005 postcondition PC-4.
    #[test]
    fn test_BC_2_05_005_remove_timed_out_prompt_removes_entry() {
        let registry = PendingDecisionRegistry::new();
        let (tx, _rx) = tokio::sync::oneshot::channel::<PermissionDecisionKind>();

        let (prompt_id, _payload) = registry.register_prompt(make_inputs("timeout"), tx);

        // remove_timed_out_prompt must return Some and remove the entry.
        let result = registry.remove_timed_out_prompt(prompt_id);
        assert!(
            result.is_some(),
            "remove_timed_out_prompt must return Some(payload) for a registered prompt"
        );
        assert_eq!(
            result
                .expect("remove_timed_out_prompt returned None after is_some check")
                .prompt_id,
            prompt_id,
            "removed payload prompt_id must match"
        );

        // Registry must be empty after removal.
        let payloads = registry.snapshot_payloads();
        assert!(
            payloads.is_empty(),
            "registry must be empty after remove_timed_out_prompt"
        );
    }

    /// test_BC_2_05_005_remove_timed_out_prompt_idempotent:
    /// Calling remove_timed_out_prompt twice for the same prompt_id is a no-op on the
    /// second call: returns None without panicking.
    /// Traces to BC-2.05.005 postcondition PC-4 (if already resolved, no-op / None).
    #[test]
    fn test_BC_2_05_005_remove_timed_out_prompt_idempotent() {
        let registry = PendingDecisionRegistry::new();
        let (tx, _rx) = tokio::sync::oneshot::channel::<PermissionDecisionKind>();

        let (prompt_id, _payload) = registry.register_prompt(make_inputs("idem"), tx);

        // First removal succeeds.
        let first = registry.remove_timed_out_prompt(prompt_id);
        assert!(
            first.is_some(),
            "first remove_timed_out_prompt must return Some"
        );

        // Second removal on same id: must return None and must NOT panic.
        let second = registry.remove_timed_out_prompt(prompt_id);
        assert!(
            second.is_none(),
            "second remove_timed_out_prompt for same prompt_id must return None"
        );
    }

    /// test_BC_2_05_005_snapshot_payloads_cloning:
    /// snapshot_payloads returns independent clones — mutating the returned Vec does
    /// not affect the registry state.
    /// Traces to BC-2.05.005 / overlay_stack snapshot contract (AC-015 EC-003).
    #[test]
    fn test_BC_2_05_005_snapshot_payloads_cloning() {
        let registry = PendingDecisionRegistry::new();
        let (tx1, _rx1) = tokio::sync::oneshot::channel::<PermissionDecisionKind>();
        let (tx2, _rx2) = tokio::sync::oneshot::channel::<PermissionDecisionKind>();

        let (id1, _) = registry.register_prompt(make_inputs("snap1"), tx1);
        let (id2, _) = registry.register_prompt(make_inputs("snap2"), tx2);

        // Capture snapshot, then clear the returned Vec.
        let mut snapshot = registry.snapshot_payloads();
        assert_eq!(snapshot.len(), 2, "snapshot must contain both entries");

        // Mutate the snapshot clone.
        snapshot.clear();

        // Registry must be unaffected — snapshot_payloads still returns both entries.
        let still_there = registry.snapshot_payloads();
        assert_eq!(
            still_there.len(),
            2,
            "registry must be unaffected by clearing the snapshot Vec"
        );
        let ids: Vec<Uuid> = still_there.iter().map(|p| p.prompt_id).collect();
        assert!(ids.contains(&id1), "id1 must still be in registry");
        assert!(ids.contains(&id2), "id2 must still be in registry");
    }
}
