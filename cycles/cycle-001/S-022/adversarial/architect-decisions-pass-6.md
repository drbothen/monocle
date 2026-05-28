---
document_type: architect-decision
story: S-022
pass: 6
producer: vsdd-factory:architect
timestamp: 2026-05-28T00:00:00Z
findings_addressed:
  - F-S022-ADV6-MED-001
---

# S-022 Architect Decisions — Adversarial Pass 6

## Context

F-S022-ADV6-MED-001 surfaces a genuine BC contradiction between BC-2.05.002 Invariant 3
(AC-006, "no gap window") and BC-2.05.002 EC-005 ("no duplicate events; no gap"). The
adversary's trace is correct: the register-subscriber-before-snapshot ordering can produce a
`PermissionPromptQueued` message that is already present in `InitialState.overlay_stack`,
delivering the same prompt to the TUI twice via two different code paths.

This document adjudicates the contradiction and produces the canonical resolution.

---

## F-S022-ADV6-MED-001: Duplicate PermissionPromptQueued Window

### Issue Summary

The connection handshake sequence in `ipc_server.rs` registers the new client's `tx` in the
subscribers list, then takes the state snapshot, then sends `InitialState`. Between registration
and snapshot, a concurrent `PermissionPromptQueued` broadcast can write the new prompt into
BOTH the client's mpsc channel AND the `pending_decisions` registry. The snapshot reads the
registry; the initial state push includes the new prompt in `overlay_stack`. The streaming send
loop also drains the mpsc channel; the `PermissionPromptQueued` message is delivered a second
time. TUI receives the same prompt twice.

**BC-2.05.002 AC-006 (Invariant 3):** Register subscriber before snapshot. Events that arrive
after connection is accepted but before `InitialState` is fully written are delivered as
subsequent incremental updates. No gap window.

**BC-2.05.002 EC-005:** "No duplicate events; no gap."

The adversary correctly identifies that both cannot be satisfied simultaneously when concurrent
state mutations occur under the current ordering. This is a genuine contradiction that requires
architect adjudication.

### Options Considered

**Option A — Snapshot-epoch dedup at TUI side:**
Add `snapshot_epoch: u64` to `InitialState` and to all push messages; TUI discards messages
with epoch <= snapshot epoch.
- Pros: Cleanly handles the race; idempotent by construction.
- Cons: Protocol change to all message types (BC-2.05.003, BC-2.05.004, BC-2.05.005). S-026
  must implement epoch tracking across all message types. Wire format breaks backward compat
  for future clients that do not implement epoch tracking.

**Option B — Snapshot-epoch dedup at daemon per-client task:**
Per-client task records the snapshot's `pending_decisions` epoch and discards streaming
`PermissionPromptQueued` messages whose `prompt_id` was already in the snapshot.
- Pros: TUI stays simple.
- Cons: Daemon task must store snapshot set; dedup logic diverges per message type
  (`prompt_id` for prompts, fabricated keys for hooks and sessions). Fragile across future
  message type additions.

**Option C — Global "snapshot in progress" lock:**
Serialize register+snapshot+send; no concurrent broadcasts during this window.
- Pros: Zero new fields; eliminates the race by construction.
- Cons: Head-of-line blocking for concurrent connects under load. All broadcasts stall
  while a client snapshots 256 KiB. Violates the daemon's bounded-latency invariants for
  hook event delivery.

**Option D — Accept wire-level duplicate; mandate TUI `prompt_id` idempotency:**
Document that the IPC layer delivers at-least-once semantics for `PermissionPromptQueued`.
Clarify EC-005 to mean "no semantic state duplication" (the TUI's rendered overlay contains
each prompt exactly once), not "no wire-level duplicate messages." Mandate TUI-side dedup via
`prompt_id` as an explicit architectural invariant.
- Pros: Zero daemon code change; zero protocol change; no fragile per-message-type dedup logic.
  The `VecDeque<PromptModal>` is already keyed by `prompt_id` (SS-ipc.md §Risk Mitigations
  "Multiple TUI Clients Resolving the Same Prompt"; `PermissionPromptResolved` no-op if absent).
  Idempotency on insert mirrors the existing no-op on remove.
- Cons: Wire delivers at-least-once for `PermissionPromptQueued` across the snapshot window.
  The documentation of EC-005 requires clarification to avoid future misreading.

### Decision: Option D

**Chosen:** Option D — mandate TUI `prompt_id` idempotency; clarify EC-005 scope.

**Rationale:**

The only message type that produces a structural duplicate under this race is
`PermissionPromptQueued`. The other message types are structurally immune:

1. **`HookEventReceived` streaming** carries a new hook event that arrived after the snapshot
   was taken. `InitialState.ring_tail` contains events up to and including the connection
   moment; `HookEventReceived` delivers events after it. These windows are non-overlapping by
   construction (the ring snapshot is taken from `ring_buffer.latest_events(N)` at snapshot
   time; subsequent events are broadcast as incremental messages). No structural duplicate.

