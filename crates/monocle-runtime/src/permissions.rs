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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use monocle_ipc::types::{PermissionDecisionKind, PermissionPromptPayload};

/// A single pending-decision entry in the registry.
///
/// Each in-flight permission prompt maps to one `PendingEntry`. The `payload` field
/// is cloned into `InitialState.overlay_stack` when a new TUI client connects
/// (BC-2.05.002 PC-2 / AC-015 EC-003).
///
/// The `sender` is consumed exactly once by [`PendingDecisionRegistry::resolve_prompt`].
/// After resolution, the entry is removed from the registry.
#[allow(dead_code)]
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
        let count = self
            .inner
            .lock()
            .map(|g| g.len())
            .unwrap_or(0);
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

    /// Register a new in-flight permission prompt and return its stable `prompt_id`.
    ///
    /// # Contract (BC-2.05.005 postcondition PC-1)
    ///
    /// - Generates a fresh `Uuid::new_v4()` as the `prompt_id`.
    /// - Inserts `(payload, sender)` into the registry keyed by `prompt_id`.
    /// - Returns the `prompt_id` so the caller can include it in the
    ///   `PermissionPromptQueued { payload }` broadcast.
    /// - The `prompt_id` is stable: it will appear identically in `PermissionPromptResolved`.
    ///
    /// # Parameters
    ///
    /// - `payload`: The full `PermissionPromptPayload` to store (will also be broadcast
    ///   to TUI clients and included in `InitialState.overlay_stack` for late-connecting
    ///   clients).
    /// - `sender`: The `oneshot::Sender<PermissionDecisionKind>` that the HTTP handler is
    ///   awaiting on its receiver end.
    #[allow(clippy::todo)]
    pub fn register_prompt(
        &self,
        _payload: PermissionPromptPayload,
        _sender: tokio::sync::oneshot::Sender<PermissionDecisionKind>,
    ) -> Uuid {
        todo!("S-022: register_prompt — generate prompt_id, insert into registry, return id")
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
    #[allow(clippy::todo)]
    pub fn resolve_prompt(
        &self,
        _prompt_id: Uuid,
        _decision: PermissionDecisionKind,
    ) -> Option<PermissionPromptPayload> {
        todo!("S-022: resolve_prompt — remove entry, send decision on oneshot, return payload")
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
    #[allow(clippy::todo)]
    pub fn remove_timed_out_prompt(&self, _prompt_id: Uuid) -> Option<PermissionPromptPayload> {
        todo!("S-022: remove_timed_out_prompt — remove stale entry, return payload for broadcast")
    }

    /// Return a snapshot of all currently-pending prompt payloads.
    ///
    /// Used by `snapshot_initial_state` to populate `InitialState.overlay_stack`
    /// (BC-2.05.002 PC-2 — `overlay_stack: Vec<PermissionPromptPayload>`).
    ///
    /// The snapshot is a clone of all payloads at the moment of the call. Prompts
    /// resolved between snapshot time and `InitialState` delivery will generate a
    /// subsequent `PermissionPromptResolved` message (AC-006 no-gap-window guarantee).
    #[allow(clippy::todo)]
    pub fn snapshot_payloads(&self) -> Vec<PermissionPromptPayload> {
        todo!("S-022: snapshot_payloads — clone all pending payloads for InitialState.overlay_stack")
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