2. **`SessionListUpdate`** contains the full session roster. Whether the TUI processes it
   before or after `InitialState`, the final state is the same full roster — idempotent by
   design (full replace, not delta).

3. **`DropCounterUpdate`** is also a full state value, not a delta. Processing it twice
   produces the same state.

4. **`PermissionPromptQueued`** carries a `PermissionPromptPayload` with a stable `prompt_id`
   (Uuid). Receiving the same `prompt_id` twice is the only genuine duplicate risk. The TUI
   must render this prompt exactly once regardless of how many times the wire delivers it.

The `VecDeque<PromptModal>` overlay stack is already implicitly idempotent-on-insert: SS-ipc.md
§Risk Mitigations (§Multiple TUI Clients Resolving the Same Prompt) documents that
`PermissionPromptResolved` is a no-op if the `prompt_id` is already absent from the VecDeque.
The symmetric invariant — inserting the same `prompt_id` twice is a no-op on the second insert
— follows from the same `prompt_id`-keyed design. Making this explicit in BC-2.05.002 and
SS-ipc.md closes the gap without protocol changes.

Options A and B both require new protocol fields and per-message-type dedup logic that is
fragile under future message type additions. Option C introduces unacceptable head-of-line
blocking for a race window that is already correctly handled by TUI-side idempotency. None of
them provide correctness guarantees stronger than Option D for the actual failure mode (duplicate
overlay prompt rendering), because the TUI must handle `prompt_id` idempotency anyway:
- Multi-client scenarios (BC-2.05.002 EC-002) require idempotent overlay state when two clients
  connect simultaneously and one resolves a prompt.
- `PermissionPromptResolved` for an already-absent prompt must be a no-op (explicitly documented).
- The symmetric insert-idempotency invariant is therefore architecturally necessary regardless
  of this race; Option D simply codifies what is already required.

**Justification against CLAUDE.md Production-Grade Principle:**
CLAUDE.md Principle 1 forbids rationalizations. This decision is NOT "we can fix it later" or
"good enough for v1." It is: the correct production-grade design for at-least-once push
delivery is idempotent state application on the consumer side. Every production message bus
(Kafka, SNS, SQS) mandates consumer idempotency for this reason. The alternative (Option C)
sacrifices liveness for exactly-once semantics, which is the wrong trade for a high-throughput
local IPC path.

**Justification against CLAUDE.md Production-Grade Principle Rule 5:**
"Suggest is acceptable; default to cheap path is not." Option D is NOT the cheap path — it is
the architecturally correct path. The cheap path would be to ignore the finding. Option D
mandates an explicit spec change (BC-2.05.002 EC-005 clarification), an explicit SS-ipc.md
invariant addition, and an explicit TUI implementer directive. All three are production-grade
outputs.

---

## Implementer Directive

**Scope:** `monocle-tui` crate (S-025 or S-026, whichever first implements the overlay stack
`VecDeque<PromptModal>`). The daemon-side code (`ipc_server.rs`) requires NO CHANGES.

### Changes Required in Worktree

**1. `crates/monocle-tui/src/` — wherever `VecDeque<PromptModal>` is populated**

When processing `ServerToClient::PermissionPromptQueued { payload }`, the TUI MUST check
whether `payload.prompt_id` is already present in the `VecDeque<PromptModal>` before
inserting. If already present, silently discard (no duplicate push to the VecDeque, no
log noise at INFO; TRACE level is acceptable if diagnostics are desired).

```rust
// Production-grade insert: idempotent on prompt_id
fn apply_permission_prompt_queued(
    overlay: &mut VecDeque<PromptModal>,
    payload: PermissionPromptPayload,
) {
    if overlay.iter().any(|m| m.prompt_id == payload.prompt_id) {
        tracing::trace!(
            prompt_id = %payload.prompt_id,
            "PermissionPromptQueued: prompt_id already in overlay stack; discarding duplicate"
        );
        return;
    }
    overlay.push_back(PromptModal::from(payload));
}
```

**2. `crates/monocle-tui/src/` — wherever `InitialState.overlay_stack` is applied**

When populating `VecDeque<PromptModal>` from `InitialState.overlay_stack`, use the same
`apply_permission_prompt_queued` helper (or an equivalent idempotent insert). This ensures
that if `InitialState` is applied after a streaming `PermissionPromptQueued` has already
populated the VecDeque (e.g., on reconnect with partial state), no duplication occurs.

**3. Test: `crates/monocle-ipc/tests/` or `crates/monocle-tui/tests/`**

Add an integration test that exercises the snapshot-window race:
1. Daemon is running with one queued prompt (prompt_id=X) in `pending_decisions`.
2. A second prompt (prompt_id=Y) arrives concurrent with a new TUI connecting.
3. TUI receives `InitialState.overlay_stack` containing both X and Y (snapshot includes Y
   because it entered pending_decisions before snapshot was taken).
4. TUI also receives `PermissionPromptQueued { payload: Y }` from the streaming path.
5. Assert: TUI's `VecDeque<PromptModal>` contains prompt X and prompt Y exactly once each.

**Test name:** `test_snapshot_window_prompt_dedup` in `connection_handshake.rs` or a new
`overlay_idempotency.rs` test file.

**4. No daemon-side changes required.** `ipc_server.rs` register-before-snapshot ordering is
CORRECT and must NOT be changed. The at-least-once delivery is by design; consumer-side
idempotency is the correct resolution.

---

## Spec Updates Required

### 1. BC-2.05.002 v1.0.5 — EC-005 clarification + idempotency invariant

**File:** `.factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md`

**EC-005 wording change:**

Old (EC-005):
```
| EC-005 | Daemon is under high load (many hook events per second) when TUI connects | `InitialState` uses the ring_tail snapshot at connection time; incremental `HookEventReceived` messages deliver events that arrived after snapshot. No duplicate events; no gap. |
```

New (EC-005):
```
| EC-005 | Daemon is under high load (many hook events per second) when TUI connects | `InitialState` uses the ring_tail snapshot at connection time; incremental `HookEventReceived` messages deliver events that arrived after snapshot. `HookEventReceived` and `ring_tail` cover non-overlapping time windows by construction — no gap, no structural duplicate. |
```

**Add new Invariant 4** (in the Invariants section, after Invariant 3):
```
4. The TUI MUST apply `PermissionPromptQueued` messages with idempotent-on-`prompt_id`
   semantics: if a `prompt_id` is already present in the local overlay stack (either from
   `InitialState.overlay_stack` or from a prior `PermissionPromptQueued` message), the
   second delivery MUST be silently discarded. The IPC layer provides at-least-once delivery
   for `PermissionPromptQueued` across the snapshot window; consumer idempotency on
   `prompt_id` is the correct resolution. This invariant is symmetric with the no-op
   behavior already required for `PermissionPromptResolved` (if `prompt_id` absent, no-op).
```

**Add new Canonical Test Vector** (in the test vectors table):
```
| TUI connects while daemon concurrently queues a new prompt (race: prompt appears in both overlay_stack and streaming PermissionPromptQueued) | TUI overlay stack contains the prompt exactly once; second delivery silently discarded | race/idempotency |
```

**Version:** `1.0.4` → `1.0.5`

**Trace entry:** document as F-S022-ADV6-MED-001 resolution.

### 2. SS-ipc.md v1.8.0 — at-least-once semantics documented

**File:** `.factory/specs/architecture/SS-ipc.md`

**Add to §Risk Mitigations** (after the existing "TUI Reconnects During a Pending Decision" entry):

```markdown
### PermissionPromptQueued Delivered Twice Across Snapshot Window

Risk: A `PermissionPromptQueued` broadcast can arrive in a new client's mpsc channel
between `register_subscriber` and `snapshot_initial_state`. The snapshot includes the
prompt in `InitialState.overlay_stack`; the streaming send loop also delivers the queued
`PermissionPromptQueued` message. The TUI receives the same prompt twice.

Mitigation: The IPC layer intentionally provides **at-least-once delivery** for
`PermissionPromptQueued` across the connection snapshot window. The register-before-snapshot
ordering is preserved to guarantee no-gap delivery (BC-2.05.002 Invariant 3). The consumer
(TUI `VecDeque<PromptModal>`) is required to apply `PermissionPromptQueued` with
idempotent-on-`prompt_id` semantics: if the `prompt_id` is already present in the overlay
stack, the second delivery is silently discarded (BC-2.05.002 Invariant 4).

This design mirrors the existing no-op requirement for `PermissionPromptResolved`: if a
`prompt_id` is absent from the VecDeque, removal is a no-op. Insert and remove are both
`prompt_id`-idempotent by design. No protocol epoch fields, no daemon-side per-client dedup
logic, and no global snapshot lock are required. Decision rationale: F-S022-ADV6-MED-001,
`cycles/cycle-001/S-022/adversarial/architect-decisions-pass-6.md`.
```

**Version:** `1.7.0` → `1.8.0`

**Trace entry:** document as F-S022-ADV6-MED-001 resolution.

---

## Files Produced / Modified

| File | Action | Version Change |
|------|--------|---------------|
| `.factory/cycles/cycle-001/S-022/adversarial/architect-decisions-pass-6.md` | Created | N/A |
| `.factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md` | Updated | 1.0.4 → 1.0.5 |
| `.factory/specs/architecture/SS-ipc.md` | Updated | 1.7.0 → 1.8.0 |
